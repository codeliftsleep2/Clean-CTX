// Filesystem-only discovery retains the original completion and fallback policy.
use super::*;

pub(super) fn filesystem_only_discovery(
    state: &McpState,
    discovery: DiscoveryMode,
    roots: Vec<PathBuf>,
    query_name: &str,
    fallback_reason: &'static str,
    requirement: HydrationRequirement,
    manifests_required: bool,
) -> DiscoveryOutcome {
    let (pending_roots, cached_roots) =
        pending_discovery_roots(state, discovery, query_name, roots, requirement);
    if pending_roots.is_empty() {
        if cached_roots == 0 {
            // No root was configured (or every root was already unusable):
            // identical to scanning nothing.
            return DiscoveryOutcome {
                candidates: Vec::new(),
                project_coverage: Vec::new(),
                provider: "none",
                status: "failed",
                attempted: false,
                fallback_reason: Some("filesystem_unavailable"),
                provider_completions: Vec::new(),
                root_completions: Vec::new(),
                inventories: Vec::new(),
            };
        }
        // Nothing left to scan: every configured root's fallback discovery
        // already completed for this generation (and its candidates were
        // compiled then), so the full-repo walk is skipped entirely.
        return DiscoveryOutcome {
            candidates: Vec::new(),
            project_coverage: Vec::new(),
            provider: "filesystem",
            status: "completed",
            attempted: true,
            fallback_reason: Some(fallback_reason),
            provider_completions: Vec::new(),
            root_completions: Vec::new(),
            inventories: Vec::new(),
        };
    }
    let scan = filesystem::scan_with_manifest_demand(
        state,
        &pending_roots,
        query_name,
        manifests_required,
    );
    let root_completions = if scan.completed {
        discovery_scopes_for_roots(&pending_roots)
    } else {
        Vec::new()
    };
    let covered = scan.attempted || cached_roots > 0;
    let provider = if covered { "filesystem" } else { "none" };
    DiscoveryOutcome {
        candidates: scan.candidates,
        project_coverage: Vec::new(),
        provider,
        status: discovery_status(covered, scan.completed, false),
        attempted: covered,
        fallback_reason: Some(if covered {
            fallback_reason
        } else {
            "filesystem_unavailable"
        }),
        provider_completions: Vec::new(),
        root_completions,
        inventories: scan.inventories,
    }
}
