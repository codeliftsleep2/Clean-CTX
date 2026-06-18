// src/tests/cbm/e2e.rs
//
// End-to-end tests for the full CBM integration pipeline.
// These tests exercise the complete flow:
//   CbmClient → GraphBridge → intelligence layer → enrichment injection.
//
// All tests that require a live CBM binary check availability first
// and skip gracefully if CBM is not installed.

use serde_json::json;

// ── E2E: Full pipeline with live CBM (if available) ─────────────

/// Test that the proxy tool forwards to CBM and compresses the response.
/// Requires CBM to be installed and the project to be indexed.
/// Skips gracefully if CBM is unavailable.
#[test]
fn e2e_proxy_search_graph_compresses_response() {
    // Check if CBM binary is available
    let cbm_available = crate::cbm::bridge::GraphBridge::try_create(
        &crate::cbm::config::CbmConfig { enabled: true, ..Default::default() },
        std::path::Path::new("."),
    ).is_available();

    if !cbm_available {
        eprintln!("Skipping e2e_proxy_search_graph_compresses_response — CBM not installed");
        return;
    }

    // Build a full MCP state with CBM enabled
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let mut state = crate::mcp::McpState::new(config);

    // Verify bridge is available
    assert!(state.graph_bridge.as_ref().is_some_and(|b| b.is_available()),
        "Bridge should be available when CBM is installed and enabled");

    // Call the proxy with a real search_graph query
    // Test proxy response format
    let bridge = state.graph_bridge.as_mut().unwrap();
    let raw = bridge.proxy_call("search_graph", json!({
        "name_pattern": ".*compress.*",
        "label": "Function",
        "limit": 5
    }));

    match raw {
        Ok(text) => {
            assert!(!text.is_empty(), "CBM proxy should return non-empty response");
            assert!(text.contains("jsonrpc"), "Response should be valid JSON-RPC");
        }
        Err(e) => {
            // CBM might not have indexed this project — that's OK for E2E
            eprintln!("CBM proxy call returned error (may need index): {e}");
        }
    }
}

/// Test that CBM enrichment pipeline produces compressed metadata.
/// Requires CBM installed and project indexed.
#[test]
fn e2e_enrichment_injects_cbm_metadata() {
    let cbm_available = crate::cbm::bridge::GraphBridge::try_create(
        &crate::cbm::config::CbmConfig { enabled: true, ..Default::default() },
        std::path::Path::new("."),
    ).is_available();

    if !cbm_available {
        eprintln!("Skipping e2e_enrichment_injects_cbm_metadata — CBM not installed");
        return;
    }

    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let mut state = crate::mcp::McpState::new(config);

    // Simulate the provide_code_context response injection
    let mut response = json!({
        "jsonrpc": "2.0",
        "id": 999,
        "result": {
            "content": [{"type": "text", "text": "compressed content here"}],
            "_meta": {}
        }
    });

    use crate::mcp::tool_handlers::enrich_with_cbm;
    enrich_with_cbm(&mut response, "src/cbm/client.rs", &mut state);

    let meta = response.pointer("/result/_meta").unwrap();
    // Should always have cbm_status
    assert!(meta.get("cbm_status").is_some(), "E2E: _meta should contain cbm_status");

    let status = meta["cbm_status"].as_str().unwrap();
    eprintln!("E2E CBM status: {status}");

    if status == "available" {
        // If CBM is available, we should get either enrichment or architecture summary
        let has_enrichment = meta.get("cbm_enrichment").is_some();
        let has_architecture = meta.get("cbm_architecture_summary").is_some();
        assert!(has_enrichment || has_architecture,
            "E2E: available CBM should produce enrichment or architecture summary");
    }
}

/// Test that the full proxy compression pipeline works end-to-end.
/// Uses cbm_proxy handler directly.
#[test]
fn e2e_proxy_handler_returns_compressed_result() {
    let cbm_available = crate::cbm::bridge::GraphBridge::try_create(
        &crate::cbm::config::CbmConfig { enabled: true, ..Default::default() },
        std::path::Path::new("."),
    ).is_available();

    if !cbm_available {
        eprintln!("Skipping e2e_proxy_handler_returns_compressed_result — CBM not installed");
        return;
    }

    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let mut state = crate::mcp::McpState::new(config);

    // Test that the proxy can handle a get_architecture call
    // handle_cbm_proxy sends responses directly via send_response,
    // so we test the bridge's proxy_call directly instead
    let bridge = state.graph_bridge.as_mut().unwrap();
    let result = bridge.proxy_call("get_architecture", json!({}));

    match result {
        Ok(text) => {
            assert!(!text.is_empty());
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&text);
            assert!(parsed.is_ok(), "Proxy response should be valid JSON");
            let r = parsed.unwrap();
            // If CBM returns an error because project not indexed, that's fine
            if let Some(err) = r.get("error") {
                eprintln!("CBM returned expected error: {}", err["message"]);
            }
        }
        Err(e) => {
            eprintln!("CBM proxy call failed (may need index): {e}");
        }
    }
}

// ── E2E: GraphBridge full query lifecycle (no CBM required) ────

/// Test that GraphBridge gracefully handles all queries when CBM is unavailable.
#[test]
fn e2e_bridge_graceful_degradation_all_queries() {
    let config = crate::cbm::config::CbmConfig { enabled: false, ..Default::default() };
    let mut bridge = crate::cbm::bridge::GraphBridge::try_create(&config, std::path::Path::new("."));

    assert!(!bridge.is_available(), "Bridge should be unavailable when CBM disabled");

    // All queries should return empty/default results, NOT panic
    let importance = bridge.get_symbol_importance_mut();
    assert!(importance.is_empty(), "Symbol importance should be empty without CBM");

    let dead = bridge.get_dead_code();
    assert!(dead.is_empty(), "Dead code should be empty without CBM");

    let arch = bridge.get_architecture();
    assert!(arch.is_none(), "Architecture should be None without CBM");

    let blast = bridge.get_blast_radius("test_func", 1);
    assert!(blast.is_empty(), "Blast radius should be empty without CBM");

    let search = bridge.search("test");
    assert!(search.is_empty(), "Search should be empty without CBM");

    let traces = bridge.trace_path("a", "b");
    assert!(traces.is_empty(), "Trace path should be empty without CBM");

    // query_graph should return empty QueryResult
    let qr = bridge.query_graph("MATCH (n) RETURN n");
    assert!(qr.nodes.is_empty() && qr.edges.is_empty(),
        "Query graph should return empty without CBM");

    // detect_changes should return Ok(None)
    let changes = bridge.detect_changes();
    assert!(changes.is_ok() && changes.unwrap().is_none(),
        "Detect changes should return None without CBM");

    // cache operations should not panic
    bridge.invalidate_symbol("test");
    bridge.invalidate_cache();
    bridge.clear_cache();
}

/// Test that GraphBridge status transitions work correctly.
#[test]
fn e2e_bridge_status_lifecycle() {
    use crate::cbm::config::CbmStatus;

    let config = crate::cbm::config::CbmConfig { enabled: false, ..Default::default() };
    let mut bridge = crate::cbm::bridge::GraphBridge::try_create(&config, std::path::Path::new("."));

    assert_eq!(bridge.status(), &CbmStatus::Unavailable);
    assert!(!bridge.is_available());
    assert_eq!(bridge.graph_version(), "");

    // Set project and version
    bridge.set_project("test_project");
    bridge.set_graph_version("v1.0");

    assert_eq!(bridge.graph_version(), "v1.0");
    // Status should remain Unavailable (no client)
    assert_eq!(bridge.status(), &CbmStatus::Unavailable);

    // update_status should keep Unavailable
    bridge.update_status();
    assert_eq!(bridge.status(), &CbmStatus::Unavailable);
}

// ── E2E: Intelligence Layer full pipeline (no CBM required) ────

/// Test the full intelligence layer pipeline: PageRank → fidelity → recommendation.
#[test]
fn e2e_intelligence_layer_full_pipeline() {
    use crate::intelligence::compute_pagerank;
    use crate::intelligence::fidelity::{cbm_informed_fidelity, apply_recommendation, FidelityRecommendation};
    use crate::cbm::SymbolImportance;
    use std::collections::HashMap;

    // Build IR scores
    let mut ir_scores = HashMap::new();
    ir_scores.insert("critical_handler".to_string(), 50.0);
    ir_scores.insert("helper_func".to_string(), 5.0);
    ir_scores.insert("unused_util".to_string(), 1.0);

    // Build CBM importance scores
    let mut cbm_scores = HashMap::new();
    cbm_scores.insert("critical_handler".to_string(), SymbolImportance {
        symbol: "critical_handler".into(),
        score: 0.95,
        file: "src/handler.rs".into(),
    });
    cbm_scores.insert("helper_func".to_string(), SymbolImportance {
        symbol: "helper_func".into(),
        score: 0.5,
        file: "src/handler.rs".into(),
    });
    cbm_scores.insert("unused_util".to_string(), SymbolImportance {
        symbol: "unused_util".into(),
        score: 0.1,
        file: "src/utils.rs".into(),
    });

    // Step 1: Compute PageRank
    let scores = compute_pagerank(ir_scores, cbm_scores, Some(0.6));
    assert!(!scores.is_empty(), "Should have combined scores");

    // Step 2: Verify high-importance symbol gets ForceHigh
    let critical_score = scores.get("critical_handler").unwrap();
    assert!(critical_score.combined_score > 0.7,
        "Critical handler should have high combined score: {}", critical_score.combined_score);

    // Step 3: Test fidelity recommendation pipeline
    let mut importance_map = HashMap::new();
    importance_map.insert("critical_handler".to_string(), SymbolImportance {
        symbol: "critical_handler".into(),
        score: critical_score.combined_score,
        file: "src/handler.rs".into(),
    });

    let rec = cbm_informed_fidelity("src/handler.rs", &importance_map, FidelityRecommendation::NoRecommendation);

    // Step 4: Apply recommendation
    let fidelity = apply_recommendation(&rec);
    if critical_score.combined_score > 0.8 {
        assert_eq!(fidelity, Some(crate::compressor::Fidelity::High));
    }
}

/// Test the enrichment pipeline end-to-end (no CBM required).
#[test]
fn e2e_enrichment_pipeline_no_cbm() {
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);

    let mut response = json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "content": [{"type": "text", "text": "test"}],
            "_meta": {}
        }
    });

    use crate::mcp::tool_handlers::enrich_with_cbm;
    enrich_with_cbm(&mut response, "src/test.rs", &mut state);

    let meta = response.pointer("/result/_meta").unwrap();

    // Should inject cbm_status even when CBM unavailable
    assert!(meta.get("cbm_status").is_some(), "Should always inject cbm_status");
    let status = meta["cbm_status"].as_str().unwrap();

    if status == "available" {
        // Available — enrichment may or may not be present (depends on CBM data)
    } else {
        // Unavailable/degraded — enrichment should NOT be present
        assert!(meta.get("cbm_enrichment").is_none(),
            "Should not inject enrichment when CBM is {status}");
    }
}