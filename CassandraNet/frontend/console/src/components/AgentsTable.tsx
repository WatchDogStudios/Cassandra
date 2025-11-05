import React from 'react';
import type { AgentSummary } from '../types';
import { StatusBadge } from './StatusBadge';

interface AgentsTableProps {
  agents: AgentSummary[];
  isLoading: boolean;
  error?: Error | null;
  selectedAgentId?: string | null;
  onSelect?: (agentId: string) => void;
}

export const AgentsTable: React.FC<AgentsTableProps> = ({
  agents,
  isLoading,
  error,
  selectedAgentId,
  onSelect
}) => {
  if (isLoading) {
    return (
      <div className="empty-state">
        <strong>Checking in with the cluster…</strong>
        <span className="muted">Refreshing every 5 seconds.</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="empty-state">
        <strong>Unable to load agents.</strong>
        <span className="muted">{error.message}</span>
      </div>
    );
  }

  if (agents.length === 0) {
    return (
      <div className="empty-state">
        <strong>No agents registered yet</strong>
        <span className="muted">Once an agent connects it will show up here with live telemetry.</span>
      </div>
    );
  }

  return (
    <div className="table-scroll">
      <table>
        <thead>
          <tr>
            <th>Agent</th>
            <th>Host</th>
            <th>Status</th>
            <th>CPU</th>
            <th>Memory</th>
            <th>Last seen</th>
          </tr>
        </thead>
        <tbody>
          {agents.map(agent => {
            const lastSeen = new Date(agent.last_seen_unix_ms);
            const secondsAgo = Math.max(0, (Date.now() - agent.last_seen_unix_ms) / 1000);
            let state: 'up' | 'down' | 'unknown';

            if (secondsAgo < 30) {
              state = 'up';
            } else if (secondsAgo < 120) {
              state = 'unknown';
            } else {
              state = 'down';
            }

            const isSelected = agent.id === selectedAgentId;

            return (
              <tr
                key={agent.id}
                className={isSelected ? 'table-row table-row-selected' : 'table-row'}
                onClick={() => onSelect?.(agent.id)}
                onKeyDown={event => {
                  if (event.key === 'Enter' || event.key === ' ') {
                    event.preventDefault();
                    onSelect?.(agent.id);
                  }
                }}
                role="button"
                tabIndex={0}
                aria-pressed={isSelected}
              >
                <td style={{ fontWeight: 600 }}>{agent.id}</td>
                <td>{agent.hostname}</td>
                <td>
                  <StatusBadge status={state} />
                </td>
                <td>{agent.cpu_percent.toFixed(1)}%</td>
                <td>{(agent.memory_used_bytes / 1024 / 1024).toFixed(1)} MB</td>
                <td title={lastSeen.toLocaleString()}>{formatDuration(secondsAgo)}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};

function formatDuration(secondsAgo: number): string {
  if (Number.isNaN(secondsAgo)) {
    return 'n/a';
  }
  if (secondsAgo < 5) {
    return 'just now';
  }
  if (secondsAgo < 60) {
    return `${Math.round(secondsAgo)}s ago`;
  }
  const minutes = secondsAgo / 60;
  if (minutes < 60) {
    return `${Math.round(minutes)}m ago`;
  }
  const hours = minutes / 60;
  if (hours < 24) {
    return `${Math.round(hours)}h ago`;
  }
  const days = hours / 24;
  return `${Math.round(days)}d ago`;
}
