-- API Keys table for database-backed API key management
-- Replaces CASS_API_KEYS environment variable for production use

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    label TEXT NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    -- First 8 chars of the key for lookup (e.g., "cass_abc1")
    token_prefix VARCHAR(12) UNIQUE NOT NULL,
    -- Argon2 hash of the full token
    token_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at TIMESTAMPTZ,
    -- Key rotation tracking
    rotated_from UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    rotated_to UUID REFERENCES api_keys(id) ON DELETE SET NULL
);

-- Index for fast lookups by prefix (primary auth path)
CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys(token_prefix) WHERE deleted_at IS NULL AND revoked = FALSE;

-- Index for listing keys by tenant
CREATE INDEX IF NOT EXISTS idx_api_keys_tenant ON api_keys(tenant_id) WHERE deleted_at IS NULL;

-- Index for cleanup of soft-deleted keys
CREATE INDEX IF NOT EXISTS idx_api_keys_deleted ON api_keys(deleted_at) WHERE deleted_at IS NOT NULL;

COMMENT ON TABLE api_keys IS 'Database-backed API keys for service authentication';
COMMENT ON COLUMN api_keys.token_prefix IS 'First 8-12 chars of the key for O(1) lookup without storing the full key';
COMMENT ON COLUMN api_keys.token_hash IS 'Argon2 hash of the full API key token';
COMMENT ON COLUMN api_keys.scopes IS 'Array of permission scopes (e.g., admin, agent:read, ugc:write)';
