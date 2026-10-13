-- Migration: 0010_tenants_projects.sql
-- Creates tenants and projects tables for multi-tenancy support.
-- Tenants are the top-level isolation boundary (e.g., companies/customers).
-- Projects group agents, content, and workflows within a tenant.

CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(128) NOT NULL UNIQUE,
    description TEXT,
    settings JSONB NOT NULL DEFAULT '{}',
    quota_agents INTEGER NOT NULL DEFAULT 100,
    quota_storage_bytes BIGINT NOT NULL DEFAULT 10737418240, -- 10 GB default
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_tenants_slug ON tenants(slug);
CREATE INDEX IF NOT EXISTS idx_tenants_active ON tenants(is_active) WHERE is_active = true;

CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(128) NOT NULL,
    description TEXT,
    settings JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(tenant_id, slug)
);

CREATE INDEX IF NOT EXISTS idx_projects_tenant ON projects(tenant_id);
CREATE INDEX IF NOT EXISTS idx_projects_active ON projects(is_active) WHERE is_active = true;

-- Update nodes table to reference projects (if not already constrained)
-- Note: nodes.tenant_id and nodes.project_id were added in 0005_agent_enhancements.sql
-- but without foreign key constraints. We add advisory comments for now as altering
-- existing data requires care.

-- Update users table to reference tenants (add FK if missing)
-- Note: users.tenant_id was added in 0007_users_auth.sql without FK constraint.
-- Production deployments may need to add: ALTER TABLE users ADD CONSTRAINT fk_users_tenant 
--   FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE SET NULL;

COMMENT ON TABLE tenants IS 'Top-level tenant isolation boundary';
COMMENT ON TABLE projects IS 'Logical grouping of agents, content, and workflows within a tenant';
