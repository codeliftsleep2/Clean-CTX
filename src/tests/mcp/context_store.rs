// src/tests/mcp/context_store.rs
//
// Tests for InMemoryContextStore

use crate::compression::Fidelity;
use crate::mcp::context_store::{ContextStore, InMemoryContextStore};

#[test]
fn test_context_store_round_trip() {
    let mut store = InMemoryContextStore::new();
    let id = store
        .save_context(
            "/test/file.ts",
            Fidelity::Low,
            "compressed",
            None,
            "abc123",
            1000,
            250,
        )
        .expect("save should succeed");
    assert!(!id.is_empty());

    let meta = store
        .load_latest("/test/file.ts")
        .expect("load should succeed");
    assert!(meta.is_some());
    let meta = meta.unwrap();
    assert_eq!(meta.file_path, "/test/file.ts");
    assert_eq!(meta.fidelity, Fidelity::Low);
}

#[test]
fn test_has_context() {
    let mut store = InMemoryContextStore::new();
    assert!(!store.has_context("/test/file.ts"));
    store
        .save_context(
            "/test/file.ts",
            Fidelity::Medium,
            "compressed",
            None,
            "hash1",
            1000,
            250,
        )
        .expect("save should succeed");
    assert!(store.has_context("/test/file.ts"));
}

#[test]
fn test_clear_file_removes_context() {
    let mut store = InMemoryContextStore::new();
    store
        .save_context(
            "/test/file.ts",
            Fidelity::Low,
            "compressed",
            None,
            "hash1",
            1000,
            250,
        )
        .expect("save should succeed");
    assert!(store.has_context("/test/file.ts"));
    store.clear_file("/test/file.ts");
    assert!(!store.has_context("/test/file.ts"));
}

#[test]
fn test_delta_count() {
    let mut store = InMemoryContextStore::new();
    let id = store
        .save_context(
            "/test/file.ts",
            Fidelity::Low,
            "compressed",
            None,
            "hash1",
            1000,
            250,
        )
        .expect("save should succeed");
    assert_eq!(store.delta_count(&id), 0);
    store
        .append_delta(&id, b"delta1", Some("edit"))
        .expect("delta should succeed");
    assert_eq!(store.delta_count(&id), 1);
    store
        .append_delta(&id, b"delta2", None)
        .expect("delta should succeed");
    assert_eq!(store.delta_count(&id), 2);
}

#[test]
fn test_load_nonexistent_returns_none() {
    let store = InMemoryContextStore::new();
    let meta = store
        .load_latest("/nonexistent.ts")
        .expect("load should succeed");
    assert!(meta.is_none());
}

#[test]
fn test_save_context_with_token_counts() {
    let mut store = InMemoryContextStore::new();
    let _id = store
        .save_context(
            "/test/file.ts",
            Fidelity::Low,
            "compressed",
            None,
            "hash1",
            5000,
            800,
        )
        .expect("save should succeed");
    let meta = store
        .load_latest("/test/file.ts")
        .expect("load should succeed")
        .unwrap();
    assert_eq!(meta.raw_tokens, 5000);
    assert_eq!(meta.compressed_tokens, 800);
}
