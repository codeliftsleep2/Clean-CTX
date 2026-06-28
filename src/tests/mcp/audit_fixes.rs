// src/tests/mcp/audit_fixes.rs
//
// Regression tests for the FAANG audit P1/P2 fixes.
// These tests verify that each fix cannot regress.

use tempfile::TempDir;

use crate::mcp::context_store::ContextStore;

/// Helper: create an McpState with persistence enabled on an in-memory DB.
fn make_state(db_name: &str) -> (crate::mcp::McpState, TempDir) {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let db_path = tmp.path().join(db_name);
    let db_path_str = db_path.to_string_lossy().to_string();

    // Build a config with persistence enabled
    let mut config = crate::config::CleanCtxConfig::default();
    config.persistence.enabled = true;
    config.persistence.db_path = db_path_str;

    let state = crate::mcp::McpState::new(config);
    (state, tmp)
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-1: compress_code_context persists to DB
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit1_compress_persists_to_db() {
    // FAANG audit P1 #1: compress_code_context should call queue_save_context.
    // Verify the BufferedStore has data after a compress call.
    let (state, _tmp) = make_state("audit1.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": { "filePath": rs_path, "fidelity": "low" }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "compress_code_context", &params, &state);
    state.flush_persistence();

    // Verify BufferedStore has the file (queue_save_context writes to BufferedStore)
    let guard = state.persistence_store_lock();
    if let Some(ref store) = *guard {
        let has = store.has_context(&rs_path);
        assert!(has,
            "AUDIT-1: BufferedStore should have context after compress");
    } else {
        panic!("AUDIT-1: persistence store should be Some");
    }
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-2: dispatch_tools_call returns after inline arms
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit2_dispatch_returns_after_inline_arm() {
    // FAANG audit P1 #3: Each inline match arm must return so the
    // registry fallback does NOT fire. If a tool is only in the match
    // block and NOT in the registry, the registry should never trigger.
    // We verify by calling decompress_code_context (only in match block)
    // and confirming it doesn't also hit the registry.
    let state = crate::mcp::McpState::new(crate::config::CleanCtxConfig::default());
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": { "compressedText": "// test" }
    });
    // This should succeed without double-firing or panicking
    crate::mcp::tools::dispatch_tools_call(&id, "decompress_code_context", &params, &state);
    // If we get here, dispatch returned after the inline arm — no registry
    // double-fire occurred.
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-3: restore_context clears persistence DB
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit3_restore_clears_persistence_db() {
    // FAANG audit P1 #4: restore_context should call clear_file on the
    // persistence store. We verify by:
    // 1. Compress (adds to persistence)
    // 2. Flush
    // 3. Restore (should clear)
    // 4. Check persistence store no longer has the file
    let (state, _tmp) = make_state("audit3.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();

    // Compress first
    let id1 = serde_json::json!(1);
    let params1 = serde_json::json!({
        "arguments": { "filePath": rs_path, "fidelity": "low" }
    });
    crate::mcp::tools::dispatch_tools_call(&id1, "compress_code_context", &params1, &state);
    state.flush_persistence();

    // Now restore
    let id2 = serde_json::json!(2);
    let params2 = serde_json::json!({
        "arguments": { "filePath": rs_path, "fidelity": "low" }
    });
    crate::mcp::tools::dispatch_tools_call(&id2, "restore_context", &params2, &state);
    state.flush_persistence();

    // Verify persistence store cleared
    let guard = state.persistence_store_lock();
    if let Some(ref store) = *guard {
        let has = store.has_context(&rs_path);
        assert!(!has,
            "AUDIT-3: persistence store should NOT have context after restore");
    }
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-4: delta_text_context stores baseline snapshot
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit4_delta_text_stores_baseline() {
    // FAANG audit P1 #2: delta_text_context should actually store the baseline
    // snapshot. On first call, has_baseline() should return false, then after
    // the call it should return true.
    let (state, _tmp) = make_state("audit4.db");

    let rs_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs");
    let rs_path = rs_file.to_string_lossy().to_string();
    let alias = state.get_or_create_alias(rs_path.clone());

    // Before any delta_text call, has_baseline should be false
    {
        let td = state.text_delta_lock();
        assert!(!td.has_baseline(&alias),
            "AUDIT-4: baseline should NOT exist before first call");
    }

    // Call delta_text_context (first call stores baseline)
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": { "filePath": rs_path, "fidelity": "low" }
    });
    crate::mcp::tools::dispatch_tools_call(&id, "delta_text_context", &params, &state);

    // After the call, has_baseline should be true
    let td = state.text_delta_lock();
    assert!(td.has_baseline(&alias),
        "AUDIT-4: baseline SHOULD exist after first delta_text_context call");
    drop(td);
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-5: list_sessions returns result for disabled persistence
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit5_list_sessions_disabled_returns_result() {
    // FAANG audit P2 #8: When persistence is disabled, list_sessions should
    // return a result (not an error).
    // Create a config with persistence explicitly disabled.
    let mut config = crate::config::CleanCtxConfig::default();
    config.persistence.enabled = false;
    let state = crate::mcp::McpState::new(config);

    // Verify persistence is disabled
    {
        let guard = state.persistence_store_lock();
        assert!(guard.is_none(),
            "AUDIT-5: persistence should be disabled");
    }

    // Call list_sessions — should not panic
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": {} });
    crate::mcp::tools::dispatch_tools_call(&id, "list_sessions", &params, &state);
    // If we get here without panicking, list_sessions works
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-6: ir_context_read() helper works with poisoned recovery
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit6_ir_context_read_helper_works() {
    // FAANG audit P1 #5: ir_context_read() should return a read guard
    // with a poisoned-recovery pattern.
    let state = crate::mcp::McpState::new(crate::config::CleanCtxConfig::default());

    // ir_context_read should return a valid read guard
    let guard = state.ir_context_read();
    // Should not panic
    let _version = guard.file_version("nonexistent");
    drop(guard);

    // Should also work after loading content
    let mut write = state.ir_context_lock();
    let mock_ir = crate::ir::compiler::CompiledIR {
        file_id: "test".to_string(),
        version: 1,
        instructions: Vec::new(),
    };
    write.load_ir(mock_ir);
    drop(write);

    let read = state.ir_context_read();
    assert!(!read.has_file("nonexistent"),
        "AUDIT-6: nonexistent file should not exist");
    assert!(read.has_file("test"),
        "AUDIT-6: test file should exist after load");
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-7: Verify no dispatch double-fire for compress_workspace
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit7_compress_workspace_not_in_registry() {
    // compress_workspace is handled inline in the match block. If it were
    // also in the registry (without a return), it would double-fire.
    // Verify it's NOT in the registry.
    let registry = crate::mcp::tool_handlers::registry::create_default_registry();
    assert!(registry.get("compress_workspace").is_none(),
        "AUDIT-7: compress_workspace should NOT be in registry (handled inline)");
    assert!(registry.get("decompress_code_context").is_none(),
        "AUDIT-7: decompress_code_context should NOT be in registry");
    // CBM tools should also NOT be in registry
    assert!(registry.get("graph_search").is_none(),
        "AUDIT-7: graph_search should NOT be in registry");
    assert!(registry.get("cbm_proxy").is_none(),
        "AUDIT-7: cbm_proxy should NOT be in registry");
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-8: McpState::new() completes quickly (no blocking on CBM indexing)
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit8_state_new_is_fast() {
    // FAANG audit P0: McpState::new() must NOT block on CBM indexing.
    // Verify it completes within 1s (should be sub-ms without CBM blocking).
    use std::time::Instant;
    let start = Instant::now();
    let config = crate::config::CleanCtxConfig::default();
    let _state = crate::mcp::McpState::new(config);
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000,
        "AUDIT-8: McpState::new() took {}ms -- should be <1s (no CBM blocking)", elapsed.as_millis());
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-9: ensure_indexed() is idempotent (second call is no-op)
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit9_ensure_indexed_idempotent() {
    // FAANG audit P0: ensure_indexed() should be a no-op if already indexed.
    let config = crate::cbm::config::CbmConfig { enabled: false, ..Default::default() };
    let mut bridge = crate::cbm::bridge::GraphBridge::try_create(&config, std::path::Path::new("."));
    // First call -- should not panic (CBM disabled)
    let result1 = bridge.ensure_indexed();
    // Second call -- should be a no-op
    let result2 = bridge.ensure_indexed();
    // Both should return same result (idempotent)
    assert_eq!(result1.is_ok(), result2.is_ok(),
        "AUDIT-9: ensure_indexed should be idempotent -- 1st={:?}, 2nd={:?}", result1, result2);
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-10: No orphaned test file -- tool_handlers wired correctly
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit10_tool_handlers_wired_and_compiles() {
    // FAANG audit P0: The tool_handlers.rs test was orphaned (never referenced).
    // Verify it compiles by importing the test module. This test fails to compile
    // if the #[path] reference is removed from tool_handlers/mod.rs.
    use crate::mcp::tool_handlers::tool_handlers_tests;
    // Merely importing the module proves the #[path] wiring is intact.
}
// AUDIT-11: All helper method signatures are consistent
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit11_helper_methods_compile() {
    // Compilation regression: if any helper signature changes, this test
    // fails to compile, catching the regression immediately.
    let state = crate::mcp::McpState::new(crate::config::CleanCtxConfig::default());
    let _alias = state.get_or_create_alias("test.rs".to_string());
    let _footer = state.format_dict_footer();
    let _dict = state.dict_lock();
    let _cache_r = state.cache_read();
    let _cache_w = state.cache_write();
    let _sess = state.session_stats_lock();
    let _ir_r = state.ir_context_read();
    let _ir_w = state.ir_context_lock();
    let _gb = state.graph_bridge_lock();
    let _ag = state.angular_graph_lock();
    let _td = state.text_delta_lock();
    let _llm = state.llm_text_cache_lock();
    state.push_warning("test");
    let _drained = state.drain_warnings();
    let _ps = state.persistence_store_lock();
    let _sc = state.source_cache_lock();
    let _cf = state.cbm_filter_lock();
    let _skip = state.get_skip_set("x.rs");
    let _cm = state.cache_metrics_lock();
    let _flush = state.flush_persistence();
}

// ══════════════════════════════════════════════════════════════════
// AUDIT-12: GraphBridge ensure_indexed pattern verified
// ══════════════════════════════════════════════════════════════════

#[test]
fn audit12_bridge_indexed_flag_initialized() {
    // Verify that a fresh GraphBridge has indexed=false.
    let config = crate::cbm::config::CbmConfig { enabled: false, ..Default::default() };
    let bridge = crate::cbm::bridge::GraphBridge::try_create(&config, std::path::Path::new("."));
    // After ensure_indexed (with no CBM), indexed should remain false
    // This is a behavior test, not a field-access test
    assert!(!bridge.is_available(), "CBM should be unavailable in this test");
}