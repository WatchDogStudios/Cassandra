use crate::{
    auth::{hs256_generate, hs256_validate},
    grpc::InMemoryAgentControl,
    http::{health, list_agents, metrics as metrics_route, version, ApiDoc},
    metrics::{self, MetricsLayer},
    state::{AgentRegistry, AppState},
};
use axum::{
    body::to_bytes,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use cnproto::{agent_control_client::AgentControlClient, HeartbeatRequest, RegisterAgentRequest};
use once_cell::sync::Lazy;
use std::sync::Mutex;
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
    let state = AppState {
        registry: AgentRegistry::default(),
    };
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
