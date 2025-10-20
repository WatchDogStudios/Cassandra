import React from 'react';

interface StatCardProps {
  label: string;
  value: string;
  trend?: string;
  icon?: React.ReactNode;
}

export const StatCard: React.FC<StatCardProps> = ({ label, value, trend, icon }) => (
  <article className="card">
    <div className="card-subtitle">{label}</div>
    <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
      {icon && <span style={{ fontSize: '1.6rem' }}>{icon}</span>}
      <div>
        <div className="card-value">{value}</div>
        {trend && <div className="muted">{trend}</div>}
      </div>
    </div>
  </article>
);
