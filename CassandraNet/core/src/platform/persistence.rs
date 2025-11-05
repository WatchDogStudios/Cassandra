use crate::platform::error::{PlatformError, PlatformResult};
use crate::platform::models::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
#[cfg(feature = "db")]
use sqlx::{postgres::PgRow, Pool, Postgres, QueryBuilder, Row};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::sync::Arc;

pub trait TenantStore: Send + Sync {
    fn insert_tenant(&self, tenant: Tenant) -> PlatformResult<()>;
    fn get_tenant(&self, id: TenantId) -> PlatformResult<Option<Tenant>>;
    fn list_tenants(&self) -> PlatformResult<Vec<Tenant>>;
}

pub trait ProjectStore: Send + Sync {
    fn insert_project(&self, project: Project) -> PlatformResult<()>;
    fn list_projects(&self, tenant_id: TenantId) -> PlatformResult<Vec<Project>>;
    fn get_project(&self, id: ProjectId) -> PlatformResult<Option<Project>>;
}

pub trait AgentStore: Send + Sync {
    fn insert_agent(&self, agent: Agent) -> PlatformResult<()>;
    fn update_agent(&self, agent: Agent) -> PlatformResult<()>;
    fn list_agents(&self, tenant_id: TenantId) -> PlatformResult<Vec<Agent>>;
    fn get_agent(&self, id: AgentId) -> PlatformResult<Option<Agent>>;
}

pub trait ApiKeyStore: Send + Sync {
    fn insert_api_key(&self, record: ApiKeyRecord) -> PlatformResult<()>;
    fn get_api_key(&self, id: ApiKeyId) -> PlatformResult<Option<ApiKeyRecord>>;
    fn get_api_key_by_prefix(&self, prefix: &str) -> PlatformResult<Option<ApiKeyRecord>>;
    fn list_api_keys(&self, tenant_id: TenantId) -> PlatformResult<Vec<ApiKeyRecord>>;
    fn update_api_key(&self, record: ApiKeyRecord) -> PlatformResult<()>;
}

pub trait TaskStore: Send + Sync {
    fn enqueue_task(&self, task: Task) -> PlatformResult<()>;
    fn peek_next_task(&self, tenant_id: TenantId) -> PlatformResult<Option<Task>>;
    fn update_task(&self, task: Task) -> PlatformResult<()>;
    fn get_task(&self, id: TaskId) -> PlatformResult<Option<Task>>;
    fn list_pending_tasks(&self, tenant_id: TenantId) -> PlatformResult<Vec<Task>>;
}

pub trait WorkflowStore: Send + Sync {
    fn insert_workflow(&self, workflow: Workflow) -> PlatformResult<()>;
    fn get_workflow(&self, id: WorkflowId) -> PlatformResult<Option<Workflow>>;
    fn list_workflows(&self, tenant_id: TenantId) -> PlatformResult<Vec<Workflow>>;
}

#[async_trait]
pub trait ContentStore: Send + Sync {
    async fn create_upload_session(&self, session: UploadSession) -> PlatformResult<()>;
    async fn update_upload_session(&self, session: UploadSession) -> PlatformResult<()>;
    async fn get_upload_session(&self, id: UploadId) -> PlatformResult<Option<UploadSession>>;
    async fn record_content_metadata(&self, metadata: ContentMetadata) -> PlatformResult<()>;
    async fn get_content_metadata(&self, id: ContentId) -> PlatformResult<Option<ContentMetadata>>;
    async fn list_content_metadata(
        &self,
        query: &ContentQuery,
    ) -> PlatformResult<Vec<ContentMetadata>>;
}

#[async_trait]
pub trait OrchestrationStore: Send + Sync {
    async fn create_assignment(&self, input: NewAssignment) -> PlatformResult<WorkAssignment>;
    async fn update_assignment_status(
        &self,
        id: AssignmentId,
        status: WorkStatus,
        status_message: Option<String>,
    ) -> PlatformResult<WorkAssignment>;
    async fn list_assignments(&self, query: AssignmentQuery)
        -> PlatformResult<Vec<WorkAssignment>>;
}

#[async_trait]
pub trait ModerationStore: Send + Sync {
    async fn create_content(&self, input: NewModeratedContent) -> PlatformResult<ModeratedContent>;
    async fn update_content_state(
        &self,
        id: ContentId,
        state: ModerationState,
        reason: Option<String>,
    ) -> PlatformResult<ModeratedContent>;
    async fn list_content(&self, query: ModerationQuery) -> PlatformResult<Vec<ModeratedContent>>;
}

#[async_trait]
pub trait MessagingStore: Send + Sync {
    async fn enqueue_message(&self, input: NewMessageRecord) -> PlatformResult<MessageRecord>;
    async fn list_messages(&self, query: MessageQuery) -> PlatformResult<Vec<MessageRecord>>;
    async fn ack_message(&self, topic: &str, id: MessageId) -> PlatformResult<()>;
}

#[derive(Default)]
struct PlatformState {
    tenants: HashMap<TenantId, Tenant>,
    projects: HashMap<ProjectId, Project>,
    agents: HashMap<AgentId, Agent>,
    api_keys: HashMap<ApiKeyId, ApiKeyRecord>,
    api_keys_by_prefix: HashMap<String, ApiKeyId>,
    tasks: HashMap<TaskId, Task>,
    task_queue: VecDeque<TaskId>,
    workflows: HashMap<WorkflowId, Workflow>,
    upload_sessions: HashMap<UploadId, UploadSession>,
    content_metadata: HashMap<ContentId, ContentMetadata>,
    assignments: HashMap<AssignmentId, WorkAssignment>,
    moderation_content: HashMap<ContentId, ModeratedContent>,
    messages: HashMap<MessageId, MessageRecord>,
    messages_by_topic: HashMap<String, VecDeque<MessageId>>,
}

#[derive(Clone, Default)]
pub struct InMemoryPersistence {
    state: Arc<RwLock<PlatformState>>,
}

impl InMemoryPersistence {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TenantStore for InMemoryPersistence {
    fn insert_tenant(&self, tenant: Tenant) -> PlatformResult<()> {
        let mut state = self.state.write();
        if state.tenants.contains_key(&tenant.id) {
            return Err(PlatformError::Conflict("tenant"));
        }
        state.tenants.insert(tenant.id, tenant);
        Ok(())
    }

    fn get_tenant(&self, id: TenantId) -> PlatformResult<Option<Tenant>> {
        Ok(self.state.read().tenants.get(&id).cloned())
    }

    fn list_tenants(&self) -> PlatformResult<Vec<Tenant>> {
        let mut tenants: Vec<_> = self.state.read().tenants.values().cloned().collect();
        tenants.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tenants)
    }
}

impl ProjectStore for InMemoryPersistence {
    fn insert_project(&self, project: Project) -> PlatformResult<()> {
        let mut state = self.state.write();
        if !state.tenants.contains_key(&project.tenant_id) {
            return Err(PlatformError::NotFound("tenant"));
        }
        if state.projects.contains_key(&project.id) {
            return Err(PlatformError::Conflict("project"));
        }
        state.projects.insert(project.id, project);
        Ok(())
    }

    fn list_projects(&self, tenant_id: TenantId) -> PlatformResult<Vec<Project>> {
        let mut projects: Vec<_> = self
            .state
            .read()
            .projects
            .values()
            .filter(|p| p.tenant_id == tenant_id)
            .cloned()
            .collect();
        projects.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(projects)
    }

    fn get_project(&self, id: ProjectId) -> PlatformResult<Option<Project>> {
        Ok(self.state.read().projects.get(&id).cloned())
    }
}

impl AgentStore for InMemoryPersistence {
    fn insert_agent(&self, agent: Agent) -> PlatformResult<()> {
        let mut state = self.state.write();
        if !state.tenants.contains_key(&agent.tenant_id) {
            return Err(PlatformError::NotFound("tenant"));
        }
        if !state.projects.contains_key(&agent.project_id) {
            return Err(PlatformError::NotFound("project"));
        }
        if state.agents.contains_key(&agent.id) {
            return Err(PlatformError::Conflict("agent"));
        }
        state.agents.insert(agent.id, agent);
        Ok(())
    }

    fn update_agent(&self, agent: Agent) -> PlatformResult<()> {
        let mut state = self.state.write();
        if !state.agents.contains_key(&agent.id) {
            return Err(PlatformError::NotFound("agent"));
        }
        state.agents.insert(agent.id, agent);
        Ok(())
    }

    fn list_agents(&self, tenant_id: TenantId) -> PlatformResult<Vec<Agent>> {
        let mut agents: Vec<_> = self
            .state
            .read()
            .agents
            .values()
            .filter(|a| a.tenant_id == tenant_id)
            .cloned()
            .collect();
        agents.sort_by(|a, b| a.hostname.cmp(&b.hostname));
        Ok(agents)
    }

    fn get_agent(&self, id: AgentId) -> PlatformResult<Option<Agent>> {
        Ok(self.state.read().agents.get(&id).cloned())
    }
}

impl ApiKeyStore for InMemoryPersistence {
    fn insert_api_key(&self, record: ApiKeyRecord) -> PlatformResult<()> {
        let mut state = self.state.write();
        if state.api_keys_by_prefix.contains_key(&record.token_prefix) {
            return Err(PlatformError::Conflict("api_key"));
        }
        state
            .api_keys_by_prefix
            .insert(record.token_prefix.clone(), record.id);
        state.api_keys.insert(record.id, record);
        Ok(())
    }

    fn get_api_key(&self, id: ApiKeyId) -> PlatformResult<Option<ApiKeyRecord>> {
        Ok(self.state.read().api_keys.get(&id).cloned())
    }

    fn get_api_key_by_prefix(&self, prefix: &str) -> PlatformResult<Option<ApiKeyRecord>> {
        let state = self.state.read();
        if let Some(id) = state.api_keys_by_prefix.get(prefix) {
            Ok(state.api_keys.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    fn list_api_keys(&self, tenant_id: TenantId) -> PlatformResult<Vec<ApiKeyRecord>> {
        let mut keys: Vec<_> = self
            .state
            .read()
            .api_keys
            .values()
            .filter(|k| k.tenant_id == tenant_id)
            .cloned()
            .collect();
        keys.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(keys)
    }

    fn update_api_key(&self, record: ApiKeyRecord) -> PlatformResult<()> {
        let mut state = self.state.write();
        if !state.api_keys.contains_key(&record.id) {
            return Err(PlatformError::NotFound("api_key"));
        }
        state
            .api_keys_by_prefix
            .insert(record.token_prefix.clone(), record.id);
        state.api_keys.insert(record.id, record);
        Ok(())
    }
}

impl TaskStore for InMemoryPersistence {
    fn enqueue_task(&self, task: Task) -> PlatformResult<()> {
        let mut state = self.state.write();
        if state.tasks.contains_key(&task.id) {
            return Err(PlatformError::Conflict("task"));
        }
        state.task_queue.push_back(task.id);
        state.tasks.insert(task.id, task);
        Ok(())
    }

    fn peek_next_task(&self, tenant_id: TenantId) -> PlatformResult<Option<Task>> {
        let mut state = self.state.write();
        let mut index = None;
        for (idx, task_id) in state.task_queue.iter().enumerate() {
            if let Some(task) = state.tasks.get(task_id) {
                if task.tenant_id == tenant_id && task.status == TaskStatus::Pending {
                    index = Some(idx);
                    break;
                }
            }
        }
        if let Some(i) = index {
            let task_id = state.task_queue.remove(i).unwrap();
            if let Some(task) = state.tasks.get_mut(&task_id) {
                task.status = TaskStatus::InProgress;
                task.started_at = Some(Utc::now());
                return Ok(Some(task.clone()));
            }
        }
        Ok(None)
    }

    fn update_task(&self, task: Task) -> PlatformResult<()> {
        let mut state = self.state.write();
        if !state.tasks.contains_key(&task.id) {
            return Err(PlatformError::NotFound("task"));
        }
        // remove existing queue entries for this task
        state.task_queue.retain(|id| id != &task.id);
        if task.status == TaskStatus::Pending {
            state.task_queue.push_back(task.id);
        }
        state.tasks.insert(task.id, task);
        Ok(())
    }

    fn get_task(&self, id: TaskId) -> PlatformResult<Option<Task>> {
        Ok(self.state.read().tasks.get(&id).cloned())
    }

    fn list_pending_tasks(&self, tenant_id: TenantId) -> PlatformResult<Vec<Task>> {
        let mut tasks: Vec<_> = self
            .state
            .read()
            .tasks
            .values()
            .filter(|task| task.tenant_id == tenant_id && task.status == TaskStatus::Pending)
            .cloned()
            .collect();
        tasks.sort_by(|a, b| a.scheduled_at.cmp(&b.scheduled_at));
        Ok(tasks)
    }
}

impl WorkflowStore for InMemoryPersistence {
    fn insert_workflow(&self, workflow: Workflow) -> PlatformResult<()> {
        let mut state = self.state.write();
        if state.workflows.contains_key(&workflow.id) {
            return Err(PlatformError::Conflict("workflow"));
        }
        state.workflows.insert(workflow.id, workflow);
        Ok(())
    }

    fn get_workflow(&self, id: WorkflowId) -> PlatformResult<Option<Workflow>> {
        Ok(self.state.read().workflows.get(&id).cloned())
    }

    fn list_workflows(&self, tenant_id: TenantId) -> PlatformResult<Vec<Workflow>> {
        let mut workflows: Vec<_> = self
            .state
            .read()
            .workflows
            .values()
            .filter(|w| w.tenant_id == tenant_id)
            .cloned()
            .collect();
        workflows.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(workflows)
    }
}

#[cfg(feature = "db")]
pub struct PostgresContentStore {
    pool: Pool<Postgres>,
}

#[cfg(feature = "db")]
impl PostgresContentStore {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    async fn map_upload_row(row: PgRow) -> PlatformResult<UploadSession> {
        let status: String = row.try_get("status")?;
        let status: UploadStatus = status.parse()?;
        let headers: serde_json::Value = row.try_get("headers")?;
        let headers: HashMap<String, String> = serde_json::from_value(headers)
            .map_err(|_| PlatformError::Internal("invalid headers"))?;
        Ok(UploadSession {
            id: row.try_get("id")?,
            tenant_id: row.try_get("tenant_id")?,
            project_id: row.try_get("project_id")?,
            content_id: row.try_get("content_id")?,
            status,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            expires_at: row.try_get("expires_at")?,
            upload_url: row.try_get("upload_url")?,
            headers,
        })
    }

    async fn map_metadata_row(row: PgRow) -> PlatformResult<ContentMetadata> {
        let visibility: String = row.try_get("visibility")?;
        let visibility: ContentVisibility = visibility.parse()?;
        let labels: Vec<String> = row.try_get("labels")?;
        let attributes: serde_json::Value = row.try_get("attributes")?;
        let attributes: HashMap<String, String> = serde_json::from_value(attributes)
            .map_err(|_| PlatformError::Internal("invalid attributes"))?;
        let size_bytes: Option<i64> = row.try_get("size_bytes")?;
        Ok(ContentMetadata {
            id: row.try_get("id")?,
            tenant_id: row.try_get("tenant_id")?,
            project_id: row.try_get("project_id")?,
            filename: row.try_get("filename")?,
            mime_type: row.try_get("mime_type")?,
            size_bytes: size_bytes.map(|v| v.max(0) as u64),
            checksum: row.try_get("checksum")?,
            storage_path: row.try_get("storage_path")?,
            labels,
            attributes,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            uploaded_by: row.try_get("uploaded_by")?,
            visibility,
        })
    }
}

#[cfg(feature = "db")]
pub struct PostgresOrchestrationStore {
    pool: Pool<Postgres>,
}

#[cfg(feature = "db")]
impl PostgresOrchestrationStore {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "db")]
pub struct PostgresModerationStore {
    pool: Pool<Postgres>,
}

#[cfg(feature = "db")]
impl PostgresModerationStore {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "db")]
pub struct PostgresMessagingStore {
    pool: Pool<Postgres>,
}

#[cfg(feature = "db")]
impl PostgresMessagingStore {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "db")]
#[derive(Debug, Clone, Default)]
pub struct AgentUpsert {
    pub id: AgentId,
    pub hostname: String,
    pub os: Option<String>,
    pub arch: Option<String>,
    pub cpu_cores: Option<i32>,
    pub memory_bytes: Option<i64>,
    pub tenant_id: Option<TenantId>,
    pub project_id: Option<ProjectId>,
    pub metadata: AgentMetadata,
    pub status: Option<AgentStatus>,
    pub last_seen: Option<DateTime<Utc>>,
}

#[cfg(feature = "db")]
#[derive(Debug, Clone)]
pub struct AgentHeartbeatRecord {
    pub agent_id: AgentId,
    pub cpu_percent: f64,
    pub memory_used_bytes: i64,
    pub timestamp: DateTime<Utc>,
}

#[cfg(feature = "db")]
#[derive(Debug, Clone, Default)]
pub struct AgentQuery {
    pub tenant_id: Option<TenantId>,
    pub project_id: Option<ProjectId>,
    pub hostname_fragment: Option<String>,
    pub lifecycle_status: Option<String>,
    pub last_seen_after: Option<DateTime<Utc>>,
    pub last_seen_before: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(feature = "db")]
#[derive(Debug, Clone)]
pub struct AgentSummaryRecord {
    pub id: AgentId,
    pub hostname: String,
    pub tenant_id: Option<TenantId>,
    pub project_id: Option<ProjectId>,
    pub lifecycle_status: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub cpu_percent: Option<f64>,
    pub memory_used_bytes: Option<i64>,
    pub metadata: AgentMetadata,
}

#[cfg(feature = "db")]
#[derive(sqlx::FromRow)]
struct AgentSummaryRow {
    id: uuid::Uuid,
    hostname: String,
    tenant_id: Option<uuid::Uuid>,
    project_id: Option<uuid::Uuid>,
    status: Option<String>,
    last_seen: Option<DateTime<Utc>>,
    metadata: serde_json::Value,
    cpu_percent: Option<f64>,
    memory_used_bytes: Option<i64>,
}

#[cfg(feature = "db")]
impl AgentSummaryRow {
    fn into_record(self) -> PlatformResult<AgentSummaryRecord> {
        let metadata: AgentMetadata = serde_json::from_value(self.metadata)
            .map_err(|_| PlatformError::Internal("invalid agent metadata"))?;
        Ok(AgentSummaryRecord {
            id: self.id,
            hostname: self.hostname,
            tenant_id: self.tenant_id,
            project_id: self.project_id,
            lifecycle_status: self.status,
            last_seen: self.last_seen,
            cpu_percent: self.cpu_percent,
            memory_used_bytes: self.memory_used_bytes,
            metadata,
        })
    }
}

#[cfg(feature = "db")]
#[derive(sqlx::FromRow)]
struct AssignmentRow {
    id: uuid::Uuid,
    agent_id: uuid::Uuid,
    workload_id: String,
    tenant_id: Option<uuid::Uuid>,
    project_id: Option<uuid::Uuid>,
    status: String,
    status_message: Option<String>,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[cfg(feature = "db")]
impl AssignmentRow {
    fn into_model(self) -> PlatformResult<WorkAssignment> {
        let metadata: HashMap<String, String> = serde_json::from_value(self.metadata)
            .map_err(|_| PlatformError::Internal("invalid assignment metadata"))?;
        let status = WorkStatus::from_str(self.status.to_ascii_lowercase().as_str())?;
        Ok(WorkAssignment {
            id: self.id,
            agent_id: self.agent_id,
            workload_id: self.workload_id,
            tenant_id: self.tenant_id,
            project_id: self.project_id,
            status,
            status_message: self.status_message,
            metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[cfg(feature = "db")]
#[derive(sqlx::FromRow)]
struct ModerationRow {
    id: uuid::Uuid,
    tenant_id: uuid::Uuid,
    project_id: uuid::Uuid,
    filename: String,
    mime_type: Option<String>,
    size_bytes: Option<i64>,
    state: String,
    reason: Option<String>,
    labels: serde_json::Value,
    attributes: serde_json::Value,
    submitted_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[cfg(feature = "db")]
impl ModerationRow {
    fn into_model(self) -> PlatformResult<ModeratedContent> {
        let labels: HashMap<String, String> = serde_json::from_value(self.labels)
            .map_err(|_| PlatformError::Internal("invalid labels"))?;
        let attributes: HashMap<String, String> = serde_json::from_value(self.attributes)
            .map_err(|_| PlatformError::Internal("invalid attributes"))?;
        let state = ModerationState::from_str(self.state.to_ascii_lowercase().as_str())?;
        Ok(ModeratedContent {
            id: self.id,
            tenant_id: self.tenant_id,
            project_id: self.project_id,
            filename: self.filename,
            mime_type: self.mime_type,
            size_bytes: self.size_bytes.map(|v| v.max(0) as u64),
            state,
            reason: self.reason,
            labels,
            attributes,
            submitted_at: self.submitted_at,
            updated_at: self.updated_at,
        })
    }
}

#[cfg(feature = "db")]
#[derive(sqlx::FromRow)]
struct MessageRow {
    id: uuid::Uuid,
    tenant_id: uuid::Uuid,
    project_id: uuid::Uuid,
    topic: String,
    key: Option<String>,
    payload: Vec<u8>,
    priority: String,
    attributes: serde_json::Value,
    published_at: DateTime<Utc>,
}

#[cfg(feature = "db")]
impl MessageRow {
    fn into_model(self) -> PlatformResult<MessageRecord> {
        let attributes: HashMap<String, String> = serde_json::from_value(self.attributes)
            .map_err(|_| PlatformError::Internal("invalid message attributes"))?;
        let priority = MessagePriority::from_str(self.priority.to_ascii_lowercase().as_str())?;
        Ok(MessageRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            project_id: self.project_id,
            topic: self.topic,
            key: self.key,
            payload: self.payload,
            priority,
            attributes,
            published_at: self.published_at,
        })
    }
}

#[cfg(feature = "db")]
pub struct PostgresAgentStore {
    pool: Pool<Postgres>,
}

#[cfg(feature = "db")]
impl PostgresAgentStore {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn upsert_agent(&self, input: AgentUpsert) -> PlatformResult<()> {
        let metadata = serde_json::to_value(&input.metadata)
            .map_err(|_| PlatformError::InvalidInput("invalid agent metadata"))?;
        sqlx::query(
            "INSERT INTO nodes (
                id, hostname, os, arch, cpu_cores, memory_bytes,
                tenant_id, project_id, metadata, status, last_seen, updated_at
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,NOW())
            ON CONFLICT (id) DO UPDATE SET
                hostname = EXCLUDED.hostname,
                os = EXCLUDED.os,
                arch = EXCLUDED.arch,
                cpu_cores = EXCLUDED.cpu_cores,
                memory_bytes = EXCLUDED.memory_bytes,
                tenant_id = EXCLUDED.tenant_id,
                project_id = EXCLUDED.project_id,
                metadata = EXCLUDED.metadata,
                status = EXCLUDED.status,
                last_seen = COALESCE(EXCLUDED.last_seen, nodes.last_seen),
                updated_at = NOW()
            ",
        )
        .bind(input.id)
        .bind(input.hostname)
        .bind(input.os)
        .bind(input.arch)
        .bind(input.cpu_cores)
        .bind(input.memory_bytes)
        .bind(input.tenant_id)
        .bind(input.project_id)
        .bind(metadata)
        .bind(input.status.map(|s| s.as_str().to_string()))
        .bind(input.last_seen)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn record_heartbeat(&self, record: AgentHeartbeatRecord) -> PlatformResult<()> {
        sqlx::query(
            "INSERT INTO node_metrics (node_id, ts, cpu_percent, memory_used_bytes)
             VALUES ($1,$2,$3,$4)",
        )
        .bind(record.agent_id)
        .bind(record.timestamp)
        .bind(record.cpu_percent)
        .bind(record.memory_used_bytes)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE nodes SET last_seen = $2, status = $3, updated_at = NOW() WHERE id = $1",
        )
        .bind(record.agent_id)
        .bind(record.timestamp)
        .bind("active")
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn query_agents(
        &self,
        query: &AgentQuery,
    ) -> PlatformResult<Vec<AgentSummaryRecord>> {
        use sqlx::{Postgres, QueryBuilder};

        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT n.id, n.hostname, n.tenant_id, n.project_id, n.status, n.last_seen, n.metadata,
                    metrics.cpu_percent, metrics.memory_used_bytes
             FROM nodes n
             LEFT JOIN LATERAL (
                 SELECT nm.cpu_percent, nm.memory_used_bytes
                 FROM node_metrics nm
                 WHERE nm.node_id = n.id
                 ORDER BY nm.ts DESC
                 LIMIT 1
             ) metrics ON TRUE
             WHERE 1=1",
        );

        if let Some(tenant_id) = query.tenant_id {
            builder.push(" AND n.tenant_id = ");
            builder.push_bind(tenant_id);
        }
        if let Some(project_id) = query.project_id {
            builder.push(" AND n.project_id = ");
            builder.push_bind(project_id);
        }
        if let Some(fragment) = &query.hostname_fragment {
            builder.push(" AND n.hostname ILIKE ");
            builder.push_bind(format!("%{}%", fragment));
        }
        if let Some(status) = &query.lifecycle_status {
            builder.push(" AND LOWER(n.status) = LOWER(");
            builder.push_bind(status);
            builder.push(")");
        }
        if let Some(after) = query.last_seen_after {
            builder.push(" AND n.last_seen >= ");
            builder.push_bind(after);
        }
        if let Some(before) = query.last_seen_before {
            builder.push(" AND n.last_seen <= ");
            builder.push_bind(before);
        }

        builder.push(" ORDER BY n.last_seen DESC NULLS LAST, n.id ASC");

        if let Some(limit) = query.limit {
            let limit: i64 = limit.max(0);
            builder.push(" LIMIT ");
            builder.push_bind(limit);
        }
        if let Some(offset) = query.offset {
            let offset: i64 = offset.max(0);
            builder.push(" OFFSET ");
            builder.push_bind(offset);
        }

        let rows: Vec<AgentSummaryRow> = builder.build_query_as().fetch_all(&self.pool).await?;
        rows.into_iter().map(|row| row.into_record()).collect()
    }
}

#[cfg(feature = "db")]
#[async_trait]
impl ContentStore for PostgresContentStore {
    async fn create_upload_session(&self, session: UploadSession) -> PlatformResult<()> {
        let headers = serde_json::to_value(&session.headers)
            .map_err(|_| PlatformError::InvalidInput("invalid headers"))?;
        sqlx::query(
            "INSERT INTO ugc_upload_sessions (
                id, tenant_id, project_id, content_id, status,
                created_at, updated_at, expires_at, upload_url, headers
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(session.id)
        .bind(session.tenant_id)
        .bind(session.project_id)
        .bind(session.content_id)
        .bind(session.status.as_str())
        .bind(session.created_at)
        .bind(session.updated_at)
        .bind(session.expires_at)
        .bind(session.upload_url)
        .bind(headers)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_upload_session(&self, session: UploadSession) -> PlatformResult<()> {
        let headers = serde_json::to_value(&session.headers)
            .map_err(|_| PlatformError::InvalidInput("invalid headers"))?;
        sqlx::query(
            "UPDATE ugc_upload_sessions SET
                tenant_id = $2,
                project_id = $3,
                content_id = $4,
                status = $5,
                created_at = $6,
                updated_at = $7,
                expires_at = $8,
                upload_url = $9,
                headers = $10
            WHERE id = $1",
        )
        .bind(session.id)
        .bind(session.tenant_id)
        .bind(session.project_id)
        .bind(session.content_id)
        .bind(session.status.as_str())
        .bind(session.created_at)
        .bind(session.updated_at)
        .bind(session.expires_at)
        .bind(session.upload_url)
        .bind(headers)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_upload_session(&self, id: UploadId) -> PlatformResult<Option<UploadSession>> {
        let row = sqlx::query("SELECT * FROM ugc_upload_sessions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        if let Some(row) = row {
            Ok(Some(Self::map_upload_row(row).await?))
        } else {
            Ok(None)
        }
    }

    async fn record_content_metadata(&self, metadata: ContentMetadata) -> PlatformResult<()> {
        let attributes = serde_json::to_value(&metadata.attributes)
            .map_err(|_| PlatformError::InvalidInput("invalid attributes"))?;
        let size_bytes = metadata.size_bytes.map(|v| v as i64);
        sqlx::query(
            "INSERT INTO ugc_content_metadata (
                id, tenant_id, project_id, filename, mime_type, size_bytes,
                checksum, storage_path, labels, attributes, created_at,
                updated_at, uploaded_by, visibility
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14
            ) ON CONFLICT (id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                project_id = EXCLUDED.project_id,
                filename = EXCLUDED.filename,
                mime_type = EXCLUDED.mime_type,
                size_bytes = EXCLUDED.size_bytes,
                checksum = EXCLUDED.checksum,
                storage_path = EXCLUDED.storage_path,
                labels = EXCLUDED.labels,
                attributes = EXCLUDED.attributes,
                created_at = EXCLUDED.created_at,
                updated_at = EXCLUDED.updated_at,
                uploaded_by = EXCLUDED.uploaded_by,
                visibility = EXCLUDED.visibility",
        )
        .bind(metadata.id)
        .bind(metadata.tenant_id)
        .bind(metadata.project_id)
        .bind(metadata.filename)
        .bind(metadata.mime_type)
        .bind(size_bytes)
        .bind(metadata.checksum)
        .bind(metadata.storage_path)
        .bind(metadata.labels)
        .bind(attributes)
        .bind(metadata.created_at)
        .bind(metadata.updated_at)
        .bind(metadata.uploaded_by)
        .bind(metadata.visibility.as_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_content_metadata(&self, id: ContentId) -> PlatformResult<Option<ContentMetadata>> {
        let row = sqlx::query("SELECT * FROM ugc_content_metadata WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        if let Some(row) = row {
            Ok(Some(Self::map_metadata_row(row).await?))
        } else {
            Ok(None)
        }
    }

    async fn list_content_metadata(
        &self,
        query: &ContentQuery,
    ) -> PlatformResult<Vec<ContentMetadata>> {
        let mut builder =
            QueryBuilder::new("SELECT * FROM ugc_content_metadata WHERE tenant_id = ");
        builder.push_bind(query.tenant_id);
        if let Some(project_id) = query.project_id {
            builder.push(" AND project_id = ");
            builder.push_bind(project_id);
        }
        if let Some(term) = &query.search_term {
            let like = format!("%{}%", term.to_ascii_lowercase());
            builder.push(" AND (LOWER(filename) LIKE ");
            builder.push_bind(like.clone());
            builder.push(" OR attributes::text ILIKE ");
            builder.push_bind(like);
            builder.push(")");
        }
        if !query.tags.is_empty() {
            builder.push(" AND labels @> ");
            builder.push_bind(&query.tags);
        }
        builder.push(" ORDER BY created_at DESC");
        if let Some(limit) = query.limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit as i64);
        }
        if let Some(offset) = query.offset {
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }

        let query = builder.build();
        let rows = query.fetch_all(&self.pool).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(Self::map_metadata_row(row).await?);
        }
        Ok(out)
    }
}

#[cfg(feature = "db")]
#[async_trait]
impl OrchestrationStore for PostgresOrchestrationStore {
    async fn create_assignment(&self, input: NewAssignment) -> PlatformResult<WorkAssignment> {
        if input.workload_id.trim().is_empty() {
            return Err(PlatformError::InvalidInput("workload_id required"));
        }
        let metadata = serde_json::to_value(&input.metadata)
            .map_err(|_| PlatformError::InvalidInput("invalid metadata"))?;
        let row: AssignmentRow = sqlx::query_as(
            "INSERT INTO orchestration_assignments (
                id, agent_id, workload_id, tenant_id, project_id, status, status_message, metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING id, agent_id, workload_id, tenant_id, project_id, status, status_message, metadata, created_at, updated_at",
        )
        .bind(input.id)
        .bind(input.agent_id)
        .bind(input.workload_id)
        .bind(input.tenant_id)
        .bind(input.project_id)
        .bind(WorkStatus::Pending.as_str())
        .bind(Some(String::from("queued")))
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;
        row.into_model()
    }

    async fn update_assignment_status(
        &self,
        id: AssignmentId,
        status: WorkStatus,
        status_message: Option<String>,
    ) -> PlatformResult<WorkAssignment> {
        let row: Option<AssignmentRow> = sqlx::query_as(
            "UPDATE orchestration_assignments
             SET status = $2, status_message = $3, updated_at = NOW()
             WHERE id = $1
             RETURNING id, agent_id, workload_id, tenant_id, project_id, status, status_message, metadata, created_at, updated_at",
        )
        .bind(id)
        .bind(status.as_str())
        .bind(status_message.clone())
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => row.into_model(),
            None => Err(PlatformError::NotFound("assignment")),
        }
    }

    async fn list_assignments(
        &self,
        query: AssignmentQuery,
    ) -> PlatformResult<Vec<WorkAssignment>> {
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT id, agent_id, workload_id, tenant_id, project_id, status, status_message, metadata, created_at, updated_at
             FROM orchestration_assignments WHERE 1=1",
        );

        if let Some(agent_id) = query.agent_id {
            builder.push(" AND agent_id = ");
            builder.push_bind(agent_id);
        }
        if let Some(tenant_id) = query.tenant_id {
            builder.push(" AND tenant_id = ");
            builder.push_bind(tenant_id);
        }
        if let Some(project_id) = query.project_id {
            builder.push(" AND project_id = ");
            builder.push_bind(project_id);
        }
        if let Some(status) = query.status {
            builder.push(" AND status = ");
            builder.push_bind(status.as_str());
        }
        builder.push(" ORDER BY updated_at DESC");

        let rows: Vec<AssignmentRow> = builder.build_query_as().fetch_all(&self.pool).await?;
        rows.into_iter().map(|row| row.into_model()).collect()
    }
}

#[cfg(feature = "db")]
#[async_trait]
impl ModerationStore for PostgresModerationStore {
    async fn create_content(&self, input: NewModeratedContent) -> PlatformResult<ModeratedContent> {
        let labels = serde_json::to_value(&input.labels)
            .map_err(|_| PlatformError::InvalidInput("invalid labels"))?;
        let attributes = serde_json::to_value(&input.attributes)
            .map_err(|_| PlatformError::InvalidInput("invalid attributes"))?;
        let row: ModerationRow = sqlx::query_as(
            "INSERT INTO ugc_moderation_content (
                id, tenant_id, project_id, filename, mime_type, size_bytes, state, reason, labels, attributes
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING id, tenant_id, project_id, filename, mime_type, size_bytes, state, reason, labels, attributes, submitted_at, updated_at",
        )
        .bind(input.id)
        .bind(input.tenant_id)
        .bind(input.project_id)
        .bind(input.filename)
        .bind(input.mime_type)
        .bind(input.size_bytes.map(|v| v as i64))
        .bind(ModerationState::Pending.as_str())
        .bind::<Option<String>>(None)
        .bind(labels)
        .bind(attributes)
        .fetch_one(&self.pool)
        .await?;
        row.into_model()
    }

    async fn update_content_state(
        &self,
        id: ContentId,
        state: ModerationState,
        reason: Option<String>,
    ) -> PlatformResult<ModeratedContent> {
        let row: Option<ModerationRow> = sqlx::query_as(
            "UPDATE ugc_moderation_content
             SET state = $2, reason = $3, updated_at = NOW()
             WHERE id = $1
             RETURNING id, tenant_id, project_id, filename, mime_type, size_bytes, state, reason, labels, attributes, submitted_at, updated_at",
        )
        .bind(id)
        .bind(state.as_str())
        .bind(reason)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => row.into_model(),
            None => Err(PlatformError::NotFound("ugc_content")),
        }
    }

    async fn list_content(&self, query: ModerationQuery) -> PlatformResult<Vec<ModeratedContent>> {
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT id, tenant_id, project_id, filename, mime_type, size_bytes, state, reason, labels, attributes, submitted_at, updated_at
             FROM ugc_moderation_content WHERE 1=1",
        );
        if let Some(tenant_id) = query.tenant_id {
            builder.push(" AND tenant_id = ");
            builder.push_bind(tenant_id);
        }
        if let Some(project_id) = query.project_id {
            builder.push(" AND project_id = ");
            builder.push_bind(project_id);
        }
        if let Some(state) = query.state {
            builder.push(" AND state = ");
            builder.push_bind(state.as_str());
        }
        builder.push(" ORDER BY submitted_at DESC");
        let rows: Vec<ModerationRow> = builder.build_query_as().fetch_all(&self.pool).await?;
        rows.into_iter().map(|row| row.into_model()).collect()
    }
}

#[cfg(feature = "db")]
#[async_trait]
impl MessagingStore for PostgresMessagingStore {
    async fn enqueue_message(&self, input: NewMessageRecord) -> PlatformResult<MessageRecord> {
        if input.topic.trim().is_empty() {
            return Err(PlatformError::InvalidInput("topic required"));
        }
        let attributes = serde_json::to_value(&input.attributes)
            .map_err(|_| PlatformError::InvalidInput("invalid attributes"))?;
        let row: MessageRow = sqlx::query_as(
            "INSERT INTO messaging_messages (
                id, tenant_id, project_id, topic, key, payload, priority, attributes
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING id, tenant_id, project_id, topic, key, payload, priority, attributes, published_at",
        )
        .bind(input.id)
        .bind(input.tenant_id)
        .bind(input.project_id)
        .bind(&input.topic)
        .bind(input.key)
        .bind(input.payload)
        .bind(input.priority.as_str())
        .bind(attributes)
        .fetch_one(&self.pool)
        .await?;
        row.into_model()
    }

    async fn list_messages(&self, query: MessageQuery) -> PlatformResult<Vec<MessageRecord>> {
        if query.topic.trim().is_empty() {
            return Err(PlatformError::InvalidInput("topic required"));
        }
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT id, tenant_id, project_id, topic, key, payload, priority, attributes, published_at
             FROM messaging_messages WHERE topic = ",
        );
        builder.push_bind(&query.topic);
        if let Some(tenant_id) = query.tenant_id {
            builder.push(" AND tenant_id = ");
            builder.push_bind(tenant_id);
        }
        if let Some(project_id) = query.project_id {
            builder.push(" AND project_id = ");
            builder.push_bind(project_id);
        }
        builder.push(" ORDER BY published_at DESC");
        if let Some(limit) = query.limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit as i64);
        }
        let rows: Vec<MessageRow> = builder.build_query_as().fetch_all(&self.pool).await?;
        rows.into_iter().map(|row| row.into_model()).collect()
    }

    async fn ack_message(&self, topic: &str, id: MessageId) -> PlatformResult<()> {
        let result = sqlx::query("DELETE FROM messaging_messages WHERE id = $1 AND topic = $2")
            .bind(id)
            .bind(topic)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(PlatformError::NotFound("message"));
        }
        Ok(())
    }
}

#[async_trait]
impl ContentStore for InMemoryPersistence {
    async fn create_upload_session(&self, session: UploadSession) -> PlatformResult<()> {
        let mut state = self.state.write();
        if !state.tenants.contains_key(&session.tenant_id) {
            return Err(PlatformError::NotFound("tenant"));
        }
        if !state.projects.contains_key(&session.project_id) {
            return Err(PlatformError::NotFound("project"));
        }
        if state.upload_sessions.contains_key(&session.id) {
            return Err(PlatformError::Conflict("upload_session"));
        }
        state.upload_sessions.insert(session.id, session);
        Ok(())
    }

    async fn update_upload_session(&self, session: UploadSession) -> PlatformResult<()> {
        let mut state = self.state.write();
        if !state.upload_sessions.contains_key(&session.id) {
            return Err(PlatformError::NotFound("upload_session"));
        }
        state.upload_sessions.insert(session.id, session);
        Ok(())
    }

    async fn get_upload_session(&self, id: UploadId) -> PlatformResult<Option<UploadSession>> {
        Ok(self.state.read().upload_sessions.get(&id).cloned())
    }

    async fn record_content_metadata(&self, metadata: ContentMetadata) -> PlatformResult<()> {
        let mut state = self.state.write();
        if !state.tenants.contains_key(&metadata.tenant_id) {
            return Err(PlatformError::NotFound("tenant"));
        }
        if !state.projects.contains_key(&metadata.project_id) {
            return Err(PlatformError::NotFound("project"));
        }
        state.content_metadata.insert(metadata.id, metadata);
        Ok(())
    }

    async fn get_content_metadata(&self, id: ContentId) -> PlatformResult<Option<ContentMetadata>> {
        Ok(self.state.read().content_metadata.get(&id).cloned())
    }

    async fn list_content_metadata(
        &self,
        query: &ContentQuery,
    ) -> PlatformResult<Vec<ContentMetadata>> {
        let mut entries: Vec<_> = self
            .state
            .read()
            .content_metadata
            .values()
            .filter(|item| {
                if item.tenant_id != query.tenant_id {
                    return false;
                }
                if let Some(project_id) = query.project_id {
                    if item.project_id != project_id {
                        return false;
                    }
                }
                if let Some(term) = &query.search_term {
                    let term_lower = term.to_ascii_lowercase();
                    let filename_match = item.filename.to_ascii_lowercase().contains(&term_lower);
                    let attribute_match = item.attributes.iter().any(|(k, v)| {
                        k.to_ascii_lowercase().contains(&term_lower)
                            || v.to_ascii_lowercase().contains(&term_lower)
                    });
                    if !filename_match && !attribute_match {
                        return false;
                    }
                }
                if !query.tags.is_empty()
                    && !query
                        .tags
                        .iter()
                        .all(|tag| item.labels.iter().any(|label| label == tag))
                {
                    return false;
                }
                true
            })
            .cloned()
            .collect();

        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let offset = query.offset.unwrap_or(0) as usize;
        let limit = query.limit.unwrap_or(entries.len() as u32) as usize;
        let slice = entries.into_iter().skip(offset).take(limit).collect();
        Ok(slice)
    }
}

#[async_trait]
impl OrchestrationStore for InMemoryPersistence {
    async fn create_assignment(&self, input: NewAssignment) -> PlatformResult<WorkAssignment> {
        if input.workload_id.trim().is_empty() {
            return Err(PlatformError::InvalidInput("workload_id required"));
        }
        if let Some(tenant_id) = input.tenant_id {
            if !self.state.read().tenants.contains_key(&tenant_id) {
                return Err(PlatformError::NotFound("tenant"));
            }
        }
        if let Some(project_id) = input.project_id {
            if !self.state.read().projects.contains_key(&project_id) {
                return Err(PlatformError::NotFound("project"));
            }
        }
        let mut state = self.state.write();
        if state.assignments.contains_key(&input.id) {
            return Err(PlatformError::Conflict("assignment"));
        }
        let now = Utc::now();
        let assignment = WorkAssignment {
            id: input.id,
            agent_id: input.agent_id,
            workload_id: input.workload_id,
            tenant_id: input.tenant_id,
            project_id: input.project_id,
            status: WorkStatus::Pending,
            status_message: Some("queued".to_string()),
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
        };
        state.assignments.insert(assignment.id, assignment.clone());
        Ok(assignment)
    }

    async fn update_assignment_status(
        &self,
        id: AssignmentId,
        status: WorkStatus,
        status_message: Option<String>,
    ) -> PlatformResult<WorkAssignment> {
        let mut state = self.state.write();
        let assignment = state
            .assignments
            .get_mut(&id)
            .ok_or(PlatformError::NotFound("assignment"))?;
        assignment.status = status;
        assignment.status_message = status_message;
        assignment.updated_at = Utc::now();
        Ok(assignment.clone())
    }

    async fn list_assignments(
        &self,
        query: AssignmentQuery,
    ) -> PlatformResult<Vec<WorkAssignment>> {
        let state = self.state.read();
        let mut assignments: Vec<_> = state
            .assignments
            .values()
            .filter(|assignment| {
                if let Some(agent_id) = query.agent_id {
                    if assignment.agent_id != agent_id {
                        return false;
                    }
                }
                if let Some(tenant_id) = query.tenant_id {
                    if assignment.tenant_id != Some(tenant_id) {
                        return false;
                    }
                }
                if let Some(project_id) = query.project_id {
                    if assignment.project_id != Some(project_id) {
                        return false;
                    }
                }
                if let Some(status) = &query.status {
                    if &assignment.status != status {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        assignments.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(assignments)
    }
}

#[async_trait]
impl ModerationStore for InMemoryPersistence {
    async fn create_content(&self, input: NewModeratedContent) -> PlatformResult<ModeratedContent> {
        let mut state = self.state.write();
        if state.moderation_content.contains_key(&input.id) {
            return Err(PlatformError::Conflict("ugc_content"));
        }
        if !state.tenants.contains_key(&input.tenant_id) {
            return Err(PlatformError::NotFound("tenant"));
        }
        if !state.projects.contains_key(&input.project_id) {
            return Err(PlatformError::NotFound("project"));
        }
        let now = Utc::now();
        let record = ModeratedContent {
            id: input.id,
            tenant_id: input.tenant_id,
            project_id: input.project_id,
            filename: input.filename,
            mime_type: input.mime_type,
            size_bytes: input.size_bytes,
            state: ModerationState::Pending,
            reason: None,
            labels: input.labels,
            attributes: input.attributes,
            submitted_at: now,
            updated_at: now,
        };
        state.moderation_content.insert(record.id, record.clone());
        Ok(record)
    }

    async fn update_content_state(
        &self,
        id: ContentId,
        state: ModerationState,
        reason: Option<String>,
    ) -> PlatformResult<ModeratedContent> {
        let mut state_data = self.state.write();
        let record = state_data
            .moderation_content
            .get_mut(&id)
            .ok_or(PlatformError::NotFound("ugc_content"))?;
        record.state = state;
        record.reason = reason;
        record.updated_at = Utc::now();
        Ok(record.clone())
    }

    async fn list_content(&self, query: ModerationQuery) -> PlatformResult<Vec<ModeratedContent>> {
        let state = self.state.read();
        let mut items: Vec<_> = state
            .moderation_content
            .values()
            .filter(|item| {
                if let Some(tenant_id) = query.tenant_id {
                    if item.tenant_id != tenant_id {
                        return false;
                    }
                }
                if let Some(project_id) = query.project_id {
                    if item.project_id != project_id {
                        return false;
                    }
                }
                if let Some(state_filter) = &query.state {
                    if &item.state != state_filter {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        items.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));
        Ok(items)
    }
}

#[async_trait]
impl MessagingStore for InMemoryPersistence {
    async fn enqueue_message(&self, input: NewMessageRecord) -> PlatformResult<MessageRecord> {
        let mut state = self.state.write();
        if !state.tenants.contains_key(&input.tenant_id) {
            return Err(PlatformError::NotFound("tenant"));
        }
        if !state.projects.contains_key(&input.project_id) {
            return Err(PlatformError::NotFound("project"));
        }
        if state.messages.contains_key(&input.id) {
            return Err(PlatformError::Conflict("message"));
        }
        let record = MessageRecord {
            id: input.id,
            tenant_id: input.tenant_id,
            project_id: input.project_id,
            topic: input.topic.clone(),
            key: input.key,
            payload: input.payload,
            priority: input.priority,
            attributes: input.attributes,
            published_at: Utc::now(),
        };
        state.messages.insert(record.id, record.clone());
        state
            .messages_by_topic
            .entry(record.topic.clone())
            .or_insert_with(VecDeque::new)
            .push_back(record.id);
        Ok(record)
    }

    async fn list_messages(&self, query: MessageQuery) -> PlatformResult<Vec<MessageRecord>> {
        if query.topic.trim().is_empty() {
            return Err(PlatformError::InvalidInput("topic required"));
        }
        let state = self.state.read();
        let mut results = Vec::new();
        if let Some(queue) = state.messages_by_topic.get(&query.topic) {
            for message_id in queue {
                if let Some(message) = state.messages.get(message_id) {
                    if let Some(tenant_id) = query.tenant_id {
                        if message.tenant_id != tenant_id {
                            continue;
                        }
                    }
                    if let Some(project_id) = query.project_id {
                        if message.project_id != project_id {
                            continue;
                        }
                    }
                    results.push(message.clone());
                    if let Some(limit) = query.limit {
                        if results.len() as u32 >= limit {
                            break;
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    async fn ack_message(&self, topic: &str, id: MessageId) -> PlatformResult<()> {
        let mut state = self.state.write();
        if state.messages.remove(&id).is_none() {
            return Err(PlatformError::NotFound("message"));
        }
        if let Some(queue) = state.messages_by_topic.get_mut(topic) {
            queue.retain(|msg_id| msg_id != &id);
            if queue.is_empty() {
                state.messages_by_topic.remove(topic);
            }
        }
        Ok(())
    }
}
