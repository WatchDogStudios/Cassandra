use crate::state::audit::{AuditSink, NoopAudit};
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
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Server};
use tower::ServiceExt;
use utoipa::OpenApi;

/// Shared by every test: `CASS_API_KEYS` is parsed once per process.
const TEST_API_KEYS: &str = "test-key:agent:read|agent:write|agent:execute|ugc:write|ugc:read|\
build:read|build:write|build:deploy|script:read|script:write|script:execute";

static ENV_GUARD: Lazy<tokio::sync::Mutex<()>> = Lazy::new(|| tokio::sync::Mutex::new(()));

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
        .merge(utoipa_swagger_ui::SwaggerUi::new("/docs").url("/api-doc/openapi.json", openapi));
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api-doc/openapi.json")
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
    const NODE: &str = "6f1c2c0e-2f61-4c55-9a4b-0b5b7f0f9a11";
    cncore::init_tracing();
    let _guard = ENV_GUARD.lock().await;
    std::env::set_var("CASS_API_KEYS", TEST_API_KEYS);
    let store: Arc<dyn ContentStore> = Arc::new(InMemoryPersistence::new());
    let state = AppState::with_content_store(store);
    let audit = AuditSink::new(Arc::new(NoopAudit));
    let agent_svc = InMemoryAgentControl::new(
        state.registry.clone(),
        None,
        audit,
        state.portal_store.clone(),
        state.build_dir.clone(),
    )
    .into_server();
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
        node_id: NODE.into(),
        hostname: "host1".into(),
        os: "os".into(),
        arch: "arch".into(),
        cpu_cores: 4,
        memory_bytes: 1024,
        secret: "s".into(),
        tenant_id: String::new(),
        project_id: String::new(),
    };
    // Unauthenticated agents are turned away before touching the registry.
    let denied = client
        .register_agent(reg.clone())
        .await
        .expect_err("register without credentials");
    assert_eq!(denied.code(), tonic::Code::Unauthenticated);
    // Non-UUID ids would register yet never be listed by the Postgres store.
    let mut bad_id = tonic::Request::new(RegisterAgentRequest {
        node_id: "node1".into(),
        ..reg.clone()
    });
    bad_id
        .metadata_mut()
        .insert("x-api-key", "test-key".parse().unwrap());
    let rejected = client
        .register_agent(bad_id)
        .await
        .expect_err("non-UUID node id");
    assert_eq!(rejected.code(), tonic::Code::InvalidArgument);
    let mut authed = tonic::Request::new(reg);
    authed
        .metadata_mut()
        .insert("x-api-key", "test-key".parse().unwrap());
    let _ = client.register_agent(authed).await.unwrap();
    let hb = HeartbeatRequest {
        assigned_id: NODE.into(),
        cpu_percent: 10.0,
        memory_used_bytes: 2048,
        network_rx_bytes: 0,
        network_tx_bytes: 0,
        timestamp_unix_ms: 0,
    };
    let mut hb = tonic::Request::new(hb);
    hb.metadata_mut()
        .insert("x-api-key", "test-key".parse().unwrap());
    let _ = client.heartbeat(hb).await.unwrap();

    let app = Router::new()
        .route("/agents", get(list_agents))
        .with_state(state.clone());
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/agents")
                .header("x-api-key", "test-key")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 16 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v.as_array().unwrap().iter().any(|a| a["id"] == NODE));
}

#[tokio::test]
async fn agents_filtering_by_hostname() {
    cncore::init_tracing();
    let _guard = ENV_GUARD.lock().await;
    std::env::set_var("CASS_API_KEYS", TEST_API_KEYS);
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
                .header("x-api-key", "test-key")
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
    let _guard = ENV_GUARD.lock().await;
    std::env::set_var("CASS_API_KEYS", TEST_API_KEYS);
    let persistence = Arc::new(InMemoryPersistence::new());
    let now = Utc::now();
    let tenant_id = uuid::Uuid::new_v4();
    let project_id = uuid::Uuid::new_v4();
    persistence
        .insert_tenant(Tenant {
            id: tenant_id,
            name: "tenant".into(),
            settings: TenantSettings::default(),
            slug: "tenant".into(),
            plan: "free".into(),
            description: None,
            quota_agents: 100,
            quota_storage_bytes: 10737418240,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
        .unwrap();
    persistence
        .insert_project(Project {
            id: project_id,
            tenant_id,
            name: "project".into(),
            slug: "project".into(),
            description: None,
            settings: serde_json::Value::Null,
            is_active: true,
            created_at: now,
            updated_at: now,
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
    let _guard = ENV_GUARD.blocking_lock();
    std::env::set_var("CASS_JWT_SECRET", "test-secret");
    let token = hs256_generate("demo").unwrap();
    assert!(hs256_validate(&token).unwrap());
    std::env::set_var("CASS_JWT_SECRET", "other-secret");
    assert!(!hs256_validate(&token).unwrap());
    std::env::remove_var("CASS_JWT_SECRET");
    // No secret configured means nothing validates: there is no fallback key.
    assert!(!hs256_validate(&token).unwrap());
}

#[test]
fn jwt_claims_must_be_unexpired_access_tokens() {
    use crate::auth::claims_usable_for_api as usable;
    let now = 1_700_000_000;
    assert!(usable(&json!({"exp": now + 60}), now));
    assert!(usable(
        &json!({"exp": now + 60, "token_use": "access"}),
        now
    ));
    assert!(!usable(&json!({"exp": now}), now), "expiry is exclusive");
    assert!(!usable(&json!({"exp": now - 1}), now), "expired");
    assert!(!usable(&json!({}), now), "exp is mandatory");
    assert!(!usable(&json!({"exp": "soon"}), now), "exp must be numeric");
    assert!(
        !usable(&json!({"exp": now + 60, "token_use": "refresh"}), now),
        "refresh tokens never authorize API calls"
    );
}

fn authed<T>(message: T) -> tonic::Request<T> {
    let mut request = tonic::Request::new(message);
    request
        .metadata_mut()
        .insert("x-api-key", "test-key".parse().unwrap());
    request
}

async fn call(app: &Router, request: Request<axum::body::Body>) -> (StatusCode, serde_json::Value) {
    let res = app.clone().oneshot(request).await.unwrap();
    let status = res.status();
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let value = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
    (status, value)
}

fn json_request(method: &str, uri: &str, body: serde_json::Value) -> Request<axum::body::Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-api-key", "test-key")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}

/// Operator actions become agent jobs: queued over HTTP, handed out on the
/// agent's heartbeat, reported back over gRPC, visible again over HTTP. Builds
/// uploaded as multipart stream back to the agent byte-for-byte.
#[tokio::test]
async fn portal_jobs_and_builds_round_trip() {
    use cnproto::{DownloadBuildRequest, ReportJobResultRequest};
    use sha2::{Digest, Sha256};

    cncore::init_tracing();
    let _guard = ENV_GUARD.lock().await;
    std::env::set_var("CASS_API_KEYS", TEST_API_KEYS);

    let build_dir = std::env::temp_dir().join(format!("cass-builds-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let state = AppState {
        build_dir: Arc::new(build_dir.clone()),
        ..AppState::default()
    };
    state.registry.upsert(
        "node-1".into(),
        "host".into(),
        0.0,
        0,
        None,
        None,
        None,
        None,
    );
    let app = crate::http::router().with_state(state.clone());

    let agent_svc = InMemoryAgentControl::new(
        state.registry.clone(),
        None,
        AuditSink::new(Arc::new(NoopAudit)),
        state.portal_store.clone(),
        state.build_dir.clone(),
    )
    .into_server();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let grpc_addr = listener.local_addr().unwrap();
    tokio::spawn(
        Server::builder()
            .add_service(agent_svc)
            .serve_with_incoming(TcpListenerStream::new(listener)),
    );
    let channel = Channel::from_shared(format!("http://{grpc_addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut client = AgentControlClient::new(channel);
    let heartbeat = || HeartbeatRequest {
        assigned_id: "node-1".into(),
        ..Default::default()
    };

    // Unknown commands and missing arguments are rejected up front.
    let (status, _) = call(
        &app,
        json_request(
            "POST",
            "/agents/node-1/commands",
            json!({"command": "update"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = call(
        &app,
        json_request(
            "POST",
            "/agents/node-1/commands",
            json!({"command": "exec"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = call(
        &app,
        json_request(
            "POST",
            "/agents/ghost/commands",
            json!({"command": "restart"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Queue an exec command.
    let (status, queued) = call(
        &app,
        json_request(
            "POST",
            "/agents/node-1/commands",
            json!({"command": "exec", "args": {"command": "echo hi"}}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{queued}");
    assert_eq!(queued["status"], "pending");
    let job_id = queued["id"].as_str().unwrap().to_string();

    // The next heartbeat hands it out exactly once.
    let resp = client
        .heartbeat(authed(heartbeat()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.jobs.len(), 1);
    assert_eq!(resp.jobs[0].job_id, job_id);
    assert_eq!(resp.jobs[0].kind, "exec");
    let args: serde_json::Value = serde_json::from_str(&resp.jobs[0].args_json).unwrap();
    assert_eq!(args["command"], "echo hi");
    assert_eq!(
        resp.heartbeat_interval_seconds,
        crate::portal_handlers::DEFAULT_HEARTBEAT_INTERVAL_SECS
    );
    let again = client
        .heartbeat(authed(heartbeat()))
        .await
        .unwrap()
        .into_inner();
    assert!(again.jobs.is_empty());

    // Only the agent the job belongs to can report it.
    let report = |agent: &str| ReportJobResultRequest {
        assigned_id: agent.into(),
        job_id: job_id.clone(),
        success: true,
        exit_code: 0,
        has_exit_code: true,
        stdout: "hi\n".into(),
        ..Default::default()
    };
    let stolen = client
        .report_job_result(authed(report("node-2")))
        .await
        .expect_err("foreign agent");
    assert_eq!(stolen.code(), tonic::Code::NotFound);
    client
        .report_job_result(authed(report("node-1")))
        .await
        .unwrap();

    let (status, history) = call(
        &app,
        Request::builder()
            .uri("/agents/node-1/commands")
            .header("x-api-key", "test-key")
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(history[0]["status"], "completed");
    assert_eq!(history[0]["result"]["stdout"], "hi\n");
    assert_eq!(history[0]["result"]["exit_code"], 0);

    // Agent config changes reach the agent through its heartbeat.
    let (status, _) = call(
        &app,
        json_request(
            "PUT",
            "/agents/node-1/config",
            json!({"heartbeat_interval_seconds": 2}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "below the 5s floor");
    let (status, config) = call(
        &app,
        json_request(
            "PUT",
            "/agents/node-1/config",
            json!({"heartbeat_interval_seconds": 15}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        config["log_level"], "info",
        "partial update keeps other fields"
    );
    let resp = client
        .heartbeat(authed(heartbeat()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.heartbeat_interval_seconds, 15);

    // Upload a build as multipart, deploy it, and stream it back to the agent.
    let payload: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
    let boundary = "cassboundary";
    let mut body = Vec::new();
    for (name, value) in [
        ("name", "game-server"),
        ("version", "1.2.3"),
        ("platform", "linux"),
        ("architecture", "x64"),
    ] {
        body.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n").as_bytes(),
        );
    }
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"../server.tar.gz\"\r\n\
             Content-Type: application/octet-stream\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(&payload);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    let (status, build) = call(
        &app,
        Request::builder()
            .method("POST")
            .uri("/builds")
            .header("x-api-key", "test-key")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(axum::body::Body::from(body))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{build}");
    let expected_sha = hex::encode(Sha256::digest(&payload));
    assert_eq!(build["checksum_sha256"], expected_sha.as_str());
    assert_eq!(build["size_bytes"], payload.len() as u64);
    assert_eq!(
        build["filename"], "server.tar.gz",
        "directory parts stripped"
    );
    let build_id = build["id"].as_str().unwrap().to_string();

    let (status, deployment) = call(
        &app,
        json_request(
            "POST",
            "/deployments",
            json!({"build_id": build_id, "agent_ids": ["node-1"]}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{deployment}");
    let resp = client
        .heartbeat(authed(heartbeat()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.jobs.len(), 1);
    assert_eq!(resp.jobs[0].kind, "deploy_build");

    let mut stream = client
        .download_build(authed(DownloadBuildRequest {
            build_id: build_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    let meta = stream.message().await.unwrap().unwrap();
    assert_eq!(meta.checksum_sha256, expected_sha);
    assert_eq!(meta.filename, "server.tar.gz");
    let mut downloaded = Vec::new();
    while let Some(chunk) = stream.message().await.unwrap() {
        downloaded.extend_from_slice(&chunk.data);
    }
    assert_eq!(downloaded, payload);

    // Scripts: create, run on an agent, see the execution.
    let (status, script) = call(
        &app,
        json_request(
            "POST",
            "/scripts",
            json!({"name": "uptime", "language": "bash", "content": "uptime"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{script}");
    let script_id = script["id"].as_str().unwrap().to_string();
    let (status, runs) = call(
        &app,
        json_request(
            "POST",
            &format!("/scripts/{script_id}/execute"),
            json!({"agent_ids": ["node-1"]}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{runs}");
    let resp = client
        .heartbeat(authed(heartbeat()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.jobs.len(), 1);
    assert_eq!(resp.jobs[0].kind, "script");
    let (status, executions) = call(
        &app,
        Request::builder()
            .uri(format!("/scripts/{script_id}/executions"))
            .header("x-api-key", "test-key")
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(executions[0]["status"], "dispatched");

    // Deleting the build removes its file.
    let (status, _) = call(
        &app,
        Request::builder()
            .method("DELETE")
            .uri(format!("/builds/{build_id}"))
            .header("x-api-key", "test-key")
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(!build_dir.join(&build_id).exists());
    let _ = std::fs::remove_dir_all(&build_dir);
}
