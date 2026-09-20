use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn take_response() -> Value {
    crate::protocol::captured_responses()
        .pop()
        .expect("handler response")
}

#[cfg(feature = "typescript")]
#[test]
fn production_delta_then_apply_uses_corrected_sequence_protocol() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::TempDir::new().expect("temporary workspace");
    let file = root.path().join("sequence.ts");
    std::fs::write(
        &file,
        "class Worker {\n  first(): void {}\n  third(): void {}\n}\n",
    )
    .expect("baseline source");

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let request = || {
        json!({
            "arguments": {
                "filePath": file,
                "fidelity": "low",
                "workspaceRoot": root.path(),
            }
        })
    };

    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "delta_code_context", &request(), &state);
    let baseline = take_response();
    assert!(baseline.get("error").is_none(), "{baseline}");
    assert!(
        baseline.pointer("/result/delta").is_none(),
        "first delta request must establish the production baseline: {baseline}"
    );

    std::fs::write(
        &file,
        "class Worker {\n  first(): void {}\n  second(): void {}\n  third(): void {}\n}\n",
    )
    .expect("changed source");
    state.invalidate_source_cache(&file.to_string_lossy());

    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(2), "delta_code_context", &request(), &state);
    let delta_response = take_response();
    assert!(delta_response.get("error").is_none(), "{delta_response}");
    let delta = delta_response
        .pointer("/result/delta")
        .cloned()
        .unwrap_or_else(|| panic!("production delta payload: {delta_response}"));
    assert_eq!(delta["dv"], 2);
    assert!(
        delta["edits"]
            .as_array()
            .is_some_and(|edits| !edits.is_empty()),
        "{delta}"
    );
    let from = delta["from"].clone();
    let to = delta["to"].clone();

    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(3),
        "apply_delta",
        &json!({
            "arguments": {
                "delta": delta,
                "currentVersion": from,
            }
        }),
        &state,
    );
    let applied = take_response();
    assert!(applied.get("error").is_none(), "{applied}");
    assert_eq!(applied.pointer("/result/_meta/version"), Some(&to));
    let text = applied
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .expect("rendered replay result");
    assert!(text.contains("second"), "{text}");
}
