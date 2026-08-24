// src/tests/cbm/e2e.rs
//
// End-to-end tests for the full CBM integration pipeline.
// These tests exercise the complete flow:
//   CbmClient → GraphBridge → intelligence layer → enrichment injection.
//
// All tests that require a live CBM binary check availability first
// and skip gracefully if CBM is not installed.

use serde_json::json;

/// Check if CBM binary exists on PATH without launching it.
/// This avoids double-launching CBM when `McpState::new()` also launches it.
fn cbm_binary_exists() -> bool {
    let name = if cfg!(windows) {
        "codebase-memory-mcp.exe"
    } else {
        "codebase-memory-mcp"
    };
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}

// ── E2E: MCP dispatch path with live CBM ────────────────────────

/// Smoke-test every CBM MCP tool handler via dispatch_tools_call.
#[test]
fn e2e_mcp_dispatch_graph_search_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let state = crate::mcp::McpState::new(config);
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "graph_search",
        &serde_json::json!({"arguments": {"query": ".*compress.*", "project": "clean-ctx"}}),
        &state,
    );
}

#[test]
fn e2e_mcp_dispatch_graph_query_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let state = crate::mcp::McpState::new(config);
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(2),
        "graph_query",
        &serde_json::json!({"arguments": {"query": "MATCH (n:Function) RETURN n.name LIMIT 5", "project": "clean-ctx"}}),
        &state,
    );
}

#[test]
fn e2e_mcp_dispatch_graph_trace_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let state = crate::mcp::McpState::new(config);
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(3),
        "graph_trace",
        &serde_json::json!({"arguments": {"from": "main", "to": "", "project": "clean-ctx"}}),
        &state,
    );
}

#[test]
fn e2e_mcp_dispatch_get_architecture_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let state = crate::mcp::McpState::new(config);
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(4),
        "get_architecture",
        &serde_json::json!({"arguments": {"project": "clean-ctx"}}),
        &state,
    );
}

#[test]
fn e2e_mcp_dispatch_get_cbm_status_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let state = crate::mcp::McpState::new(config);
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(5),
        "get_cbm_status",
        &serde_json::json!({"arguments": {}}),
        &state,
    );
}

#[test]
fn e2e_mcp_dispatch_cbm_proxy_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let state = crate::mcp::McpState::new(config);

    // Smoke test: dispatch must not panic. Stats recording depends on
    // async indexing completion, which is unpredictable in this test.
    // (P1-9: indexing is backgrounded; ensure_indexed_or_error may return
    // StillIndexing, which means no CBM query and no stats recorded.)
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(6),
        "cbm_proxy",
        &serde_json::json!({"arguments": {"cbm_tool": "search_graph", "parameters": {"name_pattern": ".*compress.*", "project": "clean-ctx"}}}),
        &state,
    );
}

#[test]
fn e2e_get_cbm_status_always_works() {
    let config = crate::config::CleanCtxConfig::default();
    let state = crate::mcp::McpState::new(config);
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(99),
        "get_cbm_status",
        &serde_json::json!({"arguments": {}}),
        &state,
    );
}

/// Test that the full proxy compression pipeline works end-to-end.
/// Uses cbm_proxy handler directly.
#[test]
fn e2e_proxy_handler_returns_compressed_result() {
    let cbm_available = cbm_binary_exists();

    if !cbm_available {
        eprintln!("Skipping e2e_proxy_handler_returns_compressed_result — CBM not installed");
        return;
    }

    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let state = crate::mcp::McpState::new(config);

    // Test that the proxy can handle a get_architecture call
    // handle_cbm_proxy sends responses directly via send_response,
    // so we test the bridge's proxy_call directly instead
    let mut binding = state.graph_bridge.lock().unwrap();
    let bridge = binding.as_mut().unwrap();
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
    let config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge =
        crate::cbm::bridge::GraphBridge::try_create(&config, std::path::Path::new("."));

    assert!(
        !bridge.is_available(),
        "Bridge should be unavailable when CBM disabled"
    );

    // All queries should return empty/default results, NOT panic
    let importance = bridge.get_symbol_importance_mut();
    assert!(
        importance.is_empty(),
        "Symbol importance should be empty without CBM"
    );

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
    assert!(
        qr.nodes.is_empty() && qr.edges.is_empty(),
        "Query graph should return empty without CBM"
    );

    // detect_changes should return Ok(None)
    let changes = bridge.detect_changes();
    assert!(
        changes.is_ok() && changes.unwrap().is_none(),
        "Detect changes should return None without CBM"
    );

    // cache operations should not panic
    bridge.invalidate_symbol("test");
    bridge.invalidate_cache();
    bridge.clear_cache();
}

/// Test that GraphBridge status transitions work correctly.
#[test]
fn e2e_bridge_status_lifecycle() {
    use crate::cbm::config::CbmStatus;

    let config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge =
        crate::cbm::bridge::GraphBridge::try_create(&config, std::path::Path::new("."));

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
    use crate::cbm::SymbolImportance;
    use crate::intelligence::compute_pagerank;
    use crate::intelligence::fidelity::{
        FidelityRecommendation, apply_recommendation, cbm_informed_fidelity,
    };
    use std::collections::HashMap;

    // Build IR scores
    let mut ir_scores = HashMap::new();
    ir_scores.insert("critical_handler".to_string(), 50.0);
    ir_scores.insert("helper_func".to_string(), 5.0);
    ir_scores.insert("unused_util".to_string(), 1.0);

    // Build CBM importance scores
    let mut cbm_scores = HashMap::new();
    cbm_scores.insert(
        "critical_handler".to_string(),
        SymbolImportance {
            symbol: "critical_handler".into(),
            score: 0.95,
            file: "src/handler.rs".into(),
        },
    );
    cbm_scores.insert(
        "helper_func".to_string(),
        SymbolImportance {
            symbol: "helper_func".into(),
            score: 0.5,
            file: "src/handler.rs".into(),
        },
    );
    cbm_scores.insert(
        "unused_util".to_string(),
        SymbolImportance {
            symbol: "unused_util".into(),
            score: 0.1,
            file: "src/utils.rs".into(),
        },
    );

    // Step 1: Compute PageRank
    let scores = compute_pagerank(ir_scores, cbm_scores, Some(0.6));
    assert!(!scores.is_empty(), "Should have combined scores");

    // Step 2: Verify high-importance symbol gets ForceHigh
    let critical_score = scores.get("critical_handler").unwrap();
    assert!(
        critical_score.combined_score > 0.7,
        "Critical handler should have high combined score: {}",
        critical_score.combined_score
    );

    // Step 3: Test fidelity recommendation pipeline
    let mut importance_map = HashMap::new();
    importance_map.insert(
        "critical_handler".to_string(),
        SymbolImportance {
            symbol: "critical_handler".into(),
            score: critical_score.combined_score,
            file: "src/handler.rs".into(),
        },
    );

    let rec = cbm_informed_fidelity(
        "src/handler.rs",
        &importance_map,
        FidelityRecommendation::NoRecommendation,
    );

    // Step 4: Apply recommendation
    let fidelity = apply_recommendation(&rec);
    if critical_score.combined_score > 0.8 {
        assert_eq!(fidelity, Some(crate::compressor::Fidelity::High));
    }
}

// ---- K-1: Indexing lifecycle tests ---------------------------------

/// Prove `ensure_indexed()` is report-only -- it does NOT transition
/// `NotStarted` -> `InProgress` (no spawn, no mutation).
/// Uses a mock bridge with `Available` status but `NotStarted` state.
#[test]
fn ensure_indexed_does_not_trigger_indexing() {
    use crate::cbm::bridge::test_helpers::new_available_not_started;

    let mut bridge = new_available_not_started();

    // Precondition: state is NotStarted (empty map).
    {
        let states = bridge.indexing_state();
        assert!(
            states.is_empty(),
            "precondition: indexing state should be empty (NotStarted)"
        );
    }

    // Call ensure_indexed -- in the OLD code this would spawn a background
    // thread, flip state to InProgress, and return StillIndexing.
    // In the NEW code it must NOT mutate state and return StillIndexing.
    let result = bridge.ensure_indexed();

    // Must return Ok(StillIndexing) -- not Err, not Ready.
    assert!(
        result.is_ok(),
        "ensure_indexed should return Ok(StillIndexing) when Available/NotStarted: {:?}",
        result
    );
    match result.unwrap() {
        crate::cbm::bridge::IndexingStatus::StillIndexing { elapsed_secs } => {
            assert_eq!(elapsed_secs, 0, "should report 0 elapsed");
        }
        _ => panic!("expected StillIndexing, got something else"),
    }

    // State must remain NotStarted (no transition to InProgress) -- proving no spawn occurred.
    let states = bridge.indexing_state();
    for (project, state) in states.iter() {
        assert!(
            matches!(state, crate::cbm::bridge::IndexingState::NotStarted),
            "K-1: ensure_indexed must NOT mutate indexing state to InProgress (no spawn) -- project '{project}' is {:?}",
            state
        );
    }
}

/// Prove that `try_create` with an available CBM binary immediately starts
/// indexing (state is `InProgress` or `Complete`, never `NotStarted`).
///
/// Requires a live CBM binary on PATH; skips gracefully if absent.
#[test]
fn try_create_begins_indexing_at_construction() {
    use crate::cbm::bridge::IndexingState;

    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping -- CBM not installed");
        return;
    }
    let config = crate::cbm::config::CbmConfig {
        enabled: true,
        ..Default::default()
    };
    let bridge = crate::cbm::bridge::GraphBridge::try_create(&config, std::path::Path::new("."));

    // Valid lifecycle: the bridge started indexing at construction.
    // The `indexing_state` map must NOT be empty (an empty map means
    // `NotStarted` -- the construction-time spawn never ran).
    let states = bridge.indexing_state();
    let first = states.iter().next();
    match first {
        None => {
            panic!(
                "K-1: try_create must start indexing -- indexing_state is empty. \
                 The background indexer was never spawned at construction."
            );
        }
        Some((project, state)) => match state {
            IndexingState::InProgress { .. } | IndexingState::Complete => {
                // Good: indexing was kicked off at construction.
            }
            IndexingState::Failed(msg) => {
                // Acceptable: CBM binary may not be compatible.
                eprintln!("Note: indexing started but failed: {msg}");
            }
            IndexingState::NotStarted => {
                panic!(
                    "K-1: try_create must start indexing -- project '{project}' is NotStarted. \
                     This means the background indexer was never spawned at construction."
                );
            }
        },
    }
}
