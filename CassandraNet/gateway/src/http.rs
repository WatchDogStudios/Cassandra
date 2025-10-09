use crate::metrics::gather_metrics;
use crate::state::{AgentSummary, AppState};
use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: String,
}

#[derive(Serialize, ToSchema)]
pub struct VersionResponse {
    pub service: String,
    pub version: String,
    pub git_sha: String,
    pub git_tag: String,
    pub build_ts: String,
}

#[utoipa::path(get, path = "/health", tag = "system", responses( (status = 200, description = "Service healthy", body = HealthResponse) ))]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: cncore::config().service_name.clone(),
    })
}

#[utoipa::path(get, path = "/version", tag = "system", responses( (status = 200, description = "Version info", body = VersionResponse) ))]
pub async fn version() -> Json<VersionResponse> {
    let info = cncore::build_info();
    Json(VersionResponse {
        service: cncore::config().service_name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_sha: info.git_sha.to_string(),
        git_tag: info.git_tag.to_string(),
        build_ts: info.build_timestamp.to_string(),
    })
}

#[utoipa::path(get, path = "/metrics", tag = "system")]
pub async fn metrics() -> (axum::http::StatusCode, String) {
    gather_metrics()
}

#[utoipa::path(get, path = "/agents", tag = "system", responses( (status = 200, body = [AgentSummary]) ))]
pub async fn list_agents(State(state): State<AppState>) -> Json<Vec<AgentSummary>> {
    Json(state.registry.list())
}

#[derive(OpenApi)]
#[openapi(
    paths(health, version, metrics, list_agents),
    components(schemas(HealthResponse, VersionResponse, AgentSummary)),
    tags( (name = "system", description = "System & meta endpoints") )
)]
pub struct ApiDoc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/metrics", get(metrics))
        .route("/agents", get(list_agents))
}
