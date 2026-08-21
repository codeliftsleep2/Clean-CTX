// src/tests/cbm/cache_store.rs
//
// Tests for the SQLite-backed CBM graph cache store.

use crate::cbm::cache_store::GraphCacheStore;
use std::path::PathBuf;

/// Create a temp DB path unique to this test invocation.
fn temp_db_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("clean-ctx-cbm-cache-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(format!("{name}.db"))
}

#[test]
fn put_and_get_roundtrip() {
    let path = temp_db_path("roundtrip");
    let _ = std::fs::remove_file(&path);
    let store = GraphCacheStore::open(&path).unwrap();

    let expires = crate::cbm::cache_store::now_epoch_ms() + 60_000;
    store.put("/repo/a", "symbol_importance", r#"{"a":1}"#, expires);

    let got = store.get("/repo/a", "symbol_importance");
    assert_eq!(got.as_deref(), Some(r#"{"a":1}"#));

    // Different project should not see the entry (project-scoped).
    assert_eq!(store.get("/repo/b", "symbol_importance"), None);
}

#[test]
fn expired_entry_is_miss_and_purged() {
    let path = temp_db_path("expired");
    let _ = std::fs::remove_file(&path);
    let store = GraphCacheStore::open(&path).unwrap();

    // Insert an entry that is already expired.
    let past = crate::cbm::cache_store::now_epoch_ms() - 60_000;
    store.put("/repo/a", "call_edges", r#"[]"#, past);

    // Read should treat it as a miss (and lazily purge the expired row).
    assert_eq!(store.get("/repo/a", "call_edges"), None);
    assert_eq!(store.count_for_project("/repo/a"), 0);

    // Insert another expired entry and verify purge_expired removes it.
    store.put("/repo/a", "call_edges", r#"[]"#, past);
    let removed = store.purge_expired();
    assert!(removed >= 1);
    assert_eq!(store.count_for_project("/repo/a"), 0);
}

#[test]
fn put_overwrites_existing() {
    let path = temp_db_path("overwrite");
    let _ = std::fs::remove_file(&path);
    let store = GraphCacheStore::open(&path).unwrap();

    let expires = crate::cbm::cache_store::now_epoch_ms() + 60_000;
    store.put("/repo/a", "search:foo", r#"["old"]"#, expires);
    store.put("/repo/a", "search:foo", r#"["new"]"#, expires);

    assert_eq!(
        store.get("/repo/a", "search:foo").as_deref(),
        Some(r#"["new"]"#)
    );
}

#[test]
fn invalidate_project_and_key() {
    let path = temp_db_path("invalidate");
    let _ = std::fs::remove_file(&path);
    let store = GraphCacheStore::open(&path).unwrap();

    let expires = crate::cbm::cache_store::now_epoch_ms() + 60_000;
    store.put("/repo/a", "k1", r#"1"#, expires);
    store.put("/repo/a", "k2", r#"2"#, expires);
    store.put("/repo/b", "k1", r#"3"#, expires);

    // Invalidate a single key in project a.
    store.invalidate_key("/repo/a", "k1");
    assert_eq!(store.get("/repo/a", "k1"), None);
    assert_eq!(store.get("/repo/a", "k2").as_deref(), Some(r#"2"#));
    // Project b unaffected.
    assert_eq!(store.get("/repo/b", "k1").as_deref(), Some(r#"3"#));

    // Invalidate whole project a.
    store.invalidate_project("/repo/a");
    assert_eq!(store.count_for_project("/repo/a"), 0);
    // Project b still intact.
    assert_eq!(store.count_for_project("/repo/b"), 1);
}

#[test]
fn count_for_project() {
    let path = temp_db_path("count");
    let _ = std::fs::remove_file(&path);
    let store = GraphCacheStore::open(&path).unwrap();

    let expires = crate::cbm::cache_store::now_epoch_ms() + 60_000;
    store.put("/repo/a", "k1", r#"1"#, expires);
    store.put("/repo/a", "k2", r#"2"#, expires);
    store.put("/repo/b", "k1", r#"3"#, expires);

    assert_eq!(store.count_for_project("/repo/a"), 2);
    assert_eq!(store.count_for_project("/repo/b"), 1);
    assert_eq!(store.count_for_project("/repo/c"), 0);
}
