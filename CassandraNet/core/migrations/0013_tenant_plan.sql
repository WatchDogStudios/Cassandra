-- Migration: 0013_tenant_plan.sql
-- Adds a first-class "plan" field to tenants for onboarding and entitlement.

ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS plan VARCHAR(64) NOT NULL DEFAULT 'free';

CREATE INDEX IF NOT EXISTS idx_tenants_plan ON tenants(plan);
