// src/mcp/tool_handlers/query/entities.rs
//
// Entity-occurrence `workspace_query` surfaces: `find_entities` and
// `entities_in_file`.
//
// `find_entities` keeps the occurrences whose own file provenance
// (`EntityRef.file`) is admitted by the effective scope: the authorized
// workspace roots intersected with the optional `withinPath` narrowing, decided
// by the one shared `WorkspaceScope::admits` rule.
//
// `entities_in_file` is the exception that proves the rule: its explicit
// `file_path` is already validated against the trusted-path/root boundary (so
// the WSC-004 roots are the accepted set), and only the SECOND layer is applied
// here — when a `withinPath` is present, the explicit file must lie inside it.
// With no `withinPath` the accepted set is unchanged, so no redundant occurrence
// filter is added.

use super::{query_scope, required_str, run_query_with_hydration, send_scope_rejection};
use crate::mcp::McpState;
use crate::protocol::send_response;
use serde_json::Value;

/// `find_entities`: find entities by name (cross-domain/type).
///
/// Eligible for one-cycle hydration: has a name for CBM candidate discovery.
pub(super) fn handle_find_entities(id: &Value, args: &Value, state: &McpState) {
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
    // Workspace scope: a query issued FOR a workspace answers with the entity
    // occurrences that workspace's own files declare (`None` = the caller
    // declared no workspace, so the previous unfiltered behaviour stands).
    // Built once per query, so the initial answer and the post-hydration rerun
    // share one root set, an optional `withinPath` narrowing included, and an
    // unauthorized `withinPath` is refused before the index is consulted.
    let scope = match query_scope(state, args) {
        Ok(scope) => scope,
        Err(message) => {
            send_scope_rejection(id, message);
            return;
        }
    };
    let (results, count, hydration_attempted, hydration) =
        run_query_with_hydration(state, "find_entities", name, workspace_root, move |idx| {
            let r = match scope.as_ref() {
                Some(scope) => idx.find_entities_by_name_in_scope(name, scope),
                None => idx.find_entities_by_name(name),
            };
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

/// `entities_in_file`: list all entities defined in a given file.
pub(super) fn handle_entities_in_file(id: &Value, args: &Value, state: &McpState) {
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
    // The effective scope: the WSC-004 roots are already enforced by the
    // trusted-path/root validation below (they ARE its accepted set), and an
    // invalid `withinPath` is refused before any path is resolved.
    let scope = match query_scope(state, args) {
        Ok(scope) => scope,
        Err(message) => {
            send_scope_rejection(id, message);
            return;
        }
    };
    let resolved_path = match crate::mcp::tool_helpers::resolve_file_path_checked(
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
            send_empty_entities_in_file(id);
            return;
        }
    };
    let canonical_path = crate::dictionary::path::canonical_identity_key(&resolved_path);
    // The optional SECOND layer: when a `withinPath` is present, the explicit file
    // must lie inside it, and a file outside it is answered exactly like a file
    // outside the workspace (the same minimal zero-result shape). With no
    // `withinPath` this check is skipped, so the accepted set is unchanged and no
    // redundant occurrence scan is added.
    if scope
        .as_ref()
        .is_some_and(|scope| scope.has_narrowing() && !scope.admits(&canonical_path))
    {
        send_empty_entities_in_file(id);
        return;
    }
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

/// The minimal zero-result shape for an `entities_in_file` request whose file is
/// not answerable inside the effective scope: it does not exist, it lies outside
/// the workspace roots, or it lies outside the `withinPath` narrowing. The shape
/// is exactly the pre-`withinPath` one (no hydration metadata at all), so an
/// out-of-scope path is answered as it always was.
fn send_empty_entities_in_file(id: &Value) {
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": "Found 0 entities in file." }],
            "structuredContent": { "entities": [], "count": 0 }
        }
    }));
}
