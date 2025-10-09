use super::auth::AuthService;
use super::orchestration::OrchestrationEngine;
use super::persistence::{
    AgentStore, ApiKeyStore, InMemoryPersistence, ProjectStore, TaskStore, TenantStore,
    WorkflowStore,
};
use super::provisioning::ProvisioningService;
use once_cell::sync::OnceCell;
use std::sync::Arc;

static GLOBAL_PLATFORM: OnceCell<Arc<PlatformServices>> = OnceCell::new();

#[derive(Clone)]
pub struct PlatformServices {
    storage: Arc<InMemoryPersistence>,
    auth: Arc<AuthService>,
    provisioning: Arc<ProvisioningService>,
    orchestration: Arc<OrchestrationEngine>,
}

impl PlatformServices {
    pub fn in_memory(secret: impl Into<Vec<u8>>) -> Arc<Self> {
        let storage = Arc::new(InMemoryPersistence::new());
        let tenant_store: Arc<dyn TenantStore> = storage.clone();
        let project_store: Arc<dyn ProjectStore> = storage.clone();
        let agent_store: Arc<dyn AgentStore> = storage.clone();
        let api_key_store: Arc<dyn ApiKeyStore> = storage.clone();
        let task_store: Arc<dyn TaskStore> = storage.clone();
        let workflow_store: Arc<dyn WorkflowStore> = storage.clone();
        let auth = Arc::new(AuthService::new(
            tenant_store.clone(),
            api_key_store,
            secret,
        ));
        let provisioning = Arc::new(ProvisioningService::new(
            tenant_store,
            project_store,
            agent_store,
            auth.clone(),
        ));
        let orchestration = Arc::new(OrchestrationEngine::new(task_store, workflow_store));
        Arc::new(Self {
            storage,
            auth,
            provisioning,
            orchestration,
        })
    }

    pub fn init_global() -> Arc<Self> {
        GLOBAL_PLATFORM
            .get_or_init(|| {
                let secret =
                    std::env::var("CASS_JWT_SECRET").unwrap_or_else(|_| "dev-secret".into());
                PlatformServices::in_memory(secret)
            })
            .clone()
    }

    pub fn set_global(instance: Arc<Self>) {
        let _ = GLOBAL_PLATFORM.set(instance);
    }

    pub fn global() -> Option<Arc<Self>> {
        GLOBAL_PLATFORM.get().cloned()
    }

    pub fn auth(&self) -> Arc<AuthService> {
        self.auth.clone()
    }

    pub fn provisioning(&self) -> Arc<ProvisioningService> {
        self.provisioning.clone()
    }

    pub fn orchestration(&self) -> Arc<OrchestrationEngine> {
        self.orchestration.clone()
    }

    pub fn storage(&self) -> Arc<InMemoryPersistence> {
        self.storage.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_initializes() {
        let global = PlatformServices::init_global();
        assert!(PlatformServices::global().is_some());
        let auth = global.auth();
        let tenant = global
            .provisioning()
            .create_tenant("demo")
            .expect("tenant created");
        let list = auth.list_keys(tenant.id).unwrap();
        assert_eq!(list.len(), 1);
    }
}
