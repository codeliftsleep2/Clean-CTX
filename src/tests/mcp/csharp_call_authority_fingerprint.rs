use crate::mcp::csharp_calls::authority::{manifest_constructions, reset_manifest_constructions};
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
fn dirty_generation_builds_one_manifest_and_warm_calls_are_constant_time() {
    let _serial = crate::protocol::handler_response_serial();
    let root = fixture();
    let state = state();
    let hydration = dispatch(&state, reverse(root.path(), "TypeRef", "Target"));
    assert!(hydration.get("error").is_none(), "{hydration}");
    reset_manifest_constructions();

    let first = dispatch(&state, reverse(root.path(), "Method", "Save"));
    let second = dispatch(&state, reverse(root.path(), "Method", "Save"));

    assert!(first.get("error").is_none(), "{first}");
    assert!(second.get("error").is_none(), "{second}");
    assert_eq!(manifest_constructions(), 1);
    let fingerprint = state
        .csharp_call_authority
        .fingerprint()
        .expect("committed authority fingerprint");
    assert!(
        state
            .workspace_index_read()
            .has_current_csharp_exact_call_coverage(
                &fingerprint,
                state.csharp_authority_generation(),
            )
    );
    let caller = state.semantic_owner_path(&root.path().join("Caller.cs").to_string_lossy());
    assert_eq!(
        state
            .workspace_index_read()
            .semantic_dependency_count(&caller),
        0,
        "exact-call authority must not expand into all-file dependencies"
    );
}

#[test]
fn dirty_calls_authority_does_not_invalidate_current_type_ref_evidence() {
    let _serial = crate::protocol::handler_response_serial();
    let root = fixture();
    let state = state();
    let initial = dispatch(&state, reverse(root.path(), "TypeRef", "Target"));
    assert!(initial.get("error").is_none(), "{initial}");
    state.prepare_exact_csharp_calls().expect("exact calls");
    state.mark_csharp_authority_dirty();
    reset_manifest_constructions();
    crate::mcp::semantic_work_metrics::reset();

    let type_ref = dispatch(&state, reverse(root.path(), "TypeRef", "Target"));

    assert!(type_ref.get("error").is_none(), "{type_ref}");
    assert_eq!(manifest_constructions(), 0);
    assert_eq!(crate::mcp::semantic_work_metrics::full_semantic_parses(), 0);
}
