import React, { useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { AgentsTable } from './components/AgentsTable';
import { StatCard } from './components/StatCard';
import { StatusBadge } from './components/StatusBadge';
import type { AgentSummary, HealthResponse } from './types';

const AGENT_POLL_INTERVAL = 5_000;
const HEALTH_POLL_INTERVAL = 10_000;

async function fetchJson<T>(input: RequestInfo | URL): Promise<T> {
  const res = await fetch(input, {
    headers: {
      'accept': 'application/json'
    }
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || res.statusText);
  }
  return res.json() as Promise<T>;
}

export default function App() {
  const health = useQuery<HealthResponse>({
    queryKey: ['health'],
    queryFn: () => fetchJson<HealthResponse>('/api/health'),
    refetchInterval: HEALTH_POLL_INTERVAL
  });

  const agents = useQuery<AgentSummary[]>({
    queryKey: ['agents'],
    queryFn: () => fetchJson<AgentSummary[]>('/api/agents'),
    refetchInterval: AGENT_POLL_INTERVAL
  });

  const agentError = useMemo(() => {
    if (!agents.error) {
      return null;
    }
    return agents.error instanceof Error ? agents.error : new Error('Unexpected agent error');
  }, [agents.error]);

  const computed = useMemo(() => {
    const list = agents.data ?? [];
    const totalCpu = list.reduce((sum, agent) => sum + agent.cpu_percent, 0);
    const totalMemory = list.reduce((sum, agent) => sum + agent.memory_used_bytes, 0);
    const avgCpu = list.length ? (totalCpu / list.length).toFixed(1) : '0.0';
    const memoryGb = (totalMemory / list.length || 0) / 1024 / 1024 / 1024;
    const avgMemory = list.length ? `${memoryGb.toFixed(2)} GB` : '0.00 GB';
    const healthyAgents = list.filter(agent => Date.now() - agent.last_seen_unix_ms < 30_000).length;

    return {
      agentCount: list.length,
      healthyAgents,
      avgCpu,
      avgMemory
    };
  }, [agents.data]);

  let backendStatus: 'up' | 'down' | 'unknown' = 'unknown';
  if (health.isError) {
    backendStatus = 'down';
  } else if (!health.isFetching && health.data?.status === 'ok') {
    backendStatus = 'up';
  }

  return (
    <main>
      <header>
        <div>
          <h1>CassandraNet Control Surface</h1>
          <p>
            Monitor the edge mesh, agent health, and content pipeline throughput. Data refreshes automatically every few seconds.
          </p>
        </div>
        <nav>
          <button className="button-secondary" type="button" onClick={() => window.open('https://cassantranet.dev', '_blank')}>
            Marketing site
          </button>
          <button className="button-primary" type="button" onClick={() => window.open('mailto:support@watchdogstudios.io', '_blank')}>
            Contact support
          </button>
        </nav>
      </header>

      <section className="grid grid-3" style={{ marginBottom: '2rem' }}>
        <StatCard
          label="Registered agents"
          value={computed.agentCount.toString()}
          trend={`${computed.healthyAgents} reporting in the last 30s`}
        />
        <StatCard label="Average CPU" value={`${computed.avgCpu}%`} trend="Cluster wide" />
        <StatCard label="Avg memory footprint" value={computed.avgMemory} trend="Per active agent" />
      </section>

      <section className="card" style={{ padding: '1.75rem', marginBottom: '2rem' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <div className="card-subtitle">Backend status</div>
            <h2 className="section-title" style={{ marginBottom: '0.25rem' }}>
              CassandraNet Gateway
            </h2>
            <p className="section-subtitle" style={{ marginBottom: 0 }}>
              {health.isError
                ? 'The console cannot reach the gateway. Ensure the service is running.'
                : 'HTTP health probe checks the gateway every 10 seconds.'}
            </p>
          </div>
          <StatusBadge status={backendStatus} />
        </div>
        {health.isError && (
          <div className="card" style={{ marginTop: '1.5rem', background: 'rgba(239, 68, 68, 0.08)', borderColor: 'rgba(239, 68, 68, 0.4)' }}>
            <div className="card-subtitle">Troubleshooting</div>
            <p style={{ color: '#b91c1c', margin: 0 }}>
              Start the gateway locally with <code>cargo run -p cngateway</code> or check your reverse proxy configuration.
            </p>
          </div>
        )}
      </section>

      <section className="table-wrapper">
        <div className="table-header">
          <div>
            <h2>Connected agents</h2>
            <span>Live telemetry snapshot</span>
          </div>
          <span className="muted">Auto-refresh every {AGENT_POLL_INTERVAL / 1000}s</span>
        </div>
  <AgentsTable agents={agents.data ?? []} isLoading={agents.isLoading} error={agentError} />
      </section>

      <p className="footer-note">
        CassandraNet Alpha • Observability instrumentation powered by cncommon metrics.
      </p>
    </main>
  );
}
