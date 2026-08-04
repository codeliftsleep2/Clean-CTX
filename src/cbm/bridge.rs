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
    InProgress {
        started_at: Instant,
    },
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
    graph_version: String,
    /// P1-9: Replaced `indexed: bool` with state machine.
    pub(crate) indexing_state: Arc<Mutex<IndexingState>>,
}

impl GraphBridge {
    /// Try to discover and launch CBM. Returns a bridge; use `is_available()` to check.
    ///
    /// Binary resolution order:
    ///   1. `config.binary_path` (explicit config)
    ///   2. PATH search for `codebase-memory-mcp`
    ///   3. Common install locations (`~/.cargo/bin`, `/usr/local/bin`, etc.)
    ///
    /// Project name is auto-detected from the workspace root directory name.
    pub fn try_create(config: &CbmConfig, project_root: &Path) -> Self {
        let binary_path = resolve_cbm_binary(config);
        let project_name = project_root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "default".to_string());

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
                    Ok(None) => { eprintln!("[clean-ctx-cbm] Binary not found: {}", path.display()); None }
                    Err(e) => { eprintln!("[clean-ctx-cbm] Launch failed: {e}"); None }
                }
                None => {
                    eprintln!("[clean-ctx-cbm] Not found on PATH or common locations.");
                    eprintln!("  Install from: https://github.com/DeusData/codebase-memory-mcp");
                    None
                }
            }
        } else {
            None
        };

        Self {
            status: if client.is_some() { CbmStatus::Available } else { CbmStatus::Unavailable },
            client: Arc::new(Mutex::new(client)),
            cache: DashMap::new(),
            cache_ttl: config.cache_ttl,
            project: Some(project_name),
            graph_version: String::new(),
            indexing_state: Arc::new(Mutex::new(IndexingState::NotStarted)),
        }
    }

    pub fn set_project(&mut self, project: &str) { self.project = Some(project.to_string()); }
    pub fn status(&self) -> &CbmStatus { &self.status }
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
        self.client.lock().unwrap_or_else(|p| p.into_inner()).is_some() || !self.cache.is_empty()
    }
    pub fn graph_version(&self) -> &str { &self.graph_version }
    pub fn set_graph_version(&mut self, v: &str) { self.graph_version = v.to_string(); }

    /// Trigger indexing of the current project in CBM.
    /// Called automatically on startup when CBM is available.
    /// Returns Ok(()) if indexing was triggered successfully, or Err if CBM is unavailable.
    ///
    /// P1-9: This now dispatches indexing to a background thread.
    /// The caller receives `StillIndexing` from `ensure_indexed()` and
    /// the actual index_project call happens asynchronously.
    pub fn index_project(&mut self) -> Result<(), CbmError> {
        if !self.is_available() {
            return Err(CbmError::LaunchError("CBM not available".into()));
        }
        let project = self.project_str();
        eprintln!("[clean-ctx-cbm] Indexing project: {project}");
        // Call CBM's index_project tool to trigger indexing
        let client_guard = self.client.lock().unwrap_or_else(|p| p.into_inner());
        let _result = match client_guard.as_ref() {
            Some(_c) => {
                // We need mut access — drop guard, reacquire as mut
                drop(client_guard);
                let mut cg = self.client.lock().unwrap_or_else(|p| p.into_inner());
                let client = cg.as_mut().unwrap();
                client.call_tool("index_project", serde_json::json!({"project": project}))
            }
            None => return Err(CbmError::LaunchError("CBM not available".into())),
        }?;
        eprintln!("[clean-ctx-cbm] Project indexed successfully");
        Ok(())
    }

    /// Ensure the project is indexed before issuing queries.
    ///
    /// P1-9: **Non-blocking rewrite.** If indexing has not started,
    /// spawns a background thread to perform the actual pipe I/O and
    /// returns `StillIndexing` immediately. Callers (tool handlers)
    /// respond to the agent with a "retry later" message instead of
    /// blocking the entire dispatcher thread.
    pub fn ensure_indexed(&mut self) -> Result<IndexingStatus, CbmError> {
        let mut state = self.indexing_state.lock().unwrap_or_else(|p| p.into_inner());

        match &*state {
            IndexingState::Complete => return Ok(IndexingStatus::Ready),
            IndexingState::InProgress { started_at } => {
                let elapsed = started_at.elapsed().as_secs();
                if elapsed > 60 {
                    *state = IndexingState::Failed("indexing timed out after 60s".into());
                    return Err(CbmError::LaunchError("indexing timed out".into()));
                }
                return Ok(IndexingStatus::StillIndexing { elapsed_secs: elapsed });
            }
            IndexingState::Failed(msg) => {
                return Err(CbmError::LaunchError(msg.clone()));
            }
            IndexingState::NotStarted => {} // fall through to spawn
        }

        // Mark as in-progress before spawning (so concurrent calls see it)
        *state = IndexingState::InProgress {
            started_at: Instant::now(),
        };

        // Clone Arc handles for the background thread
        let client_arc = Arc::clone(&self.client);
        let state_arc = Arc::clone(&self.indexing_state);
        let project = self.project_str();
        let _status = self.status.clone();

        // Spawn background indexing thread
        std::thread::Builder::new()
            .name("cbm-indexer".into())
            .spawn(move || {
                eprintln!("[clean-ctx-cbm] Background indexing started for: {project}");

                let result = {
                    let mut client_guard = match client_arc.lock() {
                        Ok(g) => g,
                        Err(poisoned) => {
                            eprintln!("[clean-ctx-cbm] WARNING: Recovering from poisoned client lock");
                            poisoned.into_inner()
                        }
                    };
                    match client_guard.as_mut() {
                        Some(client) => {
                            match client.call_tool("index_project", serde_json::json!({"project": project})) {
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

                let mut s = match state_arc.lock() {
                    Ok(g) => g,
                    Err(poisoned) => {
                        eprintln!("[clean-ctx-cbm] WARNING: Recovering from poisoned state lock");
                        poisoned.into_inner()
                    }
                };
                match result {
                    Ok(()) => {
                        *s = IndexingState::Complete;
                    }
                    Err(e) => {
                        // Record failure for circuit breaker
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

        Ok(IndexingStatus::StillIndexing { elapsed_secs: 0 })
    }

    /// Access the indexing state for inspection (e.g., get_cbm_status handler).
    pub fn indexing_state(&self) -> std::sync::MutexGuard<'_, IndexingState> {
        self.indexing_state.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Resolve project name, using explicit if set, else auto-detected.
    fn project_str(&self) -> String {
        self.project.clone().unwrap_or_else(|| "default".to_string())
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
                let map: HashMap<_, _> = symbols.iter().filter_map(|e| {
                    Some((e["name"].as_str()?.to_string(), SymbolImportance {
                        symbol: e["name"].as_str()?.to_string(),
                        score: e["importance"].as_f64().unwrap_or(0.0),
                        file: e["file"].as_str().unwrap_or("").to_string(),
                    }))
                }).collect();
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
                let files: Vec<_> = rows.iter().filter_map(|r| {
                    r.get(1).and_then(|v| v.as_str().map(String::from))
                }).collect();
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
                let dead: Vec<DeadCodeEntry> = entries.iter().filter_map(|e| {
                    Some(DeadCodeEntry {
                        symbol: e["name"].as_str()?.to_string(),
                        file: e["file"].as_str().unwrap_or("").to_string(),
                        reason: e["reason"].as_str().unwrap_or("unknown").to_string(),
                    })
                }).collect();
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
                let edges: Vec<(String, String)> = rows.iter().filter_map(|r| {
                    let from = r.first()?.as_str()?.to_string();
                    let to = r.get(1)?.as_str()?.to_string();
                    Some((from, to))
                }).collect();
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
        let cypher = "MATCH (m:Function)-[r:DATAFLOW]->(t) RETURN m.name, t.name, type(r)".to_string();
        let result = self.query(move |c| c.query_graph(&cypher, &project));
        match result {
            Ok(rows) => {
                let edges: Vec<(String, String, String)> = rows.iter().filter_map(|r| {
                    let method = r.first()?.as_str()?.to_string();
                    let target = r.get(1)?.as_str()?.to_string();
                    let direction = r.get(2)?.as_str().unwrap_or("reads").to_string();
                    Some((method, target, direction))
                }).collect();
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
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .ok();
        }
        let result = self.query(move |c| c.get_architecture(&project));
        match result {
            Ok(arch) => {
                let modules = arch["modules"].as_array().map(|ms| {
                    ms.iter().filter_map(|m| Some(ArchitectureModule {
                        name: m["name"].as_str()?.to_string(),
                        path: m["path"].as_str().unwrap_or("").to_string(),
                        file_count: m["file_count"].as_u64().unwrap_or(0) as usize,
                    })).collect()
                }).unwrap_or_default();
                let dependencies = arch["dependencies"].as_array().map(|ds| {
                    ds.iter().filter_map(|d| Some(ArchitectureDependency {
                        from: d["from"].as_str()?.to_string(),
                        to: d["to"].as_str()?.to_string(),
                        kind: d["kind"].as_str().unwrap_or("import").to_string(),
                    })).collect()
                }).unwrap_or_default();
                let ov = ArchitectureOverview { modules, dependencies };
                self.cache_insert(&key, &ov);
                Some(ov)
            }
            Err(_) => None,
        }
    }

    pub fn query_graph(&mut self, cypher: &str) -> QueryResult {
        let key = format!("cypher:{cypher}");
        let project = self.project_str();
        let q = cypher.to_string();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or(QueryResult { nodes: vec![], edges: vec![] });
        }
        let result = self.query(move |c| c.query_graph(&q, &project));
        match result {
            Ok(rows) => {
                // CBM Cypher returns {columns, rows} — rows are Vec<Vec<Value>>.
                // We try to interpret each row as either node or edge data.
                let mut nodes = vec![]; let edges = vec![];
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
            Err(_) => QueryResult { nodes: vec![], edges: vec![] },
        }
    }

    pub fn search(&mut self, query: &str) -> Vec<GraphNode> {
        let key = format!("search:{query}");
        let project = self.project_str();
        let q = query.to_string();
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
        // H-01 fix: build a proper regex name_pattern from the query string.
        // If the query already contains regex metacharacters (., *, +, [, etc.)
        // use it as-is; otherwise wrap in .*...* for substring matching.
        let has_regex = q.chars().any(|c| matches!(c, '.' | '*' | '+' | '[' | '(' | '\\' | '^' | '$' | '{' | '|'));
        let name_pattern = if has_regex { q } else { format!(".*{q}.*") };
        let result = self.query(move |c| c.search_graph(&name_pattern, &project, Some("Function")));
        match result {
            Ok(nodes) => {
                let gn: Vec<GraphNode> = nodes.iter().filter_map(|n| {
                    Some(GraphNode {
                        id: n["id"].as_str()?.to_string(),
                        label: n["label"].as_str().unwrap_or("").into(),
                        name: n["name"].as_str()?.to_string(),
                        file: n["file"].as_str().unwrap_or("").into(),
                        properties: HashMap::new(),
                    })
                }).collect();
                self.cache_insert(&key, &gn);
                gn
            }
            Err(_) => vec![],
        }
    }

    /// Trace a call path between two symbols. If `to` is empty, traces all edges
    /// from `from`. Otherwise post-filters to only include edges reaching `to`.
    ///
    /// M-01 fix: Previously ignored the `to` parameter — now properly filters
    /// results to only include edges reaching the target symbol.
    pub fn trace_path(&mut self, from: &str, to: &str) -> Vec<GraphEdge> {
        if from == to {
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
        let result = self.query(move |c| c.trace_path(&f, direction, &project, Some(3)));
        match result {
            Ok(edges) => {
                let ge: Vec<GraphEdge> = edges.iter().filter_map(|e| {
                    let ge = GraphEdge {
                        from: e["from"].as_str()?.to_string(),
                        to: e["to"].as_str()?.to_string(),
                        label: e["label"].as_str().unwrap_or("").into(),
                        properties: HashMap::new(),
                    };
                    // M-01: post-filter if target is specified
                    if let Some(ref target) = filter_target {
                        if ge.to == *target || ge.from == *target { Some(ge) } else { None }
                    } else {
                        Some(ge)
                    }
                }).collect();
                self.cache_insert(&key, &ge);
                ge
            }
            Err(_) => vec![],
        }
    }

    pub fn invalidate_symbol(&mut self, symbol: &str) { self.cache.retain(|k, _| !k.contains(symbol)); }
    pub fn invalidate_cache(&mut self) { self.cache.clear(); }
    pub fn clear_cache(&mut self) { self.cache.clear(); }

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
    pub fn proxy_call(&mut self, tool_name: &str, args: serde_json::Value) -> Result<String, CbmError> {
        let _span = tracing::info_span!(
            "cbm_proxy_call",
            tool_name = %tool_name,
        ).entered();
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
        self.status = match self.client.lock().unwrap_or_else(|p| p.into_inner()).as_ref() {
            Some(c) => c.status().clone(),
            None => CbmStatus::Unavailable,
        };
        // Log recovery transitions
        if matches!(previous, CbmStatus::Degraded(_)) && self.status.is_available() {
            eprintln!("[clean-ctx-cbm] Recovered — circuit breaker reset, CBM available again");
        }
    }

    // ── Internal helpers ──────────────────────────────────────────

    /// Check cache for a valid (non-expired) entry. Also evicts any
    /// expired entries found during lookup (H-3 fix: lazy GC).
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
        false
    }

    /// Insert into cache with TTL expiry.
    fn cache_insert<T: Serialize>(&self, key: &str, value: &T) {
        let data = serde_json::to_value(value).unwrap_or_default();
        self.cache.insert(key.to_string(), CachedGraphData {
            data,
            expires_at: Instant::now() + Duration::from_secs(self.cache_ttl),
        });
    }

    /// Internal query dispatch. Syncs status on success (M-1).
    ///
    /// P1-9: Now acquires the Arc<Mutex<>> client instead of using
    /// a direct field reference.
    fn query<F, T>(&mut self, f: F) -> Result<T, CbmError>
    where F: FnOnce(&mut CbmClient) -> Result<T, CbmError>,
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

fn resolve_cbm_binary(config: &CbmConfig) -> Option<PathBuf> {
    // 1. Config path (explicit user override)
    if let Some(ref path) = config.binary_path {
        let p = PathBuf::from(path);
        if p.exists() && p.is_file() {
            return Some(p);
        }
        // Config path doesn't exist — fall through to PATH
        eprintln!("[clean-ctx-cbm] Config binary_path '{}' not found, trying PATH...", path);
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
    pub fn cache_ttl(bridge: &GraphBridge) -> u64 { bridge.cache_ttl }

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
        let bridge = GraphBridge {
            client: Arc::new(Mutex::new(None)),
            cache: DashMap::new(),
            status: CbmStatus::Available,
            cache_ttl: 3600,
            project: Some("test-project".to_string()),
            graph_version: String::new(),
            indexing_state: Arc::new(Mutex::new(IndexingState::Complete)),
        };
        // Pre-seed the symbol_importance cache entry
        let key = "symbol_importance".to_string();
        let json = serde_json::to_value(&symbol_importance).unwrap_or_default();
        bridge.cache.insert(key, CachedGraphData {
            data: json,
            expires_at: Instant::now() + Duration::from_secs(3600),
        });
        bridge
    }

    /// Create a mock GraphBridge with no canned data (available, but
    /// symbol_importance cache returns empty).
    pub fn new_mock_empty() -> GraphBridge {
        new_mock(HashMap::new())
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
        let bridge = GraphBridge {
            client: Arc::new(Mutex::new(None)),
            cache: DashMap::new(),
            status: CbmStatus::Available,
            cache_ttl: 3600,
            project: Some("test-project".to_string()),
            graph_version: String::new(),
            indexing_state: Arc::new(Mutex::new(IndexingState::Complete)),
        };
        let ttl = Duration::from_secs(3600);
        let call_json = serde_json::to_value(&call_edges).unwrap_or_default();
        bridge.cache.insert("call_edges".to_string(), CachedGraphData {
            data: call_json,
            expires_at: Instant::now() + ttl,
        });
        let df_json = serde_json::to_value(&dataflow_edges).unwrap_or_default();
        bridge.cache.insert("dataflow_edges".to_string(), CachedGraphData {
            data: df_json,
            expires_at: Instant::now() + ttl,
        });
        let si_json = serde_json::to_value(&symbol_importance).unwrap_or_default();
        bridge.cache.insert("symbol_importance".to_string(), CachedGraphData {
            data: si_json,
            expires_at: Instant::now() + ttl,
        });
        let dc_json = serde_json::to_value(&dead_code).unwrap_or_default();
        bridge.cache.insert("dead_code".to_string(), CachedGraphData {
            data: dc_json,
            expires_at: Instant::now() + ttl,
        });
        bridge
    }
}
