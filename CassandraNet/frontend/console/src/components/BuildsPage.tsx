import React, { useState, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { fetchJson, postJson, deleteJson, uploadFile } from '../api';
import { StatusBadge } from './StatusBadge';

interface Build {
  id: string;
  name: string;
  version: string;
  platform: string;
  architecture: string;
  size_bytes: number;
  filename: string;
  checksum_sha256: string;
  status: 'ready';
  created_at: string;
  uploaded_by?: string | null;
  metadata: Record<string, string>;
}

interface BuildUpload {
  name: string;
  version: string;
  platform: string;
  architecture: string;
  file: File;
}

const PLATFORMS = ['windows', 'linux', 'macos', 'android', 'ios', 'xbox', 'playstation', 'switch'];
const ARCHITECTURES = ['x64', 'x86', 'arm64', 'arm32'];

export function BuildsPage() {
  const queryClient = useQueryClient();
  const [showUploadModal, setShowUploadModal] = useState(false);
  const [uploadForm, setUploadForm] = useState<Partial<BuildUpload>>({});
  const [uploadProgress, setUploadProgress] = useState(0);
  const [filterPlatform, setFilterPlatform] = useState<string>('');

  const builds = useQuery<Build[]>({
    queryKey: ['builds', filterPlatform],
    queryFn: () => {
      const url = filterPlatform 
        ? `/api/builds?platform=${filterPlatform}`
        : '/api/builds';
      return fetchJson<Build[]>(url);
    },
  });

  const uploadBuild = useMutation({
    mutationFn: async (upload: BuildUpload) => {
      const formData = new FormData();
      formData.append('name', upload.name);
      formData.append('version', upload.version);
      formData.append('platform', upload.platform);
      formData.append('architecture', upload.architecture);
      formData.append('file', upload.file);
      
      return uploadFile<Build>('/api/builds', formData, setUploadProgress);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['builds'] });
      setShowUploadModal(false);
      setUploadForm({});
      setUploadProgress(0);
    },
  });

  const deleteBuild = useMutation({
    mutationFn: (id: string) => deleteJson(`/api/builds/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['builds'] }),
  });

  const deployBuild = useMutation({
    mutationFn: ({ buildId, agentIds }: { buildId: string; agentIds: string[] }) =>
      postJson('/api/deployments', { build_id: buildId, agent_ids: agentIds }),
    onSuccess: () => alert('Deployment queued: agents pick it up on their next heartbeat.'),
    onError: (err) => alert(`Deployment failed: ${err instanceof Error ? err.message : String(err)}`),
  });

  const handleFileSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setUploadForm(prev => ({
        ...prev,
        file,
        name: prev.name || file.name.replace(/\.[^.]+$/, ''),
      }));
    }
  }, []);

  const handleUpload = () => {
    if (uploadForm.name && uploadForm.version && uploadForm.platform && 
        uploadForm.architecture && uploadForm.file) {
      uploadBuild.mutate(uploadForm as BuildUpload);
    }
  };

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
  };

  return (
    <div className="page-container">
      <header className="page-header">
        <div>
          <h1>Build Management</h1>
          <p>Upload, version, and deploy game builds to agents</p>
        </div>
        <button className="button-primary" onClick={() => setShowUploadModal(true)}>
          + Upload Build
        </button>
      </header>

      {/* Filters */}
      <div className="filters-bar">
        <div className="form-group">
          <label>Filter by Platform</label>
          <select value={filterPlatform} onChange={e => setFilterPlatform(e.target.value)}>
            <option value="">All Platforms</option>
            {PLATFORMS.map(p => (
              <option key={p} value={p}>{p.charAt(0).toUpperCase() + p.slice(1)}</option>
            ))}
          </select>
        </div>
      </div>

      {/* Builds Grid */}
      <div className="builds-grid">
        {builds.isLoading && <div className="loading-state">Loading builds...</div>}
        {builds.data?.length === 0 && (
          <div className="empty-state card">
            <strong>No builds uploaded</strong>
            <p>Upload your first build to get started</p>
          </div>
        )}
        {builds.data?.map(build => (
          <div key={build.id} className="card build-card">
            <div className="build-header">
              <div>
                <h3 className="build-name">{build.name}</h3>
                <span className="build-version">v{build.version}</span>
              </div>
              <StatusBadge status={build.status === 'ready' ? 'up' : build.status === 'failed' ? 'down' : 'unknown'} />
            </div>
            
            <div className="build-meta">
              <div className="build-meta-row">
                <span>Platform</span>
                <strong>{build.platform}</strong>
              </div>
              <div className="build-meta-row">
                <span>Architecture</span>
                <strong>{build.architecture}</strong>
              </div>
              <div className="build-meta-row">
                <span>Size</span>
                <strong>{formatSize(build.size_bytes)}</strong>
              </div>
              <div className="build-meta-row">
                <span>Uploaded</span>
                <strong>{new Date(build.created_at).toLocaleDateString()}</strong>
              </div>
            </div>

            <div className="build-checksum">
              <span className="detail-label">SHA256</span>
              <code>{build.checksum_sha256.slice(0, 16)}...</code>
            </div>

            <div className="build-actions">
              <button 
                className="button-primary"
                onClick={() => {
                  const agentIds = prompt('Enter agent IDs (comma-separated):');
                  if (agentIds) {
                    deployBuild.mutate({
                      buildId: build.id,
                      agentIds: agentIds.split(',').map(id => id.trim())
                    });
                  }
                }}
              >
                Deploy
              </button>
              <button 
                className="button-danger-sm"
                onClick={() => {
                  if (confirm('Delete this build?')) {
                    deleteBuild.mutate(build.id);
                  }
                }}
              >
                🗑️
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Upload Modal */}
      {showUploadModal && (
        <div className="modal-overlay" onClick={() => setShowUploadModal(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Upload Build</h2>
              <button className="modal-close" onClick={() => setShowUploadModal(false)}>×</button>
            </div>
            
            <div className="modal-body">
              <div className="form-group">
                <label>Build Name *</label>
                <input
                  type="text"
                  value={uploadForm.name ?? ''}
                  onChange={e => setUploadForm(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="MyGame-Client"
                />
              </div>
              
              <div className="form-group">
                <label>Version *</label>
                <input
                  type="text"
                  value={uploadForm.version ?? ''}
                  onChange={e => setUploadForm(prev => ({ ...prev, version: e.target.value }))}
                  placeholder="1.0.0"
                />
              </div>
              
              <div className="form-row">
                <div className="form-group">
                  <label>Platform *</label>
                  <select
                    value={uploadForm.platform ?? ''}
                    onChange={e => setUploadForm(prev => ({ ...prev, platform: e.target.value }))}
                  >
                    <option value="">Select...</option>
                    {PLATFORMS.map(p => (
                      <option key={p} value={p}>{p.charAt(0).toUpperCase() + p.slice(1)}</option>
                    ))}
                  </select>
                </div>
                
                <div className="form-group">
                  <label>Architecture *</label>
                  <select
                    value={uploadForm.architecture ?? ''}
                    onChange={e => setUploadForm(prev => ({ ...prev, architecture: e.target.value }))}
                  >
                    <option value="">Select...</option>
                    {ARCHITECTURES.map(a => (
                      <option key={a} value={a}>{a}</option>
                    ))}
                  </select>
                </div>
              </div>
              
              <div className="form-group">
                <label>Build File *</label>
                <div className="file-upload">
                  <input
                    type="file"
                    accept=".zip,.tar.gz,.tgz,.exe,.app"
                    onChange={handleFileSelect}
                  />
                  {uploadForm.file && (
                    <span className="file-name">{uploadForm.file.name} ({formatSize(uploadForm.file.size)})</span>
                  )}
                </div>
              </div>

              {uploadProgress > 0 && uploadProgress < 100 && (
                <div className="progress-bar">
                  <div className="progress-fill" style={{ width: `${uploadProgress}%` }} />
                  <span className="progress-text">{uploadProgress}%</span>
                </div>
              )}
            </div>
            
            <div className="modal-footer">
              <button className="button-secondary" onClick={() => setShowUploadModal(false)}>
                Cancel
              </button>
              <button 
                className="button-primary"
                onClick={handleUpload}
                disabled={uploadBuild.isPending || !uploadForm.name || !uploadForm.version || 
                          !uploadForm.platform || !uploadForm.architecture || !uploadForm.file}
              >
                {uploadBuild.isPending ? 'Uploading...' : 'Upload'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
