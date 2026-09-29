//! Closed operation identity for the public `workspace_query` surface.

use super::prepare::{PreparationContext, PreparedQuery};
use crate::mcp::McpState;
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorkspaceQueryOperation {
    FindEntities,
    ForwardEdges,
    ReverseEdges,
    EntitiesInFile,
    TransitiveDependencies,
    HasCycle,
    CallsInFile,
}

impl WorkspaceQueryOperation {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            "find_entities" => Some(Self::FindEntities),
            "forward_edges" => Some(Self::ForwardEdges),
            "reverse_edges" => Some(Self::ReverseEdges),
            "entities_in_file" => Some(Self::EntitiesInFile),
            "transitive_dependencies" => Some(Self::TransitiveDependencies),
            "has_cycle" => Some(Self::HasCycle),
            "calls_in_file" => Some(Self::CallsInFile),
            _ => None,
        }
    }

    pub(super) fn prepare(
        self,
        args: &Value,
        state: &McpState,
        context: &mut PreparationContext,
    ) -> PreparedQuery {
        match self {
            Self::FindEntities => super::entities::prepare_find_entities(args, state, context),
            Self::ForwardEdges => super::edges::prepare_forward_edges(args, state, context),
            Self::ReverseEdges => super::edges::prepare_reverse_edges(args, state, context),
            Self::EntitiesInFile => super::entities::prepare_entities_in_file(args, state, context),
            Self::TransitiveDependencies => {
                super::graph::prepare_transitive_dependencies(args, state, context)
            }
            Self::HasCycle => super::graph::prepare_has_cycle(args, state, context),
            Self::CallsInFile => super::calls::prepare_calls_in_file(args, state, context),
        }
    }
}
