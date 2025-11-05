-- Orchestration assignments store
CREATE TABLE IF NOT EXISTS orchestration_assignments (
    id UUID PRIMARY KEY,
    agent_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    workload_id TEXT NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    status TEXT NOT NULL,
    status_message TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orchestration_assignments_agent
    ON orchestration_assignments(agent_id);
CREATE INDEX IF NOT EXISTS idx_orchestration_assignments_tenant
    ON orchestration_assignments(tenant_id);
CREATE INDEX IF NOT EXISTS idx_orchestration_assignments_project
    ON orchestration_assignments(project_id);
CREATE INDEX IF NOT EXISTS idx_orchestration_assignments_status
    ON orchestration_assignments(status);

-- UGC moderation content store
CREATE TABLE IF NOT EXISTS ugc_moderation_content (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    mime_type TEXT,
    size_bytes BIGINT,
    state TEXT NOT NULL,
    reason TEXT,
    labels JSONB NOT NULL DEFAULT '{}'::jsonb,
    attributes JSONB NOT NULL DEFAULT '{}'::jsonb,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ugc_moderation_content_tenant
    ON ugc_moderation_content(tenant_id);
CREATE INDEX IF NOT EXISTS idx_ugc_moderation_content_project
    ON ugc_moderation_content(project_id);
CREATE INDEX IF NOT EXISTS idx_ugc_moderation_content_state
    ON ugc_moderation_content(state);

-- Messaging queue store
CREATE TABLE IF NOT EXISTS messaging_messages (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    topic TEXT NOT NULL,
    key TEXT,
    payload BYTEA NOT NULL,
    priority TEXT NOT NULL,
    attributes JSONB NOT NULL DEFAULT '{}'::jsonb,
    published_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_messaging_messages_topic
    ON messaging_messages(topic);
CREATE INDEX IF NOT EXISTS idx_messaging_messages_tenant_project
    ON messaging_messages(tenant_id, project_id);
CREATE INDEX IF NOT EXISTS idx_messaging_messages_published
    ON messaging_messages(published_at DESC);
