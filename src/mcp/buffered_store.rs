// src/mcp/buffered_store.rs
//
// Buffered persistence layer with three-tier defense:
//
//   Tier 1: Batched writes — ops accumulate in memory, flushed as a
//           single SQLite transaction when the buffer hits the threshold.
//   Tier 2: Retry with exponential backoff — transient DB failures
//           (file lock, WAL contention) are retried up to MAX_RETRIES.
//   Tier 3: JSON file fallback — if all retries fail, ops are written
//           as standalone JSON files in .clean-ctx/fallback/.
//           On next successful flush, fallback files are re-imported.
//
// Flush boundaries:
//   - Auto-flush when pending.len() >= BATCH_THRESHOLD (5)
//   - Explicit flush via `context_stats` handler
//   - Server shutdown (future: flush on drop)

use crate::compression::Fidelity;
use crate::mcp::context_store::{ContextStore, StoredContextMeta};
use crate::mcp::sqlite_store::SqliteStore;
use crate::ir::compiler::CompiledIR;
use base64::Engine;
use std::sync::Arc;
use std::sync::Mutex;
use std::path::PathBuf;
use std::time::Duration;

/// Auto-flush when the buffer reaches this many pending ops.
const BATCH_THRESHOLD: usize = 5;

/// Maximum retry attempts for flush (exponential backoff).
const MAX_RETRIES: u32 = 3;

/// Backoff delays between retries: [0ms, 50ms, 200ms].
const BACKOFF_MS: &[u64] = &[0, 50, 200];

/// A queued write operation for the persistence buffer.
enum WriteOp {
    SaveContext {
        file_path: String,
        fidelity: Fidelity,
        compressed_output: String,
        ir_binary: Vec<u8>,
        source_hash: String,
        raw_tokens: u64,
        compressed_tokens: u64,
    },
    AppendDelta {
        context_id: String,
        delta_payload: Vec<u8>,
        edit_type: Option<String>,
    },
    ClearFile {
        file_path: String,
    },
}

/// Buffered SQLite persistence store with retry and fallback.
///
/// Three-tier defense:
///   1. Batched writes in SQLite transactions
///   2. Retry with exponential backoff on transient failures
///   3. JSON file fallback for total DB failure
#[derive(Clone)]
pub struct BufferedStore {
    /// Inner SQLite store (shared across clones).
    inner: Arc<Mutex<SqliteStore>>,
    /// Pending write operations (not yet flushed).
    pending: Arc<Mutex<Vec<WriteOp>>>,
    /// Project root for resolving fallback file paths.
    project_root: PathBuf,
}

impl BufferedStore {
    /// Create a new buffered store wrapping the given SQLite store.
    pub fn new(store: SqliteStore, project_root: PathBuf) -> Self {
        Self {
            inner: Arc::new(Mutex::new(store)),
            pending: Arc::new(Mutex::new(Vec::new())),
            project_root,
        }
    }

    /// Flush all pending writes to SQLite in a single transaction.
    /// Retries up to MAX_RETRIES with exponential backoff.
    /// Falls back to JSON files if all retries fail.
    /// Returns the number of operations flushed.
    pub fn flush(&self) -> usize {
        // Tier 3: check for fallback files to re-import first
        self.reimport_fallback_files();

        // Drain pending queue
        // P1-2: Recover from poisoned lock instead of discarding pending writes
        let ops = match self.pending.lock() {
            Ok(mut p) => std::mem::take(&mut *p),
            Err(e) => {
                eprintln!("[clean-ctx] WARNING: pending mutex poisoned, recovering: {e}");
                let mut p = e.into_inner();
                std::mem::take(&mut *p)
            }
        };
        if ops.is_empty() {
            return 0;
        }

        let count = ops.len();

        // Tier 2: retry with exponential backoff
        #[allow(unused_assignments)]
        let mut last_err = String::new();
        let mut succeeded = false;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let delay = BACKOFF_MS.get(attempt as usize).copied().unwrap_or(200);
                std::thread::sleep(Duration::from_millis(delay));
            }

            match self.try_flush_ops(&ops) {
                Ok(n) => {
                    eprintln!("[clean-ctx] Buffered flush OK: {} ops (attempt {})", n, attempt + 1);
                    succeeded = true;
                    break;
                }
                Err(e) => {
                    last_err = e;
                    eprintln!("[clean-ctx] Buffered flush attempt {} failed: {last_err}", attempt + 1);
                }
            }
        }

        if !succeeded {
            // Tier 3: write ops to fallback JSON files
            eprintln!("[clean-ctx] All flush attempts failed. Writing {} ops to fallback files.", ops.len());
            self.write_fallback_files(&ops);
        }

        count
    }

    /// Try to flush ops in a single SQLite transaction.
    fn try_flush_ops(&self, ops: &[WriteOp]) -> Result<usize, String> {
        let mut conn = self.inner.lock().map_err(|e| format!("lock poisoned: {e}"))?;

        conn.begin_transaction().map_err(|e| format!("BEGIN failed: {e}"))?;

        let mut flushed = 0;
        for op in ops {
            match op {
                WriteOp::SaveContext { file_path, fidelity, compressed_output, ir_binary, source_hash, raw_tokens, compressed_tokens } => {
                    if let Err(e) = crate::mcp::context_store::ContextStore::save_context(
                        &mut *conn,
                        file_path,
                        *fidelity,
                        compressed_output,
                        Some(ir_binary),
                        source_hash,
                        *raw_tokens,
                        *compressed_tokens,
                    ) {
                        let _ = conn.rollback();
                        return Err(format!("save_context failed: {e}"));
                    }
                }
                WriteOp::AppendDelta { context_id, delta_payload, edit_type } => {
                    if let Err(e) = crate::mcp::context_store::ContextStore::append_delta(
                        &mut *conn,
                        context_id,
                        delta_payload,
                        edit_type.as_deref(),
                    ) {
                        let _ = conn.rollback();
                        return Err(format!("append_delta failed: {e}"));
                    }
                }
                WriteOp::ClearFile { file_path } => {
                    conn.clear_file(file_path);
                }
            }
            flushed += 1;
        }

        conn.commit().map_err(|e| format!("COMMIT failed: {e}"))?;
        conn.wal_checkpoint();

        Ok(flushed)
    }

    /// Write failed ops to JSON fallback files.
    ///
    /// LOW-03: Timestamp collision is a non-issue because the index prefix
    /// (`op_{i}_{ts}.json`) uniquely identifies each operation even if
    /// multiple ops share the same nanosecond timestamp.
    fn write_fallback_files(&self, ops: &[WriteOp]) {
        let fallback_dir = self.project_root.join(".clean-ctx").join("fallback");
        let _ = std::fs::create_dir_all(&fallback_dir);

        for (i, op) in ops.iter().enumerate() {
            let filename = format!("op_{}_{:016x}.json", i,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            let path = fallback_dir.join(&filename);

            let json = match op {
                WriteOp::SaveContext { file_path, fidelity, compressed_output, ir_binary, source_hash, raw_tokens, compressed_tokens } => {
                    serde_json::json!({
                        "type": "save_context",
                        "file_path": file_path,
                        "fidelity": format!("{:?}", fidelity),
                        "compressed_output": compressed_output,
                        "ir_binary": base64::engine::general_purpose::STANDARD.encode(ir_binary),
                        "source_hash": source_hash,
                        "raw_tokens": raw_tokens,
                        "compressed_tokens": compressed_tokens,
                    })
                }
                WriteOp::AppendDelta { context_id, delta_payload, edit_type } => {
                    serde_json::json!({
                        "type": "append_delta",
                        "context_id": context_id,
                        "delta_payload": base64::engine::general_purpose::STANDARD.encode(delta_payload),
                        "edit_type": edit_type,
                    })
                }
                WriteOp::ClearFile { file_path } => {
                    serde_json::json!({
                        "type": "clear_file",
                        "file_path": file_path,
                    })
                }
            };

            if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap_or_default()) {
                eprintln!("[clean-ctx] Fallback write FAILED: {e}");
            }
        }
    }

    /// Re-import any fallback files into SQLite on next successful flush.
    fn reimport_fallback_files(&self) {
        let fallback_dir = self.project_root.join(".clean-ctx").join("fallback");
        if !fallback_dir.exists() {
            return;
        }

        let entries: Vec<_> = match std::fs::read_dir(&fallback_dir) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
            Err(_) => return,
        };

        if entries.is_empty() {
            return;
        }

        let mut conn = match self.inner.lock() {
            Ok(c) => c,
            Err(_) => return,
        };

        // Disable foreign key constraints during reimport to allow
        // append_delta operations to be processed before their
        // corresponding save_context (fallback files may be out of order).
        let _ = conn.execute_batch("PRAGMA foreign_keys=OFF;");

        let mut reimported = 0;
        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let json: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let op_type = json["type"].as_str().unwrap_or("");
            match op_type {
                "save_context" => {
                    let file_path = json["file_path"].as_str().unwrap_or("");
                    let fidelity_str = json["fidelity"].as_str().unwrap_or("low");
                    let fidelity = Fidelity::parse(fidelity_str).unwrap_or(Fidelity::Low);
                    let compressed = json["compressed_output"].as_str().unwrap_or("");
                    let source_hash = json["source_hash"].as_str().unwrap_or("");
                    let ir_b64 = json["ir_binary"].as_str().unwrap_or("");
                    let ir_binary = base64::engine::general_purpose::STANDARD.decode(ir_b64).unwrap_or_default();
                    let raw_tokens = json["raw_tokens"].as_u64().unwrap_or(0);
                    let compressed_tokens = json["compressed_tokens"].as_u64().unwrap_or(0);

                    if let Err(e) = conn.save_context(
                        file_path, fidelity, compressed,
                        Some(&ir_binary), source_hash,
                        raw_tokens, compressed_tokens,
                    ) {
                        eprintln!("[clean-ctx] Fallback reimport save_context failed: {e}");
                        continue;
                    }
                    reimported += 1;
                }
                "append_delta" => {
                    let context_id = json["context_id"].as_str().unwrap_or("");
                    let payload_b64 = json["delta_payload"].as_str().unwrap_or("");
                    let payload = base64::engine::general_purpose::STANDARD.decode(payload_b64).unwrap_or_default();
                    let edit_type = json["edit_type"].as_str();

                    if let Err(e) = conn.append_delta(context_id, &payload, edit_type) {
                        eprintln!("[clean-ctx] Fallback reimport append_delta failed: {e}");
                        continue;
                    }
                    reimported += 1;
                }
                "clear_file" => {
                    let file_path = json["file_path"].as_str().unwrap_or("");
                    conn.clear_file(file_path);
                    reimported += 1;
                }
                _ => continue,
            }

            // Delete the fallback file after successful reimport
            let _ = std::fs::remove_file(&path);
        }

        // Re-enable foreign key constraints
        let _ = conn.execute_batch("PRAGMA foreign_keys=ON;");

        if reimported > 0 {
            eprintln!("[clean-ctx] Reimported {reimported} ops from fallback files.");
        }
    }

    /// Get a reference to the inner SQLite store for read-only operations.
    pub fn sqlite(&self) -> Option<std::sync::MutexGuard<'_, SqliteStore>> {
        self.inner.lock().ok()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.lock().map(|p| p.len()).unwrap_or(0)
    }

    /// Queue a save_context operation. Auto-flushes if threshold reached.
    #[allow(clippy::too_many_arguments)]
    pub fn queue_save_context(
        &self,
        file_path: &str,
        fidelity: Fidelity,
        compressed_output: &str,
        ir_binary: &[u8],
        source_hash: &str,
        raw_tokens: u64,
        compressed_tokens: u64,
    ) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.push(WriteOp::SaveContext {
                file_path: file_path.to_string(),
                fidelity,
                compressed_output: compressed_output.to_string(),
                ir_binary: ir_binary.to_vec(),
                source_hash: source_hash.to_string(),
                raw_tokens,
                compressed_tokens,
            });
            let len = pending.len();
            if len >= BATCH_THRESHOLD {
                drop(pending);
                self.flush();
            }
        }
    }

    /// Queue an append_delta operation. Auto-flushes if threshold reached.
    pub fn queue_append_delta(
        &self,
        context_id: &str,
        delta_payload: &[u8],
        edit_type: Option<&str>,
    ) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.push(WriteOp::AppendDelta {
                context_id: context_id.to_string(),
                delta_payload: delta_payload.to_vec(),
                edit_type: edit_type.map(String::from),
            });
            let len = pending.len();
            if len >= BATCH_THRESHOLD {
                drop(pending);
                self.flush();
            }
        }
    }

    /// Queue a clear_file operation. Auto-flushes if threshold reached (MED-01 fix).
    pub fn queue_clear_file(&self, file_path: &str) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.push(WriteOp::ClearFile {
                file_path: file_path.to_string(),
            });
            let len = pending.len();
            if len >= BATCH_THRESHOLD {
                drop(pending);
                self.flush();
            }
        }
    }
}

impl ContextStore for BufferedStore {
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
        let id = format!("ctx-{}", source_hash);
        if let Ok(mut pending) = self.pending.lock() {
            pending.push(WriteOp::SaveContext {
                file_path: file_path.to_string(),
                fidelity,
                compressed_output: compressed_output.to_string(),
                ir_binary: ir_blobs.unwrap_or(&[]).to_vec(),
                source_hash: source_hash.to_string(),
                raw_tokens,
                compressed_tokens,
            });
            let len = pending.len();
            if len >= BATCH_THRESHOLD {
                drop(pending);
                self.flush();
            }
        }
        Ok(id)
    }

    fn load_latest(
        &self,
        file_path: &str,
    ) -> Result<Option<StoredContextMeta>, Box<dyn std::error::Error>> {
        // MED-02: Flush pending ops and read in a single lock scope to avoid
        // the double-lock that occurs when flush() acquires the inner lock
        // then releases it before sqlite() acquires it again.
        // P1-2: Recover from poisoned lock instead of discarding pending writes
        let ops = match self.pending.lock() {
            Ok(mut p) => std::mem::take(&mut *p),
            Err(e) => {
                eprintln!("[clean-ctx] WARNING: pending mutex poisoned, recovering: {e}");
                let mut p = e.into_inner();
                std::mem::take(&mut *p)
            }
        };
        let mut conn = match self.inner.lock() {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };
        if !ops.is_empty() {
            // Best-effort flush inside the same lock scope
            if let Err(e) = conn.begin_transaction() {
                eprintln!("[clean-ctx] BEGIN failed during load_latest flush: {e}");
                // Re-queue all ops so they're not lost
                if let Ok(mut pending) = self.pending.lock() {
                    pending.splice(0..0, ops);
                }
            } else {
                let mut flushed = false;
                let mut failed_at = None;
                for (idx, op) in ops.iter().enumerate() {
                    match op {
                        WriteOp::SaveContext { file_path, fidelity, compressed_output, ir_binary, source_hash, raw_tokens, compressed_tokens } => {
                            if let Err(e) = crate::mcp::context_store::ContextStore::save_context(
                                &mut *conn, file_path, *fidelity, compressed_output,
                                Some(ir_binary), source_hash, *raw_tokens, *compressed_tokens,
                            ) {
                                let _ = conn.rollback();
                                eprintln!("[clean-ctx] save_context during load_latest flush failed: {e}");
                                failed_at = Some(idx);
                                break;
                            }
                        }
                        WriteOp::AppendDelta { context_id, delta_payload, edit_type } => {
                            if let Err(e) = crate::mcp::context_store::ContextStore::append_delta(
                                &mut *conn, context_id, delta_payload, edit_type.as_deref(),
                            ) {
                                let _ = conn.rollback();
                                eprintln!("[clean-ctx] append_delta during load_latest flush failed: {e}");
                                failed_at = Some(idx);
                                break;
                            }
                        }
                        WriteOp::ClearFile { file_path } => {
                            conn.clear_file(file_path);
                        }
                    }
                    flushed = true;
                }
                if flushed {
                    let _ = conn.commit();
                    conn.wal_checkpoint();
                }
                // If any op failed, re-queue the un-flushed remainder so
                // they're not permanently lost (they were dequeued but
                // never written to the DB or fallback JSON files).
                if let Some(failed_idx) = failed_at {
                    let remaining: Vec<WriteOp> = ops.into_iter().skip(failed_idx).collect();
                    if !remaining.is_empty() {
                        if let Ok(mut pending) = self.pending.lock() {
                            pending.splice(0..0, remaining);
                        }
                    }
                }
            }
        }
        conn.load_latest(file_path)
    }

    fn has_context(&self, file_path: &str) -> bool {
        if let Some(guard) = self.sqlite() {
            guard.has_context(file_path)
        } else {
            false
        }
    }

    fn append_delta(
        &mut self,
        context_id: &str,
        delta_payload: &[u8],
        edit_type: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut pending) = self.pending.lock() {
            pending.push(WriteOp::AppendDelta {
                context_id: context_id.to_string(),
                delta_payload: delta_payload.to_vec(),
                edit_type: edit_type.map(String::from),
            });
            let len = pending.len();
            if len >= BATCH_THRESHOLD {
                drop(pending);
                self.flush();
            }
        }
        Ok(())
    }

    fn delta_count(&self, context_id: &str) -> usize {
        self.flush();
        if let Some(guard) = self.sqlite() {
            guard.delta_count(context_id)
        } else {
            0
        }
    }

    fn clear_file(&mut self, file_path: &str) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.retain(|op| match op {
                WriteOp::SaveContext { file_path: fp, .. } => fp != file_path,
                WriteOp::ClearFile { file_path: fp } => fp != file_path,
                _ => true,
            });
        }
        self.flush();
        if let Ok(mut guard) = self.inner.lock() {
            guard.clear_file(file_path);
        }
    }
}

/// Methods that are NOT part of ContextStore but are called directly
/// on the persistence store by specific handlers.
impl BufferedStore {
    pub fn load_context_with_deltas(
        &self,
        file_path: &str,
        target_seq: Option<u32>,
    ) -> Result<Option<(CompiledIR, u32)>, Box<dyn std::error::Error>> {
        self.flush();
        if let Some(guard) = self.sqlite() {
            guard.load_context_with_deltas(file_path, target_seq)
        } else {
            Ok(None)
        }
    }

    pub fn purge_old_deltas(&self, days: u32) -> Result<usize, Box<dyn std::error::Error>> {
        self.flush();
        if let Some(guard) = self.sqlite() {
            guard.purge_old_deltas(days)
        } else {
            Ok(0)
        }
    }
}

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/buffered_store.rs"]
mod tests;
