-- Migration: 0011_tasks_workflows.sql
-- Creates tables for task queue and workflow management.
-- Tasks are discrete units of work. Workflows orchestrate multiple tasks.

-- Task status enum type for better query performance
DO $$ BEGIN
    CREATE TYPE task_status AS ENUM ('pending', 'in_progress', 'completed', 'failed');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    kind VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    status task_status NOT NULL DEFAULT 'pending',
    priority INTEGER NOT NULL DEFAULT 0,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    last_error TEXT,
    result JSONB,
    timeouts JSONB,
    -- Leasing fields for distributed task processing
    leased_by UUID,
    lease_expires_at TIMESTAMPTZ,
    lease_version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_tasks_tenant ON tasks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_kind ON tasks(kind);
CREATE INDEX IF NOT EXISTS idx_tasks_scheduled ON tasks(scheduled_at) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_tasks_leased ON tasks(leased_by) WHERE leased_by IS NOT NULL;

-- Workflow definition table
CREATE TABLE IF NOT EXISTS workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    steps JSONB NOT NULL DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_workflows_tenant ON workflows(tenant_id);
CREATE INDEX IF NOT EXISTS idx_workflows_active ON workflows(is_active) WHERE is_active = true;

-- Workflow run status enum
DO $$ BEGIN
    CREATE TYPE workflow_run_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- Workflow execution runs
CREATE TABLE IF NOT EXISTS workflow_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    status workflow_run_status NOT NULL DEFAULT 'pending',
    current_step UUID,
    context JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_workflow_runs_tenant ON workflow_runs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_workflow_runs_workflow ON workflow_runs(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_runs_status ON workflow_runs(status);

-- Step execution tracking within a workflow run
CREATE TABLE IF NOT EXISTS workflow_step_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    step_id UUID NOT NULL,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    status workflow_run_status NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error TEXT
);

CREATE INDEX IF NOT EXISTS idx_workflow_step_runs_run ON workflow_step_runs(run_id);
CREATE INDEX IF NOT EXISTS idx_workflow_step_runs_task ON workflow_step_runs(task_id);

COMMENT ON TABLE tasks IS 'Task queue for discrete work units';
COMMENT ON TABLE workflows IS 'Workflow definitions with steps';
COMMENT ON TABLE workflow_runs IS 'Workflow execution instances';
COMMENT ON TABLE workflow_step_runs IS 'Individual step executions within a workflow run';
