//! Regression coverage for strict `workspace_query` name validation.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn dispatch(state: &crate::mcp::McpState, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({ "arguments": arguments }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered workspace_query response")
}

fn assert_name_type_error(error: &Value) {
    assert_eq!(error["code"], -32602, "{error}");
    let message = error["message"].as_str().expect("error message");
    assert!(message.contains("name"), "{error}");
    assert!(message.contains("string"), "{error}");
    assert!(message.contains("queries"), "{error}");
}

#[test]
fn red_single_name_queries_reject_array_input_with_actionable_type_error() {
    let _serial = crate::protocol::handler_response_serial();
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    for query_type in [
        "find_entities",
        "forward_edges",
        "reverse_edges",
        "transitive_dependencies",
    ] {
        let response = dispatch(
            &state,
            json!({
                "type": query_type,
                "name": ["MethodA", "MethodB", "MethodC"],
            }),
        );
        assert_name_type_error(&response["error"]);
        assert!(response.get("result").is_none(), "{response}");
    }
}

#[test]
fn red_batch_name_type_failure_is_isolated_to_the_invalid_item() {
    let _serial = crate::protocol::handler_response_serial();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let response = dispatch(
        &state,
        json!({
            "queries": [
                { "id": "invalid", "type": "reverse_edges", "name": ["A", "B"] },
                { "id": "valid", "type": "has_cycle" },
            ],
        }),
    );

    assert!(response.get("error").is_none(), "{response}");
    let results = response["result"]["structuredContent"]["results"]
        .as_array()
        .expect("batch results");
    assert_eq!(results[0]["status"], "error", "{response}");
    assert_name_type_error(&results[0]["error"]);
    assert_eq!(results[1]["status"], "ok", "{response}");
}
