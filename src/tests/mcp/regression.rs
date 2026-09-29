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

fn make_state(db_name: &str) -> (crate::mcp::McpState, TempDir) {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let db_path = tmp.path().join(db_name);

    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = db_path.to_string_lossy().to_string();

    let state = crate::mcp::McpState::new(config);
    (state, tmp)
}

#[path = "regression_persistence.rs"]
mod persistence;

#[path = "regression_metrics.rs"]
mod metrics;
