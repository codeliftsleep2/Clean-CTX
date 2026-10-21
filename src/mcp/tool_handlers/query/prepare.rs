//! Boundary between query preparation and final evidence evaluation.

use super::outcome::{QueryAnswer, QueryFailure, QueryResult};
use crate::mcp::McpState;
use crate::mcp::acquisition::AcquisitionTarget;
use crate::mcp::tool_handlers::hydration::{
    HydrationAuthority, HydrationReport, HydrationRequirement, hydrate_workspace_index_for,
    hydrate_workspace_index_for_acquisition,
};
use crate::workspace::index::WorkspaceIndex;
use crate::workspace::scope::WorkspaceScope;
use std::collections::HashMap;
use std::sync::Arc;

type IndexedEvaluator = Box<dyn FnOnce(&WorkspaceIndex) -> QueryResult>;
#[cfg(feature = "csharp")]
type PublicationValidator = Box<dyn Fn(&WorkspaceIndex) -> bool>;

pub(super) enum PreparedQuery {
    Indexed(IndexedEvaluator),
    #[cfg(feature = "csharp")]
    Validated(
        IndexedEvaluator,
        crate::mcp::semantic_compilation::dependencies::PublicationStamp,
        PublicationValidator,
    ),
    Ready(QueryResult),
}

impl PreparedQuery {
    pub(super) fn is_indexed(&self) -> bool {
        !matches!(self, Self::Ready(_))
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
    #[cfg(feature = "csharp")]
    pub(super) fn validated(
        stamp: crate::mcp::semantic_compilation::dependencies::PublicationStamp,
        validator: impl Fn(&WorkspaceIndex) -> bool + 'static,
        evaluator: impl FnOnce(&WorkspaceIndex) -> QueryResult + 'static,
    ) -> Self {
        Self::Validated(Box::new(evaluator), stamp, Box::new(validator))
    }

    pub(super) fn publication_is_current(&self, index: &WorkspaceIndex) -> bool {
        #[cfg(feature = "csharp")]
        if let Self::Validated(_, stamp, validator) = self {
            return stamp.is_current(index) && validator(index);
        }
        let _ = index;
        true
    }

    pub(super) fn evaluate_single(self, state: &McpState) -> QueryResult {
        match self {
            Self::Indexed(evaluator) => {
                let index = state.workspace_index_read();
                evaluator(&index)
            }
            Self::Ready(result) => result,
            #[cfg(feature = "csharp")]
            Self::Validated(evaluator, stamp, validator) => {
                let index = state.workspace_index_read();
                if !stamp.is_current(&index) || !validator(&index) {
                    return Err(QueryFailure::internal(
                        "Semantic publication changed; retry",
                    ));
                }
                evaluator(&index)
            }
        }
    }

    pub(super) fn evaluate_with_index(self, index: &WorkspaceIndex) -> QueryResult {
        match self {
            Self::Indexed(evaluator) => evaluator(index),
            #[cfg(feature = "csharp")]
            Self::Validated(evaluator, stamp, validator) => {
                if !stamp.is_current(index) || !validator(index) {
                    return Err(QueryFailure::internal(
                        "Semantic publication changed; retry",
                    ));
                }
                evaluator(index)
            }
            Self::Ready(result) => result,
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct HydrationKey {
    inbound: bool,
    name: String,
    requirement: HydrationRequirement,
    authority: HydrationAuthority,
}

/// Shared, request-local preparation state. It is deliberately neither stored
/// in `McpState` nor synchronized: one coordinator owns it for one request.
pub(super) struct PreparationContext {
    #[cfg(feature = "csharp")]
    pub(super) ef_dependencies: HashMap<String, super::csharp_resolution::EfDependencyEvidence>,
    pub(super) ef_sources: Option<std::collections::BTreeSet<String>>,
    #[cfg(feature = "csharp")]
    pub(super) ef_view_revision: usize,
    scope: Option<Result<Option<WorkspaceScope>, String>>,
    hydration: HashMap<HydrationKey, Result<HydrationReport, String>>,
    csharp_catalog: Option<Arc<super::csharp_resolution::Catalog>>,
    #[cfg(feature = "csharp")]
    csharp_catalog_generation: Option<u64>,
    efcore_catalog: Option<Arc<super::csharp_resolution::EfCoreCatalog>>,
    angular_catalogs: [Option<Arc<super::angular_templates::Catalog>>; 2],
    #[cfg(feature = "csharp")]
    exact_csharp_calls: Option<(u64, Result<(), String>)>,
}

impl PreparationContext {
    #[cfg(feature = "csharp")]
    pub(super) fn restrict_ef_sources(&mut self, sources: &std::collections::BTreeSet<String>) {
        if self
            .csharp_catalog
            .as_ref()
            .is_some_and(|catalog| catalog.has_stale_ef_sources(sources))
        {
            self.efcore_catalog = None;
            self.ef_view_revision = self.ef_view_revision.wrapping_add(1);
        }
        self.ef_sources = Some(sources.clone());
    }
    pub(super) fn new() -> Self {
        Self {
            #[cfg(feature = "csharp")]
            ef_dependencies: HashMap::new(),
            ef_sources: None,
            #[cfg(feature = "csharp")]
            ef_view_revision: 0,
            scope: None,
            hydration: HashMap::new(),
            csharp_catalog: None,
            #[cfg(feature = "csharp")]
            csharp_catalog_generation: None,
            efcore_catalog: None,
            angular_catalogs: [None, None],
            #[cfg(feature = "csharp")]
            exact_csharp_calls: None,
        }
    }

    #[cfg(feature = "csharp")]
    pub(super) fn prepare_exact_csharp_calls(
        &mut self,
        state: &McpState,
    ) -> Result<(), QueryFailure> {
        let generation = state.csharp_authority_generation();
        if let Some((prepared_generation, result)) = &self.exact_csharp_calls
            && *prepared_generation == generation
        {
            return result.clone().map_err(QueryFailure::internal);
        }
        let result = state.prepare_exact_csharp_calls().map(|_| ());
        let prepared_generation = state.csharp_authority_generation();
        self.exact_csharp_calls = Some((prepared_generation, result.clone()));
        result.map_err(QueryFailure::internal)
    }

    #[cfg(feature = "csharp")]
    pub(super) fn prepare_exact_csharp_calls_for_edges(
        &mut self,
        state: &McpState,
        args: &serde_json::Value,
    ) -> Result<(), QueryFailure> {
        // Ancestry consumers reconcile through the dependency acquisition
        // boundary after publishing their complete frontier.
        if args["domain"] == "builtin" && args["entity_type"] == "Method" {
            self.prepare_exact_csharp_calls(state)?;
        }
        Ok(())
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
        self.hydrate_keyed(
            state,
            query_type,
            name,
            workspace_root,
            requirement,
            HydrationAuthority::CleanAuthoritative,
        )
    }

    pub(super) fn hydrate_incoming_type_ref(
        &mut self,
        state: &McpState,
        name: &str,
        workspace_root: Option<&str>,
        requirement: HydrationRequirement,
        target: AcquisitionTarget,
    ) -> Result<HydrationReport, QueryFailure> {
        self.hydrate_keyed(
            state,
            "reverse_edges",
            name,
            workspace_root,
            requirement,
            HydrationAuthority::ProofBacked(target),
        )
    }

    pub(super) fn hydrate_declarations(
        &mut self,
        state: &McpState,
        name: &str,
        workspace_root: Option<&str>,
    ) -> Result<HydrationReport, QueryFailure> {
        self.hydrate_keyed(
            state,
            "find_entities",
            name,
            workspace_root,
            HydrationRequirement::Semantic(crate::workspace::index::SemanticFidelity::High),
            HydrationAuthority::ProofBacked(AcquisitionTarget::declarations(name)),
        )
    }

    fn hydrate_keyed(
        &mut self,
        state: &McpState,
        query_type: &str,
        name: &str,
        workspace_root: Option<&str>,
        requirement: HydrationRequirement,
        authority: HydrationAuthority,
    ) -> Result<HydrationReport, QueryFailure> {
        let key = HydrationKey {
            inbound: query_type == "reverse_edges",
            name: name.to_string(),
            requirement,
            authority: authority.clone(),
        };
        if let Some(result) = self.hydration.get(&key) {
            return result.clone().map_err(QueryFailure::internal);
        }
        let result = match authority {
            HydrationAuthority::ProofBacked(target) => hydrate_workspace_index_for_acquisition(
                state,
                name,
                workspace_root,
                requirement,
                target,
            ),
            HydrationAuthority::CleanAuthoritative => {
                hydrate_workspace_index_for(state, query_type, name, workspace_root, requirement)
            }
        };
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
        #[cfg(feature = "csharp")]
        {
            let generation = state.csharp_authority_generation();
            self.csharp_catalog_generation = index
                .has_csharp_publication_generation(generation)
                .then_some(generation);
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

    pub(super) fn efcore_catalog(
        &mut self,
        state: &McpState,
        index: &WorkspaceIndex,
        scope: Option<&WorkspaceScope>,
    ) -> Arc<super::csharp_resolution::EfCoreCatalog> {
        #[cfg(feature = "csharp")]
        if self.csharp_catalog.is_some()
            && self.csharp_catalog_generation != Some(state.csharp_authority_generation())
        {
            // Another preparation path may publish without calling this
            // context's hydrate method. Rebuild EF from the surviving sources.
            self.invalidate_derived_catalogs();
        }
        if let Some(catalog) = &self.efcore_catalog {
            #[cfg(test)]
            super::work_metrics::record_ef_catalog_reuse();
            return catalog.clone();
        }
        let csharp = self.csharp_catalog(state, index, scope);
        self.efcore_catalog
            .get_or_insert_with(|| {
                Arc::new(super::csharp_resolution::EfCoreCatalog::build(
                    &csharp,
                    index,
                    self.ef_sources.as_ref(),
                ))
            })
            .clone()
    }

    pub(super) fn invalidate_derived_catalogs(&mut self) {
        #[cfg(test)]
        let had_catalog = self.csharp_catalog.is_some()
            || self.efcore_catalog.is_some()
            || self.angular_catalogs.iter().any(Option::is_some);
        #[cfg(test)]
        if had_catalog {
            super::work_metrics::record_derived_catalog_invalidation();
        }
        self.csharp_catalog = None;
        #[cfg(feature = "csharp")]
        {
            self.csharp_catalog_generation = None;
        }
        self.efcore_catalog = None;
        self.angular_catalogs = [None, None];
    }

    #[cfg(feature = "csharp")]
    pub(super) fn invalidate_csharp_projection_catalogs(&mut self) {
        // Reconciliation without source publication preserves the source-only
        // C# catalog, but the EF consumer also reads projected index edges.
        self.efcore_catalog = None;
    }
}
