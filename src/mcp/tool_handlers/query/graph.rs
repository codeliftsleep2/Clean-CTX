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
    discovery_field,
    identity::IdentityRequest,
    optional_i32,
    outcome::QueryAnswer,
    outcome::QueryFailure,
    prepare::{PreparationContext, PreparedQuery},
    required_name,
};
use crate::mcp::McpState;
use serde_json::Value;

/// `transitive_dependencies`: BFS dependency traversal.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn prepare_transitive_dependencies(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> PreparedQuery {
    match try_prepare_transitive_dependencies(args, state, context) {
        Ok(prepared) => prepared,
        Err(error) => PreparedQuery::failure(error),
    }
}

fn try_prepare_transitive_dependencies(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> Result<PreparedQuery, QueryFailure> {
    let name = required_name(args, "transitive_dependencies")?;
    let depth = optional_i32(args, "depth", 1);
    let workspace_root = args["workspaceRoot"].as_str();
    // Workspace scope: reachability is computed from THIS workspace's evidence
    // only — filtering the returned list could not achieve that, since the walk
    // itself must not pass through another repository's edges (see
    // `WorkspaceIndex::transitive_dependencies_in_scope`). A `withinPath` narrows
    // it further, so an out-of-path edge cannot extend the walk either.
    let scope = context.scope(state, args)?;
    let identity = IdentityRequest::new(args, name);
    if let Some(exact) = identity.exact() {
        let index = state.workspace_index_read();
        let _ = match scope.as_ref() {
            Some(scope) => index.transitive_dependencies_in_scope(
                &exact.domain,
                &exact.entity_type,
                &exact.name,
                depth,
                scope,
            ),
            None => {
                index.transitive_dependencies(&exact.domain, &exact.entity_type, &exact.name, depth)
            }
        };
    }
    let hydration = context.hydrate(
        state,
        "transitive_dependencies",
        name,
        workspace_root,
        crate::mcp::tool_handlers::hydration::HydrationRequirement::LegacyEdit,
    )?;
    Ok(PreparedQuery::indexed(move |index| {
        let resolved = identity.resolve(index, scope.as_ref())?;
        let resolved_identity = serde_json::to_value(&resolved).unwrap_or_default();
        let dependencies = match scope.as_ref() {
            Some(scope) => index.transitive_dependencies_in_scope(
                &resolved.domain,
                &resolved.entity_type,
                &resolved.name,
                depth,
                scope,
            ),
            None => index.transitive_dependencies(
                &resolved.domain,
                &resolved.entity_type,
                &resolved.name,
                depth,
            ),
        };
        let count = dependencies.len();
        let mut structured = serde_json::json!({
            "dependencies": serde_json::to_value(&dependencies).unwrap_or_default(),
            "count": count,
            "depth_used": depth,
            "resolved_identity": resolved_identity,
        });
        if let Some(discovery) = discovery_field(&hydration) {
            structured["discovery"] = discovery;
        }
        Ok(QueryAnswer::new("transitive_dependencies", structured))
    }))
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
pub(super) fn prepare_has_cycle(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> PreparedQuery {
    match try_prepare_has_cycle(args, state, context) {
        Ok(prepared) => prepared,
        Err(error) => PreparedQuery::failure(error),
    }
}

fn try_prepare_has_cycle(
    args: &Value,
    _state: &McpState,
    context: &mut PreparationContext,
) -> Result<PreparedQuery, QueryFailure> {
    if let Some(kind) = args.get("kind")
        && kind.as_str() != Some("dependency")
    {
        return Err(QueryFailure::invalid(
            "Invalid 'kind' for has_cycle: expected 'dependency'.",
        ));
    }
    // The effective scope is resolved BEFORE the index lock is taken, so an
    // unauthorized `withinPath` is refused without touching the index at all.
    let scope = context.scope(_state, args)?;
    Ok(PreparedQuery::indexed(move |index| {
        let witness = match scope.as_ref() {
            Some(scope) => index.dependency_cycle_witness_in_scope(scope),
            None => index.dependency_cycle_witness(),
        };
        let identity_ambiguities = match (&witness, scope.as_ref()) {
            (Some(witness), Some(scope)) => {
                index.cycle_identity_ambiguities_in_scope(witness, scope)
            }
            (Some(witness), None) => index.cycle_identity_ambiguities(witness),
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
        Ok(QueryAnswer::new("has_cycle", structured))
    }))
}
