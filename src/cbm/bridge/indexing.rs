//! CBM indexing lifecycle for [`GraphBridge`]: reindexing, freshness marks, and
//! the non-blocking readiness gate. Split out of `bridge.rs` along the boundary
//! the file already had — no behavior changed.

use super::*;
use crate::cbm::client::CbmError;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

impl GraphBridge {
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

    /// Resolve the CBM project slug and repo root for a given file path.
    ///
    /// long and checks down from: the longest matching configured project root
    /// (canonicalized). Falls back to the active project if the file path
    /// doesn't match any known root.
    ///
    /// Returns `(project_slug, repo_root)`.
    pub(super) fn resolve_project_for_file(&self, file_path: &Path) -> (String, PathBuf) {
        let file_canon = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.to_path_buf());
        let repo_root = self
            .project_ids
            .keys()
            .filter(|root| file_canon.starts_with(*root))
            .max_by_key(|root| root.as_os_str().len())
            .cloned()
            .unwrap_or_else(|| self.project_root.clone());

        let project = self
            .project_ids
            .get(&repo_root)
            .cloned()
            .unwrap_or_else(|| self.project_str());

        (project, repo_root)
    }

    /// Mark the CBM project containing `file_path` as dirty.
    ///
    /// **Concurrency invariant:** This method must be called while holding
    /// `graph_bridge_lock()`. Because the lock provides mutual exclusion, an
    /// edit cannot mark a project dirty while a query's lazy reindex is in
    /// progress.
    ///
    /// `dirty_generation` is incremented on each call, enabling the lazy
    /// reindex on the next graph query. Multiple edits coalesce into a single
    /// lazy refresh because the indexed generation is only advanced after
    /// a successful reindex.
    pub fn mark_project_dirty(&mut self, file_path: &Path) {
        let (project, _repo_root) = self.resolve_project_for_file(file_path);

        let mut freshness_map = self.freshness.lock().unwrap_or_else(|p| p.into_inner());

        let entry = freshness_map.entry(project).or_insert(ProjectFreshness {
            dirty_generation: 0,
            indexed_generation: 0,
        });
        entry.dirty_generation += 1;
    }

    /// Check whether a project is dirty (needs reindexing before graph query).
    ///
    /// A project is dirty when:
    ///   - a freshness entry exists AND
    ///   - `dirty_generation > indexed_generation`
    ///
    /// No entry means the project is treated as clean (no edits have occurred).
    pub(crate) fn is_project_dirty(&self, project: &str) -> bool {
        let freshness_map = self.freshness.lock().unwrap_or_else(|p| p.into_inner());

        match freshness_map.get(project) {
            Some(entry) => entry.dirty_generation > entry.indexed_generation,
            None => false,
        }
    }

    /// Perform a synchronous lazy reindex of the active project.
    ///
    /// **Concurrency invariant:** This method is called while `graph_bridge_lock()`
    /// is held. The `client.lock()` provides serialization for the CBM call.
    ///
    /// On success, sets `indexed_generation = dirty_generation` (the generation
    /// that was current at the time the lock was acquired). Because no edit can
    /// acquire `graph_bridge_lock()` concurrently, the generation cannot change
    /// during the reindex — the snapshot is exact.
    ///
    /// On failure, the project remains dirty (indexed_generation is NOT advanced).
    pub(crate) fn reindex_active_project(&mut self, mode: &str) -> Result<(), CbmError> {
        let project = self.project_str();
        let repo_path = self.project_root.to_string_lossy().to_string();

        // Capture the dirty_generation before the CBM call.
        let dirty_gen = {
            let freshness_map = self.freshness.lock().unwrap_or_else(|p| p.into_inner());
            freshness_map
                .get(&project)
                .map(|e| e.dirty_generation)
                .unwrap_or(0)
        };

        eprintln!("[clean-ctx-cbm] Lazy reindex: {project} (repo: {repo_path}, mode: {mode})");

        // Acquire client lock and call index_repository.
        let mut cg = self.client.lock().unwrap_or_else(|p| p.into_inner());
        let client = match cg.as_mut() {
            Some(c) => c,
            None => return Err(CbmError::LaunchError("CBM not available".into())),
        };
        client.call_tool(
            "index_repository",
            serde_json::json!({"repo_path": repo_path, "mode": mode}),
        )?;
        drop(cg);

        // On success, advance indexed_generation to the captured dirty_generation.
        {
            let mut freshness_map = self.freshness.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(entry) = freshness_map.get_mut(&project) {
                entry.indexed_generation = dirty_gen;
            }
        }

        // Invalidate in-memory graph cache so pre-edit entries are not served.
        self.invalidate_cache();

        eprintln!("[clean-ctx-cbm] Lazy reindex complete for: {project}");
        Ok(())
    }

    /// Reindex the CBM project containing `file_path`.
    ///
    /// Resolves which configured project root `file_path` belongs to (longest
    /// matching prefix when roots are nested) and invokes CBM's native
    /// `index_repository` tool synchronously.
    ///
    /// **Does not change the active project** — the bridge's `project_root`,
    /// `project`, and `cache` (project-scoped) are left as-is. Only the
    /// in-memory result cache is invalidated after a successful index, so
    /// subsequent queries fetch fresh data from CBM rather than returning
    /// pre-edit cached entries.
    ///
    /// When `file_path` does not belong to any configured root, the active
    /// project is reindexed as a fallback.
    pub fn reindex_for_file(&mut self, file_path: &Path, mode: &str) -> Result<(), CbmError> {
        if !self.is_available() {
            return Err(CbmError::LaunchError("CBM not available".into()));
        }

        // Resolve the repo root for the edited file using the extracted helper.
        let (project, repo_root) = self.resolve_project_for_file(file_path);
        let repo_path = repo_root.to_string_lossy().to_string();

        eprintln!(
            "[clean-ctx-cbm] Reindexing project: {project} (repo: {repo_path}, mode: {mode})"
        );

        // Call CBM's index_repository tool synchronously through the client.
        // Acquire mutable access directly (single lock, no drop/reacquire).
        let mut cg = self.client.lock().unwrap_or_else(|p| p.into_inner());
        let client = match cg.as_mut() {
            Some(c) => c,
            None => return Err(CbmError::LaunchError("CBM not available".into())),
        };
        client.call_tool(
            "index_repository",
            serde_json::json!({"repo_path": repo_path, "mode": mode}),
        )?;
        // Release the lock before cache invalidation (no borrow conflict).
        drop(cg);

        // Invalidate the in-memory graph cache so pre-edit entries are
        // not served to subsequent queries as if they were still current.
        self.invalidate_cache();

        eprintln!("[clean-ctx-cbm] Reindex complete for: {project}");
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
    pub(super) fn start_indexing_for(&mut self, project: &str) {
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
    /// (`try_create` â†’ `start_indexing`), so this method no longer spawns a
    /// background indexing thread.** It only reports the current state:
    ///
    /// - `InProgress` â†’ `StillIndexing` (retry later); times out after 60s.
    /// - `Complete`   â†’ `Ready`.
    /// - `Failed`     â†’ `Err`.
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

        // ── Lazy freshness gate ──────────────────────────────────────
        // If the project is dirty (filesystem edits occurred since the last
        // successful reindex), perform a synchronous reindex before checking
        // the background indexing state. This ensures the graph query sees
        // the post-edit state without requiring an explicit index_repository.
        if self.is_project_dirty(&project) {
            self.reindex_active_project("fast")?;
        }
        // ── Existing IndexingState logic ─────────────────────────────

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

    /// Report the indexing state of a specific CBM project.
    ///
    /// Used by `cbm_proxy` so readiness is resolved against the project actually
    /// being queried. Semantics:
    ///   - The active project delegates to `ensure_indexed()`.
    ///   - A project that is NOT tracked (unknown slug) passes through as
    ///     `Ready` — CBM returns its authoritative error if the project doesn't
    ///     exist, so an unrelated/unknown project can never dead-end in
    ///     `StillIndexing{0}` forever.
    ///   - A tracked root reports its own per-project state (`Complete`â†’`Ready`,
    ///     `InProgress`â†’`StillIndexing`, `Failed`â†’`Err`, `NotStarted`â†’`StillIndexing{0}`).
    pub fn ensure_indexed_for(&mut self, project: &str) -> Result<IndexingStatus, CbmError> {
        // Guard before touching any state: when CBM is unavailable, return the
        // same error on every call (AUDIT-9 regression).
        if !self.is_available() {
            return Err(CbmError::LaunchError("CBM not available".into()));
        }
        if self.project.as_deref() != Some(project) {
            // Not the active project. Check freshness before proceeding.
            // ── Lazy freshness gate (non-active project) ───────────────
            if self.is_project_dirty(project) {
                // Reindex the specific target project.
                if let Some(repo_root) = self.project_paths.get(project) {
                    let repo_path = repo_root.to_string_lossy().to_string();

                    // Capture dirty_generation before the CBM call.
                    let dirty_gen = {
                        let freshness_map =
                            self.freshness.lock().unwrap_or_else(|p| p.into_inner());
                        freshness_map
                            .get(project)
                            .map(|e| e.dirty_generation)
                            .unwrap_or(0)
                    };

                    eprintln!(
                        "[clean-ctx-cbm] Lazy reindex (non-active): {project} (repo: {repo_path})"
                    );

                    let mut cg = self.client.lock().unwrap_or_else(|p| p.into_inner());
                    let client = match cg.as_mut() {
                        Some(c) => c,
                        None => return Err(CbmError::LaunchError("CBM not available".into())),
                    };
                    client.call_tool(
                        "index_repository",
                        serde_json::json!({"repo_path": repo_path, "mode": "fast"}),
                    )?;
                    drop(cg);

                    // On success, advance indexed_generation.
                    {
                        let mut freshness_map =
                            self.freshness.lock().unwrap_or_else(|p| p.into_inner());
                        if let Some(entry) = freshness_map.get_mut(project) {
                            entry.indexed_generation = dirty_gen;
                        }
                    }

                    self.invalidate_cache();

                    eprintln!("[clean-ctx-cbm] Lazy reindex complete for: {project}");
                }
            }

            // Unknown/untracked projects pass through.
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
        // Active project â†’ the active gate.
        self.ensure_indexed()
    }

    /// Access the indexing state map for inspection (e.g., get_cbm_status handler).
    pub fn indexing_state(&self) -> std::sync::MutexGuard<'_, HashMap<String, IndexingState>> {
        self.indexing_state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
    }
}
