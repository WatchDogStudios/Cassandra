-- Extend the users table created by 0001 (id, email, password_hash, created_at)
-- with the columns the auth service needs, and seed the default admin.
-- users.tenant_id deliberately has no FK (see 0010).

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS display_name TEXT,
    ADD COLUMN IF NOT EXISTS tenant_id UUID,
    ADD COLUMN IF NOT EXISTS scopes TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE;

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);

-- Placeholder admin with an unusable password hash; the gateway sets the real
-- hash from CASS_ADMIN_PASSWORD on startup. Well-known id so startup can upsert.
INSERT INTO users (id, email, password_hash, display_name, scopes, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'admin@localhost',
    '$argon2id$v=19$m=19456,t=2,p=1$placeholder$placeholder',
    'Administrator',
    ARRAY['admin:read','admin:write'],
    TRUE
)
ON CONFLICT (id) DO NOTHING;
