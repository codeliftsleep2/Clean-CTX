// src/mcp/tool_handlers/query.rs
//
// workspace_query handler — read-only MCP API over WorkspaceIndex.
//
// This handler exposes the existing WorkspaceIndex query methods as an MCP
// tool. It does not introduce new indexing, state, identity models, graph
// subsystems, or CBM integration. It is a thin read boundary over the
// already-wired write lifecycle established in Phases A/B.

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
    let idx = state.workspace_index_read();
    let results = idx.find_entities_by_name(name);
    let serialized = serde_json::to_value(&results).unwrap_or_default();
    let count = results.len();
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {count} entities.") }],
            "structuredContent": { "entities": serialized, "count": count }
        }
    }));
}

/// `forward_edges`: outgoing semantic edges from an entity.
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
    let idx = state.workspace_index_read();
    let results = idx.forward_edges_by_identity(domain, entity_type, name);
    let serialized = serde_json::to_value(&results).unwrap_or_default();
    let count = results.len();
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {count} outgoing edges.") }],
            "structuredContent": { "edges": serialized, "count": count }
        }
    }));
}

/// `reverse_edges`: incoming semantic edges to an entity.
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
    let idx = state.workspace_index_read();
    let results = idx.reverse_edges_by_identity(domain, entity_type, name);
    let serialized = serde_json::to_value(&results).unwrap_or_default();
    let count = results.len();
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {count} incoming edges.") }],
            "structuredContent": { "edges": serialized, "count": count }
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
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {count} entities in file.") }],
            "structuredContent": { "entities": serialized, "count": count }
        }
    }));
}

/// `transitive_dependencies`: BFS dependency traversal.
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
    let idx = state.workspace_index_read();
    let results = idx.transitive_dependencies(domain, entity_type, name, depth);
    let serialized = serde_json::to_value(&results).unwrap_or_default();
    let count = results.len();
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Found {count} dependencies (depth {depth}).") }],
            "structuredContent": {
                "dependencies": serialized,
                "count": count,
                "depth_used": depth
            }
        }
    }));
}

/// `has_cycle`: detect cycles in the entity graph.
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
            "structuredContent": { "has_cycle": has_cycle }
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
