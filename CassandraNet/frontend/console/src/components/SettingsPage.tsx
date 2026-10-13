import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { deleteJson, fetchJson, postJson, putJson } from '../api';
import { useAuth } from '../auth';

interface PlatformSettings {
  default_heartbeat_interval: number;
  agent_timeout_seconds: number;
  metrics_retention_days: number;
  max_builds_per_project: number;
  max_scripts_per_project: number;
  enable_auto_updates: boolean;
  webhook_url: string | null;
  notification_email: string | null;
}

interface ApiKey {
  id: string;
  tenant_id: string;
  label: string;
  prefix: string;
  created_at: string;
  last_used_at: string | null;
  revoked: boolean;
  scopes: string[];
}

interface CreateApiKeyResponse {
  id: string;
  api_key: string;
  tenant_id: string;
  prefix: string;
  label: string;
  scopes: string[];
  created_at: string;
}

interface UserInfo {
  id: string;
  email: string;
  display_name: string | null;
  tenant_id: string | null;
  scopes: string[];
  is_active: boolean;
  created_at: string;
  updated_at: string;
  last_login_at: string | null;
}

export function SettingsPage() {
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState<'general' | 'api-keys' | 'users' | 'notifications'>('general');
  const [editingSettings, setEditingSettings] = useState<Partial<PlatformSettings> | null>(null);
  const [showNewKeyModal, setShowNewKeyModal] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');
  const [newKeyScopes, setNewKeyScopes] = useState<string[]>(['agent:read']);
  const [createdKey, setCreatedKey] = useState<string | null>(null);
  const [editingApiKey, setEditingApiKey] = useState<ApiKey | null>(null);
  const [editingApiKeyScopes, setEditingApiKeyScopes] = useState<string[]>([]);
  const [editingUser, setEditingUser] = useState<UserInfo | null>(null);
  const [editingUserScopes, setEditingUserScopes] = useState<string[]>([]);

  const canAdminRead = !!user?.scopes?.some((s) => s === 'admin:read' || s === 'admin:write' || s === 'admin');
  const canAdminWrite = !!user?.scopes?.some((s) => s === 'admin:write' || s === 'admin');

  const settings = useQuery<PlatformSettings>({
    queryKey: ['settings'],
    queryFn: () => fetchJson<PlatformSettings>('/api/settings'),
  });

  const apiKeys = useQuery<ApiKey[]>({
    queryKey: ['apiKeys', user?.tenant_id ?? null],
    enabled: !!user?.tenant_id,
    queryFn: () => fetchJson<ApiKey[]>(`/api/api-keys?tenant_id=${user!.tenant_id}`),
  });

  const users = useQuery<UserInfo[]>({
    queryKey: ['users', user?.tenant_id ?? null],
    enabled: canAdminRead,
    queryFn: () => {
      const tenantParam = user?.tenant_id ? `?tenant_id=${user.tenant_id}` : '';
      return fetchJson<UserInfo[]>(`/api/users${tenantParam}`);
    },
  });

  const updateSettings = useMutation({
    mutationFn: (newSettings: Partial<PlatformSettings>) =>
      putJson('/api/settings', newSettings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] });
      setEditingSettings(null);
    },
  });

  const createApiKey = useMutation({
    mutationFn: (data: { label: string; scopes: string[] }) =>
      postJson<CreateApiKeyResponse>('/api/api-keys', {
        tenant_id: user?.tenant_id,
        label: data.label,
        scopes: data.scopes,
      }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['apiKeys'] });
      setCreatedKey(data.api_key);
      setNewKeyName('');
      setNewKeyScopes(['agent:read']);
    },
  });

  const revokeApiKey = useMutation({
    mutationFn: (id: string) => deleteJson(`/api/api-keys/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['apiKeys'] }),
  });

  const updateUserScopes = useMutation({
    mutationFn: (payload: { userId: string; scopes: string[] }) =>
      putJson<UserInfo>(`/api/users/${payload.userId}/scopes`, { scopes: payload.scopes }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['users'] }),
  });

  const updateApiKeyScopes = useMutation({
    mutationFn: (payload: { keyId: string; scopes: string[] }) =>
      putJson<ApiKey>(`/api/api-keys/${payload.keyId}/scopes`, { scopes: payload.scopes }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['apiKeys'] }),
  });

  const SCOPES = [
    { value: 'docs:read', label: 'Docs: Read (OpenAPI / API docs)' },
    { value: 'admin:read', label: 'Admin: Read' },
    { value: 'admin:write', label: 'Admin: Write' },
    { value: 'tenant:read', label: 'Tenant: Read' },
    { value: 'tenant:write', label: 'Tenant: Write' },
    { value: 'apikey:manage', label: 'API Keys: Manage' },
    { value: 'agent:read', label: 'Agents: Read' },
    { value: 'agent:execute', label: 'Agents: Execute' },
    { value: 'workflow:execute', label: 'Workflows: Execute' },
    { value: 'ugc:read', label: 'UGC: Read' },
    { value: 'ugc:write', label: 'UGC: Write' },
    { value: 'messaging:read', label: 'Messaging: Read' },
    { value: 'messaging:write', label: 'Messaging: Write' },
    { value: 'observability:read', label: 'Observability: Read' },
  ];

  return (
    <div className="page-container">
      <header className="page-header">
        <div>
          <h1>Settings</h1>
          <p>Platform configuration, API keys, and notifications</p>
        </div>
      </header>

      <div className="settings-layout">
        {/* Tabs */}
        <div className="settings-tabs">
          <button
            className={`tab ${activeTab === 'general' ? 'tab-active' : ''}`}
            onClick={() => setActiveTab('general')}
          >
            General
          </button>
          <button
            className={`tab ${activeTab === 'api-keys' ? 'tab-active' : ''}`}
            onClick={() => setActiveTab('api-keys')}
          >
            API Keys
          </button>
          {canAdminRead && (
            <button
              className={`tab ${activeTab === 'users' ? 'tab-active' : ''}`}
              onClick={() => setActiveTab('users')}
            >
              Users
            </button>
          )}
          <button
            className={`tab ${activeTab === 'notifications' ? 'tab-active' : ''}`}
            onClick={() => setActiveTab('notifications')}
          >
            Notifications
          </button>
        </div>

        {/* General Settings */}
        {activeTab === 'general' && (
          <div className="settings-content">
            <div className="card">
              <div className="card-header">
                <h2>Platform Settings</h2>
                {!editingSettings && (
                  <button
                    className="button-secondary"
                    onClick={() => setEditingSettings(settings.data ?? {})}
                  >
                    Edit
                  </button>
                )}
              </div>

              {settings.isLoading && <p className="muted">Loading settings...</p>}

              {settings.data && !editingSettings && (
                <div className="settings-display">
                  <div className="setting-row">
                    <div>
                      <strong>Agent Heartbeat Interval</strong>
                      <p className="muted">How often agents send heartbeats</p>
                    </div>
                    <span>{settings.data.default_heartbeat_interval}s</span>
                  </div>
                  <div className="setting-row">
                    <div>
                      <strong>Agent Timeout</strong>
                      <p className="muted">Mark agent offline after this duration</p>
                    </div>
                    <span>{settings.data.agent_timeout_seconds}s</span>
                  </div>
                  <div className="setting-row">
                    <div>
                      <strong>Max Builds Per Project</strong>
                      <p className="muted">Storage limit for builds</p>
                    </div>
                    <span>{settings.data.max_builds_per_project}</span>
                  </div>
                  <div className="setting-row">
                    <div>
                      <strong>Max Scripts Per Project</strong>
                      <p className="muted">Limit for uploaded scripts</p>
                    </div>
                    <span>{settings.data.max_scripts_per_project}</span>
                  </div>
                  <div className="setting-row">
                    <div>
                      <strong>Metrics Retention</strong>
                      <p className="muted">Auto-delete metrics after this period</p>
                    </div>
                    <span>{settings.data.metrics_retention_days} days</span>
                  </div>
                  <div className="setting-row">
                    <div>
                      <strong>Auto Updates</strong>
                      <p className="muted">Automatically update agents when new versions are available</p>
                    </div>
                    <span>{settings.data.enable_auto_updates ? 'Enabled' : 'Disabled'}</span>
                  </div>
                </div>
              )}

              {editingSettings && (
                <div className="settings-form">
                  <div className="form-group">
                    <label>Agent Heartbeat Interval (seconds)</label>
                    <input
                      type="number"
                      value={editingSettings.default_heartbeat_interval ?? 30}
                      onChange={e => setEditingSettings(prev => ({
                        ...prev,
                        default_heartbeat_interval: parseInt(e.target.value) || 30
                      }))}
                    />
                  </div>
                  <div className="form-group">
                    <label>Agent Timeout (seconds)</label>
                    <input
                      type="number"
                      value={editingSettings.agent_timeout_seconds ?? 120}
                      onChange={e => setEditingSettings(prev => ({
                        ...prev,
                        agent_timeout_seconds: parseInt(e.target.value) || 120
                      }))}
                    />
                  </div>
                  <div className="form-group">
                    <label>Max Builds Per Project</label>
                    <input
                      type="number"
                      value={editingSettings.max_builds_per_project ?? 50}
                      onChange={e => setEditingSettings(prev => ({
                        ...prev,
                        max_builds_per_project: parseInt(e.target.value) || 50
                      }))}
                    />
                  </div>
                  <div className="form-group">
                    <label>Max Scripts Per Project</label>
                    <input
                      type="number"
                      value={editingSettings.max_scripts_per_project ?? 100}
                      onChange={e => setEditingSettings(prev => ({
                        ...prev,
                        max_scripts_per_project: parseInt(e.target.value) || 100
                      }))}
                    />
                  </div>
                  <div className="form-group">
                    <label>
                      <input
                        type="checkbox"
                        checked={editingSettings.enable_auto_updates ?? true}
                        onChange={e => setEditingSettings(prev => ({
                          ...prev,
                          enable_auto_updates: e.target.checked
                        }))}
                      />
                      Enable Auto Updates
                    </label>
                  </div>
                  <div className="form-group">
                    <label>Metrics Retention (days)</label>
                    <input
                      type="number"
                      value={editingSettings.metrics_retention_days ?? 30}
                      onChange={e => setEditingSettings(prev => ({
                        ...prev,
                        metrics_retention_days: parseInt(e.target.value) || 30
                      }))}
                    />
                  </div>
                  <div className="form-actions">
                    <button
                      className="button-primary"
                      onClick={() => updateSettings.mutate(editingSettings)}
                      disabled={updateSettings.isPending}
                    >
                      {updateSettings.isPending ? 'Saving...' : 'Save Settings'}
                    </button>
                    <button
                      className="button-secondary"
                      onClick={() => setEditingSettings(null)}
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              )}
            </div>

            <div className="card">
              <div className="card-header">
                <h2>Account Information</h2>
              </div>
              <div className="settings-display">
                <div className="setting-row">
                  <div>
                    <strong>Email</strong>
                    <p className="muted">Your login email address</p>
                  </div>
                  <span>{user?.email ?? 'N/A'}</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* API Keys */}
        {activeTab === 'api-keys' && (
          <div className="settings-content">
            <div className="card">
              <div className="card-header">
                <h2>API Keys</h2>
                <button
                  className="button-primary"
                  onClick={() => setShowNewKeyModal(true)}
                >
                  + New API Key
                </button>
              </div>

              {apiKeys.isLoading && <p className="muted">Loading API keys...</p>}

              {apiKeys.data?.length === 0 && (
                <div className="empty-state">
                  <strong>No API keys</strong>
                  <p>Create an API key to access the CassandraNet API programmatically</p>
                </div>
              )}

              <div className="api-keys-list">
                {apiKeys.data?.map(key => (
                  <div key={key.id} className="api-key-item">
                    <div className="api-key-info">
                      <strong>{key.label}</strong>
                      <code className="api-key-prefix">{key.prefix}...</code>
                      <span className="muted">
                        Created {new Date(key.created_at).toLocaleDateString()}
                      </span>
                      {key.last_used_at && (
                        <span className="muted">
                          Last used {new Date(key.last_used_at).toLocaleDateString()}
                        </span>
                      )}
                    </div>
                    <div className="api-key-scopes">
                      {key.scopes.map(scope => (
                        <span key={scope} className="tag tag-sm">{scope}</span>
                      ))}
                    </div>
                    <button
                      className="button-secondary-sm"
                      onClick={() => {
                        setEditingApiKey(key);
                        setEditingApiKeyScopes(key.scopes);
                      }}
                    >
                      Edit scopes
                    </button>
                    <button
                      className="button-danger-sm"
                      onClick={() => {
                        if (confirm('Revoke this API key?')) {
                          revokeApiKey.mutate(key.id);
                        }
                      }}
                    >
                      Revoke
                    </button>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {/* Users */}
        {activeTab === 'users' && canAdminRead && (
          <div className="settings-content">
            <div className="card">
              <div className="card-header">
                <h2>Users</h2>
              </div>

              {users.isLoading && <p className="muted">Loading users...</p>}

              {users.data?.length === 0 && (
                <div className="empty-state">
                  <strong>No users found</strong>
                  <p>This tenant has no users (or you don't have access).</p>
                </div>
              )}

              <div className="users-list">
                {users.data?.map(u => {
                  const hasDocs = u.scopes.includes('docs:read');
                  return (
                    <div key={u.id} className="user-item">
                      <div className="user-info">
                        <strong>{u.email}</strong>
                        {u.display_name && <span className="muted">{u.display_name}</span>}
                        <span className="muted">User ID: {u.id}</span>
                        {u.last_login_at && (
                          <span className="muted">
                            Last login {new Date(u.last_login_at).toLocaleDateString()}
                          </span>
                        )}
                      </div>
                      <div className="user-actions">
                        <label className="scope-checkbox">
                          <input
                            type="checkbox"
                            checked={hasDocs}
                            disabled={!canAdminWrite || updateUserScopes.isPending}
                            onChange={e => {
                              const nextScopes = e.target.checked
                                ? Array.from(new Set([...u.scopes, 'docs:read']))
                                : u.scopes.filter(s => s !== 'docs:read');
                              updateUserScopes.mutate({ userId: u.id, scopes: nextScopes });
                            }}
                          />
                          Docs access
                        </label>

                        <button
                          className="button-secondary-sm"
                          disabled={!canAdminWrite}
                          onClick={() => {
                            setEditingUser(u);
                            setEditingUserScopes(u.scopes);
                          }}
                        >
                          Edit scopes
                        </button>
                      </div>
                      <div className="user-scopes">
                        {u.scopes.map(scope => (
                          <span key={scope} className="tag tag-sm">{scope}</span>
                        ))}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        )}

        {/* Notifications */}
        {activeTab === 'notifications' && (
          <div className="settings-content">
            <div className="card">
              <div className="card-header">
                <h2>Webhook Notifications</h2>
              </div>
              
              <div className="form-group">
                <label>Webhook URL</label>
                <input
                  type="url"
                  placeholder="https://your-server.com/webhook"
                  value={settings.data?.webhook_url ?? ''}
                  onChange={e => {
                    updateSettings.mutate({ webhook_url: e.target.value || null });
                  }}
                />
                <p className="muted">Receive POST notifications for events (agent status changes, build uploads, etc.)</p>
              </div>
            </div>

            <div className="card">
              <div className="card-header">
                <h2>Email Notifications</h2>
              </div>
              
              <div className="form-group">
                <label>Notification Email</label>
                <input
                  type="email"
                  placeholder="alerts@yourcompany.com"
                  value={settings.data?.notification_email ?? ''}
                  onChange={e => {
                    updateSettings.mutate({ notification_email: e.target.value || null });
                  }}
                />
                <p className="muted">Receive email alerts for critical events</p>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* New API Key Modal */}
      {showNewKeyModal && (
        <div className="modal-overlay" onClick={() => {
          setShowNewKeyModal(false);
          setCreatedKey(null);
        }}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>{createdKey ? 'API Key Created' : 'Create API Key'}</h2>
              <button className="modal-close" onClick={() => {
                setShowNewKeyModal(false);
                setCreatedKey(null);
              }}>×</button>
            </div>
            
            <div className="modal-body">
              {createdKey ? (
                <div className="created-key-display">
                  <p className="warning-text">
                    ⚠️ Copy this key now. You won't be able to see it again!
                  </p>
                  <div className="key-display">
                    <code>{createdKey}</code>
                    <button
                      className="button-secondary"
                      onClick={() => navigator.clipboard.writeText(createdKey)}
                    >
                      Copy
                    </button>
                  </div>
                </div>
              ) : (
                <>
                  <div className="form-group">
                    <label>Key Name *</label>
                    <input
                      type="text"
                      value={newKeyName}
                      onChange={e => setNewKeyName(e.target.value)}
                      placeholder="production-ci-key"
                    />
                  </div>
                  
                  <div className="form-group">
                    <label>Scopes</label>
                    <div className="scopes-list">
                      {SCOPES.map(scope => (
                        <label key={scope.value} className="scope-checkbox">
                          <input
                            type="checkbox"
                            checked={newKeyScopes.includes(scope.value)}
                            onChange={e => {
                              if (e.target.checked) {
                                setNewKeyScopes(prev => [...prev, scope.value]);
                              } else {
                                setNewKeyScopes(prev => prev.filter(s => s !== scope.value));
                              }
                            }}
                          />
                          {scope.label}
                        </label>
                      ))}
                    </div>
                  </div>
                </>
              )}
            </div>
            
            <div className="modal-footer">
              {createdKey ? (
                <button className="button-primary" onClick={() => {
                  setShowNewKeyModal(false);
                  setCreatedKey(null);
                }}>
                  Done
                </button>
              ) : (
                <>
                  <button className="button-secondary" onClick={() => setShowNewKeyModal(false)}>
                    Cancel
                  </button>
                  <button
                    className="button-primary"
                    onClick={() => createApiKey.mutate({ label: newKeyName, scopes: newKeyScopes })}
                    disabled={createApiKey.isPending || !newKeyName}
                  >
                    {createApiKey.isPending ? 'Creating...' : 'Create Key'}
                  </button>
                </>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Edit API Key Scopes Modal */}
      {editingApiKey && (
        <div className="modal-overlay" onClick={() => setEditingApiKey(null)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Edit API Key Scopes</h2>
              <button className="modal-close" onClick={() => setEditingApiKey(null)}>×</button>
            </div>
            <div className="modal-body">
              <p className="muted">Key: <strong>{editingApiKey.label}</strong> ({editingApiKey.prefix}…)</p>
              <div className="scopes-list">
                {SCOPES.map(scope => (
                  <label key={scope.value} className="scope-checkbox">
                    <input
                      type="checkbox"
                      checked={editingApiKeyScopes.includes(scope.value)}
                      onChange={e => {
                        if (e.target.checked) {
                          setEditingApiKeyScopes(prev => [...prev, scope.value]);
                        } else {
                          setEditingApiKeyScopes(prev => prev.filter(s => s !== scope.value));
                        }
                      }}
                    />
                    {scope.label}
                  </label>
                ))}
              </div>
            </div>
            <div className="modal-footer">
              <button className="button-secondary" onClick={() => setEditingApiKey(null)}>
                Cancel
              </button>
              <button
                className="button-primary"
                disabled={updateApiKeyScopes.isPending || editingApiKeyScopes.length === 0}
                onClick={() => {
                  updateApiKeyScopes.mutate(
                    { keyId: editingApiKey.id, scopes: editingApiKeyScopes },
                    {
                      onSuccess: () => {
                        setEditingApiKey(null);
                      },
                    }
                  );
                }}
              >
                {updateApiKeyScopes.isPending ? 'Saving...' : 'Save'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Edit User Scopes Modal */}
      {editingUser && (
        <div className="modal-overlay" onClick={() => setEditingUser(null)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Edit User Scopes</h2>
              <button className="modal-close" onClick={() => setEditingUser(null)}>×</button>
            </div>
            <div className="modal-body">
              <p className="muted">User: <strong>{editingUser.email}</strong></p>
              <div className="scopes-list">
                {SCOPES.map(scope => (
                  <label key={scope.value} className="scope-checkbox">
                    <input
                      type="checkbox"
                      checked={editingUserScopes.includes(scope.value)}
                      onChange={e => {
                        if (e.target.checked) {
                          setEditingUserScopes(prev => [...prev, scope.value]);
                        } else {
                          setEditingUserScopes(prev => prev.filter(s => s !== scope.value));
                        }
                      }}
                    />
                    {scope.label}
                  </label>
                ))}
              </div>
            </div>
            <div className="modal-footer">
              <button className="button-secondary" onClick={() => setEditingUser(null)}>
                Cancel
              </button>
              <button
                className="button-primary"
                disabled={!canAdminWrite || updateUserScopes.isPending || editingUserScopes.length === 0}
                onClick={() => {
                  updateUserScopes.mutate(
                    { userId: editingUser.id, scopes: editingUserScopes },
                    {
                      onSuccess: () => {
                        setEditingUser(null);
                      },
                    }
                  );
                }}
              >
                {updateUserScopes.isPending ? 'Saving...' : 'Save'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
