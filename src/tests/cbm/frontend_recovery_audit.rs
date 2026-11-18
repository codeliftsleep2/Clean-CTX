use super::*;

#[test]
fn cbm_a4_audit_failed_replay_preserves_original_timeout_and_later_failure() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-timeout-then-eof");
    let error = call(&mut client, &root, "a4-timeout-then-eof", "query_graph").unwrap_err();
    assert!(
        matches!(error, CbmError::Timeout(_)),
        "original timeout erased by replay error: {error}"
    );
    let status = format!("{:?}", client.status());
    assert!(
        status.contains("original") && status.contains("replay"),
        "failure chain missing: {status}"
    );
    assert_eq!(launches(&root), 2);
    assert_eq!(tool_count(&root, "query_graph"), 2);
}

#[test]
fn cbm_a4_audit_structured_rpc_cause_survives_retry_transport_loss() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-rpc-then-eof");
    let error = call(&mut client, &root, "a4-rpc-then-eof", "query_graph").unwrap_err();
    assert!(
        matches!(&error, CbmError::RpcError { code:-32603, message } if message == "uncertain accepted operation"),
        "structured original cause erased: {error}"
    );
    let status = format!("{:?}", client.status());
    assert!(status.contains("uncertain accepted operation") && status.contains("replay"));
    assert_eq!(tool_count(&root, "query_graph"), 2);
    assert_eq!(
        launches(&root),
        1,
        "retry and replacement exceeded one-retry budget"
    );
}

#[test]
fn cbm_a4_audit_baseline_terminal_half_open_constructs_one_new_epoch() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-eof-always");
    client.circuit_cooldown_secs = 30;
    assert!(call(&mut client, &root, "a4-eof-always", "query_graph").is_err());
    assert_eq!(launches(&root), 2);
    assert_eq!(fixture::owned_resources(&client), 0);
    client.degraded_since = Some(Instant::now() - Duration::from_secs(31));
    assert!(call(&mut client, &root, "normal", "query_graph").is_ok());
    assert_eq!(launches(&root), 3);
    assert!(client.status().is_available());
}

#[test]
fn cbm_a4_audit_mutation_recovery_does_not_clear_open_circuit_status() {
    for tool in ["index_repository", "future_tool"] {
        let root = TempDir::new().unwrap();
        let mut client = client(&root, "a4-eof-once");
        client.max_consecutive_failures = 1;
        client.circuit_cooldown_secs = 30;
        assert!(call(&mut client, &root, "a4-eof-once", tool).is_err());
        assert_eq!(launches(&root), 2);
        assert_eq!(tool_count(&root, tool), 1);
        assert_eq!(client.consecutive_failures, 1);
        assert!(
            !client.status().is_available(),
            "frontend admission falsely cleared failed-operation circuit status"
        );
        assert!(call(&mut client, &root, "a4-eof-once", "query_graph").is_err());
        assert_eq!(launches(&root), 2);
    }
}

#[test]
fn cbm_a4_audit_diagnostic_degradation_below_threshold_does_not_block_mcp_dispatch() {
    let _serial = crate::protocol::handler_response_serial();
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-rpc");
    assert!(call(&mut client, &root, "a4-rpc", "query_graph").is_err());
    assert_eq!(client.consecutive_failures, 2);
    assert!(client.circuit_allows());
    let config = crate::cbm::CbmConfig {
        enabled: false,
        cache_db_path: Some(root.path().join("bridge.db").to_string_lossy().into()),
        ..Default::default()
    };
    let mut bridge = crate::cbm::GraphBridge::try_create(&config, root.path());
    bridge.indexing_state.lock().unwrap().insert(
        bridge.project.clone().unwrap(),
        crate::cbm::bridge::IndexingState::Complete,
    );
    *bridge.client.lock().unwrap() = Some(client);
    bridge.update_status();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    *state.graph_bridge_lock() = Some(bridge);
    crate::mcp::tools::setup_handler_registry_for_tests();
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(905),
        "graph_query",
        &json!({"arguments":{"query":"MATCH (n) RETURN n"}}),
        &state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert_eq!(
        tool_count(&root, "query_graph"),
        4,
        "diagnostic status became permanent MCP lockout below configured threshold"
    );
    assert_eq!(response["result"]["isError"], true);
    assert!(
        response["result"]["structuredContent"]["error"]
            .as_str()
            .unwrap()
            .contains("uncertain accepted operation")
    );
    assert_eq!(launches(&root), 1);
}

#[test]
fn cbm_a4_audit_detect_changes_unproven_scope_is_not_replayed() {
    for scope in ["tests", "future-scope"] {
        let root = TempDir::new().unwrap();
        let mut client = client(&root, "a4-rpc");
        let result = client.call_tool(
            "detect_changes",
            json!({"project":"fixture", "scope":scope}),
        );
        assert!(result.is_err());
        assert_eq!(
            tool_count(&root, "detect_changes"),
            1,
            "tool-name allowlist replayed an unproven argument-dependent workflow: {scope}"
        );
        assert_eq!(launches(&root), 1);
    }
}
