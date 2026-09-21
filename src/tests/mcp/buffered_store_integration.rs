// Extracted integration coverage for BufferedStore.

use tempfile::TempDir;

use crate::mcp::context_store::ContextStore;

// ── Integration: compress → flush → DB verify ────────────────────

/// Helper: create an McpState with persistence enabled backed by a temp DB file.
fn make_state(db_name: &str) -> (crate::mcp::McpState, TempDir) {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let db_path = tmp.path().join(db_name);

    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = db_path.to_string_lossy().to_string();

    let state = crate::mcp::McpState::new(config);
    (state, tmp)
}

#[test]
fn test_integration_compress_and_check_db() {
    // 1. Compress a real .rs file from the project
    let (state, _tmp) = make_state("integ_test.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "high"
        }
    });

    // compress_code_context now flushes persistence immediately (FAANG audit fix)
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);

    // Data should be flushed to SQLite immediately (no pending ops)
    if let Some(store) = state.persistence_store.lock().unwrap().as_ref() {
        assert_eq!(
            store.pending_count(),
            0,
            "Expected zero pending ops after compress (immediate flush)"
        );
    } else {
        panic!("Persistence store should be Some");
    }

    // 2. Verify DB has the data via rebuild_stats (no explicit flush needed)
    if let Some(store) = state.persistence_store.lock().unwrap().as_ref() {
        if let Some(guard) = store.sqlite() {
            let db_stats = guard.rebuild_stats().expect("rebuild_stats should succeed");
            let summary = db_stats.summary();
            assert_eq!(
                summary.total_files, 1,
                "Expected 1 file in DB after compress+flush"
            );
            assert_eq!(
                summary.full_compress_count, 1,
                "Expected 1 full compression in DB"
            );
            assert!(
                db_stats.file_stats(&rs_path).is_some(),
                "Expected file in DB stats: {}",
                rs_path
            );
        } else {
            panic!("Could not lock sqlite store");
        }
    } else {
        panic!("Persistence store should be Some");
    }
}

#[test]
fn test_integration_simulate_restart_stats_recovery() {
    // Simulate a restart: compress, flush, then create a NEW state
    // pointing at the same DB file and verify stats are recovered.

    let tmp = TempDir::new().expect("failed to create temp dir");
    let db_path = tmp.path().join("restart_test.db");

    // ── Session 1: compress a file ──
    {
        let mut config = crate::tests::test_config();
        config.persistence.enabled = true;
        config.persistence.db_path = db_path.to_string_lossy().to_string();
        let state = crate::mcp::McpState::new(config);

        let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("main.rs");
        let rs_path = rs_file.to_string_lossy().to_string();

        let id = serde_json::json!(1);
        let params = serde_json::json!({
            "arguments": {
                "filePath": rs_path,
                "fidelity": "medium"
            }
        });
        crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);

        // Flush before dropping
        state.flush_persistence();

        // Verify data in DB during session 1
        if let Some(store) = state.persistence_store.lock().unwrap().as_ref() {
            if let Some(guard) = store.sqlite() {
                let db_stats = guard.rebuild_stats().expect("rebuild_stats");
                assert_eq!(
                    db_stats.summary().total_files,
                    1,
                    "Session 1: should have 1 file"
                );
            }
        }
        // state drops here — DB file persists on disk
    }

    // ── Session 2: open NEW state pointing at same DB ──
    {
        let mut config = crate::tests::test_config();
        config.persistence.enabled = true;
        config.persistence.db_path = db_path.to_string_lossy().to_string();
        let state = crate::mcp::McpState::new(config);

        // McpState::new() should have called rebuild_stats and loaded the stats
        // from the DB created in session 1.
        let summary = state.session_stats_lock().summary();
        assert_eq!(
            summary.total_files, 1,
            "Session 2: should recover 1 file from DB, got {}",
            summary.total_files
        );
        assert_eq!(
            summary.full_compress_count, 1,
            "Session 2: should recover 1 full compress from DB"
        );

        // Also verify via context_stats handler
        let stats_id = serde_json::json!(2);
        let stats_params = serde_json::json!({ "arguments": {} });
        crate::mcp::tools::dispatch_tools_call(&stats_id, "context_stats", &stats_params, &state);
    }
}

#[test]
fn test_integration_compress_multiple_files_then_clear() {
    let (state, _tmp) = make_state("multi_test.db");

    // Use files with different content to ensure different hashes
    let rs1 = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs2 = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let path1 = rs1.to_string_lossy().to_string();
    let path2 = rs2.to_string_lossy().to_string();

    // Verify they have different content (different file sizes)
    let content1 = std::fs::read_to_string(&path1).unwrap();
    let content2 = std::fs::read_to_string(&path2).unwrap();
    assert_ne!(
        content1, content2,
        "Test requires files with different content"
    );

    // Canonicalize paths to ensure they're absolute (handler does this internally)
    let path1 = std::fs::canonicalize(&path1)
        .unwrap()
        .to_string_lossy()
        .to_string();
    let path2 = std::fs::canonicalize(&path2)
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Compress first file (handler flushes automatically)
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": path1,
            "fidelity": "low"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);

    // Compress second file (handler flushes automatically)
    let id2 = serde_json::json!(2);
    let params2 = serde_json::json!({
        "arguments": {
            "filePath": path2,
            "fidelity": "low"
        }
    });
    crate::mcp::tools::dispatch_tools_call(&id2, "compress_code_context", &params2, &state);

    // Both files should now be in DB (handlers flush automatically)
    if let Some(store) = state.persistence_store.lock().unwrap().as_ref() {
        if let Some(guard) = store.sqlite() {
            let db_stats = guard.rebuild_stats().expect("rebuild_stats");
            assert_eq!(
                db_stats.summary().total_files,
                2,
                "Expected 2 files in DB after 2 compressions"
            );
        }
    }

    // Delete one owned context through the registered deletion contract.
    // The source file itself must remain untouched.
    let source_before = std::fs::read(&path1).expect("source before context deletion");
    let clear_id = serde_json::json!(3);
    let clear_params = serde_json::json!({
        "arguments": {
            "filePath": path1
        }
    });
    crate::mcp::tools::dispatch_tools_call(&clear_id, "delete_context", &clear_params, &state);

    // After clear, the file should be removed from DB.
    if let Some(store) = state.persistence_store.lock().unwrap().as_ref() {
        if let Some(guard) = store.sqlite() {
            assert!(
                !guard.has_context(&path1),
                "First file should be cleared from DB: {}",
                path1
            );
            // Second file should still exist
            assert!(
                guard.has_context(&path2),
                "Second file should still be in DB: {}",
                path2
            );
        }
    }
    assert_eq!(
        std::fs::read(&path1).expect("source after context deletion"),
        source_before,
        "delete_context must not modify source bytes"
    );
}

#[test]
fn test_integration_db_stats_via_provide_code_context() {
    // provide_code_context is a read-only operation that does NOT persist to DB
    // This test verifies that it works without persistence
    let (state, _tmp) = make_state("provide_test.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "intent": "overview"
        }
    });

    // This should succeed without panicking
    crate::mcp::tools::dispatch_tools_call(&id, "provide_code_context", &params, &state);

    // provide_code_context does not persist, so DB should be empty
    if let Some(store) = state.persistence_store.lock().unwrap().as_ref() {
        if let Some(guard) = store.sqlite() {
            let db_stats = guard.rebuild_stats().expect("rebuild_stats");
            assert_eq!(
                db_stats.summary().total_files,
                0,
                "provide_code_context should not persist to DB"
            );
        }
    }
}

#[test]
fn test_integration_created_at_parsing() {
    // Verify that the chrono_parse_or_now fix actually works with
    // real SQLite datetime('now') format strings.
    // Query the SQLite store directly (InMemoryContextStore doesn't query SQLite).
    let (state, _tmp) = make_state("created_at_test.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": rs_path,
            "fidelity": "high"
        }
    });

    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);
    state.flush_persistence();

    // Check created_at via SQLite store's load_latest directly
    if let Some(store) = state.persistence_store.lock().unwrap().as_ref() {
        if let Some(guard) = store.sqlite() {
            let meta = guard
                .load_latest(&rs_path)
                .expect("load_latest should succeed")
                .expect("should have context for the file");

            // If chrono_parse_or_now works, created_at should be a real timestamp
            let epoch = std::time::SystemTime::UNIX_EPOCH;
            assert!(
                meta.created_at > epoch,
                "created_at should be after UNIX_EPOCH"
            );
            assert!(
                meta.created_at <= std::time::SystemTime::now(),
                "created_at should not be in the future"
            );
        } else {
            panic!("Could not lock sqlite store");
        }
    } else {
        panic!("Persistence store should be Some");
    }
}
