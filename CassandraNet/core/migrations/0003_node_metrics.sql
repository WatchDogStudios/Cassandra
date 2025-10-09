-- Per-heartbeat metrics for nodes
CREATE TABLE IF NOT EXISTS node_metrics (
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    ts TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cpu_percent DOUBLE PRECISION NOT NULL,
    memory_used_bytes BIGINT NOT NULL,
    PRIMARY KEY (node_id, ts)
);
