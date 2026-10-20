use super::cache::mark_discovery_complete;
use super::filesystem::{configured_roots, root_key};
use super::publication::{CandidatePublication, compile_candidates_for, select_candidates};
use super::{
    HydrationAuthority, HydrationReport, HydrationRequirement, discover_candidate_paths,
    discovery_kind,
};
use crate::mcp::McpState;
use crate::mcp::acquisition::{
    AcquisitionOutcome, AcquisitionTarget, ProofBarrier, SemanticPublicationStatus,
};
use crate::mcp::hydration_flight::HydrationFlightKey;

#[cfg(all(test, feature = "rust"))]
pub(in crate::mcp::tool_handlers) fn hydrate_workspace_index(
    state: &McpState,
    query_type: &str,
    query_name: &str,
    workspace_root: Option<&str>,
) -> Result<HydrationReport, String> {
    hydrate_workspace_index_for(
        state,
        query_type,
        query_name,
        workspace_root,
        HydrationRequirement::LegacyEdit,
    )
}

pub(in crate::mcp::tool_handlers) fn hydrate_workspace_index_for(
    state: &McpState,
    query_type: &str,
    query_name: &str,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
) -> Result<HydrationReport, String> {
    hydrate_workspace_index_for_target(
        state,
        query_type,
        query_name,
        workspace_root,
        requirement,
        HydrationAuthority::CleanAuthoritative,
    )
}

pub(in crate::mcp::tool_handlers) fn hydrate_workspace_index_for_acquisition(
    state: &McpState,
    query_name: &str,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
    target: AcquisitionTarget,
) -> Result<HydrationReport, String> {
    let query_type = match target.capability() {
        crate::mcp::acquisition::ProducerCapability::BuiltinTypeRefIncoming => "reverse_edges",
        crate::mcp::acquisition::ProducerCapability::SourceDeclarations => "find_entities",
        crate::mcp::acquisition::ProducerCapability::SourceDependencyClosure => {
            return Err("Dependency closure requires structured source obligations".into());
        }
    };
    hydrate_workspace_index_for_target(
        state,
        query_type,
        query_name,
        workspace_root,
        requirement,
        HydrationAuthority::ProofBacked(target),
    )
}

fn hydrate_workspace_index_for_target(
    state: &McpState,
    query_type: &str,
    query_name: &str,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
    authority: HydrationAuthority,
) -> Result<HydrationReport, String> {
    let roots = configured_roots(state, workspace_root)
        .iter()
        .map(|root| root_key(root))
        .collect::<Vec<_>>();
    let discovery = discovery_kind(query_type);
    let semantic_fidelity = match requirement {
        HydrationRequirement::LegacyEdit => None,
        HydrationRequirement::Semantic(fidelity) => Some(fidelity),
    };
    let flight_key = HydrationFlightKey::new(
        roots,
        discovery,
        query_name,
        semantic_fidelity,
        authority.proof_target().cloned(),
    );
    state.hydration_flights.run(flight_key, || {
        hydrate_workspace_index_for_target_attempt(
            state,
            query_name,
            workspace_root,
            requirement,
            &authority,
            discovery,
        )
    })
}

fn hydrate_workspace_index_for_target_attempt(
    state: &McpState,
    query_name: &str,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
    authority: &HydrationAuthority,
    discovery: crate::mcp::discovery_cache::DiscoveryMode,
) -> Result<HydrationReport, String> {
    let cache_key = authority.proof_target().map(|target| {
        let roots = configured_roots(state, workspace_root)
            .iter()
            .map(|root| root_key(root))
            .collect();
        super::acquisition::acquisition_cache_key(
            target.clone(),
            roots,
            &state.config.exclude_patterns,
        )
    });
    let cached_satisfaction = cache_key
        .as_ref()
        .and_then(|key| state.acquisition_completion_lock().get(key));
    let mut outcome = discover_candidate_paths(
        state,
        discovery,
        query_name,
        workspace_root,
        requirement,
        authority,
    );
    // Discovery completion and proof completion are distinct authorities. A
    // prior ordinary query may have cached discovery without retaining the
    // inventories required to construct this acquisition proof.
    if authority.proof_target().is_some() && cached_satisfaction.is_none() {
        let roots = configured_roots(state, workspace_root);
        if outcome.inventories.len() != roots.len() {
            let inventory = super::filesystem::scan(state, &roots, query_name);
            outcome.inventories = inventory.inventories;
            outcome.candidates.extend(inventory.candidates);
        }
    }
    let provisional = authority.proof_target().cloned().and_then(|target| {
        super::acquisition::prepare_clean_acquisition(
            target,
            &outcome.inventories,
            &state.config.exclude_patterns,
        )
    });
    let mut candidate_paths = outcome.candidates;
    if let Some(provisional) = &provisional {
        candidate_paths.extend(provisional.owners().iter().cloned());
    }
    candidate_paths.sort();
    candidate_paths.dedup();
    let mut project_coverage = outcome.project_coverage;
    let discovered = candidate_paths.len();
    let selected = select_candidates(state, candidate_paths, requirement);
    let mut compiled = 0;
    let mut publication_failed = false;
    let mut candidate_processing_failed = false;
    for outcome in compile_candidates_for(state, &selected, workspace_root, requirement)? {
        match outcome {
            CandidatePublication::Published => compiled += 1,
            CandidatePublication::PublishedCSharp => compiled += 1,
            CandidatePublication::Current => {}
            CandidatePublication::PublicationFailed => publication_failed = true,
            CandidatePublication::Failed => candidate_processing_failed = true,
        }
    }
    if let Some(provisional) = &provisional {
        publication_failed |= !provisional.publication_is_current(state);
    }
    if !publication_failed && !candidate_processing_failed {
        for scope in outcome
            .provider_completions
            .iter()
            .chain(&outcome.root_completions)
        {
            mark_discovery_complete(state, scope, discovery, query_name, requirement);
        }
    }
    project_coverage.sort_by(|left, right| left.project.cmp(&right.project));
    let cached_outcome = cached_satisfaction
        .clone()
        .and_then(AcquisitionOutcome::from_cached_complete);
    let acquisition_outcome = cached_outcome.or_else(|| {
        provisional.map(|provisional| {
            let failed = publication_failed || candidate_processing_failed;
            provisional.finish(
                if failed {
                    SemanticPublicationStatus::Failed
                } else {
                    SemanticPublicationStatus::Complete
                },
                if candidate_processing_failed {
                    ProofBarrier::CandidateProcessingFailed
                } else {
                    ProofBarrier::PublicationFailed
                },
            )
        })
    });
    let acquisition_satisfaction = cached_satisfaction.or_else(|| {
        acquisition_outcome
            .as_ref()
            .map(|outcome| outcome.satisfaction().clone())
    });
    if let (Some(key), Some(satisfaction)) = (cache_key, acquisition_satisfaction.as_ref()) {
        state
            .acquisition_completion_lock()
            .commit(key, satisfaction.clone());
    }
    Ok(HydrationReport {
        hydration_attempted: outcome.attempted,
        discovery_provider: outcome.provider,
        discovery_status: if publication_failed || candidate_processing_failed {
            "partial"
        } else {
            outcome.status
        },
        fallback_reason: outcome.fallback_reason,
        candidates_discovered: discovered,
        candidates_compiled: compiled,
        project_coverage,
        acquisition_satisfaction,
        acquisition_outcome,
    })
}
