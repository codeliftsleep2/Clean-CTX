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
// Tool handlers take `&McpState` (interior mutability) — all mutable fields
// use `Mutex`/`RwLock` internally, so `&mut` is never required.
// is single-threaded by design) and the config is shared immutably.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, RwLock};
use std::sync::Arc;
use crate::angular_meta::graph_state::AngularGraphHandle;
use crate::cache::LocalStateCache;
use crate::config::CleanCtxConfig;
use crate::dictionary::PathDictionary;
use crate::compression::text_delta::TextDeltaComputer;
use crate::ir::replay::ContextState;
use crate::layers::LayerRegistry;
use crate::mcp::buffered_store::BufferedStore;
use crate::mcp::cache_hints::CacheMetrics;
use crate::mcp::context_store::InMemoryContextStore;
use crate::mcp::session_stats::SessionStats;
use crate::mcp::sqlite_store::SqliteStore;

/// Per-file CBM filter state: symbols to skip during compression.
///
/// Populated by the CBM Intelligence Layer **before** compression runs.
/// The compression pipeline checks this set for each capture and drops
/// low-importance symbols (score < 0.4) entirely, so CBM reduces token
/// output instead of adding enrichment data after the fact.
///
/// Keyed by absolute file path; value is the set of low-importance
/// symbol names to exclude from the compressed output.
#[derive(Debug, Clone, Default)]
pub struct CbmFilterState {
    /// Symbol names to skip, keyed by file path.
    pub skip_sets: HashMap<String, HashSet<String>>,
}

/// Per-session state shared by all MCP tool handlers.
///
/// v0.2.0: Uses top-level RwLock for parallel reads (see dispatcher).
/// v0.3.0: Will migrate to interior mutability for fine-grained parallelism.
pub struct McpState {
    /// Path-alias dictionary (`α1`, `α2`, …). Mutated in place by
    /// `compress_code_context` and `compress_workspace`.
    pub dict: Mutex<PathDictionary>,
    /// Content-hash + baseline-snapshot cache. Mutated in place by
    /// `diff_code_context` and the orchestrators.
    pub cache: RwLock<LocalStateCache>,
    /// Project-level configuration loaded from `.clean-ctx.json`.
    /// Treated as immutable for the session — the operator must
    /// restart the server to pick up edits.
    pub config: CleanCtxConfig,
    /// Angular cross-file dependency graph (Phase 3, Tier 3).
    /// Built once per `compress_workspace` call; `None` when no
    /// workspace has been compressed yet.
    pub angular_graph: Mutex<AngularGraphHandle>,
    /// Compiler IR context state — tracks all files and their IR state.
    /// Enables delta-based state transport: load full IR on first
    /// compress, then apply deltas on subsequent edits.
    pub ir_context: RwLock<ContextState>,
    /// Phase IV (Idea #12): Text-level delta compressor.
    /// Stores compressed body snapshots per file and computes
    /// line-level deltas for delta-based text transport.
    /// Wrapped in Mutex for interior mutability (v0.2.0+).
    pub text_delta: Mutex<TextDeltaComputer>,
    /// F-FULL-01/F-FULL-05: Shared file-content cache keyed by raw path.
    /// All I/O paths check this cache first, populating it on first read.
    /// Subsequent reads (from IR compiler, bundle_pass, graph_pass) are
    /// O(1) lookups. Files are stored as `Arc<String>` to avoid clones.
    pub source_cache: Mutex<HashMap<String, Arc<String>>>,
    /// F-FINAL-06: Accumulated warnings surfaced via the JSON-RPC
    /// `_warnings` field. Sub-systems that previously used `eprintln!`
    /// (e.g. duplicate class name in the Angular graph) now push
    /// here. Drained by tool handlers before each response.
    pub warnings: Mutex<Vec<String>>,
    /// Session-level stats accumulator for the dashboard.
    /// Every `provide_code_context` call records token savings here.
    pub session_stats: Mutex<SessionStats>,
    /// In-memory context store for persistence-ready baselines.
    pub context_store: InMemoryContextStore,
    /// Optional buffered SQLite persistence store for cross-session survival.
    /// Writes are queued in memory and flushed in batch transactions.
    /// Initialized from `config.persistence` — `None` if disabled or
    /// if DB open fails.
    pub persistence_store: Mutex<Option<BufferedStore>>,

    /// Tracks which cache breakpoints have already been emitted this session.
    /// Key format: "{region}::{breaker}" — e.g., "tools::tools-v1".
    /// Deduplication prevents paying the 2.0× write multiplier on re-emission.
    pub emitted_breakpoints: Mutex<HashSet<String>>,

    /// Cache efficiency metrics for the dashboard.
    /// Records hits, misses, tokens_saved, and per-region status.
    pub cache_metrics: Mutex<CacheMetrics>,

    /// CBM (codebase-memory-mcp) graph bridge for graph intelligence.
    /// `None` if CBM is not installed, disabled, or failed to launch.
    pub graph_bridge: Mutex<Option<crate::cbm::GraphBridge>>,

    /// CBM integration status, mirrored for quick access.
    pub cbm_status: crate::cbm::CbmStatus,

    /// CBM filter state: per-file skip sets populated by the Intelligence
    /// Layer before compression. When a symbol has low importance (< 0.4),
    /// it is added here and the capture pipeline drops it during compression.
    pub cbm_filter: Mutex<CbmFilterState>,

    /// Phase 1 (Fix D): Cache of rendered LLM-optimized hierarchical IR text,
    /// keyed by path alias (e.g., "α1").
    ///
    /// Cached on first render after a compile; invalidated when a delta is
    /// applied to the file or when `restore_context` is called for the file.
    /// This avoids re-rendering the full HIR on every delta-mode call,
    /// saving ~O(n) where n is the file size in HIR nodes.
    pub llm_text_cache: Mutex<HashMap<String, String>>,

    /// Phase 2: Proxy port for fetching tool-filtering and cache stats.
    /// Defaults to 8787 (the proxy's default port).
    pub proxy_port: u16,

    /// Layer registry for language/meta-layer dispatch.
    /// Initialized once at startup from the enabled Cargo features.
    pub registry: LayerRegistry,
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
            dict: Mutex::new(PathDictionary::new()),
            cache: RwLock::new(LocalStateCache::new()),
            config,
            angular_graph: Mutex::new(AngularGraphHandle::new()),
            ir_context: RwLock::new(ContextState::new()),
            text_delta: Mutex::new(TextDeltaComputer::new()),
            source_cache: Mutex::new(HashMap::new()),
            // F-FINAL-06: empty warning buffer at session start.
            warnings: Mutex::new(Vec::new()),
            session_stats: Mutex::new(session_stats),
            context_store: InMemoryContextStore::new(),
            persistence_store: Mutex::new(persistence_store),
            llm_text_cache: Mutex::new(HashMap::new()),
            emitted_breakpoints: Mutex::new(HashSet::new()),
            cache_metrics: Mutex::new(CacheMetrics::default()),
            cbm_filter: Mutex::new(CbmFilterState::default()),
            graph_bridge: Mutex::new(graph_bridge),
            cbm_status,
            proxy_port: 8787,
            registry: LayerRegistry::new(),
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
            // Indexing is deferred — `ensure_indexed()` will be called on
            // first actual query, avoiding blocking `McpState::new()`.
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

    /// Lock the path dictionary for mutation.
    pub fn dict_lock(&self) -> std::sync::MutexGuard<'_, PathDictionary> {
        match self.dict.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (dict)"); poisoned.into_inner() }
        }
    }

    /// Lock the cache for reading.
    pub fn cache_read(&self) -> std::sync::RwLockReadGuard<'_, LocalStateCache> {
        match self.cache.read() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned read lock (cache)"); poisoned.into_inner() }
        }
    }

    /// Lock the cache for writing.
    pub fn cache_write(&self) -> std::sync::RwLockWriteGuard<'_, LocalStateCache> {
        match self.cache.write() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned write lock (cache)"); poisoned.into_inner() }
        }
    }

    /// Lock the IR context for reading.
    pub fn ir_context_read(&self) -> std::sync::RwLockReadGuard<'_, ContextState> {
        match self.ir_context.read() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned read lock (ir_context)"); poisoned.into_inner() }
        }
    }

    /// Lock the IR context for writing.
    pub fn ir_context_lock(&self) -> std::sync::RwLockWriteGuard<'_, ContextState> {
        match self.ir_context.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[clean-ctx] WARNING: Recovering from poisoned write lock (ir_context)");
                poisoned.into_inner()
            }
        }
    }


    /// Lock the session stats for writing.
    pub fn session_stats_lock(&self) -> std::sync::MutexGuard<'_, SessionStats> {
        match self.session_stats.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (session_stats)"); poisoned.into_inner() }
        }
    }

    /// Lock the source cache for writing.
    pub fn source_cache_lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<String>>> {
        match self.source_cache.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (source_cache)"); poisoned.into_inner() }
        }
    }

    /// Lock the CBM filter state for writing.
    pub fn cbm_filter_lock(&self) -> std::sync::MutexGuard<'_, CbmFilterState> {
        match self.cbm_filter.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (cbm_filter)"); poisoned.into_inner() }
        }
    }

    /// Lock the LLM text cache for writing.
    pub fn llm_text_cache_lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, String>> {
        match self.llm_text_cache.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (llm_text_cache)"); poisoned.into_inner() }
        }
    }

    /// Lock the graph bridge for mutation.
    pub fn graph_bridge_lock(&self) -> std::sync::MutexGuard<'_, Option<crate::cbm::GraphBridge>> {
        match self.graph_bridge.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (graph_bridge)"); poisoned.into_inner() }
        }
    }

    /// Lock the angular graph for mutation.
    pub fn angular_graph_lock(&self) -> std::sync::MutexGuard<'_, AngularGraphHandle> {
        match self.angular_graph.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (angular_graph)"); poisoned.into_inner() }
        }
    }

    /// Lock the persistence store for writing.
    pub fn persistence_store_lock(&self) -> std::sync::MutexGuard<'_, Option<BufferedStore>> {
        match self.persistence_store.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (persistence_store)"); poisoned.into_inner() }
        }
    }

    /// Lock the emitted breakpoints for writing.
    pub fn emitted_breakpoints_lock(&self) -> std::sync::MutexGuard<'_, HashSet<String>> {
        match self.emitted_breakpoints.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (emitted_breakpoints)"); poisoned.into_inner() }
        }
    }

    /// Lock the cache metrics for writing.
    pub fn cache_metrics_lock(&self) -> std::sync::MutexGuard<'_, CacheMetrics> {
        match self.cache_metrics.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (cache_metrics)"); poisoned.into_inner() }
        }
    }

    /// Get or create a path alias (thread-safe convenience method).
    pub fn get_or_create_alias(&self, path: String) -> String {
        self.dict_lock().get_or_create_alias(path)
    }

    /// Get or create a bundle alias (thread-safe convenience method).
    pub fn get_or_create_bundle_alias(&self, component_name: String) -> String {
        self.dict_lock().get_or_create_bundle_alias(component_name)
    }

    /// Format the dictionary footer (thread-safe convenience method).
    pub fn format_dict_footer(&self) -> String {
        self.dict_lock().format_footer()
    }

    #[allow(clippy::too_many_arguments)]
    /// Record compression stats (thread-safe convenience method).
    pub fn record_compression(&self, file_path: &str, raw_tokens: usize, compressed_tokens: usize, fidelity: &str, is_angular: bool, source: &str, full_compressed_tokens: Option<usize>, domain: &str) {
        self.session_stats_lock().record_compression(file_path, raw_tokens, compressed_tokens, fidelity, is_angular, source, full_compressed_tokens, domain);
    }

    /// Get file version from IR context (thread-safe convenience method).
    pub fn file_version(&self, path_alias: &str) -> Option<u64> {
        match self.ir_context.read() {
            Ok(g) => g.file_version(path_alias),
            Err(p) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned read lock (ir_context)"); p.into_inner().file_version(path_alias) }
        }
    }

    /// Access CBM filter skip set for a file (thread-safe convenience method).
    pub fn get_skip_set(&self, file_path: &str) -> Option<HashSet<String>> {
        self.cbm_filter_lock().skip_sets.get(file_path).cloned()
    }

    pub fn flush_persistence(&self) -> usize {
        let guard = self.persistence_store_lock();
        if let Some(ref store) = *guard {
            store.flush()
        } else {
            0
        }
    }

    /// F-FINAL-06: Push a warning into the session buffer. The next
    /// `tools/call` response will surface it in the `_warnings` field
    /// and then clear the buffer. The single-threaded MCP dispatch
    /// chain guarantees no concurrent access.
    pub fn push_warning(&self, msg: impl Into<String>) {
        match self.warnings.lock() {
            Ok(mut guard) => guard.push(msg.into()),
            Err(poisoned) => {
                eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (warnings)");
                poisoned.into_inner().push(msg.into());
            }
        }
    }

    /// F-FINAL-06: Drain all accumulated warnings. Returns a `Vec`
    /// that the caller embeds in the response's `_warnings` field.
    pub fn drain_warnings(&self) -> Vec<String> {
        match self.warnings.lock() {
            Ok(mut guard) => std::mem::take(&mut *guard),
            Err(poisoned) => {
                eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (warnings)");
                std::mem::take(&mut *poisoned.into_inner())
            }
        }
    }

    /// Lock the text delta computer for mutation.
    pub fn text_delta_lock(&self) -> std::sync::MutexGuard<'_, TextDeltaComputer> {
        match self.text_delta.lock() {
            Ok(guard) => guard,
            Err(poisoned) => { eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (text_delta)"); poisoned.into_inner() }
        }
    }

    /// Resolve a cache key for `source_cache`. On Windows, `canonicalize`
    /// on TempDir paths can trigger Defender deep-scan hooks (10-30s per
    /// call). We skip canonicalize when the path is already absolute and
    /// contains no relative components (`..`), falling back to the raw
    /// string as the key.
    fn resolve_cache_key(path: &str) -> String {
        use std::path::Path;
        let p = Path::new(path);
        if p.is_absolute() && !path.contains("..") && !path.contains("./") && !path.contains(".\\") {
            #[cfg(debug_assertions)]
            eprintln!("[resolve_cache_key] FAST PATH: {}", path);
            return path.to_string();
        }
        #[cfg(debug_assertions)]
        eprintln!("[resolve_cache_key] SLOW PATH (canonicalize): {}", path);
        let canon_start = std::time::Instant::now();
        let result = p.canonicalize()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string());
        #[cfg(debug_assertions)]
        eprintln!("[resolve_cache_key] canonicalize took {:?} for {}", canon_start.elapsed(), path);
        result
    }

    /// F-FULL-01/F-FULL-05: Read file content, using the shared source cache.
    /// Returns `Arc<String>` so the cache can be shared across passes
    /// without cloning the underlying string data.
    ///
    /// **Two-phase locking:** The Mutex is held only during cache lookup
    /// and update, NOT during `read_to_string`. This prevents I/O from
    /// blocking concurrent readers.
    pub fn read_source(&self, path: &str) -> Result<Arc<String>, std::io::Error> {
        let cache_key = Self::resolve_cache_key(path);
        #[cfg(debug_assertions)]
        let overall_start = std::time::Instant::now();

        // Phase 1: Check cache (brief lock, release before I/O)
        {
            #[cfg(debug_assertions)]
            let lock_start = std::time::Instant::now();
            let cache = match self.source_cache.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (source_cache)");
                    poisoned.into_inner()
                }
            };
            #[cfg(debug_assertions)]
            eprintln!("[read_source] Phase 1 lock acquire took {:?} for {}", lock_start.elapsed(), path);
            if let Some(cached) = cache.get(&cache_key) {
                #[cfg(debug_assertions)]
                eprintln!("[read_source] CACHE HIT for {} (total: {:?})", path, overall_start.elapsed());
                return Ok(Arc::clone(cached));
            }
            #[cfg(debug_assertions)]
            eprintln!("[read_source] CACHE MISS for {}", path);
        }

        // Phase 2: Read file WITHOUT holding the lock
        #[cfg(debug_assertions)]
        let io_start = std::time::Instant::now();
        let content = Arc::new(std::fs::read_to_string(path)?);
        #[cfg(debug_assertions)]
        eprintln!("[read_source] Phase 2 read_to_string took {:?} for {} ({} bytes)", io_start.elapsed(), path, content.len());

        // Phase 3: Update cache (brief lock, with double-check)
        #[cfg(debug_assertions)]
        let lock2_start = std::time::Instant::now();
        let mut cache = match self.source_cache.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (source_cache)");
                poisoned.into_inner()
            }
        };
        #[cfg(debug_assertions)]
        eprintln!("[read_source] Phase 3 lock acquire took {:?} for {}", lock2_start.elapsed(), path);
        cache.entry(cache_key).or_insert(content.clone());
        #[cfg(debug_assertions)]
        eprintln!("[read_source] TOTAL for {}: {:?}", path, overall_start.elapsed());

        Ok(content)
    }
}

#[cfg(test)]
#[path = "../tests/mcp/state.rs"]
mod tests;