//! Boundary between query preparation and final evidence evaluation.

use super::outcome::{QueryAnswer, QueryFailure, QueryResult};
use crate::mcp::McpState;
use crate::workspace::index::WorkspaceIndex;

type IndexedEvaluator = Box<dyn FnOnce(&WorkspaceIndex) -> QueryResult>;

pub(super) enum PreparedQuery {
    Indexed(IndexedEvaluator),
    Ready(QueryResult),
}

impl PreparedQuery {
    pub(super) fn indexed(
        evaluator: impl FnOnce(&WorkspaceIndex) -> QueryResult + 'static,
    ) -> Self {
        Self::Indexed(Box::new(evaluator))
    }

    pub(super) fn answer(answer: QueryAnswer) -> Self {
        Self::Ready(Ok(answer))
    }

    pub(super) fn failure(failure: QueryFailure) -> Self {
        Self::Ready(Err(failure))
    }

    pub(super) fn evaluate_single(self, state: &McpState) -> QueryResult {
        match self {
            Self::Indexed(evaluator) => {
                let index = state.workspace_index_read();
                evaluator(&index)
            }
            Self::Ready(result) => result,
        }
    }
}
