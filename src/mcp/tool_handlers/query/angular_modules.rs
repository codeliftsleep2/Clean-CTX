//! Validate the consumed-source footprint of prepared NgModule responses.
use super::prepare::PreparationContext;
use crate::mcp::McpState;
use crate::workspace::index::{SemanticFidelity, WorkspaceIndex};
use serde_json::Value;

/// Request-local publication identity, validated by the index's existing native
/// witnesses and legacy dependencies. It does not copy or invent source authority.
pub(super) struct ProjectionWitness {
    owner: String,
    fidelity: SemanticFidelity,
    source_hash: String,
}

impl ProjectionWitness {
    pub(super) fn is_current(&self, index: &WorkspaceIndex) -> bool {
        index.has_current_semantic_projection(&self.owner, self.fidelity, &self.source_hash)
    }
}

pub(super) fn revalidate(
    state: &McpState,
    owners: &[String],
    root: Option<&str>,
) -> Result<bool, String> {
    let stale: Vec<_> = {
        let index = state.workspace_index_read();
        witnesses(&index, owners)
            .into_iter()
            .filter(|witness| !witness.is_current(&index))
            .map(|witness| (witness.owner, witness.fidelity))
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
    let index = state.workspace_index_read();
    if !witnesses(&index, owners)
        .iter()
        .all(|witness| witness.is_current(&index))
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

pub(super) fn dependencies(state: &McpState, owners: &[String]) -> Vec<ProjectionWitness> {
    let index = state.workspace_index_read();
    witnesses(&index, owners)
}

fn witnesses(index: &WorkspaceIndex, owners: &[String]) -> Vec<ProjectionWitness> {
    owners
        .iter()
        .map(|owner| ProjectionWitness {
            owner: owner.clone(),
            fidelity: index
                .semantic_projection_fidelity(owner)
                .unwrap_or(SemanticFidelity::High),
            source_hash: index
                .semantic_projection_source_hash(owner)
                .unwrap_or_default()
                .to_owned(),
        })
        .collect()
}
