import React, { useState, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { fetchJson, postJson, putJson } from '../api';
import type { AgentSummary, AgentConfig, AgentCommand } from '../types';
import { StatusBadge } from './StatusBadge';

export function AgentsPage() {
  const queryClient = useQueryClient();
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [editingConfig, setEditingConfig] = useState<Partial<AgentConfig> | null>(null);
  const [commandType, setCommandType] = useState('restart');
  const [commandPayload, setCommandPayload] = useState('{}');

  const agents = useQuery<AgentSummary[]>({
    queryKey: ['agents'],
    queryFn: () => fetchJson<AgentSummary[]>('/api/agents'),
    refetchInterval: 5000,
  });

  const agentConfig = useQuery<AgentConfig>({
    queryKey: ['agentConfig', selectedAgentId],
    queryFn: () => fetchJson<AgentConfig>(`/api/agents/${selectedAgentId}/config`),
    enabled: !!selectedAgentId,
  });

  const agentCommands = useQuery<AgentCommand[]>({
    queryKey: ['agentCommands', selectedAgentId],
    queryFn: () => fetchJson<AgentCommand[]>(`/api/agents/${selectedAgentId}/commands`),
    enabled: !!selectedAgentId,
  });

  const updateConfig = useMutation({
    mutationFn: (config: Partial<AgentConfig>) =>
      putJson(`/api/agents/${selectedAgentId}/config`, config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agentConfig', selectedAgentId] });
      setEditingConfig(null);
    },
  });

  const sendCommand = useMutation({
    mutationFn: (command: { command: string; args: Record<string, unknown> }) =>
      postJson(`/api/agents/${selectedAgentId}/commands`, command),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agentCommands', selectedAgentId] });
      setCommandPayload('{}');
    },
  });

  const selectedAgent = useMemo(() => {
    return agents.data?.find(a => a.id === selectedAgentId) ?? null;
  }, [agents.data, selectedAgentId]);

  const handleSendCommand = () => {
    try {
      const args = JSON.parse(commandPayload);
      sendCommand.mutate({ command: commandType, args });
    } catch {
      alert('Invalid JSON payload');
    }
  };

  return (
    <div className="page-container">
      <header className="page-header">
        <div>
          <h1>Agent Management</h1>
          <p>Configure agents, send commands, and manage lifecycle</p>
        </div>
      </header>

      <div className="agents-layout">
        {/* Agent List */}
        <section className="agents-list card">
          <div className="card-header">
            <h2>Connected Agents</h2>
            <span className="muted">{agents.data?.length ?? 0} agents</span>
          </div>
          
          <div className="agent-list-scroll">
            {agents.isLoading && <div className="loading-state">Loading agents...</div>}
            {agents.data?.length === 0 && (
              <div className="empty-state">
                <strong>No agents connected</strong>
                <p>Deploy an agent to get started</p>
              </div>
            )}
            {agents.data?.map(agent => (
              <div
                key={agent.id}
                className={`agent-list-item ${selectedAgentId === agent.id ? 'agent-list-item-selected' : ''}`}
                onClick={() => setSelectedAgentId(agent.id)}
              >
                <div className="agent-list-info">
                  <strong>{agent.hostname}</strong>
                  <span className="agent-id-truncated">{agent.id.slice(0, 8)}...</span>
                </div>
                <StatusBadge 
                  status={Date.now() - agent.last_seen_unix_ms < 30000 ? 'up' : 'down'} 
                />
              </div>
            ))}
          </div>
        </section>

        {/* Agent Detail & Config */}
        <section className="agent-detail">
          {!selectedAgent ? (
            <div className="card empty-state">
              <strong>Select an agent</strong>
              <p>Choose an agent from the list to view and configure</p>
            </div>
          ) : (
            <>
              {/* Agent Info */}
              <div className="card">
                <div className="card-header">
                  <h2>{selectedAgent.hostname}</h2>
                  <StatusBadge 
                    status={Date.now() - selectedAgent.last_seen_unix_ms < 30000 ? 'up' : 'down'} 
                  />
                </div>
                <div className="detail-grid">
                  <div>
                    <span className="detail-label">Agent ID</span>
                    <code className="detail-value">{selectedAgent.id}</code>
                  </div>
                  <div>
                    <span className="detail-label">CPU</span>
                    <strong className="detail-value">{selectedAgent.cpu_percent.toFixed(1)}%</strong>
                  </div>
                  <div>
                    <span className="detail-label">Memory</span>
                    <strong className="detail-value">
                      {(selectedAgent.memory_used_bytes / 1024 / 1024).toFixed(0)} MB
                    </strong>
                  </div>
                  <div>
                    <span className="detail-label">Tenant</span>
                    <code className="detail-value">{selectedAgent.tenant_id || 'N/A'}</code>
                  </div>
                  <div>
                    <span className="detail-label">Project</span>
                    <code className="detail-value">{selectedAgent.project_id || 'N/A'}</code>
                  </div>
                  <div>
                    <span className="detail-label">Last Seen</span>
                    <span className="detail-value">
                      {new Date(selectedAgent.last_seen_unix_ms).toLocaleString()}
                    </span>
                  </div>
                </div>
              </div>

              {/* Configuration */}
              <div className="card">
                <div className="card-header">
                  <h3>Configuration</h3>
                  {!editingConfig && (
                    <button 
                      className="button-secondary"
                      onClick={() => setEditingConfig(agentConfig.data ?? {})}
                    >
                      Edit
                    </button>
                  )}
                </div>
                
                {agentConfig.isLoading && <p className="muted">Loading configuration...</p>}
                
                {agentConfig.data && !editingConfig && (
                  <div className="config-display">
                    <div className="config-row">
                      <span>Heartbeat Interval</span>
                      <strong>{agentConfig.data.heartbeat_interval_seconds}s</strong>
                    </div>
                    <div className="config-row">
                      <span>Log Level</span>
                      <strong>{agentConfig.data.log_level}</strong>
                    </div>
                    <div className="config-row">
                      <span>Auto Update</span>
                      <strong>{agentConfig.data.auto_update_enabled ? 'Enabled' : 'Disabled'}</strong>
                    </div>
                    <div className="config-row">
                      <span>Max Concurrent Tasks</span>
                      <strong>{agentConfig.data.max_concurrent_tasks}</strong>
                    </div>
                    <div className="config-row">
                      <span>Metrics</span>
                      <strong>{agentConfig.data.metrics_enabled ? 'Enabled' : 'Disabled'}</strong>
                    </div>
                    {Object.keys(agentConfig.data.custom_labels ?? {}).length > 0 && (
                      <div className="config-row">
                        <span>Custom Labels</span>
                        <div className="tags-list">
                          {Object.entries(agentConfig.data.custom_labels).map(([k, v]) => (
                            <span key={k} className="tag">{k}={v}</span>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                )}

                {editingConfig && (
                  <div className="config-form">
                    <div className="form-group">
                      <label>Heartbeat Interval (seconds)</label>
                      <input
                        type="number"
                        value={editingConfig.heartbeat_interval_seconds ?? 30}
                        onChange={e => setEditingConfig({
                          ...editingConfig,
                          heartbeat_interval_seconds: parseInt(e.target.value) || 30
                        })}
                      />
                    </div>
                    <div className="form-group">
                      <label>Log Level</label>
                      <select
                        value={editingConfig.log_level ?? 'info'}
                        onChange={e => setEditingConfig({
                          ...editingConfig,
                          log_level: e.target.value
                        })}
                      >
                        <option value="trace">Trace</option>
                        <option value="debug">Debug</option>
                        <option value="info">Info</option>
                        <option value="warn">Warn</option>
                        <option value="error">Error</option>
                      </select>
                    </div>
                    <div className="form-group">
                      <label>
                        <input
                          type="checkbox"
                          checked={editingConfig.auto_update_enabled ?? true}
                          onChange={e => setEditingConfig({
                            ...editingConfig,
                            auto_update_enabled: e.target.checked
                          })}
                        />
                        Auto Update
                      </label>
                    </div>
                    <div className="form-group">
                      <label>Max Concurrent Tasks</label>
                      <input
                        type="number"
                        value={editingConfig.max_concurrent_tasks ?? 4}
                        onChange={e => setEditingConfig({
                          ...editingConfig,
                          max_concurrent_tasks: parseInt(e.target.value) || 4
                        })}
                      />
                    </div>
                    <div className="form-group">
                      <label>
                        <input
                          type="checkbox"
                          checked={editingConfig.metrics_enabled ?? true}
                          onChange={e => setEditingConfig({
                            ...editingConfig,
                            metrics_enabled: e.target.checked
                          })}
                        />
                        Enable Metrics
                      </label>
                    </div>
                    <div className="form-actions">
                      <button 
                        className="button-primary"
                        onClick={() => updateConfig.mutate(editingConfig)}
                        disabled={updateConfig.isPending}
                      >
                        {updateConfig.isPending ? 'Saving...' : 'Save'}
                      </button>
                      <button 
                        className="button-secondary"
                        onClick={() => setEditingConfig(null)}
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                )}
              </div>

              {/* Remote Commands */}
              <div className="card">
                <div className="card-header">
                  <h3>Remote Commands</h3>
                </div>
                
                <div className="command-form">
                  <div className="form-row">
                    <div className="form-group">
                      <label>Command Type</label>
                      <select value={commandType} onChange={e => setCommandType(e.target.value)}>
                        <option value="restart">Restart Agent</option>
                        <option value="exec">Run Command</option>
                        <option value="shutdown">Shutdown</option>
                        <option value="collect_logs">Collect Logs</option>
                        <option value="deploy_build">Deploy Build</option>
                      </select>
                    </div>
                    <div className="form-group flex-grow">
                      <label>Args (JSON)</label>
                      <input
                        type="text"
                        value={commandPayload}
                        onChange={e => setCommandPayload(e.target.value)}
                        placeholder='{"key": "value"}'
                      />
                    </div>
                    <button 
                      className="button-primary"
                      onClick={handleSendCommand}
                      disabled={sendCommand.isPending}
                    >
                      Send
                    </button>
                  </div>
                </div>

                <div className="command-history">
                  <h4>Recent Commands</h4>
                  {agentCommands.data?.length === 0 && (
                    <p className="muted">No commands sent yet</p>
                  )}
                  {agentCommands.data?.slice(0, 10).map(cmd => (
                    <div key={cmd.id} className="command-item">
                      <div className="command-info">
                        <strong>{cmd.command}</strong>
                        <span className="muted">
                          {new Date(cmd.created_at).toLocaleString()}
                        </span>
                      </div>
                      <StatusBadge 
                        status={
                          cmd.status === 'completed' ? 'up' : 
                          cmd.status === 'failed' ? 'down' : 'unknown'
                        }
                      />
                      {(cmd.error || cmd.result?.stdout || cmd.result?.stderr) && (
                        <details className="job-output">
                          <summary>Output</summary>
                          {cmd.error && <pre className="exit-error">{cmd.error}</pre>}
                          {cmd.result?.stdout && <pre>{cmd.result.stdout}</pre>}
                          {cmd.result?.stderr && <pre className="muted">{cmd.result.stderr}</pre>}
                        </details>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            </>
          )}
        </section>
      </div>
    </div>
  );
}
