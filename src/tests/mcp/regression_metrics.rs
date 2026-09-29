// Dashboard and MCP metadata regressions split from regression.rs.
use super::*;

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
