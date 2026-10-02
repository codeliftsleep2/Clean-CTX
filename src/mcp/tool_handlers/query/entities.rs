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
        let _ = match scope.as_ref() {
            Some(scope) => index.find_entities_by_name_in_scope(&name, scope),
            None => index.find_entities_by_name(&name),
        };
    }
    let hydration = context.hydrate(
        state,
        "find_entities",
        &name,
        workspace_root,
        crate::mcp::tool_handlers::hydration::HydrationRequirement::LegacyEdit,
    )?;
    Ok(PreparedQuery::indexed(move |index| {
        let entities = match scope.as_ref() {
            Some(scope) => index.find_entities_by_name_in_scope(&name, scope),
            None => index.find_entities_by_name(&name),
        };
        let count = entities.len();
        let mut structured = serde_json::json!({
            "entities": serde_json::to_value(&entities).unwrap_or_default(),
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
        Err(_msg) => {
            // File does not exist or is outside workspace boundary —
            // return empty results (the user asked for a file that
            // hasn't been compiled). This matches the pre-fix behavior
            // where a non-existent path produced no entities.
            return Ok(PreparedQuery::answer(empty_entities_in_file()));
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
        return Ok(PreparedQuery::answer(empty_entities_in_file()));
    }
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
        Ok((_, edges, hash)) => (edges, hash),
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

/// The minimal zero-result shape for an `entities_in_file` request whose file is
/// not answerable inside the effective scope: it does not exist, it lies outside
/// the workspace roots, or it lies outside the `withinPath` narrowing. The shape
/// is exactly the pre-`withinPath` one (no hydration metadata at all), so an
/// out-of-scope path is answered as it always was.
fn empty_entities_in_file() -> QueryAnswer {
    let structured = serde_json::json!({ "entities": [], "count": 0 });
    QueryAnswer::new("entities_in_file", structured)
}
