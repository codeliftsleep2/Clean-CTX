use crate::mcp::csharp_calls::lifecycle::CSharpCallAuthorityStatus;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};
use std::sync::Arc;

fn state() -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}

fn write_fixture(root: &std::path::Path) -> String {
    let path = root.join("Calls.cs");
    std::fs::write(
        &path,
        "class Target { public void Hit() {} } class Caller { void Go() { new Target().Hit(); } }\n",
    )
    .expect("fixture");
    path.to_string_lossy().into_owned()
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

fn exact_reverse(root: &std::path::Path, entity_type: &str, name: &str) -> Value {
    json!({
        "type": "reverse_edges",
        "domain": "builtin",
        "entity_type": entity_type,
        "name": name,
        "workspaceRoot": root,
    })
}

#[test]
fn type_ref_publication_defers_exact_call_preparation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let state = state();
    state.reset_test_csharp_reconciliation_count();

    let response = dispatch(&state, exact_reverse(root.path(), "TypeRef", "Target"));

    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(state.test_csharp_reconciliation_count(), 0);
    assert_eq!(
        state.csharp_call_authority.status(),
        CSharpCallAuthorityStatus::Dirty
    );
}

#[test]
fn first_exact_call_consumer_prepares_once_and_warm_consumer_reuses() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let state = state();
    state.reset_test_csharp_reconciliation_count();

    let first = dispatch(&state, exact_reverse(root.path(), "Method", "Hit"));
    let second = dispatch(&state, exact_reverse(root.path(), "Method", "Hit"));

    assert!(first.get("error").is_none(), "{first}");
    assert!(second.get("error").is_none(), "{second}");
    assert_eq!(state.test_csharp_reconciliation_count(), 1);
    assert_eq!(
        state.csharp_call_authority.status(),
        CSharpCallAuthorityStatus::Current
    );
}

#[test]
fn compatible_batch_call_items_prepare_once() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let state = state();
    state.reset_test_csharp_reconciliation_count();

    let response = dispatch(
        &state,
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                { "id": "a", "type": "reverse_edges", "domain": "builtin", "entity_type": "Method", "name": "Hit" },
                { "id": "b", "type": "forward_edges", "domain": "builtin", "entity_type": "Method", "name": "Go" }
            ]
        }),
    );

    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(state.test_csharp_reconciliation_count(), 1);
}

#[test]
fn concurrent_same_generation_preparations_coalesce() {
    let _serial = crate::protocol::handler_response_serial();
    let state = Arc::new(state());
    state.reset_test_csharp_reconciliation_count();
    state.mark_csharp_authority_dirty();

    std::thread::scope(|scope| {
        for _ in 0..4 {
            let state = Arc::clone(&state);
            let _handle =
                scope.spawn(move || state.prepare_exact_csharp_calls().expect("preparation"));
        }
    });

    assert_eq!(state.test_csharp_reconciliation_count(), 1);
}

#[test]
fn failed_preparation_remains_dirty_and_returns_no_current_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let state = state();
    let hydration = dispatch(&state, exact_reverse(root.path(), "TypeRef", "Target"));
    assert!(hydration.get("error").is_none(), "{hydration}");
    state.csharp_call_authority.clear_artifacts();
    state.mark_csharp_authority_dirty();
    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .expect("failure injection") = Some("class Target".to_string());

    let result = state.prepare_exact_csharp_calls();
    crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .expect("failure injection")
        .take();

    assert!(result.is_err());
    assert_eq!(
        state.csharp_call_authority.status(),
        CSharpCallAuthorityStatus::Dirty
    );
}
