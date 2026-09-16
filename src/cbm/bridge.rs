// src/cbm/bridge.rs
//
// Graph Bridge — translates CBM graph data into Clean-CTX concepts.
// Entirely self-contained with its own types and caching.

use crate::cbm::cache_store::GraphCacheStore;
use crate::cbm::client::CbmClient;
use crate::cbm::client::CbmError;
use crate::cbm::config::CbmStatus;
use dashmap::DashMap;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

// -- Semantic submodules --------------------------------------------------
//
// The bridge's cache, project lifecycle, indexing lifecycle, and graph-query
// responsibilities live in submodules; each was already a contiguous block
// here, so the split follows the boundaries the file had. The re-exports below
// keep every existing `crate::cbm::bridge::<item>` path valid.
mod binary;
mod cache;
mod indexing;
mod lifecycle;
mod parse;
mod project;
mod query;

pub use binary::checked_paths;
pub(crate) use parse::{
    QUERY_CACHE_KEY_NAMESPACE, convert_query_rows, filter_trace_edges, map_search_result,
    parse_architecture_response,
};
pub(crate) use project::{cbm_project_slug, insert_cbm_project};
// â”€â”€ Public types â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

// â”€â”€ Internal cache â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

pub(crate) struct CachedGraphData {
    pub(crate) data: Value,
    pub(crate) expires_at: Instant,
}

// â”€â”€ P1-9: Non-blocking indexing state machine â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

/// Per-project freshness state for lazy CBM graph reindexing.
///
/// Tracks whether filesystem edits have occurred since the last successful
/// reindex. Independent of `IndexingState` (which tracks startup/construction
/// indexing lifecycle).
///
/// A project is "stale" (needs reindexing before the next graph query) when
/// `dirty_generation > indexed_generation`.
#[derive(Debug, Clone)]
pub(crate) struct ProjectFreshness {
    /// Monotonically increasing counter. Incremented on every dirty mark.
    pub(crate) dirty_generation: u64,

    /// Value of `dirty_generation` represented by the last successful reindex.
    pub(crate) indexed_generation: u64,
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
    pub(crate) cache_ttl: u64,
    pub(crate) project: Option<String>,
    /// Canonicalized project root. Used as the disk-cache partition key and
    /// to derive the project name. Multi-repo support: each repo gets its
    /// own cache partition and indexing state.
    pub(crate) project_root: PathBuf,
    /// Optional SQLite-backed disk cache. When present, cache entries are
    /// hydrated from disk on first touch (avoiding CBM re-indexing on
    /// restart) and written through on insert.
    pub(crate) disk_cache: Option<GraphCacheStore>,
    pub(crate) graph_version: String,
    /// P1-9: Replaced `indexed: bool` with state machine.
    /// Multi-repo: keyed by project name so switching projects doesn't
    /// corrupt another project's indexing state.
    pub(crate) indexing_state: Arc<Mutex<HashMap<String, IndexingState>>>,
    /// Authoritative mapping: canonical repository root â†’ CBM project ID.
    ///
    /// CBM derives a project's identity from the canonical repo path (see
    /// `cbm_project_slug`), NOT from the directory basename. This map holds
    /// the primary root plus every configured additional root, so queries,
    /// readiness checks, and proxy calls always use the same CBM identity.
    /// Per-project freshness tracking for lazy graph reindexing.
    /// Keyed by CBM project slug.
    /// Independent of `indexing_state` — orthogonal concern.
    pub(crate) freshness: Arc<Mutex<HashMap<String, ProjectFreshness>>>,
    pub(crate) project_ids: HashMap<PathBuf, String>,
    /// Inverse of `project_ids`: CBM project ID â†’ canonical repository root.
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
    pub(crate) last_error: Option<CbmError>,
}

// â”€â”€ Test helpers (exported under `test_helpers` for test access) â”€â”€
#[cfg(test)]
pub mod test_helpers {
    use super::binary::resolve_cbm_binary;
    use super::*;
    use crate::cbm::config::CbmConfig;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};
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
            freshness: Arc::new(Mutex::new(HashMap::new())),
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
    /// transition `NotStarted` â†’ `InProgress` (i.e. it must not spawn an
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
            freshness: Arc::new(Mutex::new(HashMap::new())),
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

    /// Create a mock GraphBridge pre-seeded with call edges,
    /// symbol importance, and dead code for exercising
    /// `InferenceLayer::enrich_from_cbm()`.
    ///
    /// AUDIT F10: the former `dataflow_edges` parameter was removed along
    /// with the dead DATAFLOW query path (CBM 0.8.1 limitation).
    pub fn new_mock_with_edges(
        call_edges: Vec<(String, String)>,
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
            freshness: Arc::new(Mutex::new(HashMap::new())),
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
