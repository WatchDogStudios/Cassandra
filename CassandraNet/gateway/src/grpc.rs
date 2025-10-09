use crate::state::AgentRegistry;
use cnproto::{
    agent_control_server::{AgentControl, AgentControlServer},
    HeartbeatRequest, HeartbeatResponse, RegisterAgentRequest, RegisterAgentResponse,
};
use tonic::{Request as GrpcRequest, Response as GrpcResponse, Status as GrpcStatus};

#[derive(Default, Clone)]
pub struct InMemoryAgentControl {
    pub registry: AgentRegistry,
}

impl InMemoryAgentControl {
    pub fn new(registry: AgentRegistry) -> Self {
        Self { registry }
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
        self.registry
            .upsert(req.node_id.clone(), req.hostname.clone(), 0.0, 0);
        #[cfg(feature = "db")]
        {
            if let Ok(pool) = cncore::db().await {
                let _ = sqlx::query("INSERT INTO nodes (id, hostname, os, arch, cpu_cores, memory_bytes) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (id) DO UPDATE SET hostname = EXCLUDED.hostname, last_seen = NOW()")
                    .bind(&req.node_id)
                    .bind(&req.hostname)
                    .bind(&req.os)
                    .bind(&req.arch)
                    .bind(req.cpu_cores as i32)
                    .bind(req.memory_bytes as i64)
                    .execute(pool)
                    .await;
            }
        }
        let resp = RegisterAgentResponse {
            assigned_id: if req.node_id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                req.node_id
            },
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
        self.registry.upsert(
            hb.assigned_id.clone(),
            "unknown".into(),
            hb.cpu_percent,
            hb.memory_used_bytes,
        );
        #[cfg(feature = "db")]
        {
            if let Ok(pool) = cncore::db().await {
                let _ = sqlx::query("INSERT INTO node_metrics (node_id, cpu_percent, memory_used_bytes) VALUES ($1,$2,$3)")
                    .bind(&hb.assigned_id)
                    .bind(hb.cpu_percent)
                    .bind(hb.memory_used_bytes as i64)
                    .execute(pool)
                    .await;
                let _ = sqlx::query(
                    "UPDATE nodes SET last_seen = NOW(), cpu_cores = cpu_cores WHERE id = $1",
                )
                .bind(&hb.assigned_id)
                .execute(pool)
                .await;
            }
        }
        let resp = HeartbeatResponse {
            ok: true,
            rotate_credentials: false,
        };
        Ok(GrpcResponse::new(resp))
    }
}
