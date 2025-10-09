use crate::platform::error::{PlatformError, PlatformResult};
use crate::platform::models::*;
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
