use crate::auth::has_scope;
use crate::metrics::gather_metrics;
use crate::state::{AgentSummary, AppState};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use cncommon::observability::{LogEvent, LogLevel};
use cncore::platform::{
    ContentId, ContentMetadata, ContentQuery, ContentVisibility, PlatformError, UploadId,
    UploadSession, UploadStatus,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration as StdDuration;
use utoipa::{IntoParams, OpenApi, ToSchema};
use uuid::Uuid;

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

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: &'static str,
}

pub struct HttpError {
    status: StatusCode,
    message: &'static str,
}

impl HttpError {
    pub fn new(status: StatusCode, message: &'static str) -> Self {
        Self { status, message }
    }
}

impl From<PlatformError> for HttpError {
    fn from(value: PlatformError) -> Self {
        match value {
            PlatformError::NotFound(_) => HttpError::new(StatusCode::NOT_FOUND, "not found"),
            PlatformError::Conflict(_) => HttpError::new(StatusCode::CONFLICT, "conflict"),
            PlatformError::Unauthorized =>
                HttpError::new(StatusCode::UNAUTHORIZED, "unauthorized"),
            PlatformError::Forbidden => HttpError::new(StatusCode::FORBIDDEN, "forbidden"),
            PlatformError::InvalidInput(_) =>
                HttpError::new(StatusCode::BAD_REQUEST, "invalid input"),
            PlatformError::Internal(_) =>
                HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal error"),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorResponse { error: self.message })).into_response()
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUploadRequest {
    pub filename: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(default = "default_visibility")]
    pub visibility: ContentVisibility,
    #[serde(default)]
    pub expires_in_seconds: Option<i64>,
}

fn default_visibility() -> ContentVisibility {
    ContentVisibility::Project
}

#[derive(Debug, Deserialize, ToSchema, Default, IntoParams)]
pub struct ListContentParams {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

#[derive(Debug, Deserialize, ToSchema, Default, IntoParams)]
pub struct ListLogsParams {
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteUploadRequest {
    pub filename: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    pub size_bytes: u64,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub storage_path: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(default)]
    pub uploaded_by: Option<Uuid>,
    #[serde(default = "default_visibility")]
    pub visibility: ContentVisibility,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UploadSessionResponse {
    pub upload_id: UploadId,
    pub content_id: ContentId,
    pub status: String,
    pub upload_url: Option<String>,
    pub storage_path: String,
    pub headers: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ContentMetadataResponse {
    pub id: ContentId,
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub filename: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub checksum: Option<String>,
    pub storage_path: Option<String>,
    pub labels: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub uploaded_by: Option<Uuid>,
    pub visibility: ContentVisibility,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TelemetryLogResponse {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub component: Option<String>,
    pub tenant_id: Option<String>,
    pub project_id: Option<String>,
    pub metadata: serde_json::Value,
}

impl UploadSessionResponse {
    fn from_session(session: &UploadSession, storage_path: String) -> Self {
        Self {
            upload_id: session.id,
            content_id: session.content_id,
            status: session.status.as_str().to_string(),
            upload_url: session.upload_url.clone(),
            storage_path,
            headers: session.headers.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            expires_at: session.expires_at,
        }
    }
}

impl From<ContentMetadata> for ContentMetadataResponse {
    fn from(value: ContentMetadata) -> Self {
        Self {
            id: value.id,
            tenant_id: value.tenant_id,
            project_id: value.project_id,
            filename: value.filename,
            mime_type: value.mime_type,
            size_bytes: value.size_bytes,
            checksum: value.checksum,
            storage_path: value.storage_path,
            labels: value.labels,
            attributes: value.attributes,
            created_at: value.created_at,
            updated_at: value.updated_at,
            uploaded_by: value.uploaded_by,
            visibility: value.visibility,
        }
    }
}

impl From<LogEvent> for TelemetryLogResponse {
    fn from(event: LogEvent) -> Self {
        Self {
            level: event.level,
            message: event.message,
            timestamp: event.timestamp,
            component: event.component,
            tenant_id: event.tenant_id,
            project_id: event.project_id,
            metadata: event.metadata,
        }
    }
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

#[utoipa::path(
    post,
    path = "/tenants/{tenant_id}/projects/{project_id}/uploads",
    params(
        ("tenant_id" = Uuid, Path, description = "Tenant identifier"),
        ("project_id" = Uuid, Path, description = "Project identifier")
    ),
    request_body = CreateUploadRequest,
    responses(
        (status = 201, description = "Upload session created", body = UploadSessionResponse)
    ),
    security(("ApiKey" = []), ("BearerAuth" = []))
)]
pub async fn create_upload_session(
    State(state): State<AppState>,
    Path((tenant_id, project_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Json(payload): Json<CreateUploadRequest>,
) -> Result<(StatusCode, Json<UploadSessionResponse>), HttpError> {
    if payload.filename.trim().is_empty() {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "filename required"));
    }
    ensure_scope(&headers, "ugc:write")?;
    if !state
        .rate_limiter
        .check_and_increment(
            tenant_id,
            "ugc:create_upload",
            60,
            StdDuration::from_secs(60),
        )
    {
        return Err(HttpError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded",
        ));
    }
    let now = Utc::now();
    let content_id = Uuid::new_v4();
    let upload_id = Uuid::new_v4();
    let storage_path = build_storage_path(&tenant_id, &project_id, &content_id, &payload.filename);
    let upload_url = storage_base_url()
        .map(|base| format!("{base}/{storage_path}"))
        .or_else(|| Some(format!("s3://cassandranet/{storage_path}")));
    let expires_at = payload
        .expires_in_seconds
        .filter(|secs| *secs > 0)
        .map(|secs| now + Duration::seconds(secs));
    let mut headers = HashMap::new();
    if let Some(mime) = &payload.mime_type {
        headers.insert("content-type".to_string(), mime.clone());
    }
    if let Some(size) = payload.size_bytes {
        headers.insert("content-length".to_string(), size.to_string());
    }
    let session = UploadSession {
        id: upload_id,
        tenant_id,
        project_id,
        content_id,
        status: UploadStatus::Pending,
        created_at: now,
        updated_at: now,
        expires_at,
        upload_url,
        headers,
    };
    state
        .content_store
        .create_upload_session(session.clone())
        .await
        .map_err(HttpError::from)?;
    let mut metric_labels = HashMap::new();
    metric_labels.insert("tenant_id".to_string(), tenant_id.to_string());
    metric_labels.insert("project_id".to_string(), project_id.to_string());
    state
        .telemetry
        .metrics
        .increment_counter("ugc_upload_sessions_created", 1.0, Some(metric_labels.clone()));
    if let Some(size) = payload.size_bytes {
        state
            .telemetry
            .metrics
            .observe_histogram("ugc_upload_size_bytes", size as f64, Some(metric_labels.clone()));
    }
    state.telemetry.logs.emit(
        LogEvent::new(LogLevel::Info, "ugc.upload_session.created")
            .with_component("gateway")
            .with_tenant(tenant_id.to_string())
            .with_project(project_id.to_string())
            .with_metadata(json!({
                "upload_id": upload_id,
                "content_id": content_id,
                "filename": payload.filename,
                "mime_type": payload.mime_type,
                "storage_path": storage_path
            })),
    );
    let response = UploadSessionResponse::from_session(&session, storage_path);
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    post,
    path = "/tenants/{tenant_id}/projects/{project_id}/uploads/{upload_id}/complete",
    params(
        ("tenant_id" = Uuid, Path, description = "Tenant identifier"),
        ("project_id" = Uuid, Path, description = "Project identifier"),
        ("upload_id" = Uuid, Path, description = "Upload session id")
    ),
    request_body = CompleteUploadRequest,
    responses(
        (status = 200, description = "Upload finalized", body = ContentMetadataResponse)
    ),
    security(("ApiKey" = []), ("BearerAuth" = []))
)]
pub async fn complete_upload_session(
    State(state): State<AppState>,
    Path((tenant_id, project_id, upload_id)): Path<(Uuid, Uuid, Uuid)>,
    headers: HeaderMap,
    Json(payload): Json<CompleteUploadRequest>,
) -> Result<Json<ContentMetadataResponse>, HttpError> {
    ensure_scope(&headers, "ugc:write")?;
    if !state
        .rate_limiter
        .check_and_increment(
            tenant_id,
            "ugc:complete_upload",
            120,
            StdDuration::from_secs(60),
        )
    {
        return Err(HttpError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded",
        ));
    }
    let mut session = state
        .content_store
        .get_upload_session(upload_id)
        .await
        .map_err(HttpError::from)?
        .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "upload session not found"))?;
    if session.tenant_id != tenant_id || session.project_id != project_id {
        return Err(HttpError::new(StatusCode::FORBIDDEN, "upload session scope mismatch"));
    }
    let now = Utc::now();
    if let Some(expires_at) = session.expires_at {
        if expires_at < now {
            return Err(HttpError::new(StatusCode::GONE, "upload session expired"));
        }
    }
    if !matches!(session.status, UploadStatus::Pending | UploadStatus::Uploading) {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "upload session closed"));
    }
    let storage_path = payload
        .storage_path
        .clone()
        .unwrap_or_else(|| build_storage_path(&tenant_id, &project_id, &session.content_id, &payload.filename));
    session.status = UploadStatus::Completed;
    session.updated_at = now;
    session.upload_url = storage_base_url()
        .map(|base| format!("{base}/{storage_path}"))
        .or_else(|| Some(storage_path.clone()));
    state
        .content_store
        .update_upload_session(session.clone())
        .await
        .map_err(HttpError::from)?;

    let metadata = ContentMetadata {
        id: session.content_id,
        tenant_id,
        project_id,
        filename: payload.filename,
        mime_type: payload.mime_type,
        size_bytes: Some(payload.size_bytes),
        checksum: payload.checksum,
        storage_path: Some(storage_path.clone()),
        labels: payload.labels,
        attributes: payload.attributes,
        created_at: now,
        updated_at: now,
        uploaded_by: payload.uploaded_by,
        visibility: payload.visibility,
    };
    state
        .content_store
        .record_content_metadata(metadata.clone())
        .await
        .map_err(HttpError::from)?;
    let mut metric_labels = HashMap::new();
    metric_labels.insert("tenant_id".to_string(), tenant_id.to_string());
    metric_labels.insert("project_id".to_string(), project_id.to_string());
    state
        .telemetry
        .metrics
        .increment_counter("ugc_uploads_completed", 1.0, Some(metric_labels.clone()));
    state
        .telemetry
        .metrics
        .set_gauge("ugc_last_upload_size_bytes", payload.size_bytes as f64, Some(metric_labels.clone()));
    let metadata_response = ContentMetadataResponse::from(metadata.clone());
    state.telemetry.logs.emit(
        LogEvent::new(LogLevel::Info, "ugc.upload_session.completed")
            .with_component("gateway")
            .with_tenant(tenant_id.to_string())
            .with_project(project_id.to_string())
            .with_metadata(json!({
                "upload_id": upload_id,
                "content_id": session.content_id,
                "filename": metadata_response.filename,
                "size_bytes": metadata_response.size_bytes,
                "storage_path": metadata_response.storage_path
            })),
    );
    Ok(Json(metadata_response))
}

#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/projects/{project_id}/content",
    params(
        ("tenant_id" = Uuid, Path, description = "Tenant identifier"),
        ("project_id" = Uuid, Path, description = "Project identifier"),
        ListContentParams
    ),
    responses(
        (status = 200, description = "Content metadata list", body = [ContentMetadataResponse])
    ),
    security(("ApiKey" = []), ("BearerAuth" = []))
)]
pub async fn list_content_metadata(
    State(state): State<AppState>,
    Path((tenant_id, project_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Query(params): Query<ListContentParams>,
) -> Result<Json<Vec<ContentMetadataResponse>>, HttpError> {
    ensure_scope(&headers, "ugc:read")?;
    if !state
        .rate_limiter
        .check_and_increment(
            tenant_id,
            "ugc:list_content",
            120,
            StdDuration::from_secs(60),
        )
    {
        return Err(HttpError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded",
        ));
    }
    let query = ContentQuery {
        tenant_id,
        project_id: Some(project_id),
        search_term: params.search,
        tags: params.tags,
        limit: params.limit,
        offset: params.offset,
    };
    let items = state
        .content_store
        .list_content_metadata(&query)
        .await
        .map_err(HttpError::from)?;
    let mut metric_labels = HashMap::new();
    metric_labels.insert("tenant_id".to_string(), tenant_id.to_string());
    metric_labels.insert("project_id".to_string(), project_id.to_string());
    state
        .telemetry
        .metrics
        .set_gauge(
            "ugc_content_listing_size",
            items.len() as f64,
            Some(metric_labels.clone()),
        );
    state.telemetry.logs.emit(
        LogEvent::new(LogLevel::Debug, "ugc.content.listed")
            .with_component("gateway")
            .with_tenant(tenant_id.to_string())
            .with_project(project_id.to_string())
            .with_metadata(json!({
                "count": items.len(),
                "search_term": query.search_term,
                "tags": query.tags,
                "limit": query.limit,
                "offset": query.offset
            })),
    );
    Ok(Json(
        items
            .into_iter()
            .map(ContentMetadataResponse::from)
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/telemetry/logs",
    params(ListLogsParams),
    responses(
        (status = 200, description = "Recent structured log events", body = [TelemetryLogResponse])
    ),
    security(("ApiKey" = []), ("BearerAuth" = []))
)]
pub async fn list_recent_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListLogsParams>,
) -> Result<Json<Vec<TelemetryLogResponse>>, HttpError> {
    ensure_scope(&headers, "observability:read")?;
    if !state
        .rate_limiter
        .check_and_increment(
            Uuid::nil(),
            "observability:list_logs",
            30,
            StdDuration::from_secs(60),
        )
    {
        return Err(HttpError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded",
        ));
    }
    let limit = params.limit.unwrap_or(100).min(500);
    let events = state.telemetry.log_sink.snapshot();
    let start = events.len().saturating_sub(limit);
    let slice = events.into_iter().skip(start).collect::<Vec<_>>();
    state
        .telemetry
        .metrics
        .set_gauge("gateway_log_buffer_size", slice.len() as f64, None);
    Ok(Json(
        slice
            .into_iter()
            .map(TelemetryLogResponse::from)
            .collect(),
    ))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        version,
        metrics,
        list_agents,
        create_upload_session,
        complete_upload_session,
        list_content_metadata,
        list_recent_logs
    ),
    components(
        schemas(
            HealthResponse,
            VersionResponse,
            AgentSummary,
            ErrorResponse,
            CreateUploadRequest,
            CompleteUploadRequest,
            ListContentParams,
            ListLogsParams,
            UploadSessionResponse,
            ContentMetadataResponse,
            TelemetryLogResponse
        )
    ),
    tags( (name = "system", description = "System & meta endpoints") )
)]
pub struct ApiDoc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/metrics", get(metrics))
        .route("/agents", get(list_agents))
        .route(
            "/tenants/:tenant_id/projects/:project_id/uploads",
            post(create_upload_session),
        )
        .route(
            "/tenants/:tenant_id/projects/:project_id/uploads/:upload_id/complete",
            post(complete_upload_session),
        )
        .route(
            "/tenants/:tenant_id/projects/:project_id/content",
            get(list_content_metadata),
        )
        .route("/telemetry/logs", get(list_recent_logs))
}

fn ensure_scope(headers: &HeaderMap, scope: &str) -> Result<(), HttpError> {
    if has_scope(headers, scope) {
        Ok(())
    } else {
        Err(HttpError::new(StatusCode::FORBIDDEN, "scope required"))
    }
}

fn build_storage_path(
    tenant_id: &Uuid,
    project_id: &Uuid,
    content_id: &Uuid,
    filename: &str,
) -> String {
    format!(
        "tenants/{tenant_id}/projects/{project_id}/{content_id}/{filename}",
        tenant_id = tenant_id,
        project_id = project_id,
        content_id = content_id,
        filename = filename.trim()
    )
}

fn storage_base_url() -> Option<String> {
    match std::env::var("CASS_STORAGE_BASE_URL") {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}
