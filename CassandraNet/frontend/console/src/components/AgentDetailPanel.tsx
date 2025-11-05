import React, { useMemo } from 'react';
import type { AgentMetricHistory, AgentSummary } from '../types';
import { StatusBadge } from './StatusBadge';
import { Sparkline } from './Sparkline';

interface AgentDetailPanelProps {
  agent: AgentSummary | null;
  metrics?: AgentMetricHistory;
  pollIntervalMs: number;
}

export const AgentDetailPanel: React.FC<AgentDetailPanelProps> = ({ agent, metrics, pollIntervalMs }) => {
  const status = useMemo(() => {
    if (!agent) {
      return 'unknown' as const;
    }
    const secondsAgo = Math.max(0, (Date.now() - agent.last_seen_unix_ms) / 1000);
    if (secondsAgo < 30) {
      return 'up' as const;
    }
    if (secondsAgo < 120) {
      return 'unknown' as const;
    }
    return 'down' as const;
  }, [agent]);

  if (!agent) {
    return (
      <aside className="detail-panel card" aria-live="polite">
        <div className="detail-header">
          <h2>Agent detail</h2>
          <p className="muted">Select an agent to inspect recent metrics.</p>
        </div>
        <div className="detail-empty">
          <p>Choose a row from the table to drill into runtime signals, tenant metadata, and health history.</p>
        </div>
      </aside>
    );
  }

  const lastSeen = new Date(agent.last_seen_unix_ms);
  const cpuSamples = metrics?.cpu ?? [];
  const memorySamples = metrics?.memory ?? [];
  const sampleCount = metrics?.timestamps.length ?? 0;
  const firstTimestamp = metrics?.timestamps[0] ?? 0;
  const lastTimestamp = sampleCount ? metrics?.timestamps[sampleCount - 1] ?? 0 : 0;
  const sampleWindowSeconds = sampleCount ? Math.round((lastTimestamp - firstTimestamp) / 1000) : 0;

  return (
    <aside className="detail-panel card" aria-live="polite">
      <div className="detail-header">
        <h2>Agent detail</h2>
        <StatusBadge status={status} />
      </div>
      <div className="detail-meta">
        <div>
          <span className="detail-label">Agent ID</span>
          <strong className="detail-value detail-value-mono">{agent.id}</strong>
        </div>
        <div>
          <span className="detail-label">Hostname</span>
          <strong className="detail-value">{agent.hostname}</strong>
        </div>
        <div>
          <span className="detail-label">Last seen</span>
          <strong className="detail-value">{lastSeen.toLocaleString()}</strong>
        </div>
        {agent.tenant_id && (
          <div>
            <span className="detail-label">Tenant</span>
            <strong className="detail-value detail-value-mono">{agent.tenant_id}</strong>
          </div>
        )}
        {agent.project_id && (
          <div>
            <span className="detail-label">Project</span>
            <strong className="detail-value detail-value-mono">{agent.project_id}</strong>
          </div>
        )}
        {agent.lifecycle_status && (
          <div>
            <span className="detail-label">Lifecycle</span>
            <strong className="detail-value">{titleCase(agent.lifecycle_status)}</strong>
          </div>
        )}
      </div>
      <div className="detail-metrics">
        <div className="metric-trend">
          <div className="metric-header">
            <div>
              <span className="detail-label">CPU usage</span>
              <strong className="detail-value">{agent.cpu_percent.toFixed(1)}%</strong>
            </div>
            <span className="metric-window muted">
              {sampleCount ? `Last ${Math.max(sampleWindowSeconds, pollIntervalMs / 1000)}s` : `Polling every ${pollIntervalMs / 1000}s`}
            </span>
          </div>
          <Sparkline
            data={cpuSamples}
            stroke="#6366f1"
            fill="rgba(99, 102, 241, 0.12)"
            ariaLabel="CPU usage history"
          />
        </div>
        <div className="metric-trend">
          <div className="metric-header">
            <div>
              <span className="detail-label">Memory footprint</span>
              <strong className="detail-value">{formatMegabytes(agent.memory_used_bytes)} MB</strong>
            </div>
            <span className="metric-window muted">
              {sampleCount ? `Last ${Math.max(sampleWindowSeconds, pollIntervalMs / 1000)}s` : `Polling every ${pollIntervalMs / 1000}s`}
            </span>
          </div>
          <Sparkline
            data={memorySamples}
            stroke="#14b8a6"
            fill="rgba(20, 184, 166, 0.12)"
            ariaLabel="Memory usage history"
          />
        </div>
      </div>
    </aside>
  );
};

function titleCase(value: string): string {
  return value
    .split(/[_\s]+/)
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(' ');
}

function formatMegabytes(bytes: number): string {
  if (!Number.isFinite(bytes)) {
    return '0.0';
  }
  return (bytes / 1024 / 1024).toFixed(1);
}
