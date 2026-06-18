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

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::angular_meta::graph_state::AngularGraphHandle;
use crate::cache::LocalStateCache;
use crate::config::CleanCtxConfig;
use crate::dictionary::PathDictionary;
use crate::compression::text_delta::TextDeltaComputer;
use crate::ir::replay::ContextState;
use crate::mcp::buffered_store::BufferedStore;
use crate::mcp::cache_hints::CacheMetrics;
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
    /// Optional buffered SQLite persistence store for cross-session survival.
    /// Writes are queued in memory and flushed in batch transactions.
    /// Initialized from `config.persistence` — `None` if disabled or
    /// if DB open fails.
    pub persistence_store: Option<BufferedStore>,

    /// Tracks which cache breakpoints have already been emitted this session.
    /// Key format: "{region}::{breaker}" — e.g., "tools::tools-v1".
    /// Deduplication prevents paying the 2.0× write multiplier on re-emission.
    pub emitted_breakpoints: HashSet<String>,

    /// Cache efficiency metrics for the dashboard.
    /// Records hits, misses, tokens_saved, and per-region status.
    pub cache_metrics: CacheMetrics,

    /// CBM (codebase-memory-mcp) graph bridge for graph intelligence.
    /// `None` if CBM is not installed, disabled, or failed to launch.
    pub graph_bridge: Option<crate::cbm::GraphBridge>,

    /// CBM integration status, mirrored for quick access.
    pub cbm_status: crate::cbm::CbmStatus,

    /// Phase 1 (Fix D): Cache of rendered LLM-optimized hierarchical IR text,
    /// keyed by path alias (e.g., "α1").
    ///
    /// Cached on first render after a compile; invalidated when a delta is
    /// applied to the file or when `restore_context` is called for the file.
    /// This avoids re-rendering the full HIR on every delta-mode call,
    /// saving ~O(n) where n is the file size in HIR nodes.
    pub llm_text_cache: HashMap<String, String>,
}

impl McpState {
    /// Create a fresh state object with the given config and empty
    /// registries.
    pub fn new(config: CleanCtxConfig) -> Self {
        // Initialize buffered persistence store if enabled in config
        let persistence_store = if config.persistence.enabled {
            // Resolve DB path relative to project root, not CWD
            let project_root = crate::mcp::server::find_project_root();
            let db_path = project_root.join(&config.persistence.db_path);
            match SqliteStore::open(&db_path) {
                Ok(store) => {
                    eprintln!("[clean-ctx] Persistence enabled: {}", db_path.display());
                    Some(BufferedStore::new(store, project_root.clone()))
                }
                Err(e) => {
                    eprintln!("[clean-ctx] WARNING: Failed to open persistence DB: {e}");
                    None
                }
            }
        } else {
            None
        };

        // Rehydrate session stats from DB if available
        let mut session_stats = SessionStats::new();
        if let Some(ref store) = persistence_store {
            // Flush any pending writes, then rebuild stats from DB
            store.flush();
            if let Some(guard) = store.sqlite() {
                match guard.rebuild_stats() {
                    Ok(stats) => {
                        session_stats = stats;
                        eprintln!("[clean-ctx] Loaded persisted session stats.");
                    }
                    Err(e) => {
                        eprintln!("[clean-ctx] WARNING: Failed to rebuild stats from DB: {e}");
                    }
                }
            }
        }

        // Initialize CBM graph bridge from config *before* moving config
        // into Self (avoiding after-move borrow).
        let cbm_config = config.cbm.clone();
        let project_root = crate::mcp::server::find_project_root().clone();
        let (graph_bridge, cbm_status) = Self::init_cbm_bridge(&cbm_config, &project_root);

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
            session_stats,
            context_store: InMemoryContextStore::new(),
            persistence_store,
            llm_text_cache: HashMap::new(),
            emitted_breakpoints: HashSet::new(),
            cache_metrics: CacheMetrics::default(),
            graph_bridge,
            cbm_status,
        }
    }

    /// Try to initialize the CBM graph bridge.
    fn init_cbm_bridge(
        cbm_config: &crate::cbm::CbmConfig,
        project_root: &std::path::Path,
    ) -> (Option<crate::cbm::GraphBridge>, crate::cbm::CbmStatus) {
        let bridge = crate::cbm::GraphBridge::try_create(cbm_config, project_root);
        if bridge.is_available() {
            eprintln!("[clean-ctx] CBM graph intelligence: available");
            let status = bridge.status().clone();
            (Some(bridge), status)
        } else {
            let status = bridge.status().clone();
            if status.summary() != "unavailable" {
                eprintln!("[clean-ctx] CBM graph intelligence: {}", status.summary());
            }
            (None, status)
        }
    }

    /// Flush any pending persistence writes to SQLite.
    /// Returns the number of operations flushed.
    pub fn flush_persistence(&self) -> usize {
        if let Some(ref store) = self.persistence_store {
            store.flush()
        } else {
            0
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

#[cfg(test)]
#[path = "../tests/mcp/state.rs"]
mod tests;