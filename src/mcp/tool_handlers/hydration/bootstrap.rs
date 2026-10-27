//! Query-bounded conservative nomination; inventory membership grants no facts.
use super::filesystem::{configured_roots, source_inventory};
use super::publication::{CandidatePublication, compile_candidates_for, select_candidates};
use super::{HydrationReport, HydrationRequirement};
use crate::mcp::McpState;
use crate::mcp::acquisition::SourceKind;
use crate::workspace::index::SemanticFidelity;

pub(in crate::mcp::tool_handlers) fn hydrate_bootstrap_owners(
    state: &McpState,
    workspace_root: Option<&str>,
) -> Result<HydrationReport, String> {
    let roots = configured_roots(state, workspace_root);
    let inventory = source_inventory(state, &roots);
    let candidates = inventory
        .entries
        .iter()
        .filter(|entry| entry.source_kind == SourceKind::TypeScript)
        .map(|entry| entry.owner.clone())
        .collect::<Vec<_>>();
    let discovered = candidates.len();
    let requirement = HydrationRequirement::Semantic(SemanticFidelity::High);
    let selected = select_candidates(state, candidates, requirement);
    let mut compiled = 0;
    let mut failed = false;
    for outcome in compile_candidates_for(state, &selected, workspace_root, requirement)? {
        match outcome {
            CandidatePublication::Published | CandidatePublication::PublishedCSharp => {
                compiled += 1
            }
            CandidatePublication::Current => {}
            CandidatePublication::PublicationFailed | CandidatePublication::Failed => failed = true,
        }
    }
    // Completed nomination is not a semantic completeness certificate.
    Ok(HydrationReport {
        hydration_attempted: inventory.attempted,
        discovery_provider: "filesystem",
        discovery_status: if inventory.complete && !failed {
            "completed"
        } else {
            "partial"
        },
        candidates_discovered: discovered,
        candidates_compiled: compiled,
        ..HydrationReport::default()
    })
}
