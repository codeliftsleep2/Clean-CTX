// Hydration discovery invalidation.
//
// A completed discovery stops being valid when the workspace can contain newly
// relevant source. Every hook below reuses an existing workspace-lifecycle
// event — `apply_edit` (via `invalidate_discovery_for_edited_path`) and explicit
// repository reindexing (via `invalidate_discovery_for_root`) — rather than
// introducing a watcher, timer, TTL, or background worker.
//
// External edits remain the documented Clean-CTX boundary: the host editor's
// writes are invisible to Clean-CTX (docs/CLAUDE_INTEGRATION_RULES.md), apart
// from files the session re-reads through the source cache, whose mtime/size
// staleness path drops the whole discovery cache
// (`McpState::read_source` → `invalidate_hydration_discovery_all`).

use super::filesystem::{configured_roots, root_key};
use crate::mcp::McpState;
use std::path::Path;

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
        .filter(|root| Path::new(&edited).starts_with(Path::new(root)))
        .max_by_key(String::len);
    match matched {
        Some(root) => state.hydration_discovery_lock().invalidate_root(&root),
        None => state.hydration_discovery_lock().invalidate_all(),
    }
}

/// Invalidate hydration discovery for one repository root.
///
/// Called when a repository is explicitly reindexed (`index_repository`, or
/// `cbm_proxy` with `cbm_tool: "index_repository"`). That tool exists precisely
/// for external edits Clean-CTX cannot observe, so a refresh must also drop the
/// discovery recorded against the pre-refresh graph.
pub(crate) fn invalidate_discovery_for_root(state: &McpState, root: &str) {
    let identity = crate::dictionary::path::canonical_identity_key(root);
    state.hydration_discovery_lock().invalidate_root(&identity);
}
