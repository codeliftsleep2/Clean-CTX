// src/cbm/handlers.rs
//
// MCP tool handlers for CBM integration.
// Self-contained — each handler takes (id, params, state) like all other handlers.
// Dispatched from `crate::mcp::tools::dispatch_tools_call`.

use serde_json::Value;
use crate::cbm::bridge::IndexingStatus;
use crate::mcp::McpState;
use crate::protocol::send_response;

/// Circuit breaker guard: check if CBM is healthy before proceeding.
/// Returns `true` if CBM is available, otherwise sends error response.
fn check_cbm_healthy(id: &Value, state: &McpState) -> bool {
    if state.cbm_status.summary() != "available" {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": format!("CBM unavailable: {}", state.cbm_status.summary()) }
        }));
        return false;
    }
    true
}

/// M-3 fix: Factor out bridge extraction boilerplate.
/// Returns `Some(&mut GraphBridge)` or sends "not available" error and returns `None`.
fn with_bridge<'a>(id: &Value, state: &'a McpState) -> Option<std::sync::MutexGuard<'a, Option<crate::cbm::GraphBridge>>> {
    // Circuit breaker: reject early if CBM is degraded/unavailable
    if !check_cbm_healthy(id, state) {
        return None;
    }
    let bridge_opt = state.graph_bridge_lock();
    if bridge_opt.is_some() {
        Some(bridge_opt)
    } else {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": "CBM not available. Install codebase-memory-mcp on PATH." }
        }));
        None
    }
}

/// M-3: Set project from params if provided.
///
/// Multi-repo support: accepts `workspaceRoot` (canonicalized) and derives
/// the project name from its directory name, so CBM queries scope to the
/// correct repo. Explicit `project` takes precedence over `workspaceRoot`.
fn set_project_from_params(bridge: &mut crate::cbm::GraphBridge, params: &Value) {
    if let Some(p) = params["arguments"]["project"].as_str() {
        bridge.set_project(p);
        return;
    }
    if let Some(root) = params["arguments"]["workspaceRoot"].as_str() {
        // Multi-repo: switch the workspace root so both the disk-cache
        // partition key and the derived project name scope to the correct repo.
        bridge.set_workspace_root(std::path::Path::new(root));
    }
}

/// P1-9: Ensure the project is indexed before issuing a CBM query.
///
/// **Non-blocking:** If indexing is in progress, sends a "retry later"
/// response (not an error) and returns `false`. The agent will retry
/// the query on the next turn. This prevents the 10-30s blocking
/// that previously occurred on every first CBM handler call.
pub(crate) fn ensure_indexed_or_error(id: &Value, bridge: &mut crate::cbm::GraphBridge) -> bool {
    match bridge.ensure_indexed() {
        Ok(IndexingStatus::Ready) => true,
        Ok(IndexingStatus::StillIndexing { elapsed_secs }) => {
            let msg = if elapsed_secs < 5 {
                "CBM project indexing in progress. Retry the query in a few seconds, or use `get_cbm_status` to check when indexing completes.".to_string()
            } else {
                format!("CBM is still indexing this project ({elapsed_secs}s elapsed). This is normal for large codebases. Retry the query shortly.")
            };
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": msg }],
                    "still_indexing": true,
                    "elapsed_secs": elapsed_secs,
                }
            }));
            false
        }
        Err(e) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32603, "message": format!("CBM indexing failed: {e}") }
            }));
            false
        }
    }
}


/// Handle `graph_search` — search the CBM knowledge graph.
///
/// M-02 fix: Accepts `name_pattern` (regex) or `query` (plain text substring) 
/// parameters matching CBM's actual search_graph tool interface.
pub fn handle_graph_search(id: &Value, params: &Value, state: &McpState) {
    let mut bridge_guard = match with_bridge(id, state) {
        Some(b) => b,
        None => return,
    };
    let bridge = bridge_guard.as_mut().unwrap();
    // M-02: support both `name_pattern` (regex) and `query` (plain text)
    let query = params["arguments"]["name_pattern"].as_str()
        .or_else(|| params["arguments"]["query"].as_str())
        .unwrap_or("");
    if query.is_empty() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32602, "message": "Missing required: name_pattern or query" }
        }));
        return;
    }
    set_project_from_params(bridge, params);

    if !ensure_indexed_or_error(id, bridge) { return; }
    let nodes = bridge.search(query);
    let status = bridge.status().clone();
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {} symbol(s).", nodes.len()) }],
            "nodes": nodes, "count": nodes.len(),
            "cbm_status": status.summary()
        }
    }));
}

/// Handle `graph_query` — execute a Cypher-like graph query.
pub fn handle_graph_query(id: &Value, params: &Value, state: &McpState) {
    let mut bridge_guard = match with_bridge(id, state) {
        Some(b) => b,
        None => return,
    };
    let bridge = bridge_guard.as_mut().unwrap();
    let query = params["arguments"]["query"].as_str().unwrap_or("");
    if query.is_empty() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32602, "message": "Missing required: query" }
        }));
        return;
    }
    set_project_from_params(bridge, params);

    if !ensure_indexed_or_error(id, bridge) { return; }
    let result = bridge.query_graph(query);
    let status = bridge.status().clone();
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("{} node(s), {} edge(s).", result.nodes.len(), result.edges.len()) }],
            "nodes": result.nodes, "edges": result.edges,
            "cbm_status": status.summary()
        }
    }));
}

/// Handle `graph_trace` — trace a path between two symbols.
pub fn handle_graph_trace(id: &Value, params: &Value, state: &McpState) {
    let mut bridge_guard = match with_bridge(id, state) {
        Some(b) => b,
        None => return,
    };
    let bridge = bridge_guard.as_mut().unwrap();
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

    if !ensure_indexed_or_error(id, bridge) { return; }
    let edges = bridge.trace_path(from, to);
    let status = bridge.status().clone();
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("'{from}' → '{to}': {} edge(s).", edges.len()) }],
            "edges": edges, "count": edges.len(),
            "cbm_status": status.summary()
        }
    }));
}

/// Handle `get_architecture` — get project architecture overview.
pub fn handle_get_architecture(id: &Value, params: &Value, state: &McpState) {
    let mut bridge_guard = match with_bridge(id, state) {
        Some(b) => b,
        None => return,
    };
    let bridge = bridge_guard.as_mut().unwrap();
    set_project_from_params(bridge, params);

    let status = bridge.status().clone();
    if !ensure_indexed_or_error(id, bridge) { return; }
    match bridge.get_architecture() {
        Some(arch) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": format!("{} module(s), {} deps.", arch.modules.len(), arch.dependencies.len()) }],
                    "modules": arch.modules, "dependencies": arch.dependencies,
                    "cbm_status": status.summary()
                }
            }));
        }
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": "Architecture overview not available." }],
                    "modules": [], "dependencies": [],
                    "cbm_status": status.summary()
                }
            }));
        }
    }
}

/// P1-9: Handle `get_cbm_status` — check CBM availability with indexing progress.
///
/// When CBM is unavailable, the response includes `checked_paths` — a list of
/// all binary locations that were searched (config, PATH, common install dirs).
/// This makes detection issues trivially debuggable without reading stderr logs.
pub fn handle_get_cbm_status(id: &Value, _params: &Value, state: &McpState) {
    let (status, details, version, indexing_info) = match state.graph_bridge_lock().as_mut() {
        Some(bridge) => {
            bridge.update_status();
            let _ = bridge.ensure_indexed();
            let s = bridge.status().clone();
            let d = match &s {
                crate::cbm::CbmStatus::Available => "CBM is running and ready.".into(),
                crate::cbm::CbmStatus::Degraded(msg) => format!("CBM degraded: {msg}"),
                crate::cbm::CbmStatus::Unavailable =>
                    "CBM not installed or disabled. See github.com/DeusData/codebase-memory-mcp".into(),
            };
            let v = bridge.graph_version().to_string();
            // P1-9: Check indexing state for progress reporting.
            // Multi-repo: the state map is keyed by project name; report
            // the first non-NotStarted entry found.
            let idx_info = {
                let idx_states = bridge.indexing_state();
                let mut found = None;
                for (project, state) in idx_states.iter() {
                    match state {
                        crate::cbm::bridge::IndexingState::InProgress { started_at } => {
                            let elapsed = started_at.elapsed().as_secs();
                            found = Some(serde_json::json!({
                                "status": "in_progress",
                                "elapsed_secs": elapsed,
                                "project": project,
                            }));
                            break;
                        }
                        crate::cbm::bridge::IndexingState::Complete => {
                            found = Some(serde_json::json!({
                                "status": "complete",
                                "project": project,
                            }));
                            break;
                        }
                        crate::cbm::bridge::IndexingState::Failed(msg) => {
                            found = Some(serde_json::json!({
                                "status": "failed",
                                "error": msg,
                                "project": project,
                            }));
                            break;
                        }
                        crate::cbm::bridge::IndexingState::NotStarted => {}
                    }
                }
                found
            };
            (s, d, v, idx_info)
        }
        None => {
            let d = match &state.cbm_status {
                crate::cbm::CbmStatus::Available => "CBM configured but not connected.".into(),
                _ => "CBM not available.".into(),
            };
            (state.cbm_status.clone(), d, String::new(), None)
        }
    };

    // When unavailable, include diagnostic info: all checked paths
    let checked_paths = if !status.is_available() {
        Some(crate::cbm::bridge::checked_paths())
    } else {
        None
    };

    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": details }],
            "cbm_status": status.summary(),
            "graph_version": version,
        }
    });
    if let Some(idx) = indexing_info {
        response["result"]["indexing"] = idx;
    }
    if let Some(paths) = checked_paths {
        response["result"]["checked_paths"] = serde_json::json!(paths);
    }
    send_response(&response);
}
