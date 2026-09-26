//! Contract regression for the model-facing provider boundary.
//!
//! `provide_code_context` must return a complete current representation even
//! when an IR baseline exists and the compatibility `auto_delta` flag is true.
//! Structured delta transport is opt-in through `delta_code_context`.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn dispatch(state: &crate::mcp::McpState, id: i64, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(id),
        "provide_code_context",
        &json!({ "arguments": arguments }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

#[test]
fn changed_follow_up_provide_remains_complete_when_auto_delta_is_enabled() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("complete-provider.ts");
    let file = path.to_string_lossy().into_owned();
    let mut config = crate::tests::test_config();
    config.auto_delta = true;
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let args = || {
        json!({
            "filePath": file.clone(),
            "workspaceRoot": root.path().to_string_lossy()
        })
    };
    let source = |method: &str| {
        format!(
            "export class CompleteProvider {{ {method}(): number {{ return 1; }} }}\n{}",
            "// economics-padding-0123456789abcdef\n".repeat(1_000)
        )
    };

    std::fs::write(&path, source("before")).expect("baseline source");
    let baseline = dispatch(&state, 1, args());
    assert_eq!(baseline["result"]["_meta"]["strategy"], "full");

    std::fs::write(&path, source("after")).expect("changed source");
    state.invalidate_source_cache(&file);
    let follow_up = dispatch(&state, 2, args());
    assert_eq!(follow_up["result"]["_meta"]["strategy"], "full");
    assert!(follow_up["result"]["_meta"].get("delta").is_none());
    assert!(
        follow_up["result"]["content"][0]["text"]
            .as_str()
            .expect("complete model-visible content")
            .contains("M after"),
        "changed declaration must be present: {follow_up}"
    );
}
