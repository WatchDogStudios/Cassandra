//! Operator-portal resources: build artifacts, the script library, jobs
//! dispatched to agents (commands, script runs, deployments), per-agent config
//! and platform settings.
use crate::platform::error::{PlatformError, PlatformResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Upper bound on captured stdout/stderr per job, in bytes.
pub const MAX_JOB_OUTPUT_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildRecord {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub platform: String,
    pub architecture: String,
    pub filename: String,
    pub size_bytes: u64,
    pub checksum_sha256: String,
    pub uploaded_by: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptRecord {
    pub id: Uuid,
    pub name: String,
    pub language: String,
    pub content: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub version: i32,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Waiting for the agent's next heartbeat.
    Pending,
    /// Handed to the agent; awaiting its result.
    Dispatched,
    Completed,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Dispatched => "dispatched",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> PlatformResult<Self> {
        match value {
            "pending" => Ok(JobStatus::Pending),
            "dispatched" => Ok(JobStatus::Dispatched),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            _ => Err(PlatformError::Internal("invalid job status")),
        }
    }
}

/// Work for one agent: an operator command, a script run or a build deployment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentJob {
    pub id: Uuid,
    pub agent_id: String,
    /// restart | shutdown | exec | collect_logs | deploy_build | script
    pub kind: String,
    pub args: Value,
    /// Set when `kind == "script"`.
    pub script_id: Option<Uuid>,
    pub timeout_seconds: u32,
    pub status: JobStatus,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub error: Option<String>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub dispatched_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl AgentJob {
    pub fn new(agent_id: impl Into<String>, kind: impl Into<String>, args: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id: agent_id.into(),
            kind: kind.into(),
            args,
            script_id: None,
            timeout_seconds: 300,
            status: JobStatus::Pending,
            exit_code: None,
            stdout: None,
            stderr: None,
            error: None,
            created_by: None,
            created_at: Utc::now(),
            dispatched_at: None,
            completed_at: None,
        }
    }
}

/// What an agent reports back for a dispatched job.
#[derive(Debug, Clone, Default)]
pub struct JobOutcome {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct JobQuery {
    pub agent_id: Option<String>,
    pub script_id: Option<Uuid>,
    pub limit: Option<u32>,
}

#[async_trait]
pub trait PortalStore: Send + Sync {
    async fn insert_build(&self, build: BuildRecord) -> PlatformResult<()>;
    async fn list_builds(&self) -> PlatformResult<Vec<BuildRecord>>;
    async fn get_build(&self, id: Uuid) -> PlatformResult<Option<BuildRecord>>;
    async fn delete_build(&self, id: Uuid) -> PlatformResult<bool>;

    /// Insert, or replace an existing script with the same id.
    async fn save_script(&self, script: ScriptRecord) -> PlatformResult<()>;
    async fn list_scripts(&self) -> PlatformResult<Vec<ScriptRecord>>;
    async fn get_script(&self, id: Uuid) -> PlatformResult<Option<ScriptRecord>>;
    async fn delete_script(&self, id: Uuid) -> PlatformResult<bool>;

    async fn insert_jobs(&self, jobs: Vec<AgentJob>) -> PlatformResult<()>;
    /// Newest first.
    async fn list_jobs(&self, query: JobQuery) -> PlatformResult<Vec<AgentJob>>;
    /// Atomically move up to `limit` of the agent's oldest pending jobs to dispatched.
    async fn claim_jobs(&self, agent_id: &str, limit: u32) -> PlatformResult<Vec<AgentJob>>;
    /// Record the result of a dispatched job. Only the job's own agent may report it.
    async fn complete_job(
        &self,
        id: Uuid,
        agent_id: &str,
        outcome: JobOutcome,
    ) -> PlatformResult<AgentJob>;

    async fn get_agent_config(&self, agent_id: &str) -> PlatformResult<Option<Value>>;
    async fn put_agent_config(&self, agent_id: &str, config: Value) -> PlatformResult<()>;
    async fn get_settings(&self) -> PlatformResult<Option<Value>>;
    async fn put_settings(&self, settings: Value) -> PlatformResult<()>;
}

/// Truncate to at most `MAX_JOB_OUTPUT_BYTES`, on a char boundary.
pub fn clamp_output(mut text: String) -> String {
    if text.len() > MAX_JOB_OUTPUT_BYTES {
        let mut end = MAX_JOB_OUTPUT_BYTES;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
    text
}

fn apply_outcome(job: &mut AgentJob, outcome: JobOutcome) {
    job.status = if outcome.success {
        JobStatus::Completed
    } else {
        JobStatus::Failed
    };
    job.exit_code = outcome.exit_code;
    job.stdout = Some(clamp_output(outcome.stdout));
    job.stderr = Some(clamp_output(outcome.stderr));
    job.error = outcome.error;
    job.completed_at = Some(Utc::now());
}

#[derive(Default)]
pub struct InMemoryPortalStore {
    state: RwLock<PortalState>,
}

#[derive(Default)]
struct PortalState {
    builds: HashMap<Uuid, BuildRecord>,
    scripts: HashMap<Uuid, ScriptRecord>,
    jobs: HashMap<Uuid, AgentJob>,
    agent_configs: HashMap<String, Value>,
    settings: Option<Value>,
}

impl InMemoryPortalStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl PortalStore for InMemoryPortalStore {
    async fn insert_build(&self, build: BuildRecord) -> PlatformResult<()> {
        let mut state = self.state.write();
        if state.builds.contains_key(&build.id) {
            return Err(PlatformError::Conflict("build"));
        }
        state.builds.insert(build.id, build);
        Ok(())
    }

    async fn list_builds(&self) -> PlatformResult<Vec<BuildRecord>> {
        let mut builds: Vec<_> = self.state.read().builds.values().cloned().collect();
        builds.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        Ok(builds)
    }

    async fn get_build(&self, id: Uuid) -> PlatformResult<Option<BuildRecord>> {
        Ok(self.state.read().builds.get(&id).cloned())
    }

    async fn delete_build(&self, id: Uuid) -> PlatformResult<bool> {
        Ok(self.state.write().builds.remove(&id).is_some())
    }

    async fn save_script(&self, script: ScriptRecord) -> PlatformResult<()> {
        self.state.write().scripts.insert(script.id, script);
        Ok(())
    }

    async fn list_scripts(&self) -> PlatformResult<Vec<ScriptRecord>> {
        let mut scripts: Vec<_> = self.state.read().scripts.values().cloned().collect();
        scripts.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(scripts)
    }

    async fn get_script(&self, id: Uuid) -> PlatformResult<Option<ScriptRecord>> {
        Ok(self.state.read().scripts.get(&id).cloned())
    }

    async fn delete_script(&self, id: Uuid) -> PlatformResult<bool> {
        let mut state = self.state.write();
        let removed = state.scripts.remove(&id).is_some();
        if removed {
            // Mirrors ON DELETE SET NULL in Postgres.
            for job in state.jobs.values_mut() {
                if job.script_id == Some(id) {
                    job.script_id = None;
                }
            }
        }
        Ok(removed)
    }

    async fn insert_jobs(&self, jobs: Vec<AgentJob>) -> PlatformResult<()> {
        let mut state = self.state.write();
        for job in jobs {
            state.jobs.insert(job.id, job);
        }
        Ok(())
    }

    async fn list_jobs(&self, query: JobQuery) -> PlatformResult<Vec<AgentJob>> {
        let state = self.state.read();
        let mut jobs: Vec<_> = state
            .jobs
            .values()
            .filter(|j| query.agent_id.as_ref().is_none_or(|a| &j.agent_id == a))
            .filter(|j| query.script_id.is_none_or(|s| j.script_id == Some(s)))
            .cloned()
            .collect();
        jobs.sort_by_key(|j| std::cmp::Reverse(j.created_at));
        if let Some(limit) = query.limit {
            jobs.truncate(limit as usize);
        }
        Ok(jobs)
    }

    async fn claim_jobs(&self, agent_id: &str, limit: u32) -> PlatformResult<Vec<AgentJob>> {
        let mut state = self.state.write();
        let mut pending: Vec<_> = state
            .jobs
            .values()
            .filter(|j| j.agent_id == agent_id && j.status == JobStatus::Pending)
            .map(|j| (j.created_at, j.id))
            .collect();
        pending.sort();
        let now = Utc::now();
        let mut claimed = Vec::new();
        for (_, id) in pending.into_iter().take(limit as usize) {
            let job = state.jobs.get_mut(&id).expect("id came from this map");
            job.status = JobStatus::Dispatched;
            job.dispatched_at = Some(now);
            claimed.push(job.clone());
        }
        Ok(claimed)
    }

    async fn complete_job(
        &self,
        id: Uuid,
        agent_id: &str,
        outcome: JobOutcome,
    ) -> PlatformResult<AgentJob> {
        let mut state = self.state.write();
        let job = state
            .jobs
            .get_mut(&id)
            .filter(|j| j.agent_id == agent_id)
            .ok_or(PlatformError::NotFound("job"))?;
        if job.status != JobStatus::Dispatched {
            return Err(PlatformError::Conflict("job not awaiting a result"));
        }
        apply_outcome(job, outcome);
        Ok(job.clone())
    }

    async fn get_agent_config(&self, agent_id: &str) -> PlatformResult<Option<Value>> {
        Ok(self.state.read().agent_configs.get(agent_id).cloned())
    }

    async fn put_agent_config(&self, agent_id: &str, config: Value) -> PlatformResult<()> {
        self.state
            .write()
            .agent_configs
            .insert(agent_id.to_string(), config);
        Ok(())
    }

    async fn get_settings(&self) -> PlatformResult<Option<Value>> {
        Ok(self.state.read().settings.clone())
    }

    async fn put_settings(&self, settings: Value) -> PlatformResult<()> {
        self.state.write().settings = Some(settings);
        Ok(())
    }
}

#[cfg(feature = "db")]
pub use postgres::PostgresPortalStore;

#[cfg(feature = "db")]
mod postgres {
    use super::*;
    use sqlx::{Pool, Postgres};

    pub struct PostgresPortalStore {
        pool: Pool<Postgres>,
    }

    impl PostgresPortalStore {
        pub fn new(pool: Pool<Postgres>) -> Self {
            Self { pool }
        }
    }

    const BUILD_COLUMNS: &str = "id, name, version, platform, architecture, filename, size_bytes, \
         checksum_sha256, uploaded_by, metadata, created_at";
    const SCRIPT_COLUMNS: &str =
        "id, name, language, content, description, tags, version, created_by, created_at, updated_at";
    const JOB_COLUMNS: &str = "id, agent_id, kind, args, script_id, timeout_seconds, status, \
         exit_code, stdout, stderr, error, created_by, created_at, dispatched_at, completed_at";

    #[derive(sqlx::FromRow)]
    struct BuildRow {
        id: Uuid,
        name: String,
        version: String,
        platform: String,
        architecture: String,
        filename: String,
        size_bytes: i64,
        checksum_sha256: String,
        uploaded_by: Option<String>,
        metadata: Value,
        created_at: DateTime<Utc>,
    }

    impl BuildRow {
        fn into_model(self) -> PlatformResult<BuildRecord> {
            Ok(BuildRecord {
                id: self.id,
                name: self.name,
                version: self.version,
                platform: self.platform,
                architecture: self.architecture,
                filename: self.filename,
                size_bytes: self.size_bytes.max(0) as u64,
                checksum_sha256: self.checksum_sha256,
                uploaded_by: self.uploaded_by,
                metadata: serde_json::from_value(self.metadata)
                    .map_err(|_| PlatformError::Internal("invalid build metadata"))?,
                created_at: self.created_at,
            })
        }
    }

    #[derive(sqlx::FromRow)]
    struct ScriptRow {
        id: Uuid,
        name: String,
        language: String,
        content: String,
        description: Option<String>,
        tags: Vec<String>,
        version: i32,
        created_by: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    impl From<ScriptRow> for ScriptRecord {
        fn from(r: ScriptRow) -> Self {
            ScriptRecord {
                id: r.id,
                name: r.name,
                language: r.language,
                content: r.content,
                description: r.description,
                tags: r.tags,
                version: r.version,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
            }
        }
    }

    #[derive(sqlx::FromRow)]
    struct JobRow {
        id: Uuid,
        agent_id: String,
        kind: String,
        args: Value,
        script_id: Option<Uuid>,
        timeout_seconds: i32,
        status: String,
        exit_code: Option<i32>,
        stdout: Option<String>,
        stderr: Option<String>,
        error: Option<String>,
        created_by: Option<String>,
        created_at: DateTime<Utc>,
        dispatched_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
    }

    impl JobRow {
        fn into_model(self) -> PlatformResult<AgentJob> {
            Ok(AgentJob {
                id: self.id,
                agent_id: self.agent_id,
                kind: self.kind,
                args: self.args,
                script_id: self.script_id,
                timeout_seconds: self.timeout_seconds.max(0) as u32,
                status: JobStatus::parse(&self.status)?,
                exit_code: self.exit_code,
                stdout: self.stdout,
                stderr: self.stderr,
                error: self.error,
                created_by: self.created_by,
                created_at: self.created_at,
                dispatched_at: self.dispatched_at,
                completed_at: self.completed_at,
            })
        }
    }

    fn jobs(rows: Vec<JobRow>) -> PlatformResult<Vec<AgentJob>> {
        rows.into_iter().map(JobRow::into_model).collect()
    }

    #[async_trait]
    impl PortalStore for PostgresPortalStore {
        async fn insert_build(&self, b: BuildRecord) -> PlatformResult<()> {
            let metadata = serde_json::to_value(&b.metadata)
                .map_err(|_| PlatformError::InvalidInput("invalid metadata"))?;
            sqlx::query(&format!(
                "INSERT INTO builds ({BUILD_COLUMNS}) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"
            ))
            .bind(b.id)
            .bind(b.name)
            .bind(b.version)
            .bind(b.platform)
            .bind(b.architecture)
            .bind(b.filename)
            .bind(
                i64::try_from(b.size_bytes)
                    .map_err(|_| PlatformError::InvalidInput("build too large"))?,
            )
            .bind(b.checksum_sha256)
            .bind(b.uploaded_by)
            .bind(metadata)
            .bind(b.created_at)
            .execute(&self.pool)
            .await?;
            Ok(())
        }

        async fn list_builds(&self) -> PlatformResult<Vec<BuildRecord>> {
            let rows: Vec<BuildRow> = sqlx::query_as(&format!(
                "SELECT {BUILD_COLUMNS} FROM builds ORDER BY created_at DESC"
            ))
            .fetch_all(&self.pool)
            .await?;
            rows.into_iter().map(BuildRow::into_model).collect()
        }

        async fn get_build(&self, id: Uuid) -> PlatformResult<Option<BuildRecord>> {
            let row: Option<BuildRow> =
                sqlx::query_as(&format!("SELECT {BUILD_COLUMNS} FROM builds WHERE id = $1"))
                    .bind(id)
                    .fetch_optional(&self.pool)
                    .await?;
            row.map(BuildRow::into_model).transpose()
        }

        async fn delete_build(&self, id: Uuid) -> PlatformResult<bool> {
            let result = sqlx::query("DELETE FROM builds WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await?;
            Ok(result.rows_affected() > 0)
        }

        async fn save_script(&self, s: ScriptRecord) -> PlatformResult<()> {
            sqlx::query(&format!(
                "INSERT INTO scripts ({SCRIPT_COLUMNS}) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
                 ON CONFLICT (id) DO UPDATE SET
                   name = EXCLUDED.name, language = EXCLUDED.language,
                   content = EXCLUDED.content, description = EXCLUDED.description,
                   tags = EXCLUDED.tags, version = EXCLUDED.version,
                   updated_at = EXCLUDED.updated_at"
            ))
            .bind(s.id)
            .bind(s.name)
            .bind(s.language)
            .bind(s.content)
            .bind(s.description)
            .bind(s.tags)
            .bind(s.version)
            .bind(s.created_by)
            .bind(s.created_at)
            .bind(s.updated_at)
            .execute(&self.pool)
            .await?;
            Ok(())
        }

        async fn list_scripts(&self) -> PlatformResult<Vec<ScriptRecord>> {
            let rows: Vec<ScriptRow> = sqlx::query_as(&format!(
                "SELECT {SCRIPT_COLUMNS} FROM scripts ORDER BY name"
            ))
            .fetch_all(&self.pool)
            .await?;
            Ok(rows.into_iter().map(Into::into).collect())
        }

        async fn get_script(&self, id: Uuid) -> PlatformResult<Option<ScriptRecord>> {
            let row: Option<ScriptRow> = sqlx::query_as(&format!(
                "SELECT {SCRIPT_COLUMNS} FROM scripts WHERE id = $1"
            ))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
            Ok(row.map(Into::into))
        }

        async fn delete_script(&self, id: Uuid) -> PlatformResult<bool> {
            let result = sqlx::query("DELETE FROM scripts WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await?;
            Ok(result.rows_affected() > 0)
        }

        async fn insert_jobs(&self, jobs: Vec<AgentJob>) -> PlatformResult<()> {
            let mut tx = self.pool.begin().await?;
            for j in jobs {
                sqlx::query(&format!(
                    "INSERT INTO agent_jobs ({JOB_COLUMNS})
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)"
                ))
                .bind(j.id)
                .bind(j.agent_id)
                .bind(j.kind)
                .bind(j.args)
                .bind(j.script_id)
                .bind(i32::try_from(j.timeout_seconds).unwrap_or(i32::MAX))
                .bind(j.status.as_str())
                .bind(j.exit_code)
                .bind(j.stdout)
                .bind(j.stderr)
                .bind(j.error)
                .bind(j.created_by)
                .bind(j.created_at)
                .bind(j.dispatched_at)
                .bind(j.completed_at)
                .execute(&mut *tx)
                .await?;
            }
            tx.commit().await?;
            Ok(())
        }

        async fn list_jobs(&self, query: JobQuery) -> PlatformResult<Vec<AgentJob>> {
            let rows: Vec<JobRow> = sqlx::query_as(&format!(
                "SELECT {JOB_COLUMNS} FROM agent_jobs
                 WHERE ($1::text IS NULL OR agent_id = $1)
                   AND ($2::uuid IS NULL OR script_id = $2)
                 ORDER BY created_at DESC
                 LIMIT $3"
            ))
            .bind(query.agent_id)
            .bind(query.script_id)
            .bind(query.limit.map(i64::from)) // NULL means no limit
            .fetch_all(&self.pool)
            .await?;
            jobs(rows)
        }

        async fn claim_jobs(&self, agent_id: &str, limit: u32) -> PlatformResult<Vec<AgentJob>> {
            // SKIP LOCKED keeps two concurrent heartbeats from claiming the same job.
            let rows: Vec<JobRow> = sqlx::query_as(&format!(
                "UPDATE agent_jobs SET status = 'dispatched', dispatched_at = NOW()
                 WHERE id IN (
                   SELECT id FROM agent_jobs
                   WHERE agent_id = $1 AND status = 'pending'
                   ORDER BY created_at
                   LIMIT $2
                   FOR UPDATE SKIP LOCKED
                 )
                 RETURNING {JOB_COLUMNS}"
            ))
            .bind(agent_id)
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await?;
            let mut claimed = jobs(rows)?;
            claimed.sort_by_key(|j| j.created_at);
            Ok(claimed)
        }

        async fn complete_job(
            &self,
            id: Uuid,
            agent_id: &str,
            outcome: JobOutcome,
        ) -> PlatformResult<AgentJob> {
            let mut job = AgentJob::new(agent_id, "", Value::Null);
            apply_outcome(&mut job, outcome);
            let row: Option<JobRow> = sqlx::query_as(&format!(
                "UPDATE agent_jobs SET status = $3, exit_code = $4, stdout = $5, stderr = $6,
                   error = $7, completed_at = $8
                 WHERE id = $1 AND agent_id = $2 AND status = 'dispatched'
                 RETURNING {JOB_COLUMNS}"
            ))
            .bind(id)
            .bind(agent_id)
            .bind(job.status.as_str())
            .bind(job.exit_code)
            .bind(job.stdout)
            .bind(job.stderr)
            .bind(job.error)
            .bind(job.completed_at)
            .fetch_optional(&self.pool)
            .await?;
            match row {
                Some(row) => row.into_model(),
                // Unknown, someone else's, or already finished: indistinguishable
                // to the caller on purpose.
                None => Err(PlatformError::NotFound("job")),
            }
        }

        async fn get_agent_config(&self, agent_id: &str) -> PlatformResult<Option<Value>> {
            let row: Option<(Value,)> =
                sqlx::query_as("SELECT config FROM agent_configs WHERE agent_id = $1")
                    .bind(agent_id)
                    .fetch_optional(&self.pool)
                    .await?;
            Ok(row.map(|(v,)| v))
        }

        async fn put_agent_config(&self, agent_id: &str, config: Value) -> PlatformResult<()> {
            sqlx::query(
                "INSERT INTO agent_configs (agent_id, config) VALUES ($1, $2)
                 ON CONFLICT (agent_id) DO UPDATE SET config = EXCLUDED.config, updated_at = NOW()",
            )
            .bind(agent_id)
            .bind(config)
            .execute(&self.pool)
            .await?;
            Ok(())
        }

        async fn get_settings(&self) -> PlatformResult<Option<Value>> {
            let row: Option<(Value,)> =
                sqlx::query_as("SELECT settings FROM platform_settings WHERE id")
                    .fetch_optional(&self.pool)
                    .await?;
            Ok(row.map(|(v,)| v))
        }

        async fn put_settings(&self, settings: Value) -> PlatformResult<()> {
            sqlx::query(
                "INSERT INTO platform_settings (id, settings) VALUES (TRUE, $1)
                 ON CONFLICT (id) DO UPDATE SET settings = EXCLUDED.settings, updated_at = NOW()",
            )
            .bind(settings)
            .execute(&self.pool)
            .await?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn jobs_are_claimed_once_in_order_and_completed_by_their_agent() {
        let store = InMemoryPortalStore::new();
        let mut first = AgentJob::new("a1", "exec", json!({"command": "true"}));
        first.created_at = Utc::now() - chrono::Duration::seconds(5);
        let second = AgentJob::new("a1", "restart", json!({}));
        let other = AgentJob::new("a2", "restart", json!({}));
        store
            .insert_jobs(vec![second.clone(), first.clone(), other.clone()])
            .await
            .unwrap();

        let claimed = store.claim_jobs("a1", 10).await.unwrap();
        let ids: Vec<_> = claimed.iter().map(|j| j.id).collect();
        assert_eq!(
            ids,
            vec![first.id, second.id],
            "oldest first, own jobs only"
        );
        assert!(
            store.claim_jobs("a1", 10).await.unwrap().is_empty(),
            "claimed once"
        );

        // Another agent cannot report this agent's job.
        let stolen = store
            .complete_job(first.id, "a2", JobOutcome::default())
            .await;
        assert!(matches!(stolen, Err(PlatformError::NotFound(_))));

        let done = store
            .complete_job(
                first.id,
                "a1",
                JobOutcome {
                    success: true,
                    exit_code: Some(0),
                    stdout: "x".repeat(MAX_JOB_OUTPUT_BYTES + 10),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(done.status, JobStatus::Completed);
        assert_eq!(done.stdout.unwrap().len(), MAX_JOB_OUTPUT_BYTES);

        let again = store
            .complete_job(first.id, "a1", JobOutcome::default())
            .await;
        assert!(
            matches!(again, Err(PlatformError::Conflict(_))),
            "single result"
        );

        // Pending jobs can't be completed before they were handed out.
        let early = store
            .complete_job(other.id, "a2", JobOutcome::default())
            .await;
        assert!(matches!(early, Err(PlatformError::Conflict(_))));
    }

    #[test]
    fn clamp_output_respects_char_boundaries() {
        let text = "é".repeat(MAX_JOB_OUTPUT_BYTES); // 2 bytes each
        let clamped = clamp_output(text);
        assert!(clamped.len() <= MAX_JOB_OUTPUT_BYTES);
        assert!(clamped.chars().all(|c| c == 'é'));
    }
}
