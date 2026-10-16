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

use super::{
    discovery_field,
    outcome::QueryAnswer,
    outcome::QueryFailure,
    prepare::{PreparationContext, PreparedQuery},
    required_name, required_str,
};
use crate::mcp::McpState;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

fn semantic_fidelity(
    args: &Value,
    state: &McpState,
) -> Result<crate::compression::Fidelity, QueryFailure> {
    use crate::compression::Fidelity;

    let requested = match args["fidelity"].as_str() {
        Some(value) => match Fidelity::parse(value) {
            Ok(fidelity) => fidelity,
            Err(error) => return Err(QueryFailure::invalid(error.to_string())),
        },
        None => state.config.default_fidelity,
    };
    Ok(match requested {
        Fidelity::Edit | Fidelity::Verbatim => Fidelity::High,
        fidelity => fidelity,
    })
}

/// `find_entities`: find entities by name (cross-domain/type).
///
/// Eligible for one-cycle hydration: has a name for CBM candidate discovery.
pub(super) fn prepare_find_entities(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> PreparedQuery {
    match try_prepare_find_entities(args, state, context) {
        Ok(prepared) => prepared,
        Err(error) => PreparedQuery::failure(error),
    }
}

fn try_prepare_find_entities(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> Result<PreparedQuery, QueryFailure> {
    let name = required_name(args, "find_entities")?.to_string();
    let declarations_only = args["declarations_only"].as_bool().unwrap_or(false);
    let domain = args["domain"].as_str().map(str::to_owned);
    let entity_type = args["entity_type"].as_str().map(str::to_owned);
    let workspace_root = args["workspaceRoot"].as_str();
    // Workspace scope: a query issued FOR a workspace answers with the entity
    // occurrences that workspace's own files declare (`None` = the caller
    // declared no workspace, so the previous unfiltered behaviour stands).
    // Built once per query, so the initial answer and the post-hydration rerun
    // share one root set, an optional `withinPath` narrowing included, and an
    // unauthorized `withinPath` is refused before the index is consulted.
    let scope = context.scope(state, args)?;
    // Preserve the established initial-read-before-hydration lifecycle.
    {
        let index = state.workspace_index_read();
        let _ = if declarations_only {
            match scope.as_ref() {
                Some(scope) => index.declarations_by_name_in_workspace(&name, scope),
                None => index.declarations_by_name(&name),
            }
        } else {
            match scope.as_ref() {
                Some(scope) => index.find_entities_by_name_in_scope(&name, scope),
                None => index.find_entities_by_name(&name),
            }
        };
    }
    let mut hydration = context.hydrate(
        state,
        "find_entities",
        &name,
        workspace_root,
        crate::mcp::tool_handlers::hydration::HydrationRequirement::LegacyEdit,
    )?;
    if hydration.is_complete() {
        let mut current = name.clone();
        let mut visited = HashSet::new();
        while visited.insert(current.clone()) {
            let candidate = {
                let index = state.workspace_index_read();
                let catalog =
                    super::csharp_resolution::Catalog::build(state, &index, scope.as_ref());
                catalog.aspnet_base_candidate(&current)
            };
            let Some(candidate) = candidate else { break };
            let next = context.hydrate(
                state,
                "find_entities",
                &candidate,
                workspace_root,
                crate::mcp::tool_handlers::hydration::HydrationRequirement::LegacyEdit,
            )?;
            hydration.merge(next);
            if !hydration.is_complete() {
                break;
            }
            current = candidate;
        }
    }
    let (aspnet_controller, aspnet_authorized) = if !declarations_only && hydration.is_complete() {
        let index = state.workspace_index_read();
        let catalog = super::csharp_resolution::Catalog::build(state, &index, scope.as_ref());
        let authorized = catalog.has_aspnet_controller_authority(&name);
        let controller = catalog
            .aspnet_projection(&name, crate::compression::Fidelity::High)
            .map(|projection| {
                crate::layers::meta::semantic::EntityRef::new("dotnet", "Controller", &name)
                    .with_file(projection.file)
            });
        (controller, authorized)
    } else {
        (None, false)
    };
    Ok(PreparedQuery::indexed(move |index| {
        let entities = if declarations_only {
            match scope.as_ref() {
                Some(scope) => index.declarations_by_name_in_workspace(&name, scope),
                None => index.declarations_by_name(&name),
            }
        } else {
            match scope.as_ref() {
                Some(scope) => index.find_entities_by_name_in_scope(&name, scope),
                None => index.find_entities_by_name(&name),
            }
        };
        let entities: Vec<_> = entities
            .into_iter()
            .filter(|entity| {
                !declarations_only || domain.as_deref().is_none_or(|value| entity.domain == value)
            })
            .filter(|entity| {
                !declarations_only
                    || entity_type
                        .as_deref()
                        .is_none_or(|value| entity.entity_type == value)
            })
            .filter(|entity| {
                declarations_only
                    || aspnet_authorized
                    || !(entity.domain == "dotnet"
                        && entity.entity_type == "Controller"
                        && entity.name == name)
            })
            .collect();
        let mut serialized = serde_json::to_value(&entities)
            .ok()
            .and_then(|value| value.as_array().cloned())
            .unwrap_or_default();
        if let Some(controller) = &aspnet_controller
            && !entities.iter().any(|entity| **entity == *controller)
        {
            serialized.push(serde_json::to_value(controller).unwrap_or_default());
        }
        let count = serialized.len();
        let mut structured = serde_json::json!({
            "entities": serialized,
            "count": count,
        });
        if let Some(discovery) = discovery_field(&hydration) {
            structured["discovery"] = discovery;
        }
        Ok(QueryAnswer::new("find_entities", structured))
    }))
}

/// `entities_in_file`: list all entities defined in a given file.
pub(super) fn prepare_entities_in_file(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> PreparedQuery {
    match try_prepare_entities_in_file(args, state, context) {
        Ok(prepared) => prepared,
        Err(error) => PreparedQuery::failure(error),
    }
}

fn try_prepare_entities_in_file(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> Result<PreparedQuery, QueryFailure> {
    let file_path = required_str(args, "file_path").ok_or_else(|| {
        QueryFailure::invalid("Missing required argument: 'file_path' for entities_in_file query.")
    })?;
    let workspace_root = args["workspaceRoot"].as_str();
    // The effective scope: the WSC-004 roots are already enforced by the
    // trusted-path/root validation below (they ARE its accepted set), and an
    // invalid `withinPath` is refused before any path is resolved.
    let scope = context.scope(state, args)?;
    let resolved_path = match crate::mcp::tool_helpers::resolve_file_path_checked(
        file_path,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(message) => return Err(file_resolution_failure(&message)),
    };
    let canonical_path = state.semantic_owner_path(&resolved_path);
    // The optional SECOND layer: when a `withinPath` is present, the explicit file
    // must lie inside it, and a file outside it is answered exactly like a file
    // outside the workspace (the same minimal zero-result shape). With no
    // `withinPath` this check is skipped, so the accepted set is unchanged and no
    // redundant occurrence scan is added.
    if scope
        .as_ref()
        .is_some_and(|scope| scope.has_narrowing() && !scope.admits(&canonical_path))
    {
        return Err(file_scope_failure());
    }
    ensure_supported_file(&resolved_path)?;
    let fidelity = semantic_fidelity(args, state)?;
    let source = state
        .read_source(&resolved_path)
        .map_err(|error| QueryFailure::internal(error.to_string()))?;
    let source_hash = state.cache_read().compute_hash(source.as_bytes());
    let workspace_fidelity = crate::workspace::index::SemanticFidelity::for_request(fidelity);
    if state
        .workspace_index_read()
        .has_current_semantic_projection(&canonical_path, workspace_fidelity, &source_hash)
    {
        return Ok(prepared_entities_in_file(canonical_path));
    }
    state
        .preflight_semantic_publication(&resolved_path)
        .map_err(QueryFailure::internal)?;
    let (semantic_edges, compiled_hash) = match crate::mcp::tool_helpers::compile_file_ir_candidate(
        &resolved_path,
        fidelity,
        state,
    ) {
        Ok((_, edges, _, hash)) => (edges, hash),
        Err(error) => return Err(QueryFailure::internal(error.to_string())),
    };
    {
        let mut idx = state.workspace_index_lock();
        idx.replace_semantic_projection(
            &canonical_path,
            semantic_edges,
            workspace_fidelity,
            compiled_hash,
        );
    }
    #[cfg(feature = "csharp")]
    state
        .reconcile_csharp_after_authority_change(&canonical_path)
        .map_err(QueryFailure::internal)?;
    Ok(prepared_entities_in_file(canonical_path))
}

fn prepared_entities_in_file(canonical_path: String) -> PreparedQuery {
    PreparedQuery::indexed(move |index| {
        let entities = index.entities_in_file(&canonical_path);
        let count = entities.len();
        let structured = serde_json::json!({
            "entities": serde_json::to_value(&entities).unwrap_or_default(),
            "count": count,
        });
        Ok(QueryAnswer::new("entities_in_file", structured))
    })
}

/// `declarations_in_file`: source-proven declarations in one trusted file.
pub(super) fn prepare_declarations_in_file(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> PreparedQuery {
    match try_prepare_declarations_in_file(args, state, context) {
        Ok(prepared) => prepared,
        Err(error) => PreparedQuery::failure(error),
    }
}

fn try_prepare_declarations_in_file(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> Result<PreparedQuery, QueryFailure> {
    let file_path = required_str(args, "file_path").ok_or_else(|| {
        QueryFailure::invalid(
            "Missing required argument: 'file_path' for declarations_in_file query.",
        )
    })?;
    let workspace_root = args["workspaceRoot"].as_str();
    let scope = context.scope(state, args)?;
    let resolved_path = match crate::mcp::tool_helpers::resolve_file_path_checked(
        file_path,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(path) => path,
        Err(message) => return Err(file_resolution_failure(&message)),
    };
    let canonical_path = state.semantic_owner_path(&resolved_path);
    if scope
        .as_ref()
        .is_some_and(|scope| scope.has_narrowing() && !scope.admits(&canonical_path))
    {
        return Err(file_scope_failure());
    }
    ensure_supported_file(&resolved_path)?;
    let fidelity = semantic_fidelity(args, state)?;
    let source = state
        .read_source(&resolved_path)
        .map_err(|error| QueryFailure::internal(error.to_string()))?;
    let source_hash = state.cache_read().compute_hash(source.as_bytes());
    let workspace_fidelity = crate::workspace::index::SemanticFidelity::for_request(fidelity);
    if !state
        .workspace_index_read()
        .has_current_semantic_projection(&canonical_path, workspace_fidelity, &source_hash)
    {
        state
            .preflight_semantic_publication(&resolved_path)
            .map_err(QueryFailure::internal)?;
        let (_, semantic_edges, _, compiled_hash) =
            crate::mcp::tool_helpers::compile_file_ir_candidate(&resolved_path, fidelity, state)
                .map_err(|error| QueryFailure::internal(error.to_string()))?;
        state.workspace_index_lock().replace_semantic_projection(
            &canonical_path,
            semantic_edges,
            workspace_fidelity,
            compiled_hash,
        );
        #[cfg(feature = "csharp")]
        state
            .reconcile_csharp_after_authority_change(&canonical_path)
            .map_err(QueryFailure::internal)?;
    }
    Ok(prepared_declarations_in_file(canonical_path))
}

fn prepared_declarations_in_file(canonical_path: String) -> PreparedQuery {
    PreparedQuery::indexed(move |index| {
        let entities = index.declarations_in_file(&canonical_path);
        let structured = serde_json::json!({
            "count": entities.len(),
            "entities": serde_json::to_value(&entities).unwrap_or_default(),
        });
        Ok(QueryAnswer::new("declarations_in_file", structured))
    })
}

fn file_resolution_failure(message: &str) -> QueryFailure {
    if message.starts_with("path does not exist:") {
        return QueryFailure::new(
            -32004,
            "Requested file was not found in the authorized workspace.",
            Some(serde_json::json!({ "reason": "file_not_found" })),
        );
    }
    QueryFailure::new(
        -32602,
        "Requested file is outside the authorized workspace roots.",
        Some(serde_json::json!({ "reason": "outside_authorized_roots" })),
    )
}

fn file_scope_failure() -> QueryFailure {
    QueryFailure::new(
        -32602,
        "Requested file is excluded by the active withinPath scope.",
        Some(serde_json::json!({ "reason": "excluded_by_within_path" })),
    )
}

fn ensure_supported_file(path: &str) -> Result<(), QueryFailure> {
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    if crate::compression::language::language_for_extension(extension).is_some() {
        return Ok(());
    }
    Err(QueryFailure::new(
        -32602,
        "Requested file type is not supported by workspace_query.",
        Some(serde_json::json!({ "reason": "unsupported_file_type" })),
    ))
}
