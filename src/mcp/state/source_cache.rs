// src/mcp/state/source_cache.rs
//
// The session's shared file-content cache (`McpState::source_cache`) and its
// single read boundary.
//
// Relocated verbatim from `src/mcp/state.rs` (which is legacy-oversized) when
// the hydration discovery cache was added to `McpState`; the source-cache
// concern is a self-contained semantic boundary (entry shape + metadata-based
// staleness detection + reader). Behavior is unchanged except for the
// discovery-invalidation hook documented at the staleness branch below.

use crate::mcp::McpState;
use std::sync::Arc;
use std::time::SystemTime;

/// P0-3: Cache entry with metadata for invalidation.
///
/// Tracks file modification time and size to detect when a cached
/// file has changed on disk. This prevents serving stale content
/// after the user edits a file.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    content: Arc<String>,
    mtime: SystemTime,
    size: u64,
}

impl McpState {
    /// Drop the cached source snapshot for `path` so the next
    /// `read_source` re-reads from disk (apply_edit Phase 3: called after
    /// a successful commit so session reads observe the new bytes even
    /// when mtime/size granularity hides the change).
    pub fn invalidate_source_cache(&self, path: &str) {
        let cache_key = Self::resolve_cache_key(path);
        self.source_cache_lock().remove(&cache_key);
    }

    /// Resolve a cache key for `source_cache`. On Windows, `canonicalize`
    /// on TempDir paths can trigger Defender deep-scan hooks (10-30s per
    /// call). We skip canonicalize when the path has no relative components,
    /// falling back to the raw string as the key.
    ///
    /// P3-18: Uses `Path::components()` for robust detection of relative
    /// path components instead of simple string contains(), which could
    /// miss edge cases on Windows with mixed path separators
    /// (e.g., "C:\foo\.\bar" or "C:\foo\..\bar").
    fn resolve_cache_key(path: &str) -> String {
        use std::path::{Component, Path};
        let p = Path::new(path);

        // Fast path: check if path is absolute and has no relative components
        // using the robust Path::components() iterator instead of string contains().
        if p.is_absolute()
            && p.components().all(|c| {
                matches!(
                    c,
                    Component::Normal(_) | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            #[cfg(debug_assertions)]
            eprintln!("[resolve_cache_key] FAST PATH: {}", path);
            return path.to_string();
        }
        #[cfg(debug_assertions)]
        eprintln!("[resolve_cache_key] SLOW PATH (canonicalize): {}", path);
        #[cfg(debug_assertions)]
        let canon_start = std::time::Instant::now();
        let result = p
            .canonicalize()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string());
        #[cfg(debug_assertions)]
        eprintln!(
            "[resolve_cache_key] canonicalize took {:?} for {}",
            canon_start.elapsed(),
            path
        );
        result
    }

    /// F-FULL-01/F-FULL-05: Read file content, using the shared source cache.
    /// Returns `Arc<String>` so the cache can be shared across passes
    /// without cloning the underlying string data.
    ///
    /// **Two-phase locking:** The Mutex is held only during cache lookup
    /// and update, NOT during `read_to_string`. This prevents I/O from
    /// blocking concurrent readers.
    ///
    /// P0-3: Cache entries include mtime and size for invalidation.
    /// If the file has changed since it was cached, we re-read it.
    pub fn read_source(&self, path: &str) -> Result<Arc<String>, std::io::Error> {
        let cache_key = Self::resolve_cache_key(path);
        #[cfg(debug_assertions)]
        let overall_start = std::time::Instant::now();

        // Get file metadata for cache invalidation
        let metadata = std::fs::metadata(path)?;
        let current_mtime = metadata.modified()?;
        let current_size = metadata.len();

        // Phase 1: Check cache (brief lock, release before I/O)
        {
            #[cfg(debug_assertions)]
            let lock_start = std::time::Instant::now();
            let cache = self.source_cache_lock();
            #[cfg(debug_assertions)]
            eprintln!(
                "[read_source] Phase 1 lock acquire took {:?} for {}",
                lock_start.elapsed(),
                path
            );
            if let Some(cached) = cache.get(&cache_key) {
                // P0-3: Check if file has changed using mtime and size
                if cached.mtime == current_mtime && cached.size == current_size {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "[read_source] CACHE HIT for {} (total: {:?})",
                        path,
                        overall_start.elapsed()
                    );
                    return Ok(Arc::clone(&cached.content));
                }
                // External modification detected via the existing mtime/size
                // staleness path. The workspace now holds source Clean-CTX
                // never observed, so previously completed hydration discovery
                // may no longer be complete. Dropping the discovery cache is
                // cheap and fails safe (the next hydration rediscovers).
                //
                // Covering limits: only files this session has already read
                // through `source_cache` can be detected here. A file that
                // appears on disk and is never read stays invisible — the
                // documented external-edit boundary (see
                // docs/CLAUDE_INTEGRATION_RULES.md: use `index_repository`).
                self.invalidate_hydration_discovery_all();
                #[cfg(debug_assertions)]
                eprintln!(
                    "[read_source] CACHE STALE for {} (mtime/size changed)",
                    path
                );
            }
            #[cfg(debug_assertions)]
            eprintln!("[read_source] CACHE MISS for {}", path);
        }

        // Phase 2: Read file WITHOUT holding the lock
        #[cfg(debug_assertions)]
        let io_start = std::time::Instant::now();
        let content = Arc::new(std::fs::read_to_string(path)?);
        #[cfg(debug_assertions)]
        eprintln!(
            "[read_source] Phase 2 read_to_string took {:?} for {} ({} bytes)",
            io_start.elapsed(),
            path,
            content.len()
        );

        // Phase 3: Update cache (brief lock, with double-check)
        #[cfg(debug_assertions)]
        let lock2_start = std::time::Instant::now();
        let mut cache = self.source_cache_lock();
        #[cfg(debug_assertions)]
        eprintln!(
            "[read_source] Phase 3 lock acquire took {:?} for {}",
            lock2_start.elapsed(),
            path
        );

        // P0-3: Insert or REFRESH the cache entry with current metadata.
        //
        // Cache-refresh defect fix (2026-08-25): this previously used
        // `cache.entry(cache_key).or_insert(...)`, which is a NO-OP
        // whenever the key already exists — exactly the stale-entry case
        // Phase 1 just detected. After any external file modification,
        // the stale entry survived forever: every subsequent read took
        // the STALE branch and re-read from disk (permanent cache-miss:
        // stat + full I/O + double lock per read) while pinning the old
        // content Arc in memory. Plain `insert` overwrites the entry so
        // the next read converges back to a genuine cache HIT.
        // Concurrency behavior is unchanged: the map is still mutated
        // under the single brief Phase-3 lock; outstanding Arc clones
        // held by other readers remain valid immutable snapshots.
        cache.insert(
            cache_key,
            CacheEntry {
                content: Arc::clone(&content),
                mtime: current_mtime,
                size: current_size,
            },
        );

        #[cfg(debug_assertions)]
        eprintln!(
            "[read_source] TOTAL for {}: {:?}",
            path,
            overall_start.elapsed()
        );

        Ok(content)
    }
}

#[cfg(test)]
#[path = "../../tests/mcp/state.rs"]
mod tests;
