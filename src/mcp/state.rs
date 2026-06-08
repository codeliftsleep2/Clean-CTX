// src/mcp/state.rs
//
// Per-session state carried through the MCP dispatch chain.
//
// F-05 (FAANG audit): the previous design loaded `CleanCtxConfig` at
// server startup (`let _config = CleanCtxConfig::load(...)`) and then
// immediately discarded it. Nothing in the handler chain ever consulted
// the user's `exclude_patterns`, `fidelity_overrides`, or `type_aliases`.
//
// The state object below bundles all the registries a tool call may
// need into a single mutable handle:
//   - `dict`    : path-alias dictionary (shared with the orchestrators)
//   - `cache`   : content-hash + baseline-snapshot cache
//   - `config`  : the user's `CleanCtxConfig`
//
// Tool handlers take `&mut McpState` rather than three separate
// arguments; the dict and cache stay single-threaded (the MCP server
// is single-threaded by design) and the config is shared immutably.

use crate::angular_meta::graph_state::AngularGraphHandle;
use crate::cache::LocalStateCache;
use crate::config::CleanCtxConfig;
use crate::dictionary::PathDictionary;

/// Per-session state shared by all MCP tool handlers.
pub struct McpState {
    /// Path-alias dictionary (`α1`, `α2`, …). Mutated in place by
    /// `compress_code_context` and `compress_workspace`.
    pub dict: PathDictionary,
    /// Content-hash + baseline-snapshot cache. Mutated in place by
    /// `diff_code_context` and the orchestrators.
    pub cache: LocalStateCache,
    /// Project-level configuration loaded from `.clean-ctx.json`.
    /// Treated as immutable for the session — the operator must
    /// restart the server to pick up edits.
    pub config: CleanCtxConfig,
    /// Angular cross-file dependency graph (Phase 3, Tier 3).
    /// Built once per `compress_workspace` call; `None` when no
    /// workspace has been compressed yet.
    pub angular_graph: AngularGraphHandle,
}

impl McpState {
    /// Create a fresh state object with the given config and empty
    /// registries.
    pub fn new(config: CleanCtxConfig) -> Self {
        Self {
            dict: PathDictionary::new(),
            cache: LocalStateCache::new(),
            config,
            angular_graph: AngularGraphHandle::new(),
        }
    }

    /// Borrow the path dictionary mutably.
    pub fn dict_mut(&mut self) -> &mut PathDictionary {
        &mut self.dict
    }

    /// Borrow the cache mutably.
    pub fn cache_mut(&mut self) -> &mut LocalStateCache {
        &mut self.cache
    }
}
