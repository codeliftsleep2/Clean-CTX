//! Real dispatcher boundary with the established local fake frontend.
use serde_json::{Value, json};
use std::io::{self, Write};
pub(super) fn fixture_tool(mode: &str, request: &Value) -> bool {
    if mode != "b4-soft" && mode != "b4-success" {
        return false;
    }
    if mode == "b4-success" {
        println!(
            "{}",
            json!({"jsonrpc":"2.0","id":request["id"],"result":{"content":[{"type":"text","text":"{\"has_more\":false,\"next_cursor\":null}"}]}})
        );
        io::stdout().flush().unwrap();
        return true;
    }
    println!(
        "{}",
        json!({"jsonrpc":"2.0","id":request["id"],"result":{
        "isError":true,"content":[{"type":"text","text":"job failed"}],
        "structuredContent":{"serving_index_preserved":true,"generation":null,"code":"resource_limit"}}})
    );
    io::stdout().flush().unwrap();
    true
}
#[test]
fn cbm_proxy_truth_dispatcher_soft_error_preserves_additional_evidence() {
    super::b2_tests::with_bridge("b4-soft", |bridge| {
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        let state = crate::mcp::McpState::new(config.clone());
        bridge.indexing_state.lock().unwrap().insert(
            "test-project".into(),
            crate::cbm::bridge::IndexingState::Complete,
        );
        let replacement = crate::cbm::GraphBridge::try_create(&config.cbm, &bridge.project_root);
        *state.graph_bridge_lock() = Some(std::mem::replace(bridge, replacement));
        crate::protocol::captured_responses().clear();
        crate::mcp::tool_dispatch::dispatch_tools_call(
            &json!(441),
            "cbm_proxy",
            &json!({"arguments":{
            "cbm_tool":"query_graph","parameters":{"project":"test-project","query":"query"}}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert_eq!(response["result"]["isError"], true);
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("serving_index_preserved"), "{text}");
        assert!(text.contains("resource_limit"));
    });
}

#[test]
fn cbm_proxy_truth_dispatcher_reports_current_bridge_status() {
    super::b2_tests::with_bridge("b4-success", |bridge| {
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        let state = crate::mcp::McpState::new(config.clone());
        bridge.indexing_state.lock().unwrap().insert(
            "test-project".into(),
            crate::cbm::bridge::IndexingState::Complete,
        );
        let replacement = crate::cbm::GraphBridge::try_create(&config.cbm, &bridge.project_root);
        *state.graph_bridge_lock() = Some(std::mem::replace(bridge, replacement));
        crate::protocol::captured_responses().clear();
        crate::mcp::tool_dispatch::dispatch_tools_call(
            &json!(442),
            "cbm_proxy",
            &json!({"arguments":{
            "cbm_tool":"query_graph","parameters":{"project":"test-project","query":"query"}}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert_eq!(response["result"]["_meta"]["cbm_status"], "available");
    });
}
