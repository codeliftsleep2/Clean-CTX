//! Graph cache, transport, and status reporting for [`GraphBridge`].
//!
//! The TTL cache (memory plus disk write-through), cache invalidation, the raw
//! pipe-level `proxy_call`, and the typed query dispatch every graph query goes
//! through. Split out of `bridge.rs` along the boundary the file already had —
//! no behavior changed.

use super::*;
use crate::cbm::client::CbmClient;
use crate::cbm::client::CbmError;
use crate::cbm::config::CbmStatus;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use std::time::Instant;

impl GraphBridge {
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
    pub(super) fn set_last_error(&mut self, err: Option<CbmError>) {
        self.last_error = err;
    }

    /// Test-only: inject a stale error directly so tests can verify the
    /// cache-hit path clears it.
    #[cfg(test)]
    pub fn set_last_error_for_test(&mut self, err: CbmError) {
        self.set_last_error(Some(err));
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
    /// CBM produces a ~5000-token structural seed â†’ Clean-CTX catches it
    /// at the pipe level â†’ compresses to ~1100 tokens â†’ returns.
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

    // â”€â”€ Internal helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    /// Disk key scope: `{project_name}:{key}` so the effective disk
    /// partition is `(project_root, project_name)`. This prevents a
    /// cross-project data leak when `set_project("repo-b")` is called
    /// while `project_root` is still pinned to repo-a — repo-b queries
    /// would otherwise hydrate repo-a's cached results.
    pub(super) fn disk_key(&self, key: &str) -> String {
        format!("{}:{key}", self.project_str())
    }

    /// Check cache for a valid (non-expired) entry. Also evicts any
    /// expired entries found during lookup (H-3 fix: lazy GC).
    ///
    /// **Disk hydration:** On a memory miss, checks the disk cache first.
    /// If a valid entry exists on disk, it is hydrated into the in-memory
    /// `DashMap` (zero CBM round-trips) and `true` is returned. This avoids
    /// re-indexing CBM on process restart or when switching projects.
    pub(super) fn check_cache(&self, key: &str) -> bool {
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
    pub(super) fn cache_insert<T: Serialize>(&self, key: &str, value: &T) {
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
    pub(super) fn query<F, T>(&mut self, f: F) -> Result<T, CbmError>
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
