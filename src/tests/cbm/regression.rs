// src/tests/cbm/regression.rs
//
// Regression tests for CBM audit fixes, decomposed by semantic boundary.

#![allow(unnameable_test_items)]

use serde_json::json;

/// Create a unique throwaway directory usable as a repository root.
fn make_temp_root(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("clean_ctx_projid_{}_{}", tag, std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Force mock-mode availability without a real CBM subprocess.
fn prime_available(bridge: &mut crate::cbm::GraphBridge) {
    use crate::cbm::bridge::CachedGraphData;
    use crate::cbm::bridge::test_helpers::seed_active_cache;
    use std::time::{Duration, Instant};

    bridge.status = crate::cbm::config::CbmStatus::Available;
    seed_active_cache(
        bridge,
        "__avail__",
        CachedGraphData {
            data: serde_json::json!("available"),
            expires_at: Instant::now() + Duration::from_secs(300),
        },
    );
}

#[path = "regression/circuit_intelligence.rs"]
mod circuit_intelligence;
#[path = "regression/edit_proxy.rs"]
mod edit_proxy;
#[path = "regression/freshness.rs"]
mod freshness;
#[path = "regression/project_identity.rs"]
mod project_identity;
#[path = "regression/retry_cache.rs"]
mod retry_cache;
