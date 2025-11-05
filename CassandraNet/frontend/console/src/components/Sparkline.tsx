import React from 'react';

interface SparklineProps {
  data: number[];
  width?: number;
  height?: number;
  stroke?: string;
  strokeWidth?: number;
  fill?: string;
  ariaLabel?: string;
}

export const Sparkline: React.FC<SparklineProps> = ({
  data,
  width = 220,
  height = 70,
  stroke = '#6366f1',
  strokeWidth = 2,
  fill = 'rgba(99, 102, 241, 0.12)',
  ariaLabel
}) => {
  if (!data.length) {
    return <div className="sparkline-empty">Not enough samples yet</div>;
  }

  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;

  const points = data.map((value, index) => {
    const x = data.length === 1 ? width : (index / (data.length - 1)) * width;
    const normalized = (value - min) / range;
    const y = height - normalized * height;
    return `${x},${y}`;
  });

  const areaSegments = [
    `M0,${height}`,
    ...points.map(point => `L${point}`),
    `L${width},${height}`,
    'Z'
  ];
  const areaPath = areaSegments.join(' ');

  return (
    <svg
      className="sparkline"
      width={width}
      height={height}
      role="img"
      aria-label={ariaLabel}
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
    >
      <path d={areaPath} fill={fill} stroke="none" />
      <polyline
        fill="none"
        stroke={stroke}
        strokeWidth={strokeWidth}
        strokeLinejoin="round"
        strokeLinecap="round"
        points={points.join(' ')}
      />
    </svg>
  );
};
