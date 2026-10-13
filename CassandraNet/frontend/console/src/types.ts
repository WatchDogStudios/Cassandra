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

// Portal types

export interface AgentConfig {
  heartbeat_interval_seconds: number;
  log_level: string;
  auto_update_enabled: boolean;
  max_concurrent_tasks: number;
  metrics_enabled: boolean;
  custom_labels: Record<string, string>;
}

export interface AgentCommand {
  id: string;
  agent_id: string;
  command: string;
  args: Record<string, unknown>;
  status: 'pending' | 'dispatched' | 'completed' | 'failed';
  created_at: string;
  completed_at?: string | null;
  result?: { exit_code: number | null; stdout: string | null; stderr: string | null };
  error?: string | null;
}

export interface ApiKey {
  id: string;
  name: string;
  prefix: string;
  scopes: string[];
  created_at: string;
  expires_at?: string | null;
  last_used_at?: string | null;
  is_revoked: boolean;
}
