// RED regressions for the public, CBM-independent external-edit refresh boundary.

use crate::mcp::tool_dispatch::dispatch_tools_call;
use serde_json::json;
use std::path::Path;

fn filesystem_only_state() -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}

fn call(
    state: &crate::mcp::McpState,
    tool: &str,
    arguments: serde_json::Value,
) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("tool response")
}

fn find_count(state: &crate::mcp::McpState, root: &Path, name: &str) -> u64 {
    let response = call(
        state,
        "workspace_query",
        json!({
            "type": "find_entities",
            "name": name,
            "workspaceRoot": root.to_string_lossy()
        }),
    );
    response["result"]["structuredContent"]["count"]
        .as_u64()
        .unwrap_or_else(|| panic!("find_entities must succeed: {response:?}"))
}

#[test]
fn refresh_workspace_is_a_registered_read_only_idempotent_tool() {
    let tool = crate::mcp::tools::tool_list()
        .into_iter()
        .find(|tool| tool["name"] == "refresh_workspace")
        .expect("refresh_workspace must be public");

    assert_eq!(tool["inputSchema"]["required"], json!(["workspaceRoot"]));
    assert_eq!(tool["annotations"]["readOnlyHint"], true);
    assert_eq!(tool["annotations"]["destructiveHint"], false);
    assert_eq!(tool["annotations"]["idempotentHint"], true);
}

#[test]
fn refresh_workspace_discovers_external_creates_without_cbm() {
    let root = tempfile::TempDir::new().unwrap();
    let state = filesystem_only_state();

    assert_eq!(find_count(&state, root.path(), "ExternallyCreated"), 0);
    std::fs::write(
        root.path().join("ExternallyCreated.cs"),
        "public class ExternallyCreated {}",
    )
    .unwrap();
    assert_eq!(
        find_count(&state, root.path(), "ExternallyCreated"),
        0,
        "a completed discovery generation stays stable until explicit refresh"
    );

    let refreshed = call(
        &state,
        "refresh_workspace",
        json!({ "workspaceRoot": root.path().to_string_lossy() }),
    );
    assert_eq!(refreshed["result"]["structuredContent"]["refreshed"], true);
    assert_eq!(
        refreshed["result"]["structuredContent"]["workspace_root"],
        crate::dictionary::path::canonical_identity_key(&root.path().to_string_lossy())
    );
    assert_eq!(find_count(&state, root.path(), "ExternallyCreated"), 1);
}

#[test]
fn refresh_workspace_retracts_external_replacements_and_deletions() {
    let root = tempfile::TempDir::new().unwrap();
    let state = filesystem_only_state();
    let file = root.path().join("Authority.cs");
    std::fs::write(&file, "public class OldAuthority {}").unwrap();
    assert_eq!(find_count(&state, root.path(), "OldAuthority"), 1);

    std::fs::write(&file, "public class NewAuthority {}").unwrap();
    let refreshed = call(
        &state,
        "refresh_workspace",
        json!({ "workspaceRoot": root.path().to_string_lossy() }),
    );
    assert!(
        refreshed.get("result").is_some(),
        "refresh must succeed: {refreshed:?}"
    );
    assert_eq!(find_count(&state, root.path(), "OldAuthority"), 0);
    assert_eq!(find_count(&state, root.path(), "NewAuthority"), 1);

    std::fs::remove_file(file).unwrap();
    let removed = call(
        &state,
        "refresh_workspace",
        json!({ "workspaceRoot": root.path().to_string_lossy() }),
    );
    assert!(
        removed.get("result").is_some(),
        "refresh must succeed: {removed:?}"
    );
    assert_eq!(find_count(&state, root.path(), "NewAuthority"), 0);
}
