//! Boundary between query preparation and final evidence evaluation.

use super::outcome::{QueryAnswer, QueryFailure, QueryResult};
use crate::mcp::McpState;
use crate::mcp::tool_handlers::hydration::{
    HydrationReport, HydrationRequirement, hydrate_workspace_index_for,
};
use crate::workspace::index::WorkspaceIndex;
use crate::workspace::scope::WorkspaceScope;
use std::collections::HashMap;

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

    pub(super) fn evaluate_with_index(self, index: &WorkspaceIndex) -> QueryResult {
        match self {
            Self::Indexed(evaluator) => evaluator(index),
            Self::Ready(result) => result,
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct HydrationKey {
    inbound: bool,
    name: String,
    requirement: HydrationRequirement,
}

/// Shared, request-local preparation state. It is deliberately neither stored
/// in `McpState` nor synchronized: one coordinator owns it for one request.
pub(super) struct PreparationContext {
    scope: Option<Result<Option<WorkspaceScope>, String>>,
    hydration: HashMap<HydrationKey, Result<HydrationReport, String>>,
}

impl PreparationContext {
    pub(super) fn new() -> Self {
        Self {
            scope: None,
            hydration: HashMap::new(),
        }
    }

    pub(super) fn scope(
        &mut self,
        state: &McpState,
        args: &serde_json::Value,
    ) -> Result<Option<WorkspaceScope>, QueryFailure> {
        self.scope
            .get_or_insert_with(|| super::query_scope(state, args))
            .clone()
            .map_err(super::scope_failure)
    }

    pub(super) fn hydrate(
        &mut self,
        state: &McpState,
        query_type: &str,
        name: &str,
        workspace_root: Option<&str>,
        requirement: HydrationRequirement,
    ) -> Result<HydrationReport, QueryFailure> {
        let key = HydrationKey {
            inbound: query_type == "reverse_edges",
            name: name.to_string(),
            requirement,
        };
        self.hydration
            .entry(key)
            .or_insert_with(|| {
                hydrate_workspace_index_for(state, query_type, name, workspace_root, requirement)
            })
            .clone()
            .map_err(QueryFailure::internal)
    }
}
