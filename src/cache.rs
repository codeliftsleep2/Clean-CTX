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
// F-40: uses `HashMap` (not `BTreeMap`) because no caller iterates the
// registry or baseline in sorted order; `HashMap` is faster for lookups.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

use crate::diff::CapturedStructure;

/// Maximum number of entries in `raw_token_counts` before LRU eviction.
/// F-FULL-17: Prevents unbounded memory growth in long-running sessions.
const MAX_RAW_TOKEN_COUNT_ENTRIES: usize = 10_000;

pub struct LocalStateCache {
    /// Maps absolute file paths to their last calculated content hashes.
    registry: HashMap<String, String>,
    /// Maps cache key ("{path}::{fidelity}") to the last `CapturedStructure`
    /// for that file. Used by `diff_code_context` to compute AST deltas.
    baseline_snapshots: HashMap<String, CapturedStructure>,
    /// F-21 (FAANG audit): maps cache key ("{path}::{fidelity}") to the
    /// content hash at the time the baseline was stored. Used by
    /// `diff_code_context` to skip re-parsing when the file hasn't changed.
    baseline_hashes: HashMap<String, String>,
    /// F-14: Maps content hash → raw-token count so that cache-hit paths
    /// can skip the expensive BPE encode. Keyed by the hash (not the
    /// file path) because the same content in two locations yields the
    /// same raw-token count.
    /// F-FULL-17: LRU-evicting cache bounded by MAX_RAW_TOKEN_COUNT_ENTRIES.
    raw_token_counts: HashMap<String, usize>,
    /// H-3 fix: O(1) LRU ordering.
    /// Maps content hash → the generation number at which it was last used.
    raw_token_gen: HashMap<String, u64>,
    /// Maps generation number → content hash, ordered so the minimum (oldest)
    /// generation can be found and removed in O(log n) time.
    raw_token_order: BTreeMap<u64, String>,
    /// Monotonically increasing generation counter.
    raw_token_clock: u64,
}

impl Default for LocalStateCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalStateCache {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            baseline_snapshots: HashMap::new(),
            baseline_hashes: HashMap::new(),
            raw_token_counts: HashMap::new(),
            raw_token_gen: HashMap::new(),
            raw_token_order: BTreeMap::new(),
            raw_token_clock: 0,
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
        // F-41: only insert if the hash actually changed.
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
    /// F-FULL-17: LRU-evicting cache bounded by MAX_RAW_TOKEN_COUNT_ENTRIES.
    /// H-3 fix: O(log n) promote/evict using a generation counter + BTreeMap
    /// instead of O(n) VecDeque::iter().position() + remove().
    pub fn store_raw_token_count(&mut self, content_hash: &str, count: usize) {
        // Promote: remove the old ordering entry for this key (if any).
        if let Some(&old_gen) = self.raw_token_gen.get(content_hash) {
            self.raw_token_order.remove(&old_gen);
        } else if self.raw_token_counts.len() >= MAX_RAW_TOKEN_COUNT_ENTRIES {
            // Evict the entry with the smallest (oldest) generation in O(log n).
            if let Some((&oldest_gen, oldest_key)) = self.raw_token_order.iter().next() {
                let oldest_key = oldest_key.clone();
                self.raw_token_order.remove(&oldest_gen);
                self.raw_token_gen.remove(&oldest_key);
                self.raw_token_counts.remove(&oldest_key);
            }
        }
        // Assign a fresh generation and record it.
        let clock = self.raw_token_clock;
        self.raw_token_clock += 1;
        self.raw_token_gen.insert(content_hash.to_string(), clock);
        self.raw_token_order.insert(clock, content_hash.to_string());
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
        self.raw_token_gen.clear();
        self.raw_token_order.clear();
        self.raw_token_clock = 0;
    }
}

#[cfg(test)]
#[path = "tests/cache.rs"]
mod tests;
