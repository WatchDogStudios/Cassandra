-- Extend nodes schema for richer agent queries
ALTER TABLE nodes
    ADD COLUMN IF NOT EXISTS tenant_id UUID,
    ADD COLUMN IF NOT EXISTS project_id UUID,
    ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'registered',
    ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Maintain consistent defaults for existing rows
UPDATE nodes SET status = COALESCE(status, 'registered');

CREATE INDEX IF NOT EXISTS idx_nodes_tenant_status
    ON nodes (tenant_id, status);

CREATE INDEX IF NOT EXISTS idx_nodes_project
    ON nodes (project_id);

CREATE INDEX IF NOT EXISTS idx_nodes_last_seen
    ON nodes (last_seen DESC NULLS LAST);
