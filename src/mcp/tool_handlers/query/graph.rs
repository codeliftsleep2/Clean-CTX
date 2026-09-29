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
    discovery_field, identity::resolve_identity, optional_i32, outcome::QueryAnswer,
    outcome::QueryFailure, outcome::QueryResult, query_scope, required_str,
    run_query_with_hydration, scope_failure,
};
use crate::mcp::McpState;
use serde_json::Value;

/// `transitive_dependencies`: BFS dependency traversal.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn evaluate_transitive_dependencies(args: &Value, state: &McpState) -> QueryResult {
    let name = required_str(args, "name").ok_or_else(|| {
        QueryFailure::invalid(
            "Missing required argument: 'name' for transitive_dependencies query.",
        )
    })?;
    let depth = optional_i32(args, "depth", 1);
    let workspace_root = args["workspaceRoot"].as_str();
    // Workspace scope: reachability is computed from THIS workspace's evidence
    // only — filtering the returned list could not achieve that, since the walk
    // itself must not pass through another repository's edges (see
    // `WorkspaceIndex::transitive_dependencies_in_scope`). A `withinPath` narrows
    // it further, so an out-of-path edge cannot extend the walk either.
    let scope = query_scope(state, args).map_err(scope_failure)?;
    let selection = resolve_identity(args, state, "transitive_dependencies", name, scope.as_ref())?;
    let resolved_identity = serde_json::to_value(&selection.identity).unwrap_or_default();
    let domain = selection.identity.domain;
    let entity_type = selection.identity.entity_type;
    let resolved_name = selection.identity.name;
    let query = {
        let scope = scope.clone();
        move |index: &crate::workspace::index::WorkspaceIndex| {
            let dependencies = match scope.as_ref() {
                Some(scope) => index.transitive_dependencies_in_scope(
                    &domain,
                    &entity_type,
                    &resolved_name,
                    depth,
                    scope,
                ),
                None => index.transitive_dependencies(&domain, &entity_type, &resolved_name, depth),
            };
            let count = dependencies.len();
            (
                serde_json::to_value(&dependencies).unwrap_or_default(),
                count,
            )
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
        None => run_query_with_hydration(
            state,
            "transitive_dependencies",
            name,
            workspace_root,
            query,
        )
        .map_err(QueryFailure::internal)?,
    };
    // The semantic answer is `dependencies` + `count` (+ `depth_used`); discovery
    // diagnostics are attached only when discovery deviated from its expected path.
    let mut structured = serde_json::json!({
        "dependencies": results,
        "count": count,
        "depth_used": depth,
        "resolved_identity": resolved_identity,
    });
    if let Some(discovery) = discovery_field(&hydration) {
        structured["discovery"] = discovery;
    }
    Ok(QueryAnswer::new("transitive_dependencies", structured))
}

/// `has_cycle`: detect cycles in the entity graph.
///
/// NOT hydration-eligible: workspace-wide property, no entity identity.
///
/// Workspace scope: when a workspace root is declared, only edge occurrences
/// ASSERTED inside that workspace count as cycle edges, so two repositories that
/// each contribute one half of a cycle can never be combined into a cycle report
/// that neither workspace actually contains. Cycle membership itself is
/// unchanged (only the approved dependency-cycle relation set participates;
/// see `WorkspaceIndex::has_cycle_in_scope`).
pub(super) fn evaluate_has_cycle(args: &Value, state: &McpState) -> QueryResult {
    if let Some(kind) = args.get("kind")
        && kind.as_str() != Some("dependency")
    {
        return Err(QueryFailure::invalid(
            "Invalid 'kind' for has_cycle: expected 'dependency'.",
        ));
    }
    // The effective scope is resolved BEFORE the index lock is taken, so an
    // unauthorized `withinPath` is refused without touching the index at all.
    let scope = query_scope(state, args).map_err(scope_failure)?;
    let idx = state.workspace_index_read();
    let witness = match scope.as_ref() {
        Some(scope) => idx.dependency_cycle_witness_in_scope(scope),
        None => idx.dependency_cycle_witness(),
    };
    let identity_ambiguities = match (&witness, scope.as_ref()) {
        (Some(witness), Some(scope)) => idx.cycle_identity_ambiguities_in_scope(witness, scope),
        (Some(witness), None) => idx.cycle_identity_ambiguities(witness),
        (None, _) => Vec::new(),
    };
    let structured = serde_json::json!({
        "has_cycle": witness.is_some(),
        "cycle": witness.unwrap_or_default(),
        "coverage": {
            "status": "indexed_evidence_only",
            "source_complete": false,
        },
        "identity_model": "semantic_tuple",
        "identity_ambiguous": !identity_ambiguities.is_empty(),
        "identity_ambiguities": identity_ambiguities,
    });
    // NOT hydration-eligible, so the response carries no discovery diagnostics.
    Ok(QueryAnswer::new("has_cycle", structured))
}
