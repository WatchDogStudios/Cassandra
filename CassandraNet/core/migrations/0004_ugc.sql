-- UGC upload sessions and metadata catalog
CREATE TABLE IF NOT EXISTS ugc_upload_sessions (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    content_id UUID NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    upload_url TEXT,
    headers JSONB NOT NULL DEFAULT '{}'::JSONB
);

CREATE INDEX IF NOT EXISTS idx_ugc_upload_sessions_tenant_project
    ON ugc_upload_sessions (tenant_id, project_id);

CREATE TABLE IF NOT EXISTS ugc_content_metadata (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    filename TEXT NOT NULL,
    mime_type TEXT,
    size_bytes BIGINT,
    checksum TEXT,
    storage_path TEXT,
    labels TEXT[] NOT NULL DEFAULT '{}',
    attributes JSONB NOT NULL DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    uploaded_by UUID,
    visibility TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ugc_content_metadata_tenant_project
    ON ugc_content_metadata (tenant_id, project_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_ugc_content_metadata_labels
    ON ugc_content_metadata USING GIN (labels);

CREATE INDEX IF NOT EXISTS idx_ugc_content_metadata_attributes
    ON ugc_content_metadata USING GIN (attributes);
