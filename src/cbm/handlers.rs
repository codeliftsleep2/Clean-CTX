// src/cbm/handlers.rs
//
// MCP tool handlers for CBM integration.
// Self-contained — each handler takes (id, params, state) like all other handlers.
// Dispatched from `crate::mcp::tools::dispatch_tools_call`.

use serde_json::Value;
use crate::mcp::McpState;
use crate::protocol::send_response;

/// M-3 fix: Factor out bridge extraction boilerplate.
/// Returns `Some(&mut GraphBridge)` or sends "not available" error and returns `None`.
fn with_bridge<'a>(id: &Value, state: &'a mut McpState) -> Option<&'a mut crate::cbm::GraphBridge> {
    match state.graph_bridge.as_mut() {
        Some(b) => Some(b),
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32603, "message": "CBM not available. Install codebase-memory-mcp on PATH." }
            }));
            None
        }
    }
}

/// M-3: Set project from params if provided.
fn set_project_from_params(bridge: &mut crate::cbm::GraphBridge, params: &Value) {
    if let Some(p) = params["arguments"]["project"].as_str() {
        bridge.set_project(p);
    }
}

/// Handle `graph_search` — search the CBM knowledge graph.
pub fn handle_graph_search(id: &Value, params: &Value, state: &mut McpState) {
    let bridge = match with_bridge(id, state) {
        Some(b) => b,
        None => return,
    };
    let query = params["arguments"]["query"].as_str().unwrap_or("");
    if query.is_empty() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32602, "message": "Missing required: query" }
        }));
        return;
    }
    set_project_from_params(bridge, params);

    let nodes = bridge.search(query);
    state.cbm_status = bridge.status().clone();
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {} symbol(s).", nodes.len()) }],
            "nodes": nodes, "count": nodes.len(),
            "cbm_status": state.cbm_status.summary()
        }
    }));
}

/// Handle `graph_query` — execute a Cypher-like graph query.
pub fn handle_graph_query(id: &Value, params: &Value, state: &mut McpState) {
    let bridge = match with_bridge(id, state) {
        Some(b) => b,
        None => return,
    };
    let query = params["arguments"]["query"].as_str().unwrap_or("");
    if query.is_empty() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32602, "message": "Missing required: query" }
        }));
        return;
    }
    set_project_from_params(bridge, params);

    let result = bridge.query_graph(query);
    state.cbm_status = bridge.status().clone();
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("{} node(s), {} edge(s).", result.nodes.len(), result.edges.len()) }],
            "nodes": result.nodes, "edges": result.edges,
            "cbm_status": state.cbm_status.summary()
        }
    }));
}

/// Handle `graph_trace` — trace a path between two symbols.
pub fn handle_graph_trace(id: &Value, params: &Value, state: &mut McpState) {
    let bridge = match with_bridge(id, state) {
        Some(b) => b,
        None => return,
    };
    let from = params["arguments"]["from"].as_str().unwrap_or("");
    let to = params["arguments"]["to"].as_str().unwrap_or("");
    if from.is_empty() || to.is_empty() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32602, "message": "Missing required: from, to" }
        }));
        return;
    }
    set_project_from_params(bridge, params);

    let edges = bridge.trace_path(from, to);
    state.cbm_status = bridge.status().clone();
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("'{from}' → '{to}': {} edge(s).", edges.len()) }],
            "edges": edges, "count": edges.len(),
            "cbm_status": state.cbm_status.summary()
        }
    }));
}

/// Handle `get_architecture` — get project architecture overview.
pub fn handle_get_architecture(id: &Value, params: &Value, state: &mut McpState) {
    let bridge = match with_bridge(id, state) {
        Some(b) => b,
        None => return,
    };
    set_project_from_params(bridge, params);

    match bridge.get_architecture() {
        Some(arch) => {
            state.cbm_status = bridge.status().clone();
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": format!("{} module(s), {} deps.", arch.modules.len(), arch.dependencies.len()) }],
                    "modules": arch.modules, "dependencies": arch.dependencies,
                    "cbm_status": state.cbm_status.summary()
                }
            }));
        }
        None => {
            state.cbm_status = bridge.status().clone();
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": "Architecture overview not available." }],
                    "modules": [], "dependencies": [],
                    "cbm_status": state.cbm_status.summary()
                }
            }));
        }
    }
}

/// Handle `get_cbm_status` — check CBM availability.
pub fn handle_get_cbm_status(id: &Value, _params: &Value, state: &mut McpState) {
    let (status, details, version) = match state.graph_bridge.as_mut() {
        Some(bridge) => {
            bridge.update_status();
            let s = bridge.status().clone();
            let d = match &s {
                crate::cbm::CbmStatus::Available => "CBM is running and ready.".into(),
                crate::cbm::CbmStatus::Degraded(msg) => format!("CBM degraded: {msg}"),
                crate::cbm::CbmStatus::Unavailable =>
                    "CBM not installed or disabled. See github.com/DeusData/codebase-memory-mcp".into(),
            };
            let v = bridge.graph_version().to_string();
            (s, d, v)
        }
        None => {
            let d = match &state.cbm_status {
                crate::cbm::CbmStatus::Available => "CBM configured but not connected.".into(),
                _ => "CBM not available.".into(),
            };
            (state.cbm_status.clone(), d, String::new())
        }
    };
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": details }],
            "cbm_status": status.summary(),
            "graph_version": version,
        }
    }));
}
