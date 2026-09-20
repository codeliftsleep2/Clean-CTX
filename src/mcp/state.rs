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

use crate::cache::LocalStateCache;
use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::dictionary::PathDictionary;
use crate::ir::replay::ContextState;
use crate::layers::LayerRegistry;
use crate::mcp::buffered_store::BufferedStore;
use crate::mcp::cache_hints::CacheMetrics;
use crate::mcp::context_store::InMemoryContextStore;
use crate::mcp::discovery_cache::HydrationDiscoveryCache;
use crate::mcp::session_stats::SessionStats;
use crate::mcp::sqlite_store::SqliteStore;
use crate::mcp::state::source_cache::CacheEntry;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, RwLock};

// The session's shared file-content cache and its read boundary
// (`CacheEntry`, `source_cache` reads, metadata-based invalidation), split
// out of this file into one semantic module when `hydration_discovery` was
// added below.
mod source_cache;

/// P1-5: Lock recovery macro — replaces 20+ identical 4-line match patterns.
///
/// Usage: `let guard = lock_or_recover!(self.dict.lock(), "dict");`
/// instead of:
/// ```ignore
/// match self.dict.lock() {
///     Ok(guard) => guard,
///     Err(poisoned) => {
///         eprintln!("[clean-ctx] WARNING: Recovering from poisoned lock (dict)");
///         poisoned.into_inner()
///     }
/// }
/// ```
macro_rules! lock_or_recover {
    ($lock:expr, $name:expr) => {
        match $lock {
            Ok(guard) => guard,
            Err(poisoned) => {
                // The lock was poisoned by a panic on another thread. The
                // data may be in a partially-updated, inconsistent state.
                // Log a prominent warning so operators can correlate this
                // with any preceding panic messages, then proceed with the
                // potentially corrupt contents — callers must treat the
                // returned state as untrusted and re-validate critical fields.
                eprintln!(
                    "[clean-ctx] WARNING: Recovering from poisoned lock ({}). \
                     State may be inconsistent; validate before use.",
                    $name
                );
                poisoned.into_inner()
            }
        }
    };
}

#[path = "durable_semantics.rs"]
pub(crate) mod durable_semantics;

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
///
/// # Lock Ordering (canonical acquisition order)
///
/// To prevent deadlocks, handlers must acquire locks in this order:
/// 1. `ir_context` (RwLock) - IR state for delta computation
/// 2. `source_cache` (Mutex) - File content cache
/// 3. `cache` (RwLock) - Local state cache for snapshots
/// 4. `dict` (Mutex) - Path dictionary for aliases
/// 5. `persistence_store` (Mutex) - SQLite persistence
/// 6. All other Mutex fields (any order)
///
/// Violating this order may cause deadlocks under concurrent load.
///
/// # Example - CORRECT:
/// ```ignore
/// let ir = state.ir_context_lock();
/// let source = state.read_source(path)?;  // acquires source_cache
/// let cache = state.cache_write();  // acquires cache
/// ```
///
/// # Example - WRONG (potential deadlock):
/// ```ignore
/// let cache = state.cache_write();
/// let ir = state.ir_context_lock();  // Another thread holds ir_context, waiting for cache
/// ```
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
    /// Compiler IR context state — tracks all files and their IR state.
    /// Enables delta-based state transport: load full IR on first
    /// compress, then apply deltas on subsequent edits.
    pub ir_context: RwLock<ContextState>,
    /// F-FULL-01/F-FULL-05: Shared file-content cache keyed by raw path.
    /// All I/O paths check this cache first, populating it on first read.
    /// Subsequent reads (from IR compiler, bundle_pass, graph_pass) are
    /// O(1) lookups. Files are stored as `Arc<String>` to avoid clones.
    /// P0-3: Cache entries include mtime and size for invalidation.
    pub source_cache: Mutex<HashMap<String, CacheEntry>>,
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
    /// Exact durable persistence path owned by each session-local IR alias.
    persisted_paths: Mutex<HashMap<String, String>>,
    context_fidelities: Mutex<HashMap<String, Fidelity>>,
    semantic_edge_snapshots:
        Mutex<HashMap<String, Vec<crate::layers::meta::semantic::SemanticEdge>>>,
    pending_semantic_transitions: Mutex<
        HashMap<
            durable_semantics::PendingTransitionKey,
            durable_semantics::PendingSemanticTransition,
        >,
    >,

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

    /// Rendered LLM hierarchy cache keyed by path alias.
    pub llm_text_cache: Mutex<HashMap<String, String>>,

    /// Cross-file semantic index populated by production compilation.
    pub workspace_index: RwLock<crate::workspace::index::WorkspaceIndex>,

    /// Session-scoped record of completed hydration discovery.
    pub(crate) hydration_discovery: Mutex<HydrationDiscoveryCache>,

    /// Phase 2: Proxy port for fetching tool-filtering and cache stats.
    /// Defaults to 8787 (the proxy's default port).
    pub proxy_port: u16,

    /// Layer registry for language/meta-layer dispatch.
    /// Initialized once at startup from the enabled Cargo features.
    pub registry: LayerRegistry,

    /// A-04: Metrics registry for operational signals.
    /// Provides counters, histograms, and gauges for key metrics.
    /// Thread-safe; can be shared across the server via `&MetricsRegistry`.
    pub metrics_registry: std::sync::Arc<crate::observability::MetricsRegistry>,

    /// Auto-started proxy child process handle.
    /// `Some(child)` when `proxy.auto_start` is enabled and the proxy
    /// was successfully spawned; `None` otherwise. Terminated in
    /// `shutdown_proxy` when the MCP server exits.
    pub proxy_child: Mutex<Option<std::process::Child>>,

    /// Cache configuration reported by the proxy's `GET /cache/state`
    /// endpoint (Phase 5 cache-hint transport). `None` when the proxy
    /// is not running or the endpoint is unavailable. Populated at MCP
    /// server startup; used to align the MCP-side `_meta.cache_hints`
    /// breakers with the proxy's actual injection behavior.
    pub proxy_cache: Option<crate::proxy_spawner::ProxyCacheStateInfo>,
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

        // Capture config-derived values *before* moving config into Self
        // (avoiding after-move borrows).
        let cbm_config = config.cbm.clone();
        let project_root = crate::mcp::server::find_project_root().clone();
        let (graph_bridge, cbm_status) =
            Self::init_cbm_bridge(&cbm_config, &project_root, &config.additional_roots);
        let proxy_port = config.proxy.port;

        Self {
            dict: Mutex::new(PathDictionary::new()),
            cache: RwLock::new(LocalStateCache::new()),
            config,
            ir_context: RwLock::new(ContextState::new()),
            source_cache: Mutex::new(HashMap::new()),
            warnings: Mutex::new(Vec::new()),
            session_stats: Mutex::new(session_stats),
            context_store: InMemoryContextStore::new(),
            persistence_store: Mutex::new(persistence_store),
            persisted_paths: Mutex::new(HashMap::new()),
            context_fidelities: Mutex::new(HashMap::new()),
            semantic_edge_snapshots: Mutex::new(HashMap::new()),
            pending_semantic_transitions: Mutex::new(HashMap::new()),
            llm_text_cache: Mutex::new(HashMap::new()),
            emitted_breakpoints: Mutex::new(HashSet::new()),
            cache_metrics: Mutex::new(CacheMetrics::default()),
            workspace_index: RwLock::new(crate::workspace::index::WorkspaceIndex::new()),
            hydration_discovery: Mutex::new(HydrationDiscoveryCache::new()),
            cbm_filter: Mutex::new(CbmFilterState::default()),
            graph_bridge: Mutex::new(graph_bridge),
            cbm_status,
            proxy_port,
            registry: LayerRegistry::new(),
            metrics_registry: std::sync::Arc::new(crate::observability::MetricsRegistry::new()),
            proxy_child: Mutex::new(None),
            proxy_cache: None,
        }
    }

    /// Try to initialize the CBM graph bridge.
    ///
    /// Resolves the disk-cache DB path from config scope:
    ///   1. `cache_db_path` (explicit override) if set
    ///   2. `PerWorkspace` → `<project_root>/.clean-ctx/cbm-graph-cache.db`
    ///   3. `Global` (default) → `.clean-ctx/cbm-graph-cache.db`
    ///
    /// The `GraphCacheStore` is attached to the bridge so query results
    /// are hydrated from disk on first touch (avoiding CBM re-indexing
    /// on restart) and written through on insert.
    fn init_cbm_bridge(
        cbm_config: &crate::cbm::CbmConfig,
        project_root: &std::path::Path,
        additional_roots: &[String],
    ) -> (Option<crate::cbm::GraphBridge>, crate::cbm::CbmStatus) {
        // Every configured additional root becomes its own CBM project: one
        // subprocess, one async index per root, each tracked under its own
        // canonical CBM slug (see `GraphBridge::try_create_with_roots`).
        let extras: Vec<std::path::PathBuf> = additional_roots
            .iter()
            .map(std::path::PathBuf::from)
            .collect();
        let mut bridge =
            crate::cbm::GraphBridge::try_create_with_roots(cbm_config, project_root, &extras);

        // Resolve the disk-cache DB path by scope precedence.
        let cache_db_path = cbm_config
            .cache_db_path
            .as_ref()
            .map(std::path::PathBuf::from)
            .or_else(|| match cbm_config.cache_scope {
                crate::cbm::config::CacheScope::Global => {
                    Some(std::path::PathBuf::from(".clean-ctx/cbm-graph-cache.db"))
                }
                crate::cbm::config::CacheScope::PerWorkspace => {
                    Some(project_root.join(".clean-ctx/cbm-graph-cache.db"))
                }
            });

        if let Some(db_path) = cache_db_path {
            match crate::cbm::cache_store::GraphCacheStore::open(&db_path) {
                Ok(store) => {
                    eprintln!("[clean-ctx] CBM graph cache: {}", db_path.display());
                    bridge.attach_disk_cache(store);
                }
                Err(e) => {
                    eprintln!("[clean-ctx] WARNING: Failed to open CBM graph cache DB: {e}");
                }
            }
        }

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

    // ── P1-5: All lock accessors use lock_or_recover! macro ────────

    /// Lock the path dictionary for mutation.
    pub fn dict_lock(&self) -> std::sync::MutexGuard<'_, PathDictionary> {
        lock_or_recover!(self.dict.lock(), "dict")
    }

    /// Lock the cache for reading.
    pub fn cache_read(&self) -> std::sync::RwLockReadGuard<'_, LocalStateCache> {
        lock_or_recover!(self.cache.read(), "cache")
    }

    /// Lock the cache for writing.
    pub fn cache_write(&self) -> std::sync::RwLockWriteGuard<'_, LocalStateCache> {
        lock_or_recover!(self.cache.write(), "cache")
    }

    /// Lock the IR context for reading.
    pub fn ir_context_read(&self) -> std::sync::RwLockReadGuard<'_, ContextState> {
        lock_or_recover!(self.ir_context.read(), "ir_context")
    }

    /// Lock the IR context for writing.
    pub fn ir_context_lock(&self) -> std::sync::RwLockWriteGuard<'_, ContextState> {
        lock_or_recover!(self.ir_context.write(), "ir_context")
    }

    /// Lock the workspace index for reading.
    pub fn workspace_index_read(
        &self,
    ) -> std::sync::RwLockReadGuard<'_, crate::workspace::index::WorkspaceIndex> {
        lock_or_recover!(self.workspace_index.read(), "workspace_index")
    }

    /// Lock the workspace index for writing.
    pub fn workspace_index_lock(
        &self,
    ) -> std::sync::RwLockWriteGuard<'_, crate::workspace::index::WorkspaceIndex> {
        lock_or_recover!(self.workspace_index.write(), "workspace_index")
    }
    /// Lock the session stats for writing.
    pub fn session_stats_lock(&self) -> std::sync::MutexGuard<'_, SessionStats> {
        lock_or_recover!(self.session_stats.lock(), "session_stats")
    }

    /// Lock the source cache for writing.
    pub fn source_cache_lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, CacheEntry>> {
        lock_or_recover!(self.source_cache.lock(), "source_cache")
    }

    /// Lock the CBM filter state for writing.
    pub fn cbm_filter_lock(&self) -> std::sync::MutexGuard<'_, CbmFilterState> {
        lock_or_recover!(self.cbm_filter.lock(), "cbm_filter")
    }

    /// Lock the LLM text cache for writing.
    pub fn llm_text_cache_lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, String>> {
        lock_or_recover!(self.llm_text_cache.lock(), "llm_text_cache")
    }

    /// Lock the graph bridge for mutation.
    pub fn graph_bridge_lock(&self) -> std::sync::MutexGuard<'_, Option<crate::cbm::GraphBridge>> {
        lock_or_recover!(self.graph_bridge.lock(), "graph_bridge")
    }

    /// Lock the persistence store for writing.
    pub fn persistence_store_lock(&self) -> std::sync::MutexGuard<'_, Option<BufferedStore>> {
        lock_or_recover!(self.persistence_store.lock(), "persistence_store")
    }

    /// Lock the emitted breakpoints for writing.
    pub fn emitted_breakpoints_lock(&self) -> std::sync::MutexGuard<'_, HashSet<String>> {
        lock_or_recover!(self.emitted_breakpoints.lock(), "emitted_breakpoints")
    }

    /// Lock the cache metrics for writing.
    pub fn cache_metrics_lock(&self) -> std::sync::MutexGuard<'_, CacheMetrics> {
        lock_or_recover!(self.cache_metrics.lock(), "cache_metrics")
    }

    /// Lock the auto-started proxy child handle for mutation.
    pub fn proxy_child_lock(&self) -> std::sync::MutexGuard<'_, Option<std::process::Child>> {
        lock_or_recover!(self.proxy_child.lock(), "proxy_child")
    }

    /// Get or create a path alias (thread-safe convenience method).
    pub fn get_or_create_alias(&self, path: String) -> String {
        self.dict_lock().get_or_create_alias(path)
    }

    /// Resolve a session-local file alias to its durable canonical path.
    pub fn path_for_alias(&self, alias: &str) -> Option<String> {
        self.dict_lock().path_for_alias(alias).map(str::to_owned)
    }

    pub fn alias_for_path(&self, path: &str) -> Option<String> {
        self.dict_lock().alias_for_path(path).map(str::to_owned)
    }

    pub fn remember_context_fidelity(&self, alias: &str, fidelity: Fidelity) {
        lock_or_recover!(self.context_fidelities.lock(), "context_fidelities")
            .insert(alias.to_string(), fidelity);
    }

    pub fn context_fidelity(&self, alias: &str) -> Option<Fidelity> {
        lock_or_recover!(self.context_fidelities.lock(), "context_fidelities")
            .get(alias)
            .copied()
    }

    /// Get or create a bundle alias (thread-safe convenience method).
    pub fn get_or_create_bundle_alias(&self, component_name: String) -> String {
        self.dict_lock().get_or_create_bundle_alias(component_name)
    }

    /// Format the dictionary footer (thread-safe convenience method).
    pub fn format_dict_footer(&self) -> String {
        self.dict_lock().format_footer()
    }

    /// Format a request-scoped PATHMAP containing only the listed aliases
    /// (thread-safe convenience method).
    pub fn format_dict_footer_for_aliases(&self, required_aliases: &[&str]) -> String {
        self.dict_lock().format_footer_for_aliases(required_aliases)
    }

    #[allow(clippy::too_many_arguments)]
    /// Record compression stats (thread-safe convenience method).
    pub fn record_compression(
        &self,
        file_path: &str,
        raw_tokens: usize,
        compressed_tokens: usize,
        fidelity: &str,
        is_angular: bool,
        source: &str,
        full_compressed_tokens: Option<usize>,
        domain: &str,
    ) {
        self.session_stats_lock().record_compression(
            file_path,
            raw_tokens,
            compressed_tokens,
            fidelity,
            is_angular,
            source,
            full_compressed_tokens,
            domain,
        );
    }

    /// Record a CBM pipe-level proxy compression event (thread-safe convenience
    /// method). Each CBM interception call ACCUMULATES into session stats.
    pub fn record_cbm_proxy(&self, tool_name: &str, raw_tokens: usize, compressed_tokens: usize) {
        self.session_stats_lock()
            .record_cbm_proxy(tool_name, raw_tokens, compressed_tokens);
    }

    /// Get file version from IR context (thread-safe convenience method).
    pub fn file_version(&self, path_alias: &str) -> Option<u64> {
        let g = lock_or_recover!(self.ir_context.read(), "ir_context");
        g.file_version(path_alias)
    }

    /// Lock the hydration discovery cache (session-scoped record of
    /// *completed hydration discovery*).
    ///
    /// `workspace_index_read` answers "what is compiled"; this cache answers
    /// "what has already been searched" for one (project/root, discovery mode,
    /// entity name) under the current workspace generation — see
    /// `crate::mcp::discovery_cache`. The guard is held only for the
    /// check/mark itself, never across CBM calls, filesystem scans, source
    /// reads, or compilation.
    ///
    /// Crate-visible with its type: session-internal state, not public API.
    pub(crate) fn hydration_discovery_lock(
        &self,
    ) -> std::sync::MutexGuard<'_, HydrationDiscoveryCache> {
        lock_or_recover!(self.hydration_discovery.lock(), "hydration_discovery")
    }

    /// Drop every completed hydration discovery record.
    ///
    /// Used when a workspace change cannot be attributed to one configured
    /// root (an externally modified file detected through the source-cache
    /// mtime/size path). Scope-specific invalidation goes through
    /// `invalidate_root` on the guard instead.
    pub(crate) fn invalidate_hydration_discovery_all(&self) {
        self.hydration_discovery_lock().invalidate_all();
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
        lock_or_recover!(self.warnings.lock(), "warnings").push(msg.into());
    }

    /// F-FINAL-06: Drain all accumulated warnings. Returns a `Vec`
    /// that the caller embeds in the response's `_warnings` field.
    pub fn drain_warnings(&self) -> Vec<String> {
        std::mem::take(&mut *lock_or_recover!(self.warnings.lock(), "warnings"))
    }
}
