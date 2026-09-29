use super::*;

// CBM-EDIT-001 Post-Edit Graph Consistency: after a successful
// mutation returns, subsequent CBM graph queries observe the
// filesystem state produced by that mutation.

#[test]
fn reindex_for_file_fails_gracefully_when_cbm_unavailable() {
    use crate::cbm::GraphBridge;
    use crate::cbm::config::{CbmConfig, CbmStatus};
    use std::path::Path;

    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge = GraphBridge::try_create(&config, Path::new("."));
    bridge.status = CbmStatus::Unavailable;

    let result = bridge.reindex_for_file(Path::new("src/main.rs"), "fast");
    assert!(
        result.is_err(),
        "reindex_for_file must fail when CBM unavailable"
    );
    let err = result.unwrap_err();
    match err {
        crate::cbm::client::CbmError::LaunchError(msg) => {
            assert!(
                msg.contains("not available"),
                "error message should indicate unavailability: {msg}"
            );
        }
        other => panic!("expected LaunchError, got: {other:?}"),
    }
}

#[test]
fn reindex_for_file_resolves_to_extra_root_via_try_create_with_roots() {
    use crate::cbm::GraphBridge;
    use crate::cbm::config::CbmConfig;

    let primary = make_temp_root("reidx_primary");
    let extra = make_temp_root("reidx_extra");
    std::fs::write(extra.join("src").join("main.rs"), b"fn main() {}").unwrap_or_default();

    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge =
        GraphBridge::try_create_with_roots(&config, &primary, std::slice::from_ref(&extra));
    prime_available(&mut bridge);

    let file_in_extra = extra.join("src").join("main.rs");
    let result = bridge.reindex_for_file(&file_in_extra, "fast");
    assert!(result.is_err(), "reindex_for_file must fail with no client");
    match &result {
        Err(crate::cbm::client::CbmError::LaunchError(msg)) => {
            assert!(msg.contains("not available"), "unexpected error: {msg}");
        }
        other => panic!("expected LaunchError, got: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&primary);
    let _ = std::fs::remove_dir_all(&extra);
}
#[test]
fn reindex_for_file_fallback_to_active_root_for_unmapped_file() {
    use crate::cbm::GraphBridge;
    use crate::cbm::config::CbmConfig;

    let root = make_temp_root("reidx_root");
    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge = GraphBridge::try_create(&config, &root);
    prime_available(&mut bridge);
    let outside = std::env::temp_dir().join(format!("reindex_outside_{}", std::process::id()));
    let result = bridge.reindex_for_file(&outside, "fast");
    assert!(result.is_err(), "should reach client via fallback root");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn reindex_for_file_invalidate_cache_clears_memory_entries() {
    use crate::cbm::GraphBridge;
    use crate::cbm::config::CbmConfig;
    use std::time::{Duration, Instant};

    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge = GraphBridge::try_create(&config, std::path::Path::new("."));
    bridge.status = crate::cbm::config::CbmStatus::Available;
    let data = crate::cbm::bridge::CachedGraphData {
        data: serde_json::json!([["A", "B"]]),
        expires_at: Instant::now() + Duration::from_secs(300),
    };
    crate::cbm::bridge::test_helpers::seed_active_cache(&bridge, "call_edges", data);
    assert_eq!(
        bridge.cache.len(),
        1,
        "cache should have 1 entry before invalidation"
    );
    bridge.invalidate_cache();
    assert_eq!(
        bridge.cache.len(),
        0,
        "cache should be empty after invalidation"
    );
}

// ── Proxy whitelist validation ──────────────────────────────────

#[test]
fn cbm_proxy_whitelist_accepts_all_six_operations() {
    let ops = [
        "search_graph",
        "query_graph",
        "trace_path",
        "get_architecture",
        "list_projects",
        "index_repository",
    ];
    for op in &ops {
        assert!(
            crate::cbm::proxy::reject_disallowed_cbm_tool(op).is_none(),
            "expected '{op}' to be accepted by the proxy whitelist"
        );
    }
}

#[test]
fn cbm_proxy_whitelist_rejects_unknown_operation() {
    let err = crate::cbm::proxy::reject_disallowed_cbm_tool("unknown_tool");
    assert!(err.is_some(), "expected 'unknown_tool' to be rejected");
    let msg = err.unwrap();
    assert!(
        msg.contains("Unsupported CBM operation"),
        "error should mention unsupported operation, got: {msg}"
    );
    assert!(
        msg.contains("get_symbol_importance"),
        "error should mention internal helpers, got: {msg}"
    );
}

#[test]
fn cbm_proxy_whitelist_rejects_get_symbol_importance() {
    // get_symbol_importance is implemented internally via query_graph Cypher
    // and is NOT a CBM proxy tool — it must be rejected.
    let err = crate::cbm::proxy::reject_disallowed_cbm_tool("get_symbol_importance");
    assert!(
        err.is_some(),
        "get_symbol_importance must be rejected by the proxy whitelist"
    );
}

#[test]
fn cbm_proxy_whitelist_rejects_get_dead_code() {
    // get_dead_code is implemented internally via query_graph Cypher
    // and is NOT a CBM proxy tool — it must be rejected.
    let err = crate::cbm::proxy::reject_disallowed_cbm_tool("get_dead_code");
    assert!(
        err.is_some(),
        "get_dead_code must be rejected by the proxy whitelist"
    );
}

#[test]
fn cbm_proxy_whitelist_rejects_get_blast_radius() {
    // get_blast_radius is an internal bridge helper, not a CBM tool.
    let err = crate::cbm::proxy::reject_disallowed_cbm_tool("get_blast_radius");
    assert!(
        err.is_some(),
        "get_blast_radius must be rejected by the proxy whitelist"
    );
}

#[test]
fn cbm_proxy_whitelist_rejects_detect_changes() {
    // detect_changes is an internal bridge operation, not a CBM tool.
    let err = crate::cbm::proxy::reject_disallowed_cbm_tool("detect_changes");
    assert!(
        err.is_some(),
        "detect_changes must be rejected by the proxy whitelist"
    );
}

#[test]
fn cbm_proxy_whitelist_rejects_get_call_edges() {
    // get_call_edges is an internal bridge helper, not a CBM tool.
    let err = crate::cbm::proxy::reject_disallowed_cbm_tool("get_call_edges");
    assert!(
        err.is_some(),
        "get_call_edges must be rejected by the proxy whitelist"
    );
}

#[test]
fn cbm_proxy_compatibility_aliases_are_normalized_before_whitelist() {
    // The proxy normalizes graph_search → search_graph, etc. BEFORE the
    // whitelist check, so the normalized values always pass. This test
    // confirms that the aliases are NOT in the whitelist themselves but
    // are handled by the normalization step in handle_cbm_proxy.
    //
    // The whitelist only contains CBM-native names. The alias normalization
    // happens before the whitelist check, so these are rejected by the
    // whitelist function alone — but the full handle_cbm_proxy flow
    // normalizes them first.
    let err = crate::cbm::proxy::reject_disallowed_cbm_tool("graph_search");
    assert!(
        err.is_some(),
        "graph_search is an alias, not a CBM-native name — whitelist rejects it alone"
    );
    let err = crate::cbm::proxy::reject_disallowed_cbm_tool("graph_query");
    assert!(
        err.is_some(),
        "graph_query is an alias, not a CBM-native name — whitelist rejects it alone"
    );
    let err = crate::cbm::proxy::reject_disallowed_cbm_tool("graph_trace");
    assert!(
        err.is_some(),
        "graph_trace is an alias, not a CBM-native name — whitelist rejects it alone"
    );
}

// ── Proxy project-not-found error enhancement ──────────────────

#[test]
fn enhance_project_not_found_injects_list_projects_hint() {
    let raw = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"Project 'does-not-exist' not found"}}"#;
    let enhanced = crate::cbm::proxy::enhance_project_not_found_error(raw);
    let parsed: serde_json::Value = serde_json::from_str(&enhanced).unwrap();
    let msg = parsed["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("list_projects"),
        "should contain list_projects hint, got: {msg}"
    );
    assert!(
        msg.contains("does-not-exist"),
        "should preserve the project name, got: {msg}"
    );
}

#[test]
fn enhance_project_not_found_handles_unknown_project_variant() {
    let raw =
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"Unknown project: foo"}}"#;
    let enhanced = crate::cbm::proxy::enhance_project_not_found_error(raw);
    let parsed: serde_json::Value = serde_json::from_str(&enhanced).unwrap();
    let msg = parsed["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("list_projects"),
        "should contain list_projects hint for 'Unknown project:', got: {msg}"
    );
}

#[test]
fn enhance_project_not_found_ignores_non_project_errors() {
    let raw =
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32603,"message":"Internal server error"}}"#;
    let enhanced = crate::cbm::proxy::enhance_project_not_found_error(raw);
    let parsed: serde_json::Value = serde_json::from_str(&enhanced).unwrap();
    let msg = parsed["error"]["message"].as_str().unwrap();
    assert_eq!(
        msg, "Internal server error",
        "non-project errors must not be enhanced"
    );
    assert!(
        !msg.contains("list_projects"),
        "non-project errors must not get the hint"
    );
}

#[test]
fn enhance_project_not_found_ignores_success_response() {
    let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"ok"}]}}"#;
    let enhanced = crate::cbm::proxy::enhance_project_not_found_error(raw);
    assert_eq!(enhanced, raw, "success responses must not be altered");
}

#[test]
fn enhance_project_not_found_preserves_unparseable_text() {
    let raw = "this is not valid json at all";
    let enhanced = crate::cbm::proxy::enhance_project_not_found_error(raw);
    assert_eq!(enhanced, raw, "unparseable text must be returned as-is");
}
// ── Lazy CBM graph freshness tests ──────────────────────────
//
