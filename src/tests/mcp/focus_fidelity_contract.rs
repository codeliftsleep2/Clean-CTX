#![cfg(feature = "typescript")]

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn dispatch(state: &crate::mcp::McpState, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({ "arguments": arguments }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn fixture() -> (tempfile::TempDir, std::path::PathBuf, crate::mcp::McpState) {
    let root = tempfile::tempdir().expect("temporary workspace");
    let path = root.path().join("focus.ts");
    let mut source = r#"export class FocusProbe {
  selected(): number { return 41 + 1; }
  ignored(): number { return 7; }
}
"#
    .to_string();
    source.push_str(&"// focus-economics-padding-0123456789abcdef\n".repeat(1_000));
    std::fs::write(&path, source).expect("write fixture");

    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    (root, path, state)
}

fn base_arguments(root: &tempfile::TempDir, path: &std::path::Path) -> Value {
    json!({
        "filePath": path.to_string_lossy(),
        "workspaceRoot": root.path().to_string_lossy()
    })
}

#[test]
fn non_empty_focus_without_fidelity_or_intent_implies_edit() {
    let (root, path, state) = fixture();
    let mut arguments = base_arguments(&root, &path);
    arguments["focusMethods"] = json!(["FocusProbe.selected"]);

    let response = dispatch(&state, arguments);

    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"]["_meta"]["fidelity"], "edit");
    assert_eq!(
        response["result"]["_meta"]["content_kind"],
        "skeleton_with_focused_verbatim_bodies"
    );
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("model text");
    assert!(text.contains("return 41 + 1;"), "{text}");
    assert!(!text.contains("return 7;"), "{text}");
}

#[test]
fn focus_rejects_explicit_non_edit_fidelity() {
    let (root, path, state) = fixture();
    let mut arguments = base_arguments(&root, &path);
    arguments["fidelity"] = json!("high");
    arguments["focusMethods"] = json!(["FocusProbe.selected"]);

    let response = dispatch(&state, arguments);

    assert_eq!(response["error"]["code"], -32602, "{response}");
    assert!(
        response["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("focusMethods") && message.contains("edit")),
        "{response}"
    );
}

#[test]
fn focus_rejects_explicit_non_edit_intent() {
    let (root, path, state) = fixture();
    let mut arguments = base_arguments(&root, &path);
    arguments["intent"] = json!("refactor");
    arguments["focusMethods"] = json!(["FocusProbe.selected"]);

    let response = dispatch(&state, arguments);

    assert_eq!(response["error"]["code"], -32602, "{response}");
}

#[test]
fn empty_focus_without_explicit_edit_is_rejected() {
    let (root, path, state) = fixture();
    let mut arguments = base_arguments(&root, &path);
    arguments["focusMethods"] = json!([]);

    let response = dispatch(&state, arguments);

    assert_eq!(response["error"]["code"], -32602, "{response}");
}
