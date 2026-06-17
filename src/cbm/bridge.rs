// src/cbm/bridge.rs
//
// Graph Bridge — translates CBM graph data into Clean-CTX concepts.
// Entirely self-contained with its own types and caching.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    data: Value,
    expires_at: Instant,
}

/// Graph bridge with TTL caching and graceful degradation.
pub struct GraphBridge {
    client: Option<CbmClient>,
    cache: DashMap<String, CachedGraphData>,
    status: CbmStatus,
    cache_ttl: u64,
    project: Option<String>,
    graph_version: String,
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
                Some(path) => match CbmClient::try_launch(&path, Duration::from_millis(config.query_timeout_ms)) {
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
            client,
            cache: DashMap::new(),
            cache_ttl: config.cache_ttl,
            project: Some(project_name),
            graph_version: String::new(),
        }
    }

    pub fn set_project(&mut self, project: &str) { self.project = Some(project.to_string()); }
    pub fn status(&self) -> &CbmStatus { &self.status }
    pub fn is_available(&self) -> bool { self.status.is_available() && self.client.is_some() }
    pub fn graph_version(&self) -> &str { &self.graph_version }
    pub fn set_graph_version(&mut self, v: &str) { self.graph_version = v.to_string(); }

    /// Resolve project name, using explicit if set, else auto-detected.
    fn project_str(&self) -> String {
        self.project.clone().unwrap_or_else(|| "default".to_string())
    }

    pub fn get_symbol_importance_mut(&mut self) -> HashMap<String, SymbolImportance> {
        let key = "symbol_importance".to_string();
        let project = self.project_str();
        if self.check_cache(&key) {
            return serde_json::from_value(self.cache.get(&key).unwrap().value().data.clone()).unwrap_or_default();
        }
        let result = self.query(move |c| c.get_symbol_importance(&project));
        match result {
            Ok(symbols) => {
                let map: HashMap<_, _> = symbols.iter().filter_map(|e| {
                    Some((e["name"].as_str()?.to_string(), SymbolImportance {
                        symbol: e["name"].as_str()?.to_string(),
                        score: e["importance"].as_f64()?,
                        file: e["file"].as_str().unwrap_or("").to_string(),
                    }))
                }).collect();
                self.cache_insert(&key, &map);
                map
            }
            Err(_) => HashMap::new(),
        }
    }

    /// Blast radius query at depth 1. Multi-hop depth expansion is a future
    /// enhancement — the `depth` parameter is accepted for API compatibility
    /// but currently ignored.
    pub fn get_blast_radius(&mut self, symbol: &str, _depth: usize) -> Vec<String> {
        let key = format!("blast:{symbol}");
        let project = self.project_str();
        let sym = symbol.to_string();
        if self.check_cache(&key) {
            return serde_json::from_value(self.cache.get(&key).unwrap().value().data.clone()).unwrap_or_default();
        }
        let result = self.query(move |c| c.search_graph(&format!("depends_on:{sym}"), &project));
        match result {
            Ok(nodes) => {
                let files: Vec<_> = nodes.iter().filter_map(|n| n["file"].as_str().map(String::from)).collect();
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
            return serde_json::from_value(self.cache.get(&key).unwrap().value().data.clone()).unwrap_or_default();
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

    pub fn get_architecture(&mut self) -> Option<ArchitectureOverview> {
        let key = "architecture".to_string();
        let project = self.project_str();
        if self.check_cache(&key) {
            return serde_json::from_value(self.cache.get(&key).unwrap().value().data.clone()).ok();
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
            return serde_json::from_value(self.cache.get(&key).unwrap().value().data.clone()).unwrap_or(QueryResult { nodes: vec![], edges: vec![] });
        }
        let result = self.query(move |c| c.query_graph(&q, &project));
        match result {
            Ok(rows) => {
                let mut nodes = vec![]; let mut edges = vec![];
                for row in &rows {
                    if let Some(node) = row.get("node") {
                        nodes.push(GraphNode {
                            id: node["id"].as_str().unwrap_or("").to_string(),
                            label: node["label"].as_str().unwrap_or("").to_string(),
                            name: node["name"].as_str().unwrap_or("").to_string(),
                            file: node["file"].as_str().unwrap_or("").to_string(),
                            properties: HashMap::new(),
                        });
                    }
                    if let Some(edge) = row.get("edge") {
                        edges.push(GraphEdge {
                            from: edge["from"].as_str().unwrap_or("").into(),
                            to: edge["to"].as_str().unwrap_or("").into(),
                            label: edge["label"].as_str().unwrap_or("").into(),
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
            return serde_json::from_value(self.cache.get(&key).unwrap().value().data.clone()).unwrap_or_default();
        }
        let result = self.query(move |c| c.search_graph(&q, &project));
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

    pub fn trace_path(&mut self, from: &str, to: &str) -> Vec<GraphEdge> {
        if from == to {
            return vec![];
        }
        let key = format!("trace:{from}:{to}");
        let project = self.project_str();
        let f = from.to_string(); let t = to.to_string();
        if self.check_cache(&key) {
            return serde_json::from_value(self.cache.get(&key).unwrap().value().data.clone()).unwrap_or_default();
        }
        let result = self.query(move |c| c.trace_path(&f, &t, &project));
        match result {
            Ok(edges) => {
                let ge: Vec<GraphEdge> = edges.iter().filter_map(|e| {
                    Some(GraphEdge {
                        from: e["from"].as_str()?.to_string(),
                        to: e["to"].as_str()?.to_string(),
                        label: e["label"].as_str().unwrap_or("").into(),
                        properties: HashMap::new(),
                    })
                }).collect();
                self.cache_insert(&key, &ge);
                ge
            }
            Err(_) => vec![],
        }
    }

    pub fn invalidate_symbol(&mut self, symbol: &str) { self.cache.retain(|k, _| !k.contains(symbol)); }
    pub fn clear_cache(&mut self) { self.cache.clear(); }

    /// **Pipe-level proxy call:** Forwards a CBM tool request, catches
    /// the **raw response text** from CBM's stdout pipe, and returns it.
    /// The caller (proxy handler) is responsible for compressing the raw
    /// text with Clean-CTX before it reaches the agent.
    ///
    /// CBM produces a ~5000-token structural seed → Clean-CTX catches it
    /// at the pipe level → compresses to ~1100 tokens → returns.
    pub fn proxy_call(&mut self, tool_name: &str, args: serde_json::Value) -> Result<String, CbmError> {
        let result = match self.client.as_mut() {
            Some(c) => c.call_tool_raw(tool_name, args),
            None => return Err(CbmError::LaunchError("CBM not available".into())),
        };
        // M-1: sync status on every query for self-healing
        self.update_status();
        result
    }

    /// Update status from the underlying client. Also syncs on every
    /// successful query call for self-healing (M-1).
    pub fn update_status(&mut self) {
        self.status = match self.client.as_ref() {
            Some(c) => c.status().clone(),
            None => CbmStatus::Unavailable,
        };
    }

    // ── Internal helpers ──────────────────────────────────────────

    /// Check cache for a valid (non-expired) entry. Also evicts any
    /// expired entries found during lookup (H-3 fix: lazy GC).
    fn check_cache(&self, key: &str) -> bool {
        if let Some(cached) = self.cache.get(key) {
            if cached.value().expires_at > Instant::now() {
                return true;
            }
            // Expired — drop it (lazy GC eviction, H-3)
            drop(cached);
            self.cache.remove(key);
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
    fn query<F, T>(&mut self, f: F) -> Result<T, CbmError>
    where F: FnOnce(&mut CbmClient) -> Result<T, CbmError>,
    {
        let client = match self.client.as_mut() {
            Some(c) => c,
            None => return Err(CbmError::LaunchError("CBM not available".into())),
        };
        let result = f(client);
        // M-1: sync status on every query for self-healing
        self.update_status();
        result
    }
}

/// Resolve CBM binary path with proper fallback chain (C-2 fix):
///  1. Config `binary_path` (if set)
///  2. PATH search
///  3. Common install locations
// ── Test helpers (exported under `test_helpers` for test access) ──
#[cfg(test)]
pub mod test_helpers {
    use super::*;
    pub fn resolve_binary(config: &CbmConfig) -> Option<PathBuf> {
        resolve_cbm_binary(config)
    }
    // Intentionally kept simple: test access to bridge internals.
    #[allow(private_interfaces)]
    pub fn cache_ttl(bridge: &GraphBridge) -> u64 { bridge.cache_ttl }
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

    // 2. PATH search
    let name = cbm_binary_name();
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(&name);
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    // 3. Common install locations (C-3 fix: use dynamic home dir)
    common_install_locations(&name)
}

fn cbm_binary_name() -> String {
    if cfg!(windows) { "codebase-memory-mcp.exe".into() } else { "codebase-memory-mcp".into() }
}

fn common_install_locations(name: &str) -> Option<PathBuf> {
    let home = home_dir();
    let candidates: Vec<PathBuf> = if cfg!(windows) {
        vec![
            PathBuf::from(r"C:\Program Files\codebase-memory-mcp").join(name),
            home.join(".cargo\\bin").join(name),
        ]
    } else {
        vec![
            PathBuf::from("/usr/local/bin").join(name),
            PathBuf::from("/usr/bin").join(name),
            home.join(".cargo/bin").join(name),
            home.join(".local/bin").join(name),
        ]
    };
    candidates.into_iter().find(|p| p.exists() && p.is_file())
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