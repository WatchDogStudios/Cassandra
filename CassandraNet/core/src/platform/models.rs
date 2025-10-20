use crate::platform::error::PlatformError;
use chrono::{DateTime, Utc};
use cncommon::auth::Scope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub type TenantId = Uuid;
pub type ProjectId = Uuid;
pub type AgentId = Uuid;
pub type ApiKeyId = Uuid;
pub type TaskId = Uuid;
pub type WorkflowId = Uuid;
pub type ContentId = Uuid;
pub type UploadId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub settings: TenantSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub id: ProjectId,
    pub tenant_id: TenantId,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProjectStorageSettings {
    pub bucket: Option<String>,
    pub prefix: Option<String>,
    pub max_object_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Registered,
    Active,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Agent {
    pub id: AgentId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub hostname: String,
    pub status: AgentStatus,
    pub last_seen: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub metadata: AgentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiKeyRecord {
    pub id: ApiKeyId,
    pub tenant_id: TenantId,
    pub label: String,
    pub scopes: Vec<Scope>,
    pub token_prefix: String,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub rotated_from: Option<ApiKeyId>,
    pub rotated_to: Option<ApiKeyId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiKey {
    pub id: ApiKeyId,
    pub value: String,
    pub tenant_id: TenantId,
    pub label: String,
    pub scopes: Vec<Scope>,
    pub created_at: DateTime<Utc>,
    pub rotation_parent: Option<ApiKeyId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvisionedAgent {
    pub agent: Agent,
    pub api_key: ApiKey,
    pub bootstrap_commands: Vec<String>,
    pub certificate_bundle: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrincipalType {
    Tenant,
    Agent,
    Service,
    ServiceAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthContext {
    pub principal_id: Uuid,
    pub principal_type: PrincipalType,
    pub tenant_id: TenantId,
    pub scopes: Vec<Scope>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub audience: Option<String>,
    pub issuer: Option<String>,
    pub session: Option<AuthSessionMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthToken {
    pub token: String,
    pub context: AuthContext,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AuthSessionMetadata {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TenantSettings {
    pub allowed_origins: Vec<String>,
    pub token_ttl_seconds: Option<i64>,
    pub refresh_token_ttl_seconds: Option<i64>,
    pub default_storage: Option<ProjectStorageSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AgentMetadata {
    pub capabilities: Vec<String>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: TaskId,
    pub tenant_id: TenantId,
    pub kind: String,
    pub payload: serde_json::Value,
    pub status: TaskStatus,
    pub attempts: u32,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub result: Option<serde_json::Value>,
    pub timeouts: Option<TaskTimeouts>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskRequest {
    pub tenant_id: TenantId,
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskLease {
    pub task: Task,
    pub worker_id: Uuid,
    pub leased_at: DateTime<Utc>,
    pub lease_expires_at: DateTime<Utc>,
    pub lease_version: u64,
    pub lease_token: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub name: String,
    pub task_kind: String,
    pub dependencies: Vec<TaskDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Workflow {
    pub id: WorkflowId,
    pub tenant_id: TenantId,
    pub name: String,
    pub steps: Vec<WorkflowStep>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowRun {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub workflow_id: WorkflowId,
    pub status: WorkflowRunStatus,
    pub current_step: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskDependency {
    pub task_kind: String,
    pub required_status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskTimeouts {
    pub lease_seconds: Option<u64>,
    pub execution_seconds: Option<u64>,
    pub retry_backoff_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UploadStatus {
    Pending,
    Uploading,
    Completed,
    Failed,
    Cancelled,
}

impl UploadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UploadStatus::Pending => "pending",
            UploadStatus::Uploading => "uploading",
            UploadStatus::Completed => "completed",
            UploadStatus::Failed => "failed",
            UploadStatus::Cancelled => "cancelled",
        }
    }
}

impl std::str::FromStr for UploadStatus {
    type Err = PlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(UploadStatus::Pending),
            "uploading" => Ok(UploadStatus::Uploading),
            "completed" => Ok(UploadStatus::Completed),
            "failed" => Ok(UploadStatus::Failed),
            "cancelled" => Ok(UploadStatus::Cancelled),
            _ => Err(PlatformError::InvalidInput("invalid upload status")),
        }
    }
}

impl From<UploadStatus> for &'static str {
    fn from(value: UploadStatus) -> Self {
        value.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UploadSession {
    pub id: UploadId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub content_id: ContentId,
    pub status: UploadStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub upload_url: Option<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentMetadata {
    pub id: ContentId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentVisibility {
    Private,
    Project,
    Tenant,
    Public,
}

impl ContentVisibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentVisibility::Private => "private",
            ContentVisibility::Project => "project",
            ContentVisibility::Tenant => "tenant",
            ContentVisibility::Public => "public",
        }
    }
}

impl std::str::FromStr for ContentVisibility {
    type Err = PlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "private" => Ok(ContentVisibility::Private),
            "project" => Ok(ContentVisibility::Project),
            "tenant" => Ok(ContentVisibility::Tenant),
            "public" => Ok(ContentVisibility::Public),
            _ => Err(PlatformError::InvalidInput("invalid content visibility")),
        }
    }
}

impl From<ContentVisibility> for &'static str {
    fn from(value: ContentVisibility) -> Self {
        value.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContentQuery {
    pub tenant_id: TenantId,
    pub project_id: Option<ProjectId>,
    pub search_term: Option<String>,
    pub tags: Vec<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
