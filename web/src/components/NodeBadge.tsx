/**
 * Badge Components
 *
 * Reusable badge components for node types, confidence, and commits.
 * Matches existing CSS styling exactly.
 */

import React from 'react';
import type { NodeType, EdgeType, DecisionNode } from '../types/graph';
import { getConfidence, getCommit, getConfidenceLevel, shortCommit, githubCommitUrl } from '../types/graph';
import { getNodeColor, getConfidenceColors } from '../utils/colors';

// =============================================================================
// Node Type Badge
// =============================================================================

interface TypeBadgeProps {
  type: NodeType;
  size?: 'sm' | 'md';
}

export const TypeBadge: React.FC<TypeBadgeProps> = ({ type, size = 'md' }) => {
  const color = getNodeColor(type);
  const isLight = ['goal', 'decision', 'option'].includes(type);

  const styles: React.CSSProperties = {
    fontSize: size === 'sm' ? '9px' : '10px',
    textTransform: 'uppercase',
    padding: size === 'sm' ? '2px 5px' : '2px 6px',
    borderRadius: '3px',
    display: 'inline-block',
    backgroundColor: color,
    color: isLight ? '#000' : '#fff',
    fontWeight: 500,
  };

  return <span style={styles}>{type}</span>;
};

// =============================================================================
// Confidence Badge
// =============================================================================

interface ConfidenceBadgeProps {
  confidence: number | null;
}

export const ConfidenceBadge: React.FC<ConfidenceBadgeProps> = ({ confidence }) => {
  if (confidence === null || confidence === undefined) return null;

  const level = getConfidenceLevel(confidence);
  const colors = getConfidenceColors(level);

  if (!colors) return null;

  const styles: React.CSSProperties = {
    fontSize: '10px',
    padding: '2px 6px',
    borderRadius: '10px',
    fontWeight: 600,
    backgroundColor: colors.bg,
    color: colors.text,
  };

  return <span style={styles}>{confidence}%</span>;
};

// =============================================================================
// Commit Badge
// =============================================================================

interface CommitBadgeProps {
  commit: string | null;
  repo?: string;
}

export const CommitBadge: React.FC<CommitBadgeProps> = ({
  commit,
  repo = 'notactuallytreyanastasio/losselot',
}) => {
  if (!commit) return null;

  const short = shortCommit(commit);
  const url = githubCommitUrl(commit, repo);

  const styles: React.CSSProperties = {
    fontSize: '10px',
    padding: '2px 6px',
    borderRadius: '4px',
    fontWeight: 500,
    fontFamily: 'monospace',
    backgroundColor: '#3b82f633',
    color: '#60a5fa',
    textDecoration: 'none',
    transition: 'all 0.2s',
  };

  return (
    <a
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      style={styles}
      title={`View commit ${short}`}
      onMouseOver={(e) => {
        e.currentTarget.style.backgroundColor = '#3b82f655';
        e.currentTarget.style.color = '#93c5fd';
      }}
      onMouseOut={(e) => {
        e.currentTarget.style.backgroundColor = '#3b82f633';
        e.currentTarget.style.color = '#60a5fa';
      }}
    >
      {short}
    </a>
  );
};

// =============================================================================
// Edge Type Badge
// =============================================================================

interface EdgeBadgeProps {
  type: EdgeType;
}

export const EdgeBadge: React.FC<EdgeBadgeProps> = ({ type }) => {
  const isChosen = type === 'chosen';
  const isRejected = type === 'rejected';

  const styles: React.CSSProperties = {
    fontSize: '10px',
    padding: '2px 6px',
    borderRadius: '3px',
    backgroundColor: isChosen ? '#22c55e' : isRejected ? '#ef4444' : '#333',
    color: isChosen ? '#000' : '#fff',
  };

  return <span style={styles}>{type}</span>;
};

// =============================================================================
// Combined Node Badges
// =============================================================================

interface NodeBadgesProps {
  node: DecisionNode;
  repo?: string;
}

export const NodeBadges: React.FC<NodeBadgesProps> = ({ node, repo }) => {
  const confidence = getConfidence(node);
  const commit = getCommit(node);

  return (
    <span style={{ display: 'inline-flex', gap: '6px', alignItems: 'center' }}>
      <TypeBadge type={node.node_type} />
      <ConfidenceBadge confidence={confidence} />
      <CommitBadge commit={commit} repo={repo} />
    </span>
  );
};

// =============================================================================
// Status Badge
// =============================================================================

interface StatusBadgeProps {
  status: DecisionNode['status'];
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({ status }) => {
  const colors: Record<string, { bg: string; text: string }> = {
    pending: { bg: '#6b728033', text: '#9ca3af' },
    active: { bg: '#3b82f633', text: '#60a5fa' },
    completed: { bg: '#22c55e33', text: '#4ade80' },
    rejected: { bg: '#ef444433', text: '#f87171' },
  };

  const c = colors[status] || colors.pending;

  const styles: React.CSSProperties = {
    fontSize: '10px',
    padding: '2px 6px',
    borderRadius: '10px',
    backgroundColor: c.bg,
    color: c.text,
  };

  return <span style={styles}>{status}</span>;
};
