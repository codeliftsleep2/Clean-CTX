// CBM handler contracts for calls made while the graph bridge is unavailable.
//
// These tests share the captured-response sink with the other handler suites,
// so every test holds the repository's response serial guard.

#[test]
fn graph_search_returns_is_error_when_cbm_unavailable() {
    // Serialize access to the shared CAPTURED_RESPONSES sink with the
    // Phase A/B retirement suites (must hold HANDLER_RESPONSE_SERIAL).
    let _serial = crate::protocol::handler_response_serial();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    crate::mcp::tools::setup_handler_registry_for_tests();

    crate::protocol::captured_responses().clear();

    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "graph_search",
        &serde_json::json!( {"arguments": {"query": "GraphBridge"}}),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");

    // Must be a JSON-RPC error (no bridge = unavailable)
    assert!(
        response.get("error").is_some(),
        "CBM-unavailable path must produce JSON-RPC error, got: {response}"
    );
    let err = response["error"].as_object().expect("error object");
    assert_eq!(err["code"], -32603);
    let message = err["message"].as_str().expect("error message");
    assert!(
        message.contains("host-native text/file search")
            && message.contains("workspace_query")
            && message.contains("find_entities")
            && message.contains("provide_code_context"),
        "the first failed graph call must prescribe portable direct fallbacks: {message}"
    );
    assert!(
        !message.contains("search_codebase"),
        "the server must not advertise a host-specific tool as an MCP tool: {message}"
    );
    assert!(
        !message.contains("get_cbm_status"),
        "an unavailable graph call must not trigger a redundant status probe: {message}"
    );
}

#[test]
fn graph_query_returns_jsonrpc_error_when_cbm_unavailable() {
    let _serial = crate::protocol::handler_response_serial();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    crate::mcp::tools::setup_handler_registry_for_tests();
    crate::protocol::captured_responses().clear();

    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "graph_query",
        &serde_json::json!( {"arguments": {"query": "MATCH (c:Class) RETURN c"}}),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");
    assert!(
        response.get("error").is_some(),
        "CBM-unavailable path must produce JSON-RPC error, got: {response}"
    );
    assert_eq!(response["error"]["code"], -32603);
}

#[test]
fn graph_trace_returns_jsonrpc_error_when_cbm_unavailable() {
    let _serial = crate::protocol::handler_response_serial();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    crate::mcp::tools::setup_handler_registry_for_tests();
    crate::protocol::captured_responses().clear();

    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "graph_trace",
        &serde_json::json!( {"arguments": {"from": "A", "to": "B"}}),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");
    assert!(
        response.get("error").is_some(),
        "CBM-unavailable path must produce JSON-RPC error, got: {response}"
    );
    assert_eq!(response["error"]["code"], -32603);
}

#[test]
fn get_architecture_returns_jsonrpc_error_when_cbm_unavailable() {
    let _serial = crate::protocol::handler_response_serial();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    crate::mcp::tools::setup_handler_registry_for_tests();
    crate::protocol::captured_responses().clear();

    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "get_architecture",
        &serde_json::json!( {"arguments": {}}),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");
    assert!(
        response.get("error").is_some(),
        "CBM-unavailable path must produce JSON-RPC error, got: {response}"
    );
    assert_eq!(response["error"]["code"], -32603);
}
