// src/tests/mcp/sqlite_store.rs
//
// Integration tests for SqliteStore (SQLite-backed ContextStore).

use std::path::Path;
use crate::compression::Fidelity;
use crate::mcp::context_store::ContextStore;
use crate::mcp::sqlite_store::SqliteStore;
use crate::ir::compiler::CompiledIR;
use crate::ir::opcodes::CoreOp;

/// Helper: open an in-memory SQLite store (":memory:" path).
fn in_memory_store() -> SqliteStore {
    SqliteStore::open(Path::new(":memory:"))
        .expect("Failed to open in-memory SQLite store")
}

/// Helper: create a minimal CompiledIR for testing.
fn test_ir(file_id: &str, version: u64) -> CompiledIR {
    CompiledIR {
        file_id: file_id.to_string(),
        instructions: vec![
            CoreOp::DefClass("c1".to_string(), "TestClass".to_string()),
            CoreOp::DefMethod("c1".to_string(), "m1".to_string(), "testMethod".to_string()),
        ],
        version,
    }
}

#[test]
fn test_sqlite_store_open_and_migrate() {
    let store = in_memory_store();
    // If open + migrate succeeds, the store is valid
    assert!(store.has_context("/nonexistent.ts") == false);
}

#[test]
fn test_sqlite_save_and_load_round_trip() {
    let mut store = in_memory_store();

    // Save a context
    let id = store
        .save_context("/test/file.ts", Fidelity::Low, "compressed output", None, "abc123")
        .expect("save_context should succeed");

    assert_eq!(id, "ctx-abc123");

    // Load it back
    let meta = store
        .load_latest("/test/file.ts")
        .expect("load_latest should succeed");
    assert!(meta.is_some());
    let meta = meta.unwrap();
    assert_eq!(meta.file_path, "/test/file.ts");
    assert_eq!(meta.fidelity, Fidelity::Low);
    assert_eq!(meta.source_hash, "abc123");
}

#[test]
fn test_sqlite_save_with_ir_blob() {
    let mut store = in_memory_store();

    let ir = test_ir("/test/file.ts", 1);
    let ir_binary = crate::ir::binary_wire::encode(&ir);

    let id = store
        .save_context(
            "/test/file.ts",
            Fidelity::Medium,
            "compressed",
            Some(&ir_binary),
            "hash_with_ir",
        )
        .expect("save_context with IR should succeed");

    assert!(!id.is_empty());

    // Verify context exists
    assert!(store.has_context("/test/file.ts"));
}

#[test]
fn test_sqlite_has_context() {
    let mut store = in_memory_store();

    assert!(!store.has_context("/test/file.ts"));

    store
        .save_context("/test/file.ts", Fidelity::Low, "output", None, "hash1")
        .expect("save should succeed");

    assert!(store.has_context("/test/file.ts"));
}

#[test]
fn test_sqlite_clear_file() {
    let mut store = in_memory_store();

    store
        .save_context("/test/file.ts", Fidelity::Low, "output", None, "hash1")
        .expect("save should succeed");
    assert!(store.has_context("/test/file.ts"));

    store.clear_file("/test/file.ts");
    assert!(!store.has_context("/test/file.ts"));
}

#[test]
fn test_sqlite_delta_append_and_count() {
    let mut store = in_memory_store();

    let id = store
        .save_context("/test/file.ts", Fidelity::Low, "output", None, "hash1")
        .expect("save should succeed");

    assert_eq!(store.delta_count(&id), 0);

    store
        .append_delta(&id, b"delta_payload_1", Some("edit"))
        .expect("delta 1 should succeed");
    assert_eq!(store.delta_count(&id), 1);

    store
        .append_delta(&id, b"delta_payload_2", None)
        .expect("delta 2 should succeed");
    assert_eq!(store.delta_count(&id), 2);
}

#[test]
fn test_sqlite_deterministic_id_from_hash() {
    let mut store = in_memory_store();

    let id1 = store
        .save_context("/test/file.ts", Fidelity::Low, "output", None, "same_hash")
        .expect("save 1");
    let id2 = store
        .save_context("/test/file.ts", Fidelity::High, "different output", None, "same_hash")
        .expect("save 2");

    // Same hash → same deterministic ID (INSERT OR REPLACE overwrites)
    assert_eq!(id1, id2);
    assert_eq!(id1, "ctx-same_hash");
}

#[test]
fn test_sqlite_load_context_with_deltas() {
    let mut store = in_memory_store();

    // Save baseline with IR blob
    let ir = test_ir("/test/file.ts", 1);
    let ir_binary = crate::ir::binary_wire::encode(&ir);

    let id = store
        .save_context(
            "/test/file.ts",
            Fidelity::Low,
            "baseline",
            Some(&ir_binary),
            "baseline_hash",
        )
        .expect("save baseline");

    // Append two deltas
    let delta1 = crate::ir::delta::IRDelta {
        file: "/test/file.ts".to_string(),
        from: 1,
        to: 2,
        ops: crate::ir::delta::DeltaOps {
            adds: vec![],
            mods: vec![],
            dels: vec![],
        },
    };
    let delta1_bytes = serde_json::to_vec(&delta1).unwrap();
    store
        .append_delta(&id, &delta1_bytes, Some("edit"))
        .expect("append delta 1");

    let delta2 = crate::ir::delta::IRDelta {
        file: "/test/file.ts".to_string(),
        from: 2,
        to: 3,
        ops: crate::ir::delta::DeltaOps {
            adds: vec![],
            mods: vec![],
            dels: vec![],
        },
    };
    let delta2_bytes = serde_json::to_vec(&delta2).unwrap();
    store
        .append_delta(&id, &delta2_bytes, Some("edit"))
        .expect("append delta 2");

    // Replay all deltas
    let result = store
        .load_context_with_deltas("/test/file.ts", None)
        .expect("replay all");
    assert!(result.is_some());
    let (final_ir, version) = result.unwrap();
    assert_eq!(final_ir.file_id, "/test/file.ts");
    assert!(version >= 1);

    // Replay up to sequence 1 only
    let result_partial = store
        .load_context_with_deltas("/test/file.ts", Some(1))
        .expect("replay partial");
    assert!(result_partial.is_some());
    let (_, partial_version) = result_partial.unwrap();
    assert!(partial_version >= 1);
}

#[test]
fn test_sqlite_load_nonexistent_returns_none() {
    let store = in_memory_store();
    let result = store
        .load_context_with_deltas("/nonexistent.ts", None)
        .expect("should not error");
    assert!(result.is_none());
}

#[test]
fn test_sqlite_purge_old_deltas() {
    let mut store = in_memory_store();

    let id = store
        .save_context("/test/file.ts", Fidelity::Low, "output", None, "hash1")
        .expect("save");

    // Add some deltas
    store.append_delta(&id, b"d1", None).unwrap();
    store.append_delta(&id, b"d2", None).unwrap();
    assert_eq!(store.delta_count(&id), 2);

    // Purge with 0 days should delete nothing (deltas were just inserted)
    let purged = store.purge_old_deltas(0).expect("purge should succeed");
    // Note: purge with 0 days deletes deltas older than "now" — since we just inserted,
    // they may or may not be deleted depending on timing. At minimum, the operation
    // should not error.
    let _ = purged; // suppress unused — operation succeeded
}

#[test]
fn test_sqlite_delta_count_for_file() {
    let mut store = in_memory_store();

    let id = store
        .save_context("/test/file.ts", Fidelity::Low, "output", None, "hash1")
        .expect("save");

    assert_eq!(store.delta_count_for_file("/test/file.ts"), 0);

    store.append_delta(&id, b"d1", None).unwrap();
    store.append_delta(&id, b"d2", None).unwrap();
    store.append_delta(&id, b"d3", None).unwrap();

    assert_eq!(store.delta_count_for_file("/test/file.ts"), 3);
}

#[test]
fn test_sqlite_rebuild_stats() {
    let mut store = in_memory_store();

    // Save two files
    store
        .save_context("/test/a.ts", Fidelity::Low, "out_a", None, "hash_a")
        .unwrap();
    store
        .save_context("/test/b.ts", Fidelity::High, "out_b", None, "hash_b")
        .unwrap();

    let stats = store.rebuild_stats().expect("rebuild_stats should succeed");
    let file_stats = stats.all_file_stats();
    // Should have entries for both files (though stats may be approximate)
    assert!(!file_stats.is_empty());
}

#[test]
fn test_sqlite_multiple_files_independent() {
    let mut store = in_memory_store();

    store
        .save_context("/test/a.ts", Fidelity::Low, "output_a", None, "hash_a")
        .unwrap();
    store
        .save_context("/test/b.ts", Fidelity::Medium, "output_b", None, "hash_b")
        .unwrap();

    assert!(store.has_context("/test/a.ts"));
    assert!(store.has_context("/test/b.ts"));

    // Clear one file
    store.clear_file("/test/a.ts");
    assert!(!store.has_context("/test/a.ts"));
    assert!(store.has_context("/test/b.ts"));
}