#![allow(clippy::result_large_err)] // tonic::Status is large; the framework dictates it.

use crate::redis_store::RedisAgentStore;
use crate::state::audit::{AuditEvent, AuditSink};
use crate::state::{AgentRegistry, AgentSummary};
#[cfg(feature = "db")]
use chrono::DateTime;
use chrono::Utc;
#[cfg(feature = "db")]
use cncore::platform::persistence::{AgentHeartbeatRecord, AgentUpsert, PostgresAgentStore};
use cncore::platform::persistence::{MessagingStore, ModerationStore};
use cncore::platform::{JobOutcome, PlatformError, PortalStore};
use cnproto::{
    agent_control_server::{AgentControl, AgentControlServer},
    messaging_service_server::{MessagingService, MessagingServiceServer},
    ugc_service_server::{UgcService, UgcServiceServer},
    AckMessageRequest, AckMessageResponse, AgentJob as ProtoAgentJob, BuildChunk, ContentItem,
    ContentState, DownloadBuildRequest, HeartbeatRequest, HeartbeatResponse, ListContentRequest,
    ListContentResponse, MessageEnvelope, MessagePriority, PublishEventRequest,
    PublishEventResponse, PullMessagesRequest, PullMessagesResponse, RegisterAgentRequest,
    RegisterAgentResponse, ReportJobResultRequest, ReportJobResultResponse, ReviewContentRequest,
    ReviewContentResponse, SubmitContentRequest, SubmitContentResponse,
};
use serde_json::json;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tonic::{Request as GrpcRequest, Response as GrpcResponse, Status as GrpcStatus};
use uuid::Uuid;

#[derive(Clone)]
pub struct InMemoryAgentControl {
    pub registry: AgentRegistry,
    pub redis_store: Option<RedisAgentStore>,
    pub audit: AuditSink,
    pub portal: Arc<dyn PortalStore>,
    pub build_dir: Arc<PathBuf>,
    #[cfg(feature = "db")]
    pub agent_store: Option<Arc<PostgresAgentStore>>,
}

/// Jobs handed to an agent per heartbeat; the rest wait for the next one.
const JOBS_PER_HEARTBEAT: u32 = 16;
const BUILD_CHUNK_BYTES: usize = 64 * 1024;

impl InMemoryAgentControl {
    #[cfg(any(not(feature = "db"), test))]
    pub fn new(
        registry: AgentRegistry,
        redis_store: Option<RedisAgentStore>,
        audit: AuditSink,
        portal: Arc<dyn PortalStore>,
        build_dir: Arc<PathBuf>,
    ) -> Self {
        Self {
            registry,
            redis_store,
            audit,
            portal,
            build_dir,
            #[cfg(feature = "db")]
            agent_store: None,
        }
    }

    #[cfg(feature = "db")]
    pub fn with_store(
        registry: AgentRegistry,
        redis_store: Option<RedisAgentStore>,
        audit: AuditSink,
        portal: Arc<dyn PortalStore>,
        build_dir: Arc<PathBuf>,
        agent_store: Option<Arc<PostgresAgentStore>>,
    ) -> Self {
        Self {
            registry,
            redis_store,
            audit,
            portal,
            build_dir,
            agent_store,
        }
    }

    /// Interval from the agent's stored config, else the platform default,
    /// else the built-in default; matches what the HTTP API reports.
    // ponytail: two store reads per heartbeat; cache configs if heartbeat volume makes this hot.
    async fn desired_interval(&self, agent_id: &str) -> u32 {
        fn field(value: Option<serde_json::Value>, key: &str) -> Option<u32> {
            value?
                .get(key)?
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
        }
        let configured = self.portal.get_agent_config(agent_id).await.ok().flatten();
        if let Some(n) = field(configured, "heartbeat_interval_seconds") {
            return n;
        }
        let settings = self.portal.get_settings().await.ok().flatten();
        field(settings, "default_heartbeat_interval")
            .unwrap_or(crate::portal_handlers::DEFAULT_HEARTBEAT_INTERVAL_SECS)
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
        require_scope(&request, "agent:execute")?;
        let req = request.into_inner();
        // The Postgres agent store keys nodes by UUID; anything else would
        // register "successfully" yet never appear in /agents.
        if !req.node_id.is_empty() && Uuid::parse_str(&req.node_id).is_err() {
            return Err(GrpcStatus::invalid_argument("node_id must be a UUID"));
        }
        let tenant_id = parse_uuid_opt(&req.tenant_id)?;
        let project_id = parse_uuid_opt(&req.project_id)?;

        let audit_payload = json!({
            "node_id": req.node_id.clone(),
            "hostname": req.hostname.clone(),
            "os": req.os.clone(),
            "arch": req.arch.clone(),
            "cpu_cores": req.cpu_cores,
            "memory_bytes": req.memory_bytes,
            "tenant_id": req.tenant_id.clone(),
            "project_id": req.project_id.clone(),
        });
        self.audit
            .emit(AuditEvent::info("agent.register", audit_payload));

        if let Some(redis) = &self.redis_store {
            let summary = AgentSummary {
                id: req.node_id.clone(),
                hostname: req.hostname.clone(),
                last_seen_unix_ms: 0, // Will be updated on heartbeat
                cpu_percent: 0.0,
                memory_used_bytes: 0,
                tenant_id: tenant_id.map(|id| id.to_string()),
                project_id: project_id.map(|id| id.to_string()),
                lifecycle_status: Some("registered".to_string()),
            };
            if let Err(e) = redis.register_heartbeat(summary).await {
                tracing::error!(error = %e, "redis.register_failed");
            }
        } else {
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
        }

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
            Uuid::new_v4().to_string()
        } else {
            req.node_id.clone()
        };
        let heartbeat_interval_seconds = self.desired_interval(&assigned_id).await;
        let resp = RegisterAgentResponse {
            assigned_id,
            session_token: "session-placeholder".into(),
            heartbeat_interval_seconds,
        };
        Ok(GrpcResponse::new(resp))
    }

    async fn heartbeat(
        &self,
        request: GrpcRequest<HeartbeatRequest>,
    ) -> Result<GrpcResponse<HeartbeatResponse>, GrpcStatus> {
        require_scope(&request, "agent:execute")?;
        let hb = request.into_inner();
        let last_seen_override = if hb.timestamp_unix_ms > 0 {
            Some(hb.timestamp_unix_ms)
        } else {
            None
        };

        let audit_payload = json!({
            "assigned_id": hb.assigned_id.clone(),
            "cpu_percent": hb.cpu_percent,
            "memory_used_bytes": hb.memory_used_bytes,
            "timestamp_unix_ms": hb.timestamp_unix_ms,
        });
        self.audit
            .emit(AuditEvent::info("agent.heartbeat", audit_payload));

        if let Some(redis) = &self.redis_store {
            let summary = AgentSummary {
                id: hb.assigned_id.clone(),
                hostname: "unknown".into(), // Redis store might want to preserve existing hostname if possible, but for now we follow existing logic
                last_seen_unix_ms: last_seen_override
                    .unwrap_or_else(|| Utc::now().timestamp_millis() as u64),
                cpu_percent: hb.cpu_percent,
                memory_used_bytes: hb.memory_used_bytes,
                tenant_id: None,
                project_id: None,
                lifecycle_status: Some("active".to_string()),
            };
            if let Err(e) = redis.register_heartbeat(summary).await {
                tracing::error!(error = %e, "redis.heartbeat_failed");
            }
        } else {
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
        }

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

        // A dispatch failure must not cost the agent its heartbeat: the jobs
        // stay pending and go out next time.
        let jobs = self
            .portal
            .claim_jobs(&hb.assigned_id, JOBS_PER_HEARTBEAT)
            .await
            .unwrap_or_else(|err| {
                tracing::error!(error = %err, agent_id = %hb.assigned_id, "agent.jobs.claim_failed");
                Vec::new()
            });
        for job in &jobs {
            self.audit.emit(AuditEvent::info(
                "agent.job.dispatched",
                json!({ "job_id": job.id, "agent_id": job.agent_id, "kind": job.kind }),
            ));
        }
        let resp = HeartbeatResponse {
            ok: true,
            rotate_credentials: false,
            jobs: jobs
                .into_iter()
                .map(|job| ProtoAgentJob {
                    job_id: job.id.to_string(),
                    kind: job.kind,
                    args_json: job.args.to_string(),
                    timeout_seconds: job.timeout_seconds,
                })
                .collect(),
            heartbeat_interval_seconds: self.desired_interval(&hb.assigned_id).await,
        };
        Ok(GrpcResponse::new(resp))
    }

    async fn report_job_result(
        &self,
        request: GrpcRequest<ReportJobResultRequest>,
    ) -> Result<GrpcResponse<ReportJobResultResponse>, GrpcStatus> {
        require_scope(&request, "agent:execute")?;
        let req = request.into_inner();
        let job_id = Uuid::parse_str(&req.job_id)
            .map_err(|_| GrpcStatus::invalid_argument("invalid job_id"))?;
        let outcome = JobOutcome {
            success: req.success,
            exit_code: req.has_exit_code.then_some(req.exit_code),
            stdout: req.stdout,
            stderr: req.stderr,
            error: Some(req.error).filter(|e| !e.is_empty()),
        };
        let job = self
            .portal
            .complete_job(job_id, &req.assigned_id, outcome)
            .await
            .map_err(|err| match err {
                PlatformError::NotFound(_) => GrpcStatus::not_found("job not found for this agent"),
                PlatformError::Conflict(_) => {
                    GrpcStatus::failed_precondition("job is not awaiting a result")
                }
                other => {
                    tracing::error!(error = %other, "agent.job.complete_failed");
                    GrpcStatus::internal("could not record job result")
                }
            })?;
        let level = if job.error.is_some() || job.exit_code.is_some_and(|c| c != 0) {
            AuditEvent::warn
        } else {
            AuditEvent::info
        };
        self.audit.emit(level(
            "agent.job.completed",
            json!({
                "job_id": job.id,
                "agent_id": job.agent_id,
                "kind": job.kind,
                "status": job.status.as_str(),
                "exit_code": job.exit_code,
                "error": job.error,
            }),
        ));
        Ok(GrpcResponse::new(ReportJobResultResponse { ok: true }))
    }

    type DownloadBuildStream =
        Pin<Box<dyn futures::Stream<Item = Result<BuildChunk, GrpcStatus>> + Send + 'static>>;

    async fn download_build(
        &self,
        request: GrpcRequest<DownloadBuildRequest>,
    ) -> Result<GrpcResponse<Self::DownloadBuildStream>, GrpcStatus> {
        require_scope(&request, "agent:execute")?;
        let build_id = Uuid::parse_str(&request.into_inner().build_id)
            .map_err(|_| GrpcStatus::invalid_argument("invalid build_id"))?;
        let build = self
            .portal
            .get_build(build_id)
            .await
            .map_err(|_| GrpcStatus::internal("could not load build"))?
            .ok_or_else(|| GrpcStatus::not_found("build not found"))?;
        let file = tokio::fs::File::open(self.build_dir.join(build_id.to_string()))
            .await
            .map_err(|err| {
                tracing::error!(error = %err, %build_id, "build.download.missing_file");
                GrpcStatus::not_found("build file missing")
            })?;
        self.audit.emit(AuditEvent::info(
            "build.downloaded",
            json!({ "build_id": build_id }),
        ));

        let metadata = BuildChunk {
            data: Vec::new(),
            filename: build.filename,
            size_bytes: build.size_bytes,
            checksum_sha256: build.checksum_sha256,
        };
        // State: (file still readable, metadata chunk not yet sent).
        let stream = futures::stream::unfold(
            (Some(file), Some(metadata)),
            |(file, metadata)| async move {
                if let Some(meta) = metadata {
                    return Some((Ok(meta), (file, None)));
                }
                let mut file = file?;
                let mut buf = vec![0u8; BUILD_CHUNK_BYTES];
                match file.read(&mut buf).await {
                    Ok(0) => None,
                    Ok(n) => {
                        buf.truncate(n);
                        let chunk = BuildChunk {
                            data: buf,
                            ..Default::default()
                        };
                        Some((Ok(chunk), (Some(file), None)))
                    }
                    Err(err) => {
                        tracing::error!(error = %err, "build.download.read_failed");
                        Some((Err(GrpcStatus::internal("read failed")), (None, None)))
                    }
                }
            },
        );
        Ok(GrpcResponse::new(Box::pin(stream)))
    }
}

/// gRPC callers authenticate exactly like HTTP callers: an `x-api-key` or
/// `authorization: Bearer` metadata entry carrying the required scope.
fn require_scope<T>(request: &GrpcRequest<T>, scope: &str) -> Result<(), GrpcStatus> {
    let headers = request.metadata().clone().into_headers();
    if crate::auth::has_scope(&headers, scope) {
        return Ok(());
    }
    let authenticated = crate::auth::validate_api_key(&headers).allowed
        || crate::auth::validate_jwt(&headers).allowed;
    Err(if authenticated {
        GrpcStatus::permission_denied(format!("scope required: {scope}"))
    } else {
        GrpcStatus::unauthenticated("missing or invalid credentials")
    })
}

fn parse_uuid_opt(value: &str) -> Result<Option<Uuid>, GrpcStatus> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    Uuid::parse_str(value)
        .map(Some)
        .map_err(|_| GrpcStatus::invalid_argument("invalid uuid"))
}

#[cfg(feature = "db")]
fn millis_to_datetime(value: u64) -> Option<DateTime<Utc>> {
    let value = value as i64;
    DateTime::<Utc>::from_timestamp_millis(value)
}

pub struct UgcServiceImpl {
    pub store: Arc<dyn ModerationStore>,
}

impl UgcServiceImpl {
    pub fn new(store: Arc<dyn ModerationStore>) -> Self {
        Self { store }
    }
    pub fn into_server(self) -> UgcServiceServer<Self> {
        UgcServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl UgcService for UgcServiceImpl {
    async fn submit_content(
        &self,
        request: GrpcRequest<SubmitContentRequest>,
    ) -> Result<GrpcResponse<SubmitContentResponse>, GrpcStatus> {
        require_scope(&request, "ugc:write")?;
        let req = request.into_inner();
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| GrpcStatus::invalid_argument("invalid tenant_id"))?;
        let project_id = Uuid::parse_str(&req.project_id)
            .map_err(|_| GrpcStatus::invalid_argument("invalid project_id"))?;

        let input = cncore::platform::models::NewModeratedContent {
            id: if req.content_id.is_empty() {
                Uuid::new_v4()
            } else {
                Uuid::parse_str(&req.content_id)
                    .map_err(|_| GrpcStatus::invalid_argument("invalid content_id"))?
            },
            tenant_id,
            project_id,
            filename: req.filename,
            mime_type: if req.mime_type.is_empty() {
                None
            } else {
                Some(req.mime_type)
            },
            size_bytes: if req.size_bytes == 0 {
                None
            } else {
                Some(req.size_bytes)
            },
            labels: req.labels,
            attributes: req.attributes,
        };

        let content = self
            .store
            .create_content(input)
            .await
            .map_err(|e| GrpcStatus::internal(e.to_string()))?;

        let item = ContentItem {
            content_id: content.id.to_string(),
            tenant_id: content.tenant_id.to_string(),
            project_id: content.project_id.to_string(),
            filename: content.filename,
            mime_type: content.mime_type.unwrap_or_default(),
            size_bytes: content.size_bytes.unwrap_or_default(),
            state: match content.state {
                cncore::platform::models::ModerationState::Pending => ContentState::Pending as i32,
                cncore::platform::models::ModerationState::Approved => {
                    ContentState::Approved as i32
                }
                cncore::platform::models::ModerationState::Rejected => {
                    ContentState::Rejected as i32
                }
                cncore::platform::models::ModerationState::Archived => {
                    ContentState::Archived as i32
                }
            },
            reason: content.reason.unwrap_or_default(),
            submitted_at_unix_ms: content.submitted_at.timestamp_millis(),
            updated_at_unix_ms: content.updated_at.timestamp_millis(),
            labels: content.labels,
            attributes: content.attributes,
        };

        Ok(GrpcResponse::new(SubmitContentResponse {
            item: Some(item),
        }))
    }

    async fn review_content(
        &self,
        request: GrpcRequest<ReviewContentRequest>,
    ) -> Result<GrpcResponse<ReviewContentResponse>, GrpcStatus> {
        require_scope(&request, "ugc:write")?;
        let req = request.into_inner();
        let content_id = Uuid::parse_str(&req.content_id)
            .map_err(|_| GrpcStatus::invalid_argument("invalid content_id"))?;

        let state = match ContentState::try_from(req.state).unwrap_or(ContentState::Unspecified) {
            ContentState::Pending => cncore::platform::models::ModerationState::Pending,
            ContentState::Approved => cncore::platform::models::ModerationState::Approved,
            ContentState::Rejected => cncore::platform::models::ModerationState::Rejected,
            ContentState::Archived => cncore::platform::models::ModerationState::Archived,
            _ => return Err(GrpcStatus::invalid_argument("invalid state")),
        };

        let content = self
            .store
            .update_content_state(content_id, state, Some(req.reason))
            .await
            .map_err(|e| GrpcStatus::internal(e.to_string()))?;

        let item = ContentItem {
            content_id: content.id.to_string(),
            tenant_id: content.tenant_id.to_string(),
            project_id: content.project_id.to_string(),
            filename: content.filename,
            mime_type: content.mime_type.unwrap_or_default(),
            size_bytes: content.size_bytes.unwrap_or_default(),
            state: match content.state {
                cncore::platform::models::ModerationState::Pending => ContentState::Pending as i32,
                cncore::platform::models::ModerationState::Approved => {
                    ContentState::Approved as i32
                }
                cncore::platform::models::ModerationState::Rejected => {
                    ContentState::Rejected as i32
                }
                cncore::platform::models::ModerationState::Archived => {
                    ContentState::Archived as i32
                }
            },
            reason: content.reason.unwrap_or_default(),
            submitted_at_unix_ms: content.submitted_at.timestamp_millis(),
            updated_at_unix_ms: content.updated_at.timestamp_millis(),
            labels: content.labels,
            attributes: content.attributes,
        };

        Ok(GrpcResponse::new(ReviewContentResponse {
            item: Some(item),
        }))
    }

    async fn list_content(
        &self,
        request: GrpcRequest<ListContentRequest>,
    ) -> Result<GrpcResponse<ListContentResponse>, GrpcStatus> {
        require_scope(&request, "ugc:read")?;
        let req = request.into_inner();
        let tenant_id = parse_uuid_opt(&req.tenant_id)?;
        let project_id = parse_uuid_opt(&req.project_id)?;

        let state =
            match ContentState::try_from(req.state_filter).unwrap_or(ContentState::Unspecified) {
                ContentState::Unspecified => None,
                ContentState::Pending => Some(cncore::platform::models::ModerationState::Pending),
                ContentState::Approved => Some(cncore::platform::models::ModerationState::Approved),
                ContentState::Rejected => Some(cncore::platform::models::ModerationState::Rejected),
                ContentState::Archived => Some(cncore::platform::models::ModerationState::Archived),
            };

        let query = cncore::platform::models::ModerationQuery {
            tenant_id,
            project_id,
            state,
        };

        let contents = self
            .store
            .list_content(query)
            .await
            .map_err(|e| GrpcStatus::internal(e.to_string()))?;

        let items = contents
            .into_iter()
            .map(|content| ContentItem {
                content_id: content.id.to_string(),
                tenant_id: content.tenant_id.to_string(),
                project_id: content.project_id.to_string(),
                filename: content.filename,
                mime_type: content.mime_type.unwrap_or_default(),
                size_bytes: content.size_bytes.unwrap_or_default(),
                state: match content.state {
                    cncore::platform::models::ModerationState::Pending => {
                        ContentState::Pending as i32
                    }
                    cncore::platform::models::ModerationState::Approved => {
                        ContentState::Approved as i32
                    }
                    cncore::platform::models::ModerationState::Rejected => {
                        ContentState::Rejected as i32
                    }
                    cncore::platform::models::ModerationState::Archived => {
                        ContentState::Archived as i32
                    }
                },
                reason: content.reason.unwrap_or_default(),
                submitted_at_unix_ms: content.submitted_at.timestamp_millis(),
                updated_at_unix_ms: content.updated_at.timestamp_millis(),
                labels: content.labels,
                attributes: content.attributes,
            })
            .collect();

        Ok(GrpcResponse::new(ListContentResponse { items }))
    }
}

pub struct MessagingServiceImpl {
    pub store: Arc<dyn MessagingStore>,
}

impl MessagingServiceImpl {
    pub fn new(store: Arc<dyn MessagingStore>) -> Self {
        Self { store }
    }
    pub fn into_server(self) -> MessagingServiceServer<Self> {
        MessagingServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl MessagingService for MessagingServiceImpl {
    async fn publish_event(
        &self,
        request: GrpcRequest<PublishEventRequest>,
    ) -> Result<GrpcResponse<PublishEventResponse>, GrpcStatus> {
        require_scope(&request, "messaging:write")?;
        let req = request.into_inner();
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| GrpcStatus::invalid_argument("invalid tenant_id"))?;
        let project_id = Uuid::parse_str(&req.project_id)
            .map_err(|_| GrpcStatus::invalid_argument("invalid project_id"))?;

        let priority =
            match MessagePriority::try_from(req.priority).unwrap_or(MessagePriority::Unspecified) {
                MessagePriority::Low => cncore::platform::models::MessagePriority::Low,
                MessagePriority::Normal => cncore::platform::models::MessagePriority::Normal,
                MessagePriority::High => cncore::platform::models::MessagePriority::High,
                _ => cncore::platform::models::MessagePriority::Normal,
            };

        let input = cncore::platform::models::NewMessageRecord {
            id: Uuid::new_v4(),
            tenant_id,
            project_id,
            topic: req.topic,
            key: if req.key.is_empty() {
                None
            } else {
                Some(req.key)
            },
            payload: req.payload,
            priority,
            attributes: req.attributes,
        };

        let msg = self
            .store
            .enqueue_message(input)
            .await
            .map_err(|e| GrpcStatus::internal(e.to_string()))?;

        let envelope = MessageEnvelope {
            message_id: msg.id.to_string(),
            tenant_id: msg.tenant_id.to_string(),
            project_id: msg.project_id.to_string(),
            topic: msg.topic,
            key: msg.key.unwrap_or_default(),
            payload: msg.payload,
            priority: match msg.priority {
                cncore::platform::models::MessagePriority::Low => MessagePriority::Low as i32,
                cncore::platform::models::MessagePriority::Normal => MessagePriority::Normal as i32,
                cncore::platform::models::MessagePriority::High => MessagePriority::High as i32,
            },
            published_at_unix_ms: msg.published_at.timestamp_millis(),
            attributes: msg.attributes,
        };

        Ok(GrpcResponse::new(PublishEventResponse {
            message: Some(envelope),
        }))
    }

    async fn pull_messages(
        &self,
        request: GrpcRequest<PullMessagesRequest>,
    ) -> Result<GrpcResponse<PullMessagesResponse>, GrpcStatus> {
        require_scope(&request, "messaging:read")?;
        let req = request.into_inner();
        let tenant_id = parse_uuid_opt(&req.tenant_id)?;
        let project_id = parse_uuid_opt(&req.project_id)?;

        let query = cncore::platform::models::MessageQuery {
            tenant_id,
            project_id,
            topic: req.topic,
            limit: if req.limit == 0 {
                None
            } else {
                Some(req.limit)
            },
        };

        let messages = self
            .store
            .list_messages(query)
            .await
            .map_err(|e| GrpcStatus::internal(e.to_string()))?;

        let envelopes = messages
            .into_iter()
            .map(|msg| MessageEnvelope {
                message_id: msg.id.to_string(),
                tenant_id: msg.tenant_id.to_string(),
                project_id: msg.project_id.to_string(),
                topic: msg.topic,
                key: msg.key.unwrap_or_default(),
                payload: msg.payload,
                priority: match msg.priority {
                    cncore::platform::models::MessagePriority::Low => MessagePriority::Low as i32,
                    cncore::platform::models::MessagePriority::Normal => {
                        MessagePriority::Normal as i32
                    }
                    cncore::platform::models::MessagePriority::High => MessagePriority::High as i32,
                },
                published_at_unix_ms: msg.published_at.timestamp_millis(),
                attributes: msg.attributes,
            })
            .collect();

        Ok(GrpcResponse::new(PullMessagesResponse {
            messages: envelopes,
        }))
    }

    async fn ack_message(
        &self,
        request: GrpcRequest<AckMessageRequest>,
    ) -> Result<GrpcResponse<AckMessageResponse>, GrpcStatus> {
        require_scope(&request, "messaging:write")?;
        let req = request.into_inner();
        let message_id = Uuid::parse_str(&req.message_id)
            .map_err(|_| GrpcStatus::invalid_argument("invalid message_id"))?;

        self.store
            .ack_message(&req.topic, message_id)
            .await
            .map_err(|e| GrpcStatus::internal(e.to_string()))?;

        Ok(GrpcResponse::new(AckMessageResponse { success: true }))
    }
}
