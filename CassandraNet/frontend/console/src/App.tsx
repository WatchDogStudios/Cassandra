import React from 'react';
import { useQuery } from '@tanstack/react-query';

interface AgentSummary { id: string; hostname: string; cpu_percent: number; memory_used_bytes: number; last_seen_unix_ms: number; }

function Agents() {
  const { data, isLoading, error } = useQuery({
    queryKey: ['agents'],
    queryFn: async () => {
      const res = await fetch('/api/agents');
      if (!res.ok) throw new Error('failed');
      return res.json() as Promise<AgentSummary[]>;
    },
    refetchInterval: 5000
  });
  if (isLoading) return <p>Loading agents...</p>;
  if (error) return <p>Error loading agents</p>;
  return <table><thead><tr><th>ID</th><th>Host</th><th>CPU%</th><th>Mem(MB)</th></tr></thead><tbody>{data!.map(a => <tr key={a.id}><td>{a.id}</td><td>{a.hostname}</td><td>{a.cpu_percent.toFixed(1)}</td><td>{(a.memory_used_bytes/1024/1024).toFixed(1)}</td></tr>)}</tbody></table>;
}

export default function App() {
  const health = useQuery({
    queryKey: ['health'],
    queryFn: async () => {
      const res = await fetch('/api/health');
      if (!res.ok) throw new Error('unhealthy');
      return res.json() as Promise<{status:string}>;
    },
    refetchInterval: 10000
  });
  return (
    <div style={{fontFamily:'sans-serif', padding:'1rem'}}>
      <h1>CassandraNet Console</h1>
      <p>Welcome. Build status: <code>alpha</code> Backend: {health.isLoading ? '...' : health.isError ? 'DOWN' : 'UP'}</p>
  <h2>Agents</h2>
  <Agents />
      {health.isError && <div style={{color:'red', marginTop:'1rem'}}>Cannot reach backend. Start gateway: <code>cargo run -p cngateway</code></div>}
    </div>
  );
}
