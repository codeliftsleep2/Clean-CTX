use crate::mcp::tools::{dispatch_tools_call, tool_list};
use serde_json::{Value, json};

fn dispatch(arguments: Value) -> Value {
    let _serial = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(135),
        "provide_code_context",
        &json!({ "arguments": arguments }),
        &crate::mcp::McpState::new(crate::tests::test_config()),
    );
    crate::protocol::captured_responses().pop().unwrap()
}

fn assert_batch_fidelity_rejected(arguments: Value) {
    let response = dispatch(arguments);
    assert_eq!(response["error"]["code"], -32602, "{response}");
    let message = response["error"]["message"].as_str().unwrap();
    assert!(message.contains("Top-level 'fidelity'"), "{response}");
    assert!(message.contains("each file item"), "{response}");
}

#[test]
fn batch_top_level_fidelity_is_rejected_in_every_response_mode() {
    for mode in [None, Some("mirrored"), Some("structured"), Some("indexed")] {
        let mut arguments = json!({
            "files": [{ "id": "one", "filePath": "one.ts" }],
            "fidelity": "verbatim"
        });
        if let Some(mode) = mode {
            arguments["responseMode"] = mode.into();
        }
        assert_batch_fidelity_rejected(arguments);
    }
}

#[test]
fn conflicting_parent_and_item_fidelity_is_rejected_without_precedence() {
    assert_batch_fidelity_rejected(json!({
        "files": [{ "id": "one", "filePath": "one.ts", "fidelity": "high" }],
        "fidelity": "verbatim"
    }));
}

#[test]
fn schema_forbids_batch_top_level_fidelity_but_keeps_item_and_single_fidelity() {
    let provide = tool_list()
        .into_iter()
        .find(|tool| tool["name"] == "provide_code_context")
        .unwrap();
    let branches = provide["inputSchema"]["oneOf"].as_array().unwrap();
    let batch = branches
        .iter()
        .find(|branch| {
            branch["required"]
                .as_array()
                .unwrap()
                .contains(&json!("files"))
        })
        .unwrap();
    assert!(
        batch["not"]["anyOf"]
            .as_array()
            .unwrap()
            .iter()
            .any(|guard| {
                guard["required"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("fidelity"))
            })
    );
    assert!(batch["properties"]["files"]["items"]["properties"]["fidelity"].is_object());
    assert!(provide["inputSchema"]["properties"]["fidelity"].is_object());
}

#[test]
fn unrelated_unknown_batch_parameter_keeps_existing_runtime_behavior() {
    let response = dispatch(json!({
        "files": [{ "id": "one", "filePath": "missing.ts", "fidelity": "verbatim" }],
        "unknownParameter": true
    }));
    assert!(response.get("result").is_some(), "{response}");
}
