//! Closed operation identity for the public `workspace_query` surface.

use super::outcome::QueryResult;
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

    pub(super) fn evaluate(self, args: &Value, state: &McpState) -> QueryResult {
        match self {
            Self::FindEntities => super::entities::evaluate_find_entities(args, state),
            Self::ForwardEdges => super::edges::evaluate_forward_edges(args, state),
            Self::ReverseEdges => super::edges::evaluate_reverse_edges(args, state),
            Self::EntitiesInFile => super::entities::evaluate_entities_in_file(args, state),
            Self::TransitiveDependencies => {
                super::graph::evaluate_transitive_dependencies(args, state)
            }
            Self::HasCycle => super::graph::evaluate_has_cycle(args, state),
            Self::CallsInFile => super::calls::evaluate_calls_in_file(args, state),
        }
    }
}
