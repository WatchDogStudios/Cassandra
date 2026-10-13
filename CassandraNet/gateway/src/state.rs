use cncommon::observability::{InMemoryLogSink, InMemoryMetricsRegistry, LogPipeline};
#[cfg(feature = "db")]
use cncore::platform::persistence::PostgresAgentStore;
#[cfg(feature = "db")]
use cncore::platform::persistence::{
    AsyncApiKeyStore, AsyncProjectStore, AsyncTaskStore, AsyncTenantStore, AsyncWorkflowStore,
};
use cncore::platform::persistence::{
    ContentStore, InMemoryPersistence, MessagingStore, ModerationStore, OrchestrationStore,
    UserStore,
};
use serde::{Deserialize, Serialize};
pub mod audit {
    use cncommon::observability::{LogEvent, LogLevel, LogPipeline};
    use serde::Serialize;
    use std::sync::Arc;

    #[derive(Clone, Debug, Serialize)]
    pub struct AuditEvent {
        pub kind: String,
        pub payload: serde_json::Value,
        pub level: AuditLevel,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub actor_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub actor_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub ip_address: Option<String>,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum AuditLevel {
        Info,
        Warn,
        Error,
    }

    #[derive(Clone)]
    pub struct AuditSink {
        inner: Arc<dyn AuditEmitter + Send + Sync>,
    }

    pub trait AuditEmitter: Send + Sync {
        fn emit(&self, event: AuditEvent);
    }

    impl AuditSink {
        pub fn new(inner: Arc<dyn AuditEmitter + Send + Sync>) -> Self {
            Self { inner }
        }

        pub fn emit(&self, event: AuditEvent) {
            self.inner.emit(event);
        }

        pub fn from_pipeline(pipeline: LogPipeline) -> Self {
            Self::new(Arc::new(PipelineAudit { pipeline }))
        }
    }

    /// Lets a whole sink act as another emitter's fallback.
    impl AuditEmitter for AuditSink {
        fn emit(&self, event: AuditEvent) {
            self.inner.emit(event);
        }
    }

    #[allow(dead_code)]
    pub struct NoopAudit;

    impl AuditEmitter for NoopAudit {
        fn emit(&self, event: AuditEvent) {
            match event.level {
                AuditLevel::Info => {
                    tracing::info!(kind = %event.kind, payload = %event.payload, "audit.event")
                }
                AuditLevel::Warn => {
                    tracing::warn!(kind = %event.kind, payload = %event.payload, "audit.event")
                }
                AuditLevel::Error => {
                    tracing::error!(kind = %event.kind, payload = %event.payload, "audit.event")
                }
            }
        }
    }

    struct PipelineAudit {
        pipeline: LogPipeline,
    }

    impl AuditEmitter for PipelineAudit {
        fn emit(&self, event: AuditEvent) {
            let level = LogLevel::from(&event.level);
            let message = format!("audit::{}", event.kind);
            let log = LogEvent::new(level, message)
                .with_component("gateway.audit")
                .with_metadata(event.payload.clone());
            self.pipeline.emit(log);
        }
    }

    impl AuditEvent {
        pub fn info(event: &str, payload: serde_json::Value) -> Self {
            Self {
                kind: event.to_string(),
                payload,
                level: AuditLevel::Info,
                actor_id: None,
                actor_type: None,
                ip_address: None,
            }
        }

        #[allow(dead_code)]
        pub fn warn(event: &str, payload: serde_json::Value) -> Self {
            Self {
                kind: event.to_string(),
                payload,
                level: AuditLevel::Warn,
                actor_id: None,
                actor_type: None,
                ip_address: None,
            }
        }

        #[allow(dead_code)]
        pub fn error(event: &str, payload: serde_json::Value) -> Self {
            Self {
                kind: event.to_string(),
                payload,
                level: AuditLevel::Error,
                actor_id: None,
                actor_type: None,
                ip_address: None,
            }
        }

        /// Set the actor (user/service) that performed the action
        pub fn with_actor(
            mut self,
            actor_id: impl Into<String>,
            actor_type: impl Into<String>,
        ) -> Self {
            self.actor_id = Some(actor_id.into());
            self.actor_type = Some(actor_type.into());
            self
        }

        /// Set the client IP address
        pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
            self.ip_address = Some(ip.into());
            self
        }
    }

    impl From<&AuditLevel> for LogLevel {
        fn from(value: &AuditLevel) -> Self {
            match value {
                AuditLevel::Info => LogLevel::Info,
                AuditLevel::Warn => LogLevel::Warn,
                AuditLevel::Error => LogLevel::Error,
            }
        }
    }

    /// PostgreSQL-backed audit emitter that persists events to the audit_logs table.
    /// Events are written asynchronously to avoid blocking the request path.
    #[cfg(feature = "db")]
    pub struct PostgresAuditEmitter {
        pool: sqlx::PgPool,
        /// Optional fallback emitter (e.g., log pipeline) for redundancy
        fallback: Option<Arc<dyn AuditEmitter + Send + Sync>>,
    }

    #[cfg(feature = "db")]
    impl PostgresAuditEmitter {
        pub fn new(pool: sqlx::PgPool) -> Self {
            Self {
                pool,
                fallback: None,
            }
        }

        pub fn with_fallback(mut self, emitter: Arc<dyn AuditEmitter + Send + Sync>) -> Self {
            self.fallback = Some(emitter);
            self
        }
    }

    #[cfg(feature = "db")]
    impl AuditEmitter for PostgresAuditEmitter {
        fn emit(&self, event: AuditEvent) {
            // Always emit to fallback first for immediate logging
            if let Some(ref fallback) = self.fallback {
                fallback.emit(event.clone());
            }

            // Spawn async task to persist to DB
            let pool = self.pool.clone();
            let kind = event.kind.clone();
            let level = match event.level {
                AuditLevel::Info => "info",
                AuditLevel::Warn => "warn",
                AuditLevel::Error => "error",
            };
            let actor_id = event.actor_id.clone();
            let actor_type = event.actor_type.clone();
            let ip_address = event.ip_address.clone();
            let payload = event.payload.clone();

            tokio::spawn(async move {
                let result = sqlx::query(
                    r#"
                    INSERT INTO audit_logs (event_kind, level, actor_id, actor_type, ip_address, payload)
                    VALUES ($1, $2, $3::uuid, $4, $5::inet, $6)
                    "#,
                )
                .bind(&kind)
                .bind(level)
                .bind(&actor_id)
                .bind(&actor_type)
                .bind(&ip_address)
                .bind(&payload)
                .execute(&pool)
                .await;

                if let Err(err) = result {
                    tracing::error!(
                        error = %err,
                        event_kind = %kind,
                        "audit.db_persist_failed"
                    );
                }
            });
        }
    }
}
use cncore::platform::{InMemoryPortalStore, PortalStore};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration as StdDuration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
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
    // Mirrors the heartbeat wire fields one-to-one.
    #[allow(clippy::too_many_arguments)]
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

use crate::redis_store::RedisAgentStore;

#[derive(Clone)]
pub struct AppState {
    pub registry: AgentRegistry,
    pub redis_store: Option<RedisAgentStore>,
    pub content_store: Arc<dyn ContentStore>,
    #[allow(dead_code)] // no orchestration API is exposed yet (ROADMAP Phase 2)
    pub orchestration_store: Arc<dyn OrchestrationStore>,
    pub moderation_store: Arc<dyn ModerationStore>,
    pub messaging_store: Arc<dyn MessagingStore>,
    pub user_store: Arc<dyn UserStore>,
    #[cfg(feature = "db")]
    pub agent_store: Option<Arc<PostgresAgentStore>>,
    #[cfg(feature = "db")]
    pub tenant_store: Option<Arc<dyn AsyncTenantStore>>,
    #[cfg(feature = "db")]
    pub project_store: Option<Arc<dyn AsyncProjectStore>>,
    #[cfg(feature = "db")]
    pub task_store: Option<Arc<dyn AsyncTaskStore>>,
    #[cfg(feature = "db")]
    pub workflow_store: Option<Arc<dyn AsyncWorkflowStore>>,
    #[cfg(feature = "db")]
    pub api_key_store: Option<Arc<dyn AsyncApiKeyStore>>,
    pub telemetry: TelemetryState,
    pub rate_limiter: RateLimiter,
    pub audit: audit::AuditSink,
    pub jwt_secret: Arc<Vec<u8>>,
    pub portal_store: Arc<dyn PortalStore>,
    /// Where uploaded build artifacts are stored (`CASS_BUILD_STORAGE_DIR`).
    pub build_dir: Arc<PathBuf>,
}

/// `CASS_BUILD_STORAGE_DIR`, defaulting to `data/builds` under the working directory.
pub fn build_dir_from_env() -> PathBuf {
    std::env::var("CASS_BUILD_STORAGE_DIR")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/builds"))
}

impl Default for AppState {
    fn default() -> Self {
        let persistence = Arc::new(InMemoryPersistence::new());
        let content_store: Arc<dyn ContentStore> = persistence.clone();
        let orchestration_store: Arc<dyn OrchestrationStore> = persistence.clone();
        let moderation_store: Arc<dyn ModerationStore> = persistence.clone();
        let messaging_store: Arc<dyn MessagingStore> = persistence.clone();
        let user_store: Arc<dyn UserStore> = persistence.clone();
        let telemetry = TelemetryState::default();
        let audit = audit::AuditSink::from_pipeline(telemetry.logs.clone());
        // main() refuses to start without a secret; the empty fallback only
        // ever applies to unit tests, and an empty key never matches env tokens.
        let jwt_secret = crate::auth::jwt_secret().unwrap_or_default();
        Self {
            registry: AgentRegistry::default(),
            redis_store: None,
            content_store,
            orchestration_store,
            moderation_store,
            messaging_store,
            user_store,
            #[cfg(feature = "db")]
            agent_store: None,
            #[cfg(feature = "db")]
            tenant_store: None,
            #[cfg(feature = "db")]
            project_store: None,
            #[cfg(feature = "db")]
            task_store: None,
            #[cfg(feature = "db")]
            workflow_store: None,
            #[cfg(feature = "db")]
            api_key_store: None,
            telemetry,
            rate_limiter: RateLimiter::new(),
            audit,
            jwt_secret: Arc::new(jwt_secret),
            portal_store: Arc::new(InMemoryPortalStore::new()),
            build_dir: Arc::new(build_dir_from_env()),
        }
    }
}

impl AppState {
    #[cfg(test)]
    pub fn with_content_store(content_store: Arc<dyn ContentStore>) -> Self {
        Self {
            content_store,
            ..Self::default()
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

/// Configuration for a single rate limit rule
#[derive(Clone, Debug)]
pub struct RateLimitRule {
    pub limit: u32,
    pub window_secs: u64,
}

impl Default for RateLimitRule {
    fn default() -> Self {
        Self {
            limit: 60,
            window_secs: 60,
        }
    }
}

/// Configurable rate limit settings loaded from CASS_RATE_LIMITS env var
/// Format: route:limit:window_secs,route:limit:window_secs,...
/// Example: auth.login:5:60,ugc:create_upload:100:60
#[derive(Clone, Debug, Default)]
pub struct RateLimitConfig {
    rules: HashMap<String, RateLimitRule>,
}

impl RateLimitConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Ok(value) = std::env::var("CASS_RATE_LIMITS") {
            for entry in value.split(',') {
                let parts: Vec<&str> = entry.trim().split(':').collect();
                if parts.len() == 3 {
                    if let (Ok(limit), Ok(window)) =
                        (parts[1].parse::<u32>(), parts[2].parse::<u64>())
                    {
                        config.rules.insert(
                            parts[0].to_string(),
                            RateLimitRule {
                                limit,
                                window_secs: window,
                            },
                        );
                        tracing::info!(
                            route = parts[0],
                            limit,
                            window = window,
                            "rate_limit.configured"
                        );
                    }
                }
            }
        }
        config
    }

    pub fn get(&self, route: &str) -> Option<&RateLimitRule> {
        self.rules.get(route)
    }
}

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<RwLock<HashMap<(Uuid, String), RateWindow>>>,
    config: RateLimitConfig,
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
            config: RateLimitConfig::from_env(),
        }
    }

    pub fn check_and_increment(
        &self,
        tenant_id: Uuid,
        route: &str,
        limit: u32,
        window: StdDuration,
    ) -> bool {
        // Check for config override
        let (actual_limit, actual_window) = if let Some(rule) = self.config.get(route) {
            (rule.limit, StdDuration::from_secs(rule.window_secs))
        } else {
            (limit, window)
        };
        self.check_key((tenant_id, route.to_string()), actual_limit, actual_window)
    }

    /// Check rate limit by IP address for unauthenticated endpoints like login
    pub fn check_ip(&self, ip: &str, route: &str, limit: u32, window: StdDuration) -> bool {
        // Check for config override
        let (actual_limit, actual_window) = if let Some(rule) = self.config.get(route) {
            (rule.limit, StdDuration::from_secs(rule.window_secs))
        } else {
            (limit, window)
        };
        self.check_key(
            (Uuid::nil(), format!("ip:{}:{}", ip, route)),
            actual_limit,
            actual_window,
        )
    }

    fn check_key(&self, key: (Uuid, String), limit: u32, window: StdDuration) -> bool {
        let mut guard = self.inner.write().unwrap();
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
