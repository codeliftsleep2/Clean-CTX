//! CBM project lifecycle for [`GraphBridge`].
//!
//! Construction (including launching the CBM subprocess and seeding the
//! authoritative root -> project map), active project/workspace switching, and
//! canonical project identity resolution. Split out of `bridge.rs` along the
//! boundary the file already had — no behavior changed.

use super::binary::resolve_cbm_binary;
use super::*;
use crate::cbm::cache_store::GraphCacheStore;
use crate::cbm::client::CbmClient;
use crate::cbm::config::CbmConfig;
use crate::cbm::config::CbmStatus;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

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

        // Build the authoritative root â†’ project-id map for this bridge.
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
            freshness: Arc::new(Mutex::new(HashMap::new())),
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
        // Resolve the new root's canonical CBM slug: known configured root â†’ its
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

    /// Resolve a caller-supplied project reference to the canonical CBM project slug.
    ///
    /// Resolution order:
    ///   1. If `raw` looks like a path (or canonicalizes to an existing path),
    ///      canonicalize and map through the authoritative rootâ†’slug map; derive+register on miss.
    ///   2. If `raw` is a known project slug â†’ it is used as-is.
    ///   3. If `raw` matches a configured root's directory basename â†’ that root's canonical slug.
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

        // 3. A configured root's basename (e.g. "RustContextLayerAI" â†’ primary slug).
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
    pub(super) fn ensure_tracked(&mut self, slug: &str) {
        if self.project_paths.contains_key(slug) {
            let already_gated = self.indexing_state().contains_key(slug);
            if !already_gated && self.is_available() {
                self.start_indexing_for(slug);
            }
        }
    }

    /// Resolve the ACTIVE project's canonical CBM slug.
    ///
    /// Since the canonical-identity fix, this is always a CBM-canonical path
    /// slug (never a directory basename). Used as:
    ///   - the `project` argument on every CBM tool call,
    ///   - the `indexing_state` key for readiness gating.
    ///     (The disk graph-cache partitions by the canonical root *path*,
    ///     `self.project_root` — a 1:1 equivalent of this slug per repo.)
    pub(crate) fn project_str(&self) -> String {
        self.project
            .clone()
            .unwrap_or_else(|| "default".to_string())
    }
}
