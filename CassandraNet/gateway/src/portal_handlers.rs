//! Operator-portal API: per-agent config and commands, build artifacts and
//! deployments, the script library and runs, and platform settings.
//!
//! Commands, script runs and deployments all become `AgentJob`s that agents
//! pick up on their next heartbeat and report back over gRPC.
use crate::http::{ensure_scope, HttpError};
use crate::state::audit::AuditEvent;
use crate::state::AppState;
use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Utc};
use cncore::platform::{AgentJob, BuildRecord, JobQuery, ScriptRecord};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use utoipa::ToSchema;
use uuid::Uuid;

/// Commands an operator may send; the agent implements exactly these.
pub const AGENT_COMMANDS: &[&str] = &[
    "restart",
    "shutdown",
    "exec",
    "collect_logs",
    "deploy_build",
];
/// Languages the agent knows how to interpret.
pub const SCRIPT_LANGUAGES: &[&str] = &["bash", "sh", "powershell", "python", "javascript", "lua"];
/// Heartbeat interval agents use until an operator configures another.
pub const DEFAULT_HEARTBEAT_INTERVAL_SECS: u32 = 5;
const DEFAULT_JOB_TIMEOUT_SECS: u32 = 300;
const MAX_JOB_TIMEOUT_SECS: u32 = 3600;
const MAX_SCRIPT_BYTES: usize = 256 * 1024;
const MAX_LISTED_JOBS: u32 = 100;

// === Types ===

/// Agent configuration. The agent currently applies `heartbeat_interval_seconds`;
/// the other fields are stored for operators and future agent versions.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentConfig {
    pub heartbeat_interval_seconds: u32,
    pub log_level: String,
    pub auto_update_enabled: bool,
    pub max_concurrent_tasks: u32,
    pub metrics_enabled: bool,
    pub custom_labels: HashMap<String, String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_seconds: DEFAULT_HEARTBEAT_INTERVAL_SECS,
            log_level: "info".to_string(),
            auto_update_enabled: true,
            max_concurrent_tasks: 4,
            metrics_enabled: true,
            custom_labels: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAgentConfigRequest {
    #[serde(default)]
    pub heartbeat_interval_seconds: Option<u32>,
    #[serde(default)]
    pub log_level: Option<String>,
    #[serde(default)]
    pub auto_update_enabled: Option<bool>,
    #[serde(default)]
    pub max_concurrent_tasks: Option<u32>,
    #[serde(default)]
    pub metrics_enabled: Option<bool>,
    #[serde(default)]
    pub custom_labels: Option<HashMap<String, String>>,
}

/// A job as seen from the agent's command history.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentCommand {
    pub id: Uuid,
    pub agent_id: String,
    pub command: String,
    #[serde(default)]
    pub args: HashMap<String, Value>,
    /// pending | dispatched | completed | failed
    pub status: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    /// `{exit_code, stdout, stderr}` once the agent has reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<AgentJob> for AgentCommand {
    fn from(job: AgentJob) -> Self {
        let result = job.completed_at.map(
            |_| json!({"exit_code": job.exit_code, "stdout": job.stdout, "stderr": job.stderr}),
        );
        Self {
            id: job.id,
            agent_id: job.agent_id,
            command: job.kind,
            args: serde_json::from_value(job.args).unwrap_or_default(),
            status: job.status.as_str().to_string(),
            created_at: job.created_at,
            completed_at: job.completed_at,
            result,
            error: job.error,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SendAgentCommandRequest {
    /// One of restart, shutdown, exec, collect_logs, deploy_build.
    pub command: String,
    /// exec: `{"command": "...", "timeout_seconds": 60}`; deploy_build: `{"build_id": "..."}`.
    #[serde(default)]
    pub args: HashMap<String, Value>,
}

/// Build artifact stored by the gateway and deployable to agents.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Build {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub platform: String,
    pub architecture: String,
    pub filename: String,
    pub size_bytes: u64,
    pub checksum_sha256: String,
    /// Always `ready`: a build is only recorded once its upload has finished.
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploaded_by: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

impl From<BuildRecord> for Build {
    fn from(b: BuildRecord) -> Self {
        Self {
            id: b.id,
            name: b.name,
            version: b.version,
            platform: b.platform,
            architecture: b.architecture,
            filename: b.filename,
            size_bytes: b.size_bytes,
            checksum_sha256: b.checksum_sha256,
            status: "ready".to_string(),
            uploaded_by: b.uploaded_by,
            metadata: b.metadata,
            created_at: b.created_at,
        }
    }
}

/// `multipart/form-data` body for `POST /builds`.
#[derive(ToSchema)]
#[allow(dead_code)] // documents the multipart form for OpenAPI only
pub struct BuildUploadForm {
    pub name: String,
    pub version: String,
    pub platform: String,
    pub architecture: String,
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeployBuildRequest {
    pub build_id: Uuid,
    pub agent_ids: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeploymentResponse {
    pub deployment_id: Uuid,
    pub build_id: Uuid,
    pub agent_ids: Vec<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// Script that can be executed on agents.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Script {
    pub id: Uuid,
    pub name: String,
    pub language: String,
    pub content: String,
    pub description: Option<String>,
    /// Incremented on every update.
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl From<ScriptRecord> for Script {
    fn from(s: ScriptRecord) -> Self {
        Self {
            id: s.id,
            name: s.name,
            language: s.language,
            content: s.content,
            description: s.description,
            version: s.version,
            created_at: s.created_at,
            updated_at: s.updated_at,
            created_by: s.created_by,
            tags: s.tags,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateScriptRequest {
    pub name: String,
    pub language: String,
    pub content: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateScriptRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecuteScriptRequest {
    pub agent_ids: Vec<String>,
    /// Exposed to the script as `CASS_ARG_<NAME>` environment variables.
    #[serde(default)]
    pub args: HashMap<String, Value>,
    #[serde(default)]
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScriptExecution {
    pub id: Uuid,
    pub script_id: Uuid,
    pub agent_id: String,
    /// pending | dispatched | completed | failed
    pub status: String,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ScriptExecution {
    fn from_job(job: AgentJob, script_id: Uuid) -> Self {
        Self {
            id: job.id,
            script_id,
            agent_id: job.agent_id,
            status: job.status.as_str().to_string(),
            exit_code: job.exit_code,
            stdout: job.stdout,
            stderr: job.stderr,
            error: job.error,
            started_at: job.dispatched_at.unwrap_or(job.created_at),
            completed_at: job.completed_at,
        }
    }
}

/// Platform-wide settings. `default_heartbeat_interval` seeds the config of
/// agents that have none; the rest are stored for operators.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlatformSettings {
    pub default_heartbeat_interval: u32,
    pub agent_timeout_seconds: u32,
    pub metrics_retention_days: u32,
    pub max_builds_per_project: u32,
    pub max_scripts_per_project: u32,
    pub enable_auto_updates: bool,
    pub webhook_url: Option<String>,
    pub notification_email: Option<String>,
}

impl Default for PlatformSettings {
    fn default() -> Self {
        Self {
            default_heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL_SECS,
            agent_timeout_seconds: 90,
            metrics_retention_days: 30,
            max_builds_per_project: 50,
            max_scripts_per_project: 100,
            enable_auto_updates: true,
            webhook_url: None,
            notification_email: None,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePlatformSettingsRequest {
    #[serde(default)]
    pub default_heartbeat_interval: Option<u32>,
    #[serde(default)]
    pub agent_timeout_seconds: Option<u32>,
    #[serde(default)]
    pub metrics_retention_days: Option<u32>,
    #[serde(default)]
    pub max_builds_per_project: Option<u32>,
    #[serde(default)]
    pub max_scripts_per_project: Option<u32>,
    #[serde(default)]
    pub enable_auto_updates: Option<bool>,
    #[serde(default)]
    pub webhook_url: Option<String>,
    #[serde(default)]
    pub notification_email: Option<String>,
}

// === Helpers ===

fn bad_request(message: &'static str) -> HttpError {
    HttpError::new(StatusCode::BAD_REQUEST, message)
}

fn not_found(message: &'static str) -> HttpError {
    HttpError::new(StatusCode::NOT_FOUND, message)
}

/// Who is making the request, for `created_by`/`uploaded_by` and audit trails.
fn actor(headers: &HeaderMap) -> Option<String> {
    let api = crate::auth::validate_api_key(headers);
    if api.allowed {
        return api.subject.or_else(|| Some("api_key".to_string()));
    }
    crate::auth::validate_jwt(headers).subject
}

fn ensure_agent_known(state: &AppState, agent_id: &str) -> Result<(), HttpError> {
    if state.registry.list().iter().any(|a| a.id == agent_id) {
        Ok(())
    } else {
        Err(not_found("agent not found"))
    }
}

fn clamp_timeout(requested: Option<u32>) -> Result<u32, HttpError> {
    match requested {
        None => Ok(DEFAULT_JOB_TIMEOUT_SECS),
        Some(t) if (1..=MAX_JOB_TIMEOUT_SECS).contains(&t) => Ok(t),
        Some(_) => Err(bad_request("timeout_seconds must be between 1 and 3600")),
    }
}

/// Drop any directory part and anything unusual from an uploaded file name.
fn sanitize_filename(raw: Option<&str>) -> String {
    let base = raw
        .and_then(|n| n.rsplit(['/', '\\']).next())
        .unwrap_or_default();
    let cleaned: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .take(128)
        .collect();
    match cleaned.trim_matches('.') {
        "" => "build.bin".to_string(),
        name => name.to_string(),
    }
}

async fn stored_settings(state: &AppState) -> Result<PlatformSettings, HttpError> {
    let stored = state
        .portal_store
        .get_settings()
        .await
        .map_err(HttpError::from)?;
    Ok(stored
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

/// The agent's stored config, or defaults seeded from platform settings.
pub(crate) async fn effective_agent_config(
    state: &AppState,
    agent_id: &str,
) -> Result<AgentConfig, HttpError> {
    let stored = state
        .portal_store
        .get_agent_config(agent_id)
        .await
        .map_err(HttpError::from)?;
    if let Some(config) = stored.and_then(|v| serde_json::from_value(v).ok()) {
        return Ok(config);
    }
    Ok(AgentConfig {
        heartbeat_interval_seconds: stored_settings(state).await?.default_heartbeat_interval,
        ..AgentConfig::default()
    })
}

fn validate_heartbeat_interval(seconds: u32) -> Result<(), HttpError> {
    if (5..=3600).contains(&seconds) {
        Ok(())
    } else {
        Err(bad_request(
            "heartbeat interval must be between 5 and 3600 seconds",
        ))
    }
}

// === Agent config & commands ===

#[utoipa::path(
    get,
    path = "/agents/{agent_id}/config",
    tag = "portal",
    params(("agent_id" = String, Path, description = "Agent identifier")),
    responses(
        (status = 200, body = AgentConfig, description = "Agent configuration"),
        (status = 404, description = "Agent not found")
    )
)]
pub async fn get_agent_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentConfig>, HttpError> {
    ensure_scope(&headers, "agent:read")?;
    ensure_agent_known(&state, &agent_id)?;
    Ok(Json(effective_agent_config(&state, &agent_id).await?))
}

#[utoipa::path(
    put,
    path = "/agents/{agent_id}/config",
    tag = "portal",
    params(("agent_id" = String, Path, description = "Agent identifier")),
    request_body = UpdateAgentConfigRequest,
    responses(
        (status = 200, body = AgentConfig, description = "Updated configuration"),
        (status = 404, description = "Agent not found")
    )
)]
pub async fn update_agent_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
    Json(req): Json<UpdateAgentConfigRequest>,
) -> Result<Json<AgentConfig>, HttpError> {
    ensure_scope(&headers, "agent:write")?;
    ensure_agent_known(&state, &agent_id)?;

    let mut config = effective_agent_config(&state, &agent_id).await?;
    if let Some(v) = req.heartbeat_interval_seconds {
        validate_heartbeat_interval(v)?;
        config.heartbeat_interval_seconds = v;
    }
    if let Some(v) = req.log_level {
        config.log_level = v;
    }
    if let Some(v) = req.auto_update_enabled {
        config.auto_update_enabled = v;
    }
    if let Some(v) = req.max_concurrent_tasks {
        config.max_concurrent_tasks = v;
    }
    if let Some(v) = req.metrics_enabled {
        config.metrics_enabled = v;
    }
    if let Some(v) = req.custom_labels {
        config.custom_labels = v;
    }

    let value = serde_json::to_value(&config).map_err(|_| bad_request("invalid config"))?;
    state
        .portal_store
        .put_agent_config(&agent_id, value)
        .await
        .map_err(HttpError::from)?;
    state.audit.emit(AuditEvent::info(
        "agent.config.updated",
        json!({ "agent_id": agent_id, "config": config, "actor": actor(&headers) }),
    ));
    Ok(Json(config))
}

#[utoipa::path(
    get,
    path = "/agents/{agent_id}/commands",
    tag = "portal",
    params(("agent_id" = String, Path, description = "Agent identifier")),
    responses((status = 200, body = [AgentCommand], description = "Newest first, at most 100"))
)]
pub async fn list_agent_commands(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
) -> Result<Json<Vec<AgentCommand>>, HttpError> {
    ensure_scope(&headers, "agent:read")?;
    let jobs = state
        .portal_store
        .list_jobs(JobQuery {
            agent_id: Some(agent_id),
            limit: Some(MAX_LISTED_JOBS),
            ..Default::default()
        })
        .await
        .map_err(HttpError::from)?;
    Ok(Json(jobs.into_iter().map(AgentCommand::from).collect()))
}

#[utoipa::path(
    post,
    path = "/agents/{agent_id}/commands",
    tag = "portal",
    params(("agent_id" = String, Path, description = "Agent identifier")),
    request_body = SendAgentCommandRequest,
    responses(
        (status = 201, body = AgentCommand, description = "Queued for the agent's next heartbeat"),
        (status = 400, description = "Unknown command or invalid arguments"),
        (status = 404, description = "Agent or build not found")
    )
)]
pub async fn send_agent_command(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
    Json(req): Json<SendAgentCommandRequest>,
) -> Result<(StatusCode, Json<AgentCommand>), HttpError> {
    ensure_scope(&headers, "agent:write")?;
    ensure_agent_known(&state, &agent_id)?;
    if !AGENT_COMMANDS.contains(&req.command.as_str()) {
        return Err(bad_request(
            "command must be one of restart, shutdown, exec, collect_logs, deploy_build",
        ));
    }

    let mut timeout = DEFAULT_JOB_TIMEOUT_SECS;
    match req.command.as_str() {
        "exec" => {
            let command = req
                .args
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or("");
            if command.trim().is_empty() {
                return Err(bad_request("exec requires args.command"));
            }
            let requested = req
                .args
                .get("timeout_seconds")
                .and_then(Value::as_u64)
                .map(|t| u32::try_from(t).unwrap_or(u32::MAX));
            timeout = clamp_timeout(requested)?;
        }
        "deploy_build" => {
            let build_id = req
                .args
                .get("build_id")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| bad_request("deploy_build requires args.build_id"))?;
            if state
                .portal_store
                .get_build(build_id)
                .await
                .map_err(HttpError::from)?
                .is_none()
            {
                return Err(not_found("build not found"));
            }
        }
        _ => {}
    }

    let mut job = AgentJob::new(&agent_id, &req.command, json!(req.args));
    job.timeout_seconds = timeout;
    job.created_by = actor(&headers);
    state
        .portal_store
        .insert_jobs(vec![job.clone()])
        .await
        .map_err(HttpError::from)?;
    state.audit.emit(AuditEvent::info(
        "agent.command.sent",
        json!({
            "agent_id": agent_id,
            "command_id": job.id,
            "command": req.command,
            "actor": job.created_by,
        }),
    ));
    Ok((StatusCode::CREATED, Json(job.into())))
}

// === Builds & deployments ===

#[utoipa::path(
    get,
    path = "/builds",
    tag = "portal",
    responses((status = 200, body = [Build], description = "Builds, newest first"))
)]
pub async fn list_builds(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Build>>, HttpError> {
    ensure_scope(&headers, "build:read")?;
    let builds = state
        .portal_store
        .list_builds()
        .await
        .map_err(HttpError::from)?;
    Ok(Json(builds.into_iter().map(Build::from).collect()))
}

/// Stream one multipart field to `path`, returning (size, sha256 hex).
async fn write_upload(
    mut field: axum::extract::multipart::Field<'_>,
    path: &std::path::Path,
) -> Result<(u64, String), HttpError> {
    let mut file = tokio::fs::File::create(path).await.map_err(|err| {
        tracing::error!(error = %err, path = %path.display(), "build.upload.create_failed");
        HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "could not store build")
    })?;
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|err| HttpError::new(err.status(), "upload interrupted"))?
    {
        hasher.update(&chunk);
        size += chunk.len() as u64;
        file.write_all(&chunk).await.map_err(|err| {
            tracing::error!(error = %err, "build.upload.write_failed");
            HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "could not store build")
        })?;
    }
    file.flush()
        .await
        .map_err(|_| HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "could not store build"))?;
    Ok((size, hex::encode(hasher.finalize())))
}

#[utoipa::path(
    post,
    path = "/builds",
    tag = "portal",
    request_body(content = BuildUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 201, body = Build, description = "Build stored"),
        (status = 400, description = "Missing fields or file"),
        (status = 413, description = "Larger than CASS_MAX_BUILD_BYTES")
    )
)]
pub async fn create_build(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Build>), HttpError> {
    ensure_scope(&headers, "build:write")?;

    let id = Uuid::new_v4();
    let final_path = state.build_dir.join(id.to_string());
    // Written under a temporary name so a partial upload is never served.
    let part_path = state.build_dir.join(format!("{id}.part"));

    let result = async {
        let mut fields: HashMap<String, String> = HashMap::new();
        let mut upload: Option<(String, u64, String)> = None;
        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|err| HttpError::new(err.status(), "invalid multipart body"))?
        {
            let name = field.name().unwrap_or_default().to_string();
            if name == "file" {
                if upload.is_some() {
                    return Err(bad_request("only one file per build"));
                }
                let filename = sanitize_filename(field.file_name());
                let (size, checksum) = write_upload(field, &part_path).await?;
                upload = Some((filename, size, checksum));
            } else {
                let text = field
                    .text()
                    .await
                    .map_err(|err| HttpError::new(err.status(), "invalid multipart field"))?;
                fields.insert(name, text.trim().to_string());
            }
        }

        let (filename, size_bytes, checksum_sha256) =
            upload.ok_or_else(|| bad_request("file is required"))?;
        if size_bytes == 0 {
            return Err(bad_request("file is empty"));
        }
        let mut take = |key: &str| fields.remove(key).filter(|v| !v.is_empty());
        let (Some(name), Some(version), Some(platform), Some(architecture)) = (
            take("name"),
            take("version"),
            take("platform"),
            take("architecture"),
        ) else {
            return Err(bad_request(
                "name, version, platform and architecture are required",
            ));
        };

        tokio::fs::rename(&part_path, &final_path)
            .await
            .map_err(|err| {
                tracing::error!(error = %err, "build.upload.rename_failed");
                HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "could not store build")
            })?;
        let record = BuildRecord {
            id,
            name,
            version,
            platform,
            architecture,
            filename,
            size_bytes,
            checksum_sha256,
            uploaded_by: actor(&headers),
            metadata: fields,
            created_at: Utc::now(),
        };
        state
            .portal_store
            .insert_build(record.clone())
            .await
            .map_err(HttpError::from)?;
        Ok(record)
    }
    .await;

    match result {
        Ok(record) => {
            state.audit.emit(AuditEvent::info(
                "build.created",
                json!({
                    "build_id": record.id,
                    "name": record.name,
                    "version": record.version,
                    "size_bytes": record.size_bytes,
                    "checksum_sha256": record.checksum_sha256,
                    "actor": record.uploaded_by,
                }),
            ));
            Ok((StatusCode::CREATED, Json(record.into())))
        }
        Err(err) => {
            let _ = tokio::fs::remove_file(&part_path).await;
            let _ = tokio::fs::remove_file(&final_path).await;
            Err(err)
        }
    }
}

#[utoipa::path(
    get,
    path = "/builds/{build_id}",
    tag = "portal",
    params(("build_id" = Uuid, Path, description = "Build identifier")),
    responses(
        (status = 200, body = Build, description = "Build details"),
        (status = 404, description = "Build not found")
    )
)]
pub async fn get_build(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(build_id): Path<Uuid>,
) -> Result<Json<Build>, HttpError> {
    ensure_scope(&headers, "build:read")?;
    state
        .portal_store
        .get_build(build_id)
        .await
        .map_err(HttpError::from)?
        .map(|b| Json(b.into()))
        .ok_or_else(|| not_found("build not found"))
}

#[utoipa::path(
    delete,
    path = "/builds/{build_id}",
    tag = "portal",
    params(("build_id" = Uuid, Path, description = "Build identifier")),
    responses(
        (status = 204, description = "Build and its file deleted"),
        (status = 404, description = "Build not found")
    )
)]
pub async fn delete_build(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(build_id): Path<Uuid>,
) -> Result<StatusCode, HttpError> {
    ensure_scope(&headers, "build:write")?;
    if !state
        .portal_store
        .delete_build(build_id)
        .await
        .map_err(HttpError::from)?
    {
        return Err(not_found("build not found"));
    }
    if let Err(err) = tokio::fs::remove_file(state.build_dir.join(build_id.to_string())).await {
        if err.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(error = %err, %build_id, "build.delete.file_remove_failed");
        }
    }
    state.audit.emit(AuditEvent::warn(
        "build.deleted",
        json!({ "build_id": build_id, "actor": actor(&headers) }),
    ));
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/deployments",
    tag = "portal",
    request_body = DeployBuildRequest,
    responses(
        (status = 201, body = DeploymentResponse, description = "deploy_build queued for each agent"),
        (status = 404, description = "Build or agent not found")
    )
)]
pub async fn create_deployment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<DeployBuildRequest>,
) -> Result<(StatusCode, Json<DeploymentResponse>), HttpError> {
    ensure_scope(&headers, "build:deploy")?;
    if req.agent_ids.is_empty() {
        return Err(bad_request("agent_ids must not be empty"));
    }
    if state
        .portal_store
        .get_build(req.build_id)
        .await
        .map_err(HttpError::from)?
        .is_none()
    {
        return Err(not_found("build not found"));
    }
    for agent_id in &req.agent_ids {
        ensure_agent_known(&state, agent_id)?;
    }

    let deployment_id = Uuid::new_v4();
    let created_by = actor(&headers);
    let jobs: Vec<AgentJob> = req
        .agent_ids
        .iter()
        .map(|agent_id| {
            let mut job = AgentJob::new(
                agent_id,
                "deploy_build",
                json!({ "build_id": req.build_id, "deployment_id": deployment_id }),
            );
            job.created_by = created_by.clone();
            job
        })
        .collect();
    let created_at = jobs[0].created_at;
    state
        .portal_store
        .insert_jobs(jobs)
        .await
        .map_err(HttpError::from)?;
    state.audit.emit(AuditEvent::info(
        "build.deployment.created",
        json!({
            "deployment_id": deployment_id,
            "build_id": req.build_id,
            "agent_ids": req.agent_ids,
            "actor": created_by,
        }),
    ));
    Ok((
        StatusCode::CREATED,
        Json(DeploymentResponse {
            deployment_id,
            build_id: req.build_id,
            agent_ids: req.agent_ids,
            status: "pending".to_string(),
            created_at,
        }),
    ))
}

// === Scripts ===

fn validate_script_fields(name: &str, language: &str, content: &str) -> Result<(), HttpError> {
    if name.trim().is_empty() {
        return Err(bad_request("name is required"));
    }
    if !SCRIPT_LANGUAGES.contains(&language) {
        return Err(bad_request(
            "language must be one of bash, sh, powershell, python, javascript, lua",
        ));
    }
    if content.trim().is_empty() {
        return Err(bad_request("content is required"));
    }
    if content.len() > MAX_SCRIPT_BYTES {
        return Err(bad_request("content is larger than 256 KiB"));
    }
    Ok(())
}

async fn load_script(state: &AppState, id: Uuid) -> Result<ScriptRecord, HttpError> {
    state
        .portal_store
        .get_script(id)
        .await
        .map_err(HttpError::from)?
        .ok_or_else(|| not_found("script not found"))
}

#[utoipa::path(
    get,
    path = "/scripts",
    tag = "portal",
    responses((status = 200, body = [Script], description = "Scripts by name"))
)]
pub async fn list_scripts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Script>>, HttpError> {
    ensure_scope(&headers, "script:read")?;
    let scripts = state
        .portal_store
        .list_scripts()
        .await
        .map_err(HttpError::from)?;
    Ok(Json(scripts.into_iter().map(Script::from).collect()))
}

#[utoipa::path(
    post,
    path = "/scripts",
    tag = "portal",
    request_body = CreateScriptRequest,
    responses(
        (status = 201, body = Script, description = "Script created"),
        (status = 400, description = "Invalid script")
    )
)]
pub async fn create_script(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateScriptRequest>,
) -> Result<(StatusCode, Json<Script>), HttpError> {
    ensure_scope(&headers, "script:write")?;
    validate_script_fields(&req.name, &req.language, &req.content)?;
    let now = Utc::now();
    let script = ScriptRecord {
        id: Uuid::new_v4(),
        name: req.name.trim().to_string(),
        language: req.language,
        content: req.content,
        description: req.description,
        tags: req.tags,
        version: 1,
        created_by: actor(&headers),
        created_at: now,
        updated_at: now,
    };
    state
        .portal_store
        .save_script(script.clone())
        .await
        .map_err(HttpError::from)?;
    state.audit.emit(AuditEvent::info(
        "script.created",
        json!({ "script_id": script.id, "name": script.name, "actor": script.created_by }),
    ));
    Ok((StatusCode::CREATED, Json(script.into())))
}

#[utoipa::path(
    get,
    path = "/scripts/{script_id}",
    tag = "portal",
    params(("script_id" = Uuid, Path, description = "Script identifier")),
    responses(
        (status = 200, body = Script, description = "Script details"),
        (status = 404, description = "Script not found")
    )
)]
pub async fn get_script(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(script_id): Path<Uuid>,
) -> Result<Json<Script>, HttpError> {
    ensure_scope(&headers, "script:read")?;
    Ok(Json(load_script(&state, script_id).await?.into()))
}

#[utoipa::path(
    put,
    path = "/scripts/{script_id}",
    tag = "portal",
    params(("script_id" = Uuid, Path, description = "Script identifier")),
    request_body = UpdateScriptRequest,
    responses(
        (status = 200, body = Script, description = "Script updated; version incremented"),
        (status = 404, description = "Script not found")
    )
)]
pub async fn update_script(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(script_id): Path<Uuid>,
    Json(req): Json<UpdateScriptRequest>,
) -> Result<Json<Script>, HttpError> {
    ensure_scope(&headers, "script:write")?;
    let mut script = load_script(&state, script_id).await?;
    if let Some(v) = req.name {
        script.name = v.trim().to_string();
    }
    if let Some(v) = req.language {
        script.language = v;
    }
    if let Some(v) = req.content {
        script.content = v;
    }
    if let Some(v) = req.description {
        script.description = Some(v);
    }
    if let Some(v) = req.tags {
        script.tags = v;
    }
    validate_script_fields(&script.name, &script.language, &script.content)?;
    script.version += 1;
    script.updated_at = Utc::now();
    state
        .portal_store
        .save_script(script.clone())
        .await
        .map_err(HttpError::from)?;
    state.audit.emit(AuditEvent::info(
        "script.updated",
        json!({ "script_id": script.id, "version": script.version, "actor": actor(&headers) }),
    ));
    Ok(Json(script.into()))
}

#[utoipa::path(
    delete,
    path = "/scripts/{script_id}",
    tag = "portal",
    params(("script_id" = Uuid, Path, description = "Script identifier")),
    responses(
        (status = 204, description = "Script deleted"),
        (status = 404, description = "Script not found")
    )
)]
pub async fn delete_script(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(script_id): Path<Uuid>,
) -> Result<StatusCode, HttpError> {
    ensure_scope(&headers, "script:write")?;
    if !state
        .portal_store
        .delete_script(script_id)
        .await
        .map_err(HttpError::from)?
    {
        return Err(not_found("script not found"));
    }
    state.audit.emit(AuditEvent::warn(
        "script.deleted",
        json!({ "script_id": script_id, "actor": actor(&headers) }),
    ));
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/scripts/{script_id}/execute",
    tag = "portal",
    params(("script_id" = Uuid, Path, description = "Script identifier")),
    request_body = ExecuteScriptRequest,
    responses(
        (status = 201, body = [ScriptExecution], description = "Queued for each agent"),
        (status = 404, description = "Script or agent not found")
    )
)]
pub async fn execute_script(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(script_id): Path<Uuid>,
    Json(req): Json<ExecuteScriptRequest>,
) -> Result<(StatusCode, Json<Vec<ScriptExecution>>), HttpError> {
    ensure_scope(&headers, "script:execute")?;
    if req.agent_ids.is_empty() {
        return Err(bad_request("agent_ids must not be empty"));
    }
    let timeout = clamp_timeout(req.timeout_seconds)?;
    let script = load_script(&state, script_id).await?;
    for agent_id in &req.agent_ids {
        ensure_agent_known(&state, agent_id)?;
    }

    let created_by = actor(&headers);
    // The job carries a snapshot of the script, so later edits don't change
    // what an already-queued run executes.
    let jobs: Vec<AgentJob> = req
        .agent_ids
        .iter()
        .map(|agent_id| {
            let mut job = AgentJob::new(
                agent_id,
                "script",
                json!({
                    "script_id": script.id,
                    "script_version": script.version,
                    "language": script.language,
                    "content": script.content,
                    "args": req.args,
                }),
            );
            job.script_id = Some(script.id);
            job.timeout_seconds = timeout;
            job.created_by = created_by.clone();
            job
        })
        .collect();
    state
        .portal_store
        .insert_jobs(jobs.clone())
        .await
        .map_err(HttpError::from)?;
    state.audit.emit(AuditEvent::info(
        "script.execute",
        json!({
            "script_id": script.id,
            "script_version": script.version,
            "agent_ids": req.agent_ids,
            "timeout_seconds": timeout,
            "actor": created_by,
        }),
    ));
    Ok((
        StatusCode::CREATED,
        Json(
            jobs.into_iter()
                .map(|job| ScriptExecution::from_job(job, script_id))
                .collect(),
        ),
    ))
}

#[utoipa::path(
    get,
    path = "/scripts/{script_id}/executions",
    tag = "portal",
    params(("script_id" = Uuid, Path, description = "Script identifier")),
    responses((status = 200, body = [ScriptExecution], description = "Newest first, at most 100"))
)]
pub async fn list_script_executions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(script_id): Path<Uuid>,
) -> Result<Json<Vec<ScriptExecution>>, HttpError> {
    ensure_scope(&headers, "script:read")?;
    let jobs = state
        .portal_store
        .list_jobs(JobQuery {
            script_id: Some(script_id),
            limit: Some(MAX_LISTED_JOBS),
            ..Default::default()
        })
        .await
        .map_err(HttpError::from)?;
    Ok(Json(
        jobs.into_iter()
            .map(|job| ScriptExecution::from_job(job, script_id))
            .collect(),
    ))
}

// === Platform settings ===

#[utoipa::path(
    get,
    path = "/settings",
    tag = "portal",
    responses((status = 200, body = PlatformSettings, description = "Platform settings"))
)]
pub async fn get_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PlatformSettings>, HttpError> {
    ensure_scope(&headers, "admin:read")?;
    Ok(Json(stored_settings(&state).await?))
}

#[utoipa::path(
    put,
    path = "/settings",
    tag = "portal",
    request_body = UpdatePlatformSettingsRequest,
    responses((status = 200, body = PlatformSettings, description = "Updated settings"))
)]
pub async fn update_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdatePlatformSettingsRequest>,
) -> Result<Json<PlatformSettings>, HttpError> {
    ensure_scope(&headers, "admin:write")?;
    let mut settings = stored_settings(&state).await?;
    if let Some(v) = req.default_heartbeat_interval {
        validate_heartbeat_interval(v)?;
        settings.default_heartbeat_interval = v;
    }
    if let Some(v) = req.agent_timeout_seconds {
        settings.agent_timeout_seconds = v;
    }
    if let Some(v) = req.metrics_retention_days {
        settings.metrics_retention_days = v;
    }
    if let Some(v) = req.max_builds_per_project {
        settings.max_builds_per_project = v;
    }
    if let Some(v) = req.max_scripts_per_project {
        settings.max_scripts_per_project = v;
    }
    if let Some(v) = req.enable_auto_updates {
        settings.enable_auto_updates = v;
    }
    if let Some(v) = req.webhook_url {
        settings.webhook_url = Some(v).filter(|s| !s.trim().is_empty());
    }
    if let Some(v) = req.notification_email {
        settings.notification_email = Some(v).filter(|s| !s.trim().is_empty());
    }
    let value = serde_json::to_value(&settings).map_err(|_| bad_request("invalid settings"))?;
    state
        .portal_store
        .put_settings(value)
        .await
        .map_err(HttpError::from)?;
    state.audit.emit(AuditEvent::info(
        "settings.updated",
        json!({ "settings": settings, "actor": actor(&headers) }),
    ));
    Ok(Json(settings))
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;

    #[test]
    fn upload_filenames_cannot_escape_the_build_dir() {
        assert_eq!(sanitize_filename(Some("../../etc/passwd")), "passwd");
        assert_eq!(
            sanitize_filename(Some("C:\\games\\server.zip")),
            "server.zip"
        );
        assert_eq!(sanitize_filename(Some("..")), "build.bin");
        assert_eq!(sanitize_filename(Some("a b$c.tar.gz")), "abc.tar.gz");
        assert_eq!(sanitize_filename(None), "build.bin");
    }
}
