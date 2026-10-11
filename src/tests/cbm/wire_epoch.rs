//! B0: legacy adapter payloads must not hydrate into the current wire contract.
use crate::cbm::bridge::{GraphBridge, cbm_project_slug};
use crate::cbm::cache_store::GraphCacheStore;
use crate::cbm::config::CbmConfig;
use serde_json::json;

#[test]
fn cbm_b0_legacy_disk_payload_is_ignored_without_deleting_it() {
    let workspace = tempfile::tempdir().expect("workspace");
    let cache = tempfile::tempdir().expect("cache");
    let root = workspace.path().canonicalize().expect("canonical root");
    let project = cbm_project_slug(&root);
    let partition = root.to_string_lossy();
    let legacy_key = format!("{project}:symbol_importance");
    let payload = json!({"LegacyOnly": {
        "symbol":"LegacyOnly", "file":"legacy.ts", "score":0.7
    }})
    .to_string();
    let store = GraphCacheStore::open(&cache.path().join("graph.db")).expect("store");
    store.put(&partition, &legacy_key, &payload, i64::MAX);
    let mut bridge = GraphBridge::try_create(
        &CbmConfig {
            enabled: false,
            ..Default::default()
        },
        &root,
    );
    bridge.attach_disk_cache(store);
    assert!(
        bridge.get_symbol_importance_mut().is_err(),
        "pre-pinned-wire payload must be a miss, never current provider evidence"
    );
    let untouched = GraphCacheStore::open(&cache.path().join("graph.db")).expect("reopen");
    assert_eq!(
        untouched.get(&partition, &legacy_key),
        Some(payload),
        "compatibility rejection must not delete legacy or unrelated stored data"
    );
}
