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
    discovery_field, identity::resolve_identity, outcome::QueryAnswer, outcome::QueryFailure,
    outcome::QueryResult, query_scope, required_str, run_query_with_hydration, scope_failure,
};
use crate::mcp::McpState;
use serde_json::Value;

/// `forward_edges`: outgoing semantic edges from an entity.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn evaluate_forward_edges(args: &Value, state: &McpState) -> QueryResult {
    let name = required_str(args, "name").ok_or_else(|| {
        QueryFailure::invalid("Missing required argument: 'name' for forward_edges query.")
    })?;
    let workspace_root = args["workspaceRoot"].as_str();
    // Workspace scope: a query issued FOR a workspace answers with the evidence
    // asserted from inside that workspace (primary root + its configured
    // additional roots). `None` when the caller declared no workspace — a
    // root-less query keeps its previous unfiltered behaviour. An unauthorized
    // `withinPath` is refused before the index is consulted.
    let scope = query_scope(state, args).map_err(scope_failure)?;
    let selection = resolve_identity(args, state, "forward_edges", name, scope.as_ref())?;
    let resolved_identity = serde_json::to_value(&selection.identity).unwrap_or_default();
    let domain_owned = selection.identity.domain;
    let et_owned = selection.identity.entity_type;
    let name_owned = selection.identity.name;
    let query = {
        let scope = scope.clone();
        move |idx: &crate::workspace::index::WorkspaceIndex| {
            let r = match scope.as_ref() {
                Some(scope) => idx.forward_edges_by_identity_in_scope(
                    &domain_owned,
                    &et_owned,
                    &name_owned,
                    scope,
                ),
                None => idx.forward_edges_by_identity(&domain_owned, &et_owned, &name_owned),
            };
            let c = r.len();
            (serde_json::to_value(&r).unwrap_or_default(), c)
        }
    };
    let (results, count, hydration) = match selection.hydration {
        Some(hydration) => {
            let (results, count) = {
                let index = state.workspace_index_read();
                query(&index)
            };
            (results, count, hydration)
        }
        None => run_query_with_hydration(state, "forward_edges", name, workspace_root, query)
            .map_err(QueryFailure::internal)?,
    };
    // The semantic answer is `edges` + `count`; discovery diagnostics are
    // attached only when discovery deviated from its expected path.
    let mut structured = serde_json::json!({
        "edges": results,
        "count": count,
        "resolved_identity": resolved_identity,
    });
    if let Some(discovery) = discovery_field(&hydration) {
        structured["discovery"] = discovery;
    }
    Ok(QueryAnswer::new("forward_edges", structured))
}

/// `reverse_edges`: incoming semantic edges to an entity.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn evaluate_reverse_edges(args: &Value, state: &McpState) -> QueryResult {
    let name = required_str(args, "name").ok_or_else(|| {
        QueryFailure::invalid("Missing required argument: 'name' for reverse_edges query.")
    })?;
    let workspace_root = args["workspaceRoot"].as_str();
    // Workspace scope: the primary defect this closes — `reverse_edges` used to
    // answer with every occurrence of the identity across the WHOLE session,
    // including real call facts authored by an unrelated indexed repository.
    // Occurrence provenance (`asserting_file`) now constrains the answer, and an
    // optional `withinPath` narrows it further to one provenance subtree.
    let scope = query_scope(state, args).map_err(scope_failure)?;
    let selection = resolve_identity(args, state, "reverse_edges", name, scope.as_ref())?;
    let resolved_identity = serde_json::to_value(&selection.identity).unwrap_or_default();
    let domain = selection.identity.domain;
    let entity_type = selection.identity.entity_type;
    let resolved_name = selection.identity.name;
    let query = {
        let scope = scope.clone();
        move |index: &crate::workspace::index::WorkspaceIndex| {
            let edges = match scope.as_ref() {
                Some(scope) => index.reverse_edges_by_identity_in_scope(
                    &domain,
                    &entity_type,
                    &resolved_name,
                    scope,
                ),
                None => index.reverse_edges_by_identity(&domain, &entity_type, &resolved_name),
            };
            let count = edges.len();
            (serde_json::to_value(&edges).unwrap_or_default(), count)
        }
    };
    let (results, count, hydration) = match selection.hydration {
        Some(hydration) => {
            let (results, count) = {
                let index = state.workspace_index_read();
                query(&index)
            };
            (results, count, hydration)
        }
        None => run_query_with_hydration(state, "reverse_edges", name, workspace_root, query)
            .map_err(QueryFailure::internal)?,
    };
    // The semantic answer is `edges` + `count`; discovery diagnostics are
    // attached only when discovery deviated from its expected path.
    let mut structured = serde_json::json!({
        "edges": results,
        "count": count,
        "resolved_identity": resolved_identity,
    });
    if let Some(discovery) = discovery_field(&hydration) {
        structured["discovery"] = discovery;
    }
    Ok(QueryAnswer::new("reverse_edges", structured))
}
