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

fn filesystem_state_with_additional_root(additional: &Path) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(additional.to_string_lossy().into_owned());
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

#[test]
fn refresh_workspace_reconciles_the_effective_workspace_root_set() {
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    let state = filesystem_state_with_additional_root(additional.path());

    assert_eq!(find_count(&state, primary.path(), "ExternalAdditional"), 0);
    std::fs::write(
        additional.path().join("ExternalAdditional.cs"),
        "public class ExternalAdditional {}",
    )
    .unwrap();
    assert_eq!(find_count(&state, primary.path(), "ExternalAdditional"), 0);

    let refreshed = call(
        &state,
        "refresh_workspace",
        json!({ "workspaceRoot": primary.path().to_string_lossy() }),
    );
    let roots = refreshed["result"]["structuredContent"]["workspace_roots"]
        .as_array()
        .expect("refresh reports every effective root");
    assert_eq!(roots.len(), 2, "primary and additional roots are refreshed");
    assert_eq!(find_count(&state, primary.path(), "ExternalAdditional"), 1);
}

#[test]
fn refresh_workspace_reports_reconciled_authority_counts() {
    let root = tempfile::TempDir::new().unwrap();
    let state = filesystem_only_state();
    std::fs::write(root.path().join("Counted.cs"), "public class Counted {}").unwrap();
    assert_eq!(find_count(&state, root.path(), "Counted"), 1);

    let refreshed = call(
        &state,
        "refresh_workspace",
        json!({ "workspaceRoot": root.path().to_string_lossy() }),
    );
    let result = &refreshed["result"]["structuredContent"];
    assert_eq!(result["refreshed"], true);
    assert_eq!(result["roots_refreshed"], 1);
    assert!(result["indexed_owners_retracted"].as_u64().is_some());
    assert!(result["source_snapshots_invalidated"].as_u64().is_some());
    assert!(result["pending_transitions_retired"].as_u64().is_some());
}

#[test]
fn refresh_workspace_retracts_a_workspace_whose_root_was_deleted() {
    let parent = tempfile::TempDir::new().unwrap();
    let root = parent.path().join("deleted-workspace");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("Gone.cs"), "public class Gone {}").unwrap();
    let state = filesystem_only_state();
    assert_eq!(find_count(&state, &root, "Gone"), 1);

    std::fs::remove_dir_all(&root).unwrap();
    let refreshed = call(
        &state,
        "refresh_workspace",
        json!({ "workspaceRoot": root.to_string_lossy() }),
    );
    assert!(
        refreshed.get("result").is_some(),
        "a missing formerly indexed root must still be refreshable: {refreshed:?}"
    );
    assert_eq!(find_count(&state, &root, "Gone"), 0);
}

#[cfg(windows)]
#[test]
fn refresh_workspace_invalidates_case_varied_source_cache_identity() {
    let root = tempfile::TempDir::new().unwrap();
    let file = root.path().join("CaseIdentity.cs");
    std::fs::write(&file, "public class Before {}").unwrap();
    let state = filesystem_only_state();
    let lower_file = file.to_string_lossy().to_ascii_lowercase();
    let upper_root = root.path().to_string_lossy().to_ascii_uppercase();
    state.read_source(&lower_file).unwrap();
    assert!(state.cached_source_snapshot(&lower_file).is_some());

    let refreshed = call(
        &state,
        "refresh_workspace",
        json!({ "workspaceRoot": upper_root }),
    );
    assert!(refreshed.get("result").is_some(), "{refreshed:?}");
    assert!(
        state.cached_source_snapshot(&lower_file).is_none(),
        "refresh containment must follow Windows filesystem identity"
    );
}

#[test]
fn refresh_workspace_retires_pending_delta_semantics() {
    let root = tempfile::TempDir::new().unwrap();
    let state = filesystem_only_state();
    let file = root.path().join("Pending.cs");
    let file_text = file.to_string_lossy().into_owned();
    std::fs::write(&file, "public class Pending { void Old() {} }").unwrap();

    let arguments = || {
        json!({
            "filePath": file_text,
            "workspaceRoot": root.path(),
            "fidelity": "low"
        })
    };
    let baseline = call(&state, "delta_code_context", arguments());
    assert!(baseline.get("error").is_none(), "{baseline:?}");
    std::fs::write(&file, "public class Pending { void Target() {} }").unwrap();
    state.invalidate_source_cache(&file_text);
    let generated = call(&state, "delta_code_context", arguments());
    let delta = generated["result"]["delta"].clone();
    let from = generated["result"]["from_version"].clone();
    assert_eq!(delta["dv"], 2, "{generated:?}");

    std::fs::write(&file, "public class ExternalReplacement {}").unwrap();
    let refreshed = call(
        &state,
        "refresh_workspace",
        json!({ "workspaceRoot": root.path().to_string_lossy() }),
    );
    assert!(refreshed.get("result").is_some(), "{refreshed:?}");

    let applied = call(
        &state,
        "apply_delta",
        json!({ "delta": delta, "currentVersion": from }),
    );
    assert!(
        applied.get("error").is_some(),
        "a pre-refresh pending transition must not regain current authority: {applied:?}"
    );
    assert_eq!(find_count(&state, root.path(), "ExternalReplacement"), 1);
}
