use crate::state::AgentRegistry;
use chrono::{DateTime, Utc};
#[cfg(feature = "db")]
use cncore::platform::persistence::{AgentHeartbeatRecord, AgentUpsert, PostgresAgentStore};
use cnproto::{
    agent_control_server::{AgentControl, AgentControlServer},
    HeartbeatRequest, HeartbeatResponse, RegisterAgentRequest, RegisterAgentResponse,
};
use std::sync::Arc;
use tonic::{Request as GrpcRequest, Response as GrpcResponse, Status as GrpcStatus};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct InMemoryAgentControl {
    pub registry: AgentRegistry,
    #[cfg(feature = "db")]
    pub agent_store: Option<Arc<PostgresAgentStore>>,
}

impl InMemoryAgentControl {
    pub fn new(registry: AgentRegistry) -> Self {
        Self {
            registry,
            #[cfg(feature = "db")]
            agent_store: None,
        }
    }

    #[cfg(feature = "db")]
    pub fn with_store(
        registry: AgentRegistry,
        agent_store: Option<Arc<PostgresAgentStore>>,
    ) -> Self {
        Self {
            registry,
            agent_store,
        }
    }

    pub fn into_server(self) -> AgentControlServer<Self> {
        AgentControlServer::new(self)
    }
}

#[tonic::async_trait]
impl AgentControl for InMemoryAgentControl {
    async fn register_agent(
        &self,
        request: GrpcRequest<RegisterAgentRequest>,
    ) -> Result<GrpcResponse<RegisterAgentResponse>, GrpcStatus> {
        let req = request.into_inner();
        let tenant_id = parse_uuid_opt(&req.tenant_id)?;
        let project_id = parse_uuid_opt(&req.project_id)?;
        self.registry.upsert(
            req.node_id.clone(),
            req.hostname.clone(),
            0.0,
            0,
            tenant_id.map(|id| id.to_string()),
            project_id.map(|id| id.to_string()),
            Some(String::from("registered")),
            None,
        );
        #[cfg(feature = "db")]
        if let (Some(store), Ok(agent_id)) =
            (self.agent_store.as_ref(), Uuid::parse_str(&req.node_id))
        {
            let upsert = AgentUpsert {
                id: agent_id,
                hostname: req.hostname.clone(),
                os: Some(req.os.clone()),
                arch: Some(req.arch.clone()),
                cpu_cores: Some(req.cpu_cores as i32),
                memory_bytes: Some(req.memory_bytes as i64),
                tenant_id,
                project_id,
                metadata: Default::default(),
                status: Some(cncore::platform::models::AgentStatus::Registered),
                last_seen: Some(Utc::now()),
            };
            if let Err(err) = store.upsert_agent(upsert).await {
                tracing::error!(error = %err, "agent.upsert_failed");
                return Err(GrpcStatus::internal("agent persistence failure"));
            }
        }
        let assigned_id = if req.node_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            req.node_id
        };
        let resp = RegisterAgentResponse {
            assigned_id,
            session_token: "session-placeholder".into(),
            heartbeat_interval_seconds: 5,
        };
        Ok(GrpcResponse::new(resp))
    }

    async fn heartbeat(
        &self,
        request: GrpcRequest<HeartbeatRequest>,
    ) -> Result<GrpcResponse<HeartbeatResponse>, GrpcStatus> {
        let hb = request.into_inner();
        let last_seen_override = if hb.timestamp_unix_ms > 0 {
            Some(hb.timestamp_unix_ms)
        } else {
            None
        };
        self.registry.upsert(
            hb.assigned_id.clone(),
            "unknown".into(),
            hb.cpu_percent,
            hb.memory_used_bytes,
            None,
            None,
            Some(String::from("active")),
            last_seen_override,
        );
        #[cfg(feature = "db")]
        if let (Some(store), Ok(agent_id)) =
            (self.agent_store.as_ref(), Uuid::parse_str(&hb.assigned_id))
        {
            let timestamp = millis_to_datetime(hb.timestamp_unix_ms).unwrap_or_else(Utc::now);
            let record = AgentHeartbeatRecord {
                agent_id,
                cpu_percent: hb.cpu_percent,
                memory_used_bytes: hb.memory_used_bytes as i64,
                timestamp,
            };
            if let Err(err) = store.record_heartbeat(record).await {
                tracing::error!(error = %err, "agent.heartbeat_persist_failed");
            }
        }
        let resp = HeartbeatResponse {
            ok: true,
            rotate_credentials: false,
        };
        Ok(GrpcResponse::new(resp))
    }
}

fn parse_uuid_opt(value: &str) -> Result<Option<Uuid>, GrpcStatus> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    Uuid::parse_str(value)
        .map(Some)
        .map_err(|_| GrpcStatus::invalid_argument("invalid uuid"))
}

fn millis_to_datetime(value: u64) -> Option<DateTime<Utc>> {
    let value = value as i64;
    DateTime::<Utc>::from_timestamp_millis(value)
}
