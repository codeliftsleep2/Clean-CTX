use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state() -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}

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
        .expect("workspace_query response")
}

fn reverse(root: &std::path::Path, entity_type: &str, name: &str) -> Value {
    json!({
        "type": "reverse_edges",
        "domain": "builtin",
        "entity_type": entity_type,
        "name": name,
        "workspaceRoot": root,
    })
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("workspace");
    std::fs::write(
        root.path().join("Caller.cs"),
        "class Caller { void Run(Target target) { target.Save(); } }",
    )
    .expect("caller");
    std::fs::write(
        root.path().join("Target.cs"),
        "class Target { public void Save() {} }",
    )
    .expect("target");
    root
}

#[test]
fn retained_artifacts_project_cross_file_calls_without_reparsing() {
    let _serial = crate::protocol::handler_response_serial();
    let root = fixture();
    let state = state();
    let hydration = dispatch(&state, reverse(root.path(), "TypeRef", "Target"));
    assert!(hydration.get("error").is_none(), "{hydration}");
    state.reset_test_full_semantic_parses();

    let calls = dispatch(&state, reverse(root.path(), "Method", "Save"));

    assert!(calls.get("error").is_none(), "{calls}");
    let edges = calls["result"]["structuredContent"]["edges"]
        .as_array()
        .expect("structured exact-call edges");
    assert_eq!(edges.len(), 1, "{calls}");
    assert_eq!(edges[0]["subject"]["name"], "Run", "{calls}");
    assert_eq!(edges[0]["object"]["name"], "Save", "{calls}");
    assert_eq!(state.test_full_semantic_parses(), 0);
}

#[test]
fn missing_artifacts_are_compiled_once_then_reused() {
    let _serial = crate::protocol::handler_response_serial();
    let root = fixture();
    let state = state();
    let hydration = dispatch(&state, reverse(root.path(), "TypeRef", "Target"));
    assert!(hydration.get("error").is_none(), "{hydration}");
    state.csharp_call_authority.clear_artifacts();
    state.mark_csharp_authority_dirty();
    state.reset_test_full_semantic_parses();

    let first = dispatch(&state, reverse(root.path(), "Method", "Save"));
    let first_parses = state.test_full_semantic_parses();
    let second = dispatch(&state, reverse(root.path(), "Method", "Save"));

    assert!(first.get("error").is_none(), "{first}");
    assert!(second.get("error").is_none(), "{second}");
    assert!(
        first_parses > 0,
        "missing artifacts must fail closed through compilation"
    );
    assert_eq!(
        state.test_full_semantic_parses(),
        first_parses,
        "a current exact-call generation must reuse retained artifacts"
    );
}
