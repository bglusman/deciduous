/**
 * Chains Hook
 *
 * Computes chains and sessions from graph data.
 * Memoizes the computation to avoid recalculating on every render.
 */

import { useMemo } from 'react';
import type { GraphData, Chain, Session } from '../types/graph';
import { buildChains, buildSessions, calculateStats, type GraphStats } from '../utils/graphProcessing';

interface UseChainsResult {
  chains: Chain[];
  sessions: Session[];
  stats: GraphStats;
}

/**
 * Hook for computing chains and sessions from graph data
 *
 * Usage:
 * ```tsx
 * const { chains, sessions, stats } = useChains(graphData);
 * ```
 */
export function useChains(graphData: GraphData | null): UseChainsResult {
  return useMemo(() => {
    if (!graphData) {
      return {
        chains: [],
        sessions: [],
        stats: {
          nodeCount: 0,
          edgeCount: 0,
          chainCount: 0,
          sessionCount: 0,
          linkedCommitCount: 0,
          nodesByType: {},
        },
      };
    }

    const chains = buildChains(graphData);
    const sessions = buildSessions(graphData.nodes, chains);
    const stats = calculateStats(graphData, chains, sessions);

    return { chains, sessions, stats };
  }, [graphData]);
}
