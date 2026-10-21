//! Validate the consumed-source footprint of prepared NgModule responses.
use super::prepare::PreparationContext;
use crate::layers::meta::semantic::SemanticDependency;
use crate::mcp::McpState;
use serde_json::Value;

pub(super) fn revalidate(
    state: &McpState,
    owners: &[String],
    root: Option<&str>,
) -> Result<bool, String> {
    let stale: Vec<_> = {
        let index = state.workspace_index_read();
        owners
            .iter()
            .filter(|owner| {
                index
                    .semantic_dependencies_for_file(owner)
                    .iter()
                    .any(|dependency| !dependency.is_current())
            })
            .map(|owner| {
                (
                    owner.clone(),
                    index
                        .semantic_projection_fidelity(owner)
                        .unwrap_or(crate::workspace::index::SemanticFidelity::High),
                )
            })
            .collect()
    };
    for (owner, fidelity) in &stale {
        // Discovery completion is not projection validity. Recompile this
        // already admitted owner without rediscovering its source by name.
        super::super::hydration::revalidate_module_owners(
            state,
            std::slice::from_ref(owner),
            root,
            super::super::hydration::HydrationRequirement::Semantic(*fidelity),
        )?;
    }
    if !dependencies(state, owners)
        .iter()
        .all(|dependency| dependency.is_current())
    {
        return Err("Angular module contributors changed during revalidation; retry".into());
    }
    Ok(!stale.is_empty())
}

pub(super) fn owners(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> Vec<String> {
    if args["type"] != "entities_in_file"
        && !(args["type"] == "forward_edges"
            && args["domain"] == "angular"
            && args["entity_type"] == "Module")
    {
        return Vec::new();
    }
    let Ok(scope) = context.scope(state, args) else {
        return Vec::new();
    };
    let index = state.workspace_index_read();
    if args["type"] == "forward_edges"
        && args["domain"] == "angular"
        && args["entity_type"] == "Module"
    {
        let Some(name) = args["name"].as_str() else {
            return Vec::new();
        };
        return index
            .find_entities_by_name(name)
            .into_iter()
            .filter(|entity| entity.domain == "angular" && entity.entity_type == "Module")
            .filter_map(|entity| entity.file.clone())
            .filter(|owner| scope.as_ref().is_none_or(|scope| scope.admits(owner)))
            .collect();
    }
    if args["type"] == "entities_in_file" {
        let Some(file) = args["file_path"].as_str() else {
            return Vec::new();
        };
        let Ok(file) = crate::mcp::tool_helpers::resolve_file_path_checked(
            file,
            args["workspaceRoot"].as_str(),
            &state.config.additional_roots,
        ) else {
            return Vec::new();
        };
        let owner = state.semantic_owner_path(&file);
        if scope.as_ref().is_none_or(|scope| scope.admits(&owner))
            && index
                .entities_in_file(&owner)
                .iter()
                .any(|entity| entity.domain == "angular" && entity.entity_type == "Module")
        {
            return vec![owner];
        }
    }
    Vec::new()
}

pub(super) fn dependencies(state: &McpState, owners: &[String]) -> Vec<SemanticDependency> {
    let index = state.workspace_index_read();
    owners
        .iter()
        .flat_map(|owner| index.semantic_dependencies_for_file(owner))
        .collect()
}
