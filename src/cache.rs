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
    pub fn update_and_verify(&mut self, absolute_path: String, current_hash: String) -> bool {
        if let Some(existing_hash) = self.registry.get(&absolute_path)
            && *existing_hash == current_hash {
                return false;
            }
        self.registry.insert(absolute_path, current_hash);
        true
    }

    /// Persist a `CapturedStructure` for the given cache key. This is what
    /// `diff_code_context` later reads to compute an AST-level delta.
    pub fn store_baseline(&mut self, key: String, snapshot: CapturedStructure) {
        self.baseline_snapshots.insert(key, snapshot);
    }

    /// Retrieve a previously-stored baseline snapshot, if any.
    pub fn get_baseline(&self, key: &str) -> Option<&CapturedStructure> {
        self.baseline_snapshots.get(key)
    }

    /// Drop a baseline entry (used when we want to force a fresh comparison
    /// or invalidate stale state after a manual overwrite).
    pub fn invalidate_baseline(&mut self, key: &str) {
        self.baseline_snapshots.remove(key);
    }

    /// Clear both registries. Useful for tests and for clients that want
    /// to reset session state.
    pub fn clear(&mut self) {
        self.registry.clear();
        self.baseline_snapshots.clear();
    }
}

#[cfg(test)]
#[path = "tests/cache.rs"]
mod tests;
