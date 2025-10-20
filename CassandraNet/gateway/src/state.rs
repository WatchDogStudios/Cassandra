use cncommon::observability::{InMemoryLogSink, InMemoryMetricsRegistry, LogPipeline};
use cncore::platform::persistence::{ContentStore, InMemoryPersistence};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration as StdDuration, Instant};
use uuid::Uuid;
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
    pub content_store: Arc<dyn ContentStore>,
    pub telemetry: TelemetryState,
    pub rate_limiter: RateLimiter,
}

impl Default for AppState {
    fn default() -> Self {
        let persistence: Arc<dyn ContentStore> = Arc::new(InMemoryPersistence::new());
        Self {
            registry: AgentRegistry::default(),
            content_store: persistence,
            telemetry: TelemetryState::default(),
            rate_limiter: RateLimiter::new(),
        }
    }
}

impl AppState {
    pub fn with_content_store(content_store: Arc<dyn ContentStore>) -> Self {
        Self {
            registry: AgentRegistry::default(),
            content_store,
            telemetry: TelemetryState::default(),
            rate_limiter: RateLimiter::new(),
        }
    }

    pub fn with_dependencies(
        content_store: Arc<dyn ContentStore>,
        telemetry: TelemetryState,
        rate_limiter: RateLimiter,
    ) -> Self {
        Self {
            registry: AgentRegistry::default(),
            content_store,
            telemetry,
            rate_limiter,
        }
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
