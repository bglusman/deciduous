/**
 * Main Application Component
 *
 * Sets up routing and data loading for the unified graph viewer.
 */

import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useGraphData } from './hooks/useGraphData';
import { useChains } from './hooks/useChains';
import { Layout } from './components/Layout';
import { ChainsView } from './views/ChainsView';
import { TimelineView } from './views/TimelineView';
import { GraphView } from './views/GraphView';
import { DagView } from './views/DagView';

export const App: React.FC = () => {
  // Load graph data with optional SSE for live updates
  // Use static file by default; only use /api/graph when deciduous serve is running
  const {
    graphData,
    gitHistory,
    loading,
    error,
    lastUpdated,
  } = useGraphData({
    graphUrl: './graph-data.json',
    gitHistoryUrl: './git-history.json',
    enableSSE: false, // Disable SSE until deciduous serve is implemented
  });

  // Compute chains and sessions
  const { chains, sessions, stats } = useChains(graphData);

  // Loading state
  if (loading) {
    return (
      <div style={styles.loading}>
        <div style={styles.spinner} />
        <p>Loading decision graph...</p>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div style={styles.error}>
        <h2>Error Loading Graph</h2>
        <p>{error}</p>
        <p style={styles.hint}>
          Make sure graph-data.json exists, or run <code>deciduous serve</code> for live data.
        </p>
      </div>
    );
  }

  // No data
  if (!graphData || graphData.nodes.length === 0) {
    return (
      <div style={styles.empty}>
        <h2>No Decision Data</h2>
        <p>The graph is empty. Start adding decisions!</p>
        <pre style={styles.code}>
          deciduous add goal "My first goal" -c 90
        </pre>
      </div>
    );
  }

  return (
    <BrowserRouter>
      <Layout stats={stats} lastUpdated={lastUpdated}>
        <Routes>
          <Route
            path="/"
            element={
              <ChainsView
                graphData={graphData}
                chains={chains}
                sessions={sessions}
              />
            }
          />
          <Route
            path="/timeline"
            element={
              <TimelineView
                graphData={graphData}
                gitHistory={gitHistory}
              />
            }
          />
          <Route
            path="/graph"
            element={
              <GraphView graphData={graphData} />
            }
          />
          <Route
            path="/dag"
            element={
              <DagView
                graphData={graphData}
                chains={chains}
              />
            }
          />
          {/* Fallback redirect */}
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
};

// =============================================================================
// Styles
// =============================================================================

const styles: Record<string, React.CSSProperties> = {
  loading: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100vh',
    backgroundColor: '#1a1a2e',
    color: '#888',
  },
  spinner: {
    width: '40px',
    height: '40px',
    border: '3px solid #333',
    borderTopColor: '#00d9ff',
    borderRadius: '50%',
    animation: 'spin 1s linear infinite',
    marginBottom: '20px',
  },
  error: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100vh',
    backgroundColor: '#1a1a2e',
    color: '#eee',
    textAlign: 'center',
    padding: '20px',
  },
  hint: {
    color: '#888',
    fontSize: '14px',
  },
  empty: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100vh',
    backgroundColor: '#1a1a2e',
    color: '#eee',
    textAlign: 'center',
    padding: '20px',
  },
  code: {
    backgroundColor: '#16213e',
    padding: '15px 20px',
    borderRadius: '8px',
    fontFamily: 'monospace',
    fontSize: '14px',
    color: '#00d9ff',
    marginTop: '10px',
  },
};

export default App;
