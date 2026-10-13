import React, { useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { fetchJson } from '../api';

type OpenApiDoc = unknown;

export function DocsPage() {
  const spec = useQuery<OpenApiDoc>({
    queryKey: ['openapi'],
    queryFn: () => fetchJson<OpenApiDoc>('/api/api-doc/openapi.json'),
    retry: false,
  });

  const pretty = useMemo(() => {
    if (!spec.data) return '';
    try {
      return JSON.stringify(spec.data, null, 2);
    } catch {
      return String(spec.data);
    }
  }, [spec.data]);

  const download = () => {
    if (!pretty) return;
    const blob = new Blob([pretty], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'openapi.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="page-container">
      <header className="page-header">
        <div>
          <h1>API Docs</h1>
          <p>Authenticated OpenAPI schema (requires <code>docs:read</code>)</p>
        </div>
        <div style={{ display: 'flex', gap: '0.75rem' }}>
          <button className="button-secondary" onClick={() => spec.refetch()} disabled={spec.isFetching}>
            {spec.isFetching ? 'Refreshing…' : 'Refresh'}
          </button>
          <button className="button-primary" onClick={download} disabled={!pretty}>
            Download JSON
          </button>
        </div>
      </header>

      {spec.isLoading && (
        <div className="card">
          <p className="muted">Loading OpenAPI…</p>
        </div>
      )}

      {spec.isError && (
        <div className="card" style={{ background: 'rgba(239, 68, 68, 0.08)', borderColor: 'rgba(239, 68, 68, 0.4)' }}>
          <strong>Unable to load API docs</strong>
          <p className="muted" style={{ marginTop: '0.5rem' }}>
            {spec.error instanceof Error ? spec.error.message : 'Unknown error'}
          </p>
          <p className="muted" style={{ marginTop: '0.5rem' }}>
            If you believe you should have access, ask an administrator to grant <code>docs:read</code>.
          </p>
        </div>
      )}

      {spec.data != null && (
        <div className="card" style={{ padding: '1.25rem' }}>
          <div className="card-header" style={{ padding: 0, marginBottom: '1rem' }}>
            <h2 style={{ margin: 0 }}>OpenAPI JSON</h2>
            <span className="muted">/api/api-doc/openapi.json</span>
          </div>
          <pre style={{ margin: 0, overflowX: 'auto', fontSize: '0.85rem', lineHeight: 1.45 }}>
{pretty}
          </pre>
        </div>
      )}
    </div>
  );
}
