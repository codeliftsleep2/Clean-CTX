// src/cache.rs
//
// Two-tier state cache:
//   1. A SHA-256 content-hash registry used for "has this file changed?"
//      queries (the cheap fast-path).
//   2. A baseline-snapshot registry that stores the last compressed
//      `CapturedStructure` for each (file, fidelity) pair so the
//      `diff_code_context` tool can produce AST-level deltas without
//      re-reading historical source.
//
// The baseline registry uses a `BTreeMap` so the implementation is
// deterministic and free of unsafe code; persistence across restarts is
// not currently required (the MCP server is session-scoped), but the API
// is shaped so that a future JSON snapshot could be dropped in here.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

use crate::diff::CapturedStructure;

pub struct LocalStateCache {
    /// Maps absolute file paths to their last calculated content hashes.
    registry: BTreeMap<String, String>,
    /// Maps cache key ("{path}::{fidelity}") to the last `CapturedStructure`
    /// for that file. Used by `diff_code_context` to compute AST deltas.
    baseline_snapshots: BTreeMap<String, CapturedStructure>,
    /// F-21 (FAANG audit): maps cache key ("{path}::{fidelity}") to the
    /// content hash at the time the baseline was stored. Used by
    /// `diff_code_context` to skip re-parsing when the file hasn't changed.
    baseline_hashes: BTreeMap<String, String>,
    /// F-14: Maps content hash → raw-token count so that cache-hit paths
    /// can skip the expensive BPE encode. Keyed by the hash (not the
    /// file path) because the same content in two locations yields the
    /// same raw-token count.
    raw_token_counts: BTreeMap<String, usize>,
}

impl Default for LocalStateCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalStateCache {
    pub fn new() -> Self {
        Self {
            registry: BTreeMap::new(),
            baseline_snapshots: BTreeMap::new(),
            baseline_hashes: BTreeMap::new(),
            raw_token_counts: BTreeMap::new(),
        }
    }

    /// Computes an ultra-fast local SHA-256 hash string from file bytes.
    pub fn compute_hash(&self, bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }

    /// Checks if a file has changed. Returns `true` if it is a brand-new or
    /// modified file, `false` if the content is byte-for-byte identical to
    /// the last seen version.
    pub fn update_and_verify(&mut self, absolute_path: &str, current_hash: &str) -> bool {
        if let Some(existing_hash) = self.registry.get(absolute_path)
            && *existing_hash == current_hash {
                return false;
            }
        self.registry.insert(absolute_path.to_string(), current_hash.to_string());
        true
    }

    /// Persist a `CapturedStructure` for the given cache key. This is what
    /// `diff_code_context` later reads to compute an AST-level delta.
    pub fn store_baseline(&mut self, key: String, snapshot: CapturedStructure) {
        self.baseline_snapshots.insert(key, snapshot);
    }

    /// F-21: Persist the content hash alongside the baseline so
    /// `diff_code_context` can short-circuit when the file hasn't changed.
    pub fn store_baseline_hash(&mut self, key: &str, hash: &str) {
        self.baseline_hashes.insert(key.to_string(), hash.to_string());
    }

    /// F-21: Retrieve the stored content hash for a baseline.
    pub fn get_baseline_hash(&self, key: &str) -> Option<&str> {
        self.baseline_hashes.get(key).map(|s| s.as_str())
    }

    /// Retrieve a previously-stored baseline snapshot, if any.
    pub fn get_baseline(&self, key: &str) -> Option<&CapturedStructure> {
        self.baseline_snapshots.get(key)
    }

    /// Drop a baseline entry (used when we want to force a fresh comparison
    /// or invalidate stale state after a manual overwrite).
    pub fn invalidate_baseline(&mut self, key: &str) {
        self.baseline_snapshots.remove(key);
        self.baseline_hashes.remove(key);
    }

    /// F-14: Store the raw-token count for a content hash so the cache-hit
    /// path can skip the BPE encode.
    pub fn store_raw_token_count(&mut self, content_hash: &str, count: usize) {
        self.raw_token_counts.insert(content_hash.to_string(), count);
    }

    /// F-14: Retrieve a previously-stored raw-token count for the given
    /// content hash. Returns `None` if the hash has never been seen.
    pub fn get_raw_token_count(&self, content_hash: &str) -> Option<usize> {
        self.raw_token_counts.get(content_hash).copied()
    }

    /// Clear all registries. Useful for tests and for clients that want
    /// to reset session state.
    pub fn clear(&mut self) {
        self.registry.clear();
        self.baseline_snapshots.clear();
        self.baseline_hashes.clear();
        self.raw_token_counts.clear();
    }
}

#[cfg(test)]
#[path = "tests/cache.rs"]
mod tests;
