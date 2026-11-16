//! New-operation-only conservative nomination using the shared source pipeline.
use super::filesystem::{configured_roots, source_inventory};
use super::publication::{CandidatePublication, compile_candidates_for, select_candidates};
use super::{HydrationReport, HydrationRequirement};
use crate::mcp::{McpState, acquisition::SourceKind};
use crate::workspace::index::SemanticFidelity;

#[derive(Clone)]
pub(in crate::mcp::tool_handlers) struct ResolverNomination {
    pub report: HydrationReport,
    pub owners: Vec<String>,
}

pub(in crate::mcp::tool_handlers) fn hydrate_resolver_owners(
    state: &McpState,
    root: &str,
) -> Result<ResolverNomination, String> {
    #[cfg(test)]
    super::super::query::work_metrics::record_resolver_nomination();
    let inventory = source_inventory(state, &configured_roots(state, Some(root)));
    let candidates: Vec<_> = inventory
        .entries
        .iter()
        .filter(|entry| entry.source_kind == SourceKind::TypeScript)
        .map(|entry| entry.owner.clone())
        .collect();
    let discovered = candidates.len();
    let expected: std::collections::HashSet<_> = candidates
        .iter()
        .map(|owner| state.semantic_owner_path(owner))
        .collect();
    let requirement = HydrationRequirement::Semantic(SemanticFidelity::High);
    let owners = select_candidates(state, candidates, requirement);
    if owners
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        != expected
    {
        return Err("Required resolver candidate changed before acquisition; retry".into());
    }
    let mut compiled = 0;
    for outcome in compile_candidates_for(state, &owners, Some(root), requirement)? {
        match outcome {
            CandidatePublication::Published | CandidatePublication::PublishedCSharp => {
                compiled += 1
            }
            CandidatePublication::Current => {}
            CandidatePublication::PublicationFailed | CandidatePublication::Failed => {
                return Err("Required resolver candidate acquisition/publication failed".into());
            }
        }
    }
    Ok(ResolverNomination {
        owners,
        report: HydrationReport {
            hydration_attempted: inventory.attempted,
            discovery_provider: "filesystem",
            discovery_status: if inventory.complete {
                "completed"
            } else {
                "partial"
            },
            candidates_discovered: discovered,
            candidates_compiled: compiled,
            ..HydrationReport::default()
        },
    })
}
