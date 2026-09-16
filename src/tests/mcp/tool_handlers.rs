// src/tests/mcp/tool_handlers.rs
//
// Tests for tool handler helper functions and handler robustness.
// Note: The handlers themselves call send_response (stdout), so we
// test the pure helper functions and verify handlers don't panic.
//
// Phase 6 IR-first audit: Added integration tests verifying response
// format compliance (content[0].text = LLM text, "ir" = hierarchical,
// "pretty" = fallback), cache invalidation, and angular/spring
// meta-layer abbreviation in compiled output.

use crate::compression::Fidelity;
use crate::mcp::tool_handlers::core::{
    contract_fields, contract_fields_focused, handle_compress_code_context,
};
use crate::mcp::tools::{dispatch_tools_call, parse_fidelity_arg, resolve_fidelity};
use serde_json::json;

// ── resolve_fidelity tests ──
// Signature: resolve_fidelity(explicit: Option<&str>, ext: Option<&str>, config: &CleanCtxConfig) -> Fidelity

#[test]
fn resolve_fidelity_explicit_low() {
    let result = resolve_fidelity(Some("low"), None, &crate::config::CleanCtxConfig::default());
    assert_eq!(result, Fidelity::Low);
}

#[test]
fn resolve_fidelity_explicit_medium() {
    let result = resolve_fidelity(
        Some("medium"),
        None,
        &crate::config::CleanCtxConfig::default(),
    );
    assert_eq!(result, Fidelity::Medium);
}

#[test]
fn resolve_fidelity_explicit_high() {
    let result = resolve_fidelity(
        Some("high"),
        None,
        &crate::config::CleanCtxConfig::default(),
    );
    assert_eq!(result, Fidelity::High);
}

#[test]
fn resolve_fidelity_none_uses_default() {
    let result = resolve_fidelity(None, None, &crate::config::CleanCtxConfig::default());
    // Default fidelity is "low" per config
    assert_eq!(result, Fidelity::Low);
}

#[test]
fn resolve_fidelity_invalid_string_falls_back_to_default() {
    let result = resolve_fidelity(
        Some("bogus"),
        None,
        &crate::config::CleanCtxConfig::default(),
    );
    assert_eq!(result, Fidelity::Low);
}

#[test]
fn resolve_fidelity_extension_override() {
    let mut config = crate::config::CleanCtxConfig::default();
    config
        .fidelity_overrides
        .insert("ts".to_string(), crate::compression::Fidelity::High);
    let result = resolve_fidelity(None, Some("ts"), &config);
    assert_eq!(result, Fidelity::High);
}

// ── parse_fidelity_arg tests ──

#[test]
fn parse_fidelity_arg_with_explicit_value() {
    let params = json!({ "arguments": { "fidelity": "high" } });
    let config = crate::config::CleanCtxConfig::default();
    let result = parse_fidelity_arg(&json!(1), &params, &config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Fidelity::High);
}

#[test]
fn parse_fidelity_arg_missing_defaults() {
    let params = json!({ "arguments": {} });
    let config = crate::config::CleanCtxConfig::default();
    let result = parse_fidelity_arg(&json!(1), &params, &config);
    assert!(result.is_ok());
}

#[test]
fn parse_fidelity_arg_invalid_returns_error() {
    let params = json!({ "arguments": { "fidelity": "turbo" } });
    let config = crate::config::CleanCtxConfig::default();
    let result = parse_fidelity_arg(&json!(1), &params, &config);
    assert!(result.is_err());
}

// ── Handler smoke tests (verify no panic) ──

#[test]
fn handle_context_stats_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    dispatch_tools_call(&id, "context_stats", &params, &state);
}

#[test]
fn handle_list_sessions_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    dispatch_tools_call(&id, "list_sessions", &params, &state);
}

#[test]
fn handle_context_history_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    dispatch_tools_call(&id, "context_history", &params, &state);
}

#[test]
fn handle_save_context_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent.ts" } });
    // Should not panic (returns error for nonexistent file, but doesn't panic)
    dispatch_tools_call(&id, "save_context", &params, &state);
}

#[test]
fn handle_restore_context_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent.ts" } });
    // Should not panic
    dispatch_tools_call(&id, "restore_context", &params, &state);
}

#[test]
fn handle_purge_old_deltas_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "days": 30 } });
    // Should not panic
    dispatch_tools_call(&id, "purge_old_deltas", &params, &state);
}

// ── IR-first integration tests ──

#[test]
fn handle_compress_code_context_with_fallback() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    // Use nonexistent file — should fall back to text pipeline gracefully
    let params = json!({ "arguments": { "filePath": "/nonexistent/file.ts", "fidelity": "low" } });
    // Should not panic
    dispatch_tools_call(&id, "compress_code_context", &params, &state);
}

#[test]
fn handle_delta_code_context_no_baseline() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent/file.ts", "fidelity": "low" } });
    // Should not panic — stores baseline IR, returns "no baseline" message
    dispatch_tools_call(&id, "delta_code_context", &params, &state);
}

// ── Cache breakpoint injection regression tests ─────────────────────
// These tests guard against regressions in the cache breakpoint wiring
// added during the smart-cache FAANG audit.

#[test]
fn inject_baseline_breakpoint_helper_injects_hint() {
    use crate::mcp::tool_helpers::inject_baseline_breakpoint;
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "content": [{ "type": "text", "text": "compressed output" }] }
    });

    inject_baseline_breakpoint(&mut response, &state, "compressed output");

    assert!(
        response.get("_meta").is_none(),
        "_meta should NOT be at response root"
    );
    let hints = &response["result"]["_meta"]["cache_hints"];
    let breakpoints = hints["breakpoints"].as_array().unwrap();
    assert_eq!(breakpoints.len(), 1);
    assert_eq!(breakpoints[0]["region"], "baseline");
    assert_eq!(breakpoints[0]["ttl"], "1h");
    assert!(
        breakpoints[0]["breaker"]
            .as_str()
            .unwrap()
            .starts_with("bl_")
    );
    assert_eq!(state.cache_metrics_lock().misses, 1);
}

#[test]
fn inject_tail_breakpoint_helper_injects_hint() {
    use crate::mcp::tool_helpers::inject_tail_breakpoint;
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "content": [{ "type": "text", "text": "delta output" }] }
    });

    inject_tail_breakpoint(&mut response, &state);

    assert!(
        response.get("_meta").is_none(),
        "_meta should NOT be at response root"
    );
    let hints = &response["result"]["_meta"]["cache_hints"];
    let breakpoints = hints["breakpoints"].as_array().unwrap();
    assert_eq!(breakpoints.len(), 1);
    assert_eq!(breakpoints[0]["region"], "tail");
    assert_eq!(breakpoints[0]["ttl"], "5m");
    assert_eq!(breakpoints[0]["breaker"], "rolling");
    assert_eq!(
        state.cache_metrics_lock().breakpoints.get("tail").unwrap(),
        "ephemeral"
    );
}

#[test]
fn inject_baseline_breakpoint_skips_when_cache_disabled() {
    use crate::mcp::tool_helpers::inject_baseline_breakpoint;
    let mut config = crate::tests::test_config();
    config.cache.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "content": [{ "type": "text", "text": "compressed output" }] }
    });

    inject_baseline_breakpoint(&mut response, &state, "compressed output");

    assert!(response["result"].get("_meta").is_none());
    assert_eq!(state.cache_metrics_lock().misses, 0);
}

#[test]
fn inject_tail_breakpoint_skips_when_cache_disabled() {
    use crate::mcp::tool_helpers::inject_tail_breakpoint;
    let mut config = crate::tests::test_config();
    config.cache.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "content": [{ "type": "text", "text": "delta output" }] }
    });

    inject_tail_breakpoint(&mut response, &state);

    assert!(response["result"].get("_meta").is_none());
    assert_eq!(state.cache_metrics_lock().misses, 0);
}

// REGRESSION: The cached-IR fast path in `delta_code_context` must not panic.
#[test]
fn delta_code_context_cached_ir_path_does_not_panic() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    dispatch_tools_call(&id, "delta_code_context", &params, &state);
    dispatch_tools_call(&id, "delta_code_context", &params, &state);
}

#[test]
fn handle_apply_delta_no_baseline() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "delta": { "file": "α1", "from": 1, "to": 2, "ops": { "+": [], "-": [], "~": [] } } } });
    // Should not panic — returns "UnknownFile" error
    dispatch_tools_call(&id, "apply_delta", &params, &state);
}

#[path = "tool_handlers_render.rs"]
mod tool_handlers_render;

#[path = "tool_handlers_contract.rs"]
mod tool_handlers_contract;

#[path = "tool_handlers_economics.rs"]
mod tool_handlers_economics;
