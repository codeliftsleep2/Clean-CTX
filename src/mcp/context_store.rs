// src/mcp/context_store.rs
//
// Persistence boundary for compression contexts.
//
// Defines the `ContextStore` trait that abstracts how compression
// baselines, deltas, and metadata are persisted. The current
// implementation (`InMemoryContextStore`) lives entirely in RAM
// and is session-scoped. A future `SqliteContextStore` will back
// the same trait with SQLite for cross-session persistence.
//
// Design invariant: tool handlers talk to `dyn ContextStore`, not
// to concrete storage implementations. This means zero handler
// changes when SQLite arrives.

use std::collections::HashMap;
use std::time::SystemTime;
use crate::compression::Fidelity;

/// Metadata about a stored compression context for a file.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StoredContextMeta {
    /// The original file path.
    pub file_path: String,
    /// The fidelity level used during compression.
    pub fidelity: Fidelity,
    /// Monotonic version number (increments on each delta).
    pub version: u64,
    /// Whether Angular Meta-Layer was detected and applied.
    pub is_angular: bool,
    /// Hash of the source content at the time of compression.
    pub source_hash: String,
    /// When this context was first created.
    pub created_at: SystemTime,
    /// Raw token count at time of compression (0 if unknown).
    pub raw_tokens: u64,
    /// Compressed token count at time of compression (0 if unknown).
    pub compressed_tokens: u64,
}

/// A delta record appended to a context's history.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DeltaRecord {
    /// Raw delta payload bytes.
    pub payload: Vec<u8>,
    /// Optional edit type tag (e.g., "edit", "refactor").
    pub edit_type: Option<String>,
    /// When this delta was applied.
    pub applied_at: SystemTime,
}

/// Trait for persisting and restoring compression contexts.
///
/// Current: `InMemoryContextStore` (session-only, lives in `McpState`).
/// Future: `SqliteContextStore` (survives IDE restarts, backed by
/// `persistence.db` in the `.clean-ctx/` directory).
///
/// All methods are fallible to accommodate future I/O-bound
/// implementations.
///
/// # Persistence readiness
///
/// Most methods are unused today because the SQLite layer is deferred.
/// They define the contract that the future `SqliteContextStore` will
/// fulfill. The `#[allow(dead_code)]` supressions keep the codebase
/// warning-free while preserving the trait boundary.
#[allow(dead_code)]
pub trait ContextStore {
    /// Save a full compression context (baseline) for a file.
    ///
    /// Returns a context ID string that can be used for subsequent
    /// `append_delta` calls.
    /// `raw_tokens` and `compressed_tokens` may be 0 if unknown.
    #[allow(clippy::too_many_arguments)]
    fn save_context(
        &mut self,
        file_path: &str,
        fidelity: Fidelity,
        compressed_output: &str,
        ir_blobs: Option<&[u8]>,
        source_hash: &str,
        raw_tokens: u64,
        compressed_tokens: u64,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Load the latest context metadata for a file, if any.
    fn load_latest(
        &self,
        file_path: &str,
    ) -> Result<Option<StoredContextMeta>, Box<dyn std::error::Error>>;

    /// Check if a context exists for this file (fast path, no I/O).
    fn has_context(&self, file_path: &str) -> bool;

    /// Append a delta to the context's history.
    fn append_delta(
        &mut self,
        context_id: &str,
        delta_payload: &[u8],
        edit_type: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Get the number of deltas applied to a context.
    fn delta_count(&self, context_id: &str) -> usize;

    /// Clear all stored data for a file (used by `restore_context`).
    fn clear_file(&mut self, file_path: &str);
}

/// In-memory implementation of [`ContextStore`].
///
/// Backed by `HashMap` for both context metadata and delta records.
/// All data is session-scoped and lost on server restart.
#[derive(Debug)]
pub struct InMemoryContextStore {
    /// Context metadata keyed by file path.
    contexts: HashMap<String, StoredContextMeta>,
    /// Delta records keyed by context ID.
    deltas: HashMap<String, Vec<DeltaRecord>>,
    /// Map from context ID to file path (reverse lookup).
    id_to_path: HashMap<String, String>,
}

impl InMemoryContextStore {
    /// Create a new empty in-memory store.
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            deltas: HashMap::new(),
            id_to_path: HashMap::new(),
        }
    }

    /// Generate a unique context ID for a file.
    fn generate_id(file_path: &str) -> String {
        // Simple deterministic ID based on file path + timestamp
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("ctx-{}-{:016x}", file_path.replace(['/', '\\'], "_"), ts)
    }
}

impl Default for InMemoryContextStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextStore for InMemoryContextStore {
    fn save_context(
        &mut self,
        file_path: &str,
        fidelity: Fidelity,
        _compressed_output: &str,
        _ir_blobs: Option<&[u8]>,
        source_hash: &str,
        raw_tokens: u64,
        compressed_tokens: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let id = Self::generate_id(file_path);
        let meta = StoredContextMeta {
            file_path: file_path.to_string(),
            fidelity,
            version: 1,
            is_angular: false,
            source_hash: source_hash.to_string(),
            created_at: SystemTime::now(),
            raw_tokens,
            compressed_tokens,
        };
        self.contexts.insert(file_path.to_string(), meta);
        self.id_to_path.insert(id.clone(), file_path.to_string());
        Ok(id)
    }

    fn load_latest(
        &self,
        file_path: &str,
    ) -> Result<Option<StoredContextMeta>, Box<dyn std::error::Error>> {
        Ok(self.contexts.get(file_path).cloned())
    }

    fn has_context(&self, file_path: &str) -> bool {
        self.contexts.contains_key(file_path)
    }

    fn append_delta(
        &mut self,
        context_id: &str,
        delta_payload: &[u8],
        edit_type: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let record = DeltaRecord {
            payload: delta_payload.to_vec(),
            edit_type: edit_type.map(String::from),
            applied_at: SystemTime::now(),
        };
        self.deltas
            .entry(context_id.to_string())
            .or_default()
            .push(record);

        // Update version on the associated context
        if let Some(file_path) = self.id_to_path.get(context_id) {
            if let Some(meta) = self.contexts.get_mut(file_path) {
                meta.version += 1;
            }
        }
        Ok(())
    }

    fn delta_count(&self, context_id: &str) -> usize {
        self.deltas
            .get(context_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    fn clear_file(&mut self, file_path: &str) {
        // Remove context metadata
        if let Some(_meta) = self.contexts.remove(file_path) {
            // Find and remove associated ID mappings and their deltas
            let ids_to_remove: Vec<String> = self.id_to_path
                .iter()
                .filter(|(_, v)| *v == file_path)
                .map(|(k, _)| k.clone())
                .collect();
            for id in &ids_to_remove {
                self.deltas.remove(id);
                self.id_to_path.remove(id);
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/mcp/context_store.rs"]
mod tests;