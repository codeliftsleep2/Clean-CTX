// src/mcp/tool_handlers/query/graph.rs
//
// Graph-traversal `workspace_query` surfaces: `transitive_dependencies` and
// `has_cycle`.
//
// The effective scope is applied DURING the walk, never to the returned list:
// reachability and cycle membership are properties of the traversal, so an edge
// that is outside the authorized workspace — or outside the `withinPath`
// narrowing — may neither extend a path nor close a cycle.

use super::{
    optional_i32, query_scope, required_str, run_query_with_hydration, send_scope_rejection,
};
use crate::mcp::McpState;
use crate::protocol::send_response;
use serde_json::Value;

/// `transitive_dependencies`: BFS dependency traversal.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn handle_transitive_dependencies(id: &Value, args: &Value, state: &McpState) {
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
    // Workspace scope: reachability is computed from THIS workspace's evidence
    // only — filtering the returned list could not achieve that, since the walk
    // itself must not pass through another repository's edges (see
    // `WorkspaceIndex::transitive_dependencies_in_scope`). A `withinPath` narrows
    // it further, so an out-of-path edge cannot extend the walk either.
    let scope = match query_scope(state, args) {
        Ok(scope) => scope,
        Err(message) => {
            send_scope_rejection(id, message);
            return;
        }
    };
    let domain_owned = domain.to_string();
    let et_owned = entity_type.to_string();
    let name_owned = name.to_string();
    let depth_captured = depth;
    let (results, count, hydration_attempted, hydration) =
        run_query_with_hydration(state, "transitive_dependencies", name, workspace_root, {
            let domain = domain_owned.clone();
            let et = et_owned.clone();
            let name = name_owned.clone();
            move |idx| {
                let r = match scope.as_ref() {
                    Some(scope) => idx.transitive_dependencies_in_scope(
                        &domain,
                        &et,
                        &name,
                        depth_captured,
                        scope,
                    ),
                    None => idx.transitive_dependencies(&domain, &et, &name, depth_captured),
                };
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
                "discovery_provider": hydration.discovery_provider,
                "discovery_status": hydration.discovery_status,
                "discovery_completed": hydration.discovery_completed,
                "fallback_occurred": hydration.fallback_occurred,
                "fallback_reason": hydration.fallback_reason,
                "candidates_discovered": hydration.candidates_discovered,
                "candidates_compiled": hydration.candidates_compiled,
                "project_coverage": hydration.project_coverage,
                "project_coverage_truncated": hydration.project_coverage_truncated,
            }
        }
    }));
}

/// `has_cycle`: detect cycles in the entity graph.
///
/// NOT hydration-eligible: workspace-wide property, no entity identity.
///
/// Workspace scope: when a workspace root is declared, only edge occurrences
/// ASSERTED inside that workspace count as cycle edges, so two repositories that
/// each contribute one half of a cycle can never be combined into a cycle report
/// that neither workspace actually contains. Cycle membership itself is
/// unchanged (`Calls` stays excluded, see `WorkspaceIndex::has_cycle_in_scope`).
pub(super) fn handle_has_cycle(id: &Value, args: &Value, state: &McpState) {
    // The effective scope is resolved BEFORE the index lock is taken, so an
    // unauthorized `withinPath` is refused without touching the index at all.
    let scope = match query_scope(state, args) {
        Ok(scope) => scope,
        Err(message) => {
            send_scope_rejection(id, message);
            return;
        }
    };
    let idx = state.workspace_index_read();
    let has_cycle = match scope.as_ref() {
        Some(scope) => idx.has_cycle_in_scope(scope),
        None => idx.has_cycle(),
    };
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
