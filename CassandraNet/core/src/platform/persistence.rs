use crate::platform::error::{PlatformError, PlatformResult};
use crate::platform::models::*;
use async_trait::async_trait;
#[cfg(feature = "db")]
use sqlx::{postgres::PgRow, Pool, Postgres, QueryBuilder, Row};
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
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
fn map_db_err(err: sqlx::Error) -> PlatformError {
    match err {
        sqlx::Error::RowNotFound => PlatformError::NotFound("record"),
        sqlx::Error::Database(db_err) => {
            if db_err.code().as_deref() == Some("23505") {
                PlatformError::Conflict("record")
            } else {
                PlatformError::Internal("database error")
            }
        }
        _ => PlatformError::Internal("database error"),
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
        Ok(ContentMetadata {
            id: row.try_get("id")?,
            tenant_id: row.try_get("tenant_id")?,
            project_id: row.try_get("project_id")?,
            filename: row.try_get("filename")?,
            mime_type: row.try_get("mime_type")?,
            size_bytes: row.try_get("size_bytes")?,
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
#[async_trait]
impl ContentStore for PostgresContentStore {
    async fn create_upload_session(&self, session: UploadSession) -> PlatformResult<()> {
        let headers = serde_json::to_value(&session.headers)
            .map_err(|_| PlatformError::InvalidInput("invalid headers"))?;
        sqlx::query(
            "INSERT INTO ugc_upload_sessions (
                id, tenant_id, project_id, content_id, status,
                created_at, updated_at, expires_at, upload_url, headers
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"
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
        .await
        .map_err(map_db_err)?;
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
            WHERE id = $1"
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
        .await
        .map_err(map_db_err)?;
        Ok(())
    }

    async fn get_upload_session(&self, id: UploadId) -> PlatformResult<Option<UploadSession>> {
        let row = sqlx::query("SELECT * FROM ugc_upload_sessions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err)?;
        if let Some(row) = row {
            Ok(Some(Self::map_upload_row(row).await?))
        } else {
            Ok(None)
        }
    }

    async fn record_content_metadata(&self, metadata: ContentMetadata) -> PlatformResult<()> {
        let attributes = serde_json::to_value(&metadata.attributes)
            .map_err(|_| PlatformError::InvalidInput("invalid attributes"))?;
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
                visibility = EXCLUDED.visibility"
        )
        .bind(metadata.id)
        .bind(metadata.tenant_id)
        .bind(metadata.project_id)
        .bind(metadata.filename)
        .bind(metadata.mime_type)
        .bind(metadata.size_bytes)
        .bind(metadata.checksum)
        .bind(metadata.storage_path)
        .bind(metadata.labels)
        .bind(attributes)
        .bind(metadata.created_at)
        .bind(metadata.updated_at)
        .bind(metadata.uploaded_by)
        .bind(metadata.visibility.as_str())
        .execute(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(())
    }

    async fn get_content_metadata(&self, id: ContentId) -> PlatformResult<Option<ContentMetadata>> {
        let row = sqlx::query("SELECT * FROM ugc_content_metadata WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err)?;
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
        let mut builder = QueryBuilder::new(
            "SELECT * FROM ugc_content_metadata WHERE tenant_id = "
        );
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
        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(Self::map_metadata_row(row).await?);
        }
        Ok(out)
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
        state
            .content_metadata
            .insert(metadata.id, metadata);
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
                    let attribute_match = item
                        .attributes
                        .iter()
                        .any(|(k, v)| k.to_ascii_lowercase().contains(&term_lower) || v.to_ascii_lowercase().contains(&term_lower));
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
        let slice = entries
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        Ok(slice)
    }
}
