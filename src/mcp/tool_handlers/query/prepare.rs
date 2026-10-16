//! Boundary between query preparation and final evidence evaluation.

use super::outcome::{QueryAnswer, QueryFailure, QueryResult};
use crate::mcp::McpState;
use crate::mcp::tool_handlers::hydration::{
    HydrationReport, HydrationRequirement, hydrate_workspace_index_for,
};
use crate::workspace::index::WorkspaceIndex;
use crate::workspace::scope::WorkspaceScope;
use std::collections::HashMap;
use std::sync::Arc;

type IndexedEvaluator = Box<dyn FnOnce(&WorkspaceIndex) -> QueryResult>;

pub(super) enum PreparedQuery {
    Indexed(IndexedEvaluator),
    Ready(QueryResult),
}

impl PreparedQuery {
    pub(super) fn is_indexed(&self) -> bool {
        matches!(self, Self::Indexed(_))
    }

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
    csharp_catalog: Option<Arc<super::csharp_resolution::Catalog>>,
    angular_catalogs: [Option<Arc<super::angular_templates::Catalog>>; 2],
}

impl PreparationContext {
    pub(super) fn new() -> Self {
        Self {
            scope: None,
            hydration: HashMap::new(),
            csharp_catalog: None,
            angular_catalogs: [None, None],
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
        if let Some(result) = self.hydration.get(&key) {
            return result.clone().map_err(QueryFailure::internal);
        }
        let result =
            hydrate_workspace_index_for(state, query_type, name, workspace_root, requirement);
        // Publication (and its required reconciliation) changes the evidence
        // from which request-local catalogs are derived. An error may follow a
        // partial publication, so it also conservatively invalidates. A
        // successful no-publication acquisition leaves compatible views valid.
        let evidence_changed = match &result {
            Ok(report) => report.candidates_compiled > 0,
            Err(_) => true,
        };
        if evidence_changed {
            self.invalidate_derived_catalogs();
        }
        self.hydration.insert(key, result.clone());
        result.map_err(QueryFailure::internal)
    }

    pub(super) fn csharp_catalog(
        &mut self,
        state: &McpState,
        index: &WorkspaceIndex,
        scope: Option<&WorkspaceScope>,
    ) -> Arc<super::csharp_resolution::Catalog> {
        if let Some(catalog) = &self.csharp_catalog {
            #[cfg(test)]
            super::work_metrics::record_csharp_catalog_reuse();
            return catalog.clone();
        }
        self.csharp_catalog
            .get_or_insert_with(|| {
                Arc::new(super::csharp_resolution::Catalog::build(
                    state, index, scope,
                ))
            })
            .clone()
    }

    pub(super) fn angular_catalog(
        &mut self,
        state: &McpState,
        index: &WorkspaceIndex,
        scope: Option<&WorkspaceScope>,
        discovery_complete: bool,
    ) -> Arc<super::angular_templates::Catalog> {
        let slot = usize::from(discovery_complete);
        self.angular_catalogs[slot]
            .get_or_insert_with(|| {
                Arc::new(super::angular_templates::Catalog::build(
                    state,
                    index,
                    scope,
                    discovery_complete,
                ))
            })
            .clone()
    }

    pub(super) fn invalidate_derived_catalogs(&mut self) {
        #[cfg(test)]
        let had_catalog =
            self.csharp_catalog.is_some() || self.angular_catalogs.iter().any(Option::is_some);
        #[cfg(test)]
        if had_catalog {
            super::work_metrics::record_derived_catalog_invalidation();
        }
        self.csharp_catalog = None;
        self.angular_catalogs = [None, None];
    }
}
