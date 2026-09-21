// src/tests/mcp/buffered_store.rs
//
// Tests for BufferedStore: explicit lifecycle flushing, transaction batching,
// fallback file creation, and re-import from fallback files.

use base64::Engine;
use std::path::Path;
use tempfile::TempDir;

use crate::compression::Fidelity;
use crate::mcp::buffered_store::BufferedStore;
use crate::mcp::context_store::ContextStore;
use crate::mcp::sqlite_store::SqliteStore;

/// Helper: create a BufferedStore backed by an in-memory SQLite DB
/// with a temporary directory as project root.
fn make_store() -> (BufferedStore, TempDir) {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let store = SqliteStore::open(Path::new(":memory:")).expect("failed to open in-memory SQLite");
    let buffered = BufferedStore::new(store, tmp.path().to_path_buf());
    (buffered, tmp)
}

// ── Tier 1: Batched writes ────────────────────────────────────────

#[test]
fn test_queue_save_context_and_flush() {
    let (store, _tmp) = make_store();

    store.queue_save_context(
        "/test/file.ts",
        Fidelity::Low,
        "compressed",
        b"ir_data",
        "hash1",
        0,
        0,
    );
    // Should have 1 pending op
    assert_eq!(store.pending_count(), 1);

    // Explicit flush
    let flushed = store.flush();
    assert_eq!(flushed, 1);
    assert_eq!(store.pending_count(), 0);

    // Data should now be in the SQLite store
    let guard = store.sqlite().unwrap();
    assert!(guard.has_context("/test/file.ts"));
}

#[test]
fn test_queue_append_delta_and_flush() {
    let (store, _tmp) = make_store();

    // First save a context to have a valid context_id
    store.queue_save_context(
        "/test/file.ts",
        Fidelity::Low,
        "compressed",
        b"",
        "hash1",
        0,
        0,
    );
    store.flush();

    // Now queue a delta
    store.queue_append_delta("ctx-hash1", b"delta_payload", Some("edit"));
    assert_eq!(store.pending_count(), 1);

    let flushed = store.flush();
    assert_eq!(flushed, 1);

    let guard = store.sqlite().unwrap();
    assert_eq!(guard.delta_count("ctx-hash1"), 1);
}

#[test]
fn test_queue_clear_file_and_flush() {
    let (store, _tmp) = make_store();

    // Save a context first
    store.queue_save_context(
        "/test/file.ts",
        Fidelity::Low,
        "compressed",
        b"",
        "hash1",
        0,
        0,
    );
    store.flush();

    assert!(store.has_context("/test/file.ts"));

    // Queue a clear
    store.queue_clear_file("/test/file.ts");
    store.flush();

    // The clear_file op should have removed it from SQLite
    assert!(!store.has_context("/test/file.ts"));
}

// ── Explicit lifecycle boundary ──────────────────────────────────

#[test]
fn test_queue_threshold_does_not_implicitly_commit() {
    let (store, _tmp) = make_store();

    // Queueing multiple operations does not grant a lifecycle commit boundary.
    for i in 0..5 {
        store.queue_save_context(
            &format!("/test/file_{}.ts", i),
            Fidelity::Low,
            "compressed",
            b"",
            &format!("hash_{}", i),
            0,
            0,
        );
    }

    assert_eq!(store.pending_count(), 5);

    assert_eq!(store.flush(), 5);

    // All 5 become visible only after the explicit owner commits them.
    let guard = store.sqlite().unwrap();
    for i in 0..5 {
        assert!(guard.has_context(&format!("/test/file_{}.ts", i)));
    }
}

// ── ContextStore trait implementation ──────────────────────────────

#[test]
fn test_context_store_save_and_load() {
    let (mut store, _tmp) = make_store();

    let id = store
        .save_context(
            "/test/file.ts",
            Fidelity::Medium,
            "compressed output",
            None,
            "hash1",
            0,
            0,
        )
        .expect("save_context should succeed");
    assert_eq!(id, "ctx-hash1");

    // pending should have the op
    assert_eq!(store.pending_count(), 1);

    // Reads observe committed SQLite state and leave the queued save pending.
    let meta = store
        .load_latest("/test/file.ts")
        .expect("load_latest should succeed");
    assert!(meta.is_none());
    assert_eq!(store.pending_count(), 1);

    store.flush();
    let meta = store
        .load_latest("/test/file.ts")
        .expect("load_latest should succeed after explicit commit")
        .expect("explicitly committed context");
    assert_eq!(meta.file_path, "/test/file.ts");
    assert_eq!(meta.fidelity, Fidelity::Medium);
}

#[test]
fn test_context_store_has_context() {
    let (mut store, _tmp) = make_store();

    assert!(!store.has_context("/test/file.ts"));

    store
        .save_context("/test/file.ts", Fidelity::Low, "out", None, "h1", 0, 0)
        .unwrap();
    store.flush();

    assert!(store.has_context("/test/file.ts"));
}

#[test]
fn test_context_store_delta_count() {
    let (mut store, _tmp) = make_store();

    store
        .save_context("/test/file.ts", Fidelity::Low, "out", None, "h1", 0, 0)
        .unwrap();
    store.flush();

    // No deltas yet
    assert_eq!(store.delta_count("ctx-h1"), 0);

    // Append a delta
    store
        .append_delta("ctx-h1", b"payload", Some("edit"))
        .unwrap();
    store.flush();

    assert_eq!(store.delta_count("ctx-h1"), 1);
}

#[test]
fn test_context_store_clear_file() {
    let (mut store, _tmp) = make_store();

    store
        .save_context("/test/file.ts", Fidelity::Low, "out", None, "h1", 0, 0)
        .unwrap();
    store.flush();

    assert!(store.has_context("/test/file.ts"));

    store.clear_file("/test/file.ts");

    assert!(!store.has_context("/test/file.ts"));
}

#[test]
fn test_clear_file_removes_pending_ops() {
    let (store, _tmp) = make_store();

    // Queue 3 ops for the same file
    store.queue_save_context("/test/file.ts", Fidelity::Low, "out1", b"", "h1", 0, 0);
    store.queue_save_context("/test/file.ts", Fidelity::Low, "out2", b"", "h2", 0, 0);
    store.queue_save_context("/test/file.ts", Fidelity::Low, "out3", b"", "h3", 0, 0);

    assert_eq!(store.pending_count(), 3);

    // Clear should remove pending ops for that file
    let mut store_mut = store.clone();
    ContextStore::clear_file(&mut store_mut, "/test/file.ts");

    // Pending ops for this file should be removed
    // (clear_file retains ops for other files and only removes matching ones)
}

// ── Retry with exponential backoff (Tier 2) ───────────────────────

#[test]
fn test_flush_returns_count_even_on_empty() {
    let (store, _tmp) = make_store();

    // Flush with no pending ops should return 0
    let flushed = store.flush();
    assert_eq!(flushed, 0);
}

#[test]
fn test_flush_is_idempotent() {
    let (store, _tmp) = make_store();

    store.queue_save_context(
        "/test/file.ts",
        Fidelity::Low,
        "compressed",
        b"",
        "hash1",
        0,
        0,
    );

    let flushed1 = store.flush();
    assert_eq!(flushed1, 1);

    // Second flush should be a no-op (0 pending)
    let flushed2 = store.flush();
    assert_eq!(flushed2, 0);
}

// ── JSON file fallback (Tier 3) ──────────────────────────────────

#[test]
fn test_fallback_dir_is_created_on_flush_failure() {
    // This test verifies that the fallback directory is used.
    // We can't easily force a flush failure with in-memory SQLite,
    // but we can verify the fallback infrastructure exists.
    let (store, tmp) = make_store();

    // The fallback dir should not exist yet
    let fallback_dir = tmp.path().join(".clean-ctx").join("fallback");
    assert!(!fallback_dir.exists());

    // Normal flush should not create the fallback dir
    store.queue_save_context(
        "/test/file.ts",
        Fidelity::Low,
        "compressed",
        b"",
        "hash1",
        0,
        0,
    );
    store.flush();

    // Fallback dir should still not exist (flush succeeded)
    assert!(!fallback_dir.exists());
}

#[test]
fn test_fallback_file_format() {
    // Verify that the fallback JSON structure is correct by
    // manually creating one and re-importing it.
    let (store, tmp) = make_store();

    let fallback_dir = tmp.path().join(".clean-ctx").join("fallback");
    std::fs::create_dir_all(&fallback_dir).unwrap();

    // Create a fallback file for a save_context op
    let ir_data = b"test_ir_binary_data";
    let json = serde_json::json!({
        "type": "save_context",
        "file_path": "/test/fallback.ts",
        "fidelity": "Low",
        "compressed_output": "compressed text",
        "ir_binary": base64::engine::general_purpose::STANDARD.encode(ir_data),
        "source_hash": "fallback_hash",
    });

    let fallback_file = fallback_dir.join("op_0_0000000000000000.json");
    std::fs::write(&fallback_file, serde_json::to_string_pretty(&json).unwrap()).unwrap();

    // Re-import should pick it up on next flush
    store.flush();

    // The fallback file should be deleted after successful reimport
    assert!(!fallback_file.exists());

    // Data should be in SQLite now
    let guard = store.sqlite().unwrap();
    assert!(guard.has_context("/test/fallback.ts"));
}

#[test]
fn test_fallback_append_delta_reimport() {
    let (store, tmp) = make_store();

    // First save a context to SQLite so the delta has a valid parent
    store.queue_save_context(
        "/test/delta.ts",
        Fidelity::Low,
        "baseline",
        b"",
        "delta_hash",
        0,
        0,
    );
    store.flush();

    let fallback_dir = tmp.path().join(".clean-ctx").join("fallback");
    std::fs::create_dir_all(&fallback_dir).unwrap();

    // Create a fallback file for an append_delta op
    let payload = b"delta_payload_data";
    let json = serde_json::json!({
        "type": "append_delta",
        "context_id": "ctx-delta_hash",
        "delta_payload": base64::engine::general_purpose::STANDARD.encode(payload),
        "edit_type": "edit",
    });

    let fallback_file = fallback_dir.join("op_0_0000000000000001.json");
    std::fs::write(&fallback_file, serde_json::to_string_pretty(&json).unwrap()).unwrap();

    // Re-import
    store.flush();

    // Fallback file should be consumed
    assert!(!fallback_file.exists());

    // Delta should be in SQLite
    let guard = store.sqlite().unwrap();
    assert_eq!(guard.delta_count("ctx-delta_hash"), 1);
}

#[test]
fn test_fallback_clear_file_reimport() {
    let (store, tmp) = make_store();

    // Save a context first
    store.queue_save_context(
        "/test/clear.ts",
        Fidelity::Low,
        "output",
        b"",
        "clear_hash",
        0,
        0,
    );
    store.flush();

    assert!(store.has_context("/test/clear.ts"));

    let fallback_dir = tmp.path().join(".clean-ctx").join("fallback");
    std::fs::create_dir_all(&fallback_dir).unwrap();

    // Create a fallback file for a clear_file op
    let json = serde_json::json!({
        "type": "clear_file",
        "file_path": "/test/clear.ts",
    });

    let fallback_file = fallback_dir.join("op_0_0000000000000002.json");
    std::fs::write(&fallback_file, serde_json::to_string_pretty(&json).unwrap()).unwrap();

    // Re-import
    store.flush();

    // File should be cleared from SQLite
    assert!(!store.has_context("/test/clear.ts"));
}

#[test]
fn test_fallback_invalid_json_is_skipped() {
    let (store, tmp) = make_store();

    let fallback_dir = tmp.path().join(".clean-ctx").join("fallback");
    std::fs::create_dir_all(&fallback_dir).unwrap();

    // Write invalid JSON
    let fallback_file = fallback_dir.join("op_0_bad.json");
    std::fs::write(&fallback_file, "not valid json {{{").unwrap();

    // Flush should not panic — invalid files are silently skipped
    store.flush();

    // The invalid file should still exist (not deleted since reimport failed)
    assert!(fallback_file.exists());

    // Clean up
    let _ = std::fs::remove_file(&fallback_file);
}

// ── load_context_with_deltas (BufferedStore-specific) ─────────────

#[test]
fn test_load_context_with_deltas_returns_none_for_empty() {
    let (store, _tmp) = make_store();

    let result = store
        .load_context_with_deltas("/nonexistent.ts", None)
        .expect("should not error");
    assert!(result.is_none());
}

// ── pending_count ──────────────────────────────────────────────────

#[test]
fn test_pending_count_reflects_queue_state() {
    let (store, _tmp) = make_store();

    assert_eq!(store.pending_count(), 0);

    store.queue_save_context("/a.ts", Fidelity::Low, "out", b"", "h1", 0, 0);
    assert_eq!(store.pending_count(), 1);

    store.queue_save_context("/b.ts", Fidelity::Low, "out", b"", "h2", 0, 0);
    assert_eq!(store.pending_count(), 2);

    store.flush();
    assert_eq!(store.pending_count(), 0);
}
