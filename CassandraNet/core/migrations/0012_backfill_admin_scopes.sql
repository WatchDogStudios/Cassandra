-- Migration: 0012_backfill_admin_scopes.sql
-- Backfill legacy scope string "admin" to fine-grained admin scopes.
--
-- Rationale:
-- Earlier deployments seeded users (and sometimes API keys) with scopes = ['admin'].
-- The platform now uses explicit admin scopes: 'admin:read' and 'admin:write'.
--
-- This migration is intentionally conservative:
-- - It does NOT grant 'docs:read' (docs access remains explicitly whitelisted).
-- - It does NOT remove the legacy 'admin' string (kept for a deprecation window).

-- Users: if a user has legacy 'admin', ensure they also have admin:read/admin:write
UPDATE users
SET scopes = (
    SELECT array_agg(DISTINCT s)
    FROM unnest(scopes || ARRAY['admin:read','admin:write']) AS s
)
WHERE scopes @> ARRAY['admin'];

-- API keys: if a key has legacy 'admin', ensure it also has admin:read/admin:write
UPDATE api_keys
SET scopes = (
    SELECT array_agg(DISTINCT s)
    FROM unnest(scopes || ARRAY['admin:read','admin:write']) AS s
)
WHERE scopes @> ARRAY['admin'];
