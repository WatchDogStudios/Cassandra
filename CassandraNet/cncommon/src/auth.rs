use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    Admin,
    TenantRead,
    TenantWrite,
    ProvisioningManage,
    OrchestrationManage,
    ApiKeyManage,
    AgentExecute,
    WorkflowExecute,
    Custom(String),
}

impl Scope {
    pub fn as_str(&self) -> &str {
        match self {
            Scope::Admin => "admin",
            Scope::TenantRead => "tenant:read",
            Scope::TenantWrite => "tenant:write",
            Scope::ProvisioningManage => "provisioning:manage",
            Scope::OrchestrationManage => "orchestration:manage",
            Scope::ApiKeyManage => "apikey:manage",
            Scope::AgentExecute => "agent:execute",
            Scope::WorkflowExecute => "workflow:execute",
            Scope::Custom(value) => value.as_str(),
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Scope::Custom(_))
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "admin" => Scope::Admin,
            "tenant:read" => Scope::TenantRead,
            "tenant:write" => Scope::TenantWrite,
            "provisioning:manage" => Scope::ProvisioningManage,
            "orchestration:manage" => Scope::OrchestrationManage,
            "apikey:manage" => Scope::ApiKeyManage,
            "agent:execute" => Scope::AgentExecute,
            "workflow:execute" => Scope::WorkflowExecute,
            other => Scope::Custom(other.to_string()),
        }
    }
}

impl From<&Scope> for String {
    fn from(scope: &Scope) -> Self {
        scope.as_str().to_string()
    }
}

impl From<Scope> for String {
    fn from(scope: Scope) -> Self {
        scope.as_str().to_string()
    }
}

impl From<&str> for Scope {
    fn from(value: &str) -> Self {
        Scope::from_str(value)
    }
}

impl From<String> for Scope {
    fn from(value: String) -> Self {
        Scope::from_str(&value)
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<Scope>,
}

#[derive(Debug)]
pub struct ScopeRegistry {
    roles: Vec<Role>,
}

impl Default for ScopeRegistry {
    fn default() -> Self {
        let mut registry = Self { roles: Vec::new() };
        registry.seed_defaults();
        registry
    }
}

impl ScopeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn seed_defaults(&mut self) {
        self.roles.push(Role {
            name: "admin".to_string(),
            description: Some("Full administrative access".to_string()),
            scopes: vec![Scope::Admin],
        });
        self.roles.push(Role {
            name: "operator".to_string(),
            description: Some("Manage provisioning and orchestration".to_string()),
            scopes: vec![
                Scope::TenantRead,
                Scope::TenantWrite,
                Scope::ProvisioningManage,
                Scope::OrchestrationManage,
                Scope::ApiKeyManage,
            ],
        });
        self.roles.push(Role {
            name: "agent".to_string(),
            description: Some("Execute workflows and tasks".to_string()),
            scopes: vec![Scope::AgentExecute, Scope::WorkflowExecute],
        });
        self.roles.push(Role {
            name: "viewer".to_string(),
            description: Some("Read-only access to tenant resources".to_string()),
            scopes: vec![Scope::TenantRead],
        });
    }

    pub fn register_role(&mut self, role: Role) {
        if let Some(existing) = self.roles.iter_mut().find(|r| r.name == role.name) {
            *existing = role;
        } else {
            self.roles.push(role);
        }
    }

    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.iter().find(|r| r.name == name)
    }

    pub fn roles(&self) -> &[Role] {
        &self.roles
    }
}

static GLOBAL_SCOPE_REGISTRY: OnceLock<ScopeRegistry> = OnceLock::new();

pub fn default_scope_registry() -> &'static ScopeRegistry {
    GLOBAL_SCOPE_REGISTRY.get_or_init(ScopeRegistry::default)
}
