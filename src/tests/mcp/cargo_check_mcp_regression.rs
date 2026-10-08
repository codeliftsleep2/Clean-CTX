use crate::mcp::{tool_dispatch, tools};

#[test]
fn cargo_check_mcp_regression_closed_tool_schema() {
    let catalog = tools::tool_list();
    let tool = catalog
        .iter()
        .find(|tool| tool["name"] == "cargo_check")
        .expect("the typed CargoCheck tool must be registered");
    assert_eq!(tool["inputSchema"]["type"], "object");
    assert_eq!(tool["inputSchema"]["properties"], serde_json::json!({}));
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
}

#[test]
fn cargo_check_mcp_regression_has_dispatch_owner() {
    assert!(
        tool_dispatch::inline_tool_names().contains("cargo_check"),
        "the advertised native tool needs a dispatch owner"
    );
}

fn dispatch(arguments: serde_json::Value) -> serde_json::Value {
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = false;
    config.persistence.enabled = false;
    let state = crate::mcp::McpState::new(config);
    crate::protocol::captured_responses().clear();
    tool_dispatch::dispatch_tools_call(
        &serde_json::json!("cargo-check-request"),
        "cargo_check",
        &serde_json::json!({"name":"cargo_check", "arguments":arguments}),
        &state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("one response")
}

#[test]
fn cargo_check_mcp_regression_model_cannot_supply_execution_authority() {
    let response = dispatch(serde_json::json!({"workspaceRoot":"PRIVATE_ARGUMENT_CANARY"}));
    assert_eq!(response["error"]["code"], -32602);
    assert!(!response.to_string().contains("PRIVATE_ARGUMENT_CANARY"));
}

#[test]
fn cargo_check_mcp_regression_unconfigured_session_explains_unavailability() {
    let response = dispatch(serde_json::json!({}));
    assert_eq!(response["result"]["isError"], true);
    assert_eq!(
        response["result"]["structuredContent"]["failure"],
        "workspace_authority_missing"
    );
}
