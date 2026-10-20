//! One dependency boundary for ancestry-consuming workspace queries.
use super::outcome::QueryFailure;
use super::prepare::{PreparationContext, PreparedQuery};
use super::request::WorkspaceQueryOperation;
use crate::mcp::McpState;
use crate::mcp::tool_handlers::hydration::DependencyEvidence;
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn prepare(
    operation: WorkspaceQueryOperation,
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
    initial: PreparedQuery,
) -> PreparedQuery {
    if matches!(&initial, PreparedQuery::Ready(Err(_)))
        || matches!(
            operation,
            WorkspaceQueryOperation::CallsInFile | WorkspaceQueryOperation::DeclarationsInFile
        )
        || (operation == WorkspaceQueryOperation::FindEntities && args["declarations_only"] == true)
        || (matches!(
            operation,
            WorkspaceQueryOperation::ForwardEdges | WorkspaceQueryOperation::ReverseEdges
        ) && args["domain"]
            .as_str()
            .is_some_and(|domain| domain != "dotnet")
            && !(args["domain"] == "builtin" && args["entity_type"] == "Class"))
    {
        return initial;
    }
    let generation = state.csharp_authority_generation();
    let published = state
        .workspace_index_read()
        .has_csharp_publication_generation(generation);
    match acquire(args, state, context) {
        Ok(evidence) if evidence.is_empty() => initial,
        Ok(evidence) => {
            let changed = generation != state.csharp_authority_generation();
            if changed {
                context.invalidate_derived_catalogs();
            } else if !published {
                context.invalidate_csharp_projection_catalogs();
            }
            let prepared = if changed || !published {
                operation.prepare_base(args, state, context)
            } else {
                initial
            };
            let stamp = evidence[0].publication_stamp();
            PreparedQuery::validated(stamp, move |index| {
                let mut answer = prepared.evaluate_with_index(index)?;
                for proof in evidence {
                    proof.apply(index, &mut answer.structured["coverage"]);
                }
                Ok(answer)
            })
        }
        Err(error) => PreparedQuery::failure(error),
    }
}

fn acquire(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> Result<Vec<DependencyEvidence>, QueryFailure> {
    let scope = context.scope(state, args)?;
    let mut owners = BTreeSet::new();
    if let Some(file) = args["file_path"].as_str() {
        let resolved = crate::mcp::tool_helpers::resolve_file_path_checked(
            file,
            args["workspaceRoot"].as_str(),
            &state.config.additional_roots,
        )
        .map_err(QueryFailure::invalid)?;
        let owner = state.semantic_owner_path(&resolved);
        if scope.as_ref().is_none_or(|scope| scope.admits(&owner)) {
            owners.insert(owner);
        }
    } else {
        let index = state.workspace_index_read();
        if let Some(name) = args["name"].as_str() {
            let entities = match scope.as_ref() {
                Some(scope) => index.find_entities_by_name_in_scope(name, scope),
                None => index.find_entities_by_name(name),
            };
            owners.extend(
                entities
                    .into_iter()
                    .filter_map(|entity| entity.file.clone()),
            );
        } else if args["type"] == "has_cycle" {
            owners.extend(
                index
                    .file_map()
                    .keys()
                    .filter(|file| scope.as_ref().is_none_or(|scope| scope.admits(file)))
                    .cloned(),
            );
        }
    }
    let owners: Vec<_> = owners
        .into_iter()
        .filter(|file| file.to_ascii_lowercase().ends_with(".cs"))
        .collect();
    loop {
        let mut evidence = Vec::new();
        for owner in &owners {
            evidence.push(
                state
                    .prepare_csharp_semantic_owner(owner, args["workspaceRoot"].as_str())
                    .map_err(QueryFailure::internal)?,
            );
        }
        let index = state.workspace_index_read();
        if evidence
            .iter()
            .all(|proof| proof.publication_is_current(&index))
        {
            return Ok(evidence);
        }
    }
}
