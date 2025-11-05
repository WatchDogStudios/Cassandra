use super::error::{PlatformError, PlatformResult};
use super::models::*;
use super::persistence::{TaskStore, WorkflowStore};
use chrono::{Duration, Utc};
use parking_lot::RwLock;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TaskPolicy {
    pub timeouts: Option<TaskTimeouts>,
    pub max_retries: u32,
    pub backoff_seconds: Option<u64>,
    pub priority: u32,
}

impl Default for TaskPolicy {
    fn default() -> Self {
        Self {
            timeouts: None,
            max_retries: 3,
            backoff_seconds: Some(30),
            priority: 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerStrategy {
    Fifo,
    Priority,
    FairnessByKind,
}

struct WorkflowRunState {
    run: WorkflowRun,
    step_lookup: HashMap<Uuid, WorkflowStep>,
    waiting_steps: HashSet<Uuid>,
    inflight_steps: HashSet<Uuid>,
    completed_kinds: HashSet<String>,
    failed_kinds: HashSet<String>,
}

impl WorkflowRunState {
    fn new(run: WorkflowRun, steps: &[WorkflowStep]) -> Self {
        let step_lookup = steps
            .iter()
            .cloned()
            .map(|step| (step.id, step))
            .collect::<HashMap<_, _>>();
        let waiting_steps = step_lookup.keys().cloned().collect();
        Self {
            run,
            step_lookup,
            waiting_steps,
            inflight_steps: HashSet::new(),
            completed_kinds: HashSet::new(),
            failed_kinds: HashSet::new(),
        }
    }

    fn pop_ready_steps(&mut self) -> Vec<WorkflowStep> {
        let ready_ids: Vec<Uuid> = self
            .waiting_steps
            .iter()
            .filter(|id| {
                self.step_lookup
                    .get(id)
                    .map(|step| {
                        step.dependencies
                            .iter()
                            .all(|dep| match dep.required_status {
                                TaskStatus::Completed => {
                                    self.completed_kinds.contains(&dep.task_kind)
                                }
                                TaskStatus::Failed => self.failed_kinds.contains(&dep.task_kind),
                                TaskStatus::Pending | TaskStatus::InProgress => true,
                            })
                    })
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        for id in &ready_ids {
            self.waiting_steps.remove(id);
            self.inflight_steps.insert(*id);
        }

        ready_ids
            .into_iter()
            .filter_map(|id| self.step_lookup.get(&id).cloned())
            .collect()
    }

    fn mark_step_outcome(&mut self, step_id: &Uuid, success: bool) {
        if let Some(step) = self.step_lookup.get(step_id) {
            if success {
                self.completed_kinds.insert(step.task_kind.clone());
            } else {
                self.failed_kinds.insert(step.task_kind.clone());
            }
        }
        self.inflight_steps.remove(step_id);
        self.run.current_step = Some(*step_id);
        self.run.updated_at = Utc::now();
        if self.waiting_steps.is_empty() && self.inflight_steps.is_empty() {
            self.run.completed_at = Some(Utc::now());
            if self.failed_kinds.is_empty() {
                self.run.status = WorkflowRunStatus::Completed;
            } else {
                self.run.status = WorkflowRunStatus::Failed;
            }
        }
    }
}

#[derive(Clone)]
struct LeaseState {
    version: u64,
    token: Uuid,
    worker_id: Uuid,
    leased_at: chrono::DateTime<Utc>,
    lease_expires_at: chrono::DateTime<Utc>,
}

struct WorkflowContext {
    _workflow_id: Uuid,
    run_id: Uuid,
    step_id: Uuid,
}

pub struct OrchestrationEngine {
    tasks: Arc<dyn TaskStore>,
    workflows: Arc<dyn WorkflowStore>,
    scheduler: RwLock<SchedulerStrategy>,
    task_policies: RwLock<HashMap<String, TaskPolicy>>,
    workflow_runs: RwLock<HashMap<Uuid, WorkflowRunState>>,
    lease_states: RwLock<HashMap<TaskId, LeaseState>>,
    last_kind: RwLock<Option<String>>,
}

impl OrchestrationEngine {
    pub fn new(tasks: Arc<dyn TaskStore>, workflows: Arc<dyn WorkflowStore>) -> Self {
        Self {
            tasks,
            workflows,
            scheduler: RwLock::new(SchedulerStrategy::Fifo),
            task_policies: RwLock::new(HashMap::new()),
            workflow_runs: RwLock::new(HashMap::new()),
            lease_states: RwLock::new(HashMap::new()),
            last_kind: RwLock::new(None),
        }
    }

    pub fn set_scheduler_strategy(&self, strategy: SchedulerStrategy) {
        *self.last_kind.write() = None;
        *self.scheduler.write() = strategy;
    }

    pub fn register_task_policy(&self, kind: impl Into<String>, policy: TaskPolicy) {
        self.task_policies.write().insert(kind.into(), policy);
    }

    pub fn register_workflow(
        &self,
        tenant_id: TenantId,
        name: impl Into<String>,
        steps: Vec<WorkflowStep>,
    ) -> PlatformResult<Workflow> {
        if steps.is_empty() {
            return Err(PlatformError::InvalidInput("workflow steps required"));
        }
        let workflow = Workflow {
            id: Uuid::new_v4(),
            tenant_id,
            name: name.into(),
            steps,
            created_at: Utc::now(),
        };
        self.workflows.insert_workflow(workflow.clone())?;
        Ok(workflow)
    }

    pub fn schedule_task(&self, request: TaskRequest) -> PlatformResult<Task> {
        let policy = self
            .task_policies
            .read()
            .get(&request.kind)
            .cloned()
            .unwrap_or_default();
        let timeouts = policy.timeouts.clone();
        let now = Utc::now();
        let task = Task {
            id: Uuid::new_v4(),
            tenant_id: request.tenant_id,
            kind: request.kind,
            payload: request.payload,
            status: TaskStatus::Pending,
            attempts: 0,
            scheduled_at: now,
            started_at: None,
            completed_at: None,
            last_error: None,
            result: None,
            timeouts,
        };
        self.tasks.enqueue_task(task.clone())?;
        Ok(task)
    }

    pub fn schedule_workflow(
        &self,
        workflow_id: WorkflowId,
        tenant_id: TenantId,
        initial_payload: Value,
    ) -> PlatformResult<Vec<Task>> {
        let workflow = self
            .workflows
            .get_workflow(workflow_id)?
            .ok_or(PlatformError::NotFound("workflow"))?;
        if workflow.tenant_id != tenant_id {
            return Err(PlatformError::Forbidden);
        }
        let now = Utc::now();
        let mut run = WorkflowRun {
            id: Uuid::new_v4(),
            tenant_id,
            workflow_id: workflow.id,
            status: WorkflowRunStatus::Running,
            current_step: None,
            created_at: now,
            updated_at: now,
            started_at: Some(now),
            completed_at: None,
            context: initial_payload.clone(),
        };
        let mut state = WorkflowRunState::new(run.clone(), &workflow.steps);
        let mut scheduled = Vec::new();
        let ready_steps = state.pop_ready_steps();
        for step in ready_steps {
            let payload = json!({
                "workflow_id": workflow.id,
                "workflow_run_id": run.id,
                "step_id": step.id,
                "input": initial_payload,
            });
            let task = self.schedule_task(TaskRequest {
                tenant_id,
                kind: step.task_kind.clone(),
                payload,
            })?;
            scheduled.push(task);
        }
        if scheduled.is_empty() {
            run.status = WorkflowRunStatus::Pending;
        }
        state.run = run;
        self.workflow_runs.write().insert(state.run.id, state);
        Ok(scheduled)
    }

    pub fn lease_next_task(
        &self,
        tenant_id: TenantId,
        worker_id: Uuid,
        lease_ttl: Duration,
    ) -> PlatformResult<Option<TaskLease>> {
        let pending = self.tasks.list_pending_tasks(tenant_id)?;
        let Some(mut task) = self.select_task(&pending) else {
            return Ok(None);
        };
        let now = Utc::now();
        task.status = TaskStatus::InProgress;
        task.started_at = Some(now);
        self.tasks.update_task(task.clone())?;
        let lease = self.start_lease(&task, worker_id, lease_ttl);
        Ok(Some(lease))
    }

    pub fn complete_task(
        &self,
        task_id: TaskId,
        result: Option<serde_json::Value>,
    ) -> PlatformResult<Task> {
        let mut task = self
            .tasks
            .get_task(task_id)?
            .ok_or(PlatformError::NotFound("task"))?;
        task.status = TaskStatus::Completed;
        task.completed_at = Some(Utc::now());
        task.result = result;
        self.tasks.update_task(task.clone())?;
        self.clear_lease(task_id);
        self.handle_task_outcome(&task, true)?;
        Ok(task)
    }

    pub fn fail_task(
        &self,
        task_id: TaskId,
        error: impl Into<String>,
        retry: bool,
    ) -> PlatformResult<Task> {
        let mut task = self
            .tasks
            .get_task(task_id)?
            .ok_or(PlatformError::NotFound("task"))?;
        task.attempts += 1;
        task.last_error = Some(error.into());
        let policy = self
            .task_policies
            .read()
            .get(&task.kind)
            .cloned()
            .unwrap_or_default();
        let should_retry = retry && task.attempts <= policy.max_retries;
        if should_retry {
            task.status = TaskStatus::Pending;
            task.started_at = None;
            task.completed_at = None;
            let backoff = policy.backoff_seconds.unwrap_or(0);
            task.scheduled_at = Utc::now()
                + if backoff > 0 {
                    Duration::seconds(backoff as i64)
                } else {
                    Duration::zero()
                };
        } else {
            task.status = TaskStatus::Failed;
            task.completed_at = Some(Utc::now());
        }
        self.tasks.update_task(task.clone())?;
        self.clear_lease(task_id);
        if !should_retry {
            self.handle_task_outcome(&task, false)?;
        }
        Ok(task)
    }

    pub fn renew_task_lease(
        &self,
        task_id: TaskId,
        worker_id: Uuid,
        lease_token: Uuid,
        extend_by: Duration,
    ) -> PlatformResult<TaskLease> {
        let mut leases = self.lease_states.write();
        let state = leases
            .get_mut(&task_id)
            .ok_or(PlatformError::InvalidInput("lease not found"))?;
        if state.worker_id != worker_id {
            return Err(PlatformError::InvalidInput("worker mismatch"));
        }
        if state.token != lease_token {
            return Err(PlatformError::InvalidInput("invalid lease token"));
        }
        if state.lease_expires_at < Utc::now() {
            return Err(PlatformError::InvalidInput("lease expired"));
        }
        state.version += 1;
        state.lease_expires_at += extend_by;
        let lease_state = state.clone();
        drop(leases);
        let task = self
            .tasks
            .get_task(task_id)?
            .ok_or(PlatformError::NotFound("task"))?;
        Ok(TaskLease {
            task,
            worker_id: lease_state.worker_id,
            leased_at: lease_state.leased_at,
            lease_expires_at: lease_state.lease_expires_at,
            lease_version: lease_state.version,
            lease_token: lease_state.token,
        })
    }

    pub fn get_workflow_run(&self, run_id: Uuid) -> Option<WorkflowRun> {
        self.workflow_runs
            .read()
            .get(&run_id)
            .map(|state| state.run.clone())
    }

    fn select_task(&self, pending: &[Task]) -> Option<Task> {
        if pending.is_empty() {
            return None;
        }
        let strategy = *self.scheduler.read();
        let candidate = match strategy {
            SchedulerStrategy::Fifo => pending
                .iter()
                .cloned()
                .min_by(|a, b| a.scheduled_at.cmp(&b.scheduled_at)),
            SchedulerStrategy::Priority => {
                let policies = self.task_policies.read();
                pending.iter().cloned().min_by(|a, b| {
                    let ap = policies.get(&a.kind).map(|p| p.priority).unwrap_or(100);
                    let bp = policies.get(&b.kind).map(|p| p.priority).unwrap_or(100);
                    ap.cmp(&bp)
                        .then_with(|| a.scheduled_at.cmp(&b.scheduled_at))
                })
            }
            SchedulerStrategy::FairnessByKind => {
                let last_kind = self.last_kind.read().clone();
                let mut sorted = pending.iter().cloned().collect::<Vec<_>>();
                sorted.sort_by(|a, b| a.scheduled_at.cmp(&b.scheduled_at));
                let mut fallback = None;
                for task in sorted.into_iter() {
                    if fallback.is_none() {
                        fallback = Some(task.clone());
                    }
                    if Some(task.kind.as_str()) != last_kind.as_deref() {
                        fallback = Some(task);
                        break;
                    }
                }
                fallback
            }
        };
        if let Some(task) = &candidate {
            if matches!(strategy, SchedulerStrategy::FairnessByKind) {
                *self.last_kind.write() = Some(task.kind.clone());
            }
        }
        candidate
    }

    fn start_lease(&self, task: &Task, worker_id: Uuid, lease_ttl: Duration) -> TaskLease {
        let lease_window = task
            .timeouts
            .as_ref()
            .and_then(|timeouts| timeouts.lease_seconds)
            .map(|secs| Duration::seconds(secs as i64))
            .unwrap_or(lease_ttl);
        let now = Utc::now();
        let expires_at = now + lease_window;
        let mut leases = self.lease_states.write();
        let version = leases
            .get(&task.id)
            .map(|state| state.version + 1)
            .unwrap_or(1);
        let lease_state = LeaseState {
            version,
            token: Uuid::new_v4(),
            worker_id,
            leased_at: now,
            lease_expires_at: expires_at,
        };
        leases.insert(task.id, lease_state.clone());
        TaskLease {
            task: task.clone(),
            worker_id,
            leased_at: lease_state.leased_at,
            lease_expires_at: lease_state.lease_expires_at,
            lease_version: lease_state.version,
            lease_token: lease_state.token,
        }
    }

    fn clear_lease(&self, task_id: TaskId) {
        self.lease_states.write().remove(&task_id);
    }

    fn workflow_context(task: &Task) -> Option<WorkflowContext> {
        let payload = task.payload.as_object()?;
        let workflow_id = payload
            .get("workflow_id")?
            .as_str()
            .and_then(|value| Uuid::parse_str(value).ok())?;
        let run_id = payload
            .get("workflow_run_id")?
            .as_str()
            .and_then(|value| Uuid::parse_str(value).ok())?;
        let step_id = payload
            .get("step_id")?
            .as_str()
            .and_then(|value| Uuid::parse_str(value).ok())?;
        Some(WorkflowContext {
            _workflow_id: workflow_id,
            run_id,
            step_id,
        })
    }

    fn handle_task_outcome(&self, task: &Task, success: bool) -> PlatformResult<()> {
        let Some(ctx) = Self::workflow_context(task) else {
            return Ok(());
        };
        let (run, ready_steps, finished) = {
            let mut runs = self.workflow_runs.write();
            let Some(state) = runs.get_mut(&ctx.run_id) else {
                return Ok(());
            };
            state.mark_step_outcome(&ctx.step_id, success);
            let ready = state.pop_ready_steps();
            let finished = matches!(
                state.run.status,
                WorkflowRunStatus::Completed
                    | WorkflowRunStatus::Failed
                    | WorkflowRunStatus::Cancelled
            );
            (state.run.clone(), ready, finished)
        };
        for step in ready_steps {
            let payload = json!({
                "workflow_id": run.workflow_id,
                "workflow_run_id": run.id,
                "step_id": step.id,
                "input": run.context.clone(),
            });
            self.schedule_task(TaskRequest {
                tenant_id: run.tenant_id,
                kind: step.task_kind.clone(),
                payload,
            })?;
        }
        if finished {
            self.workflow_runs.write().remove(&run.id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::persistence::{
        InMemoryPersistence, TaskStore, TenantStore, WorkflowStore,
    };

    #[test]
    fn workflow_schedule_and_completion() {
        let storage = Arc::new(InMemoryPersistence::new());
        let task_store: Arc<dyn TaskStore> = storage.clone();
        let workflow_store: Arc<dyn WorkflowStore> = storage.clone();
        let engine = OrchestrationEngine::new(task_store, workflow_store);
        let tenant_id = Uuid::new_v4();
        storage
            .insert_tenant(Tenant {
                id: tenant_id,
                name: "Tenant".into(),
                created_at: Utc::now(),
                settings: TenantSettings::default(),
            })
            .unwrap();
        let workflow = engine
            .register_workflow(
                tenant_id,
                "bootstrap",
                vec![WorkflowStep {
                    id: Uuid::new_v4(),
                    name: "step1".into(),
                    task_kind: "configure".into(),
                    dependencies: Vec::new(),
                }],
            )
            .unwrap();
        let tasks = engine
            .schedule_workflow(workflow.id, tenant_id, json!({"foo": "bar"}))
            .unwrap();
        assert_eq!(tasks.len(), 1);
        let lease = engine
            .lease_next_task(tenant_id, Uuid::new_v4(), Duration::minutes(5))
            .unwrap()
            .expect("task available");
        let completed = engine
            .complete_task(lease.task.id, Some(json!({"status": "ok"})))
            .unwrap();
        assert_eq!(completed.status, TaskStatus::Completed);
        assert_eq!(completed.result.unwrap()["status"], "ok");
    }

    #[test]
    fn lease_can_be_renewed() {
        let storage = Arc::new(InMemoryPersistence::new());
        let task_store: Arc<dyn TaskStore> = storage.clone();
        let workflow_store: Arc<dyn WorkflowStore> = storage.clone();
        let engine = OrchestrationEngine::new(task_store, workflow_store);
        let tenant_id = Uuid::new_v4();
        storage
            .insert_tenant(Tenant {
                id: tenant_id,
                name: "Tenant".into(),
                created_at: Utc::now(),
                settings: TenantSettings::default(),
            })
            .unwrap();

        let task = engine
            .schedule_task(TaskRequest {
                tenant_id,
                kind: "simple".into(),
                payload: json!({"foo": "bar"}),
            })
            .unwrap();

        let lease = engine
            .lease_next_task(tenant_id, Uuid::new_v4(), Duration::minutes(5))
            .unwrap()
            .expect("task available");
        assert_eq!(lease.task.id, task.id);
        assert_eq!(lease.lease_version, 1);

        let renewed = engine
            .renew_task_lease(
                lease.task.id,
                lease.worker_id,
                lease.lease_token,
                Duration::minutes(10),
            )
            .unwrap();
        assert_eq!(renewed.lease_version, 2);
        assert!(renewed.lease_expires_at > lease.lease_expires_at);
    }

    #[test]
    fn workflow_failure_triggers_compensation() {
        let storage = Arc::new(InMemoryPersistence::new());
        let task_store: Arc<dyn TaskStore> = storage.clone();
        let workflow_store: Arc<dyn WorkflowStore> = storage.clone();
        let engine = OrchestrationEngine::new(task_store, workflow_store);
        let tenant_id = Uuid::new_v4();
        storage
            .insert_tenant(Tenant {
                id: tenant_id,
                name: "Tenant".into(),
                created_at: Utc::now(),
                settings: TenantSettings::default(),
            })
            .unwrap();

        let workflow = engine
            .register_workflow(
                tenant_id,
                "with-compensation",
                vec![
                    WorkflowStep {
                        id: Uuid::new_v4(),
                        name: "primary".into(),
                        task_kind: "configure".into(),
                        dependencies: Vec::new(),
                    },
                    WorkflowStep {
                        id: Uuid::new_v4(),
                        name: "compensate".into(),
                        task_kind: "cleanup".into(),
                        dependencies: vec![TaskDependency {
                            task_kind: "configure".into(),
                            required_status: TaskStatus::Failed,
                        }],
                    },
                ],
            )
            .unwrap();

        let tasks = engine
            .schedule_workflow(workflow.id, tenant_id, json!({"foo": "bar"}))
            .unwrap();
        assert_eq!(tasks.len(), 1);
        let lease = engine
            .lease_next_task(tenant_id, Uuid::new_v4(), Duration::minutes(5))
            .unwrap()
            .expect("primary task available");
        assert_eq!(lease.task.kind, "configure");

        let run_id = Uuid::parse_str(
            lease.task.payload["workflow_run_id"]
                .as_str()
                .expect("run id"),
        )
        .unwrap();
        assert!(engine.get_workflow_run(run_id).is_some());

        engine
            .fail_task(lease.task.id, "boom", false)
            .expect("task failure handled");

        let pending = storage.list_pending_tasks(tenant_id).unwrap();
        assert!(pending.iter().any(|task| task.kind == "cleanup"));

        let compensation = engine
            .lease_next_task(tenant_id, Uuid::new_v4(), Duration::minutes(5))
            .unwrap()
            .expect("compensation task");
        assert_eq!(compensation.task.kind, "cleanup");

        engine
            .complete_task(compensation.task.id, None)
            .expect("compensation completed");

        assert!(engine.get_workflow_run(run_id).is_none());
    }
}
