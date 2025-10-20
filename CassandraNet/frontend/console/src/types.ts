export interface AgentSummary {
  id: string;
  hostname: string;
  cpu_percent: number;
  memory_used_bytes: number;
  last_seen_unix_ms: number;
}

export interface HealthResponse {
  status: string;
}
