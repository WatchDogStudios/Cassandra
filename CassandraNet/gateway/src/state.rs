use cncommon::observability::{InMemoryLogSink, InMemoryMetricsRegistry, LogPipeline};
#[cfg(feature = "db")]
use cncore::platform::persistence::PostgresAgentStore;
#[cfg(feature = "db")]
use cncore::platform::persistence::PostgresContentStore;
use cncore::platform::persistence::{
    ContentStore, InMemoryPersistence, MessagingStore, ModerationStore, OrchestrationStore,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration as StdDuration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Clone, Debug, ToSchema)]
pub struct AgentSummary {
    pub id: String,
    pub hostname: String,
    pub last_seen_unix_ms: u64,
    pub cpu_percent: f64,
    pub memory_used_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_status: Option<String>,
}

#[derive(Default, Clone)]
pub struct AgentRegistry(pub(crate) Arc<RwLock<HashMap<String, AgentSummary>>>);

impl AgentRegistry {
    pub fn upsert(
        &self,
        id: String,
        hostname: String,
        cpu: f64,
        mem: u64,
        tenant_id: Option<String>,
        project_id: Option<String>,
        lifecycle_status: Option<String>,
        last_seen_override: Option<u64>,
    ) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let effective_last_seen = last_seen_override.filter(|ts| *ts > 0).unwrap_or(now_ms);
        let mut map = self.0.write().unwrap();
        map.entry(id.clone())
            .and_modify(|a| {
                a.last_seen_unix_ms = effective_last_seen;
                a.cpu_percent = cpu;
                a.memory_used_bytes = mem;
                a.hostname = hostname.clone();
                if tenant_id.is_some() {
                    a.tenant_id = tenant_id.clone();
                }
                if project_id.is_some() {
                    a.project_id = project_id.clone();
                }
                if lifecycle_status.is_some() {
                    a.lifecycle_status = lifecycle_status.clone();
                }
            })
            .or_insert(AgentSummary {
                id,
                hostname,
                last_seen_unix_ms: effective_last_seen,
                cpu_percent: cpu,
                memory_used_bytes: mem,
                tenant_id,
                project_id,
                lifecycle_status,
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
    pub content_store: Arc<dyn ContentStore>,
    pub orchestration_store: Arc<dyn OrchestrationStore>,
    pub moderation_store: Arc<dyn ModerationStore>,
    pub messaging_store: Arc<dyn MessagingStore>,
    #[cfg(feature = "db")]
    pub agent_store: Option<Arc<PostgresAgentStore>>,
    pub telemetry: TelemetryState,
    pub rate_limiter: RateLimiter,
}

impl Default for AppState {
    fn default() -> Self {
        let persistence = Arc::new(InMemoryPersistence::new());
        let content_store: Arc<dyn ContentStore> = persistence.clone();
        let orchestration_store: Arc<dyn OrchestrationStore> = persistence.clone();
        let moderation_store: Arc<dyn ModerationStore> = persistence.clone();
        let messaging_store: Arc<dyn MessagingStore> = persistence.clone();
        Self {
            registry: AgentRegistry::default(),
            content_store,
            orchestration_store,
            moderation_store,
            messaging_store,
            #[cfg(feature = "db")]
            agent_store: None,
            telemetry: TelemetryState::default(),
            rate_limiter: RateLimiter::new(),
        }
    }
}

impl AppState {
    pub fn with_content_store(content_store: Arc<dyn ContentStore>) -> Self {
        let mut state = Self::default();
        state.content_store = content_store;
        state
    }

    pub fn with_dependencies(
        content_store: Arc<dyn ContentStore>,
        telemetry: TelemetryState,
        rate_limiter: RateLimiter,
    ) -> Self {
        let mut state = Self::default();
        state.content_store = content_store;
        state.telemetry = telemetry;
        state.rate_limiter = rate_limiter;
        state
    }
}

#[derive(Clone)]
pub struct TelemetryState {
    pub metrics: InMemoryMetricsRegistry,
    pub logs: LogPipeline,
    pub log_sink: Arc<InMemoryLogSink>,
}

impl Default for TelemetryState {
    fn default() -> Self {
        let metrics = InMemoryMetricsRegistry::new();
        let logs = LogPipeline::new();
        let sink = Arc::new(InMemoryLogSink::new());
        logs.register_sink(sink.clone());
        Self {
            metrics,
            logs,
            log_sink: sink,
        }
    }
}

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<RwLock<HashMap<(Uuid, String), RateWindow>>>,
}

#[derive(Clone)]
struct RateWindow {
    window_start: Instant,
    count: u32,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn check_and_increment(
        &self,
        tenant_id: Uuid,
        route: &str,
        limit: u32,
        window: StdDuration,
    ) -> bool {
        let mut guard = self.inner.write().unwrap();
        let key = (tenant_id, route.to_string());
        let entry = guard.entry(key).or_insert(RateWindow {
            window_start: Instant::now(),
            count: 0,
        });
        let now = Instant::now();
        if now.duration_since(entry.window_start) >= window {
            entry.window_start = now;
            entry.count = 0;
        }
        if entry.count >= limit {
            return false;
        }
        entry.count += 1;
        true
    }
}
