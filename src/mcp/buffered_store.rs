// src/mcp/buffered_store.rs
//
// Buffered persistence layer with three-tier defense:
//
//   Tier 1: Batched writes — ops accumulate in memory and are flushed as a
//           single SQLite transaction by their explicit lifecycle owner.
//   Tier 2: Retry with exponential backoff — transient DB failures
//           (file lock, WAL contention) are retried up to MAX_RETRIES.
//   Tier 3: Failed legacy batches remain pending for their explicit owner.
//           Incomplete historical fallback artifacts are inspection-only.
//
// Flush boundaries are owned explicitly by the lifecycle operation that
// produced the pending work. Reads and maintenance operations never flush.

use crate::compression::Fidelity;
use crate::ir::compiler::CompiledIR;
use crate::mcp::context_store::{ContextStore, StoredContextMeta};
use crate::mcp::sqlite_store::SqliteStore;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

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

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Quarantine inspection is an explicit maintenance boundary.
pub(crate) struct LegacyFallbackArtifact {
    pub path: PathBuf,
    pub operation: Option<String>,
    pub identity: Option<String>,
    pub available_metadata: serde_json::Value,
    pub reason: &'static str,
}

/// Legacy buffered SQLite adapter retained for explicitly owned asynchronous
/// work and tests. Registered semantic lifecycle handlers use scoped SQLite
/// transactions directly.
///
/// Failed batches remain pending; they never create or import incomplete
/// fallback semantic artifacts.
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
                    eprintln!(
                        "[clean-ctx] Buffered flush OK: {} ops (attempt {})",
                        n,
                        attempt + 1
                    );
                    succeeded = true;
                    break;
                }
                Err(e) => {
                    last_err = e;
                    eprintln!(
                        "[clean-ctx] Buffered flush attempt {} failed: {last_err}",
                        attempt + 1
                    );
                }
            }
        }

        if !succeeded {
            eprintln!("[clean-ctx] All flush attempts failed; retaining pending operations.");
            if let Ok(mut pending) = self.pending.lock() {
                pending.splice(0..0, ops);
            }
            return 0;
        }

        count
    }

    /// Inspect incomplete historical fallback artifacts without importing,
    /// rewriting, deleting, or publishing any semantic state.
    pub(crate) fn inspect_legacy_fallbacks(&self) -> Vec<LegacyFallbackArtifact> {
        let directory = self.project_root.join(".clean-ctx").join("fallback");
        let Ok(entries) = std::fs::read_dir(directory) else {
            return Vec::new();
        };
        let mut artifacts = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
            .map(|entry| {
                let path = entry.path();
                let parsed = std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok());
                let operation = parsed
                    .as_ref()
                    .and_then(|value| value["type"].as_str())
                    .map(str::to_owned);
                let identity = parsed.as_ref().and_then(|value| {
                    value["file_path"]
                        .as_str()
                        .or_else(|| value["context_id"].as_str())
                        .map(str::to_owned)
                });
                LegacyFallbackArtifact {
                    path,
                    operation,
                    identity,
                    available_metadata: parsed.unwrap_or(serde_json::Value::Null),
                    reason: "legacy artifact lacks complete aligned semantic authority",
                }
            })
            .collect::<Vec<_>>();
        artifacts.sort_by(|left, right| left.path.cmp(&right.path));
        artifacts
    }

    /// Try to flush ops in a single SQLite transaction.
    fn try_flush_ops(&self, ops: &[WriteOp]) -> Result<usize, String> {
        let mut conn = self
            .inner
            .lock()
            .map_err(|e| format!("lock poisoned: {e}"))?;

        conn.begin_transaction()
            .map_err(|e| format!("BEGIN failed: {e}"))?;

        let mut flushed = 0;
        for op in ops {
            match op {
                WriteOp::SaveContext {
                    file_path,
                    fidelity,
                    compressed_output,
                    ir_binary,
                    source_hash,
                    raw_tokens,
                    compressed_tokens,
                } => {
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
                WriteOp::AppendDelta {
                    context_id,
                    delta_payload,
                    edit_type,
                } => {
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

    /// Get a reference to the inner SQLite store for read-only operations.
    pub fn sqlite(&self) -> Option<std::sync::MutexGuard<'_, SqliteStore>> {
        self.inner.lock().ok()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.lock().map(|p| p.len()).unwrap_or(0)
    }

    /// Queue a save operation for its producing lifecycle to commit explicitly.
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
        }
    }

    /// Queue a delta operation for its producing lifecycle to commit explicitly.
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
        }
    }

    /// Queue a clear operation for its producing lifecycle to commit explicitly.
    pub fn queue_clear_file(&self, file_path: &str) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.push(WriteOp::ClearFile {
                file_path: file_path.to_string(),
            });
        }
    }
}

/// Non-authoritative legacy adapter. Queueing never establishes durable state;
/// its caller must own and invoke the explicit commit boundary.
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
        }
        Ok(id)
    }

    fn load_latest(
        &self,
        file_path: &str,
    ) -> Result<Option<StoredContextMeta>, Box<dyn std::error::Error>> {
        match self.sqlite() {
            Some(guard) => guard.load_latest(file_path),
            None => Ok(None),
        }
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
        }
        Ok(())
    }

    fn delta_count(&self, context_id: &str) -> usize {
        if let Some(guard) = self.sqlite() {
            guard.delta_count(context_id)
        } else {
            0
        }
    }

    fn clear_file(&mut self, file_path: &str) {
        let committed_context_id = self
            .sqlite()
            .and_then(|guard| guard.load_latest(file_path).ok().flatten())
            .map(|meta| format!("ctx-{}", meta.source_hash));
        if let Ok(mut pending) = self.pending.lock() {
            let mut owned_context_ids = pending
                .iter()
                .filter_map(|op| match op {
                    WriteOp::SaveContext {
                        file_path: pending_file,
                        source_hash,
                        ..
                    } if pending_file == file_path => Some(format!("ctx-{source_hash}")),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if let Some(context_id) = committed_context_id {
                owned_context_ids.push(context_id);
            }
            pending.retain(|op| match op {
                WriteOp::SaveContext { file_path: fp, .. } => fp != file_path,
                WriteOp::ClearFile { file_path: fp } => fp != file_path,
                WriteOp::AppendDelta { context_id, .. } => !owned_context_ids.contains(context_id),
            });
        }
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
        if let Some(guard) = self.sqlite() {
            guard.load_context_with_deltas(file_path, target_seq)
        } else {
            Ok(None)
        }
    }

    pub fn purge_old_deltas(&self, days: u32) -> Result<usize, Box<dyn std::error::Error>> {
        if let Some(guard) = self.sqlite() {
            guard.purge_old_deltas(days)
        } else {
            Ok(0)
        }
    }

    /// Enumerate persisted contexts (Non-CBM audit 2026-08-25 #7).
    /// Returns an empty list when persistence is disabled.
    pub fn list_contexts(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::mcp::sqlite_store::PersistedContextSummary>, Box<dyn std::error::Error>>
    {
        if let Some(guard) = self.sqlite() {
            guard.list_contexts(limit)
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/buffered_store.rs"]
mod tests;

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/buffered_store_integration.rs"]
mod integration_tests;

#[cfg(test)]
#[path = "../tests/mcp/buffered_store_authority.rs"]
mod authority_tests;
