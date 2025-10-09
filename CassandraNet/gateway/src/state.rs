use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

#[derive(Serialize, Clone, Debug, ToSchema)]
pub struct AgentSummary {
    pub id: String,
    pub hostname: String,
    pub last_seen_unix_ms: u64,
    pub cpu_percent: f64,
    pub memory_used_bytes: u64,
}

#[derive(Default, Clone)]
pub struct AgentRegistry(pub(crate) Arc<RwLock<HashMap<String, AgentSummary>>>);

impl AgentRegistry {
    pub fn upsert(&self, id: String, hostname: String, cpu: f64, mem: u64) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let mut map = self.0.write().unwrap();
        map.entry(id.clone())
            .and_modify(|a| {
                a.last_seen_unix_ms = now_ms;
                a.cpu_percent = cpu;
                a.memory_used_bytes = mem;
                a.hostname = hostname.clone();
            })
            .or_insert(AgentSummary {
                id,
                hostname,
                last_seen_unix_ms: now_ms,
                cpu_percent: cpu,
                memory_used_bytes: mem,
            });
    }

    pub fn list(&self) -> Vec<AgentSummary> {
        let map = self.0.read().unwrap();
        let mut v: Vec<_> = map.values().cloned().collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    }
}

#[derive(Clone)]
pub struct AppState {
    pub registry: AgentRegistry,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            registry: AgentRegistry::default(),
        }
    }
}
