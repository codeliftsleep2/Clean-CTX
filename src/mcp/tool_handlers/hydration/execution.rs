use super::cache::mark_discovery_complete;
use super::filesystem::{configured_roots, root_key};
use super::publication::{CandidatePublication, compile_candidate_for, select_candidates};
use super::{HydrationReport, HydrationRequirement, discover_candidate_paths, discovery_kind};
use crate::mcp::McpState;
use crate::mcp::acquisition::{AcquisitionTarget, SemanticPublicationStatus};

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
        None,
    )
}

pub(in crate::mcp::tool_handlers) fn hydrate_workspace_index_for_acquisition(
    state: &McpState,
    query_name: &str,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
    target: AcquisitionTarget,
) -> Result<HydrationReport, String> {
    hydrate_workspace_index_for_target(
        state,
        "reverse_edges",
        query_name,
        workspace_root,
        requirement,
        Some(target),
    )
}

fn hydrate_workspace_index_for_target(
    state: &McpState,
    query_type: &str,
    query_name: &str,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
    target: Option<AcquisitionTarget>,
) -> Result<HydrationReport, String> {
    let cache_key = target.as_ref().map(|target| {
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
    let discovery = discovery_kind(query_type);
    let outcome =
        discover_candidate_paths(state, discovery, query_name, workspace_root, requirement);
    let provisional = target.and_then(|target| {
        super::acquisition::prepare_clean_type_ref_acquisition(
            target,
            &outcome.inventories,
            &state.config.exclude_patterns,
        )
    });
    let candidate_paths = outcome.candidates;
    let mut project_coverage = outcome.project_coverage;
    let discovered = candidate_paths.len();
    let selected = select_candidates(state, candidate_paths, requirement);
    let mut compiled = 0;
    let mut publication_failed = false;
    let mut csharp_reconciliation_required = false;
    for path in &selected {
        match compile_candidate_for(state, path, workspace_root, requirement)? {
            CandidatePublication::Published => compiled += 1,
            CandidatePublication::PublishedCSharp => {
                compiled += 1;
                csharp_reconciliation_required = true;
            }
            CandidatePublication::Current => {}
            CandidatePublication::Failed => publication_failed = true,
        }
    }
    if csharp_reconciliation_required {
        state.reconcile_csharp_exact_calls()?;
    }
    if !publication_failed {
        for scope in &outcome.completions {
            mark_discovery_complete(state, scope, discovery, query_name, requirement);
        }
    }
    project_coverage.sort_by(|left, right| left.project.cmp(&right.project));
    let acquisition_satisfaction = cached_satisfaction.or_else(|| {
        provisional.map(|provisional| {
            provisional.finish(if publication_failed {
                SemanticPublicationStatus::Failed
            } else {
                SemanticPublicationStatus::Complete
            })
        })
    });
    if let (Some(key), Some(satisfaction)) =
        (cache_key, acquisition_satisfaction.as_ref())
    {
        state
            .acquisition_completion_lock()
            .commit(key, satisfaction.clone());
    }
    Ok(HydrationReport {
        hydration_attempted: outcome.attempted,
        discovery_provider: outcome.provider,
        discovery_status: if publication_failed {
            "partial"
        } else {
            outcome.status
        },
        fallback_reason: outcome.fallback_reason,
        candidates_discovered: discovered,
        candidates_compiled: compiled,
        project_coverage,
        acquisition_satisfaction,
    })
}
