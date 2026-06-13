// src/mcp/sqlite_store.rs
//
// SQLite-backed implementation of [`ContextStore`].
//
// Uses WAL mode for concurrent read/write safety.
// Schema versioning via a `_schema_version` pragma table.
//
// Design decisions:
//   - Lives in `mcp` module (not `ir`) to avoid circular dependency —
//     `ir` does NOT depend on `mcp`, but `mcp` depends on `ir`.
//   - Context ID is content-hash-derived for deterministic, idempotent saves.
//   - Version tracking derived from `COUNT(deltas) + 1`.
//   - `binary_wire::decode()` loses `file_id` and `version` (Gap 2 in plan);
//     those are restored from DB columns on load.
//   - LOW-05: SqliteStore is now wrapped in `Arc<Mutex<>>` by BufferedStore
//     for the retry/fallback pattern. The comment has been updated to reflect
//     that the store is used in a multi-threaded context (retry with sleep).

use std::path::Path;
use rusqlite::{Connection, params};
use crate::compression::Fidelity;
use crate::mcp::context_store::{ContextStore, StoredContextMeta};
use crate::ir::compiler::CompiledIR;

/// SQLite-backed implementation of [`ContextStore`].
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
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
                CREATE TABLE IF NOT EXISTS contexts (
                    id TEXT PRIMARY KEY,
                    file_path TEXT NOT NULL,
                    content_hash TEXT NOT NULL UNIQUE,
                    fidelity INTEGER NOT NULL,
                    ir_binary BLOB NOT NULL,
                    pretty_text TEXT,
                    symbol_table_json TEXT,
                    metadata TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE TABLE IF NOT EXISTS deltas (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    context_id TEXT REFERENCES contexts(id) ON DELETE CASCADE,
                    edit_sequence INTEGER NOT NULL,
                    delta_payload BLOB NOT NULL,
                    raw_tokens INTEGER,
                    delta_tokens INTEGER,
                    edit_type TEXT,
                    description TEXT,
                    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE TABLE IF NOT EXISTS symbols (
                    symbol_id TEXT PRIMARY KEY,
                    context_id TEXT REFERENCES contexts(id),
                    name TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    phi_markers TEXT,
                    last_seen TEXT
                );

                CREATE TABLE IF NOT EXISTS sessions (
                    session_id TEXT PRIMARY KEY,
                    workspace_root TEXT NOT NULL,
                    active_contexts TEXT,
                    last_active TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE INDEX IF NOT EXISTS idx_contexts_file ON contexts(file_path);
                CREATE INDEX IF NOT EXISTS idx_deltas_context ON deltas(context_id, edit_sequence);
                CREATE INDEX IF NOT EXISTS idx_symbols_context ON symbols(context_id);

                INSERT INTO _schema_version (version) VALUES (1);
            ")?;
        }

        if current_version < 2 {
            // v2: Add token count columns so rebuild_stats() uses real data
            self.conn.execute_batch("
                ALTER TABLE contexts ADD COLUMN raw_tokens INTEGER NOT NULL DEFAULT 0;
                ALTER TABLE contexts ADD COLUMN compressed_tokens INTEGER NOT NULL DEFAULT 0;
                INSERT INTO _schema_version (version) VALUES (2);
            ")?;
        }

        Ok(())
    }

    /// Load a context and replay deltas up to a target version.
    /// Returns the reconstructed CompiledIR and the final version reached.
    pub fn load_context_with_deltas(
        &self,
        file_path: &str,
        target_sequence: Option<u32>,
    ) -> Result<Option<(CompiledIR, u32)>, Box<dyn std::error::Error>> {
        // 1. Load baseline IR from contexts table
        let mut stmt = self.conn.prepare(
            "SELECT ir_binary, content_hash, fidelity FROM contexts WHERE file_path = ?1 ORDER BY updated_at DESC LIMIT 1"
        )?;

        let row = match stmt.query_row(params![file_path], |row| {
            let blob: Vec<u8> = row.get(0)?;
            let hash: String = row.get(1)?;
            let fid: i32 = row.get(2)?;
            Ok((blob, hash, fid))
        }) {
            Ok(r) => r,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // 2. Decode baseline IR
        let mut ir = crate::ir::binary_wire::decode(&row.0)?;
        ir.file_id = file_path.to_string();

        // 3. Load deltas up to target_sequence
        let max_seq = target_sequence.unwrap_or(u32::MAX);
        let mut delta_stmt = self.conn.prepare(
            "SELECT edit_sequence, delta_payload FROM deltas
             WHERE context_id = (SELECT id FROM contexts WHERE file_path = ?1 ORDER BY updated_at DESC LIMIT 1)
             AND edit_sequence <= ?2
             ORDER BY edit_sequence"
        )?;

        let delta_rows = delta_stmt.query_map(params![file_path, max_seq as i32], |row| {
            let seq: i32 = row.get(0)?;
            let payload: Vec<u8> = row.get(1)?;
            Ok((seq, payload))
        })?;

        // 4. Build a ContextState and replay deltas
        let mut context_state = crate::ir::replay::ContextState::new();
        context_state.load_ir(ir);

        for delta_result in delta_rows {
            let (_seq, payload) = delta_result?;
            let delta: crate::ir::delta::IRDelta = serde_json::from_slice(&payload)?;
            context_state.apply(delta)?;
        }

        // 5. Reconstruct CompiledIR from context state
        let instructions = context_state.get_ir(file_path)
            .cloned()
            .unwrap_or_default();
        let version = context_state.file_version(file_path).unwrap_or(1);

        let instructions_ops: Vec<crate::ir::opcodes::CoreOp> = instructions
            .iter()
            .filter_map(|t| crate::ir::wire::tuple_to_op(t))
            .collect();

        let final_ir = CompiledIR {
            file_id: file_path.to_string(),
            instructions: instructions_ops,
            version,
        };

        Ok(Some((final_ir, version as u32)))
    }

    /// Rebuild SessionStats from DB (schema v2 — reads real token counts).
    ///
    /// Queries all contexts and their delta counts to reconstruct
    /// a SessionStats that reflects what's persisted.
    pub fn rebuild_stats(&self) -> Result<crate::mcp::session_stats::SessionStats, Box<dyn std::error::Error>> {
        let mut stats = crate::mcp::session_stats::SessionStats::new();

        let mut stmt = self.conn.prepare(
            "SELECT c.file_path, c.fidelity,
                    COALESCE(c.raw_tokens, 0) as raw_tokens,
                    COALESCE(c.compressed_tokens, 0) as compressed_tokens,
                    (SELECT COUNT(*) FROM deltas d WHERE d.context_id = c.id) as delta_count
             FROM contexts c"
        )?;

        let rows = stmt.query_map([], |row| {
            let path: String = row.get(0)?;
            let fid: i32 = row.get(1)?;
            let raw_tokens: i64 = row.get(2)?;
            let compressed_tokens: i64 = row.get(3)?;
            let dc: i32 = row.get(4)?;
            Ok((path, fid, raw_tokens, compressed_tokens, dc))
        })?;

        for row in rows {
            let (path, fid, raw_tokens, compressed_tokens, dc) = row?;
            let strategy = if dc > 0 { "delta" } else { "full" };
            // Use real token counts from the DB, falling back to estimates
            // for rows created before the v2 migration.
            let raw = if raw_tokens > 0 { raw_tokens as usize } else { 0 };
            let compressed = if compressed_tokens > 0 { compressed_tokens as usize } else { 0 };
            let fidelity_str = match fid {
                0 => "low",
                1 => "medium",
                2 => "high",
                _ => "low",
            };
            stats.record_compression(
                &path, raw, compressed, fidelity_str, false, strategy
            );
        }

        Ok(stats)
    }

    /// Begin a SQLite transaction.
    pub fn begin_transaction(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute_batch("BEGIN TRANSACTION")?;
        Ok(())
    }

    /// Commit the current SQLite transaction.
    pub fn commit(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute_batch("COMMIT")?;
        Ok(())
    }

    /// Rollback the current SQLite transaction.
    pub fn rollback(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute_batch("ROLLBACK")?;
        Ok(())
    }

    /// Execute a WAL checkpoint to keep file size bounded.
    pub fn wal_checkpoint(&self) {
        let _ = self.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    }

    /// Purge deltas older than the specified number of days.
    /// Returns the number of deltas deleted.
    pub fn purge_old_deltas(&self, days: u32) -> Result<usize, Box<dyn std::error::Error>> {
        let affected = self.conn.execute(
            "DELETE FROM deltas WHERE applied_at < datetime('now', ?1)",
            params![format!("-{} days", days)],
        )?;
        Ok(affected)
    }

/// Get delta count for a specific file (helper for purge handler).
    pub fn delta_count_for_file(&self, file_path: &str) -> usize {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM deltas WHERE context_id IN (SELECT id FROM contexts WHERE file_path = ?1)",
                params![file_path],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0) as usize
    }
}

/// Parse a SQLite datetime string (e.g. "2026-06-12 18:47:09") into SystemTime.
/// SQLite timestamps are naive (no timezone), so we use NaiveDateTime
/// and treat them as UTC. Falls back to SystemTime::now() if parsing fails.
fn chrono_parse_or_now(dt_str: &str) -> std::time::SystemTime {
    // SQLite datetime('now') produces "YYYY-MM-DD HH:MM:SS" format
    use std::time::Duration;
    if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S") {
        let secs = parsed.and_utc().timestamp() as u64;
        std::time::UNIX_EPOCH + Duration::from_secs(secs)
    } else {
        std::time::SystemTime::now()
    }
}

impl ContextStore for SqliteStore {
    fn save_context(
        &mut self,
        file_path: &str,
        fidelity: Fidelity,
        compressed_output: &str,
        ir_blobs: Option<&[u8]>,
        source_hash: &str,
        raw_tokens: u64,
        compressed_tokens: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let id = format!("ctx-{}", source_hash); // deterministic ID from content hash
        let fid = fidelity as i32;
        let ir_binary = ir_blobs.unwrap_or(&[]);

        self.conn.execute(
            "INSERT OR REPLACE INTO contexts (id, file_path, content_hash, fidelity, ir_binary, pretty_text, updated_at, raw_tokens, compressed_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), ?7, ?8)",
            params![id, file_path, source_hash, fid, ir_binary, compressed_output, raw_tokens as i64, compressed_tokens as i64],
        )?;

        Ok(id)
    }

    fn load_latest(
        &self,
        file_path: &str,
    ) -> Result<Option<StoredContextMeta>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.file_path, c.content_hash, c.fidelity, c.pretty_text, c.created_at,
                    (SELECT COUNT(*) FROM deltas d WHERE d.context_id = c.id) as delta_count
             FROM contexts c WHERE c.file_path = ?1 ORDER BY c.updated_at DESC LIMIT 1"
        )?;

        let mut rows = stmt.query(params![file_path])?;
        if let Some(row) = rows.next()? {
            let _id: String = row.get(0)?;
            let fp: String = row.get(1)?;
            let hash: String = row.get(2)?;
            let fid: i32 = row.get(3)?;
            let _pretty: Option<String> = row.get(4)?;
            let created_at_str: String = row.get(5)?;
            let delta_count: i32 = row.get(6)?;

            // Parse Fidelity from i32
            let fidelity = match fid {
                0 => Fidelity::Low,
                1 => Fidelity::Medium,
                2 => Fidelity::High,
                _ => Fidelity::Low,
            };

            // HIGH-01 fix: compute version from delta count
            let version = (delta_count as u64) + 1;

            // HIGH-02 fix: parse actual created_at from DB
            let created_at = chrono_parse_or_now(&created_at_str);

            Ok(Some(StoredContextMeta {
                file_path: fp,
                fidelity,
                version,
                is_angular: false,
                source_hash: hash,
                created_at,
                raw_tokens: 0,
                compressed_tokens: 0,
            }))
        } else {
            Ok(None)
        }
    }

    fn has_context(&self, file_path: &str) -> bool {
        self.conn
            .query_row(
                "SELECT 1 FROM contexts WHERE file_path = ?1 LIMIT 1",
                params![file_path],
                |_| Ok(()),
            )
            .is_ok()
    }

    fn append_delta(
        &mut self,
        context_id: &str,
        delta_payload: &[u8],
        edit_type: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get the next edit_sequence
        let next_seq: i32 = self.conn
            .query_row(
                "SELECT COALESCE(MAX(edit_sequence), 0) + 1 FROM deltas WHERE context_id = ?1",
                params![context_id],
                |row| row.get(0),
            )
            .unwrap_or(1);

        self.conn.execute(
            "INSERT INTO deltas (context_id, edit_sequence, delta_payload, edit_type, applied_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
            params![context_id, next_seq, delta_payload, edit_type],
        )?;

        // Update the context's updated_at
        self.conn.execute(
            "UPDATE contexts SET updated_at = datetime('now') WHERE id = ?1",
            params![context_id],
        )?;

        Ok(())
    }

    fn delta_count(&self, context_id: &str) -> usize {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM deltas WHERE context_id = ?1",
                params![context_id],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0) as usize
    }

    fn clear_file(&mut self, file_path: &str) {
        if let Err(e) = self.conn.execute(
            "DELETE FROM contexts WHERE file_path = ?1",
            params![file_path],
        ) {
            eprintln!("[clean-ctx] Failed to clear file from DB: {e}");
        }
    }
}

#[cfg(test)]
#[path = "../tests/mcp/sqlite_store.rs"]
mod tests;
