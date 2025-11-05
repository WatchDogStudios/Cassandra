use crate::{
    auth::{hs256_generate, hs256_validate},
    grpc::InMemoryAgentControl,
    http::{
        health, list_agents, metrics as metrics_route, version, ApiDoc, ContentMetadataResponse,
        UploadSessionResponse,
    },
    metrics::{self, MetricsLayer},
    state::AppState,
};
use axum::{
    body::to_bytes,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use chrono::Utc;
use cncore::platform::models::{Project, Tenant, TenantSettings};
use cncore::platform::persistence::{ContentStore, InMemoryPersistence, ProjectStore, TenantStore};
use cnproto::{agent_control_client::AgentControlClient, HeartbeatRequest, RegisterAgentRequest};
use once_cell::sync::Lazy;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Server};
use tower::ServiceExt;
use utoipa::OpenApi;

static ENV_GUARD: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[tokio::test]
async fn health_ok() {
    cncore::init_tracing();
    let app = Router::new().route("/health", get(health));
    let res = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 16 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "ok");
}

#[tokio::test]
async fn version_endpoint_has_build_info() {
    cncore::init_tracing();
    let app = Router::new().route("/version", get(version));
    let res = app
        .oneshot(
            Request::builder()
                .uri("/version")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 16 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["service"].as_str().unwrap(),
        cncore::config().service_name.as_str()
    );
    for key in ["version", "git_sha", "build_ts"] {
        assert!(v[key].as_str().is_some(), "missing field {:?}: {}", key, v);
        assert!(
            !v[key].as_str().unwrap().is_empty(),
            "empty field {:?}: {}",
            key,
            v
        );
    }
}

#[tokio::test]
async fn metrics_exists() {
    cncore::init_tracing();
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics_route))
        .layer(MetricsLayer);
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        text.contains("gateway_http_requests_total"),
        "metrics output missing custom counter: {}",
        text
    );
    assert!(
        text.contains("gateway_build_info"),
        "missing build info gauge"
    );
}

#[tokio::test]
async fn openapi_has_security_schemes() {
    cncore::init_tracing();
    let mut openapi = ApiDoc::openapi();
    {
        use utoipa::openapi::security::{
            ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme,
        };
        let mut comps = openapi.components.unwrap_or_default();
        comps.add_security_scheme(
            "ApiKey",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
        );
        comps.add_security_scheme(
            "BearerAuth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
        openapi.components = Some(comps);
    }
    let app = Router::new()
        .merge(utoipa_swagger_ui::SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi));
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api-docs/openapi.json")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), 128 * 1024)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let comps = &v["components"]["securitySchemes"];
    assert!(
        comps.get("ApiKey").is_some(),
        "ApiKey scheme missing: {}",
        v
    );
    assert!(
        comps.get("BearerAuth").is_some(),
        "BearerAuth scheme missing: {}",
        v
    );
}

#[tokio::test]
async fn agents_list_after_grpc_heartbeat() {
    cncore::init_tracing();
    let store: Arc<dyn ContentStore> = Arc::new(InMemoryPersistence::new());
    let state = AppState::with_content_store(store);
    let agent_svc = InMemoryAgentControl::new(state.registry.clone()).into_server();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let grpc_addr = listener.local_addr().unwrap();
    let incoming = TcpListenerStream::new(listener);
    let grpc = Server::builder()
        .add_service(agent_svc)
        .serve_with_incoming(incoming);
    tokio::spawn(async move {
        let _ = grpc.await;
    });

    let channel = Channel::from_shared(format!("http://{}", grpc_addr))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut client = AgentControlClient::new(channel);
    let reg = RegisterAgentRequest {
        node_id: "node1".into(),
        hostname: "host1".into(),
        os: "os".into(),
        arch: "arch".into(),
        cpu_cores: 4,
        memory_bytes: 1024,
        secret: "s".into(),
        tenant_id: String::new(),
        project_id: String::new(),
    };
    let _ = client.register_agent(reg).await.unwrap();
    let hb = HeartbeatRequest {
        assigned_id: "node1".into(),
        cpu_percent: 10.0,
        memory_used_bytes: 2048,
        network_rx_bytes: 0,
        network_tx_bytes: 0,
        timestamp_unix_ms: 0,
    };
    let _ = client.heartbeat(hb).await.unwrap();

    let app = Router::new()
        .route("/agents", get(list_agents))
        .with_state(state.clone());
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/agents")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 16 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v.as_array().unwrap().iter().any(|a| a["id"] == "node1"));
}

#[tokio::test]
async fn agents_filtering_by_hostname() {
    cncore::init_tracing();
    let state = AppState::default();
    state.registry.upsert(
        "alpha".into(),
        "host-alpha".into(),
        20.0,
        512,
        None,
        None,
        None,
        None,
    );
    state.registry.upsert(
        "beta".into(),
        "host-beta".into(),
        10.0,
        256,
        None,
        None,
        None,
        None,
    );

    let app = Router::new()
        .route("/agents", get(list_agents))
        .with_state(state.clone());

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/agents?hostname=alpha")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 16 * 1024).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let items = list.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "alpha");
}

#[tokio::test]
async fn ugc_upload_flow_round_trip() {
    cncore::init_tracing();
    let persistence = Arc::new(InMemoryPersistence::new());
    let tenant_id = uuid::Uuid::new_v4();
    let project_id = uuid::Uuid::new_v4();
    persistence
        .insert_tenant(Tenant {
            id: tenant_id,
            name: "tenant".into(),
            created_at: Utc::now(),
            settings: TenantSettings::default(),
        })
        .unwrap();
    persistence
        .insert_project(Project {
            id: project_id,
            tenant_id,
            name: "project".into(),
            created_at: Utc::now(),
        })
        .unwrap();

    let store: Arc<dyn ContentStore> = persistence.clone();
    let state = AppState::with_content_store(store);
    let app = crate::http::router().with_state(state.clone());

    let create_body = json!({
        "filename": "avatar.png",
        "mime_type": "image/png",
        "size_bytes": 1024,
        "labels": ["avatar", "profile"],
        "attributes": {"resolution": "512x512"},
        "visibility": "tenant"
    });
    let create_req = axum::http::Request::builder()
        .method("POST")
        .uri(format!(
            "/tenants/{}/projects/{}/uploads",
            tenant_id, project_id
        ))
        .header("content-type", "application/json")
        .header("x-api-key", "test-key")
        .body(axum::body::Body::from(create_body.to_string()))
        .unwrap();
    let create_res = app
        .clone()
        .oneshot(create_req)
        .await
        .expect("create upload response");
    let create_status = create_res.status();
    let create_bytes = axum::body::to_bytes(create_res.into_body(), 16 * 1024)
        .await
        .unwrap();
    assert_eq!(
        create_status,
        axum::http::StatusCode::CREATED,
        "create failed: {}",
        String::from_utf8_lossy(&create_bytes)
    );
    let session: UploadSessionResponse = serde_json::from_slice(&create_bytes).unwrap();

    let complete_body = json!({
        "filename": "avatar.png",
        "mime_type": "image/png",
        "size_bytes": 1024,
        "checksum": "abc123",
        "labels": ["avatar", "profile"],
        "attributes": {"resolution": "512x512"},
        "visibility": "tenant"
    });
    let complete_req = axum::http::Request::builder()
        .method("POST")
        .uri(format!(
            "/tenants/{}/projects/{}/uploads/{}/complete",
            tenant_id, project_id, session.upload_id
        ))
        .header("content-type", "application/json")
        .header("x-api-key", "test-key")
        .body(axum::body::Body::from(complete_body.to_string()))
        .unwrap();
    let complete_res = app
        .clone()
        .oneshot(complete_req)
        .await
        .expect("complete upload response");
    let complete_status = complete_res.status();
    let complete_bytes = axum::body::to_bytes(complete_res.into_body(), 16 * 1024)
        .await
        .unwrap();
    assert_eq!(
        complete_status,
        axum::http::StatusCode::OK,
        "complete failed: {}",
        String::from_utf8_lossy(&complete_bytes)
    );
    let metadata: ContentMetadataResponse = serde_json::from_slice(&complete_bytes).unwrap();
    assert_eq!(metadata.filename, "avatar.png");
    assert_eq!(metadata.size_bytes, Some(1024));

    let list_req = axum::http::Request::builder()
        .method("GET")
        .uri(format!(
            "/tenants/{}/projects/{}/content",
            tenant_id, project_id
        ))
        .header("x-api-key", "test-key")
        .body(axum::body::Body::empty())
        .unwrap();
    let list_res = app.oneshot(list_req).await.expect("list content response");
    let list_status = list_res.status();
    let list_bytes = axum::body::to_bytes(list_res.into_body(), 16 * 1024)
        .await
        .unwrap();
    assert_eq!(
        list_status,
        axum::http::StatusCode::OK,
        "list failed: {}",
        String::from_utf8_lossy(&list_bytes)
    );
    let entries: Vec<ContentMetadataResponse> = serde_json::from_slice(&list_bytes).unwrap();
    assert!(entries.iter().any(|m| m.id == metadata.id));
}

#[test]
fn normalize_path_reduces_ids() {
    assert_eq!(metrics::normalize_path("/agents").as_ref(), "/agents");
    assert_eq!(
        metrics::normalize_path("/agents/123").as_ref(),
        "/agents/:id"
    );
    assert_eq!(
        metrics::normalize_path("/agents/550e8400-e29b-41d4-a716-446655440000").as_ref(),
        "/agents/:id"
    );
}

#[test]
fn hs256_roundtrip() {
    let _guard = ENV_GUARD.lock().unwrap();
    std::env::set_var("CASS_JWT_SECRET", "test-secret");
    let token = hs256_generate("demo").unwrap();
    assert!(hs256_validate(&token).unwrap());
    std::env::set_var("CASS_JWT_SECRET", "other-secret");
    assert!(!hs256_validate(&token).unwrap());
    std::env::remove_var("CASS_JWT_SECRET");
}
