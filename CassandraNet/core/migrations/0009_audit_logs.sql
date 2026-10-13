-- Audit logs table for persistent audit trail
-- Retention handled by external process (not auto-cleaned)

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Event classification
    event_kind TEXT NOT NULL,
    level TEXT NOT NULL DEFAULT 'info',
    -- Actor information (who performed the action)
    actor_id UUID,
    actor_type TEXT,  -- 'user', 'service', 'api_key', 'system'
    -- Context
    tenant_id UUID,
    ip_address INET,
    -- Event details as JSON
    payload JSONB NOT NULL DEFAULT '{}',
    -- Timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Primary query pattern: recent events by time
CREATE INDEX IF NOT EXISTS idx_audit_logs_created ON audit_logs(created_at DESC);

-- Query by tenant for compliance/security reviews
CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant ON audit_logs(tenant_id, created_at DESC) WHERE tenant_id IS NOT NULL;

-- Query by actor for user activity tracking
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor ON audit_logs(actor_id, created_at DESC) WHERE actor_id IS NOT NULL;

-- Query by event kind for monitoring specific actions
CREATE INDEX IF NOT EXISTS idx_audit_logs_kind ON audit_logs(event_kind, created_at DESC);

-- GIN index for JSONB payload queries
CREATE INDEX IF NOT EXISTS idx_audit_logs_payload ON audit_logs USING GIN (payload);

COMMENT ON TABLE audit_logs IS 'Immutable audit trail for security and compliance. Retention managed externally.';
COMMENT ON COLUMN audit_logs.event_kind IS 'Hierarchical event name (e.g., auth.login.success, ugc.upload.created)';
COMMENT ON COLUMN audit_logs.level IS 'Severity level: info, warn, error';
COMMENT ON COLUMN audit_logs.actor_type IS 'Type of actor: user, service, api_key, system';
