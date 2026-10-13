use crate::auth::has_scope;
#[cfg(feature = "db")]
use crate::auth::validate_jwt;
use crate::messaging_handlers::{ack_message, list_messages, publish_message};
use crate::metrics::gather_metrics;
use crate::portal_handlers::*;
use crate::state::audit::AuditEvent;
use crate::state::{AgentSummary, AppState};
use crate::ugc_handlers::{create_content, list_content, update_content_state};
use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use cncommon::auth::Scope;
use cncommon::observability::{LogEvent, LogLevel};
#[cfg(feature = "db")]
use cncore::platform::persistence::{AgentQuery, AgentSummaryRecord};
use cncore::platform::{
    ContentId, ContentMetadata, ContentQuery, ContentVisibility, PlatformError, UploadId,
    UploadSession, UploadStatus,
};
#[cfg(feature = "db")]
use cncore::platform::{Project, Task, TaskStatus, Tenant, Workflow};
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
            PlatformError::Unauthorized => HttpError::new(StatusCode::UNAUTHORIZED, "unauthorized"),
            PlatformError::Forbidden => HttpError::new(StatusCode::FORBIDDEN, "forbidden"),
            PlatformError::InvalidInput(_) => {
                HttpError::new(StatusCode::BAD_REQUEST, "invalid input")
            }
            PlatformError::Internal(_) => {
                HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
            }
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
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

// === Tenant/Project/Task/Workflow Request/Response Types ===

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    pub name: String,
    /// Optional explicit slug; if omitted, derived from name.
    #[serde(default)]
    pub slug: Option<String>,
    /// Optional plan identifier; defaults to "free".
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub quota_agents: Option<i32>,
    #[serde(default)]
    pub quota_storage_bytes: Option<i64>,
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTenantRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub quota_agents: Option<i32>,
    #[serde(default)]
    pub quota_storage_bytes: Option<i64>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProjectRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    pub kind: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema)]
pub struct LeaseTaskRequest {
    pub worker_id: Uuid,
    #[serde(default = "default_lease_seconds")]
    pub lease_seconds: u32,
}

#[cfg(feature = "db")]
fn default_lease_seconds() -> u32 {
    300
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteTaskRequest {
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub steps: Vec<serde_json::Value>,
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema)]
pub struct StartWorkflowRunRequest {
    #[serde(default)]
    pub context: Option<serde_json::Value>,
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema, Default, IntoParams)]
pub struct ListTenantsParams {
    #[serde(default)]
    pub active_only: Option<bool>,
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema, Default, IntoParams)]
pub struct ListProjectsParams {
    #[serde(default)]
    pub active_only: Option<bool>,
}

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema, Default, IntoParams)]
pub struct ListTasksParams {
    #[serde(default)]
    pub limit: Option<u32>,
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

#[derive(Debug, Deserialize, ToSchema, Default, IntoParams)]
pub struct ListAgentsParams {
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub lifecycle_status: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub last_seen_after: Option<i64>,
    #[serde(default)]
    pub last_seen_before: Option<i64>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/agents",
    tag = "system",
    params(ListAgentsParams),
    responses( (status = 200, body = [AgentSummary]) )
)]
pub async fn list_agents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListAgentsParams>,
) -> Result<Json<Vec<AgentSummary>>, HttpError> {
    ensure_scope(&headers, "agent:read")?;
    state.audit.emit(AuditEvent::info(
        "agents.list",
        json!({
            "hostname": params.hostname,
            "status": params.status,
            "tenant": params.tenant_id,
            "project": params.project_id,
        }),
    ));
    #[cfg(feature = "db")]
    {
        if let Some(store) = state.agent_store.as_ref() {
            match build_agent_query(&params) {
                Ok(query) => match store.query_agents(&query).await {
                    Ok(records) => {
                        let mapped: Vec<AgentSummary> =
                            records.into_iter().map(map_agent_record).collect();
                        return Ok(Json(filter_agent_summaries(mapped, &params)));
                    }
                    Err(err) => {
                        tracing::error!(error = %err, "agents.query_failed_fallback");
                    }
                },
                Err(err) => return Err(err),
            }
        }
    }

    if let Some(redis) = &state.redis_store {
        match redis.list_active_agents().await {
            Ok(agents) => {
                return Ok(Json(filter_agent_summaries(agents, &params)));
            }
            Err(e) => {
                tracing::error!("Failed to list agents from Redis: {}", e);
            }
        }
    }

    let agents = state.registry.list();
    Ok(Json(filter_agent_summaries(agents, &params)))
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
    if !state.rate_limiter.check_and_increment(
        tenant_id,
        "ugc:create_upload",
        60,
        StdDuration::from_secs(60),
    ) {
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
    let labels_count = payload.labels.len();
    let filename_for_events = payload.filename.clone();
    let mime_type_for_events = payload.mime_type.clone();
    let size_bytes_for_events = payload.size_bytes;
    let storage_path_for_log = storage_path.clone();
    let storage_path_for_audit = storage_path.clone();
    state
        .content_store
        .create_upload_session(session.clone())
        .await
        .map_err(HttpError::from)?;
    let mut metric_labels = HashMap::new();
    metric_labels.insert("tenant_id".to_string(), tenant_id.to_string());
    metric_labels.insert("project_id".to_string(), project_id.to_string());
    state.telemetry.metrics.increment_counter(
        "ugc_upload_sessions_created",
        1.0,
        Some(metric_labels.clone()),
    );
    if let Some(size) = payload.size_bytes {
        state.telemetry.metrics.observe_histogram(
            "ugc_upload_size_bytes",
            size as f64,
            Some(metric_labels.clone()),
        );
    }
    state.telemetry.logs.emit(
        LogEvent::new(LogLevel::Info, "ugc.upload_session.created")
            .with_component("gateway")
            .with_tenant(tenant_id.to_string())
            .with_project(project_id.to_string())
            .with_metadata(json!({
                "upload_id": upload_id,
                "content_id": content_id,
                "filename": filename_for_events.clone(),
                "mime_type": mime_type_for_events.clone(),
                "storage_path": storage_path_for_log
            })),
    );
    state.audit.emit(AuditEvent::info(
        "ugc.upload_session.created",
        json!({
            "tenant_id": tenant_id,
            "project_id": project_id,
            "upload_id": upload_id,
            "content_id": content_id,
            "filename": filename_for_events,
            "mime_type": mime_type_for_events,
            "storage_path": storage_path_for_audit,
            "size_bytes": size_bytes_for_events,
            "expires_at": expires_at,
            "labels_count": labels_count,
        }),
    ));
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
    if !state.rate_limiter.check_and_increment(
        tenant_id,
        "ugc:complete_upload",
        120,
        StdDuration::from_secs(60),
    ) {
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
        return Err(HttpError::new(
            StatusCode::FORBIDDEN,
            "upload session scope mismatch",
        ));
    }
    let now = Utc::now();
    if let Some(expires_at) = session.expires_at {
        if expires_at < now {
            return Err(HttpError::new(StatusCode::GONE, "upload session expired"));
        }
    }
    if !matches!(
        session.status,
        UploadStatus::Pending | UploadStatus::Uploading
    ) {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "upload session closed",
        ));
    }
    let storage_path = payload.storage_path.clone().unwrap_or_else(|| {
        build_storage_path(
            &tenant_id,
            &project_id,
            &session.content_id,
            &payload.filename,
        )
    });
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
    let filename_for_events = metadata.filename.clone();
    let storage_path_for_events = metadata.storage_path.clone();
    let size_bytes_for_events = metadata.size_bytes;
    let uploaded_by_for_events = metadata.uploaded_by;
    state
        .content_store
        .record_content_metadata(metadata.clone())
        .await
        .map_err(HttpError::from)?;
    let mut metric_labels = HashMap::new();
    metric_labels.insert("tenant_id".to_string(), tenant_id.to_string());
    metric_labels.insert("project_id".to_string(), project_id.to_string());
    state.telemetry.metrics.increment_counter(
        "ugc_uploads_completed",
        1.0,
        Some(metric_labels.clone()),
    );
    state.telemetry.metrics.set_gauge(
        "ugc_last_upload_size_bytes",
        payload.size_bytes as f64,
        Some(metric_labels.clone()),
    );
    let metadata_response = ContentMetadataResponse::from(metadata.clone());
    state.telemetry.logs.emit(
        LogEvent::new(LogLevel::Info, "ugc.upload_session.completed")
            .with_component("gateway")
            .with_tenant(tenant_id.to_string())
            .with_project(project_id.to_string())
            .with_metadata(json!({
                "upload_id": upload_id,
                "content_id": session.content_id,
                "filename": filename_for_events.clone(),
                "size_bytes": size_bytes_for_events,
                "storage_path": storage_path_for_events.clone()
            })),
    );
    state.audit.emit(AuditEvent::info(
        "ugc.upload_session.completed",
        json!({
            "tenant_id": tenant_id,
            "project_id": project_id,
            "upload_id": upload_id,
            "content_id": session.content_id,
            "filename": filename_for_events,
            "size_bytes": size_bytes_for_events,
            "storage_path": storage_path_for_events,
            "uploaded_by": uploaded_by_for_events,
        }),
    ));
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
    if !state.rate_limiter.check_and_increment(
        tenant_id,
        "ugc:list_content",
        120,
        StdDuration::from_secs(60),
    ) {
        return Err(HttpError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded",
        ));
    }
    let ListContentParams {
        search,
        tags,
        limit,
        offset,
    } = params;
    let search_term = search.clone();
    let tags_for_events = tags.clone();
    let query = ContentQuery {
        tenant_id,
        project_id: Some(project_id),
        search_term: search,
        tags,
        limit,
        offset,
    };
    let items = state
        .content_store
        .list_content_metadata(&query)
        .await
        .map_err(HttpError::from)?;
    let item_count = items.len();
    let mut metric_labels = HashMap::new();
    metric_labels.insert("tenant_id".to_string(), tenant_id.to_string());
    metric_labels.insert("project_id".to_string(), project_id.to_string());
    state.telemetry.metrics.set_gauge(
        "ugc_content_listing_size",
        item_count as f64,
        Some(metric_labels.clone()),
    );
    state.telemetry.logs.emit(
        LogEvent::new(LogLevel::Debug, "ugc.content.listed")
            .with_component("gateway")
            .with_tenant(tenant_id.to_string())
            .with_project(project_id.to_string())
            .with_metadata(json!({
                "count": item_count,
                "search_term": search_term.clone(),
                "tags": tags_for_events.clone(),
                "limit": limit,
                "offset": offset
            })),
    );
    state.audit.emit(AuditEvent::info(
        "ugc.content.list",
        json!({
            "tenant_id": tenant_id,
            "project_id": project_id,
            "limit": limit,
            "offset": offset,
            "search_term": search_term,
            "tags": tags_for_events,
            "result_count": item_count,
        }),
    ));
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
    if !state.rate_limiter.check_and_increment(
        Uuid::nil(),
        "observability:list_logs",
        30,
        StdDuration::from_secs(60),
    ) {
        return Err(HttpError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded",
        ));
    }
    let limit = params.limit.unwrap_or(100).min(500);
    let events = state.telemetry.log_sink.snapshot();
    let start = events.len().saturating_sub(limit);
    let slice = events.into_iter().skip(start).collect::<Vec<_>>();
    let result_count = slice.len();
    state
        .telemetry
        .metrics
        .set_gauge("gateway_log_buffer_size", result_count as f64, None);
    state.audit.emit(AuditEvent::info(
        "observability.logs.list",
        json!({
            "limit": limit,
            "returned": result_count,
        }),
    ));
    Ok(Json(
        slice.into_iter().map(TelemetryLogResponse::from).collect(),
    ))
}

// === Tenant CRUD Handlers ===

// === Self-serve Org Onboarding ===

#[cfg(feature = "db")]
#[derive(Debug, Deserialize, ToSchema)]
pub struct OnboardOrgRequest {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub plan: Option<String>,
}

#[cfg(feature = "db")]
#[derive(Debug, Serialize, ToSchema)]
pub struct OnboardOrgResponse {
    pub tenant: Tenant,
    pub user_id: Uuid,
}

#[cfg(feature = "db")]
#[utoipa::path(
    post,
    path = "/onboarding/org",
    tag = "system",
    request_body = OnboardOrgRequest,
    responses(
        (status = 201, body = OnboardOrgResponse, description = "Tenant created and user attached"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Already onboarded or slug conflict")
    ),
    security(("BearerAuth" = []))
)]
pub async fn onboard_org(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<OnboardOrgRequest>,
) -> Result<(StatusCode, Json<OnboardOrgResponse>), HttpError> {
    let user_id = require_authenticated_user_id(&headers)?;
    let tenant_store = state.tenant_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "tenant store not available",
        )
    })?;

    let name = req.name.trim();
    if name.is_empty() {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "name required"));
    }

    let slug = normalize_slug(&req.slug)?;
    if tenant_store
        .get_tenant_by_slug(&slug)
        .await
        .map_err(platform_error)?
        .is_some()
    {
        return Err(HttpError::new(StatusCode::CONFLICT, "slug already exists"));
    }

    let plan = normalize_plan(req.plan);
    let (quota_agents, quota_storage_bytes) = match plan.as_str() {
        "free" => (10, 10 * 1024 * 1024 * 1024),
        "pro" => (100, 100 * 1024 * 1024 * 1024),
        "enterprise" => (1000, 1024 * 1024 * 1024 * 1024),
        _ => (10, 10 * 1024 * 1024 * 1024),
    };

    let mut user = state
        .user_store
        .get_user_by_id(user_id)
        .await
        .map_err(platform_error)?
        .ok_or_else(|| HttpError::new(StatusCode::UNAUTHORIZED, "unauthorized"))?;

    if user.tenant_id.is_some() {
        return Err(HttpError::new(
            StatusCode::CONFLICT,
            "user already onboarded",
        ));
    }

    let now = Utc::now();
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: name.to_string(),
        slug,
        plan,
        description: None,
        settings: Default::default(),
        quota_agents,
        quota_storage_bytes,
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    let created = tenant_store
        .create_tenant(tenant)
        .await
        .map_err(platform_error)?;

    user.tenant_id = Some(created.id);
    // Give the onboarding user basic tenant management permissions.
    // (Does not grant docs:read; that is explicitly admin-controlled.)
    for scope in [
        Scope::TenantRead,
        Scope::TenantWrite,
        Scope::ApiKeyManage,
        Scope::AgentRead,
        Scope::AgentExecute,
    ] {
        if !user.scopes.contains(&scope) {
            user.scopes.push(scope);
        }
    }

    state
        .user_store
        .update_user(user.clone())
        .await
        .map_err(platform_error)?;

    state.audit.emit(AuditEvent::info(
        "onboarding.org.created",
        json!({
            "tenant_id": created.id,
            "user_id": user_id,
            "slug": created.slug,
            "plan": created.plan,
        }),
    ));

    Ok((
        StatusCode::CREATED,
        Json(OnboardOrgResponse {
            tenant: created,
            user_id,
        }),
    ))
}

#[cfg(feature = "db")]
pub async fn list_tenants(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListTenantsParams>,
) -> Result<Json<Vec<Tenant>>, HttpError> {
    ensure_scope(&headers, "admin:read")?;
    let store = state.tenant_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "tenant store not available",
        )
    })?;
    let active_only = params.active_only.unwrap_or(false);
    let tenants = store
        .list_tenants(active_only)
        .await
        .map_err(platform_error)?;
    Ok(Json(tenants))
}

#[cfg(feature = "db")]
pub async fn get_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Tenant>, HttpError> {
    ensure_scope(&headers, "admin:read")?;
    let store = state.tenant_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "tenant store not available",
        )
    })?;
    let tenant = store
        .get_tenant(tenant_id)
        .await
        .map_err(platform_error)?
        .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "tenant not found"))?;
    Ok(Json(tenant))
}

#[cfg(feature = "db")]
pub async fn create_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<Tenant>), HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let store = state.tenant_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "tenant store not available",
        )
    })?;
    if req.name.trim().is_empty() {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "name required"));
    }
    let slug = match req.slug.as_deref() {
        Some(s) if !s.trim().is_empty() => normalize_slug(s)?,
        _ => slug_from_name(&req.name)?,
    };
    let plan = normalize_plan(req.plan);
    if store
        .get_tenant_by_slug(&slug)
        .await
        .map_err(platform_error)?
        .is_some()
    {
        return Err(HttpError::new(StatusCode::CONFLICT, "slug already exists"));
    }
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: req.name,
        slug,
        plan,
        description: req.description,
        settings: Default::default(),
        quota_agents: req.quota_agents.unwrap_or(10),
        quota_storage_bytes: req.quota_storage_bytes.unwrap_or(10 * 1024 * 1024 * 1024),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let created = store.create_tenant(tenant).await.map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "tenant.created",
        json!({ "tenant_id": created.id }),
    ));
    Ok((StatusCode::CREATED, Json(created)))
}

#[cfg(feature = "db")]
pub async fn update_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<UpdateTenantRequest>,
) -> Result<Json<Tenant>, HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let store = state.tenant_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "tenant store not available",
        )
    })?;
    let mut tenant = store
        .get_tenant(tenant_id)
        .await
        .map_err(platform_error)?
        .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "tenant not found"))?;
    if let Some(name) = req.name {
        tenant.name = name.clone();
        tenant.slug = slug_from_name(&name)?;
    }
    if let Some(plan) = req.plan {
        tenant.plan = normalize_plan(Some(plan));
    }
    if let Some(desc) = req.description {
        tenant.description = Some(desc);
    }
    if let Some(qa) = req.quota_agents {
        tenant.quota_agents = qa;
    }
    if let Some(qs) = req.quota_storage_bytes {
        tenant.quota_storage_bytes = qs;
    }
    if let Some(active) = req.is_active {
        tenant.is_active = active;
    }
    tenant.updated_at = Utc::now();
    let updated = store.update_tenant(tenant).await.map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "tenant.updated",
        json!({ "tenant_id": updated.id }),
    ));
    Ok(Json(updated))
}

#[cfg(feature = "db")]
pub async fn delete_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<StatusCode, HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let store = state.tenant_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "tenant store not available",
        )
    })?;
    store
        .delete_tenant(tenant_id)
        .await
        .map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "tenant.deleted",
        json!({ "tenant_id": tenant_id }),
    ));
    Ok(StatusCode::NO_CONTENT)
}

// === User Admin (scopes / docs access) ===

#[derive(Debug, Deserialize, ToSchema, Default, IntoParams)]
pub struct ListUsersParams {
    #[serde(default)]
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub tenant_id: Option<Uuid>,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserScopesRequest {
    pub scopes: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/users",
    tag = "system",
    params(ListUsersParams),
    responses((status = 200, body = [UserInfo])),
    security(("BearerAuth" = []))
)]
pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListUsersParams>,
) -> Result<Json<Vec<UserInfo>>, HttpError> {
    ensure_scope(&headers, "admin:read")?;
    let users = state
        .user_store
        .list_users(params.tenant_id)
        .await
        .map_err(platform_error)?;

    Ok(Json(
        users
            .into_iter()
            .map(|u| UserInfo {
                id: u.id,
                email: u.email,
                display_name: u.display_name,
                tenant_id: u.tenant_id,
                scopes: u.scopes.iter().map(|s| s.as_str().to_string()).collect(),
                is_active: u.is_active,
                created_at: u.created_at,
                updated_at: u.updated_at,
                last_login_at: u.last_login_at,
            })
            .collect(),
    ))
}

#[utoipa::path(
    put,
    path = "/users/{user_id}/scopes",
    tag = "system",
    params(("user_id" = Uuid, Path, description = "User ID")),
    request_body = UpdateUserScopesRequest,
    responses((status = 200, body = UserInfo)),
    security(("BearerAuth" = []))
)]
pub async fn update_user_scopes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserScopesRequest>,
) -> Result<Json<UserInfo>, HttpError> {
    ensure_scope(&headers, "admin:write")?;
    if req.scopes.is_empty() {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "at least one scope required",
        ));
    }

    let mut user = state
        .user_store
        .get_user_by_id(user_id)
        .await
        .map_err(platform_error)?
        .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "user not found"))?;

    // Normalize + dedupe scopes.
    let mut out: Vec<Scope> = Vec::new();
    for s in req.scopes {
        let scope = Scope::from_name(s.trim());
        if !out.contains(&scope) {
            out.push(scope);
        }
    }
    user.scopes = out;
    user.updated_at = Utc::now();

    let user = state
        .user_store
        .update_user(user)
        .await
        .map_err(platform_error)?;

    state.audit.emit(AuditEvent::info(
        "user.scopes.updated",
        json!({ "user_id": user_id }),
    ));

    Ok(Json(UserInfo {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        tenant_id: user.tenant_id,
        scopes: user.scopes.iter().map(|s| s.as_str().to_string()).collect(),
        is_active: user.is_active,
        created_at: user.created_at,
        updated_at: user.updated_at,
        last_login_at: user.last_login_at,
    }))
}

// === Project CRUD Handlers ===

#[cfg(feature = "db")]
pub async fn list_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Query(params): Query<ListProjectsParams>,
) -> Result<Json<Vec<Project>>, HttpError> {
    ensure_scope(&headers, "admin:read")?;
    let store = state.project_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "project store not available",
        )
    })?;
    let active_only = params.active_only.unwrap_or(false);
    let projects = store
        .list_projects(tenant_id, active_only)
        .await
        .map_err(platform_error)?;
    Ok(Json(projects))
}

#[cfg(feature = "db")]
pub async fn get_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Project>, HttpError> {
    ensure_scope(&headers, "admin:read")?;
    let _ = tenant_id; // Validate tenant ownership in production
    let store = state.project_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "project store not available",
        )
    })?;
    let project = store
        .get_project(project_id)
        .await
        .map_err(platform_error)?
        .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "project not found"))?;
    Ok(Json(project))
}

#[cfg(feature = "db")]
pub async fn create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let store = state.project_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "project store not available",
        )
    })?;
    let slug = req.name.to_lowercase().replace(' ', "-");
    let project = Project {
        id: Uuid::new_v4(),
        tenant_id,
        name: req.name,
        slug,
        description: req.description,
        settings: req.settings.unwrap_or(json!({})),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let created = store
        .create_project(project)
        .await
        .map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "project.created",
        json!({ "project_id": created.id, "tenant_id": tenant_id }),
    ));
    Ok((StatusCode::CREATED, Json(created)))
}

#[cfg(feature = "db")]
pub async fn update_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let _ = tenant_id;
    let store = state.project_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "project store not available",
        )
    })?;
    let mut project = store
        .get_project(project_id)
        .await
        .map_err(platform_error)?
        .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "project not found"))?;
    if let Some(name) = req.name {
        project.name = name.clone();
        project.slug = name.to_lowercase().replace(' ', "-");
    }
    if let Some(desc) = req.description {
        project.description = Some(desc);
    }
    if let Some(settings) = req.settings {
        project.settings = settings;
    }
    if let Some(active) = req.is_active {
        project.is_active = active;
    }
    project.updated_at = Utc::now();
    let updated = store
        .update_project(project)
        .await
        .map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "project.updated",
        json!({ "project_id": updated.id }),
    ));
    Ok(Json(updated))
}

#[cfg(feature = "db")]
pub async fn delete_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let _ = tenant_id;
    let store = state.project_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "project store not available",
        )
    })?;
    store
        .delete_project(project_id)
        .await
        .map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "project.deleted",
        json!({ "project_id": project_id }),
    ));
    Ok(StatusCode::NO_CONTENT)
}

// === Task Queue Handlers ===

#[cfg(feature = "db")]
pub async fn list_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Query(params): Query<ListTasksParams>,
) -> Result<Json<Vec<Task>>, HttpError> {
    ensure_scope(&headers, "admin:read")?;
    let store = state.task_store.as_ref().ok_or_else(|| {
        HttpError::new(StatusCode::SERVICE_UNAVAILABLE, "task store not available")
    })?;
    let limit = params.limit.unwrap_or(100);
    let tasks = store
        .list_pending_tasks(tenant_id, limit)
        .await
        .map_err(platform_error)?;
    Ok(Json(tasks))
}

#[cfg(feature = "db")]
pub async fn get_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Task>, HttpError> {
    ensure_scope(&headers, "admin:read")?;
    let _ = tenant_id;
    let store = state.task_store.as_ref().ok_or_else(|| {
        HttpError::new(StatusCode::SERVICE_UNAVAILABLE, "task store not available")
    })?;
    let task = store
        .get_task(task_id)
        .await
        .map_err(platform_error)?
        .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "task not found"))?;
    Ok(Json(task))
}

#[cfg(feature = "db")]
pub async fn create_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let store = state.task_store.as_ref().ok_or_else(|| {
        HttpError::new(StatusCode::SERVICE_UNAVAILABLE, "task store not available")
    })?;
    let task = Task {
        id: Uuid::new_v4(),
        tenant_id,
        kind: req.kind,
        payload: req.payload,
        status: TaskStatus::Pending,
        attempts: 0,
        scheduled_at: req.scheduled_at.unwrap_or_else(Utc::now),
        started_at: None,
        completed_at: None,
        last_error: None,
        result: None,
        timeouts: None,
    };
    let created = store.enqueue_task(task).await.map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "task.created",
        json!({ "task_id": created.id, "kind": &created.kind }),
    ));
    Ok((StatusCode::CREATED, Json(created)))
}

#[cfg(feature = "db")]
pub async fn lease_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<LeaseTaskRequest>,
) -> Result<Json<serde_json::Value>, HttpError> {
    ensure_scope(&headers, "agent:read")?;
    let store = state.task_store.as_ref().ok_or_else(|| {
        HttpError::new(StatusCode::SERVICE_UNAVAILABLE, "task store not available")
    })?;
    let lease = store
        .lease_next_task(tenant_id, req.worker_id, req.lease_seconds)
        .await
        .map_err(platform_error)?;
    match lease {
        Some(l) => Ok(Json(json!({
            "task": l.task,
            "worker_id": l.worker_id,
            "leased_at": l.leased_at,
            "lease_expires_at": l.lease_expires_at,
            "lease_token": l.lease_token,
        }))),
        None => Ok(Json(json!({ "task": null }))),
    }
}

#[cfg(feature = "db")]
pub async fn complete_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, task_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<CompleteTaskRequest>,
) -> Result<StatusCode, HttpError> {
    ensure_scope(&headers, "agent:write")?;
    let _ = tenant_id;
    let store = state.task_store.as_ref().ok_or_else(|| {
        HttpError::new(StatusCode::SERVICE_UNAVAILABLE, "task store not available")
    })?;
    store
        .complete_task(task_id, req.result, req.error)
        .await
        .map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "task.completed",
        json!({ "task_id": task_id }),
    ));
    Ok(StatusCode::NO_CONTENT)
}

// === Workflow Handlers ===

#[cfg(feature = "db")]
pub async fn list_workflows(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Vec<Workflow>>, HttpError> {
    ensure_scope(&headers, "admin:read")?;
    let store = state.workflow_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "workflow store not available",
        )
    })?;
    let workflows = store
        .list_workflows(tenant_id, false)
        .await
        .map_err(platform_error)?;
    Ok(Json(workflows))
}

#[cfg(feature = "db")]
pub async fn get_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, workflow_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Workflow>, HttpError> {
    ensure_scope(&headers, "admin:read")?;
    let _ = tenant_id;
    let store = state.workflow_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "workflow store not available",
        )
    })?;
    let workflow = store
        .get_workflow(workflow_id)
        .await
        .map_err(platform_error)?
        .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "workflow not found"))?;
    Ok(Json(workflow))
}

#[cfg(feature = "db")]
pub async fn create_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<(StatusCode, Json<Workflow>), HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let store = state.workflow_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "workflow store not available",
        )
    })?;
    let steps: Vec<cncore::platform::WorkflowStep> = req
        .steps
        .into_iter()
        .map(|s| {
            serde_json::from_value(s).unwrap_or_else(|_| cncore::platform::WorkflowStep {
                id: Uuid::new_v4(),
                name: "unknown".to_string(),
                task_kind: "unknown".to_string(),
                dependencies: vec![],
            })
        })
        .collect();
    let workflow = Workflow {
        id: Uuid::new_v4(),
        tenant_id,
        name: req.name,
        steps,
        created_at: Utc::now(),
    };
    let created = store
        .create_workflow(workflow)
        .await
        .map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "workflow.created",
        json!({ "workflow_id": created.id }),
    ));
    Ok((StatusCode::CREATED, Json(created)))
}

#[cfg(feature = "db")]
pub async fn delete_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, workflow_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let _ = tenant_id;
    let store = state.workflow_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "workflow store not available",
        )
    })?;
    store
        .delete_workflow(workflow_id)
        .await
        .map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "workflow.deleted",
        json!({ "workflow_id": workflow_id }),
    ));
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(feature = "db")]
pub async fn start_workflow_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, workflow_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<StartWorkflowRunRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let _ = tenant_id;
    let store = state.workflow_store.as_ref().ok_or_else(|| {
        HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "workflow store not available",
        )
    })?;
    let context = req.context.unwrap_or(json!({}));
    let run = store
        .start_run(workflow_id, context)
        .await
        .map_err(platform_error)?;
    state.audit.emit(AuditEvent::info(
        "workflow.run.started",
        json!({ "workflow_id": workflow_id, "run_id": run.id }),
    ));
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": run.id,
            "workflow_id": run.workflow_id,
            "status": run.status,
            "created_at": run.created_at,
        })),
    ))
}

fn platform_error(e: PlatformError) -> HttpError {
    match e {
        PlatformError::NotFound(_) => HttpError::new(StatusCode::NOT_FOUND, "not found"),
        PlatformError::Conflict(_) => HttpError::new(StatusCode::CONFLICT, "already exists"),
        PlatformError::Unauthorized => HttpError::new(StatusCode::UNAUTHORIZED, "unauthorized"),
        PlatformError::Forbidden => HttpError::new(StatusCode::FORBIDDEN, "forbidden"),
        PlatformError::InvalidInput(_) => HttpError::new(StatusCode::BAD_REQUEST, "invalid input"),
        _ => HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal error"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentPresenceFilter {
    Active,
    Stale,
    Offline,
}

impl AgentPresenceFilter {
    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "active" => Some(Self::Active),
            "stale" => Some(Self::Stale),
            "offline" => Some(Self::Offline),
            _ => None,
        }
    }

    fn matches(&self, last_seen: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
        const ACTIVE_SECS: i64 = 30;
        const STALE_SECS: i64 = 300;
        match (self, last_seen) {
            (_, None) => matches!(self, Self::Offline),
            (Self::Active, Some(ts)) => (now - ts) <= Duration::seconds(ACTIVE_SECS),
            (Self::Stale, Some(ts)) => {
                let age = now - ts;
                age > Duration::seconds(ACTIVE_SECS) && age <= Duration::seconds(STALE_SECS)
            }
            (Self::Offline, Some(ts)) => (now - ts) > Duration::seconds(STALE_SECS),
        }
    }
}

fn filter_agent_summaries(
    mut agents: Vec<AgentSummary>,
    params: &ListAgentsParams,
) -> Vec<AgentSummary> {
    let hostname_filter = params.hostname.as_ref().map(|s| s.to_ascii_lowercase());
    let status_filter = params
        .status
        .as_deref()
        .and_then(AgentPresenceFilter::parse);
    let lifecycle_filter = params
        .lifecycle_status
        .as_ref()
        .map(|s| s.to_ascii_lowercase())
        .or_else(|| {
            if status_filter.is_none() {
                params.status.as_ref().map(|s| s.to_ascii_lowercase())
            } else {
                None
            }
        });
    let tenant_filter = params.tenant_id.as_ref().map(|s| s.to_ascii_lowercase());
    let project_filter = params.project_id.as_ref().map(|s| s.to_ascii_lowercase());
    let last_seen_after = params.last_seen_after;
    let last_seen_before = params.last_seen_before;
    let now = Utc::now();

    agents.retain(|agent| {
        if let Some(hostname) = &hostname_filter {
            if !agent.hostname.to_ascii_lowercase().contains(hostname) {
                return false;
            }
        }
        if let Some(tenant) = &tenant_filter {
            match &agent.tenant_id {
                Some(value) if value.to_ascii_lowercase() == *tenant => {}
                _ => return false,
            }
        }
        if let Some(project) = &project_filter {
            match &agent.project_id {
                Some(value) if value.to_ascii_lowercase() == *project => {}
                _ => return false,
            }
        }
        if let Some(lifecycle) = &lifecycle_filter {
            match &agent.lifecycle_status {
                Some(current) if current.to_ascii_lowercase() == *lifecycle => {}
                _ => return false,
            }
        }
        if let Some(after) = last_seen_after {
            let after = after.max(0) as u64;
            if agent.last_seen_unix_ms < after {
                return false;
            }
        }
        if let Some(before) = last_seen_before {
            let before = before.max(0) as u64;
            if agent.last_seen_unix_ms > before {
                return false;
            }
        }
        if let Some(filter) = status_filter {
            let last_seen_dt = agent_last_seen_datetime(agent);
            if !filter.matches(last_seen_dt, now) {
                return false;
            }
        }
        true
    });

    agents.sort_by(|a, b| {
        b.last_seen_unix_ms
            .cmp(&a.last_seen_unix_ms)
            .then_with(|| a.id.cmp(&b.id))
    });

    let offset = params.offset.unwrap_or(0) as usize;
    let limit = params.limit.map(|v| v as usize).unwrap_or(usize::MAX);
    agents.into_iter().skip(offset).take(limit).collect()
}

#[cfg(feature = "db")]
fn build_agent_query(params: &ListAgentsParams) -> Result<AgentQuery, HttpError> {
    use uuid::Uuid;

    let tenant_id = params
        .tenant_id
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|value| {
            Uuid::parse_str(value)
                .map_err(|_| HttpError::new(StatusCode::BAD_REQUEST, "invalid tenant_id"))
        })
        .transpose()?;
    let project_id = params
        .project_id
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|value| {
            Uuid::parse_str(value)
                .map_err(|_| HttpError::new(StatusCode::BAD_REQUEST, "invalid project_id"))
        })
        .transpose()?;
    let last_seen_after = params.last_seen_after.and_then(datetime_from_millis);
    let last_seen_before = params.last_seen_before.and_then(datetime_from_millis);
    let hostname_fragment = params.hostname.clone();
    let lifecycle_status = params
        .lifecycle_status
        .as_ref()
        .map(|s| s.to_ascii_lowercase());

    Ok(AgentQuery {
        tenant_id,
        project_id,
        hostname_fragment,
        lifecycle_status,
        last_seen_after,
        last_seen_before,
        limit: params.limit.map(|v| v as i64),
        offset: params.offset.map(|v| v as i64),
    })
}

#[cfg(feature = "db")]
fn map_agent_record(record: AgentSummaryRecord) -> AgentSummary {
    AgentSummary {
        id: record.id.to_string(),
        hostname: record.hostname,
        last_seen_unix_ms: record
            .last_seen
            .map(|dt| dt.timestamp_millis().max(0) as u64)
            .unwrap_or(0),
        cpu_percent: record.cpu_percent.unwrap_or(0.0),
        memory_used_bytes: record
            .memory_used_bytes
            .map(|v| v.max(0) as u64)
            .unwrap_or(0),
        tenant_id: record.tenant_id.map(|id| id.to_string()),
        project_id: record.project_id.map(|id| id.to_string()),
        lifecycle_status: record.lifecycle_status,
    }
}

fn agent_last_seen_datetime(agent: &AgentSummary) -> Option<DateTime<Utc>> {
    if agent.last_seen_unix_ms == 0 {
        return None;
    }
    let millis = i64::try_from(agent.last_seen_unix_ms).ok()?;
    DateTime::<Utc>::from_timestamp_millis(millis)
}

#[cfg(feature = "db")]
fn datetime_from_millis(value: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp_millis(value)
}

// ==== API Key Management ====

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    /// Tenant the key belongs to.
    pub tenant_id: Uuid,
    /// Human-readable label for the key
    pub label: String,
    /// List of scopes the key grants (e.g., ["agent:read", "ugc:write"])
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateApiKeyResponse {
    /// The API key ID (for management operations)
    pub id: Uuid,
    /// The full API key - ONLY returned once at creation time
    pub api_key: String,
    /// Tenant the key belongs to
    pub tenant_id: Uuid,
    /// Prefix of the key for identification
    pub prefix: String,
    /// Label for the key
    pub label: String,
    /// Granted scopes
    pub scopes: Vec<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub prefix: String,
    pub label: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked: bool,
}

#[derive(Debug, Deserialize, ToSchema, Default, IntoParams)]
pub struct ListApiKeysParams {
    #[serde(default)]
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateApiKeyScopesRequest {
    pub scopes: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/api-keys",
    tag = "system",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, body = CreateApiKeyResponse, description = "API key created"),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn create_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreateApiKeyResponse>), HttpError> {
    ensure_scope(&headers, "apikey:manage")?;

    if payload.label.trim().is_empty() {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "label required"));
    }
    if payload.scopes.is_empty() {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "at least one scope required",
        ));
    }

    // Generate a secure random API key
    let key_id = Uuid::new_v4();
    let random_bytes: [u8; 32] = rand::random();
    let full_key = format!(
        "cass_{}",
        base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            random_bytes
        )
    );
    let prefix = full_key[..12].to_string(); // "cass_" + 7 chars

    // Hash the key for storage
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(full_key.as_bytes());
    let _token_hash = hex::encode(hasher.finalize());

    let now = Utc::now();

    state.audit.emit(AuditEvent::info(
        "apikey.created",
        json!({
            "key_id": key_id.to_string(),
            "label": &payload.label,
            "scopes": &payload.scopes,
            "prefix": &prefix,
        }),
    ));

    #[cfg(feature = "db")]
    {
        use cncore::platform::ApiKeyRecord;
        let store = state.api_key_store.as_ref().ok_or_else(|| {
            HttpError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "api key store not available",
            )
        })?;

        let scopes: Vec<Scope> = payload
            .scopes
            .iter()
            .map(|s| Scope::from_name(s.trim()))
            .collect();

        let record = ApiKeyRecord {
            id: key_id,
            tenant_id: payload.tenant_id,
            label: payload.label.clone(),
            scopes,
            token_prefix: prefix.clone(),
            token_hash: _token_hash.clone(),
            created_at: now,
            last_used_at: None,
            revoked: false,
            deleted_at: None,
            rotated_from: None,
            rotated_to: None,
        };
        store.insert_api_key(record).await.map_err(platform_error)?;

        crate::auth::upsert_db_api_key_cache(
            key_id,
            prefix.clone(),
            _token_hash.clone(),
            payload.scopes.clone(),
        );
    }

    #[cfg(not(feature = "db"))]
    {
        // In non-DB mode, just return the generated key (caller must store in CASS_API_KEYS)
        tracing::warn!(
            key_id = %key_id,
            prefix = %prefix,
            "api_key.created_no_persistence - add to CASS_API_KEYS env var"
        );
    }

    Ok((
        StatusCode::CREATED,
        Json(CreateApiKeyResponse {
            id: key_id,
            api_key: full_key,
            tenant_id: payload.tenant_id,
            prefix,
            label: payload.label,
            scopes: payload.scopes,
            created_at: now,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api-keys",
    tag = "system",
    responses(
        (status = 200, body = [ApiKeyInfo], description = "List of API keys"),
        (status = 403, description = "Insufficient permissions")
    )
)]
#[cfg_attr(not(feature = "db"), allow(unused_variables))]
pub async fn list_api_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListApiKeysParams>,
) -> Result<Json<Vec<ApiKeyInfo>>, HttpError> {
    ensure_scope(&headers, "apikey:manage")?;

    let Some(tenant_id) = params.tenant_id else {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "tenant_id required",
        ));
    };

    state.audit.emit(AuditEvent::info("apikey.list", json!({})));

    #[cfg(feature = "db")]
    {
        let store = state.api_key_store.as_ref().ok_or_else(|| {
            HttpError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "api key store not available",
            )
        })?;
        let keys = store
            .list_api_keys(tenant_id)
            .await
            .map_err(platform_error)?;
        let mapped = keys
            .into_iter()
            .map(|k| ApiKeyInfo {
                id: k.id,
                tenant_id: k.tenant_id,
                prefix: k.token_prefix,
                label: k.label,
                scopes: k.scopes.iter().map(|s| s.as_str().to_string()).collect(),
                created_at: k.created_at,
                last_used_at: k.last_used_at,
                revoked: k.revoked,
            })
            .collect();
        Ok(Json(mapped))
    }

    #[cfg(not(feature = "db"))]
    {
        // In env-var mode we don't expose those keys.
        Ok(Json(vec![]))
    }
}

#[utoipa::path(
    delete,
    path = "/api-keys/{key_id}",
    tag = "system",
    params(("key_id" = Uuid, Path, description = "API key ID to revoke")),
    responses(
        (status = 204, description = "API key revoked"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "API key not found")
    )
)]
pub async fn revoke_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(key_id): Path<Uuid>,
) -> Result<StatusCode, HttpError> {
    ensure_scope(&headers, "apikey:manage")?;

    state.audit.emit(AuditEvent::warn(
        "apikey.revoked",
        json!({ "key_id": key_id.to_string() }),
    ));

    #[cfg(feature = "db")]
    {
        let store = state.api_key_store.as_ref().ok_or_else(|| {
            HttpError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "api key store not available",
            )
        })?;
        let mut record = store
            .get_api_key(key_id)
            .await
            .map_err(platform_error)?
            .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "api key not found"))?;
        record.revoked = true;
        store
            .update_api_key(record.clone())
            .await
            .map_err(platform_error)?;
        crate::auth::remove_db_api_key_cache(&record.token_prefix);
        Ok(StatusCode::NO_CONTENT)
    }

    #[cfg(not(feature = "db"))]
    {
        tracing::warn!(
            key_id = %key_id,
            "api_key.revoke_no_persistence - remove from CASS_API_KEYS env var manually"
        );
        Ok(StatusCode::NO_CONTENT)
    }
}

#[utoipa::path(
    put,
    path = "/api-keys/{key_id}/scopes",
    tag = "system",
    params(("key_id" = Uuid, Path, description = "API key ID")),
    request_body = UpdateApiKeyScopesRequest,
    responses((status = 200, body = ApiKeyInfo)),
    security(("BearerAuth" = []), ("ApiKey" = []))
)]
#[cfg_attr(not(feature = "db"), allow(unused_variables))]
pub async fn update_api_key_scopes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(key_id): Path<Uuid>,
    Json(req): Json<UpdateApiKeyScopesRequest>,
) -> Result<Json<ApiKeyInfo>, HttpError> {
    ensure_scope(&headers, "apikey:manage")?;
    if req.scopes.is_empty() {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "at least one scope required",
        ));
    }

    #[cfg(feature = "db")]
    {
        let store = state.api_key_store.as_ref().ok_or_else(|| {
            HttpError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "api key store not available",
            )
        })?;
        let mut record = store
            .get_api_key(key_id)
            .await
            .map_err(platform_error)?
            .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "api key not found"))?;
        record.scopes = req
            .scopes
            .iter()
            .map(|s| Scope::from_name(s.trim()))
            .collect();
        store
            .update_api_key(record.clone())
            .await
            .map_err(platform_error)?;

        // Keep auth cache in sync.
        crate::auth::upsert_db_api_key_cache(
            record.id,
            record.token_prefix.clone(),
            record.token_hash.clone(),
            record
                .scopes
                .iter()
                .map(|s| s.as_str().to_string())
                .collect(),
        );

        Ok(Json(ApiKeyInfo {
            id: record.id,
            tenant_id: record.tenant_id,
            prefix: record.token_prefix,
            label: record.label,
            scopes: record
                .scopes
                .iter()
                .map(|s| s.as_str().to_string())
                .collect(),
            created_at: record.created_at,
            last_used_at: record.last_used_at,
            revoked: record.revoked,
        }))
    }

    #[cfg(not(feature = "db"))]
    {
        Err(HttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "api key store not available",
        ))
    }
}

#[cfg(not(feature = "db"))]
#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        version,
        metrics,
        list_agents,
        get_agent_config,
        update_agent_config,
        list_agent_commands,
        send_agent_command,
        // System API
        create_upload_session,
        complete_upload_session,
        list_content_metadata,
        list_recent_logs,
        create_api_key,
        list_api_keys,
        revoke_api_key,
        update_api_key_scopes,
        list_users,
        update_user_scopes,
        // Portal API
        list_builds,
        create_build,
        get_build,
        delete_build,
        create_deployment,
        list_scripts,
        create_script,
        get_script,
        update_script,
        delete_script,
        execute_script,
        list_script_executions,
        // Platform settings
        get_settings,
        update_settings
    ),
    components(
        schemas(
            HealthResponse,
            VersionResponse,
            AgentSummary,
            ListAgentsParams,
            ErrorResponse,
            AgentConfig,
            UpdateAgentConfigRequest,
            SendAgentCommandRequest,
            AgentCommand,
            Build,
            BuildUploadForm,
            DeployBuildRequest,
            DeploymentResponse,
            Script,
            CreateScriptRequest,
            UpdateScriptRequest,
            ExecuteScriptRequest,
            ScriptExecution,
            PlatformSettings,
            UpdatePlatformSettingsRequest,
            CreateUploadRequest,
            CompleteUploadRequest,
            ListContentParams,
            ListLogsParams,
            UploadSessionResponse,
            ContentMetadataResponse,
            TelemetryLogResponse,
            CreateApiKeyRequest,
            CreateApiKeyResponse,
            ApiKeyInfo,
            UpdateApiKeyScopesRequest,
            ListUsersParams,
            UserInfo,
            UpdateUserScopesRequest
        )
    ),
    tags(
        (name = "system", description = "System & meta endpoints"),
        (name = "portal", description = "Portal (console) endpoints")
    )
)]
pub struct ApiDoc;

#[cfg(feature = "db")]
#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        version,
        metrics,
        list_agents,
        get_agent_config,
        update_agent_config,
        list_agent_commands,
        send_agent_command,
        // System API
        create_upload_session,
        complete_upload_session,
        list_content_metadata,
        list_recent_logs,
        create_api_key,
        list_api_keys,
        revoke_api_key,
        update_api_key_scopes,
        list_users,
        update_user_scopes,
        onboard_org,
        // Portal API
        list_builds,
        create_build,
        get_build,
        delete_build,
        create_deployment,
        list_scripts,
        create_script,
        get_script,
        update_script,
        delete_script,
        execute_script,
        list_script_executions,
        // Platform settings
        get_settings,
        update_settings
    ),
    components(
        schemas(
            HealthResponse,
            VersionResponse,
            AgentSummary,
            ListAgentsParams,
            ErrorResponse,
            AgentConfig,
            UpdateAgentConfigRequest,
            SendAgentCommandRequest,
            AgentCommand,
            Build,
            BuildUploadForm,
            DeployBuildRequest,
            DeploymentResponse,
            Script,
            CreateScriptRequest,
            UpdateScriptRequest,
            ExecuteScriptRequest,
            ScriptExecution,
            PlatformSettings,
            UpdatePlatformSettingsRequest,
            CreateUploadRequest,
            CompleteUploadRequest,
            ListContentParams,
            ListLogsParams,
            UploadSessionResponse,
            ContentMetadataResponse,
            TelemetryLogResponse,
            CreateApiKeyRequest,
            CreateApiKeyResponse,
            ApiKeyInfo,
            UpdateApiKeyScopesRequest,
            ListUsersParams,
            UserInfo,
            UpdateUserScopesRequest,
            OnboardOrgRequest,
            OnboardOrgResponse
        )
    ),
    tags(
        (name = "system", description = "System & meta endpoints"),
        (name = "portal", description = "Portal (console) endpoints")
    )
)]
pub struct ApiDoc;

/// Largest accepted build upload (`CASS_MAX_BUILD_BYTES`, default 2 GiB).
fn max_build_bytes() -> usize {
    std::env::var("CASS_MAX_BUILD_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2 * 1024 * 1024 * 1024)
}

pub fn router() -> Router<AppState> {
    let base = Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/metrics", get(metrics))
        .route("/agents", get(list_agents))
        .route(
            "/agents/:agent_id/config",
            get(get_agent_config).put(update_agent_config),
        )
        .route(
            "/agents/:agent_id/commands",
            get(list_agent_commands).post(send_agent_command),
        )
        .route("/api-keys", get(list_api_keys).post(create_api_key))
        .route("/api-keys/:key_id", delete(revoke_api_key))
        .route("/api-keys/:key_id/scopes", put(update_api_key_scopes))
        .route("/users", get(list_users))
        .route("/users/:user_id/scopes", put(update_user_scopes))
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
        // Portal API routes
        .route(
            "/builds",
            get(list_builds)
                .post(create_build)
                .layer(DefaultBodyLimit::max(max_build_bytes())),
        )
        .route("/builds/:build_id", get(get_build).delete(delete_build))
        .route("/deployments", post(create_deployment))
        .route("/scripts", get(list_scripts).post(create_script))
        .route(
            "/scripts/:script_id",
            get(get_script).put(update_script).delete(delete_script),
        )
        .route("/scripts/:script_id/execute", post(execute_script))
        .route(
            "/scripts/:script_id/executions",
            get(list_script_executions),
        )
        .route("/settings", get(get_settings).put(update_settings))
        // UGC routes
        .route("/ugc/content", get(list_content).post(create_content))
        .route("/ugc/content/:id/state", put(update_content_state))
        // Messaging routes
        .route("/messaging/publish", post(publish_message))
        .route("/messaging/messages", get(list_messages))
        .route("/messaging/messages/:id/ack", post(ack_message))
        .merge(crate::auth_handlers::auth_router());

    #[cfg(feature = "db")]
    let base = base
        .route("/onboarding/org", post(onboard_org))
        // Tenant routes
        .route("/tenants", get(list_tenants).post(create_tenant))
        .route(
            "/tenants/:tenant_id",
            get(get_tenant).put(update_tenant).delete(delete_tenant),
        )
        // Project routes (nested under tenants)
        .route(
            "/tenants/:tenant_id/projects",
            get(list_projects).post(create_project),
        )
        .route(
            "/tenants/:tenant_id/projects/:project_id",
            get(get_project).put(update_project).delete(delete_project),
        )
        // Task queue routes
        .route(
            "/tenants/:tenant_id/tasks",
            get(list_tasks).post(create_task),
        )
        .route("/tenants/:tenant_id/tasks/lease", post(lease_task))
        .route("/tenants/:tenant_id/tasks/:task_id", get(get_task))
        .route(
            "/tenants/:tenant_id/tasks/:task_id/complete",
            post(complete_task),
        )
        // Workflow routes
        .route(
            "/tenants/:tenant_id/workflows",
            get(list_workflows).post(create_workflow),
        )
        .route(
            "/tenants/:tenant_id/workflows/:workflow_id",
            get(get_workflow).delete(delete_workflow),
        )
        .route(
            "/tenants/:tenant_id/workflows/:workflow_id/runs",
            post(start_workflow_run),
        );

    base
}

pub(crate) fn ensure_scope(headers: &HeaderMap, scope: &str) -> Result<(), HttpError> {
    if has_scope(headers, scope) {
        Ok(())
    } else {
        Err(HttpError::new(StatusCode::FORBIDDEN, "scope required"))
    }
}

#[cfg(feature = "db")]
fn require_authenticated_user_id(headers: &HeaderMap) -> Result<Uuid, HttpError> {
    let status = validate_jwt(headers);
    if !status.allowed {
        return Err(HttpError::new(StatusCode::UNAUTHORIZED, "unauthorized"));
    }
    let sub = status
        .subject
        .ok_or_else(|| HttpError::new(StatusCode::UNAUTHORIZED, "unauthorized"))?;
    Uuid::parse_str(&sub).map_err(|_| HttpError::new(StatusCode::UNAUTHORIZED, "unauthorized"))
}

#[cfg(feature = "db")]
fn normalize_plan(input: Option<String>) -> String {
    // Keep this intentionally permissive: allow arbitrary plan identifiers.
    // We just normalize case/whitespace and clamp length.
    let plan = input.unwrap_or_else(|| "free".to_string());
    let plan = plan.trim().to_lowercase();
    if plan.is_empty() {
        "free".to_string()
    } else if plan.len() > 64 {
        plan[..64].to_string()
    } else {
        plan
    }
}

#[cfg(feature = "db")]
fn normalize_slug(input: &str) -> Result<String, HttpError> {
    let slug = input.trim().to_lowercase();
    if slug.len() < 3 || slug.len() > 63 {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "invalid slug"));
    }
    let bytes = slug.as_bytes();
    if bytes.first() == Some(&b'-') || bytes.last() == Some(&b'-') {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "invalid slug"));
    }
    if !bytes
        .iter()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-'))
    {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "invalid slug"));
    }
    if slug.contains("--") {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "invalid slug"));
    }
    Ok(slug)
}

#[cfg(feature = "db")]
fn slug_from_name(name: &str) -> Result<String, HttpError> {
    let slug = name
        .trim()
        .to_lowercase()
        .replace([' ', '_'], "-")
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .collect::<String>();
    normalize_slug(&slug)
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
