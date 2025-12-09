//! HTTP server for decision graph viewer
//!
//! `deciduous serve` → starts server, opens browser, shows graph

use crate::db::{Database, DecisionGraph};
use serde::Serialize;
use tiny_http::{Header, Method, Request, Response, Server};

#[derive(Serialize)]
struct ApiResponse<T> {
    ok: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self { ok: true, data: Some(data), error: None }
    }
}

// Embedded graph viewer HTML
const GRAPH_VIEWER_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Deciduous - Decision Graph</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: system-ui, -apple-system, sans-serif; background: #0d1117; color: #c9d1d9; }
        .header { padding: 1rem 2rem; background: #161b22; border-bottom: 1px solid #30363d; }
        .header h1 { font-size: 1.25rem; color: #58a6ff; }
        .container { display: flex; height: calc(100vh - 60px); }
        .sidebar { width: 300px; background: #161b22; border-right: 1px solid #30363d; overflow-y: auto; }
        .main { flex: 1; padding: 1rem; overflow: auto; }
        .node-list { padding: 1rem; }
        .node-item { padding: 0.75rem; margin-bottom: 0.5rem; background: #21262d; border-radius: 6px; cursor: pointer; }
        .node-item:hover { background: #30363d; }
        .node-type { font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.25rem; }
        .node-type.goal { color: #f0e68c; }
        .node-type.decision { color: #00ced1; }
        .node-type.action { color: #90ee90; }
        .node-type.outcome { color: #87ceeb; }
        .node-type.observation { color: #dda0dd; }
        .node-type.option { color: #ffa07a; }
        .node-title { font-weight: 500; }
        .node-status { font-size: 0.75rem; color: #8b949e; margin-top: 0.25rem; }
        .stats { padding: 1rem; border-bottom: 1px solid #30363d; }
        .stat { display: flex; justify-content: space-between; padding: 0.25rem 0; }
        .detail-panel { background: #161b22; border-radius: 8px; padding: 1.5rem; max-width: 600px; }
        .detail-panel h2 { margin-bottom: 1rem; color: #58a6ff; }
        .detail-field { margin-bottom: 1rem; }
        .detail-label { font-size: 0.75rem; color: #8b949e; margin-bottom: 0.25rem; }
        .detail-value { color: #c9d1d9; }
        .edges-section { margin-top: 1.5rem; }
        .edge-item { padding: 0.5rem; background: #21262d; border-radius: 4px; margin-bottom: 0.5rem; font-size: 0.875rem; }
        .empty { color: #8b949e; text-align: center; padding: 2rem; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🌳 Deciduous - Decision Graph</h1>
    </div>
    <div class="container">
        <div class="sidebar">
            <div class="stats" id="stats"></div>
            <div class="node-list" id="nodeList"></div>
        </div>
        <div class="main" id="main">
            <div class="empty">Select a node to view details</div>
        </div>
    </div>
    <script>
        let graphData = { nodes: [], edges: [] };

        async function loadGraph() {
            try {
                const res = await fetch('/api/graph');
                const json = await res.json();
                if (json.ok) {
                    graphData = json.data;
                    renderStats();
                    renderNodeList();
                }
            } catch (e) {
                console.error('Failed to load graph:', e);
            }
        }

        function renderStats() {
            const stats = document.getElementById('stats');
            const types = {};
            graphData.nodes.forEach(n => {
                types[n.node_type] = (types[n.node_type] || 0) + 1;
            });
            stats.innerHTML = `
                <div class="stat"><span>Total Nodes</span><span>${graphData.nodes.length}</span></div>
                <div class="stat"><span>Total Edges</span><span>${graphData.edges.length}</span></div>
                ${Object.entries(types).map(([t, c]) => `<div class="stat"><span>${t}</span><span>${c}</span></div>`).join('')}
            `;
        }

        function renderNodeList() {
            const list = document.getElementById('nodeList');
            const sorted = [...graphData.nodes].sort((a, b) => b.id - a.id);
            list.innerHTML = sorted.map(n => `
                <div class="node-item" onclick="showNode(${n.id})">
                    <div class="node-type ${n.node_type}">${n.node_type}</div>
                    <div class="node-title">${n.title}</div>
                    <div class="node-status">#${n.id} · ${n.status}</div>
                </div>
            `).join('');
        }

        function showNode(id) {
            const node = graphData.nodes.find(n => n.id === id);
            if (!node) return;

            const inEdges = graphData.edges.filter(e => e.to_node_id === id);
            const outEdges = graphData.edges.filter(e => e.from_node_id === id);

            const main = document.getElementById('main');
            main.innerHTML = `
                <div class="detail-panel">
                    <h2>${node.title}</h2>
                    <div class="detail-field">
                        <div class="detail-label">Type</div>
                        <div class="detail-value">${node.node_type}</div>
                    </div>
                    <div class="detail-field">
                        <div class="detail-label">Status</div>
                        <div class="detail-value">${node.status}</div>
                    </div>
                    ${node.description ? `
                    <div class="detail-field">
                        <div class="detail-label">Description</div>
                        <div class="detail-value">${node.description}</div>
                    </div>
                    ` : ''}
                    <div class="detail-field">
                        <div class="detail-label">Created</div>
                        <div class="detail-value">${node.created_at}</div>
                    </div>
                    ${inEdges.length > 0 ? `
                    <div class="edges-section">
                        <div class="detail-label">Incoming Edges (${inEdges.length})</div>
                        ${inEdges.map(e => {
                            const from = graphData.nodes.find(n => n.id === e.from_node_id);
                            return `<div class="edge-item" onclick="showNode(${e.from_node_id})">
                                ← ${from?.title || 'Unknown'} (${e.edge_type})
                                ${e.rationale ? `<br><small>${e.rationale}</small>` : ''}
                            </div>`;
                        }).join('')}
                    </div>
                    ` : ''}
                    ${outEdges.length > 0 ? `
                    <div class="edges-section">
                        <div class="detail-label">Outgoing Edges (${outEdges.length})</div>
                        ${outEdges.map(e => {
                            const to = graphData.nodes.find(n => n.id === e.to_node_id);
                            return `<div class="edge-item" onclick="showNode(${e.to_node_id})">
                                → ${to?.title || 'Unknown'} (${e.edge_type})
                                ${e.rationale ? `<br><small>${e.rationale}</small>` : ''}
                            </div>`;
                        }).join('')}
                    </div>
                    ` : ''}
                </div>
            `;
        }

        loadGraph();
    </script>
</body>
</html>"#;

/// Start the decision graph viewer server
pub fn start_graph_server(port: u16) -> std::io::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let server = Server::http(&addr).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;

    let url = format!("http://localhost:{}", port);

    eprintln!("\n\x1b[1;32m🌳 Deciduous\x1b[0m");
    eprintln!("   Graph viewer: {}", url);
    eprintln!("   Press Ctrl+C to stop\n");

    // Handle requests
    for request in server.incoming_requests() {
        if let Err(e) = handle_request(request) {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}

fn handle_request(request: Request) -> std::io::Result<()> {
    let url = request.url().to_string();
    let path = url.split('?').next().unwrap_or("/");
    let method = request.method().clone();

    match (&method, path) {
        // Serve graph viewer UI
        (&Method::Get, "/") | (&Method::Get, "/graph") => {
            let response = Response::from_string(GRAPH_VIEWER_HTML)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
            request.respond(response)
        }

        // API: Get decision graph
        (&Method::Get, "/api/graph") => {
            let graph = get_decision_graph();
            let json = serde_json::to_string(&ApiResponse::success(graph))?;

            let response = Response::from_string(json)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
            request.respond(response)
        }

        // API: Get command log
        (&Method::Get, "/api/commands") => {
            let commands = get_command_log();
            let json = serde_json::to_string(&ApiResponse::success(commands))?;

            let response = Response::from_string(json)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
            request.respond(response)
        }

        // 404
        _ => {
            let response = Response::from_string("Not found").with_status_code(404);
            request.respond(response)
        }
    }
}

fn get_decision_graph() -> DecisionGraph {
    match Database::open() {
        Ok(db) => db.get_graph().unwrap_or_else(|_| DecisionGraph { nodes: vec![], edges: vec![] }),
        Err(_) => DecisionGraph { nodes: vec![], edges: vec![] },
    }
}

fn get_command_log() -> Vec<crate::db::CommandLog> {
    match Database::open() {
        Ok(db) => db.get_recent_commands(100).unwrap_or_default(),
        Err(_) => vec![],
    }
}
