// src/cbm/bridge.rs
//
// Graph Bridge — translates CBM graph data into Clean-CTX concepts.
// Entirely self-contained with its own types and caching.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cbm::cache_store::GraphCacheStore;
use crate::cbm::client::{CbmClient, CbmError};
use crate::cbm::config::CbmConfig;
use crate::cbm::config::CbmStatus;

// ── Public types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolImportance {
    pub symbol: String,
    pub score: f64,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedSymbol {
    pub file: String,
    pub symbol: String,
    pub change_type: String,
    pub impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureOverview {
    pub modules: Vec<ArchitectureModule>,
    pub dependencies: Vec<ArchitectureDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureModule {
    pub name: String,
    pub path: String,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureDependency {
    pub from: String,
    pub to: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeEntry {
    pub symbol: String,
    pub file: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub name: String,
    pub file: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub changes: Vec<AffectedSymbol>,
    pub graph_version: String,
}

// ── Internal cache ──────────────────────────────────────────────

pub(crate) struct CachedGraphData {
    pub(crate) data: Value,
    pub(crate) expires_at: Instant,
}

// ── P1-9: Non-blocking indexing state machine ───────────────────

/// Tracks the state of project indexing for the CBM graph bridge.
///
/// P1-9: Indexing is now non-blocking — `ensure_indexed()` returns
/// immediately with `StillIndexing` if indexing is in progress, and
/// a background thread handles the actual pipe I/O.
#[derive(Debug, Clone)]
pub enum IndexingState {
    /// No indexing has been attempted yet.
    NotStarted,
    /// Indexing is in progress, started at the given instant.
    InProgress { started_at: Instant },
    /// Indexing completed successfully.
    Complete,
    /// Indexing failed with an error message.
    Failed(String),
}

/// Returned by `ensure_indexed()` indicating whether the caller can
/// proceed with a CBM query or should retry later.
#[derive(Debug, Clone, PartialEq)]
pub enum IndexingStatus {
    /// Project is indexed and ready for queries.
    Ready,
    /// Indexing is in progress (retry later).
    StillIndexing { elapsed_secs: u64 },
}

/// Graph bridge with TTL caching and graceful degradation.
pub struct GraphBridge {
    /// CBM subprocess client, wrapped in Arc<Mutex<>> so the background
    /// indexing thread can access it without blocking the main bridge.
    /// P1-9: Changed from `Option<CbmClient>` to allow spawning the
    /// indexing thread while sharing the client handle.
    pub(crate) client: Arc<Mutex<Option<CbmClient>>>,
    pub(crate) cache: DashMap<String, CachedGraphData>,
    pub(crate) status: CbmStatus,
    cache_ttl: u64,
    project: Option<String>,
    /// Canonicalized project root. Used as the disk-cache partition key and
    /// to derive the project name. Multi-repo support: each repo gets its
    /// own cache partition and indexing state.
    project_root: PathBuf,
    /// Optional SQLite-backed disk cache. When present, cache entries are
    /// hydrated from disk on first touch (avoiding CBM re-indexing on
    /// restart) and written through on insert.
    disk_cache: Option<GraphCacheStore>,
    graph_version: String,
    /// P1-9: Replaced `indexed: bool` with state machine.
    /// Multi-repo: keyed by project name so switching projects doesn't
    /// corrupt another project's indexing state.
    pub(crate) indexing_state: Arc<Mutex<HashMap<String, IndexingState>>>,
    /// Authoritative mapping: canonical repository root → CBM project ID.
    ///
    /// CBM derives a project's identity from the canonical repo path (see
    /// `cbm_project_slug`), NOT from the directory basename. This map holds
    /// the primary root plus every configured additional root, so queries,
    /// readiness checks, and proxy calls always use the same CBM identity.
    pub(crate) project_ids: HashMap<PathBuf, String>,
    /// Inverse of `project_ids`: CBM project ID → canonical repository root.
    /// Used to resolve the `repo_path` when (re)indexing a specific project.
    pub(crate) project_paths: HashMap<String, PathBuf>,

    /// Last error from the most recent user-facing graph query
    /// (`search`, `trace_path`, `query_graph`, `get_architecture`).
    ///
    /// These methods historically returned empty/default results on failure
    /// (e.g. "indexing in progress"), which made the graph tools report
    /// "0 nodes, 0 edges" — a confidently wrong answer. The handlers now
    /// check this after each call via `take_last_error()` and surface the
    /// error to the agent instead of the empty result.
    ///
    /// Cleared on every successful query so a stale error is never
    /// re-reported.
    last_error: Option<CbmError>,
}

impl GraphBridge {
    /// Try to discover and launch CBM. Returns a bridge; use `is_available()` to check.
    ///
    /// Binary resolution order:
    ///   1. `config.binary_path` (explicit config)
    ///   2. PATH search for `codebase-memory-mcp`
    ///   3. Common install locations (`~/.cargo/bin`, `/usr/local/bin`, etc.)
    ///
    /// CBM project identity is the canonical path-derived slug (never the
    /// directory basename). Equivalent to `try_create_with_roots(config, root, &[])`.
    pub fn try_create(config: &CbmConfig, project_root: &Path) -> Self {
        Self::try_create_with_roots(config, project_root, &[])
    }

    /// Create a `GraphBridge` for a primary root plus additional roots.
    ///
    /// The authoritative CBM project identity for every root is the canonical
    /// slug CBM derives from the repository path (`cbm_project_slug`) — the
    /// directory basename is never treated as a CBM project ID. Each configured
    /// root is registered in `project_ids`/`project_paths` and, when CBM is
    /// available, begins indexing asynchronously at construction.
    pub fn try_create_with_roots(
        config: &CbmConfig,
        project_root: &Path,
        additional_roots: &[PathBuf],
    ) -> Self {
        let binary_path = resolve_cbm_binary(config);

        // Canonicalize the primary root: CBM identity is path-derived, so the
        // same path must be used everywhere (indexing, readiness, queries).
        let project_root_canon = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());

        // Build the authoritative root → project-id map for this bridge.
        let mut project_ids: HashMap<PathBuf, String> = HashMap::new();
        let mut project_paths: HashMap<String, PathBuf> = HashMap::new();
        insert_cbm_project(&mut project_ids, &mut project_paths, &project_root_canon);
        for extra in additional_roots {
            // Canonicalize lazily; skip roots that don't exist on this machine
            // (mirrors `resolve_file_path_checked`'s tolerant additional_roots).
            if let Ok(extra_canon) = extra.canonicalize() {
                if !project_ids.contains_key(&extra_canon) {
                    insert_cbm_project(&mut project_ids, &mut project_paths, &extra_canon);
                }
            }
        }

        let client = if config.enabled {
            match binary_path {
                Some(path) => match CbmClient::try_launch(
                    &path,
                    Duration::from_millis(config.query_timeout_ms),
                    config.max_retries,
                    config.circuit_cooldown_secs,
                ) {
                    Ok(Some(c)) => {
                        eprintln!("[clean-ctx-cbm] Launched from: {}", path.display());
                        Some(c)
                    }
                    Ok(None) => {
                        eprintln!("[clean-ctx-cbm] Binary not found: {}", path.display());
                        None
                    }
                    Err(e) => {
                        eprintln!("[clean-ctx-cbm] Launch failed: {e}");
                        None
                    }
                },
                None => {
                    eprintln!("[clean-ctx-cbm] Not found on PATH or common locations.");
                    eprintln!("  Install from: https://github.com/DeusData/codebase-memory-mcp");
                    None
                }
            }
        } else {
            None
        };

        let is_available = client.is_some();
        let mut bridge = Self {
            status: if is_available {
                CbmStatus::Available
            } else {
                CbmStatus::Unavailable
            },
            client: Arc::new(Mutex::new(client)),
            cache: DashMap::new(),
            cache_ttl: config.cache_ttl,
            project: project_ids.get(&project_root_canon).cloned(),
            project_root: project_root_canon,
            project_ids,
            project_paths,
            disk_cache: None,
            graph_version: String::new(),
            indexing_state: Arc::new(Mutex::new(HashMap::new())),
            last_error: None,
        };

        // K-1: Start indexing immediately when CBM launched successfully.
        //
        // Previously indexing was deferred until the first CBM-dependent
        // request called `ensure_indexed()`, which blocked/spawned lazily.
        // Worse, the MANUAL construction in this method (via the `Self { }`
        // literal) used to pre-seed `indexing_state` to `Complete`, hiding the
        // cold-start problem: on a fresh session with a warm disk cache the
        // first graph query would still return empty results until indexing
        // finished. Now the background indexer begins as soon as the bridge is
        // constructed, so by the time a request arrives the graph is ready (or
        // `ensure_indexed()` reports `StillIndexing` and the agent retries).
        if is_available {
            bridge.start_indexing_roots();
        }

        bridge
    }

    /// Attach a disk cache store to this bridge. Called by `McpState` after
    /// resolving the cache DB path from config scope.
    pub fn attach_disk_cache(&mut self, store: GraphCacheStore) {
        self.disk_cache = Some(store);
    }

    /// Switch the active project by name. Also clears the in-memory cache
    /// so cached results from the previous project are never served to the
    /// new project (the disk cache is project-partitioned and remains).
    pub fn set_project(&mut self, project: &str) {
        // Resolve the requested identity to CBM's canonical slug (path, known
        // root basename, or literal slug). A raw dirname must never become a
        // divergent CBM project ID.
        let resolved = self.resolve_project_id(project);
        let changed = self.project.as_deref() != Some(resolved.as_str());
        self.project = Some(resolved.clone());
        if changed {
            self.cache.clear();
        }
        self.ensure_tracked(&resolved);
    }

    /// Switch the active workspace (multi-repo support).
    ///
    /// Updates both the canonicalized `project_root` (the disk-cache
    /// partition key) and the derived project name, then clears the
    /// in-memory cache. This ensures memory AND disk caches are scoped
    /// to the correct repo when a handler passes `workspaceRoot`.
    ///
    /// Only the switched-to project's indexing state is reset (so it
    /// re-indexes on first query). Other projects' states are preserved,
    /// avoiding unnecessary re-indexing when bouncing between repos.
    pub fn set_workspace_root(&mut self, root: &Path) {
        let root_canon = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        if self.project_root == root_canon {
            return;
        }
        // Resolve the new root's canonical CBM slug: known configured root → its
        // canonical project ID; otherwise derive + register on demand (multi-repo).
        let slug = if let Some(s) = self.project_ids.get(&root_canon) {
            s.clone()
        } else {
            insert_cbm_project(&mut self.project_ids, &mut self.project_paths, &root_canon)
        };
        self.project_root = root_canon;
        self.project = Some(slug.clone());
        self.cache.clear();
        // A root introduced at runtime starts indexing immediately when usable.
        self.ensure_tracked(&slug);
    }
    pub fn status(&self) -> &CbmStatus {
        &self.status
    }
    /// Check if the bridge can serve data.
    ///
    /// Returns true when:
    ///   - A real CBM client is available AND status is Available, OR
    ///   - The cache has pre-seeded entries (mock/test mode) AND status is Available
    ///
    /// P0-2: Previously required `self.client.is_some()` which broke the mock —
    /// tests using `new_mock()` pre-seed the cache but set client to None.
    /// Now `is_available()` also returns true when cached data exists, allowing
    /// mocks to serve pre-seeded data without a real CBM binary.
    pub fn is_available(&self) -> bool {
        if !self.status.is_available() {
            return false;
        }
        // Real client OR pre-seeded cache (mock mode)
        self.client
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_some()
            || !self.cache.is_empty()
    }
    pub fn graph_version(&self) -> &str {
        &self.graph_version
    }
    pub fn set_graph_version(&mut self, v: &str) {
        self.graph_version = v.to_string();
    }

    /// Take (and clear) the last error from a user-facing graph query.
    ///
    /// The `search`, `trace_path`, `query_graph`, and `get_architecture`
    /// methods return empty/default results on failure for backward
    /// compatibility with the intelligence layer. The MCP handlers call
    /// this after each query: when it returns `Some`, they respond with
    /// the error message instead of the confident "0 nodes, 0 edges".
    pub fn take_last_error(&mut self) -> Option<CbmError> {
        self.last_error.take()
    }

    /// Record a query error for `take_last_error()`, or clear it on success.
    fn set_last_error(&mut self, err: Option<CbmError>) {
        self.last_error = err;
    }

    /// Test-only: inject a stale error directly so tests can verify the
    /// cache-hit path clears it.
    #[cfg(test)]
    pub fn set_last_error_for_test(&mut self, err: CbmError) {
        self.set_last_error(Some(err));
    }

    /// Trigger indexing of the current project in CBM.
    /// Called automatically on startup when CBM is available.
    /// Returns Ok(()) if indexing was triggered successfully, or Err if CBM is unavailable.
    ///
    /// K-1: Indexing is now started by `start_indexing()` in a background thread
    /// at bridge construction. This method is kept as a public API for manual
    /// re-indexing (e.g. after `set_project` or `invalidate_cache`).
    pub fn index_repository(&mut self) -> Result<(), CbmError> {
        if !self.is_available() {
            return Err(CbmError::LaunchError("CBM not available".into()));
        }
        let project = self.project_str();
        eprintln!("[clean-ctx-cbm] Indexing project: {project}");
        // Call CBM's index_repository tool to trigger indexing
        let repo_path = self.project_root.to_string_lossy().to_string();
        let client_guard = self.client.lock().unwrap_or_else(|p| p.into_inner());
        let _result = match client_guard.as_ref() {
            Some(_c) => {
                // We need mut access — drop guard, reacquire as mut
                drop(client_guard);
                let mut cg = self.client.lock().unwrap_or_else(|p| p.into_inner());
                let client = cg.as_mut().unwrap();
                client.call_tool(
                    "index_repository",
                    serde_json::json!({"repo_path": repo_path, "mode": "fast"}),
                )
            }
            None => return Err(CbmError::LaunchError("CBM not available".into())),
        }?;
        eprintln!("[clean-ctx-cbm] Project indexed successfully");
        Ok(())
    }

    /// Start project indexing in a background thread.
    ///
    /// K-1: Extracted from `ensure_indexed()` so indexing can begin at bridge
    /// construction (`try_create`) rather than lazily on the first query.
    ///
    /// Marks this project's state `InProgress` and spawns a dedicated thread
    /// that performs the actual `index_repository(repo_path, "fast")` pipe I/O,
    /// then transitions the state to `Complete` or `Failed`. The thread flips
    /// the circuit breaker via `record_failure()` on error.
    fn start_indexing_for(&mut self, project: &str) {
        let project_owned = project.to_string();
        let repo_path = self
            .project_paths
            .get(project)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.project_root.to_string_lossy().into_owned());
        let project_for_spawn = project_owned.clone();

        // Mark as in-progress before spawning (so concurrent calls see it).
        {
            let mut states = self
                .indexing_state
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            let state = states
                .entry(project_owned.clone())
                .or_insert(IndexingState::NotStarted);
            *state = IndexingState::InProgress {
                started_at: Instant::now(),
            };
        }

        // Clone Arc handles for the background thread.
        let client_arc = Arc::clone(&self.client);
        let state_arc = Arc::clone(&self.indexing_state);
        let _status = self.status.clone();

        // Spawn background indexing thread.
        std::thread::Builder::new()
            .name("cbm-indexer".into())
            .spawn(move || {
                eprintln!("[clean-ctx-cbm] Background indexing started for: {project_for_spawn}");
                let result = {
                    let mut client_guard = match client_arc.lock() {
                        Ok(g) => g,
                        Err(poisoned) => {
                            eprintln!(
                                "[clean-ctx-cbm] WARNING: Recovering from poisoned client lock"
                            );
                            poisoned.into_inner()
                        }
                    };
                    match client_guard.as_mut() {
                        Some(client) => {
                            match client.call_tool(
                                "index_repository",
                                serde_json::json!({"repo_path": repo_path, "mode": "fast"}),
                            ) {
                                Ok(_) => {
                                    eprintln!("[clean-ctx-cbm] Project indexed successfully");
                                    Ok(())
                                }
                                Err(e) => {
                                    eprintln!("[clean-ctx-cbm] Indexing failed: {e}");
                                    Err(e)
                                }
                            }
                        }
                        None => Err(CbmError::LaunchError("CBM not available".into())),
                    }
                };

                let mut states = match state_arc.lock() {
                    Ok(g) => g,
                    Err(poisoned) => {
                        eprintln!("[clean-ctx-cbm] WARNING: Recovering from poisoned state lock");
                        poisoned.into_inner()
                    }
                };
                let s = states
                    .entry(project_for_spawn.clone())
                    .or_insert(IndexingState::NotStarted);
                match result {
                    Ok(()) => {
                        *s = IndexingState::Complete;
                    }
                    Err(e) => {
                        // Record failure for circuit breaker.
                        if let Ok(mut cg) = client_arc.lock() {
                            if let Some(ref mut c) = *cg {
                                c.record_failure();
                            }
                        }
                        *s = IndexingState::Failed(e.to_string());
                    }
                }
            })
            .ok();
    }
    /// Start indexing for every configured root (primary + additional).
    ///
    /// Called from `try_create_with_roots()` when CBM is available, so all
    /// roots begin indexing immediately at construction. Each root is tracked
    /// under its own canonical CBM slug; one root remaining `StillIndexing`
    /// never blocks another root that is already `Complete`.
    pub(crate) fn start_indexing_roots(&mut self) {
        let slugs: Vec<String> = self.project_ids.values().cloned().collect();
        let mut started: Vec<String> = Vec::new();
        for slug in &slugs {
            if !started.contains(slug) {
                self.start_indexing_for(slug);
                started.push(slug.clone());
            }
        }
    }

    /// Ensure the project is indexed before issuing queries.
    ///
    /// K-1: **Report-only rewrite. Indexing is started at bridge construction
    /// (`try_create` → `start_indexing`), so this method no longer spawns a
    /// background indexing thread.** It only reports the current state:
    ///
    /// - `InProgress` → `StillIndexing` (retry later); times out after 60s.
    /// - `Complete`   → `Ready`.
    /// - `Failed`     → `Err`.
    ///
    /// `NotStarted` is treated as freshly-launched so the caller retries; this
    /// avoids duplicating the construction-time spawn while never blocking.
    pub fn ensure_indexed(&mut self) -> Result<IndexingStatus, CbmError> {
        // Guard before touching any state: when CBM is unavailable (disabled,
        // binary missing, launch failed), return the same error on every call.
        // (AUDIT-9 regression: previously the first call could spawn a doomed
        // background thread when unavailable.)
        if !self.is_available() {
            return Err(CbmError::LaunchError("CBM not available".into()));
        }
        let project = self.project_str();
        let mut states = self
            .indexing_state
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let state = states
            .entry(project.clone())
            .or_insert(IndexingState::NotStarted);

        match state {
            IndexingState::Complete => Ok(IndexingStatus::Ready),
            IndexingState::InProgress { started_at } => {
                let elapsed = started_at.elapsed().as_secs();
                if elapsed > 60 {
                    *state = IndexingState::Failed("indexing timed out after 60s".into());
                    Err(CbmError::LaunchError("indexing timed out".into()))
                } else {
                    Ok(IndexingStatus::StillIndexing {
                        elapsed_secs: elapsed,
                    })
                }
            }
            IndexingState::Failed(msg) => Err(CbmError::LaunchError(msg.clone())),
            // Construction-time `start_indexing()` marks the state InProgress
            // before the thread runs; if we observe NotStarted (e.g. the thread
            // hasn't flipped state yet), report as freshly started so the caller
            // retries instead of blocking or double-spawning.
            IndexingState::NotStarted => Ok(IndexingStatus::StillIndexing { elapsed_secs: 0 }),
        }
    }

    /// Resolve a caller-supplied project reference to the canonical CBM project slug.
    ///
    /// Resolution order:
    ///   1. If `raw` looks like a path (or canonicalizes to an existing path),
    ///      canonicalize and map through the authoritative root→slug map; derive+register on miss.
    ///   2. If `raw` is a known project slug → it is used as-is.
    ///   3. If `raw` matches a configured root's directory basename → that root's canonical slug.
    ///   4. Otherwise the literal string is treated as a CBM slug (CBM itself will
    ///      authoritatively reject it if it has no such project).
    pub fn resolve_project_id(&self, raw: &str) -> String {
        let trimmed = raw.trim();

        // 1. Path-like (Windows/Linux separators or a canonicalizable existing path).
        if trimmed.contains('/')
            || trimmed.contains('\\')
            || trimmed.contains(':')
            || Path::new(trimmed).canonicalize().is_ok()
        {
            let canon = Path::new(trimmed)
                .canonicalize()
                .unwrap_or_else(|_| Path::new(trimmed).to_path_buf());
            if let Some(slug) = self.project_ids.get(&canon) {
                return slug.clone();
            }
            // Unknown path: derive its canonical slug WITHOUT registering it —
            // this method is identity-resolution only (and takes `&self` so the
            // proxy can call it). Registration happens in `set_workspace_root`.
            return cbm_project_slug(&canon);
        }

        // 2. Literal known slug.
        if self.project_paths.contains_key(trimmed) {
            return trimmed.to_string();
        }

        // 3. A configured root's basename (e.g. "RustContextLayerAI" → primary slug).
        for (root, slug) in &self.project_ids {
            if root.file_name().map(|n| n.to_string_lossy().into_owned())
                == Some(trimmed.to_string())
            {
                return slug.clone();
            }
        }

        // 4. Literal fallback.
        trimmed.to_string()
    }

    /// Ensure a known root is tracked (has an indexing entry). If a root is
    /// introduced at runtime with no entry, indexing starts for it when available.
    fn ensure_tracked(&mut self, slug: &str) {
        if self.project_paths.contains_key(slug) {
            let already_gated = self.indexing_state().contains_key(slug);
            if !already_gated && self.is_available() {
                self.start_indexing_for(slug);
            }
        }
    }
    /// Report the indexing state of a specific CBM project.
    ///
    /// Used by `cbm_proxy` so readiness is resolved against the project actually
    /// being queried. Semantics:
    ///   - The active project delegates to `ensure_indexed()`.
    ///   - A project that is NOT tracked (unknown slug) passes through as
    ///     `Ready` — CBM returns its authoritative error if the project doesn't
    ///     exist, so an unrelated/unknown project can never dead-end in
    ///     `StillIndexing{0}` forever.
    ///   - A tracked root reports its own per-project state (`Complete`→`Ready`,
    ///     `InProgress`→`StillIndexing`, `Failed`→`Err`, `NotStarted`→`StillIndexing{0}`).
    pub fn ensure_indexed_for(&mut self, project: &str) -> Result<IndexingStatus, CbmError> {
        // Guard before touching any state: when CBM is unavailable, return the
        // same error on every call (AUDIT-9 regression).
        if !self.is_available() {
            return Err(CbmError::LaunchError("CBM not available".into()));
        }
        if self.project.as_deref() != Some(project) {
            // Not the active project. Unknown/untracked projects pass through.
            if !self.project_paths.contains_key(project) {
                return Ok(IndexingStatus::Ready);
            }
            let mut states = self
                .indexing_state
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            let state = states
                .entry(project.to_string())
                .or_insert(IndexingState::NotStarted);
            return match state {
                IndexingState::Complete => Ok(IndexingStatus::Ready),
                IndexingState::InProgress { started_at } => {
                    let elapsed = started_at.elapsed().as_secs();
                    if elapsed > 60 {
                        *state = IndexingState::Failed("indexing timed out after 60s".into());
                        Err(CbmError::LaunchError("indexing timed out".into()))
                    } else {
                        Ok(IndexingStatus::StillIndexing {
                            elapsed_secs: elapsed,
                        })
                    }
                }
                IndexingState::Failed(msg) => Err(CbmError::LaunchError(msg.clone())),
                IndexingState::NotStarted => Ok(IndexingStatus::StillIndexing { elapsed_secs: 0 }),
            };
        }
        // Active project → the active gate.
        self.ensure_indexed()
    }
    /// Access the indexing state map for inspection (e.g., get_cbm_status handler).
    pub fn indexing_state(&self) -> std::sync::MutexGuard<'_, HashMap<String, IndexingState>> {
        self.indexing_state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
    }

    /// Resolve the ACTIVE project's canonical CBM slug.
    ///
    /// Since the canonical-identity fix, this is always a CBM-canonical path
    /// slug (never a directory basename). Used as:
    ///   - the `project` argument on every CBM tool call,
    ///   - the `indexing_state` key for readiness gating,
    ///   - the disk-cache partition label.
    pub(crate) fn project_str(&self) -> String {
        self.project
            .clone()
            .unwrap_or_else(|| "default".to_string())
    }

    pub fn get_symbol_importance_mut(&mut self) -> HashMap<String, SymbolImportance> {
        let key = "symbol_importance".to_string();
        let project = self.project_str();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
        }
        let result = self.query(move |c| c.get_symbol_importance(&project, Some(1)));
        match result {
            Ok(symbols) => {
                let map: HashMap<_, _> = symbols
                    .iter()
                    .filter_map(|e| {
                        Some((
                            e["name"].as_str()?.to_string(),
                            SymbolImportance {
                                symbol: e["name"].as_str()?.to_string(),
                                score: e["importance"].as_f64().unwrap_or(0.0),
                                file: e["file"].as_str().unwrap_or("").to_string(),
                            },
                        ))
                    })
                    .collect();
                self.cache_insert(&key, &map);
                map
            }
            Err(_) => HashMap::new(),
        }
    }

    /// Blast radius query at depth 1. Uses CBM Cypher to find callers of a symbol.
    ///
    /// H-01/M-04 fix: Replaced invalid `search_graph` name_pattern with valid Cypher
    /// query_graph call. Previous code passed `"depends_on:{sym}"` as a name_pattern
    /// regex, which CBM treated literally and returned zero matches.
    pub fn get_blast_radius(&mut self, symbol: &str, _depth: usize) -> Vec<String> {
        let key = format!("blast:{symbol}");
        let project = self.project_str();
        let sym = symbol.to_string();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
        }
        let escaped = sym.replace('\'', "\\'");
        let cypher = format!(
            "MATCH (caller:Function)-[:CALLS]->(f:Function) WHERE f.name = '{escaped}' RETURN caller.name, caller.file_path"
        );
        let result = self.query(move |c| c.query_graph(&cypher, &project));
        match result {
            Ok(rows) => {
                let files: Vec<_> = rows
                    .iter()
                    .filter_map(|r| r.get(1).and_then(|v| v.as_str().map(String::from)))
                    .collect();
                self.cache_insert(&key, &files);
                files
            }
            Err(_) => vec![],
        }
    }

    pub fn get_dead_code(&mut self) -> Vec<DeadCodeEntry> {
        let key = "dead_code".to_string();
        let project = self.project_str();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
        }
        let result = self.query(move |c| c.get_dead_code(&project));
        match result {
            Ok(entries) => {
                let dead: Vec<DeadCodeEntry> = entries
                    .iter()
                    .filter_map(|e| {
                        Some(DeadCodeEntry {
                            symbol: e["name"].as_str()?.to_string(),
                            file: e["file"].as_str().unwrap_or("").to_string(),
                            reason: e["reason"].as_str().unwrap_or("unknown").to_string(),
                        })
                    })
                    .collect();
                self.cache_insert(&key, &dead);
                dead
            }
            Err(_) => vec![],
        }
    }

    /// Get all CALLS edges from CBM's knowledge graph.
    /// Returns `(caller, callee)` pairs across all files.
    ///
    /// R-43b Phase 3: Consumed by `InferenceLayer::enrich_from_cbm()` to
    /// populate cross-file call edges (confidence = 0.75). Cached with TTL.
    pub fn get_call_edges(&mut self) -> Vec<(String, String)> {
        let key = "call_edges".to_string();
        let project = self.project_str();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
        }
        let cypher = "MATCH (a:Function)-[:CALLS]->(b:Function) RETURN a.name, b.name".to_string();
        let result = self.query(move |c| c.query_graph(&cypher, &project));
        match result {
            Ok(rows) => {
                let edges: Vec<(String, String)> = rows
                    .iter()
                    .filter_map(|r| {
                        let from = r.first()?.as_str()?.to_string();
                        let to = r.get(1)?.as_str()?.to_string();
                        Some((from, to))
                    })
                    .collect();
                self.cache_insert(&key, &edges);
                edges
            }
            Err(_) => vec![],
        }
    }

    /// Get all dataflow edges from CBM's knowledge graph.
    /// Returns `(method, target, direction)` triples where direction is
    /// "reads" or "writes".
    ///
    /// R-43b Phase 3: Consumed by `InferenceLayer::enrich_from_cbm()` to
    /// populate cross-file dataflow edges (confidence = 0.75). Cached with TTL.
    pub fn get_dataflow_edges(&mut self) -> Vec<(String, String, String)> {
        let key = "dataflow_edges".to_string();
        let project = self.project_str();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
        }
        let cypher =
            "MATCH (m:Function)-[r:DATAFLOW]->(t) RETURN m.name, t.name, type(r)".to_string();
        let result = self.query(move |c| c.query_graph(&cypher, &project));
        match result {
            Ok(rows) => {
                let edges: Vec<(String, String, String)> = rows
                    .iter()
                    .filter_map(|r| {
                        let method = r.first()?.as_str()?.to_string();
                        let target = r.get(1)?.as_str()?.to_string();
                        let direction = r.get(2)?.as_str().unwrap_or("reads").to_string();
                        Some((method, target, direction))
                    })
                    .collect();
                self.cache_insert(&key, &edges);
                edges
            }
            Err(_) => vec![],
        }
    }

    pub fn get_architecture(&mut self) -> Option<ArchitectureOverview> {
        let key = "architecture".to_string();
        let project = self.project_str();
        if self.check_cache(&key) {
            // Cache hit is a successful query — clear any stale error so a
            // prior failure is never re-surfaced after a good result.
            let result = serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .ok();
            self.set_last_error(None);
            return result;
        }
        let result = self.query(move |c| c.get_architecture(&project));
        match result {
            Ok(arch) => {
                self.set_last_error(None);
                let modules = arch["modules"]
                    .as_array()
                    .map(|ms| {
                        ms.iter()
                            .filter_map(|m| {
                                Some(ArchitectureModule {
                                    name: m["name"].as_str()?.to_string(),
                                    path: m["path"].as_str().unwrap_or("").to_string(),
                                    file_count: m["file_count"].as_u64().unwrap_or(0) as usize,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let dependencies = arch["dependencies"]
                    .as_array()
                    .map(|ds| {
                        ds.iter()
                            .filter_map(|d| {
                                Some(ArchitectureDependency {
                                    from: d["from"].as_str()?.to_string(),
                                    to: d["to"].as_str()?.to_string(),
                                    kind: d["kind"].as_str().unwrap_or("import").to_string(),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let ov = ArchitectureOverview {
                    modules,
                    dependencies,
                };
                self.cache_insert(&key, &ov);
                Some(ov)
            }
            Err(e) => {
                // Surface the failure so handlers don't report "0 modules,
                // 0 deps" when CBM is still indexing or unavailable.
                self.set_last_error(Some(e));
                None
            }
        }
    }

    /// Resolve a cross-language endpoint for a method name (Angular
    /// Ecosystem Deepening — NgRx effect → .NET controller endpoint).
    ///
    /// Returns `None` when CBM is unavailable or no candidate matches.
    /// Uses the existing `query_graph` Cypher path with TTL in-memory +
    /// disk caching — **no new tool**.
    ///
    /// Returns the best-match as `"{ClassName}.{MethodName}"` (e.g.
    /// `"UserController.GetAll"`); empty/no-match → `None`.
    ///
    /// The Cypher now joins the declaring `Class` node so the returned
    /// endpoint is **Controller-qualified** — the LLM can trace
    /// `Φeffect:loadUsers$ → UserController.GetAll` as a single semantic
    /// chain, not just a bare method name.
    pub fn resolve_cross_language_endpoint(&mut self, method_name: &str) -> Option<String> {
        // Graceful skip when CBM is unavailable — no error, no graph line.
        if !self.is_available() {
            return None;
        }
        let key = format!("endpoint:{method_name}");
        let project = self.project_str();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
        }

        let escaped = method_name.replace('\'', "\\'");
        // Join the declaring Class node (e.g. `UserController`) so the
        // result is `"{Class}.{Method}"`. Prefer Controller classes.
        // We call the client directly (via `self.query`) to get the raw
        // `{columns, rows}` — the generic `query_graph` flattens rows to
        // the first column, discarding the Controller class. We match on
        // the exact method name (case-sensitive) so we never return an
        // arbitrary fuzzy match.
        let cypher = format!(
            "MATCH (c:Class)-[:DECLARES]->(f:Function) \
             WHERE f.name = '{escaped}' AND f.file_path =~ '.*\\.cs$' \
             RETURN f.name, c.name LIMIT 5"
        );
        let rows = self.query(move |c| c.query_graph(&cypher, &project));
        let result: Option<String> = match rows {
            Ok(rows) => {
                // rows are Vec<Vec<Value>>: [f.name, c.name].
                // Prefer a row whose class name contains "Controller".
                let controller_hit = rows.iter().find_map(|row| {
                    let fname = row.first().and_then(|v| v.as_str())?;
                    let cname = row.get(1).and_then(|v| v.as_str())?;
                    if cname.contains("Controller") {
                        Some(format!("{cname}.{fname}"))
                    } else {
                        None
                    }
                });
                controller_hit.or_else(|| {
                    rows.first().and_then(|row| {
                        let fname = row.first().and_then(|v| v.as_str())?;
                        let cname = row.get(1).and_then(|v| v.as_str())?;
                        Some(format!("{cname}.{fname}"))
                    })
                })
            }
            Err(_) => None,
        };

        // Cache (write-through to disk when present).
        self.cache_insert(&key, &result);
        result
    }

    pub fn query_graph(&mut self, cypher: &str) -> QueryResult {
        let key = format!("cypher:{cypher}");
        let project = self.project_str();
        let q = cypher.to_string();
        if self.check_cache(&key) {
            // Cache hit is a successful query — clear any stale error.
            let result = serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or(QueryResult {
                nodes: vec![],
                edges: vec![],
            });
            self.set_last_error(None);
            return result;
        }
        let result = self.query(move |c| c.query_graph(&q, &project));
        match result {
            Ok(rows) => {
                self.set_last_error(None);
                // CBM Cypher returns {columns, rows} — rows are Vec<Vec<Value>>.
                // We try to interpret each row as either node or edge data.
                let mut nodes = vec![];
                let edges = vec![];
                for row in &rows {
                    // First column is typically a name or label
                    if let Some(first) = row.first() {
                        let label = first.as_str().unwrap_or("");
                        nodes.push(GraphNode {
                            id: label.to_string(),
                            label: String::new(),
                            name: label.to_string(),
                            file: String::new(),
                            properties: HashMap::new(),
                        });
                    }
                }
                let r = QueryResult { nodes, edges };
                self.cache_insert(&key, &r);
                r
            }
            Err(e) => {
                self.set_last_error(Some(e));
                QueryResult {
                    nodes: vec![],
                    edges: vec![],
                }
            }
        }
    }

    pub fn search(&mut self, query: &str) -> Vec<GraphNode> {
        let key = format!("search:{query}");
        let project = self.project_str();
        let q = query.to_string();
        if self.check_cache(&key) {
            // Cache hit is a successful query — clear any stale error.
            let result = serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
            self.set_last_error(None);
            return result;
        }
        // H-01 fix: build a proper regex name_pattern from the query string.
        // If the query already contains regex metacharacters (., *, +, [, etc.)
        // use it as-is; otherwise wrap in .*...* for substring matching.
        let has_regex = q.chars().any(|c| {
            matches!(
                c,
                '.' | '*' | '+' | '[' | '(' | '\\' | '^' | '$' | '{' | '|'
            )
        });
        let name_pattern = if has_regex { q } else { format!(".*{q}.*") };
        let result = self.query(move |c| c.search_graph(&name_pattern, &project, Some("Function")));
        match result {
            Ok(nodes) => {
                self.set_last_error(None);
                let gn: Vec<GraphNode> = nodes
                    .iter()
                    .filter_map(|n| {
                        Some(GraphNode {
                            id: n["id"].as_str()?.to_string(),
                            label: n["label"].as_str().unwrap_or("").into(),
                            name: n["name"].as_str()?.to_string(),
                            file: n["file"].as_str().unwrap_or("").into(),
                            properties: HashMap::new(),
                        })
                    })
                    .collect();
                self.cache_insert(&key, &gn);
                gn
            }
            Err(e) => {
                self.set_last_error(Some(e));
                vec![]
            }
        }
    }

    /// Trace a call path between two symbols. If `to` is empty, traces all edges
    /// from `from`. Otherwise post-filters to only include edges reaching `to`.
    ///
    /// M-01 fix: Previously ignored the `to` parameter — now properly filters
    /// results to only include edges reaching the target symbol.
    pub fn trace_path(&mut self, from: &str, to: &str) -> Vec<GraphEdge> {
        if from == to {
            // Trivially successful (no path needed) — clear stale error.
            self.set_last_error(None);
            return vec![];
        }
        // Determine direction: if we have a target, trace outbound; otherwise both
        let (direction, filter_target) = if to.is_empty() {
            ("both", None)
        } else {
            ("outbound", Some(to.to_string()))
        };
        let key = format!("trace:{from}:{to}");
        let project = self.project_str();
        let f = from.to_string();
        if self.check_cache(&key) {
            // Cache hit is a successful query — clear any stale error.
            let result = serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
            self.set_last_error(None);
            return result;
        }
        let result = self.query(move |c| c.trace_path(&f, direction, &project, Some(3)));
        match result {
            Ok(edges) => {
                self.set_last_error(None);
                let ge: Vec<GraphEdge> = edges
                    .iter()
                    .filter_map(|e| {
                        let ge = GraphEdge {
                            from: e["from"].as_str()?.to_string(),
                            to: e["to"].as_str()?.to_string(),
                            label: e["label"].as_str().unwrap_or("").into(),
                            properties: HashMap::new(),
                        };
                        // M-01: post-filter if target is specified
                        if let Some(ref target) = filter_target {
                            if ge.to == *target || ge.from == *target {
                                Some(ge)
                            } else {
                                None
                            }
                        } else {
                            Some(ge)
                        }
                    })
                    .collect();
                self.cache_insert(&key, &ge);
                ge
            }
            Err(e) => {
                self.set_last_error(Some(e));
                vec![]
            }
        }
    }

    pub fn invalidate_symbol(&mut self, symbol: &str) {
        self.cache.retain(|k, _| !k.contains(symbol));
    }

    /// Invalidate both the in-memory AND disk caches for the current project.
    ///
    /// Critical: clearing only memory would allow stale data to be re-hydrated
    /// from disk on the next lookup within the TTL window (e.g. after a graph
    /// version change). This must purge the current project's disk partition.
    pub fn invalidate_cache(&mut self) {
        self.cache.clear();
        if let Some(ref disk) = self.disk_cache {
            let project_root = self.project_root.to_string_lossy().into_owned();
            disk.invalidate_project(&project_root);
        }
    }

    /// Alias for `invalidate_cache` — clears memory and the current project's
    /// disk partition (disk coherence).
    pub fn clear_cache(&mut self) {
        self.invalidate_cache();
    }

    /// Detect whether the CBM graph has changed since the last call.
    /// Returns the new graph version if changed, or `None` if CBM is unavailable.
    ///
    /// Cache invalidation is the caller's responsibility — when a new version
    /// is detected, the cache should be invalidated and the version updated.
    pub fn detect_changes(&mut self) -> Result<Option<String>, CbmError> {
        let client_guard = self.client.lock().unwrap_or_else(|p| p.into_inner());
        if client_guard.is_none() {
            return Ok(None);
        }
        drop(client_guard);

        let project = self.project_str();
        let result = self.query(|c| {
            let r = c.call_tool("detect_changes", serde_json::json!({"project": project}))?;
            Ok(r["graph_version"].as_str().map(|s| s.to_string()))
        });
        match result {
            Ok(version) => Ok(version),
            Err(_) => Ok(None),
        }
    }

    /// **Pipe-level proxy call:** Forwards a CBM tool request, catches
    /// the **raw response text** from CBM's stdout pipe, and returns it.
    /// The caller (proxy handler) is responsible for compressing the raw
    /// text with Clean-CTX before it reaches the agent.
    ///
    /// CBM produces a ~5000-token structural seed → Clean-CTX catches it
    /// at the pipe level → compresses to ~1100 tokens → returns.
    pub fn proxy_call(
        &mut self,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<String, CbmError> {
        let _span = tracing::info_span!(
            "cbm_proxy_call",
            tool_name = %tool_name,
        )
        .entered();
        let start = std::time::Instant::now();
        let result = {
            let mut client_guard = self.client.lock().unwrap_or_else(|p| p.into_inner());
            match client_guard.as_mut() {
                Some(c) => c.call_tool_raw(tool_name, args),
                None => return Err(CbmError::LaunchError("CBM not available".into())),
            }
        };
        let latency_ms = start.elapsed().as_millis() as u64;
        // M-1: sync status on every query for self-healing
        self.update_status();
        let _output_len = result.as_ref().map(|s| s.len()).unwrap_or(0);
        tracing::info!(
            tool_name = %tool_name,
            latency_ms = latency_ms,
            output_len = _output_len,
            is_ok = result.is_ok(),
            "cbm_proxy_call complete"
        );
        result
    }

    /// Update status from the underlying client. Also syncs on every
    /// successful query call for self-healing (M-1).
    ///
    /// If the status transitions from Degraded to Available (circuit cooldown
    /// elapsed), we log the recovery.
    pub fn update_status(&mut self) {
        let previous = self.status.clone();
        self.status = match self
            .client
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
        {
            Some(c) => c.status().clone(),
            None => CbmStatus::Unavailable,
        };
        // Log recovery transitions
        if matches!(previous, CbmStatus::Degraded(_)) && self.status.is_available() {
            eprintln!("[clean-ctx-cbm] Recovered — circuit breaker reset, CBM available again");
        }
    }

    // ── Internal helpers ──────────────────────────────────────────

    /// Disk key scope: `{project_name}:{key}` so the effective disk
    /// partition is `(project_root, project_name)`. This prevents a
    /// cross-project data leak when `set_project("repo-b")` is called
    /// while `project_root` is still pinned to repo-a — repo-b queries
    /// would otherwise hydrate repo-a's cached results.
    fn disk_key(&self, key: &str) -> String {
        format!("{}:{key}", self.project_str())
    }

    /// Check cache for a valid (non-expired) entry. Also evicts any
    /// expired entries found during lookup (H-3 fix: lazy GC).
    ///
    /// **Disk hydration:** On a memory miss, checks the disk cache first.
    /// If a valid entry exists on disk, it is hydrated into the in-memory
    /// `DashMap` (zero CBM round-trips) and `true` is returned. This avoids
    /// re-indexing CBM on process restart or when switching projects.
    fn check_cache(&self, key: &str) -> bool {
        if let Some(cached) = self.cache.get(key) {
            if cached.value().expires_at > Instant::now() {
                return true;
            }
            // Expired — clone key then drop guard before remove (avoids borrow conflict)
            let owned_key = key.to_string();
            drop(cached);
            self.cache.remove(&owned_key);
        }

        // Memory miss — try disk cache (lazy hydration on first touch).
        if let Some(ref disk) = self.disk_cache {
            let project_root = self.project_root.to_string_lossy().into_owned();
            let disk_key = self.disk_key(key);
            if let Some(data_json) = disk.get(&project_root, &disk_key) {
                if let Ok(data) = serde_json::from_str::<Value>(&data_json) {
                    let expires_at = Instant::now() + Duration::from_secs(self.cache_ttl);
                    self.cache
                        .insert(key.to_string(), CachedGraphData { data, expires_at });
                    return true;
                }
            }
        }
        false
    }

    /// Insert into cache with TTL expiry. Write-through to disk when a
    /// disk cache is attached, so memory and disk stay in sync.
    fn cache_insert<T: Serialize>(&self, key: &str, value: &T) {
        let data = serde_json::to_value(value).unwrap_or_default();
        let expires_at = Instant::now() + Duration::from_secs(self.cache_ttl);
        self.cache.insert(
            key.to_string(),
            CachedGraphData {
                data: data.clone(),
                expires_at,
            },
        );

        // Write-through to disk cache (scoped by project name).
        if let Some(ref disk) = self.disk_cache {
            let project_root = self.project_root.to_string_lossy().into_owned();
            let disk_key = self.disk_key(key);
            let data_json = data.to_string();
            let expires_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0)
                + (self.cache_ttl as i64 * 1000);
            disk.put(&project_root, &disk_key, &data_json, expires_ms);
        }
    }

    /// Internal query dispatch. Syncs status on success (M-1).
    ///
    /// P1-9: Now acquires the Arc<Mutex<>> client instead of using
    /// a direct field reference.
    fn query<F, T>(&mut self, f: F) -> Result<T, CbmError>
    where
        F: FnOnce(&mut CbmClient) -> Result<T, CbmError>,
    {
        let mut client_guard = self.client.lock().unwrap_or_else(|p| p.into_inner());
        let client = match client_guard.as_mut() {
            Some(c) => c,
            None => return Err(CbmError::LaunchError("CBM not available".into())),
        };
        let result = f(client);
        // Drop the guard before calling update_status to avoid borrow conflict
        drop(client_guard);
        // M-1: sync status on every query for self-healing
        self.update_status();
        result
    }
}

/// Derive CBM's canonical project ID from a repository path.
///
/// Verified against the CBM 0.8.1 wire contract (`index_repository` response +
/// `list_projects`):
///   - `C:/Users/MNasty/Desktop/RustContextLayerAI`
///     → `C-Users-MNasty-Desktop-RustContextLayerAI`
///   - `C:/Users/MNasty/AppData/Local/Temp/CleanCtx_Probe.Repo`
///     → `C-Users-MNasty-AppData-Local-Temp-CleanCtx_Probe.Repo` (dots/underscores kept)
///   - `C:/Users/MNasty/AppData/Local/Temp/My space_probe`
///     → `C-Users-MNasty-AppData-Local-Temp-My-space_probe` (space → dash)
///
/// Algorithm: every character outside `[A-Za-z0-9._-]` becomes `-`, runs of
/// `-` collapse, and leading/trailing `-` are trimmed. The directory basename
/// is NEVER used as a project ID.
pub(crate) fn cbm_project_slug(canonical_root: &Path) -> String {
    let raw = canonical_root.to_string_lossy();
    let mut out = String::with_capacity(raw.len());
    let mut last_dash = false;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
            out.push(c);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        } else if out.is_empty() {
            // Leading separator — skip without emitting a dash.
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "default".to_string()
    } else {
        out
    }
}

/// Register a repository root in the bridge's authoritative identity maps.
///
/// Returns the canonical CBM slug for `root`. Both maps are updated:
/// `project_ids` (root → slug) and `project_paths` (slug → root), so indexing,
/// readiness checks, queries, and proxy calls all resolve to one identity.
fn insert_cbm_project(
    project_ids: &mut HashMap<PathBuf, String>,
    project_paths: &mut HashMap<String, PathBuf>,
    root: &Path,
) -> String {
    let slug = cbm_project_slug(root);
    project_paths.insert(slug.clone(), root.to_path_buf());
    project_ids.insert(root.to_path_buf(), slug.clone());
    slug
}

fn resolve_cbm_binary(config: &CbmConfig) -> Option<PathBuf> {
    // 1. Config path (explicit user override)
    if let Some(ref path) = config.binary_path {
        let p = PathBuf::from(path);
        if p.exists() && p.is_file() {
            return Some(p);
        }
        // Config path doesn't exist — fall through to PATH
        eprintln!(
            "[clean-ctx-cbm] Config binary_path '{}' not found, trying PATH...",
            path
        );
    }

    // 2. PATH search — try all candidate names (e.g. .exe, .cmd on Windows)
    let names = cbm_binary_names();
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for name in &names {
                let candidate = dir.join(name);
                if candidate.exists() && candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    // 3. Common install locations — try all candidate names per directory
    common_install_locations(&names)
}

/// Return all candidate binary names for the current platform.
///
/// On Windows, npm global installs create `.cmd` shims, while standalone
/// installers and cargo builds produce `.exe`. We check all variants.
/// On Unix, there's only one name (no extension).
fn cbm_binary_names() -> Vec<String> {
    if cfg!(windows) {
        vec![
            "codebase-memory-mcp.exe".into(),
            "codebase-memory-mcp.cmd".into(),
            "codebase-memory-mcp.bat".into(),
        ]
    } else {
        vec!["codebase-memory-mcp".into()]
    }
}

/// Return the list of common install directories for the current platform.
///
/// Shared between `common_install_locations()` (binary resolution) and
/// `checked_paths()` (diagnostics) to avoid duplication.
fn install_dirs() -> Vec<PathBuf> {
    let home = home_dir();
    if cfg!(windows) {
        vec![
            // Standalone installer (e.g. %LOCALAPPDATA%\Programs\codebase-memory-mcp)
            home.join("AppData\\Local\\Programs\\codebase-memory-mcp"),
            // Program Files
            PathBuf::from(r"C:\Program Files\codebase-memory-mcp"),
            // Cargo install
            home.join(".cargo\\bin"),
            // Manual install / Claude setup
            home.join(".local\\bin"),
            // npm global (Windows)
            home.join("AppData\\Roaming\\npm"),
            // Alternative local install
            home.join("AppData\\Local\\codebase-memory-mcp"),
        ]
    } else {
        vec![
            // System-wide
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/bin"),
            // Cargo install
            home.join(".cargo/bin"),
            // Manual install / Claude setup
            home.join(".local/bin"),
            // npm global (Unix)
            home.join(".npm-global/bin"),
            // Homebrew Apple Silicon
            PathBuf::from("/opt/homebrew/bin"),
        ]
    }
}

fn common_install_locations(names: &[String]) -> Option<PathBuf> {
    // Try each directory × each candidate name
    for dir in &install_dirs() {
        for name in names {
            let candidate = dir.join(name);
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Return a list of all candidate binary paths that `resolve_cbm_binary` checks.
///
/// This is used by `get_cbm_status` for diagnostics — when CBM is unavailable,
/// the response includes this list so the user can see exactly what was searched
/// and identify why detection failed (e.g. binary installed in an unlisted dir).
pub fn checked_paths() -> Vec<String> {
    let names = cbm_binary_names();
    let mut paths = Vec::new();

    for dir in &install_dirs() {
        for name in &names {
            paths.push(dir.join(name).to_string_lossy().into_owned());
        }
    }

    // Also note PATH search
    paths.push("(PATH search for all candidate names)".into());

    paths
}

fn home_dir() -> PathBuf {
    // C-3 fix: use HOME/USERPROFILE env var instead of hardcoded username
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        return PathBuf::from(home);
    }
    // Last resort for legacy windows
    if let Ok(drive) = std::env::var("HOMEDRIVE") {
        if let Ok(path) = std::env::var("HOMEPATH") {
            return PathBuf::from(drive).join(path);
        }
    }
    PathBuf::from(".")
}

// ── Test helpers (exported under `test_helpers` for test access) ──
#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use std::collections::HashMap;
    pub fn resolve_binary(config: &CbmConfig) -> Option<PathBuf> {
        resolve_cbm_binary(config)
    }
    // Intentionally kept simple: test access to bridge internals.
    #[allow(private_interfaces)]
    pub fn cache_ttl(bridge: &GraphBridge) -> u64 {
        bridge.cache_ttl
    }

    /// Create a mock GraphBridge with canned symbol importance data.
    ///
    /// P0-2: Fixed — the mock now sets `status: CbmStatus::Available` and
    /// overrides `is_available()` behavior by pre-seeding the cache.
    /// The mock has no real CBM client, so `query()` will return Err.
    ///
    /// P1-9: The mock pre-sets `indexing_state` to `Complete` so that
    /// `ensure_indexed()` returns `Ready` immediately. Tests that need
    /// to exercise indexing states should override this after creation.
    pub fn new_mock(symbol_importance: HashMap<String, SymbolImportance>) -> GraphBridge {
        let mut states = HashMap::new();
        states.insert("test-project".to_string(), IndexingState::Complete);
        let bridge = GraphBridge {
            client: Arc::new(Mutex::new(None)),
            cache: DashMap::new(),
            status: CbmStatus::Available,
            cache_ttl: 3600,
            project: Some("test-project".to_string()),
            project_root: PathBuf::from("."),
            disk_cache: None,
            graph_version: String::new(),
            indexing_state: Arc::new(Mutex::new(states)),
            project_ids: HashMap::new(),
            project_paths: HashMap::new(),
            last_error: None,
        };
        // Pre-seed the symbol_importance cache entry
        let key = "symbol_importance".to_string();
        let json = serde_json::to_value(&symbol_importance).unwrap_or_default();
        bridge.cache.insert(
            key,
            CachedGraphData {
                data: json,
                expires_at: Instant::now() + Duration::from_secs(3600),
            },
        );
        bridge
    }

    /// Create a mock GraphBridge with no canned data (available, but
    /// symbol_importance cache returns empty).
    pub fn new_mock_empty() -> GraphBridge {
        new_mock(HashMap::new())
    }

    /// Create a mock GraphBridge that is `Available` but whose indexing state
    /// is `NotStarted` (a state that only exists transiently before the
    /// construction-time `start_indexing()` thread flips it to `InProgress`).
    ///
    /// K-1: Used to prove `ensure_indexed()` is **report-only** — it must NOT
    /// transition `NotStarted` → `InProgress` (i.e. it must not spawn an
    /// indexing thread). The old behavior spawned on `NotStarted`.
    pub fn new_available_not_started() -> GraphBridge {
        let bridge = GraphBridge {
            client: Arc::new(Mutex::new(None)),
            cache: DashMap::new(),
            status: CbmStatus::Available,
            cache_ttl: 3600,
            project: Some("test-project".to_string()),
            project_root: PathBuf::from("."),
            disk_cache: None,
            graph_version: String::new(),
            indexing_state: Arc::new(Mutex::new(HashMap::new())),
            project_ids: HashMap::new(),
            project_paths: HashMap::new(),
            last_error: None,
        };
        // Seed a cache entry so is_available() is true (client is None).
        bridge.cache.insert(
            "__available__".to_string(),
            CachedGraphData {
                data: serde_json::json!("available"),
                expires_at: Instant::now() + Duration::from_secs(3600),
            },
        );
        bridge
    }

    /// Create a mock GraphBridge pre-seeded with call edges, dataflow edges,
    /// symbol importance, and dead code for exercising
    /// `InferenceLayer::enrich_from_cbm()`.
    ///
    /// R-43b Phase 3: Pre-seeds the `call_edges`, `dataflow_edges`,
    /// `symbol_importance`, and `dead_code` cache entries so the mock serves
    /// canned data without a real CBM binary.
    pub fn new_mock_with_edges(
        call_edges: Vec<(String, String)>,
        dataflow_edges: Vec<(String, String, String)>,
        symbol_importance: HashMap<String, SymbolImportance>,
        dead_code: Vec<DeadCodeEntry>,
    ) -> GraphBridge {
        let mut states = HashMap::new();
        states.insert("test-project".to_string(), IndexingState::Complete);
        let bridge = GraphBridge {
            client: Arc::new(Mutex::new(None)),
            cache: DashMap::new(),
            status: CbmStatus::Available,
            cache_ttl: 3600,
            project: Some("test-project".to_string()),
            project_root: PathBuf::from("."),
            disk_cache: None,
            graph_version: String::new(),
            indexing_state: Arc::new(Mutex::new(states)),
            project_ids: HashMap::new(),
            project_paths: HashMap::new(),
            last_error: None,
        };
        let ttl = Duration::from_secs(3600);
        let call_json = serde_json::to_value(&call_edges).unwrap_or_default();
        bridge.cache.insert(
            "call_edges".to_string(),
            CachedGraphData {
                data: call_json,
                expires_at: Instant::now() + ttl,
            },
        );
        let df_json = serde_json::to_value(&dataflow_edges).unwrap_or_default();
        bridge.cache.insert(
            "dataflow_edges".to_string(),
            CachedGraphData {
                data: df_json,
                expires_at: Instant::now() + ttl,
            },
        );
        let si_json = serde_json::to_value(&symbol_importance).unwrap_or_default();
        bridge.cache.insert(
            "symbol_importance".to_string(),
            CachedGraphData {
                data: si_json,
                expires_at: Instant::now() + ttl,
            },
        );
        let dc_json = serde_json::to_value(&dead_code).unwrap_or_default();
        bridge.cache.insert(
            "dead_code".to_string(),
            CachedGraphData {
                data: dc_json,
                expires_at: Instant::now() + ttl,
            },
        );
        bridge
    }
}
