//! Selector declarations must be public because of declaration evidence, not use spelling.
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn call(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(140), tool, &json!({"arguments": args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"].clone()
}

fn selectors_in_file(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    path: &std::path::Path,
) -> Vec<Value> {
    let result = call(
        state,
        "workspace_query",
        json!({"workspaceRoot": root, "type": "entities_in_file", "file_path": path}),
    );
    result["structuredContent"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Selector")
        .cloned()
        .collect()
}

#[test]
fn ngrx_selector_declaration_without_use_is_publicly_reachable() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let selector_path = root.path().join("audit.selectors.ts");
    let ordinary_path = root.path().join("ordinary.ts");
    let selector_source = "import { createSelector } from '@ngrx/store';\nexport const selectAudit = createSelector(selectAuditState, state => state.audit);";
    let ordinary_source = "export const selectAudit = 'not a selector';";
    assert!(!selector_source.contains(".select("));
    std::fs::write(&selector_path, selector_source).unwrap();
    std::fs::write(&ordinary_path, ordinary_source).unwrap();

    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);

    assert!(
        selectors_in_file(&state, root.path(), &ordinary_path).is_empty(),
        "ordinary same-name declaration must not gain Selector authority"
    );
    let selectors = selectors_in_file(&state, root.path(), &selector_path);
    assert_eq!(
        selectors.len(),
        1,
        "source-proven Selector declaration is missing: {selectors:?}"
    );
    assert_eq!(selectors[0]["name"], "selectAudit");
    assert_eq!(
        selectors[0]["file"],
        selector_path.to_string_lossy().as_ref()
    );
    let index = state.workspace_index_read();
    let declarations = index.declarations_by_identity("ngrx", "Selector", "selectAudit");
    assert_eq!(declarations.len(), 1);
    assert_eq!(
        declarations[0].file.as_deref(),
        Some(selector_path.to_string_lossy().as_ref())
    );
}

#[test]
fn ngrx_selector_use_endpoints_do_not_gain_declaration_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let identifier_path = root.path().join("identifier-use.ts");
    let quoted_path = root.path().join("quoted-use.ts");
    std::fs::write(
        &identifier_path,
        "import { Store } from '@ngrx/store';\ndeclare const store: Store<unknown>;\nstore.select(selectAudit);",
    )
    .unwrap();
    std::fs::write(
        &quoted_path,
        "import { Store } from '@ngrx/store';\ndeclare const store: Store<unknown>;\nstore.select('selectAudit');",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);

    assert_eq!(
        selectors_in_file(&state, root.path(), &identifier_path).len(),
        1
    );
    assert_eq!(
        selectors_in_file(&state, root.path(), &quoted_path).len(),
        1
    );
    let index = state.workspace_index_read();
    assert!(
        index
            .declarations_in_file(&identifier_path.to_string_lossy())
            .is_empty()
    );
    assert!(
        index
            .declarations_in_file(&quoted_path.to_string_lossy())
            .is_empty()
    );
    assert!(
        index
            .declarations_by_identity("ngrx", "Selector", "selectAudit")
            .is_empty()
    );
}

#[test]
fn declaration_authority_respects_disabled_ngrx_and_preserves_action_publication() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let selector_path = root.path().join("disabled-selector.ts");
    let action_path = root.path().join("action.ts");
    std::fs::write(
        &selector_path,
        "import { createSelector } from '@ngrx/store';\nexport const selectAudit = createSelector(selectAuditState, state => state.audit);",
    )
    .unwrap();
    std::fs::write(
        &action_path,
        "import { createAction } from '@ngrx/store';\nexport const setAudit = createAction('[Audit] Set');",
    )
    .unwrap();

    let mut disabled_config = crate::tests::test_config();
    disabled_config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    disabled_config
        .meta_layers
        .entry("angular".into())
        .or_default()
        .ngrx
        .enabled = false;
    let disabled = crate::mcp::McpState::new(disabled_config);
    assert!(selectors_in_file(&disabled, root.path(), &selector_path).is_empty());
    assert!(
        disabled
            .workspace_index_read()
            .declarations_in_file(&selector_path.to_string_lossy())
            .is_empty()
    );

    let mut enabled_config = crate::tests::test_config();
    enabled_config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let enabled = crate::mcp::McpState::new(enabled_config);
    let result = call(
        &enabled,
        "workspace_query",
        json!({"workspaceRoot": root.path(), "type": "entities_in_file", "file_path": action_path}),
    );
    assert!(
        result["structuredContent"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| entity["domain"] == "ngrx"
                && entity["entity_type"] == "Action"
                && entity["name"] == "setAudit")
    );
    assert_eq!(
        enabled
            .workspace_index_read()
            .declarations_by_identity("ngrx", "Action", "setAudit")
            .len(),
        1
    );
}
