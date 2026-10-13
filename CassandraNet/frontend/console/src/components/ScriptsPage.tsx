import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { fetchJson, postJson, putJson, deleteJson } from '../api';
import { StatusBadge } from './StatusBadge';

interface Script {
  id: string;
  name: string;
  description?: string | null;
  language: 'lua' | 'python' | 'bash' | 'sh' | 'powershell' | 'javascript';
  content: string;
  version: number;
  tags: string[];
  created_at: string;
  updated_at: string;
  created_by?: string | null;
}

interface ScriptExecution {
  id: string;
  script_id: string;
  agent_id: string;
  status: 'pending' | 'dispatched' | 'completed' | 'failed';
  exit_code: number | null;
  stdout: string | null;
  stderr: string | null;
  error: string | null;
  started_at: string;
  completed_at: string | null;
}

const LANGUAGES = [
  { value: 'lua', label: 'Lua', icon: '🌙' },
  { value: 'python', label: 'Python', icon: '🐍' },
  { value: 'bash', label: 'Bash', icon: '🐚' },
  { value: 'powershell', label: 'PowerShell', icon: '💠' },
  { value: 'javascript', label: 'JavaScript', icon: '📜' },
];

export function ScriptsPage() {
  const queryClient = useQueryClient();
  const [selectedScriptId, setSelectedScriptId] = useState<string | null>(null);
  const [editMode, setEditMode] = useState(false);
  const [showNewModal, setShowNewModal] = useState(false);
  const [newScript, setNewScript] = useState<Partial<Script>>({ language: 'lua' });
  const [scriptContent, setScriptContent] = useState('');
  const [showExecuteModal, setShowExecuteModal] = useState(false);
  const [executeAgentId, setExecuteAgentId] = useState('');

  const scripts = useQuery<Script[]>({
    queryKey: ['scripts'],
    queryFn: () => fetchJson<Script[]>('/api/scripts'),
  });

  const selectedScript = scripts.data?.find(s => s.id === selectedScriptId) ?? null;

  const executions = useQuery<ScriptExecution[]>({
    queryKey: ['scriptExecutions', selectedScriptId],
    queryFn: () => fetchJson<ScriptExecution[]>(`/api/scripts/${selectedScriptId}/executions`),
    enabled: !!selectedScriptId,
  });

  const createScript = useMutation({
    mutationFn: (script: Partial<Script>) => postJson('/api/scripts', script),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scripts'] });
      setShowNewModal(false);
      setNewScript({ language: 'lua' });
    },
  });

  const updateScript = useMutation({
    mutationFn: (script: Partial<Script>) => putJson(`/api/scripts/${selectedScriptId}`, script),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scripts'] });
      setEditMode(false);
    },
  });

  const deleteScript = useMutation({
    mutationFn: (id: string) => deleteJson(`/api/scripts/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scripts'] });
      setSelectedScriptId(null);
    },
  });

  const executeScript = useMutation({
    mutationFn: ({ scriptId, agentId }: { scriptId: string; agentId: string }) =>
      postJson(`/api/scripts/${scriptId}/execute`, { agent_ids: [agentId] }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scriptExecutions', selectedScriptId] });
      setShowExecuteModal(false);
      setExecuteAgentId('');
    },
  });

  const handleCreateScript = () => {
    if (newScript.name && newScript.content && newScript.language) {
      createScript.mutate(newScript);
    }
  };

  const handleSaveScript = () => {
    updateScript.mutate({ content: scriptContent });
  };

  const getLanguageIcon = (lang: string) => {
    return LANGUAGES.find(l => l.value === lang)?.icon ?? '📄';
  };

  return (
    <div className="page-container">
      <header className="page-header">
        <div>
          <h1>Script Library</h1>
          <p>Manage, edit, and execute scripts on remote agents</p>
        </div>
        <button className="button-primary" onClick={() => setShowNewModal(true)}>
          + New Script
        </button>
      </header>

      <div className="scripts-layout">
        {/* Script List */}
        <section className="scripts-list card">
          <div className="card-header">
            <h2>Scripts</h2>
            <span className="muted">{scripts.data?.length ?? 0} scripts</span>
          </div>
          
          <div className="script-list-scroll">
            {scripts.isLoading && <div className="loading-state">Loading scripts...</div>}
            {scripts.data?.length === 0 && (
              <div className="empty-state">
                <strong>No scripts yet</strong>
                <p>Create your first script</p>
              </div>
            )}
            {scripts.data?.map(script => (
              <div
                key={script.id}
                className={`script-list-item ${selectedScriptId === script.id ? 'script-list-item-selected' : ''}`}
                onClick={() => {
                  setSelectedScriptId(script.id);
                  setScriptContent(script.content);
                  setEditMode(false);
                }}
              >
                <div className="script-list-info">
                  <span className="script-icon">{getLanguageIcon(script.language)}</span>
                  <div>
                    <strong>{script.name}</strong>
                    <span className="script-meta">v{script.version} • {script.language}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* Script Editor */}
        <section className="script-editor">
          {!selectedScript ? (
            <div className="card empty-state">
              <strong>Select a script</strong>
              <p>Choose a script from the list to view and edit</p>
            </div>
          ) : (
            <>
              <div className="card">
                <div className="card-header">
                  <div className="script-title-row">
                    <span className="script-icon-lg">{getLanguageIcon(selectedScript.language)}</span>
                    <div>
                      <h2>{selectedScript.name}</h2>
                      <span className="muted">
                        {selectedScript.language} • Version {selectedScript.version}
                      </span>
                    </div>
                  </div>
                  <div className="script-actions">
                    <button 
                      className="button-primary"
                      onClick={() => setShowExecuteModal(true)}
                    >
                      ▶ Execute
                    </button>
                    {!editMode ? (
                      <button 
                        className="button-secondary"
                        onClick={() => setEditMode(true)}
                      >
                        Edit
                      </button>
                    ) : (
                      <>
                        <button 
                          className="button-primary"
                          onClick={handleSaveScript}
                          disabled={updateScript.isPending}
                        >
                          Save
                        </button>
                        <button 
                          className="button-secondary"
                          onClick={() => {
                            setEditMode(false);
                            setScriptContent(selectedScript.content);
                          }}
                        >
                          Cancel
                        </button>
                      </>
                    )}
                  </div>
                </div>

                {selectedScript.description && (
                  <p className="script-description">{selectedScript.description}</p>
                )}

                {selectedScript.tags.length > 0 && (
                  <div className="tags-list">
                    {selectedScript.tags.map(tag => (
                      <span key={tag} className="tag">{tag}</span>
                    ))}
                  </div>
                )}

                <div className="code-editor-wrapper">
                  <textarea
                    className="code-editor"
                    value={scriptContent}
                    onChange={e => setScriptContent(e.target.value)}
                    readOnly={!editMode}
                    spellCheck={false}
                  />
                </div>

                <div className="script-meta-row">
                  <span className="muted">
                    Created {new Date(selectedScript.created_at).toLocaleDateString()}
                    {selectedScript.created_by ? ` by ${selectedScript.created_by}` : ''}
                  </span>
                  <span className="muted">
                    Last updated {new Date(selectedScript.updated_at).toLocaleString()}
                  </span>
                </div>
              </div>

              {/* Execution History */}
              <div className="card">
                <div className="card-header">
                  <h3>Execution History</h3>
                </div>
                
                <div className="execution-list">
                  {executions.isLoading && <p className="muted">Loading executions...</p>}
                  {executions.data?.length === 0 && (
                    <p className="muted">No executions yet</p>
                  )}
                  {executions.data?.slice(0, 10).map(exec => (
                    <div key={exec.id} className="execution-item">
                      <div className="execution-info">
                        <StatusBadge status={
                          exec.status === 'completed' ? 'up' : 
                          exec.status === 'failed' ? 'down' : 'unknown'
                        } />
                        <span>Agent: <code>{exec.agent_id.slice(0, 8)}...</code></span>
                        <span className="muted">{new Date(exec.started_at).toLocaleString()}</span>
                      </div>
                      {exec.exit_code !== null && (
                        <span className={`exit-code ${exec.exit_code === 0 ? 'exit-success' : 'exit-error'}`}>
                          Exit: {exec.exit_code}
                        </span>
                      )}
                      {(exec.error || exec.stdout || exec.stderr) && (
                        <details className="job-output">
                          <summary>Output</summary>
                          {exec.error && <pre className="exit-error">{exec.error}</pre>}
                          {exec.stdout && <pre>{exec.stdout}</pre>}
                          {exec.stderr && <pre className="muted">{exec.stderr}</pre>}
                        </details>
                      )}
                    </div>
                  ))}
                </div>
              </div>

              {/* Danger Zone */}
              <div className="card danger-zone">
                <h3>Danger Zone</h3>
                <button 
                  className="button-danger"
                  onClick={() => {
                    if (confirm('Delete this script?')) {
                      deleteScript.mutate(selectedScript.id);
                    }
                  }}
                >
                  Delete Script
                </button>
              </div>
            </>
          )}
        </section>
      </div>

      {/* New Script Modal */}
      {showNewModal && (
        <div className="modal-overlay" onClick={() => setShowNewModal(false)}>
          <div className="modal modal-lg" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>New Script</h2>
              <button className="modal-close" onClick={() => setShowNewModal(false)}>×</button>
            </div>
            
            <div className="modal-body">
              <div className="form-group">
                <label>Script Name *</label>
                <input
                  type="text"
                  value={newScript.name ?? ''}
                  onChange={e => setNewScript(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="health-check"
                />
              </div>
              
              <div className="form-group">
                <label>Description</label>
                <input
                  type="text"
                  value={newScript.description ?? ''}
                  onChange={e => setNewScript(prev => ({ ...prev, description: e.target.value }))}
                  placeholder="Checks agent health and reports status"
                />
              </div>
              
              <div className="form-group">
                <label>Language *</label>
                <select
                  value={newScript.language ?? 'lua'}
                  onChange={e => setNewScript(prev => ({ ...prev, language: e.target.value as Script['language'] }))}
                >
                  {LANGUAGES.map(lang => (
                    <option key={lang.value} value={lang.value}>
                      {lang.icon} {lang.label}
                    </option>
                  ))}
                </select>
              </div>
              
              <div className="form-group">
                <label>Script Content *</label>
                <textarea
                  className="code-editor modal-code"
                  value={newScript.content ?? ''}
                  onChange={e => setNewScript(prev => ({ ...prev, content: e.target.value }))}
                  placeholder="-- Enter your script here"
                  spellCheck={false}
                />
              </div>

              <div className="form-group">
                <label>Tags (comma-separated)</label>
                <input
                  type="text"
                  value={newScript.tags?.join(', ') ?? ''}
                  onChange={e => setNewScript(prev => ({ 
                    ...prev, 
                    tags: e.target.value.split(',').map(t => t.trim()).filter(Boolean)
                  }))}
                  placeholder="monitoring, health, utility"
                />
              </div>
            </div>
            
            <div className="modal-footer">
              <button className="button-secondary" onClick={() => setShowNewModal(false)}>
                Cancel
              </button>
              <button 
                className="button-primary"
                onClick={handleCreateScript}
                disabled={createScript.isPending || !newScript.name || !newScript.content}
              >
                {createScript.isPending ? 'Creating...' : 'Create Script'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Execute Modal */}
      {showExecuteModal && (
        <div className="modal-overlay" onClick={() => setShowExecuteModal(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Execute Script</h2>
              <button className="modal-close" onClick={() => setShowExecuteModal(false)}>×</button>
            </div>
            
            <div className="modal-body">
              <p>Execute <strong>{selectedScript?.name}</strong> on an agent</p>
              
              <div className="form-group">
                <label>Agent ID *</label>
                <input
                  type="text"
                  value={executeAgentId}
                  onChange={e => setExecuteAgentId(e.target.value)}
                  placeholder="Enter agent ID or select from list"
                />
              </div>
            </div>
            
            <div className="modal-footer">
              <button className="button-secondary" onClick={() => setShowExecuteModal(false)}>
                Cancel
              </button>
              <button 
                className="button-primary"
                onClick={() => {
                  if (selectedScriptId && executeAgentId) {
                    executeScript.mutate({ scriptId: selectedScriptId, agentId: executeAgentId });
                  }
                }}
                disabled={executeScript.isPending || !executeAgentId}
              >
                {executeScript.isPending ? 'Executing...' : 'Execute'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
