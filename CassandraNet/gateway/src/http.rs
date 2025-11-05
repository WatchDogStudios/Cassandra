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
#[cfg(feature = "db")]
use cncore::platform::persistence::{AgentQuery, AgentSummaryRecord};
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
    Query(params): Query<ListAgentsParams>,
) -> Result<Json<Vec<AgentSummary>>, HttpError> {
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
    state.telemetry.metrics.set_gauge(
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
    state
        .telemetry
        .metrics
        .set_gauge("gateway_log_buffer_size", slice.len() as f64, None);
    Ok(Json(
        slice.into_iter().map(TelemetryLogResponse::from).collect(),
    ))
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

fn datetime_from_millis(value: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp_millis(value)
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
            ListAgentsParams,
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
