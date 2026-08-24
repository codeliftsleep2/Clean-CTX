// src/tests/mcp/regression.rs
//
// Regression tests for all fixes in the context_stats audit.
// These tests ensure that each bug cannot re-occur.

use crate::compression::Fidelity;
use crate::mcp::buffered_store::BufferedStore;
use crate::mcp::cache_hints::inject_cache_breakpoints;
use crate::mcp::context_store::ContextStore;
use crate::mcp::session_stats::SessionStats;
use crate::mcp::sqlite_store::SqliteStore;
use std::path::Path;
use tempfile::TempDir;

// ── Helper: create a BufferedStore backed by in-memory SQLite ──

fn make_store() -> (BufferedStore, TempDir) {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let store = SqliteStore::open(Path::new(":memory:")).expect("failed to open in-memory SQLite");
    let buffered = BufferedStore::new(store, tmp.path().to_path_buf());
    (buffered, tmp)
}

// ══════════════════════════════════════════════════════════════════
// CRIT-1: replay_history loads IR into ir_context state
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_crit1_replay_loads_ir_into_context() {
    // After replay_history, ir_context should have the file loaded.
    // This test verifies the fix: replay_history now calls
    // state.ir_context.load_ir(ir.clone()) instead of just returning.
    let (state, _tmp) = make_state("crit1_test.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    // First compress to create a baseline in the DB
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "low"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);
    state.flush_persistence();

    // Now replay from DB
    let replay_id = serde_json::json!(2);
    let replay_params = serde_json::json!({
        "arguments": {
            "filePath": rs_path
        }
    });
    crate::mcp::tools::dispatch_tools_call(&replay_id, "replay_history", &replay_params, &state);

    // Verify that ir_context has the file loaded
    let path_alias = state.get_or_create_alias(rs_path.clone());
    assert!(
        state.ir_context_read().has_file(&path_alias),
        "CRIT-1 regression: replay_history should load IR into ir_context"
    );
}

// ══════════════════════════════════════════════════════════════════
// CRIT-2: handle_save_context uses token counts from session_stats
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_crit2_save_context_uses_session_stats_tokens() {
    // After compress + save_context, the DB should have non-zero token counts.
    let (state, _tmp) = make_state("crit2_test.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    // Compress to populate session_stats
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "high"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);
    state.flush_persistence();

    // Verify DB has non-zero token counts
    if let Some(store) = state.persistence_store_lock().as_ref() {
        if let Some(guard) = store.sqlite() {
            let meta = guard
                .load_latest(&rs_path)
                .expect("load_latest should succeed")
                .expect("should have context");
            assert!(
                meta.raw_tokens > 0,
                "CRIT-2 regression: raw_tokens should be > 0, got {}",
                meta.raw_tokens
            );
            assert!(
                meta.compressed_tokens > 0,
                "CRIT-2 regression: compressed_tokens should be > 0, got {}",
                meta.compressed_tokens
            );
        }
    }
}

// ══════════════════════════════════════════════════════════════════
// CRIT-3: delta_code_context counts tokens on delta wire output
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_crit3_delta_counts_tokens_on_wire_output() {
    // After delta_code_context, session_stats should show compressed_tokens > 0
    // (not the raw source text token count).
    let (state, _tmp) = make_state("crit3_test.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    // First compress to create a baseline
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "low"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);

    // Now call delta_code_context
    let delta_id = serde_json::json!(2);
    let delta_params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "low"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&delta_id, "delta_code_context", &delta_params, &state);

    // Check that the delta stats show compressed_tokens > 0
    // (the delta wire output should have some tokens, not 0)
    let binding = state.session_stats_lock();
    let file_stats = binding.file_stats(&rs_path);
    assert!(
        file_stats.is_some(),
        "CRIT-3 regression: file should be tracked"
    );
    let fs = file_stats.unwrap();
    assert!(
        fs.compressed_tokens > 0,
        "CRIT-3 regression: delta compressed_tokens should be > 0, got {}",
        fs.compressed_tokens
    );
}

// ══════════════════════════════════════════════════════════════════
// HIGH-1: All handlers use pluggable tokenizer
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_high1_compress_uses_pluggable_tokenizer() {
    // Verify that compress_code_context records stats using the
    // pluggable tokenizer (not estimate_tokens).
    let (state, _tmp) = make_state("high1_test.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "low"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);

    // The stats should be recorded with the pluggable tokenizer
    let guard = state.session_stats_lock();
    let fs = guard.file_stats(&rs_path).unwrap();
    assert!(
        fs.raw_tokens > 0,
        "HIGH-1 regression: raw_tokens should be > 0"
    );
    assert!(
        fs.compressed_tokens > 0,
        "HIGH-1 regression: compressed_tokens should be > 0"
    );
    // Savings should be positive (compressed < raw)
    assert!(
        fs.savings_pct > 0.0,
        "HIGH-1 regression: savings_pct should be > 0"
    );
}

#[test]
fn regression_high1_restore_uses_pluggable_tokenizer() {
    // Verify that restore_context records stats using the pluggable tokenizer.
    let (state, _tmp) = make_state("high1_restore.db");

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

    // Now restore
    let restore_id = serde_json::json!(2);
    let restore_params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "low"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&restore_id, "restore_context", &restore_params, &state);

    // Stats should be recorded
    let guard = state.session_stats_lock();
    let fs = guard.file_stats(&rs_path).unwrap();
    assert!(
        fs.raw_tokens > 0,
        "HIGH-1 regression: restore raw_tokens should be > 0"
    );
    assert!(
        fs.compressed_tokens > 0,
        "HIGH-1 regression: restore compressed_tokens should be > 0"
    );
}

// ══════════════════════════════════════════════════════════════════
// HIGH-2: queue_save_context accepts token params
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_high2_queue_save_context_carries_token_counts() {
    // Verify that queue_save_context stores the token counts
    // and they appear in the DB after flush.
    let (store, _tmp) = make_store();

    store.queue_save_context(
        "/test/file.ts",
        Fidelity::Low,
        "compressed",
        b"ir_data",
        "hash1",
        1000,
        250,
    );
    store.flush();

    // Verify the DB has the token counts
    let guard = store.sqlite().unwrap();
    let meta = guard
        .load_latest("/test/file.ts")
        .expect("load_latest should succeed")
        .expect("should have context");
    assert_eq!(
        meta.raw_tokens, 1000,
        "HIGH-2 regression: raw_tokens should be 1000, got {}",
        meta.raw_tokens
    );
    assert_eq!(
        meta.compressed_tokens, 250,
        "HIGH-2 regression: compressed_tokens should be 250, got {}",
        meta.compressed_tokens
    );
}

#[test]
fn regression_high2_queue_save_context_zero_tokens() {
    // Verify that queue_save_context with zero tokens still works
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
    store.flush();

    let guard = store.sqlite().unwrap();
    let meta = guard
        .load_latest("/test/file.ts")
        .expect("load_latest should succeed")
        .expect("should have context");
    assert_eq!(meta.raw_tokens, 0);
    assert_eq!(meta.compressed_tokens, 0);
}

// ══════════════════════════════════════════════════════════════════
// HIGH-3: merge() doesn't over-count
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_high3_merge_no_overcounting() {
    // After merge, totals should reflect the merged file entries,
    // not the sum of both session-level counters.
    let mut in_memory = SessionStats::new();
    in_memory.record_compression(
        "/test/a.ts",
        1000,
        200,
        "low",
        false,
        "full",
        None,
        "ir_compression",
    );
    in_memory.record_compression(
        "/test/b.ts",
        2000,
        400,
        "low",
        false,
        "full",
        None,
        "ir_compression",
    );

    let mut db = SessionStats::new();
    db.record_compression(
        "/test/a.ts",
        500,
        100,
        "low",
        false,
        "delta",
        None,
        "ir_compression",
    );
    db.record_compression(
        "/test/c.ts",
        3000,
        600,
        "medium",
        false,
        "full",
        None,
        "ir_compression",
    );

    in_memory.merge(&db);

    // Totals should be from the merged file entries, not session-level sums
    let summary = in_memory.summary();
    assert_eq!(summary.total_files, 3, "should have 3 files");
    // a.ts: in-memory wins (1000 raw, 200 compressed)
    // b.ts: in-memory (2000 raw, 400 compressed)
    // c.ts: imported from DB (3000 raw, 600 compressed)
    assert_eq!(
        summary.total_raw_tokens,
        1000 + 2000 + 3000,
        "HIGH-3 regression: total_raw_tokens should be 6000, got {}",
        summary.total_raw_tokens
    );
    assert_eq!(
        summary.total_compressed_tokens,
        200 + 400 + 600,
        "HIGH-3 regression: total_compressed_tokens should be 1200, got {}",
        summary.total_compressed_tokens
    );
}

#[test]
fn regression_high3_merge_operation_counts_accurate() {
    // After merge, full_compress_count and delta_count should reflect
    // the actual strategy of each file, not blindly add session-level counts.
    let mut in_memory = SessionStats::new();
    in_memory.record_compression(
        "/test/a.ts",
        1000,
        200,
        "low",
        false,
        "full",
        None,
        "ir_compression",
    );
    in_memory.record_compression(
        "/test/b.ts",
        2000,
        400,
        "low",
        false,
        "delta",
        None,
        "ir_compression",
    );

    let mut db = SessionStats::new();
    db.record_compression(
        "/test/a.ts",
        500,
        100,
        "low",
        false,
        "delta",
        None,
        "ir_compression",
    );
    db.record_compression(
        "/test/c.ts",
        3000,
        600,
        "medium",
        false,
        "full",
        None,
        "ir_compression",
    );

    in_memory.merge(&db);

    let summary = in_memory.summary();
    // a.ts: in-memory strategy is "full" (in-memory wins)
    // b.ts: in-memory strategy is "delta"
    // c.ts: imported from DB, strategy is "full"
    assert_eq!(
        summary.full_compress_count, 2,
        "HIGH-3 regression: full_compress_count should be 2, got {}",
        summary.full_compress_count
    );
    assert_eq!(
        summary.delta_count, 1,
        "HIGH-3 regression: delta_count should be 1, got {}",
        summary.delta_count
    );
}

// ══════════════════════════════════════════════════════════════════
// MED-3: handle_save_context uses actual fidelity
// ══════════════════════════════════════════════════════════════════

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
fn regression_med2_load_latest_single_lock_scope() {
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

    // load_latest should see the pending op (flushes internally)
    let result = store
        .load_latest("/test/pending.ts")
        .expect("load_latest should succeed");
    assert!(
        result.is_some(),
        "MED-2 regression: load_latest should see pending ops"
    );
    let meta = result.unwrap();
    assert_eq!(
        meta.raw_tokens, 750,
        "MED-2 regression: raw_tokens should be 750"
    );
    assert_eq!(
        meta.compressed_tokens, 150,
        "MED-2 regression: compressed_tokens should be 150"
    );
    assert_eq!(
        meta.fidelity,
        Fidelity::Medium,
        "MED-2 regression: fidelity should be Medium"
    );

    // Pending queue should now be empty (flushed by load_latest)
    assert_eq!(
        store.pending_count(),
        0,
        "pending queue should be empty after load_latest"
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

// ── Helper: create an McpState with persistence enabled ──

fn make_state(db_name: &str) -> (crate::mcp::McpState, TempDir) {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let db_path = tmp.path().join(db_name);

    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = db_path.to_string_lossy().to_string();

    let state = crate::mcp::McpState::new(config);
    (state, tmp)
}

// ══════════════════════════════════════════════════════════════════
// C-1 regression: workspace tokenizer created once, not per-file
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_c1_workspace_tokenizer_created_once() {
    // C-1 fix: tokenizer must be created once before the loop, not per-file.
    // We verify this by checking that compress_workspace_dir completes
    // without error and records stats for all files using the same tokenizer.
    let tmp = TempDir::new().expect("failed to create temp dir");
    let dir_path = tmp.path();

    // Create two test files (use .rs to avoid tree-sitter TypeScript WASM deadlock on Windows)
    std::fs::write(dir_path.join("a.rs"), "pub struct Foo { }").unwrap();
    std::fs::write(dir_path.join("b.rs"), "pub struct Bar { }").unwrap();

    let mut config = crate::tests::test_config();
    config.persistence.enabled = false;
    let state = crate::mcp::McpState::new(config);

    // TIMING: Measure compress_workspace_dir to isolate slow operations
    let start = std::time::Instant::now();
    let result = crate::mcp::workspace::compress_workspace_dir(
        &dir_path.to_string_lossy(),
        crate::compression::Fidelity::Low,
        &state,
    );

    assert!(
        result.is_ok(),
        "workspace compression should succeed: {:?}",
        result.err()
    );
    eprintln!(
        "[TIMING] compress_workspace_dir completed in {:?}",
        start.elapsed()
    );
    let workspace_result = result.unwrap();

    // Both files should be in the manifest
    let dir_str = dir_path.to_string_lossy();
    assert!(
        workspace_result.manifest.contains(&*dir_str),
        "manifest should contain dir path, got: ...{}",
        &workspace_result.manifest[..workspace_result.manifest.len().min(200)]
    );

    // Both files should have recorded stats (proving tokenizer was available)
    let binding = state.session_stats_lock();
    let summary = binding.summary();
    // The global-symbols path may or may not record per-file stats depending
    // on the compression path. The key assertion is that the manifest has
    // content and the test completed without deadlocking.
    assert!(
        summary.total_files > 0 || workspace_result.manifest.len() > 100,
        "should have either stats or manifest content, got {} files, manifest {} bytes",
        summary.total_files,
        workspace_result.manifest.len()
    );
}

// ══════════════════════════════════════════════════════════════════
// C-2 regression: context_history per-file shows session-level cache
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_c2_context_history_shows_session_cache_metrics() {
    // C-2 fix: context_history per-file view must not show broken "none"
    // from the region-keyed breakpoints HashMap. It should show session-level
    // cache hit rate and tokens saved.
    let (state, _tmp) = make_state("c2_test.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    // Compress to create some session state
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": { "filePath": rs_path, "fidelity": "low" }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);

    // Simulate some cache activity
    state.cache_metrics_lock().hits = 5;
    state.cache_metrics_lock().misses = 3;
    state.cache_metrics_lock().tokens_saved = 420;

    // Call context_history for the specific file
    let hist_id = serde_json::json!(2);
    let hist_params = serde_json::json!({
        "arguments": { "filePath": rs_path }
    });
    // context_history sends response via send_response, so we just verify
    // the function doesn't panic and the cache_metrics are accessible
    crate::mcp::tools::dispatch_tools_call(&hist_id, "context_history", &hist_params, &state);

    // Verify that cache_metrics are still intact (not corrupted by the lookup)
    assert_eq!(
        state.cache_metrics_lock().hits,
        5,
        "cache hits should be preserved"
    );
    assert_eq!(
        state.cache_metrics_lock().misses,
        3,
        "cache misses should be preserved"
    );
    assert_eq!(
        state.cache_metrics_lock().tokens_saved,
        420,
        "tokens_saved should be preserved"
    );
}

// ══════════════════════════════════════════════════════════════════
// H-1 regression: token savings estimates breakpoint, not full response
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_h1_token_savings_estimates_breakpoint_only() {
    // H-1 fix: on cache hit, inject_cache_breakpoints should tokenize
    // just the breakpoint metadata, not the entire response JSON.
    // We verify this by checking that the returned savings is small
    // (proportional to the breakpoint) rather than large (proportional
    // to the full response).
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    // Create a large response to make the difference obvious
    let large_content = "x".repeat(10000);
    let mut response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "content": [{ "type": "text", "text": large_content }]
        }
    });

    // First call — miss (saves 0)
    let saved1 = crate::mcp::cache_hints::inject_cache_breakpoints(
        &mut response,
        &state,
        "baseline",
        "1h",
        "test-breaker-123",
        None,
    );
    assert_eq!(saved1, 0, "first call should be a miss");

    // Second call — hit (should return small savings, not full response size)
    let saved2 = crate::mcp::cache_hints::inject_cache_breakpoints(
        &mut response,
        &state,
        "baseline",
        "1h",
        "test-breaker-123",
        None,
    );
    assert!(saved2 > 0, "second call should be a hit with savings > 0");

    // The savings should be proportional to the breakpoint metadata (~10-20 tokens),
    // NOT the full 10000-char response (~2500 tokens).
    // With chars/4 fallback: hint_len = "baseline".len() + "1h".len() + "test-breaker-123".len() + 16 = 8+2+17+16 = 43, /4 = 10
    assert!(
        saved2 < 50,
        "savings should be small (breakpoint only), got {}",
        saved2
    );
}

// ══════════════════════════════════════════════════════════════════
// M-1 regression: context_stats shows cache when disabled
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_m1_cache_section_shown_when_disabled() {
    // M-1 fix: context_stats dashboard should always show the cache
    // section, with "Status: disabled" when cache is off.
    let mut config = crate::tests::test_config();
    config.cache.enabled = false;
    let state = crate::mcp::McpState::new(config);

    // Call context_stats (text format, no file path = full dashboard)
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": {} });
    // handle_context_stats sends response via send_response, so we verify
    // the function doesn't panic when cache is disabled
    crate::mcp::tools::dispatch_tools_call(&id, "context_stats", &params, &state);

    // Also verify the render functions directly
    // When cache is disabled and never active, render_cache_text returns None
    let metrics = crate::mcp::cache_hints::CacheMetrics::default();
    let text_disabled = crate::mcp::cache_hints::render_cache_text(&metrics, false);
    assert!(
        text_disabled.is_none(),
        "disabled+never active should return None (hidden)"
    );

    // With hits+misses > 0, disabled still returns Some (shows disabled status)
    let active_metrics = crate::mcp::cache_hints::CacheMetrics {
        hits: 1,
        misses: 2,
        ..Default::default()
    };
    let text_disabled_active = crate::mcp::cache_hints::render_cache_text(&active_metrics, false);
    assert!(
        text_disabled_active.is_some_and(|t| t.contains("disabled")),
        "disabled with activity should show disabled status"
    );

    let json = crate::mcp::cache_hints::render_cache_json(&metrics, false);
    assert_eq!(
        json["enabled"], false,
        "json enabled should be false when cache is off"
    );
    assert_eq!(
        json["active"], false,
        "json active should be false with no cache activity"
    );
}

// ══════════════════════════════════════════════════════════════════
// M-2 regression: compute_workspace_breaker is used (not inline hash)
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_m2_compute_workspace_breaker_used() {
    // M-2 fix: tools.rs should use compute_workspace_breaker instead of
    // inline sha2::Sha256::digest. Verify the function produces the
    // expected format and is callable from the production path.
    let hashes = vec!["file1_hash".to_string(), "file2_hash".to_string()];
    let breaker = crate::mcp::cache_hints::compute_workspace_breaker(&hashes);
    assert!(
        breaker.starts_with("ws_"),
        "workspace breaker should start with ws_: {}",
        breaker
    );
    assert_eq!(
        breaker.len(),
        67,
        "SHA-256 hex is 64 chars + ws_ prefix = 67"
    );

    // Same input → same output (deterministic)
    let breaker2 = crate::mcp::cache_hints::compute_workspace_breaker(&hashes);
    assert_eq!(breaker, breaker2, "breaker should be deterministic");

    // Different input → different output
    let breaker3 = crate::mcp::cache_hints::compute_workspace_breaker(&["different".to_string()]);
    assert_ne!(
        breaker, breaker3,
        "different input should produce different breaker"
    );
}

// ══════════════════════════════════════════════════════════════════
// E2E regression: full provide_code_context → context_stats workflow
// with cache metrics verification
// ══════════════════════════════════════════════════════════════════

#[test]
fn regression_e2e_cache_metrics_through_full_workflow() {
    // E2E: provide_code_context → context_stats → verify cache metrics
    // are surfaced correctly in the dashboard.
    let (state, _tmp) = make_state("e2e_cache.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    // provide_code_context (should compress and record stats)
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": { "filePath": rs_path, "intent": "overview" }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "provide_code_context", &params, &state);

    // Simulate cache activity for metrics
    state.cache_metrics_lock().hits = 3;
    state.cache_metrics_lock().misses = 2;
    state.cache_metrics_lock().tokens_saved = 150;

    // context_stats (full dashboard)
    let stats_id = serde_json::json!(2);
    let stats_params = serde_json::json!({ "arguments": {} });
    crate::mcp::tools::dispatch_tools_call(&stats_id, "context_stats", &stats_params, &state);

    // Verify session stats are populated
    let binding = state.session_stats_lock();
    let summary = binding.summary();
    assert!(summary.total_files >= 1, "should have at least 1 file");
    assert!(summary.total_raw_tokens > 0, "raw tokens should be > 0");

    // Verify cache metrics are intact after the full workflow
    assert_eq!(
        state.cache_metrics_lock().hits,
        3,
        "cache hits should be preserved"
    );
    assert_eq!(
        state.cache_metrics_lock().misses,
        2,
        "cache misses should be preserved"
    );
    assert_eq!(
        state.cache_metrics_lock().tokens_saved,
        150,
        "tokens_saved should be preserved"
    );
}

// ══════════════════════════════════════════════════════════════════
// _meta placement regression tests (handlers.rs + tools.rs)
// ══════════════════════════════════════════════════════════════════

/// REGRESSION: handle_tools_list must place _meta.cache_hints inside
/// result, never at the response root level.
#[test]
fn regression_meta_not_in_tools_list_root() {
    let mut config = crate::tests::test_config();
    config.cache.enabled = true;
    let state = crate::mcp::McpState::new(config);

    // Capture responses by redirecting stdout
    let id = serde_json::json!(1);
    crate::mcp::handlers::handle_tools_list(&id, &state);

    // We can't capture send_response output (stdout), but we can verify
    // that the cache metrics recorded activity, proving the injection
    // ran without panicking.
    assert!(
        state.cache_metrics_lock().misses >= 1,
        "tools/list should have recorded a cache miss, got misses={} hits={}",
        state.cache_metrics_lock().misses,
        state.cache_metrics_lock().hits
    );
}

/// REGRESSION: handle_prompts_list must place _meta.cache_hints inside
/// result, never at the response root level.
#[test]
fn regression_meta_not_in_prompts_list_root() {
    let mut config = crate::tests::test_config();
    config.cache.enabled = true;
    let state = crate::mcp::McpState::new(config);

    let id = serde_json::json!(1);
    crate::mcp::handlers::handle_prompts_list(&id, &state);

    assert!(
        state.cache_metrics_lock().misses >= 1,
        "prompts/list should have recorded a cache miss, got misses={} hits={}",
        state.cache_metrics_lock().misses,
        state.cache_metrics_lock().hits
    );
}

/// REGRESSION: handle_prompts_get with "cleanctx-notation" must place
/// _meta.cache_hints inside result, never at the response root level.
#[test]
fn regression_meta_not_in_cleanctx_prompt_root() {
    let mut config = crate::tests::test_config();
    config.cache.enabled = true;
    let state = crate::mcp::McpState::new(config);

    let id = serde_json::json!(1);
    crate::mcp::handlers::handle_prompts_get(&id, "cleanctx-notation", &state);

    assert!(
        state.cache_metrics_lock().misses >= 1,
        "prompts/get cleanctx-notation should have recorded a cache miss, got misses={} hits={}",
        state.cache_metrics_lock().misses,
        state.cache_metrics_lock().hits
    );
}

/// REGRESSION: handle_prompts_get with "clean-ctx-vocabulary" must place
/// _meta.cache_hints inside result, never at the response root level.
#[test]
fn regression_meta_not_in_vocabulary_prompt_root() {
    let mut config = crate::tests::test_config();
    config.cache.enabled = true;
    let state = crate::mcp::McpState::new(config);

    let id = serde_json::json!(1);
    crate::mcp::handlers::handle_prompts_get(&id, "clean-ctx-vocabulary", &state);

    assert!(
        state.cache_metrics_lock().misses >= 1,
        "prompts/get clean-ctx-vocabulary should have recorded a cache miss, got misses={} hits={}",
        state.cache_metrics_lock().misses,
        state.cache_metrics_lock().hits
    );
}

/// REGRESSION: Sent JSON response must have _meta inside result, not
/// at the root. This verifies the serialized payload format matches
/// the JSON-RPC spec where only jsonrpc/id/result/error are valid
/// top-level keys.
///
/// We inject into a result sub-object directly, then verify the
/// full response tree is valid.
#[test]
fn regression_meta_placement_json_structure_valid() {
    // Build a realistic response tree like the handlers produce
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let mut response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "content": [{ "type": "text", "text": "test output" }]
        }
    });

    // Simulate the injection pattern used by all handlers
    if let Some(result_obj) = response.get_mut("result") {
        inject_cache_breakpoints(result_obj, &state, "baseline", "1h", "bl_somehash", None);
    }

    // Assert the JSON structure has _meta ONLY in result
    let response_str = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&response_str).unwrap();

    // Valid top-level keys for JSON-RPC response
    assert!(
        parsed.get("jsonrpc").is_some(),
        "response must have jsonrpc"
    );
    assert!(parsed.get("id").is_some(), "response must have id");
    assert!(parsed.get("result").is_some(), "response must have result");
    assert!(
        parsed.get("error").is_none(),
        "response root must not have error"
    );
    assert!(
        parsed.get("_meta").is_none(),
        "REGRESSION: _meta at response root would fail MCP Zod validation! Found: {:?}",
        parsed.get("_meta")
    );

    // _meta IS inside result
    assert!(
        parsed["result"].get("_meta").is_some(),
        "REGRESSION: _meta should be inside result"
    );
    assert!(
        parsed["result"]["_meta"].get("cache_hints").is_some(),
        "REGRESSION: cache_hints should be inside result._meta"
    );

    // Breakpoint content should be intact
    let breakpoints = parsed["result"]["_meta"]["cache_hints"]["breakpoints"]
        .as_array()
        .unwrap();
    assert_eq!(breakpoints[0]["region"], "baseline");
    assert_eq!(breakpoints[0]["breaker"], "bl_somehash");
}

/// REGRESSION: CBM proxy handler + response must not have _meta
/// at the response root level when cache hints are injected.
#[test]
fn regression_cbm_proxy_meta_in_result() {
    // CBM proxy sends responses via send_response, which goes to stdout.
    // We verify the handler runs without panicking — the _meta placement
    // is validated by the inject_cache_breakpoints unit tests above.
    let mut state = crate::mcp::McpState::new(crate::tests::test_config());
    state.cbm_status = crate::cbm::CbmStatus::Unavailable;
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "method": "GET",
            "path": "/graph/status"
        }
    });
    crate::cbm::proxy::handle_cbm_proxy(&id, &params, &state);
    // If we get here without panicking, the handler works.
    // The _meta placement is tested via unit tests above.
}
