// src/mcp/tool_handlers/query/edges.rs
//
// Edge-occurrence `workspace_query` surfaces: `forward_edges` and
// `reverse_edges`.
//
// Both select one adjacency bucket and keep the occurrences whose ASSERTING file
// is admitted by the effective scope (`WorkspaceScope::admits` over
// `StoredEdge::asserting_file`): the authorized workspace roots intersected with
// the optional `withinPath` narrowing. For a call fact the asserting file is the
// CALLER's file, so narrowing selects provenance and never the callee's
// declaration file.

use super::{
    discovery_field, query_scope, required_str, run_query_with_hydration, send_hydration_failure,
    send_scope_rejection,
};
use crate::mcp::McpState;
use crate::protocol::send_response;
use serde_json::Value;

/// `forward_edges`: outgoing semantic edges from an entity.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn handle_forward_edges(id: &Value, args: &Value, state: &McpState) {
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
    // Workspace scope: a query issued FOR a workspace answers with the evidence
    // asserted from inside that workspace (primary root + its configured
    // additional roots). `None` when the caller declared no workspace — a
    // root-less query keeps its previous unfiltered behaviour. An unauthorized
    // `withinPath` is refused before the index is consulted.
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
    let (results, count, hydration) =
        match run_query_with_hydration(state, "forward_edges", name, workspace_root, {
            let domain = domain_owned.clone();
            let et = et_owned.clone();
            let name = name_owned.clone();
            move |idx| {
                let r = match scope.as_ref() {
                    Some(scope) => {
                        idx.forward_edges_by_identity_in_scope(&domain, &et, &name, scope)
                    }
                    None => idx.forward_edges_by_identity(&domain, &et, &name),
                };
                let c = r.len();
                (serde_json::to_value(&r).unwrap_or_default(), c)
            }
        }) {
            Ok(result) => result,
            Err(error) => return send_hydration_failure(id, error),
        };
    // The semantic answer is `edges` + `count`; discovery diagnostics are
    // attached only when discovery deviated from its expected path.
    let mut structured = serde_json::json!({
        "edges": results,
        "count": count,
    });
    if let Some(discovery) = discovery_field(&hydration) {
        structured["discovery"] = discovery;
    }
    let content = super::content::render(
        "forward_edges",
        args,
        &structured,
        &state.config.additional_roots,
    );
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": content }],
            "structuredContent": structured,
        }
    }));
}

/// `reverse_edges`: incoming semantic edges to an entity.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn handle_reverse_edges(id: &Value, args: &Value, state: &McpState) {
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
    // Workspace scope: the primary defect this closes — `reverse_edges` used to
    // answer with every occurrence of the identity across the WHOLE session,
    // including real call facts authored by an unrelated indexed repository.
    // Occurrence provenance (`asserting_file`) now constrains the answer, and an
    // optional `withinPath` narrows it further to one provenance subtree.
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
    let (results, count, hydration) =
        match run_query_with_hydration(state, "reverse_edges", name, workspace_root, {
            let domain = domain_owned.clone();
            let et = et_owned.clone();
            let name = name_owned.clone();
            move |idx| {
                let r = match scope.as_ref() {
                    Some(scope) => {
                        idx.reverse_edges_by_identity_in_scope(&domain, &et, &name, scope)
                    }
                    None => idx.reverse_edges_by_identity(&domain, &et, &name),
                };
                let c = r.len();
                (serde_json::to_value(&r).unwrap_or_default(), c)
            }
        }) {
            Ok(result) => result,
            Err(error) => return send_hydration_failure(id, error),
        };
    // The semantic answer is `edges` + `count`; discovery diagnostics are
    // attached only when discovery deviated from its expected path.
    let mut structured = serde_json::json!({
        "edges": results,
        "count": count,
    });
    if let Some(discovery) = discovery_field(&hydration) {
        structured["discovery"] = discovery;
    }
    let content = super::content::render(
        "reverse_edges",
        args,
        &structured,
        &state.config.additional_roots,
    );
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": content }],
            "structuredContent": structured,
        }
    }));
}
