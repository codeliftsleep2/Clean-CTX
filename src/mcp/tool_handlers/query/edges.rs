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
    coverage::{CapabilityDirection, exact_identity_coverage},
    discovery_field,
    identity::IdentityRequest,
    outcome::QueryAnswer,
    outcome::QueryFailure,
    prepare::{PreparationContext, PreparedQuery},
    required_name,
};
use crate::mcp::McpState;
use crate::mcp::tool_handlers::hydration::HydrationRequirement;
use crate::workspace::index::SemanticFidelity;
use serde_json::Value;

/// `forward_edges`: outgoing semantic edges from an entity.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn prepare_forward_edges(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> PreparedQuery {
    match try_prepare_forward_edges(args, state, context) {
        Ok(prepared) => prepared,
        Err(error) => PreparedQuery::failure(error),
    }
}

fn try_prepare_forward_edges(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> Result<PreparedQuery, QueryFailure> {
    let name = required_name(args, "forward_edges")?;
    let workspace_root = args["workspaceRoot"].as_str();
    // Workspace scope: a query issued FOR a workspace answers with the evidence
    // asserted from inside that workspace (primary root + its configured
    // additional roots). `None` when the caller declared no workspace — a
    // root-less query keeps its previous unfiltered behaviour. An unauthorized
    // `withinPath` is refused before the index is consulted.
    let scope = context.scope(state, args)?;
    let identity = IdentityRequest::new(args, name);
    let exact_identity = identity.exact().is_some();
    if let Some(exact) = identity.exact() {
        let index = state.workspace_index_read();
        let _ = forward_edges(
            &index,
            &exact.domain,
            &exact.entity_type,
            &exact.name,
            scope.as_ref(),
        );
    }
    let requirement = edge_hydration_requirement(args);
    let mut hydration =
        context.hydrate(state, "forward_edges", name, workspace_root, requirement)?;
    if hydration.is_complete()
        && let Some(exact) = identity.exact()
        && exact.domain == "builtin"
        && exact.entity_type == "Class"
    {
        let targets = {
            let index = state.workspace_index_read();
            super::classification::neutral_targets_for_forward(&index, &exact, scope.as_ref())
        };
        for target in targets {
            let target_hydration =
                context.hydrate(state, "reverse_edges", &target, workspace_root, requirement)?;
            hydration.merge(target_hydration);
        }
    }
    let classification_complete = hydration.is_complete();
    Ok(PreparedQuery::indexed(move |index| {
        let resolved = identity.resolve(index, scope.as_ref())?;
        let resolved_identity = serde_json::to_value(&resolved).unwrap_or_default();
        let edges = super::classification::forward(
            index,
            &resolved,
            scope.as_ref(),
            classification_complete,
        );
        let count = edges.len();
        let mut structured = serde_json::json!({
            "edges": serde_json::to_value(&edges).unwrap_or_default(),
            "count": count,
            "resolved_identity": resolved_identity,
        });
        if exact_identity {
            structured["coverage"] = exact_identity_coverage(
                index,
                &resolved,
                scope.as_ref(),
                CapabilityDirection::Forward,
                &hydration,
                has_typed_inheritance(&edges),
            );
        }
        if let Some(discovery) = discovery_field(&hydration) {
            structured["discovery"] = discovery;
        }
        Ok(QueryAnswer::new("forward_edges", structured))
    }))
}

/// `reverse_edges`: incoming semantic edges to an entity.
///
/// Eligible for one-cycle hydration: has (domain, entity_type, name) identity.
pub(super) fn prepare_reverse_edges(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> PreparedQuery {
    match try_prepare_reverse_edges(args, state, context) {
        Ok(prepared) => prepared,
        Err(error) => PreparedQuery::failure(error),
    }
}

fn try_prepare_reverse_edges(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> Result<PreparedQuery, QueryFailure> {
    let name = required_name(args, "reverse_edges")?;
    let workspace_root = args["workspaceRoot"].as_str();
    // Workspace scope: the primary defect this closes — `reverse_edges` used to
    // answer with every occurrence of the identity across the WHOLE session,
    // including real call facts authored by an unrelated indexed repository.
    // Occurrence provenance (`asserting_file`) now constrains the answer, and an
    // optional `withinPath` narrows it further to one provenance subtree.
    let scope = context.scope(state, args)?;
    let identity = IdentityRequest::new(args, name);
    let exact_identity = identity.exact().is_some();
    if let Some(exact) = identity.exact() {
        let index = state.workspace_index_read();
        let _ = reverse_edges(
            &index,
            &exact.domain,
            &exact.entity_type,
            &exact.name,
            scope.as_ref(),
        );
    }
    let requirement = edge_hydration_requirement(args);
    let hydration = context.hydrate(state, "reverse_edges", name, workspace_root, requirement)?;
    let classification_complete = hydration.is_complete();
    Ok(PreparedQuery::indexed(move |index| {
        let resolved = identity.resolve(index, scope.as_ref())?;
        let resolved_identity = serde_json::to_value(&resolved).unwrap_or_default();
        let edges = super::classification::reverse(
            index,
            &resolved,
            scope.as_ref(),
            classification_complete,
        );
        let count = edges.len();
        let mut structured = serde_json::json!({
            "edges": serde_json::to_value(&edges).unwrap_or_default(),
            "count": count,
            "resolved_identity": resolved_identity,
        });
        if exact_identity {
            structured["coverage"] = exact_identity_coverage(
                index,
                &resolved,
                scope.as_ref(),
                CapabilityDirection::Reverse,
                &hydration,
                has_typed_inheritance(&edges),
            );
        }
        if let Some(discovery) = discovery_field(&hydration) {
            structured["discovery"] = discovery;
        }
        Ok(QueryAnswer::new("reverse_edges", structured))
    }))
}

/// Angular service edges include constructor `Injects`, whose authoritative
/// producer is High-only. Keep this requirement local to the proven identity;
/// workspace edge operations do not name a relation and therefore cannot
/// support a sound global relation/fidelity matrix here.
fn edge_hydration_requirement(args: &Value) -> HydrationRequirement {
    if args["domain"] == "angular" && args["entity_type"] == "Service" {
        HydrationRequirement::Semantic(SemanticFidelity::High)
    } else {
        HydrationRequirement::LegacyEdit
    }
}

fn has_typed_inheritance(edges: &[crate::layers::meta::semantic::SemanticEdge]) -> bool {
    edges.iter().any(|edge| {
        matches!(
            edge.relation,
            crate::layers::meta::semantic::SemanticRelation::Extends
                | crate::layers::meta::semantic::SemanticRelation::Implements
        )
    })
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
