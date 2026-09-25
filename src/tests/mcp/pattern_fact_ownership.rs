// src/tests/mcp/pattern_fact_ownership.rs
//
// Production-path regression for method-local pattern-fact evidence.

#![cfg(feature = "typescript")]

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn fixture() -> String {
    let padding = " observable-ownership-economics-padding".repeat(300);
    format!(
        "import {{ of }} from 'rxjs';\n\
         export class Probe {{\n\
           methodB(): void {{}}\n\
           async methodA(): Promise<number> {{ return Promise.resolve(1); }}\n\
         }}\n\
         /*{padding} */\n"
    )
}

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn assert_plain_method_has_no_observable_fact(tool: &str, id: i64) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("probe.ts");
    std::fs::write(&path, fixture()).expect("fixture");
    let response = dispatch(
        &state(&root),
        id,
        tool,
        json!({
            "filePath": path.to_string_lossy(),
            "workspaceRoot": root.path().to_string_lossy(),
            "intent": "refactor",
            "fidelity": "high",
        }),
    );

    assert!(response.get("error").is_none(), "{response}");
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("model-visible text");
    assert!(
        text.starts_with("// SCHEMA v5"),
        "fixture must exercise the SCHEMA-v5 presentation, not raw passthrough:\n{text}"
    );
    let method = text
        .lines()
        .find(|line| line.trim_start().starts_with("M methodB"))
        .unwrap_or_else(|| panic!("methodB missing from presentation:\n{text}"));
    assert!(
        !method.contains("pf:OBSERVABLE"),
        "methodB has no Observable/RxJS usage and must not inherit another method's fact:\n{method}\n\n{text}"
    );
}

#[test]
fn provide_code_context_keeps_observable_facts_method_local() {
    assert_plain_method_has_no_observable_fact("provide_code_context", 1);
}

#[test]
fn delta_code_context_full_response_keeps_observable_facts_method_local() {
    assert_plain_method_has_no_observable_fact("delta_code_context", 2);
}
