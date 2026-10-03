// Hydration discovery-cache integration.
//
// Discovery is the expensive half of hydration: a CBM project search (plus its
// lazy-reindex gate) or a full filesystem walk-and-read of the root.
// Compilation is deduplicated by the current semantic projections in
// `WorkspaceIndex`, so before this cache every repeated identical query
// rediscovered the same candidates only to discard them as already current.
//
// These helpers are the only place hydration touches the session-scoped
// discovery cache (`crate::mcp::discovery_cache`). They record *completion* of
// discovery — never candidates and never query answers — and they hold the
// cache lock only for the individual lookup/insert, never across CBM calls,
// filesystem scans, source reads, or compilation.

use super::{HydrationRequirement, filesystem::root_key};
use crate::mcp::McpState;
use crate::mcp::discovery_cache::{DiscoveryMode, DiscoveryScope};
use std::path::PathBuf;

/// True when discovery for this scope/mode/name already completed in the
/// scope's current generation.
pub(super) fn discovery_is_complete(
    state: &McpState,
    scope: &DiscoveryScope,
    discovery: DiscoveryMode,
    query_name: &str,
    requirement: HydrationRequirement,
) -> bool {
    let cache = state.hydration_discovery_lock();
    match requirement {
        HydrationRequirement::LegacyEdit => cache.is_complete(scope, discovery, query_name),
        HydrationRequirement::Semantic(fidelity) => {
            cache.is_complete_for(scope, discovery, query_name, Some(fidelity))
        }
    }
}

/// Record a successful discovery for one scope/mode/name.
pub(super) fn mark_discovery_complete(
    state: &McpState,
    scope: &DiscoveryScope,
    discovery: DiscoveryMode,
    query_name: &str,
    requirement: HydrationRequirement,
) {
    let mut cache = state.hydration_discovery_lock();
    match requirement {
        HydrationRequirement::LegacyEdit => cache.mark_complete(scope, discovery, query_name),
        HydrationRequirement::Semantic(fidelity) => {
            cache.mark_complete_for(scope, discovery, query_name, Some(fidelity));
        }
    }
}

/// Split `roots` into the roots whose discovery must still run and the number
/// of roots whose discovery is already complete for this generation.
///
/// Scope identity resolution canonicalizes paths, so it happens outside the
/// cache lock; the lock is only held for the individual lookup.
pub(super) fn pending_discovery_roots(
    state: &McpState,
    discovery: DiscoveryMode,
    query_name: &str,
    roots: Vec<PathBuf>,
    requirement: HydrationRequirement,
) -> (Vec<PathBuf>, usize) {
    let mut pending = Vec::new();
    let mut cached = 0;
    for root in roots {
        let scope = DiscoveryScope::filesystem(root_key(&root));
        if discovery_is_complete(state, &scope, discovery, query_name, requirement) {
            cached += 1;
        } else {
            pending.push(root);
        }
    }
    (pending, cached)
}

/// Produce the cache scopes for a completed filesystem scan. Hydration commits
/// them only after every selected candidate is current or publishes.
pub(super) fn discovery_scopes_for_roots(
    roots: &[PathBuf],
) -> Vec<DiscoveryScope> {
    roots
        .iter()
        .map(|root| DiscoveryScope::filesystem(root_key(root)))
        .collect()
}
