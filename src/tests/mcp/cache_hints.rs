use super::*;
use crate::config::CleanCtxConfig;
use crate::mcp::McpState;

// ── Phase 1: CacheConfig tests ─────────────────────────────────────

/// Verify that CacheConfig::default() returns expected values.
#[test]
fn test_cache_config_defaults() {
    let config = crate::config::CacheConfig::default();
    assert!(config.enabled);
    assert_eq!(config.system_prompt_ttl, "1h");
    assert_eq!(config.tools_ttl, "1h");
    assert_eq!(config.baseline_ttl, "1h");
    assert_eq!(config.tail_ttl, "5m");
    assert_eq!(config.vocab_version, "v1");
    assert_eq!(config.tool_defs_version, "v1");
}

/// Verify that CacheConfig serializes and deserializes correctly.
#[test]
fn test_cache_config_serde() {
    let config = crate::config::CacheConfig::default();
    let json = serde_json::to_string(&config).expect("serialize");
    let deserialized: crate::config::CacheConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.system_prompt_ttl, deserialized.system_prompt_ttl);
    assert_eq!(config.tools_ttl, deserialized.tools_ttl);
    assert_eq!(config.baseline_ttl, deserialized.baseline_ttl);
    assert_eq!(config.tail_ttl, deserialized.tail_ttl);
    assert_eq!(config.vocab_version, deserialized.vocab_version);
    assert_eq!(config.tool_defs_version, deserialized.tool_defs_version);
}

// ── Phase 2: CacheMetrics tests ────────────────────────────────────

/// Verify that CacheMetrics::default() starts empty.
#[test]
fn test_cache_metrics_defaults() {
    let metrics = CacheMetrics::default();
    assert_eq!(metrics.hits, 0);
    assert_eq!(metrics.misses, 0);
    assert_eq!(metrics.tokens_saved, 0);
    assert!(metrics.breakpoints.is_empty());
}

// ── Phase 2: Breaker computation tests ─────────────────────────────

/// Verify that compute_baseline_breaker produces a stable bl_ prefixed hash.
#[test]
fn test_compute_baseline_breaker() {
    let text = "class Foo { }";
    let breaker = compute_baseline_breaker(text);
    assert!(breaker.starts_with("bl_"), "breaker should start with bl_");
    assert_eq!(breaker.len(), 67, "SHA-256 hex is 64 chars + bl_ prefix = 67");

    // Same input → same breaker
    let breaker2 = compute_baseline_breaker(text);
    assert_eq!(breaker, breaker2);

    // Different input → different breaker
    let breaker3 = compute_baseline_breaker("different content");
    assert_ne!(breaker, breaker3);
}

/// Verify that compute_workspace_breaker produces a stable ws_ prefixed hash.
#[test]
fn test_compute_workspace_breaker() {
    let hashes = vec!["abc".to_string(), "def".to_string()];
    let breaker = compute_workspace_breaker(&hashes);
    assert!(breaker.starts_with("ws_"), "breaker should start with ws_");

    // Same input → same breaker
    let breaker2 = compute_workspace_breaker(&hashes);
    assert_eq!(breaker, breaker2);

    // Different input → different breaker
    let hashes3 = vec!["xyz".to_string()];
    let breaker3 = compute_workspace_breaker(&hashes3);
    assert_ne!(breaker, breaker3);
}

// ── Phase 2: inject_cache_breakpoints tests ────────────────────────

/// Verify that inject_cache_breakpoints skips when cache is disabled.
#[test]
fn test_cache_disabled_skips_injection() {
    let mut config = CleanCtxConfig::default();
    config.cache.enabled = false; // disable cache
    let mut state = McpState::new(config);
    let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "result": {} });

    let saved = inject_cache_breakpoints(&mut response, &mut state, "baseline", "1h", "test-breaker", None);
    assert_eq!(saved, 0, "no tokens saved when cache disabled");
    assert_eq!(state.cache_metrics.hits, 0);
    assert_eq!(state.cache_metrics.misses, 0);
    // Should NOT have injected _meta.cache_hints
    assert!(response.get("_meta").is_none() || response["_meta"].get("cache_hints").is_none());
}

/// Verify that inject_cache_breakpoints correctly injects a system_prompt breakpoint.
#[test]
fn test_inject_system_prompt_hint() {
    let mut state = McpState::new(CleanCtxConfig::default());
    let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "result": {} });

    inject_cache_breakpoints(&mut response, &mut state, "system_prompt", "1h", "vocab-v1", None);

    let hints = &response["_meta"]["cache_hints"];
    let breakpoints = hints["breakpoints"].as_array().unwrap();
    assert_eq!(breakpoints.len(), 1);
    assert_eq!(breakpoints[0]["region"], "system_prompt");
    assert_eq!(breakpoints[0]["ttl"], "1h");
    assert_eq!(breakpoints[0]["breaker"], "vocab-v1");
    assert_eq!(state.cache_metrics.misses, 1, "first emission = miss");
}

/// Verify that inject_cache_breakpoints correctly injects a tools breakpoint.
#[test]
fn test_inject_tools_hint() {
    let mut state = crate::mcp::McpState::new(crate::config::CleanCtxConfig::default());
    let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "result": {} });

    inject_cache_breakpoints(&mut response, &mut state, "tools", "1h", "tools-v1", None);

    let hints = &response["_meta"]["cache_hints"];
    let breakpoints = hints["breakpoints"].as_array().unwrap();
    assert_eq!(breakpoints[0]["region"], "tools");
    assert_eq!(breakpoints[0]["ttl"], "1h");
    assert_eq!(breakpoints[0]["breaker"], "tools-v1");
}

/// Verify that inject_cache_breakpoints correctly injects a baseline breakpoint.
#[test]
fn test_inject_baseline_hint() {
    let mut state = crate::mcp::McpState::new(crate::config::CleanCtxConfig::default());
    let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "result": {} });

    let breaker = compute_baseline_breaker("compressed text here");
    inject_cache_breakpoints(&mut response, &mut state, "baseline", "1h", &breaker, None);

    let hints = &response["_meta"]["cache_hints"];
    let breakpoints = hints["breakpoints"].as_array().unwrap();
    assert_eq!(breakpoints[0]["region"], "baseline");
    assert_eq!(breakpoints[0]["ttl"], "1h");
    assert!(breakpoints[0]["breaker"].as_str().unwrap().starts_with("bl_"));
}

/// Verify that inject_cache_breakpoints correctly injects a tail breakpoint with "rolling" breaker.
#[test]
fn test_inject_tail_hint() {
    let mut state = crate::mcp::McpState::new(crate::config::CleanCtxConfig::default());
    let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "result": {} });

    inject_cache_breakpoints(&mut response, &mut state, "tail", "5m", "rolling", None);
    mark_tail_ephemeral(&mut state);

    let hints = &response["_meta"]["cache_hints"];
    let breakpoints = hints["breakpoints"].as_array().unwrap();
    assert_eq!(breakpoints[0]["region"], "tail");
    assert_eq!(breakpoints[0]["ttl"], "5m");
    assert_eq!(breakpoints[0]["breaker"], "rolling");
    // Verify tail is marked ephemeral
    assert_eq!(
        state.cache_metrics.breakpoints.get("tail").unwrap(),
        "ephemeral"
    );
}

/// Verify that the same region+breaker combo is not injected twice (dedup).
#[test]
fn test_emitted_dedup() {
    let mut state = crate::mcp::McpState::new(crate::config::CleanCtxConfig::default());
    let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "result": {} });

    // First call — should be a miss
    let saved1 = inject_cache_breakpoints(&mut response, &mut state, "tools", "1h", "tools-v1", None);
    assert_eq!(saved1, 0, "first call = miss, no tokens saved");
    assert_eq!(state.cache_metrics.misses, 1);
    assert_eq!(state.cache_metrics.hits, 0);

    // Same region+breaker combination — should be a hit (deduped)
    let saved2 = inject_cache_breakpoints(&mut response, &mut state, "tools", "1h", "tools-v1", None);
    assert!(saved2 > 0, "second call = hit, tokens should be saved");
    assert_eq!(state.cache_metrics.hits, 1);
    assert_eq!(state.cache_metrics.misses, 1);
}

/// Verify that cache metrics accumulate correctly across multiple calls.
#[test]
fn test_cache_metrics_accumulate() {
    let mut state = crate::mcp::McpState::new(crate::config::CleanCtxConfig::default());
    let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "result": {} });

    // Two different breakpoints
    inject_cache_breakpoints(&mut response, &mut state, "system_prompt", "1h", "vocab-v1", None);
    inject_cache_breakpoints(&mut response, &mut state, "tools", "1h", "tools-v1", None);

    assert_eq!(state.cache_metrics.misses, 2);
    assert_eq!(state.cache_metrics.hits, 0);
}

/// Verify that render_cache_text returns the expected format.
#[test]
fn test_cache_dashboard_text() {
    let mut metrics = CacheMetrics { hits: 12, misses: 3, tokens_saved: 18420, ..Default::default() };
    metrics.breakpoints.insert("tools".to_string(), "hit".to_string());

    // With hits+misses > 0, enabled=true returns Some with full stats
    let text = render_cache_text(&metrics, true).expect("should return text when active");
    assert!(text.contains("12"), "should show 12 hits");
    assert!(text.contains("3"), "should show 3 misses");
    assert!(text.contains("Prompt Cache (LLM"), "should contain LLM header");
    assert!(text.contains("enabled"), "should show enabled status");
    assert!(text.contains("LLM Tokens Saved"), "should show LLM token savings");

    // With hits+misses > 0, enabled=false still returns Some (shows disabled status)
    let text_disabled = render_cache_text(&metrics, false).expect("should return text with disabled status");
    assert!(text_disabled.contains("disabled"), "should show disabled status when cache is off");
}

/// Verify that render_cache_json returns the expected structured output.
#[test]
fn test_cache_dashboard_json() {
    let mut metrics = CacheMetrics { hits: 12, misses: 3, tokens_saved: 18420, ..Default::default() };
    metrics.breakpoints.insert("tools".to_string(), "hit".to_string());

    let json = render_cache_json(&metrics, true);
    assert_eq!(json["hits"], 12);
    assert_eq!(json["misses"], 3);
    assert!(json["hit_rate"].as_f64().unwrap() > 0.75);
    assert_eq!(json["llm_tokens_saved"], 18420);
    assert_eq!(json["breakpoints"]["tools"], "hit");
    assert_eq!(json["enabled"], true, "enabled should be true");

    // M-1 regression: disabled JSON should have enabled=false
    let json_disabled = render_cache_json(&metrics, false);
    assert_eq!(json_disabled["enabled"], false, "enabled should be false when cache is off");
}

/// Verify that generate_vocabulary_text returns expected content.
#[test]
fn test_generate_vocabulary_text() {
    let text = generate_vocabulary_text();
    assert!(text.contains("Clean-CTX Opcode/Marker Vocabulary"), "should have header");
    assert!(text.contains("$c   → class"), "should include $c opcode");
    assert!(text.contains("Φcmp"), "should include Angular component marker");
    assert!(text.contains("⊕guard"), "should include guard marker");
}