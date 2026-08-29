// src/cbm/handlers.rs
//
// MCP tool handlers for CBM integration.
// Self-contained — each handler takes (id, params, state) like all other handlers.
// Dispatched from `crate::mcp::tools::dispatch_tools_call`.

use crate::cbm::bridge::IndexingStatus;
use crate::mcp::McpState;
use crate::protocol::send_response;
use serde_json::Value;

/// Circuit breaker guard: check if CBM is healthy before proceeding.
/// Returns `true` if CBM is available, otherwise sends error response.
///
/// **Live-status fix:** This previously read `state.cbm_status` — a field
/// snapshotted at server startup and never refreshed. `bridge.update_status()`
/// keeps the bridge's own status current after every query, so the bridge is
/// the authoritative source here. Reading the stale `state.cbm_status` made
/// `graph_search` & co. disagree with `get_cbm_status` (which reads the bridge):
/// the gate could pass a query while indexing was still in progress (or block
/// one when the bridge had recovered).
fn check_cbm_healthy(id: &Value, status: &crate::cbm::CbmStatus) -> bool {
    if !status.is_available() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": format!("CBM unavailable: {}", status.summary()) }
        }));
        return false;
    }
    true
}

/// M-3 fix: Factor out bridge extraction boilerplate.
/// Returns `Some(&mut GraphBridge)` or sends "not available" error and returns `None`.
///
/// The bridge guard already holds the authoritative live status — we check
/// `bridge.status()`, not the stale `state.cbm_status` snapshot.
fn with_bridge<'a>(
    id: &Value,
    state: &'a McpState,
) -> Option<std::sync::MutexGuard<'a, Option<crate::cbm::GraphBridge>>> {
    let bridge_opt = state.graph_bridge_lock();
    if bridge_opt.is_none() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": "CBM not available. Install codebase-memory-mcp on PATH." }
        }));
        return None;
    }
    // Circuit breaker: reject early if the LIVE bridge status is degraded/unavailable.
    // (The bridge status is refreshed on every query via update_status().)
    if !check_cbm_healthy(id, bridge_opt.as_ref().unwrap().status()) {
        return None;
    }
    Some(bridge_opt)
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
    send_indexing_gate(id, bridge.ensure_indexed())
}

/// Variant of `ensure_indexed_or_error` that checks a SPECIFIC project's
/// indexing state. Used by `cbm_proxy`, whose target project is the one in
/// the request parameters — never an unrelated/stale active-project entry.
/// Unknown (untracked) projects pass through so they can never dead-end in
/// `StillIndexing{0}` forever.
pub(crate) fn ensure_indexed_or_error_for(
    id: &Value,
    bridge: &mut crate::cbm::GraphBridge,
    project: &str,
) -> bool {
    send_indexing_gate(id, bridge.ensure_indexed_for(project))
}

/// Shared indexing-gate responder. Sends the retry/ready/error MCP response
/// and returns `true` only when the project is ready for queries.
fn send_indexing_gate(
    id: &Value,
    status: Result<IndexingStatus, crate::cbm::client::CbmError>,
) -> bool {
    match status {
        Ok(IndexingStatus::Ready) => true,
        Ok(IndexingStatus::StillIndexing { elapsed_secs }) => {
            let msg = if elapsed_secs < 5 {
                "CBM project indexing in progress. Retry the query in a few seconds, or use `get_cbm_status` to check when indexing completes.".to_string()
            } else {
                format!(
                    "CBM is still indexing this project ({elapsed_secs}s elapsed). This is normal for large codebases. Retry the query shortly."
                )
            };
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": msg }],
                    "_meta": { "still_indexing": true, "elapsed_secs": elapsed_secs },
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
    let query = params["arguments"]["name_pattern"]
        .as_str()
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

    if !ensure_indexed_or_error(id, bridge) {
        return;
    }
    let nodes = bridge.search(query);
    let status = bridge.status().clone();
    if let Some(err) = bridge.take_last_error() {
        // Surface the failure: CBM is indexing, unavailable, or errored —
        // do NOT report "0 symbols" as if the search genuinely found nothing.
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": format!("CBM search failed: {err}") }],
                "isError": true,
                "structuredContent": {
                    "error": err.to_string(),
                    "cbm_status": status.summary()
                }
            }
        }));
        return;
    }
    // Build a human-readable content summary with node details.
    // This is the MINIMUM that every MCP client forwards to the model.
    let content_text = if nodes.is_empty() {
        "Found 0 symbol(s). Try a different query.".to_string()
    } else {
        let node_lines: Vec<String> = nodes
            .iter()
            .map(|n| {
                let label_str = if n.label.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", n.label)
                };
                format!("  - {}{} @ {}", n.name, label_str, n.file)
            })
            .collect();
        format!(
            "Found {} symbol(s):\n{}",
            nodes.len(),
            node_lines.join("\n")
        )
    };
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": content_text }],
            "structuredContent": {
                "nodes": nodes,
                "count": nodes.len(),
                "cbm_status": status.summary()
            }
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

    if !ensure_indexed_or_error(id, bridge) {
        return;
    }
    let result = bridge.query_graph(query);
    let status = bridge.status().clone();
    if let Some(err) = bridge.take_last_error() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": format!("CBM query failed: {err}") }],
                "isError": true,
                "structuredContent": {
                    "error": err.to_string(),
                    "cbm_status": status.summary()
                }
            }
        }));
        return;
    }
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("{} node(s), {} edge(s).", result.nodes.len(), result.edges.len()) }],
            "structuredContent": {
                "nodes": result.nodes, "edges": result.edges,
                "cbm_status": status.summary()
            }
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

    if !ensure_indexed_or_error(id, bridge) {
        return;
    }
    let edges = bridge.trace_path(from, to);
    let status = bridge.status().clone();
    if let Some(err) = bridge.take_last_error() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": format!("CBM trace failed: {err}") }],
                "isError": true,
                "structuredContent": {
                    "error": err.to_string(),
                    "cbm_status": status.summary()
                }
            }
        }));
        return;
    }
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("'{from}' → '{to}': {} edge(s).", edges.len()) }],
            "structuredContent": {
                "edges": edges, "count": edges.len(),
                "cbm_status": status.summary()
            }
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
    if !ensure_indexed_or_error(id, bridge) {
        return;
    }
    // F11: get_architecture now returns Result — failures are translated
    // into an error response instead of being conflated with empty data.
    match bridge.get_architecture() {
        Ok(arch) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": format!("{} module(s), {} deps.", arch.modules.len(), arch.dependencies.len()) }],
                    "structuredContent": {
                        "modules": arch.modules, "dependencies": arch.dependencies,
                        "cbm_status": status.summary()
                    }
                }
            }));
        }
        Err(err) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": format!("CBM architecture query failed: {err}") }],
                    "isError": true,
                    "structuredContent": {
                        "error": err.to_string(),
                        "cbm_status": status.summary()
                    }
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
///
/// **Report-only fix:** This handler no longer calls `bridge.ensure_indexed()`.
/// That call *triggered* indexing (spawning a background thread) and its result
/// was discarded (`let _ =`), so the first `get_cbm_status` call could set
/// indexing `InProgress` while still reporting "running and ready" (the state
/// map wasn't populated yet). The handler now only *reads* the current
/// indexing state via `bridge.indexing_state()`.
pub fn handle_get_cbm_status(id: &Value, _params: &Value, state: &McpState) {
    let (status, details, version, indexing_info, freshness_info) = match state
        .graph_bridge_lock()
        .as_mut()
    {
        Some(bridge) => {
            // Refresh the live circuit-breaker status (no indexing trigger).
            bridge.update_status();
            let s = bridge.status().clone();

            // Report the current indexing state WITHOUT triggering indexing.
            // `ensure_indexed()` has the side effect of starting indexing when
            // NotStarted; `get_cbm_status` must report, not mutate.
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

            // The human-readable detail must AGREE with the indexing state.
            // Before this fix: "CBM is running and ready." was returned
            // unconditionally when status was Available — even while a
            // background indexer was churning. That's the disagreement with
            // `graph_search`, which correctly reports `still_indexing`.
            let d = match &s {
                crate::cbm::CbmStatus::Available => match &idx_info {
                    // Indexing is progressing — queries will return empty
                    // results until it finishes; say so.
                    Some(info) if info["status"] == "in_progress" => {
                        let secs = info["elapsed_secs"].as_u64().unwrap_or(0);
                        format!(
                            "CBM is indexing this project ({secs}s elapsed) — graph queries will return empty or error results until indexing completes."
                        )
                    }
                    Some(info) if info["status"] == "failed" => {
                        format!(
                            "CBM indexing failed: {}",
                            info["error"].as_str().unwrap_or("unknown")
                        )
                    }
                    // Complete, or NotStarted (no indexing attempted — still
                    // ready for queries since ensure_indexed will kick it off).
                    _ => "CBM is running and ready.".into(),
                },
                crate::cbm::CbmStatus::Degraded(msg) => format!("CBM degraded: {msg}"),
                crate::cbm::CbmStatus::Unavailable => {
                    "CBM not installed or disabled. See github.com/DeusData/codebase-memory-mcp"
                        .into()
                }
            };
            let v = bridge.graph_version().to_string();

            // ── Freshness information (read-only, no indexing trigger) ──
            let freshness_info = {
                let f_map = bridge.freshness.lock().unwrap_or_else(|p| p.into_inner());
                let mut projects = serde_json::Map::new();
                for (slug, entry) in f_map.iter() {
                    projects.insert(
                        slug.clone(),
                        serde_json::json!({
                            "dirty_generation": entry.dirty_generation,
                            "indexed_generation": entry.indexed_generation,
                            "is_stale": entry.dirty_generation > entry.indexed_generation,
                        }),
                    );
                }
                if projects.is_empty() {
                    None
                } else {
                    Some(serde_json::json!({ "projects": projects }))
                }
            };

            (s, d, v, idx_info, freshness_info)
        }
        None => {
            let d = match &state.cbm_status {
                crate::cbm::CbmStatus::Available => "CBM configured but not connected.".into(),
                _ => "CBM not available.".into(),
            };
            (state.cbm_status.clone(), d, String::new(), None, None)
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
            "_meta": {
                "cbm_status": status.summary(),
                "graph_version": version,
            }
        }
    });
    if let Some(idx) = indexing_info {
        response["result"]["_meta"]["indexing"] = idx;
    }
    if let Some(fresh) = freshness_info {
        response["result"]["_meta"]["freshness"] = fresh;
    }
    if let Some(paths) = checked_paths {
        response["result"]["_meta"]["checked_paths"] = serde_json::json!(paths);
    }
    send_response(&response);
}

/// Handle the `index_repository` MCP tool.
///
/// D2 migration: moved verbatim from the inline dispatch arm in
/// `src/mcp/tools.rs` into the canonical registry handler path. Parameter
/// validation (`repo_path` required, `mode` defaults to `"fast"`) and the
/// routing through `cbm_proxy` are preserved exactly.
pub fn handle_index_repository(id: &Value, params: &Value, state: &McpState) {
    // Route through cbm_proxy with the caller's parameters
    let repo_path = crate::mcp::tool_helpers::arg_str_or_empty(params, "repo_path");
    if repo_path.is_empty() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32602, "message": "Missing required: repo_path" }
        }));
        return;
    }
    let mode = crate::mcp::tool_helpers::arg_str(params, "mode").unwrap_or("fast");
    crate::cbm::proxy::handle_cbm_proxy(
        id,
        &serde_json::json!({"arguments": {
            "cbm_tool": "index_repository",
            "parameters": {
                "repo_path": repo_path,
                "mode": mode
            }
        }}),
        state,
    );
}
