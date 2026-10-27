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
    identity::{IdentityRequest, ResolvedIdentity},
    outcome::QueryAnswer,
    outcome::QueryFailure,
    prepare::{PreparationContext, PreparedQuery},
    required_name,
};
use crate::mcp::McpState;
use crate::mcp::acquisition::AcquisitionTarget;
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
    #[cfg(feature = "angular")]
    if identity.exact().is_some_and(|exact| {
        exact.domain == "angular"
            && exact.entity_type == "ApplicationBootstrap"
            && exact.name == "bootstrapApplication"
    }) {
        hydration.merge(
            crate::mcp::tool_handlers::hydration::hydrate_bootstrap_owners(state, workspace_root)
                .map_err(QueryFailure::internal)?,
        );
    }
    if hydration.is_complete()
        && let Some(exact) = identity.exact()
        && exact.domain == "dotnet"
        && exact.entity_type == "TestClass"
    {
        let targets = {
            let index = state.workspace_index_read();
            super::dotnet_testing::candidate_targets_for_test(
                state,
                &index,
                scope.as_ref(),
                &exact.name,
            )
        };
        for target in targets {
            let target_hydration =
                context.hydrate(state, "reverse_edges", &target, workspace_root, requirement)?;
            hydration.merge(target_hydration);
        }
    }
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
            let hydration_name =
                super::csharp_resolution::terminal_lookup_name(&target).unwrap_or(target);
            let target_hydration = context.hydrate(
                state,
                "reverse_edges",
                &hydration_name,
                workspace_root,
                requirement,
            )?;
            hydration.merge(target_hydration);
        }
    }
    hydrate_angular_template_owner(
        state,
        context,
        &identity,
        workspace_root,
        requirement,
        &mut hydration,
    )?;
    hydrate_angular_selector_candidates(
        state,
        context,
        &identity,
        scope.as_ref(),
        workspace_root,
        requirement,
        &mut hydration,
    )?;
    #[cfg(feature = "csharp")]
    context.prepare_exact_csharp_calls_for_edges(state, args)?;
    let classification_complete = hydration.is_complete();
    let needs_csharp_resolution = identity
        .exact()
        .is_some_and(|exact| exact.domain == "builtin" && exact.entity_type == "Class");
    let resolution_catalog = (classification_complete && needs_csharp_resolution).then(|| {
        let index = state.workspace_index_read();
        context.csharp_catalog(state, &index, scope.as_ref())
    });
    let dotnet_testing_catalog = (classification_complete
        && identity
            .exact()
            .is_some_and(|exact| exact.domain == "dotnet" && exact.entity_type == "TestClass"))
    .then(|| {
        let index = state.workspace_index_read();
        let csharp = context.csharp_catalog(state, &index, scope.as_ref());
        super::dotnet_testing::Catalog::build(state, &index, scope.as_ref(), &csharp)
    });
    let efcore_catalog = identity
        .exact()
        .is_some_and(|exact| exact.domain == "dotnet" && exact.entity_type == "DbContext")
        .then(|| {
            let index = state.workspace_index_read();
            context.efcore_catalog(state, &index, scope.as_ref())
        });
    let angular_catalog = needs_angular_template_resolution(&identity).then(|| {
        let index = state.workspace_index_read();
        context.angular_catalog(state, &index, scope.as_ref(), classification_complete)
    });
    Ok(PreparedQuery::indexed(move |index| {
        let resolved = identity.resolve(index, scope.as_ref())?;
        let resolved_identity = serde_json::to_value(&resolved).unwrap_or_default();
        let stored_edges = super::classification::forward(
            index,
            &resolved,
            scope.as_ref(),
            classification_complete,
            resolution_catalog.as_deref(),
        );
        let mut edges = serde_json::to_value(&stored_edges)
            .ok()
            .and_then(|value| value.as_array().cloned())
            .unwrap_or_default();
        let stored_count = edges.len();
        if let (Some(exact), Some(catalog)) = (identity.exact(), angular_catalog.as_ref()) {
            edges.extend(catalog.forward(&exact.domain, &exact.entity_type, &exact.name));
        }
        if let (Some(exact), Some(catalog)) = (identity.exact(), dotnet_testing_catalog.as_ref()) {
            edges.extend(
                serde_json::to_value(catalog.forward(
                    &exact.domain,
                    &exact.entity_type,
                    &exact.name,
                ))
                .ok()
                .and_then(|value| value.as_array().cloned())
                .unwrap_or_default(),
            );
        }
        if let (Some(exact), Some(catalog)) = (identity.exact(), efcore_catalog.as_ref()) {
            super::edge_merge::append_distinct_json_edges(
                &mut edges,
                catalog.forward(&exact.domain, &exact.entity_type, &exact.name),
            );
        }
        let count = edges.len();
        let mut structured = serde_json::json!({
            "edges": edges,
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
                has_typed_inheritance(&stored_edges) || count > stored_count,
            );
        }
        apply_angular_template_metadata(
            &mut structured,
            &resolved,
            angular_catalog.as_deref(),
            classification_complete,
            CapabilityDirection::Forward,
        );
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
    let mut hydration = if identity
        .exact()
        .is_some_and(|exact| exact.domain == "builtin" && exact.entity_type == "TypeRef")
    {
        context.hydrate_incoming_type_ref(
            state,
            name,
            workspace_root,
            requirement,
            AcquisitionTarget::exact_type_ref(name),
        )?
    } else {
        context.hydrate(state, "reverse_edges", name, workspace_root, requirement)?
    };
    hydrate_angular_template_owner(
        state,
        context,
        &identity,
        workspace_root,
        requirement,
        &mut hydration,
    )?;
    hydrate_angular_selector_candidates(
        state,
        context,
        &identity,
        scope.as_ref(),
        workspace_root,
        requirement,
        &mut hydration,
    )?;
    #[cfg(feature = "csharp")]
    context.prepare_exact_csharp_calls_for_edges(state, args)?;
    let classification_complete = hydration.is_complete();
    let needs_csharp_resolution = identity.exact().is_some_and(|exact| {
        exact.domain == "builtin" && matches!(exact.entity_type.as_str(), "Class" | "Interface")
    });
    let resolution_catalog = (classification_complete && needs_csharp_resolution).then(|| {
        let index = state.workspace_index_read();
        context.csharp_catalog(state, &index, scope.as_ref())
    });
    let dotnet_testing_catalog = identity
        .exact()
        .is_some_and(|exact| exact.domain == "builtin" && exact.entity_type == "Class")
        .then(|| {
            let index = state.workspace_index_read();
            let csharp = context.csharp_catalog(state, &index, scope.as_ref());
            super::dotnet_testing::Catalog::build(state, &index, scope.as_ref(), &csharp)
        });
    let efcore_catalog = identity
        .exact()
        .is_some_and(|exact| exact.domain == "dotnet" && exact.entity_type == "Entity")
        .then(|| {
            let index = state.workspace_index_read();
            context.efcore_catalog(state, &index, scope.as_ref())
        });
    let angular_catalog = needs_angular_template_resolution(&identity).then(|| {
        let index = state.workspace_index_read();
        context.angular_catalog(state, &index, scope.as_ref(), classification_complete)
    });
    Ok(PreparedQuery::indexed(move |index| {
        let resolved = identity.resolve(index, scope.as_ref())?;
        let resolved_identity = serde_json::to_value(&resolved).unwrap_or_default();
        let stored_edges = super::classification::reverse(
            index,
            &resolved,
            scope.as_ref(),
            classification_complete,
            resolution_catalog.as_deref(),
        );
        let mut edges = serde_json::to_value(&stored_edges)
            .ok()
            .and_then(|value| value.as_array().cloned())
            .unwrap_or_default();
        let stored_count = edges.len();
        if let (Some(exact), Some(catalog)) = (identity.exact(), angular_catalog.as_ref()) {
            edges.extend(catalog.reverse(&exact.domain, &exact.entity_type, &exact.name));
        }
        if let (Some(exact), Some(catalog)) = (identity.exact(), dotnet_testing_catalog.as_ref()) {
            edges.extend(
                serde_json::to_value(catalog.reverse(
                    &exact.domain,
                    &exact.entity_type,
                    &exact.name,
                ))
                .ok()
                .and_then(|value| value.as_array().cloned())
                .unwrap_or_default(),
            );
        }
        if let (Some(exact), Some(catalog)) = (identity.exact(), efcore_catalog.as_ref()) {
            super::edge_merge::append_distinct_json_edges(
                &mut edges,
                catalog.reverse(&exact.domain, &exact.entity_type, &exact.name),
            );
        }
        let count = edges.len();
        let mut structured = serde_json::json!({
            "edges": edges,
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
                has_typed_inheritance(&stored_edges) || count > stored_count,
            );
        }
        apply_angular_template_metadata(
            &mut structured,
            &resolved,
            angular_catalog.as_deref(),
            classification_complete,
            CapabilityDirection::Reverse,
        );
        if let Some(discovery) = discovery_field(&hydration) {
            structured["discovery"] = discovery;
        }
        Ok(QueryAnswer::new("reverse_edges", structured))
    }))
}

fn apply_angular_template_metadata(
    structured: &mut Value,
    identity: &ResolvedIdentity,
    catalog: Option<&super::angular_templates::Catalog>,
    discovery_complete: bool,
    direction: CapabilityDirection,
) {
    let Some(catalog) = catalog else {
        return;
    };
    let diagnostics = catalog.diagnostics(&identity.domain, &identity.entity_type, &identity.name);
    if !diagnostics.is_empty() {
        structured["template_diagnostics"] = Value::Array(diagnostics.clone());
        if let Some(coverage) = structured.get_mut("coverage") {
            coverage["reasons"] = Value::Array(diagnostics);
        }
    }
    let completeness_supported = identity.entity_type == "Template"
        || (identity.entity_type == "Component"
            && matches!(direction, CapabilityDirection::Forward));
    if !completeness_supported {
        return;
    }
    let source_complete = catalog.source_complete(
        &identity.domain,
        &identity.entity_type,
        &identity.name,
        discovery_complete,
    );
    if let Some(coverage) = structured.get_mut("coverage") {
        coverage["source_complete"] = source_complete.into();
        coverage["omitted_possible"] = (!source_complete).into();
        if source_complete {
            coverage["status"] = "authoritative_complete".into();
            coverage["result_semantics"] = "exact".into();
            coverage["reasons"] = Value::Array(Vec::new());
        }
    }
}

fn needs_angular_template_resolution(identity: &IdentityRequest) -> bool {
    identity.exact().is_some_and(|exact| {
        exact.domain == "angular"
            && matches!(
                exact.entity_type.as_str(),
                "Component" | "Template" | "SelectorRef"
            )
    })
}

fn hydrate_angular_template_owner(
    state: &McpState,
    context: &mut PreparationContext,
    identity: &IdentityRequest,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
    hydration: &mut crate::mcp::tool_handlers::hydration::HydrationReport,
) -> Result<(), QueryFailure> {
    let Some(exact) = identity.exact() else {
        return Ok(());
    };
    if exact.domain != "angular" || exact.entity_type != "Template" {
        return Ok(());
    }
    let lookup = if let Some(inline) = exact.name.strip_prefix("inline:") {
        inline
            .rsplit_once("#component=")
            .map(|(_, component)| component.to_string())
    } else {
        std::path::Path::new(&exact.name)
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
    };
    if let Some(lookup) = lookup {
        let owner_hydration =
            context.hydrate(state, "forward_edges", &lookup, workspace_root, requirement)?;
        hydration.merge(owner_hydration);
    }
    Ok(())
}

fn hydrate_angular_selector_candidates(
    state: &McpState,
    context: &mut PreparationContext,
    identity: &IdentityRequest,
    scope: Option<&crate::workspace::scope::WorkspaceScope>,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
    hydration: &mut crate::mcp::tool_handlers::hydration::HydrationReport,
) -> Result<(), QueryFailure> {
    if !hydration.is_complete() || !needs_angular_template_resolution(identity) {
        return Ok(());
    }
    let selectors = {
        let index = state.workspace_index_read();
        context
            .angular_catalog(state, &index, scope, false)
            .selectors()
    };
    for selector in selectors {
        let selector_hydration = context.hydrate(
            state,
            "reverse_edges",
            &selector,
            workspace_root,
            requirement,
        )?;
        hydration.merge(selector_hydration);
    }
    Ok(())
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
