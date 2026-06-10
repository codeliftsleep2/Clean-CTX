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

use std::collections::HashMap;
use std::sync::Arc;
use crate::angular_meta::graph_state::AngularGraphHandle;
use crate::cache::LocalStateCache;
use crate::config::CleanCtxConfig;
use crate::dictionary::PathDictionary;
use crate::compression::text_delta::TextDeltaComputer;
use crate::ir::replay::ContextState;
use crate::mcp::context_store::InMemoryContextStore;
use crate::mcp::session_stats::SessionStats;
use crate::mcp::sqlite_store::SqliteStore;

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
    /// Compiler IR context state — tracks all files and their IR state.
    /// Enables delta-based state transport: load full IR on first
    /// compress, then apply deltas on subsequent edits.
    pub ir_context: ContextState,
    /// Phase IV (Idea #12): Text-level delta compressor.
    /// Stores compressed body snapshots per file and computes
    /// line-level deltas for delta-based text transport.
    pub text_delta: TextDeltaComputer,
    /// F-FULL-01/F-FULL-05: Shared file-content cache keyed by raw path.
    /// All I/O paths check this cache first, populating it on first read.
    /// Subsequent reads (from IR compiler, bundle_pass, graph_pass) are
    /// O(1) lookups. Files are stored as `Arc<String>` to avoid clones.
    pub source_cache: HashMap<String, Arc<String>>,
    /// F-FINAL-06: Accumulated warnings surfaced via the JSON-RPC
    /// `_warnings` field. Sub-systems that previously used `eprintln!`
    /// (e.g. duplicate class name in the Angular graph) now push
    /// here. Drained by tool handlers before each response.
    pub warnings: Vec<String>,
    /// Session-level stats accumulator for the dashboard.
    /// Every `provide_code_context` call records token savings here.
    pub session_stats: SessionStats,
    /// In-memory context store for persistence-ready baselines.
    pub context_store: InMemoryContextStore,
    /// Optional SQLite persistence store for cross-session survival.
    /// Initialized from `config.persistence` — `None` if disabled or
    /// if DB open fails.
    pub persistence_store: Option<SqliteStore>,
}

impl McpState {
    /// Create a fresh state object with the given config and empty
    /// registries.
    pub fn new(config: CleanCtxConfig) -> Self {
        // Initialize SQLite persistence store if enabled in config
        let persistence_store = if config.persistence.enabled {
            let db_path = std::path::Path::new(&config.persistence.db_path);
            match SqliteStore::open(db_path) {
                Ok(store) => {
                    eprintln!("[clean-ctx] Persistence enabled: {}", db_path.display());
                    Some(store)
                }
                Err(e) => {
                    eprintln!("[clean-ctx] WARNING: Failed to open persistence DB: {e}");
                    None
                }
            }
        } else {
            None
        };

        Self {
            dict: PathDictionary::new(),
            cache: LocalStateCache::new(),
            config,
            angular_graph: AngularGraphHandle::new(),
            ir_context: ContextState::new(),
            text_delta: TextDeltaComputer::new(),
            source_cache: HashMap::new(),
            // F-FINAL-06: empty warning buffer at session start.
            warnings: Vec::new(),
            session_stats: SessionStats::new(),
            context_store: InMemoryContextStore::new(),
            persistence_store,
        }
    }

    /// F-FINAL-06: Push a warning into the session buffer. The next
    /// `tools/call` response will surface it in the `_warnings` field
    /// and then clear the buffer. The single-threaded MCP dispatch
    /// chain guarantees no concurrent access.
    pub fn push_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    /// F-FINAL-06: Drain all accumulated warnings. Returns a `Vec`
    /// that the caller embeds in the response's `_warnings` field.
    pub fn drain_warnings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.warnings)
    }

    /// Borrow the path dictionary mutably.
    pub fn dict_mut(&mut self) -> &mut PathDictionary {
        &mut self.dict
    }

    /// Borrow the cache mutably.
    pub fn cache_mut(&mut self) -> &mut LocalStateCache {
        &mut self.cache
    }

    /// Borrow the IR context mutably.
    pub fn ir_context_mut(&mut self) -> &mut ContextState {
        &mut self.ir_context
    }

    /// Borrow the text delta computer mutably.
    pub fn text_delta_mut(&mut self) -> &mut TextDeltaComputer {
        &mut self.text_delta
    }

    /// F-FULL-01/F-FULL-05: Read file content, using the shared source cache.
    /// Returns `Arc<String>` so the cache can be shared across passes
    /// without cloning the underlying string data.
    pub fn read_source(&mut self, path: &str) -> Result<Arc<String>, std::io::Error> {
        use std::path::Path;
        let cache_key = Path::new(path)
            .canonicalize()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string());
        if let Some(cached) = self.source_cache.get(&cache_key) {
            return Ok(Arc::clone(cached));
        }
        let content = Arc::new(std::fs::read_to_string(path)?);
        self.source_cache.insert(cache_key, Arc::clone(&content));
        Ok(content)
    }
}
