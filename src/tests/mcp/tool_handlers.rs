// src/tests/mcp/tool_handlers.rs
//
// Tests for tool handler helper functions and handler robustness.
// Note: The handlers themselves call send_response (stdout), so we
// test the pure helper functions and verify handlers don't panic.

use crate::mcp::tools::{parse_fidelity_arg, resolve_fidelity};
use crate::compression::Fidelity;
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
    let result = resolve_fidelity(Some("medium"), None, &crate::config::CleanCtxConfig::default());
    assert_eq!(result, Fidelity::Medium);
}

#[test]
fn resolve_fidelity_explicit_high() {
    let result = resolve_fidelity(Some("high"), None, &crate::config::CleanCtxConfig::default());
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
    let result = resolve_fidelity(Some("bogus"), None, &crate::config::CleanCtxConfig::default());
    assert_eq!(result, Fidelity::Low);
}

#[test]
fn resolve_fidelity_extension_override() {
    let mut config = crate::config::CleanCtxConfig::default();
    config.fidelity_overrides.insert("ts".to_string(), "high".to_string());
    let result = resolve_fidelity(None, Some("ts"), &config);
    assert_eq!(result, Fidelity::High);
}

// ── parse_fidelity_arg tests ──

#[test]
fn parse_fidelity_arg_with_explicit_value() {
    let params = json!({ "arguments": { "fidelity": "high" } });
    let result = parse_fidelity_arg(&json!(1), &params);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Fidelity::High);
}

#[test]
fn parse_fidelity_arg_missing_defaults() {
    let params = json!({ "arguments": {} });
    let result = parse_fidelity_arg(&json!(1), &params);
    assert!(result.is_ok());
}

#[test]
fn parse_fidelity_arg_invalid_returns_error() {
    let params = json!({ "arguments": { "fidelity": "turbo" } });
    let result = parse_fidelity_arg(&json!(1), &params);
    assert!(result.is_err());
}

// ── Handler smoke tests (verify no panic) ──

#[test]
fn handle_context_stats_smoke() {
    use crate::mcp::tool_handlers::handle_context_stats;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    handle_context_stats(&id, &params, &mut state);
}

#[test]
fn handle_list_sessions_smoke() {
    use crate::mcp::tool_handlers::handle_list_sessions;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    handle_list_sessions(&id, &params, &mut state);
}

#[test]
fn handle_context_history_smoke() {
    use crate::mcp::tool_handlers::handle_context_history;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    handle_context_history(&id, &params, &mut state);
}

#[test]
fn handle_save_context_smoke() {
    use crate::mcp::tool_handlers::handle_save_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent.ts" } });
    // Should not panic (returns error for nonexistent file, but doesn't panic)
    handle_save_context(&id, &params, &mut state);
}

#[test]
fn handle_restore_context_smoke() {
    use crate::mcp::tool_handlers::handle_restore_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent.ts" } });
    // Should not panic
    handle_restore_context(&id, &params, &mut state);
}

#[test]
fn handle_purge_old_deltas_smoke() {
    use crate::mcp::tool_handlers::handle_purge_old_deltas;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": { "days": 30 } });
    // Should not panic
    handle_purge_old_deltas(&id, &params, &mut state);
}