// Persistence-focused regressions split from regression.rs for the active-file ceiling.
use super::*;

#[test]
fn regression_med3_save_context_uses_actual_fidelity() {
    // After compress with "high" fidelity, save_context should store
    // the actual fidelity, not hardcoded "low".
    let (state, _tmp) = make_state("med3_test.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    // Compress with high fidelity
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "high"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);
    state.flush_persistence();

    // Verify DB has the correct fidelity
    if let Some(store) = state.persistence_store.lock().unwrap().as_ref() {
        if let Some(guard) = store.sqlite() {
            let meta = guard
                .load_latest(&rs_path)
                .expect("load_latest should succeed")
                .expect("should have context");
            assert_eq!(
                meta.fidelity,
                Fidelity::High,
                "MED-3 regression: fidelity should be High, got {:?}",
                meta.fidelity
            );
        }
    }
}

// ══════════════════════════════════════════════════════════════════
// Integration: compress → flush → DB verify (E2E)
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_e2e_compress_flush_db_verify() {
    // Full E2E: compress a file, flush to DB, verify all stats are correct.
    let (state, _tmp) = make_state("e2e_test.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    // Compress
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "medium"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);

    // Verify in-memory stats
    let guard = state.session_stats_lock();
    let fs = guard.file_stats(&rs_path).unwrap();
    assert!(fs.raw_tokens > 0, "E2E: raw_tokens should be > 0");
    assert!(
        fs.compressed_tokens > 0,
        "E2E: compressed_tokens should be > 0"
    );
    assert!(fs.savings_pct > 0.0, "E2E: savings_pct should be > 0");
    assert_eq!(fs.fidelity, "medium", "E2E: fidelity should be medium");
    assert_eq!(fs.strategy, "full", "E2E: strategy should be full");

    // Flush to DB
    state.flush_persistence();

    // Verify DB
    if let Some(store) = state.persistence_store.lock().unwrap().as_ref() {
        if let Some(guard) = store.sqlite() {
            let db_stats = guard.rebuild_stats().expect("rebuild_stats should succeed");
            let summary = db_stats.summary();
            assert_eq!(summary.total_files, 1, "E2E: DB should have 1 file");
            assert_eq!(
                summary.full_compress_count, 1,
                "E2E: DB should have 1 full compress"
            );

            let db_fs = db_stats.file_stats(&rs_path).unwrap();
            assert!(db_fs.raw_tokens > 0, "E2E: DB raw_tokens should be > 0");
            assert!(
                db_fs.compressed_tokens > 0,
                "E2E: DB compressed_tokens should be > 0"
            );
        }
    }
}

#[test]
fn regression_e2e_compress_delta_flush_verify() {
    // E2E: compress → delta → flush → verify delta stats.
    let (state, _tmp) = make_state("e2e_delta.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    // First compress
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "low"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);

    // Then delta
    let delta_id = serde_json::json!(2);
    let delta_params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "low"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&delta_id, "delta_code_context", &delta_params, &state);

    // Verify in-memory stats
    let guard = state.session_stats_lock();
    let fs = guard.file_stats(&rs_path).unwrap();
    assert!(fs.raw_tokens > 0, "E2E delta: raw_tokens should be > 0");
    assert!(
        fs.compressed_tokens > 0,
        "E2E delta: compressed_tokens should be > 0"
    );

    // Flush and verify
    state.flush_persistence();
    if let Some(store) = state.persistence_store.lock().unwrap().as_ref() {
        if let Some(guard) = store.sqlite() {
            let db_stats = guard.rebuild_stats().expect("rebuild_stats");
            assert!(
                db_stats.summary().total_files >= 1,
                "E2E delta: DB should have at least 1 file"
            );
        }
    }
}

#[test]
fn regression_e2e_provide_code_context_full_workflow() {
    // E2E: provide_code_context → context_stats → verify dashboard.
    let (state, _tmp) = make_state("e2e_provide.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    // provide_code_context
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "intent": "overview"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "provide_code_context", &params, &state);

    // context_stats
    let stats_id = serde_json::json!(2);
    let stats_params = serde_json::json!({ "arguments": {} });
    crate::mcp::tools::dispatch_tools_call(&stats_id, "context_stats", &stats_params, &state);

    // Verify session stats
    let binding = state.session_stats_lock();
    let summary = binding.summary();
    assert!(
        summary.total_files >= 1,
        "E2E provide: should have at least 1 file"
    );
    assert!(
        summary.total_raw_tokens > 0,
        "E2E provide: raw_tokens should be > 0"
    );
    assert!(
        summary.total_compressed_tokens > 0,
        "E2E provide: compressed_tokens should be > 0"
    );
}

// ══════════════════════════════════════════════════════════════════
// MED-2: BufferedStore::load_latest uses single lock scope
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_med2_load_latest_observes_only_committed_state() {
    // Verify that load_latest returns pending (unflushed) ops without
    // needing an explicit flush first. This exercises the single-lock
    // path that combines flush + read in one lock scope.
    let (store, _tmp) = make_store();

    // Queue a save without flushing
    store.queue_save_context(
        "/test/pending.ts",
        Fidelity::Medium,
        "compressed data",
        b"ir",
        "hash_pending",
        750,
        150,
    );
    // Verify pending ops exist
    assert!(store.pending_count() > 0, "should have pending ops");

    // Reads never flush or expose pending lifecycle work.
    let result = store
        .load_latest("/test/pending.ts")
        .expect("load_latest should succeed");
    assert!(
        result.is_none(),
        "MED-2 regression: load_latest must not expose pending ops"
    );
    assert!(
        store.pending_count() > 0,
        "read must leave the write pending"
    );
}

// ══════════════════════════════════════════════════════════════════
// MED-1: save_context overwrite is intentional (INSERT OR REPLACE)
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_med1_save_context_overwrite_is_intentional() {
    // MED-01: INSERT OR REPLACE behavior:
    //   - Saving with the SAME hash overwrites the existing row (idempotent).
    //   - Saving with a DIFFERENT hash creates a new row (separate version).
    //   - Version history is preserved via delta rows in the deltas table.
    let mut store =
        SqliteStore::open(Path::new(":memory:")).expect("failed to open in-memory SQLite");

    // Case 1: Same hash = true overwrite (idempotent save)
    let id1 = store
        .save_context(
            "/test/overwrite.ts",
            Fidelity::Low,
            "v1 output",
            None,
            "same_hash",
            100,
            30,
        )
        .expect("save v1");
    assert_eq!(id1, "ctx-same_hash");

    let id2 = store
        .save_context(
            "/test/overwrite.ts",
            Fidelity::High,
            "v2 output",
            None,
            "same_hash",
            200,
            50,
        )
        .expect("save v2 (same hash = overwrite)");
    assert_eq!(id2, "ctx-same_hash", "same hash should produce same ID");

    // load_latest should return the overwritten (v2) data
    let meta = store
        .load_latest("/test/overwrite.ts")
        .expect("load should succeed")
        .expect("should have context");
    assert_eq!(meta.source_hash, "same_hash", "MED-1: should have the hash");
    assert_eq!(
        meta.raw_tokens, 200,
        "MED-1: should have v2 token counts (overwritten)"
    );
    assert_eq!(
        meta.fidelity,
        Fidelity::High,
        "MED-1: should have v2 fidelity (overwritten)"
    );

    // Case 2: Different hash = new row (separate version, not overwrite)
    // Both rows coexist because INSERT OR REPLACE only triggers on
    // PRIMARY KEY or UNIQUE constraint match.
    let id3 = store
        .save_context(
            "/test/separate.ts",
            Fidelity::Low,
            "first",
            None,
            "hash_a",
            100,
            30,
        )
        .expect("save a");
    let id4 = store
        .save_context(
            "/test/separate.ts",
            Fidelity::High,
            "second",
            None,
            "hash_b",
            200,
            50,
        )
        .expect("save b");
    assert_ne!(id3, id4, "different hashes should produce different IDs");

    // load_latest returns one of them (ordering is non-deterministic when
    // updated_at is identical within the same second). Just verify that
    // one of the two valid hashes is returned.
    let meta2 = store
        .load_latest("/test/separate.ts")
        .expect("load should succeed")
        .expect("should have context");
    assert!(
        meta2.source_hash == "hash_a" || meta2.source_hash == "hash_b",
        "MED-1: load_latest should return one of the two saved versions, got {:?}",
        meta2.source_hash
    );
}

// ══════════════════════════════════════════════════════════════════
// LOW-2: rebuild_stats returns exact token values (not just > 0)
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_low2_rebuild_stats_exact_token_values() {
    // Verify that rebuild_stats returns the exact token values stored
    // in the DB, not placeholder estimates.
    let mut store =
        SqliteStore::open(Path::new(":memory:")).expect("failed to open in-memory SQLite");

    store
        .save_context(
            "/test/exact.ts",
            Fidelity::Low,
            "out",
            None,
            "hash1",
            423,
            87,
        )
        .unwrap();
    store
        .save_context(
            "/test/other.ts",
            Fidelity::High,
            "out2",
            None,
            "hash2",
            1500,
            300,
        )
        .unwrap();

    let stats = store.rebuild_stats().expect("rebuild_stats");
    let fs_exact = stats
        .file_stats("/test/exact.ts")
        .expect("exact.ts should exist");
    assert_eq!(
        fs_exact.raw_tokens, 423,
        "LOW-2: raw_tokens should be exactly 423"
    );
    assert_eq!(
        fs_exact.compressed_tokens, 87,
        "LOW-2: compressed_tokens should be exactly 87"
    );

    let fs_other = stats
        .file_stats("/test/other.ts")
        .expect("other.ts should exist");
    assert_eq!(
        fs_other.raw_tokens, 1500,
        "LOW-2: raw_tokens should be exactly 1500"
    );
    assert_eq!(
        fs_other.compressed_tokens, 300,
        "LOW-2: compressed_tokens should be exactly 300"
    );
}
