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
    let ef_view_revision = context.ef_view_revision;
    let published = state
        .workspace_index_read()
        .has_csharp_publication_generation(generation);
    match acquire(args, state, context) {
        Ok((evidence, None)) if evidence.is_empty() => initial,
        Ok((evidence, ef)) => {
            let changed = generation != state.csharp_authority_generation();
            if changed {
                context.invalidate_derived_catalogs();
            } else if !published {
                context.invalidate_csharp_projection_catalogs();
            }
            let prepared = if changed || !published || context.ef_view_revision != ef_view_revision
            {
                operation.prepare_base(args, state, context)
            } else {
                initial
            };
            let stamp = evidence
                .first()
                .map(DependencyEvidence::publication_stamp)
                .or_else(|| ef.as_ref().map(|proof| proof.publication_stamp()))
                .expect("nonempty dependency evidence owns its publication stamp");
            let ef_validation = ef.clone();
            PreparedQuery::validated(
                stamp,
                move |index| {
                    ef_validation
                        .as_ref()
                        .is_none_or(|proof| proof.current(index))
                },
                move |index| {
                    let mut answer = prepared.evaluate_with_index(index)?;
                    for proof in evidence {
                        proof.apply(index, &mut answer.structured["coverage"]);
                    }
                    if let Some(proof) = ef {
                        proof.apply(index, &mut answer.structured["coverage"]);
                    }
                    Ok(answer)
                },
            )
        }
        Err(error) => PreparedQuery::failure(error),
    }
}

fn acquire(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> Result<
    (
        Vec<DependencyEvidence>,
        Option<super::csharp_resolution::EfDependencyEvidence>,
    ),
    QueryFailure,
> {
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
        .filter(|file| !ef_request(args) || std::path::Path::new(file).is_file())
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
        let ef = if ef_request(args) {
            let key = serde_json::to_string(args)
                .map_err(|error| QueryFailure::internal(error.to_string()))?;
            let proof = super::csharp_resolution::prepare_ef_dependencies(
                state,
                &owners,
                args["workspaceRoot"].as_str(),
                scope.as_ref(),
                args["type"] == "reverse_edges",
                context.ef_dependencies.get(&key),
                &key,
            )
            .map_err(QueryFailure::internal)?;
            context.ef_dependencies.insert(key, proof.clone());
            if context
                .ef_sources
                .as_ref()
                .is_some_and(|sources| sources != proof.sources())
            {
                context.invalidate_derived_catalogs();
            }
            // The first prepared catalog may contain stale source evidence even
            // when no source was published in this attempt (notably deletion).
            context.restrict_ef_sources(proof.sources());
            Some(proof)
        } else {
            None
        };
        let index = state.workspace_index_read();
        if evidence
            .iter()
            .all(|proof| proof.publication_is_current(&index))
            && ef.as_ref().is_none_or(|proof| proof.current(&index))
        {
            return Ok((evidence, ef));
        }
    }
}

fn ef_request(args: &Value) -> bool {
    args["domain"] == "dotnet"
        && ((args["type"] == "forward_edges" && args["entity_type"] == "DbContext")
            || (args["type"] == "reverse_edges" && args["entity_type"] == "Entity"))
}
