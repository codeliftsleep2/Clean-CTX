// src/cbm/cache_store.rs
//
// SQLite-backed disk cache for CBM graph query results.
//
// The `GraphBridge` keeps an in-memory `DashMap` of query results with TTL
// expiry. This store persists those entries to disk so that on process
// restart (or when switching between projects) the expensive CBM indexing
// (10-30s per project) can be avoided — query results are hydrated from disk
// on first touch instead of re-querying CBM.
//
// Design:
//   - Global DB by default, partitioned by `project_root` (the canonicalized
//     workspace root). `PerWorkspace` scope stores the DB under each project's
//     `.clean-ctx/` dir instead. Both scopes use the identical schema.
//   - WAL mode for concurrent read/write safety.
//   - Schema versioning via a `_schema_version` pragma table.
//   - Entries are keyed by `(project_root, cache_key)` where `cache_key` is
//     the same key used in the in-memory `DashMap` (project-scoped).
//   - `expires_at` is stored as unix epoch milliseconds; expired entries are
//     treated as misses and lazily purged.

use std::path::Path;
use rusqlite::{Connection, params};

/// SQLite-backed disk cache for CBM graph query results.
pub struct GraphCacheStore {
    conn: Connection,
}

impl GraphCacheStore {
    /// Open (or create) the SQLite database at the given path.
    /// Automatically creates the parent directory if it doesn't exist.
    pub fn open(db_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;

        // WAL mode for concurrent read/write safety
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Run schema migrations. Idempotent.
    fn migrate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS _schema_version (
                version INTEGER PRIMARY KEY
            );
        ")?;

        let current_version: i32 = self.conn
            .query_row("SELECT COALESCE(MAX(version), 0) FROM _schema_version", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        if current_version < 1 {
            self.conn.execute_batch("
                CREATE TABLE IF NOT EXISTS cbm_graph_cache (
                    project_root TEXT NOT NULL,
                    cache_key   TEXT NOT NULL,
                    data_json   TEXT NOT NULL,
                    expires_at  INTEGER NOT NULL,
                    PRIMARY KEY (project_root, cache_key)
                );

                CREATE INDEX IF NOT EXISTS idx_cbm_cache_project ON cbm_graph_cache(project_root);

                INSERT INTO _schema_version (version) VALUES (1);
            ")?;
        }

        Ok(())
    }

    /// Load a cache entry for the given project and key.
    ///
    /// Returns `Some(data_json)` if a valid (non-expired) entry exists,
    /// otherwise `None`. Expired entries are lazily purged on read.
    pub fn get(&self, project_root: &str, cache_key: &str) -> Option<String> {
        let now_ms = now_epoch_ms();
        let result = self.conn
            .query_row(
                "SELECT data_json, expires_at FROM cbm_graph_cache WHERE project_root = ?1 AND cache_key = ?2",
                params![project_root, cache_key],
                |row| {
                    let data: String = row.get(0)?;
                    let expires_at: i64 = row.get(1)?;
                    Ok((data, expires_at))
                },
            )
            .ok()?;

        let (data, expires_at) = result;
        if expires_at > now_ms {
            Some(data)
        } else {
            // Expired — purge lazily
            let _ = self.conn.execute(
                "DELETE FROM cbm_graph_cache WHERE project_root = ?1 AND cache_key = ?2",
                params![project_root, cache_key],
            );
            None
        }
    }

    /// Insert (or update) a cache entry for the given project and key.
    ///
    /// `expires_at` is unix epoch milliseconds. Write-through from the
    /// in-memory `DashMap` so disk and memory stay in sync.
    pub fn put(&self, project_root: &str, cache_key: &str, data_json: &str, expires_at_ms: i64) {
        let _ = self.conn.execute(
            "INSERT INTO cbm_graph_cache (project_root, cache_key, data_json, expires_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(project_root, cache_key) DO UPDATE SET
                 data_json = excluded.data_json,
                 expires_at = excluded.expires_at",
            params![project_root, cache_key, data_json, expires_at_ms],
        );
    }

    /// Invalidate all cache entries for a project (e.g. on graph version change).
    pub fn invalidate_project(&self, project_root: &str) {
        let _ = self.conn.execute(
            "DELETE FROM cbm_graph_cache WHERE project_root = ?1",
            params![project_root],
        );
    }

    /// Invalidate a single cache entry for a project.
    pub fn invalidate_key(&self, project_root: &str, cache_key: &str) {
        let _ = self.conn.execute(
            "DELETE FROM cbm_graph_cache WHERE project_root = ?1 AND cache_key = ?2",
            params![project_root, cache_key],
        );
    }

    /// Purge all expired entries across all projects. Returns the number removed.
    pub fn purge_expired(&self) -> usize {
        let now_ms = now_epoch_ms();
        self.conn
            .execute("DELETE FROM cbm_graph_cache WHERE expires_at <= ?1", params![now_ms])
            .unwrap_or(0)
    }

    /// Count entries for a project (used for diagnostics/stats).
    pub fn count_for_project(&self, project_root: &str) -> usize {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM cbm_graph_cache WHERE project_root = ?1",
                params![project_root],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0) as usize
    }
}

/// Current unix epoch time in milliseconds.
fn now_epoch_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "../tests/cbm/cache_store.rs"]
mod tests;