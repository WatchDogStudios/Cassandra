use super::auth::AuthService;
use super::error::{PlatformError, PlatformResult};
use super::models::*;
use super::persistence::{AgentStore, ProjectStore, TenantStore};
use chrono::{Duration, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use cncommon::auth::{default_scope_registry, Scope};

#[derive(Debug, Clone)]
pub struct TenantCreateRequest {
    pub name: String,
    pub idempotency_key: Option<String>,
    pub settings: Option<TenantSettings>,
    pub bootstrap_scopes: Vec<Scope>,
    pub bootstrap_scripts: Vec<String>,
}

impl TenantCreateRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            idempotency_key: None,
            settings: None,
            bootstrap_scopes: vec![Scope::Admin],
            bootstrap_scripts: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TenantBootstrap {
    pub tenant: Tenant,
    pub default_api_key: Option<ApiKey>,
    pub bootstrap_scripts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectCreateRequest {
    pub tenant_id: TenantId,
    pub name: String,
    pub idempotency_key: Option<String>,
    pub bootstrap_scripts: Vec<String>,
}

impl ProjectCreateRequest {
    pub fn new(tenant_id: TenantId, name: impl Into<String>) -> Self {
        Self {
            tenant_id,
            name: name.into(),
            idempotency_key: None,
            bootstrap_scripts: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectBootstrap {
    pub project: Project,
    pub bootstrap_scripts: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AgentRegistrationOptions {
    pub metadata: Option<AgentMetadata>,
    pub bootstrap_commands: Vec<String>,
    pub certificate_bundle: Option<Vec<u8>>,
}

#[derive(Clone)]
pub struct ProvisioningService {
    tenants: Arc<dyn TenantStore>,
    projects: Arc<dyn ProjectStore>,
    agents: Arc<dyn AgentStore>,
    auth: Arc<AuthService>,
    tenant_idempotency: Arc<RwLock<HashMap<String, TenantBootstrap>>>,
    project_idempotency: Arc<RwLock<HashMap<String, ProjectBootstrap>>>,
    heartbeat_timeout: Duration,
}

impl ProvisioningService {
    pub fn new(
        tenants: Arc<dyn TenantStore>,
        projects: Arc<dyn ProjectStore>,
        agents: Arc<dyn AgentStore>,
        auth: Arc<AuthService>,
    ) -> Self {
        Self {
            tenants,
            projects,
            agents,
            auth,
            tenant_idempotency: Arc::new(RwLock::new(HashMap::new())),
            project_idempotency: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_timeout: Duration::minutes(5),
        }
    }

    pub fn with_heartbeat_timeout(mut self, timeout: Duration) -> Self {
        self.heartbeat_timeout = timeout;
        self
    }

    pub fn create_tenant(&self, name: impl Into<String>) -> PlatformResult<Tenant> {
        let result = self.create_tenant_with_options(TenantCreateRequest::new(name))?;
        Ok(result.tenant)
    }

    pub fn create_tenant_with_options(
        &self,
        request: TenantCreateRequest,
    ) -> PlatformResult<TenantBootstrap> {
        if let Some(key) = request.idempotency_key.as_ref() {
            if let Some(existing) = self.tenant_idempotency.read().get(key).cloned() {
                return Ok(existing);
            }
        }

        let TenantCreateRequest {
            name,
            idempotency_key,
            settings,
            bootstrap_scopes,
            bootstrap_scripts,
        } = request;

        if name.trim().is_empty() {
            return Err(PlatformError::InvalidInput("tenant name required"));
        }

        let tenant = Tenant {
            id: Uuid::new_v4(),
            name,
            created_at: Utc::now(),
            settings: settings.unwrap_or_default(),
        };
        self.tenants.insert_tenant(tenant.clone())?;

        let scopes = if bootstrap_scopes.is_empty() {
            vec![Scope::Admin]
        } else {
            bootstrap_scopes
        };
        let default_api_key = if scopes.is_empty() {
            None
        } else {
            Some(self.auth.issue_api_key(
                tenant.id,
                format!("tenant:{}:default", tenant.id),
                scopes,
            )?)
        };
        let scripts = if bootstrap_scripts.is_empty() {
            vec![format!("cassctl bootstrap --tenant {}", tenant.id)]
        } else {
            bootstrap_scripts
        };
        let bundle = TenantBootstrap {
            tenant: tenant.clone(),
            default_api_key,
            bootstrap_scripts: scripts.clone(),
        };
        if let Some(key) = idempotency_key {
            self.tenant_idempotency.write().insert(key, bundle.clone());
        }
        Ok(bundle)
    }

    pub fn create_project(
        &self,
        tenant_id: TenantId,
        name: impl Into<String>,
    ) -> PlatformResult<Project> {
        let result =
            self.create_project_with_options(ProjectCreateRequest::new(tenant_id, name))?;
        Ok(result.project)
    }

    pub fn create_project_with_options(
        &self,
        request: ProjectCreateRequest,
    ) -> PlatformResult<ProjectBootstrap> {
        if self.tenants.get_tenant(request.tenant_id)?.is_none() {
            return Err(PlatformError::NotFound("tenant"));
        }
        if let Some(key) = request.idempotency_key.as_ref() {
            if let Some(existing) = self.project_idempotency.read().get(key).cloned() {
                return Ok(existing);
            }
        }
        if request.name.trim().is_empty() {
            return Err(PlatformError::InvalidInput("project name required"));
        }
        let ProjectCreateRequest {
            tenant_id,
            name,
            idempotency_key,
            bootstrap_scripts,
        } = request;
        let project = Project {
            id: Uuid::new_v4(),
            tenant_id,
            name,
            created_at: Utc::now(),
        };
        self.projects.insert_project(project.clone())?;
        let scripts = if bootstrap_scripts.is_empty() {
            vec![format!("cassctl project init --project {}", project.id)]
        } else {
            bootstrap_scripts
        };
        let bundle = ProjectBootstrap {
            project: project.clone(),
            bootstrap_scripts: scripts.clone(),
        };
        if let Some(key) = idempotency_key {
            self.project_idempotency.write().insert(key, bundle.clone());
        }
        Ok(bundle)
    }

    pub fn register_agent(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        hostname: impl Into<String>,
    ) -> PlatformResult<ProvisionedAgent> {
        self.register_agent_with_options(
            tenant_id,
            project_id,
            hostname,
            AgentRegistrationOptions::default(),
        )
    }

    pub fn register_agent_with_options(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        hostname: impl Into<String>,
        options: AgentRegistrationOptions,
    ) -> PlatformResult<ProvisionedAgent> {
        let project = self
            .projects
            .get_project(project_id)?
            .ok_or(PlatformError::NotFound("project"))?;
        if project.tenant_id != tenant_id {
            return Err(PlatformError::Forbidden);
        }
        if self.tenants.get_tenant(tenant_id)?.is_none() {
            return Err(PlatformError::NotFound("tenant"));
        }
        let hostname = hostname.into();
        if hostname.trim().is_empty() {
            return Err(PlatformError::InvalidInput("hostname required"));
        }
        let AgentRegistrationOptions {
            metadata,
            bootstrap_commands,
            certificate_bundle,
        } = options;
        let metadata = metadata.unwrap_or_default();
        let agent = Agent {
            id: Uuid::new_v4(),
            tenant_id,
            project_id,
            hostname: hostname.clone(),
            status: AgentStatus::Registered,
            last_seen: None,
            created_at: Utc::now(),
            metadata,
        };
        self.agents.insert_agent(agent.clone())?;
        let registry = default_scope_registry();
        let agent_scopes = registry
            .get_role("agent")
            .map(|role| role.scopes.clone())
            .unwrap_or_else(|| vec![Scope::AgentExecute]);
        let mut scopes = agent_scopes;
        scopes.push(Scope::Custom(format!("project:{project_id}")));
        let api_key = self
            .auth
            .issue_api_key(tenant_id, format!("agent:{hostname}"), scopes)?;
        let commands = if bootstrap_commands.is_empty() {
            vec![format!("cass-agent enroll --agent {}", agent.id)]
        } else {
            bootstrap_commands
        };
        Ok(ProvisionedAgent {
            agent,
            api_key,
            bootstrap_commands: commands,
            certificate_bundle,
        })
    }

    pub fn provision_service_account(
        &self,
        tenant_id: TenantId,
        label: impl Into<String>,
        scopes: Vec<Scope>,
    ) -> PlatformResult<ApiKey> {
        if self.tenants.get_tenant(tenant_id)?.is_none() {
            return Err(PlatformError::NotFound("tenant"));
        }
        self.auth.issue_api_key(tenant_id, label, scopes)
    }

    pub fn record_agent_heartbeat(
        &self,
        agent_id: AgentId,
        when: Option<chrono::DateTime<Utc>>,
    ) -> PlatformResult<()> {
        let mut agent = self
            .agents
            .get_agent(agent_id)?
            .ok_or(PlatformError::NotFound("agent"))?;
        agent.last_seen = Some(when.unwrap_or_else(Utc::now));
        agent.status = AgentStatus::Active;
        self.agents.update_agent(agent)
    }

    pub fn set_agent_status(&self, agent_id: AgentId, status: AgentStatus) -> PlatformResult<()> {
        let mut agent = self
            .agents
            .get_agent(agent_id)?
            .ok_or(PlatformError::NotFound("agent"))?;
        agent.status = status;
        self.agents.update_agent(agent)
    }

    pub fn list_agents(&self, tenant_id: TenantId) -> PlatformResult<Vec<Agent>> {
        self.agents.list_agents(tenant_id)
    }

    pub fn sweep_inactive_agents(&self) -> PlatformResult<Vec<Agent>> {
        let mut suspended = Vec::new();
        let tenants = self.tenants.list_tenants()?;
        let threshold = Utc::now() - self.heartbeat_timeout;
        for tenant in tenants {
            for mut agent in self.agents.list_agents(tenant.id)? {
                let is_stale = match agent.last_seen {
                    Some(last_seen) => last_seen < threshold,
                    None => true,
                };
                if is_stale && agent.status != AgentStatus::Suspended {
                    agent.status = AgentStatus::Suspended;
                    self.agents.update_agent(agent.clone())?;
                    suspended.push(agent);
                }
            }
        }
        Ok(suspended)
    }

    pub fn issue_agent_token(&self, agent_id: AgentId) -> PlatformResult<AuthToken> {
        let agent = self
            .agents
            .get_agent(agent_id)?
            .ok_or(PlatformError::NotFound("agent"))?;
        let context = AuthContext {
            principal_id: agent.id,
            principal_type: PrincipalType::Agent,
            tenant_id: agent.tenant_id,
            scopes: vec![
                Scope::AgentExecute,
                Scope::Custom(format!("project:{}", agent.project_id)),
            ],
            issued_at: Utc::now(),
            expires_at: Utc::now(),
            audience: Some("agents".into()),
            issuer: None,
            session: Some(AuthSessionMetadata {
                user_agent: None,
                ip_address: None,
                device_id: Some(agent.hostname.clone()),
            }),
        };
        self.auth
            .issue_token_from_context(context, Some(Duration::minutes(15)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::persistence::{
        AgentStore, ApiKeyStore, InMemoryPersistence, ProjectStore, TenantStore,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn tenant_and_agent_flow() {
        let storage = Arc::new(InMemoryPersistence::new());
        let tenant_store: Arc<dyn TenantStore> = storage.clone();
        let project_store: Arc<dyn ProjectStore> = storage.clone();
        let agent_store: Arc<dyn AgentStore> = storage.clone();
        let api_key_store: Arc<dyn ApiKeyStore> = storage.clone();
        let auth = Arc::new(AuthService::new(
            tenant_store.clone(),
            api_key_store,
            b"secret".to_vec(),
        ));
        let provisioning = ProvisioningService::new(tenant_store, project_store, agent_store, auth)
            .with_heartbeat_timeout(Duration::minutes(1));

        let mut tenant_request = TenantCreateRequest::new("Example");
        tenant_request.idempotency_key = Some("tenant-key".into());
        tenant_request.bootstrap_scripts = vec!["setup.sh".into()];
        let tenant_bundle = provisioning
            .create_tenant_with_options(tenant_request)
            .unwrap();
        let tenant = tenant_bundle.tenant.clone();
        assert!(tenant_bundle.default_api_key.is_some());
        // idempotency returns same tenant
        let mut tenant_retry = TenantCreateRequest::new("Example");
        tenant_retry.idempotency_key = Some("tenant-key".into());
        let tenant_second = provisioning
            .create_tenant_with_options(tenant_retry)
            .unwrap();
        assert_eq!(tenant_second.tenant.id, tenant.id);

        let mut project_request = ProjectCreateRequest::new(tenant.id, "ExampleGame");
        project_request.idempotency_key = Some("project-key".into());
        project_request.bootstrap_scripts = vec!["deploy.sh".into()];
        let project_bundle = provisioning
            .create_project_with_options(project_request)
            .unwrap();
        let project = project_bundle.project.clone();
        assert_eq!(project_bundle.bootstrap_scripts.len(), 1);
        let mut project_retry = ProjectCreateRequest::new(tenant.id, "ExampleGame");
        project_retry.idempotency_key = Some("project-key".into());
        let project_second = provisioning
            .create_project_with_options(project_retry)
            .unwrap();
        assert_eq!(project_second.project.id, project.id);

        let mut metadata_tags = HashMap::new();
        metadata_tags.insert("region".into(), "us-east".into());
        let mut agent_options = AgentRegistrationOptions::default();
        agent_options.metadata = Some(AgentMetadata {
            capabilities: vec!["compute".into()],
            tags: metadata_tags,
        });
        agent_options.bootstrap_commands = vec!["install.sh".into()];
        let provisioned = provisioning
            .register_agent_with_options(tenant.id, project.id, "agent-1", agent_options)
            .unwrap();
        assert_eq!(provisioned.agent.tenant_id, tenant.id);
        assert_eq!(provisioned.api_key.tenant_id, tenant.id);
        assert_eq!(provisioned.bootstrap_commands.len(), 1);

        provisioning
            .record_agent_heartbeat(provisioned.agent.id, None)
            .unwrap();
        // make agent stale and ensure suspension sweep works
        let mut stale_agent = provisioning
            .agents
            .get_agent(provisioned.agent.id)
            .unwrap()
            .unwrap();
        stale_agent.last_seen = Some(Utc::now() - Duration::minutes(5));
        stale_agent.status = AgentStatus::Active;
        provisioning.agents.update_agent(stale_agent).unwrap();
        let suspended = provisioning.sweep_inactive_agents().unwrap();
        assert_eq!(suspended.len(), 1);
        assert_eq!(suspended[0].status, AgentStatus::Suspended);

        let svc_key = provisioning
            .provision_service_account(tenant.id, "svc:metrics", vec![Scope::ProvisioningManage])
            .unwrap();
        assert_eq!(svc_key.tenant_id, tenant.id);
    }
}
