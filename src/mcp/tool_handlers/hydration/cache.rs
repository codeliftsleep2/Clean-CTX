// Hydration discovery-cache integration.
//
// Discovery is the expensive half of hydration: a CBM project search (plus its
// lazy-reindex gate) or a full filesystem walk-and-read of the root.
// Compilation was already deduplicated by `WorkspaceIndex::file_map()`, so
// before this cache every repeated identical query rediscovered the same
// candidates only to discard them as already compiled.
//
// These helpers are the only place hydration touches the session-scoped
// discovery cache (`crate::mcp::discovery_cache`). They record *completion* of
// discovery — never candidates and never query answers — and they hold the
// cache lock only for the individual lookup/insert, never across CBM calls,
// filesystem scans, source reads, or compilation.

use super::filesystem::root_key;
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
) -> bool {
    state
        .hydration_discovery_lock()
        .is_complete(scope, discovery, query_name)
}

/// Record a successful discovery for one scope/mode/name.
pub(super) fn mark_discovery_complete(
    state: &McpState,
    scope: &DiscoveryScope,
    discovery: DiscoveryMode,
    query_name: &str,
) {
    state
        .hydration_discovery_lock()
        .mark_complete(scope, discovery, query_name);
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
) -> (Vec<PathBuf>, usize) {
    let mut pending = Vec::new();
    let mut cached = 0;
    for root in roots {
        let scope = DiscoveryScope::filesystem(root_key(&root));
        if discovery_is_complete(state, &scope, discovery, query_name) {
            cached += 1;
        } else {
            pending.push(root);
        }
    }
    (pending, cached)
}

/// Record successful filesystem discovery for every root in a completed scan.
/// A partial scan deliberately records nothing, so its roots stay retryable.
pub(super) fn mark_discovery_complete_for_roots(
    state: &McpState,
    discovery: DiscoveryMode,
    query_name: &str,
    roots: &[PathBuf],
) {
    let scopes: Vec<DiscoveryScope> = roots
        .iter()
        .map(|root| DiscoveryScope::filesystem(root_key(root)))
        .collect();
    let mut cache = state.hydration_discovery_lock();
    for scope in &scopes {
        cache.mark_complete(scope, discovery, query_name);
    }
}
