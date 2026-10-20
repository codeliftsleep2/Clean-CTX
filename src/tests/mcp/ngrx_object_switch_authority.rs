use serde_json::{Value, json};
use std::fs;

fn call(state: &crate::mcp::McpState, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(151),
        "workspace_query",
        &json!({"arguments": arguments}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("MCP response");
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"].clone()
}

fn publish(state: &crate::mcp::McpState, root: &std::path::Path, file: &std::path::Path) {
    call(
        state,
        json!({
            "type": "entities_in_file",
            "file_path": file,
            "workspaceRoot": root,
            "fidelity": "high"
        }),
    );
}

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("actions.ts"),
        "export const ACTIONS = { SET_X: '[Feature] Same', CLEAR_X: '[Feature] Same' };",
    )
    .unwrap();
    fs::write(
        root.path().join("reducer.ts"),
        r#"
import { ACTIONS as FeatureActions } from './actions';
export function reducer(state: unknown, evt: any) {
  switch (evt.type) {
    case FeatureActions.SET_X: return state;
    case FeatureActions.CLEAR_X: return state;
    default: return state;
  }
}
"#,
    )
    .unwrap();
    fs::write(
        root.path().join("dispatch.ts"),
        r#"
import { Store } from '@ngrx/store';
import { ACTIONS as FeatureActions } from './actions';
export class FeatureComponent {
  constructor(private state: Store<unknown>) {}
  send() {
    this.state.dispatch({ type: FeatureActions.SET_X });
    this.state.dispatch({ type: FeatureActions.CLEAR_X });
  }
}
"#,
    )
    .unwrap();
    fs::write(
        root.path().join("module.ts"),
        r#"
import { StoreModule } from '@ngrx/store';
import { reducer as featureReducer } from './reducer';
StoreModule.forFeature('feature', featureReducer);
"#,
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    (root, state)
}

fn publish_all(root: &std::path::Path, state: &crate::mcp::McpState, order: &[&str]) {
    for file in order {
        publish(state, root, &root.join(file));
    }
}

#[test]
fn authorized_action_and_reducer_publish_triggers_reducer() {
    let (root, state) = fixture();
    publish_all(
        root.path(),
        &state,
        &["actions.ts", "reducer.ts", "dispatch.ts", "module.ts"],
    );
    let answer = call(
        &state,
        json!({
            "type": "forward_edges", "name": "ACTIONS.SET_X", "domain": "ngrx",
            "entity_type": "Action", "workspaceRoot": root.path()
        }),
    );
    let edges = answer["edges"].as_array().unwrap();
    assert!(
        edges.iter().any(
            |edge| edge["relation"] == "TriggersReducer" && edge["object"]["name"] == "reducer"
        ),
        "{answer}"
    );
    let reverse = call(
        &state,
        json!({
            "type": "reverse_edges", "name": "ACTIONS.SET_X", "domain": "ngrx",
            "entity_type": "Action", "workspaceRoot": root.path()
        }),
    );
    assert!(
        reverse["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| edge["relation"] == "Dispatches"),
        "{reverse}"
    );
}

#[test]
fn publication_order_and_multiple_aliases_converge_on_member_identity() {
    for order in [
        ["module.ts", "dispatch.ts", "reducer.ts", "actions.ts"],
        ["dispatch.ts", "actions.ts", "module.ts", "reducer.ts"],
    ] {
        let (root, state) = fixture();
        publish_all(root.path(), &state, &order);
        let answer = call(
            &state,
            json!({
                "type": "find_entities", "name": "ACTIONS.SET_X", "workspaceRoot": root.path()
            }),
        );
        assert!(
            answer["entities"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Action"),
            "{answer}"
        );
        assert!(
            answer["entities"]
                .as_array()
                .unwrap()
                .iter()
                .all(|entity| entity["name"] != "FeatureActions.SET_X"),
            "{answer}"
        );
    }
}

#[test]
fn fake_dispatch_and_unregistered_switch_fail_closed() {
    let (root, state) = fixture();
    fs::remove_file(root.path().join("module.ts")).unwrap();
    fs::write(
        root.path().join("dispatch.ts"),
        r#"
import { ACTIONS } from './actions';
const something = { dispatch(value: unknown) {} };
something.dispatch({ type: ACTIONS.SET_X });
"#,
    )
    .unwrap();
    publish_all(
        root.path(),
        &state,
        &["actions.ts", "reducer.ts", "dispatch.ts"],
    );
    let action = call(
        &state,
        json!({
            "type": "find_entities", "name": "ACTIONS.SET_X", "workspaceRoot": root.path()
        }),
    );
    assert!(
        !action["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Action"),
        "{action}"
    );
    let reducer = call(
        &state,
        json!({
            "type": "find_entities", "name": "reducer", "workspaceRoot": root.path()
        }),
    );
    assert!(
        !reducer["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Reducer"),
        "{reducer}"
    );
}

#[test]
fn action_looking_string_and_fake_store_spelling_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("lookalike.ts"),
        r#"
const X = '[Feature] Set X';
const ACTIONS = { SET_X: X };
const Store = { dispatch(value: unknown) {} };
Store.dispatch({ type: ACTIONS.SET_X });
function reducer(state: unknown, action: any) {
  switch (action.type) { case ACTIONS.SET_X: return state; default: return state; }
}
"#,
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    publish(&state, root.path(), &root.path().join("lookalike.ts"));
    for name in ["ACTIONS.SET_X", "reducer"] {
        let answer = call(
            &state,
            json!({"type":"find_entities", "name":name, "workspaceRoot":root.path()}),
        );
        assert!(
            !answer["entities"].as_array().unwrap().iter().any(|entity| {
                entity["domain"] == "ngrx"
                    && matches!(entity["entity_type"].as_str(), Some("Action" | "Reducer"))
            }),
            "{answer}"
        );
    }
}

#[test]
fn removing_authority_retracts_derived_roles_and_handlers() {
    let (root, state) = fixture();
    publish_all(
        root.path(),
        &state,
        &["actions.ts", "reducer.ts", "dispatch.ts", "module.ts"],
    );
    fs::write(
        root.path().join("dispatch.ts"),
        "export const noDispatch = true;",
    )
    .unwrap();
    publish(&state, root.path(), &root.path().join("dispatch.ts"));
    let answer = call(
        &state,
        json!({
            "type": "find_entities", "name": "ACTIONS.SET_X", "workspaceRoot": root.path()
        }),
    );
    assert!(
        !answer["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Action"),
        "{answer}"
    );
}

#[test]
fn remaining_dispatch_authority_preserves_action() {
    let (root, state) = fixture();
    fs::copy(
        root.path().join("dispatch.ts"),
        root.path().join("dispatch-two.ts"),
    )
    .unwrap();
    publish_all(
        root.path(),
        &state,
        &["actions.ts", "dispatch.ts", "dispatch-two.ts"],
    );
    fs::write(
        root.path().join("dispatch.ts"),
        "export const removed = true;",
    )
    .unwrap();
    publish(&state, root.path(), &root.path().join("dispatch.ts"));
    let answer = call(
        &state,
        json!({
            "type": "find_entities", "name": "ACTIONS.SET_X", "workspaceRoot": root.path()
        }),
    );
    assert!(
        answer["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Action"),
        "{answer}"
    );
}

#[test]
fn registration_removal_retracts_reducer_and_handler() {
    let (root, state) = fixture();
    publish_all(
        root.path(),
        &state,
        &["actions.ts", "reducer.ts", "dispatch.ts", "module.ts"],
    );
    fs::write(
        root.path().join("module.ts"),
        "export const removed = true;",
    )
    .unwrap();
    publish(&state, root.path(), &root.path().join("module.ts"));
    let edges = call(
        &state,
        json!({
            "type": "forward_edges", "name": "ACTIONS.SET_X", "domain": "ngrx",
            "entity_type": "Action", "workspaceRoot": root.path()
        }),
    );
    assert!(
        !edges["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| edge["relation"] == "TriggersReducer"),
        "{edges}"
    );
    let reducer = call(
        &state,
        json!({
            "type": "find_entities", "name": "reducer", "workspaceRoot": root.path()
        }),
    );
    assert!(
        !reducer["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Reducer"),
        "{reducer}"
    );
}

#[test]
fn removing_action_member_retracts_action_and_handler() {
    let (root, state) = fixture();
    publish_all(
        root.path(),
        &state,
        &["actions.ts", "reducer.ts", "dispatch.ts", "module.ts"],
    );
    fs::write(
        root.path().join("actions.ts"),
        "export const ACTIONS = { CLEAR_X: '[Feature] Same' };",
    )
    .unwrap();
    publish(&state, root.path(), &root.path().join("actions.ts"));
    let answer = call(
        &state,
        json!({"type":"find_entities", "name":"ACTIONS.SET_X", "workspaceRoot":root.path()}),
    );
    assert!(
        !answer["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| { entity["domain"] == "ngrx" && entity["entity_type"] == "Action" }),
        "{answer}"
    );
}

#[test]
fn wrong_switch_variable_and_case_change_do_not_leave_stale_handler() {
    let (root, state) = fixture();
    publish_all(
        root.path(),
        &state,
        &["actions.ts", "dispatch.ts", "module.ts"],
    );
    fs::write(root.path().join("reducer.ts"), r#"
import { ACTIONS } from './actions';
export function reducer(state: unknown, evt: any) {
  switch (message.type) { case ACTIONS.SET_X: return state; default: return state; }
}

#[test]
fn supported_form_clears_only_earned_coverage_omissions() {
    let (root, state) = fixture();
    publish_all(root.path(), &state, &["actions.ts", "reducer.ts", "dispatch.ts", "module.ts"]);
    let reducer = call(&state, json!({
        "type": "entities_in_file", "file_path": root.path().join("reducer.ts"),
        "workspaceRoot": root.path(), "fidelity": "high"
    }));
    assert!(reducer["coverage"].get("semantic_omissions").is_none(), "{reducer}");

    fs::write(root.path().join("module.ts"), "export const removed = true;").unwrap();
    publish(&state, root.path(), &root.path().join("module.ts"));
    let reducer = call(&state, json!({
        "type": "entities_in_file", "file_path": root.path().join("reducer.ts"),
        "workspaceRoot": root.path(), "fidelity": "high"
    }));
    assert_eq!(reducer["coverage"]["semantic_omissions"], json!(["ngrx_switch_action_type_handlers"]));
}

#[test]
fn cold_file_query_acquires_the_same_authority_as_warm_publication() {
    let (cold_root, cold_state) = fixture();
    let cold = call(
        &cold_state,
        json!({
            "type": "entities_in_file", "file_path": cold_root.path().join("reducer.ts"),
            "workspaceRoot": cold_root.path(), "fidelity": "high"
        }),
    );
    assert!(
        cold["coverage"].get("semantic_omissions").is_none(),
        "{cold}"
    );

    let (warm_root, warm_state) = fixture();
    publish_all(
        warm_root.path(),
        &warm_state,
        &["actions.ts", "reducer.ts", "dispatch.ts", "module.ts"],
    );
    let warm = call(
        &warm_state,
        json!({
            "type": "entities_in_file", "file_path": warm_root.path().join("reducer.ts"),
            "workspaceRoot": warm_root.path(), "fidelity": "high"
        }),
    );
    assert_eq!(cold["coverage"], warm["coverage"]);
}
"#).unwrap();
    publish(&state, root.path(), &root.path().join("reducer.ts"));
    let set_x = call(
        &state,
        json!({
            "type": "forward_edges", "name": "ACTIONS.SET_X", "domain": "ngrx",
            "entity_type": "Action", "workspaceRoot": root.path()
        }),
    );
    assert!(
        !set_x["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| edge["relation"] == "TriggersReducer"),
        "{set_x}"
    );

    fs::write(
        root.path().join("reducer.ts"),
        r#"
import { ACTIONS } from './actions';
export function reducer(state: unknown, evt: any) {
  switch (evt.type) { case ACTIONS.CLEAR_X: return state; default: return state; }
}
"#,
    )
    .unwrap();
    publish(&state, root.path(), &root.path().join("reducer.ts"));
    let clear = call(
        &state,
        json!({
            "type": "forward_edges", "name": "ACTIONS.CLEAR_X", "domain": "ngrx",
            "entity_type": "Action", "workspaceRoot": root.path()
        }),
    );
    assert!(
        clear["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| edge["relation"] == "TriggersReducer"),
        "{clear}"
    );
}
