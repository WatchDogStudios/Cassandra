-- Additional data model tables
CREATE TABLE IF NOT EXISTS titles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, name)
);
CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY,
    hostname TEXT NOT NULL,
    os TEXT,
    arch TEXT,
    cpu_cores INT,
    memory_bytes BIGINT,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
