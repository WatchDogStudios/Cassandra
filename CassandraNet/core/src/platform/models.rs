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
