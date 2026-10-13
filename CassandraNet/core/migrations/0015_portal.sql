-- Operator portal: build artifacts, script library, jobs dispatched to agents,
-- per-agent config and platform settings.

CREATE TABLE IF NOT EXISTS builds (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    platform TEXT NOT NULL,
    architecture TEXT NOT NULL,
    filename TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    checksum_sha256 TEXT NOT NULL,
    uploaded_by TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_builds_created ON builds(created_at DESC);

CREATE TABLE IF NOT EXISTS scripts (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    language TEXT NOT NULL,
    content TEXT NOT NULL,
    description TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    version INTEGER NOT NULL DEFAULT 1,
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- agent_id is the agent's registered node id (free-form text, not a nodes FK:
-- agents can run against the in-memory or Redis registry).
CREATE TABLE IF NOT EXISTS agent_jobs (
    id UUID PRIMARY KEY,
    agent_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    args JSONB NOT NULL DEFAULT '{}'::jsonb,
    script_id UUID REFERENCES scripts(id) ON DELETE SET NULL,
    timeout_seconds INTEGER NOT NULL,
    status TEXT NOT NULL,
    exit_code INTEGER,
    stdout TEXT,
    stderr TEXT,
    error TEXT,
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    dispatched_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_agent_jobs_agent_pending ON agent_jobs(agent_id, created_at) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_agent_jobs_agent ON agent_jobs(agent_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_jobs_script ON agent_jobs(script_id, created_at DESC) WHERE script_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS agent_configs (
    agent_id TEXT PRIMARY KEY,
    config JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Single-row table: the CHECK pins id to TRUE.
CREATE TABLE IF NOT EXISTS platform_settings (
    id BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),
    settings JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
