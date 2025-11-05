export interface AgentSummary {
  id: string;
  hostname: string;
  cpu_percent: number;
  memory_used_bytes: number;
  last_seen_unix_ms: number;
  tenant_id?: string | null;
  project_id?: string | null;
  lifecycle_status?: string | null;
}

export interface HealthResponse {
  status: string;
}

export interface AgentMetricHistory {
  cpu: number[];
  memory: number[];
  timestamps: number[];
}
