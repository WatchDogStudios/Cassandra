use crate::state::AgentSummary;
use anyhow::Result;
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct RedisAgentStore {
    pool: Pool,
}

impl RedisAgentStore {
    pub fn new(url: &str) -> Result<Self> {
        let cfg = Config::from_url(url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(Self { pool })
    }

    pub async fn register_heartbeat(&self, agent: AgentSummary) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 1. Store agent metadata
        let key = format!("agent:{}", agent.id);
        let data = serde_json::to_string(&agent)?;
        let _: () = conn.set_ex(&key, data, 60).await?; // Expire metadata after 60s

        // 2. Update active set (ZSET) with timestamp score
        let _: () = conn.zadd("agents:active", now, &agent.id).await?;

        Ok(())
    }

    pub async fn list_active_agents(&self) -> Result<Vec<AgentSummary>> {
        let mut conn = self.pool.get().await?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let threshold = now - 30; // Agents seen in last 30s

        // 1. Remove expired agents from ZSET
        let _: () = conn.zrembyscore("agents:active", "-inf", threshold).await?;

        // 2. Get active agent IDs
        let ids: Vec<String> = conn.zrange("agents:active", 0, -1).await?;

        // 3. Fetch metadata for each ID
        let mut agents = Vec::new();
        for id in ids {
            let key = format!("agent:{}", id);
            if let Ok(data) = conn.get::<_, String>(&key).await {
                if let Ok(agent) = serde_json::from_str::<AgentSummary>(&data) {
                    agents.push(agent);
                }
            }
        }

        Ok(agents)
    }
}
