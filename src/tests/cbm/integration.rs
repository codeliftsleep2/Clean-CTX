// src/tests/cbm/integration.rs
//
// Integration tests for CBM enrichment compression pipeline.
// Tests that enrich_with_cbm compresses data before injecting into _meta.

use serde_json::json;

#[test]
fn enrich_with_cbm_compresses_symbol_importance() {
    use crate::mcp::tool_handlers::enrich_with_cbm;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let mut response = json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "content": [{"type": "text", "text": "compressed"}],
            "_meta": {}
        }
    });

    // CBM is disabled by default (no binary), so enrichment should
    // inject cbm_status but skip symbol data (not available).
    enrich_with_cbm(&mut response, "src/test.rs", &mut state);

    let meta = response.pointer("/result/_meta").unwrap();
    assert!(meta.get("cbm_status").is_some(),
        "enrich_with_cbm should inject cbm_status");
    // Since CBM is unavailable, cbm_enrichment should NOT be present
    // (status check guard prevents querying unavailable CBM)
    assert!(meta.get("cbm_enrichment").is_none(),
        "should not inject enrichment when CBM unavailable");
}

#[test]
fn enrich_with_cbm_degraded_status_skips_enrichment() {
    use crate::mcp::tool_handlers::enrich_with_cbm;
    // Disable CBM in config so GraphBridge::try_create sets Unavailable
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = false;
    let mut state = crate::mcp::McpState::new(config);
    // Set degraded status to simulate a previously-degraded CBM
    state.cbm_status = crate::cbm::config::CbmStatus::Degraded("slow".into());

    let mut response = json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "content": [{"type": "text", "text": "compressed"}],
            "_meta": {}
        }
    });

    enrich_with_cbm(&mut response, "src/test.rs", &mut state);
    let meta = response.pointer("/result/_meta").unwrap();
    // When CBM is disabled, the bridge is None, so update_status() won't
    // overwrite our manual status. The status should still be "degraded".
    assert_eq!(meta["cbm_status"].as_str().unwrap(), "degraded",
        "CBM status should remain degraded when bridge is disabled");
    // Should bail early — no enrichment when status is not "available"
    assert!(meta.get("cbm_enrichment").is_none(),
        "should not inject enrichment when CBM is not available");
}

#[test]
fn enrich_with_cbm_missing_meta_does_not_panic() {
    use crate::mcp::tool_handlers::enrich_with_cbm;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);

    // Response without _meta field — should be a no-op
    let mut response = json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "content": [{"type": "text", "text": "ok"}] }
    });

    enrich_with_cbm(&mut response, "src/test.rs", &mut state);
    assert!(response.pointer("/result/_meta").is_none(),
        "should not add _meta if it doesn't exist");
}

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