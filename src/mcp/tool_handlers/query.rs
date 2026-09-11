// src/mcp/tool_handlers/query.rs
//
// workspace_query handler — read-only MCP API over WorkspaceIndex.
//
// This handler exposes the existing WorkspaceIndex query methods as an MCP
// tool. It is a thin read boundary over the already-wired write lifecycle
// established in Phases A/B.
//
// Bounded hydration: for eligible query types, after the initial WorkspaceIndex
// query, ONE bounded CBM candidate-file discovery pass may run across the
// primary root plus configured additional roots. CBM supplies ONLY candidate
// file paths. Those paths flow through resolve_file_path_checked →
// compile_file_ir_focused → Clean-CTX semantic extraction → WorkspaceIndex.
// CBM graph semantics (edge counts, relationship types, etc.) never enter the
// WorkspaceIndex. The original query reruns exactly once after hydration.

use crate::mcp::McpState;
use crate::protocol::send_response;
use serde_json::Value;

/// Handle `workspace_query` — read-only cross-file semantic queries.
pub(crate) fn handle_workspace_query(id: &Value, params: &Value, state: &McpState) {
    let args = &params["arguments"];
    let query_type = match args["type"].as_str() {
        Some(t) => t,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'type'. Supported values: \
                     find_entities, forward_edges, reverse_edges, entities_in_file, \
                     transitive_dependencies, has_cycle.".to_string()
                }
            }));
            return;
        }
    };

    match query_type {
        "find_entities" => handle_find_entities(id, args, state),
        "forward_edges" => handle_forward_edges(id, args, state),
        "reverse_edges" => handle_reverse_edges(id, args, state),
        "entities_in_file" => handle_entities_in_file(id, args, state),
        "transitive_dependencies" => handle_transitive_dependencies(id, args, state),
        "has_cycle" => handle_has_cycle(id, state),
        _ => {
            // ... error handling unchanged
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": format!(
                        "Unknown query type: '{}'. Supported values: find_entities, \
                         forward_edges, reverse_edges, entities_in_file, \
                         transitive_dependencies, has_cycle.",
                        query_type
                    )
                }
            }));
        }
    }
}

/// `find_entities`: find entities by name (cross-domain/type).
///
/// Eligible for bounded hydration: has a name for CBM candidate discovery.
fn handle_find_entities(id: &Value, args: &Value, state: &McpState) {
    let name = match required_str(args, "name") {
        Some(n) => n,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'name' for find_entities query.".to_string()
                }
            }));
            return;
        }
    };
    let workspace_root = args["workspaceRoot"].as_str();
    let (results, count, hydration_attempted, candidates_discovered, candidates_compiled) =
        run_query_with_hydration(state, "find_entities", name, workspace_root, |idx| {
            let r = idx.find_entities_by_name(name);
            let c = r.len();
            (serde_json::to_value(&r).unwrap_or_default(), c)
        });
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {count} entities.") }],
            "structuredContent": {
                "entities": results,
                "count": count,
                "hydration_attempted": hydration_attempted,
                "candidates_discovered": candidates_discovered,
                "candidates_compiled": candidates_compiled,
            }
        }
    }));
}

/// Run a WorkspaceIndex query with optional bounded hydration.
///
/// 1. Run the initial query against current WorkspaceIndex (authoritative for
///    what Clean-CTX currently knows).
/// 2. If the query is hydration-eligible, perform ONE bounded hydration pass.
/// 3. Rerun the original query exactly once.
/// 4. Return final results + hydration metadata.
fn run_query_with_hydration<F>(
    state: &McpState,
    query_type: &str,
    query_name: &str,
    workspace_root: Option<&str>,
    query_fn: F,
) -> (Value, usize, bool, usize, usize)
where
    F: Fn(&crate::workspace::index::WorkspaceIndex) -> (Value, usize),
{
    // Step 1: Initial authoritative query.
    let (initial_results, initial_count) = {
        let idx = state.workspace_index_read();
        query_fn(&idx)
    };

    // Step 2: Evaluate hydration eligibility (independent of result count).
    let eligible = super::hydration::is_hydration_eligible(
        query_type,
        &serde_json::json!({ "name": query_name }),
    );
    if !eligible {
        return (initial_results, initial_count, false, 0, 0);
    }

    // Step 3: One bounded hydration pass.
    let (discovered, compiled) =
        super::hydration::hydrate_workspace_index(state, query_name, workspace_root);

    // Step 4: Rerun original query exactly once.
    let (final_results, final_count) = {
        let idx = state.workspace_index_read();
        query_fn(&idx)
    };

    (final_results, final_count, true, discovered, compiled)
}

/// `forward_edges`: outgoing semantic edges from an entity.
///
/// Eligible for bounded hydration: has (domain, entity_type, name) identity.
fn handle_forward_edges(id: &Value, args: &Value, state: &McpState) {
    let domain = match required_str(args, "domain") {
        Some(d) => d,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'domain' for forward_edges query.".to_string()
                }
            }));
            return;
        }
    };
    let entity_type = match required_str(args, "entity_type") {
        Some(t) => t,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'entity_type' for forward_edges query.".to_string()
                }
            }));
            return;
        }
    };
    let name = match required_str(args, "name") {
        Some(n) => n,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'name' for forward_edges query.".to_string()
                }
            }));
            return;
        }
    };
    let workspace_root = args["workspaceRoot"].as_str();
    let domain_owned = domain.to_string();
    let et_owned = entity_type.to_string();
    let name_owned = name.to_string();
    let (results, count, hydration_attempted, candidates_discovered, candidates_compiled) =
        run_query_with_hydration(state, "forward_edges", name, workspace_root, {
            let domain = domain_owned.clone();
            let et = et_owned.clone();
            let name = name_owned.clone();
            move |idx| {
                let r = idx.forward_edges_by_identity(&domain, &et, &name);
                let c = r.len();
                (serde_json::to_value(&r).unwrap_or_default(), c)
            }
        });
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {count} outgoing edges.") }],
            "structuredContent": {
                "edges": results,
                "count": count,
                "hydration_attempted": hydration_attempted,
                "candidates_discovered": candidates_discovered,
                "candidates_compiled": candidates_compiled,
            }
        }
    }));
}

/// `reverse_edges`: incoming semantic edges to an entity.
///
/// Eligible for bounded hydration: has (domain, entity_type, name) identity.
fn handle_reverse_edges(id: &Value, args: &Value, state: &McpState) {
    let domain = match required_str(args, "domain") {
        Some(d) => d,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'domain' for reverse_edges query.".to_string()
                }
            }));
            return;
        }
    };
    let entity_type = match required_str(args, "entity_type") {
        Some(t) => t,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'entity_type' for reverse_edges query.".to_string()
                }
            }));
            return;
        }
    };
    let name = match required_str(args, "name") {
        Some(n) => n,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'name' for reverse_edges query.".to_string()
                }
            }));
            return;
        }
    };
    let workspace_root = args["workspaceRoot"].as_str();
    let domain_owned = domain.to_string();
    let et_owned = entity_type.to_string();
    let name_owned = name.to_string();
    let (results, count, hydration_attempted, candidates_discovered, candidates_compiled) =
        run_query_with_hydration(state, "reverse_edges", name, workspace_root, {
            let domain = domain_owned.clone();
            let et = et_owned.clone();
            let name = name_owned.clone();
            move |idx| {
                let r = idx.reverse_edges_by_identity(&domain, &et, &name);
                let c = r.len();
                (serde_json::to_value(&r).unwrap_or_default(), c)
            }
        });
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {count} incoming edges.") }],
            "structuredContent": {
                "edges": results,
                "count": count,
                "hydration_attempted": hydration_attempted,
                "candidates_discovered": candidates_discovered,
                "candidates_compiled": candidates_compiled,
            }
        }
    }));
}

/// `entities_in_file`: list all entities defined in a given file.
fn handle_entities_in_file(id: &Value, args: &Value, state: &McpState) {
    let file_path = match required_str(args, "file_path") {
        Some(p) => p,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'file_path' for entities_in_file query.".to_string()
                }
            }));
            return;
        }
    };
    let workspace_root = args["workspaceRoot"].as_str();
    let resolved_path = match super::super::tool_helpers::resolve_file_path_checked(
        file_path,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(_msg) => {
            // File does not exist or is outside workspace boundary —
            // return empty results (the user asked for a file that
            // hasn't been compiled). This matches the pre-fix behavior
            // where a non-existent path produced no entities.
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": "Found 0 entities in file." }],
                    "structuredContent": { "entities": [], "count": 0 }
                }
            }));
            return;
        }
    };
    let canonical_path = crate::dictionary::path::canonical_identity_key(&resolved_path);
    let idx = state.workspace_index_read();
    let results = idx.entities_in_file(&canonical_path);
    let serialized = serde_json::to_value(&results).unwrap_or_default();
    let count = results.len();
    // entities_in_file is NOT hydration-eligible (no entity name for CBM search).
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {count} entities in file.") }],
            "structuredContent": {
                "entities": serialized,
                "count": count,
                "hydration_attempted": false,
            }
        }
    }));
}

/// `transitive_dependencies`: BFS dependency traversal.
///
/// Eligible for bounded hydration: has (domain, entity_type, name) identity.
fn handle_transitive_dependencies(id: &Value, args: &Value, state: &McpState) {
    let domain = match required_str(args, "domain") {
        Some(d) => d,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'domain' for transitive_dependencies query.".to_string()
                }
            }));
            return;
        }
    };
    let entity_type = match required_str(args, "entity_type") {
        Some(t) => t,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'entity_type' for transitive_dependencies query.".to_string()
                }
            }));
            return;
        }
    };
    let name = match required_str(args, "name") {
        Some(n) => n,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'name' for transitive_dependencies query.".to_string()
                }
            }));
            return;
        }
    };
    let depth = optional_i32(args, "depth", 1);
    let workspace_root = args["workspaceRoot"].as_str();
    let domain_owned = domain.to_string();
    let et_owned = entity_type.to_string();
    let name_owned = name.to_string();
    let depth_captured = depth;
    let (results, count, hydration_attempted, candidates_discovered, candidates_compiled) =
        run_query_with_hydration(state, "transitive_dependencies", name, workspace_root, {
            let domain = domain_owned.clone();
            let et = et_owned.clone();
            let name = name_owned.clone();
            move |idx| {
                let r = idx.transitive_dependencies(&domain, &et, &name, depth_captured);
                let c = r.len();
                (serde_json::to_value(&r).unwrap_or_default(), c)
            }
        });
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {count} dependencies (depth {depth}).") }],
            "structuredContent": {
                "dependencies": results,
                "count": count,
                "depth_used": depth,
                "hydration_attempted": hydration_attempted,
                "candidates_discovered": candidates_discovered,
                "candidates_compiled": candidates_compiled,
            }
        }
    }));
}

/// `has_cycle`: detect cycles in the entity graph.
///
/// NOT hydration-eligible: workspace-wide property, no entity identity.
fn handle_has_cycle(id: &Value, state: &McpState) {
    let idx = state.workspace_index_read();
    let has_cycle = idx.has_cycle();
    let text = if has_cycle {
        "Cycle detected."
    } else {
        "No cycle detected."
    };
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": text }],
            "structuredContent": {
                "has_cycle": has_cycle,
                "hydration_attempted": false,
            }
        }
    }));
}

/// Extract a required string argument from the arguments object.
fn required_str<'a>(args: &'a Value, name: &str) -> Option<&'a str> {
    args[name].as_str().filter(|s| !s.is_empty())
}

/// Extract an optional integer argument; returns `default` if missing.
fn optional_i32(args: &Value, name: &str, default: i32) -> i32 {
    args[name].as_i64().map(|v| v as i32).unwrap_or(default)
}

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query.rs"]
mod tests;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query_2.rs"]
mod tests_hydration;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query_3.rs"]
mod tests_multi_root_hydration;
