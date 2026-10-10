// Hydration discovery invalidation.
//
// A completed discovery stops being valid when the workspace can contain newly
// relevant source. Every hook below reuses an existing workspace-lifecycle
// event — `apply_edit` (via `invalidate_discovery_for_edited_path`) and explicit
// explicit workspace refresh (via `reconcile_external_refresh_for_root`) —
// rather than introducing a watcher, timer, TTL, or background worker.
//
// Host/editor writes are invisible until the operator calls `refresh_workspace`
// (or a successful CBM repository reindex reuses the same reconciliation),
// apart from files the session re-reads through the source cache, whose
// mtime/size staleness path drops the whole discovery cache
// (`McpState::read_source` → `invalidate_hydration_discovery_all`).

use super::filesystem::{configured_roots, root_key};
use crate::mcp::McpState;

/// Invalidate hydration discovery for the configured root containing
/// `file_path`.
///
/// Called after a successful `apply_edit` commit. The edit may have added or
/// removed a declaration/consumer that only a fresh discovery pass can see, so
/// every discovery recorded for that root in the current generation is dropped
/// (the file's CBM project is marked dirty separately by the caller).
///
/// A path inside no configured root — possible when a caller passed an explicit
/// `workspaceRoot` outside the configured set — invalidates every scope.
/// Over-invalidating only costs one rediscovery; missing an invalidation would
/// silently serve an incomplete candidate set.
pub(crate) fn invalidate_discovery_for_edited_path(state: &McpState, file_path: &str) {
    let edited = crate::dictionary::path::canonical_identity_key(file_path);
    let matched = configured_roots(state, None)
        .iter()
        .map(|root| root_key(root))
        .filter(|root| crate::workspace::path_identity::is_within_root(&edited, root))
        .max_by_key(String::len);
    match matched {
        Some(root) => state.hydration_discovery_lock().invalidate_root(&root),
        None => state.hydration_discovery_lock().invalidate_all(),
    }
}

/// Establish a new current-source generation for one repository root.
///
/// Called directly by `refresh_workspace` and reused after a successful CBM
/// repository reindex. After the refresh succeeds,
/// both discovery and semantic projections from the previous generation cease
/// to be current. Fresh hydration may republish eligible owners from canonical
/// source; durable historical artifacts do not independently regain authority.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RefreshReconciliation {
    pub(crate) indexed_owners_retracted: usize,
    pub(crate) source_snapshots_invalidated: usize,
    pub(crate) pending_transitions_retired: usize,
}

impl RefreshReconciliation {
    pub(crate) fn merge(&mut self, other: Self) {
        self.indexed_owners_retracted += other.indexed_owners_retracted;
        self.source_snapshots_invalidated += other.source_snapshots_invalidated;
        self.pending_transitions_retired += other.pending_transitions_retired;
    }
}

pub(crate) fn reconcile_external_refresh_for_root(
    state: &McpState,
    root: &str,
) -> RefreshReconciliation {
    let identity = crate::dictionary::path::canonical_identity_key(root);
    let source_snapshots_invalidated = state.invalidate_source_cache_in_root(&identity);
    let pending_transitions_retired = state.forget_pending_transitions_in_root(&identity);
    let indexed_owners_retracted = state.workspace_index_lock().remove_files_in_root(&identity);
    let reconciliation = RefreshReconciliation {
        indexed_owners_retracted,
        source_snapshots_invalidated,
        pending_transitions_retired,
    };
    // Invalidate discovery last. A concurrent hydration that finishes before
    // this point is either retracted above or publishes source-current facts;
    // after this point every new query observes the fresh discovery generation.
    // Invalidating first could let a query mark that generation complete and
    // then have its newly published projection removed by this refresh.
    invalidate_discovery_for_root(state, &identity);
    reconciliation
}

/// Drop completed discovery for one root without changing semantic authority.
///
/// This remains available to focused cache-granularity tests and lifecycle
/// events that establish only discovery staleness. External repository refresh
/// uses [`reconcile_external_refresh_for_root`] instead.
pub(crate) fn invalidate_discovery_for_root(state: &McpState, root: &str) {
    let identity = crate::dictionary::path::canonical_identity_key(root);
    state.hydration_discovery_lock().invalidate_root(&identity);
}
