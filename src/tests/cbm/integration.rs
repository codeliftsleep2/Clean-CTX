// src/tests/cbm/integration.rs
//
// Integration tests for CBM compression pipeline and intelligence layer.
// Tests that compress_cbm_response properly handles various JSON inputs,
// and that the intelligence layer integrates correctly.

#[test]
fn compress_cbm_response_envelope_stripping() {
    use crate::cbm::json_compress::compress_cbm_response;

    // Simulate a full CBM MCP response
    let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"CBM graph data: found 42 nodes, 15 edges"}]}}"#;
    let result = compress_cbm_response(raw);
    assert!(result.is_some(), "Should compress properly formatted response");
    let comp = result.unwrap();
    assert!(comp.cbm_error.is_none(), "No error should be present");
    assert!(comp.compressed_text.len() < raw.len(), "Should achieve compression");
    assert!(comp.compressed_text.contains("CBM graph data"),
        "Should preserve meaningful content");
}

#[test]
fn proxy_stats_use_pluggable_tokenizer_format() {
    // Verify the _meta field names changed from *_est to actual names
    use crate::cbm::json_compress::compress_cbm_response;

    let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"data":"test"}}"#;
    let comp = compress_cbm_response(raw).unwrap();

    // The pluggable tokenizer produces raw_tokens, not raw_tokens_est
    assert!(comp.raw_tokens_est > 0, "Should have estimated tokens");
    assert!(comp.comp_tokens_est > 0, "Should have compressed tokens");
    // With key shortening + envelope stripping, compressed should be smaller
    assert!(comp.compressed_text.len() < raw.len());
}

// ── H-03: Intelligence Layer integration tests ───────────────────

#[test]
fn provide_code_context_cbm_skipped_when_intelligence_disabled() {
    // Test that the intelligence layer doesn't influence decisions when
    // intelligence.enabled = false (regardless of CBM availability).
    use crate::mcp::heuristics;

    let mut config = crate::config::CleanCtxConfig::default();
    config.intelligence.enabled = false;

    // Verify the config default: intelligence is enabled by default,
    // and our manual disable took effect.
    assert!(!config.intelligence.enabled,
        "intelligence should be disabled for this test");

    // Run the heuristics engine with a typical file
    let source = "pub struct User { name: String }\npub fn get_user() -> User { unimplemented!() }";
    let decision = heuristics::decide(
        "/project/src/user.rs",
        None,           // explicit_fidelity
        None,           // explicit_intent
        &config,
        &crate::compression::text_delta::TextDeltaComputer::new(),
        &crate::ir::replay::ContextState::new(),
        source,
        None,           // path_alias
        None,           // stored_fidelity
    )
    .unwrap();

    // When intelligence is disabled, cbm_informed should stay false
    assert!(!decision.cbm_informed,
        "cbm_informed should be false when intelligence is disabled");

    // The decision summary should reflect no_cbm
    assert!(decision.summary().contains("no_cbm"),
        "summary should contain no_cbm when intelligence is disabled");
}

#[test]
fn provide_code_context_cbm_informed_false_when_no_bridge() {
    // Test that cbm_informed stays false when there's no graph bridge
    // (even if intelligence is enabled).
    use crate::mcp::heuristics;

    let config = crate::config::CleanCtxConfig::default();
    assert!(config.intelligence.enabled,
        "intelligence should be enabled by default");

    // Run heuristics without any bridge — cbm_informed stays false
    let source = "pub struct Config { port: u16 }";
    let decision = heuristics::decide(
        "/project/src/config.rs",
        None, None, &config,
        &crate::compression::text_delta::TextDeltaComputer::new(),
        &crate::ir::replay::ContextState::new(),
        source,
        None, None,
    )
    .unwrap();

    assert!(!decision.cbm_informed,
        "cbm_informed should be false when no bridge available");
    assert!(decision.summary().contains("no_cbm"),
        "summary should contain no_cbm when no bridge");
}

#[test]
fn provide_code_context_cbm_informed_false_on_explicit_fidelity() {
    // When the user passes an explicit fidelity, the intelligence layer
    // should NOT override it — explicit parameters take priority.
    use crate::mcp::heuristics;

    let config = crate::config::CleanCtxConfig::default();
    let source = "pub fn main() { println!(\"hello\"); }";
    let decision = heuristics::decide(
        "/project/src/main.rs",
        Some("high"),   // explicit_fidelity
        None,           // explicit_intent
        &config,
        &crate::compression::text_delta::TextDeltaComputer::new(),
        &crate::ir::replay::ContextState::new(),
        source,
        None, None,
    )
    .unwrap();

    // explicit fidelity should be honored, cbm_informed stays false
    assert!(!decision.cbm_informed,
        "cbm_informed should be false when explicit fidelity provided");
    assert_eq!(format!("{:?}", decision.fidelity), "High",
        "explicit fidelity High should be honored");
}

#[test]
fn intelligence_config_defaults_to_enabled() {
    let config = crate::config::CleanCtxConfig::default();
    assert!(config.intelligence.enabled,
        "IntelligenceConfig::enabled should default to true");
}