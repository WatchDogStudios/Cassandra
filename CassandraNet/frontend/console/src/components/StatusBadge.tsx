import React from 'react';

interface StatusBadgeProps {
  status: 'up' | 'down' | 'unknown';
  label?: string;
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({ status, label }) => {
  let statusLabel: string;
  let className: string;
  let dotColor: string;

  switch (status) {
    case 'up':
      statusLabel = label ?? 'Operational';
      className = 'badge status-up';
      dotColor = '#10b981';
      break;
    case 'down':
      statusLabel = label ?? 'Degraded';
      className = 'badge status-down';
      dotColor = '#ef4444';
      break;
    default:
      statusLabel = label ?? 'Unknown';
      className = 'badge muted';
      dotColor = '#94a3b8';
      break;
  }

  return (
    <span className={className}>
      <span className="badge-dot" style={{ backgroundColor: dotColor }} />
      {statusLabel}
    </span>
  );
};
