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
    prepare::PreparedQuery, query_scope, required_str, scope_failure,
};
use crate::mcp::McpState;
use crate::mcp::tool_handlers::hydration::hydrate_workspace_index;
use serde_json::Value;

/// `forward_edges`: outgoing semantic edges from an entity.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn prepare_forward_edges(args: &Value, state: &McpState) -> PreparedQuery {
    match try_prepare_forward_edges(args, state) {
        Ok(prepared) => prepared,
        Err(error) => PreparedQuery::failure(error),
    }
}

fn try_prepare_forward_edges(
    args: &Value,
    state: &McpState,
) -> Result<PreparedQuery, QueryFailure> {
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
    let hydration = match selection.hydration {
        Some(hydration) => hydration,
        None => {
            let index = state.workspace_index_read();
            let _ = forward_edges(
                &index,
                &domain_owned,
                &et_owned,
                &name_owned,
                scope.as_ref(),
            );
            drop(index);
            hydrate_workspace_index(state, "forward_edges", name, workspace_root)
                .map_err(QueryFailure::internal)?
        }
    };
    Ok(PreparedQuery::indexed(move |index| {
        let edges = forward_edges(index, &domain_owned, &et_owned, &name_owned, scope.as_ref());
        let count = edges.len();
        let mut structured = serde_json::json!({
            "edges": serde_json::to_value(&edges).unwrap_or_default(),
            "count": count,
            "resolved_identity": resolved_identity,
        });
        if let Some(discovery) = discovery_field(&hydration) {
            structured["discovery"] = discovery;
        }
        Ok(QueryAnswer::new("forward_edges", structured))
    }))
}

/// `reverse_edges`: incoming semantic edges to an entity.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn prepare_reverse_edges(args: &Value, state: &McpState) -> PreparedQuery {
    match try_prepare_reverse_edges(args, state) {
        Ok(prepared) => prepared,
        Err(error) => PreparedQuery::failure(error),
    }
}

fn try_prepare_reverse_edges(
    args: &Value,
    state: &McpState,
) -> Result<PreparedQuery, QueryFailure> {
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
    let hydration = match selection.hydration {
        Some(hydration) => hydration,
        None => {
            let index = state.workspace_index_read();
            let _ = reverse_edges(
                &index,
                &domain,
                &entity_type,
                &resolved_name,
                scope.as_ref(),
            );
            drop(index);
            hydrate_workspace_index(state, "reverse_edges", name, workspace_root)
                .map_err(QueryFailure::internal)?
        }
    };
    Ok(PreparedQuery::indexed(move |index| {
        let edges = reverse_edges(index, &domain, &entity_type, &resolved_name, scope.as_ref());
        let count = edges.len();
        let mut structured = serde_json::json!({
            "edges": serde_json::to_value(&edges).unwrap_or_default(),
            "count": count,
            "resolved_identity": resolved_identity,
        });
        if let Some(discovery) = discovery_field(&hydration) {
            structured["discovery"] = discovery;
        }
        Ok(QueryAnswer::new("reverse_edges", structured))
    }))
}

fn forward_edges<'a>(
    index: &'a crate::workspace::index::WorkspaceIndex,
    domain: &str,
    entity_type: &str,
    name: &str,
    scope: Option<&crate::workspace::scope::WorkspaceScope>,
) -> Vec<&'a crate::layers::meta::semantic::SemanticEdge> {
    match scope {
        Some(scope) => index.forward_edges_by_identity_in_scope(domain, entity_type, name, scope),
        None => index.forward_edges_by_identity(domain, entity_type, name),
    }
}

fn reverse_edges<'a>(
    index: &'a crate::workspace::index::WorkspaceIndex,
    domain: &str,
    entity_type: &str,
    name: &str,
    scope: Option<&crate::workspace::scope::WorkspaceScope>,
) -> Vec<&'a crate::layers::meta::semantic::SemanticEdge> {
    match scope {
        Some(scope) => index.reverse_edges_by_identity_in_scope(domain, entity_type, name, scope),
        None => index.reverse_edges_by_identity(domain, entity_type, name),
    }
}
