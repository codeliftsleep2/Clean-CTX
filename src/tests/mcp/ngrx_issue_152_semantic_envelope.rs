use serde_json::{Value, json};
use std::fs;

fn call(state: &crate::mcp::McpState, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(152),
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

fn new_state(root: &std::path::Path) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn entities_in_file(state: &crate::mcp::McpState, root: &std::path::Path, file: &str) -> Value {
    call(
        state,
        json!({
            "type": "entities_in_file",
            "file_path": root.join(file),
            "workspaceRoot": root,
            "fidelity": "high"
        }),
    )
}

fn publish_all(state: &crate::mcp::McpState, root: &std::path::Path, files: &[&str]) {
    for file in files {
        entities_in_file(state, root, file);
    }
}

fn find(state: &crate::mcp::McpState, root: &std::path::Path, name: &str) -> Value {
    call(
        state,
        json!({"type":"find_entities", "name":name, "workspaceRoot":root}),
    )
}

fn forward(state: &crate::mcp::McpState, root: &std::path::Path, name: &str) -> Value {
    call(
        state,
        json!({
            "type":"forward_edges", "domain":"ngrx", "entity_type":"Action",
            "name":name, "workspaceRoot":root
        }),
    )
}

fn has_role(answer: &Value, entity_type: &str, name: &str) -> bool {
    answer["entities"].as_array().unwrap().iter().any(|entity| {
        entity["domain"] == "ngrx" && entity["entity_type"] == entity_type && entity["name"] == name
    })
}

fn write_dispatch(root: &std::path::Path) {
    fs::write(
        root.join("dispatch.ts"),
        r#"
import { Store } from '@ngrx/store';
import { ActionObjectA as AliasA } from './actions';
export class ComponentA {
  constructor(private store: Store<unknown>) {}
  send() { this.store.dispatch({ type: AliasA.ACTION_1 }); }
}
"#,
    )
    .unwrap();
}

fn write_bare_reducer(root: &std::path::Path) {
    fs::write(
        root.join("reducer.ts"),
        r#"
import { Action } from '@ngrx/store';
import { ACTION_1 } from './actions';
export function ReducerA(state: unknown, action: Action) {
  switch (action.type) { case ACTION_1: return state; default: return state; }
}
"#,
    )
    .unwrap();
}

fn write_direct_registration(root: &std::path::Path) {
    fs::write(
        root.join("module.ts"),
        r#"
import { StoreModule } from '@ngrx/store';
import { ReducerA } from './reducer';
StoreModule.forFeature('feature', ReducerA);
"#,
    )
    .unwrap();
}

#[test]
fn issue_152_cold_module_query_matches_warm_authoritative_truth() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("actions.ts"),
        "export const ActionObjectA = { ACTION_1: 'v1' };",
    )
    .unwrap();
    fs::write(
        root.path().join("reducer.ts"),
        r#"
import { ActionObjectA } from './actions';
export function ReducerA(state: unknown, action: any) {
  switch (action.type) { case ActionObjectA.ACTION_1: return state; default: return state; }
}
"#,
    )
    .unwrap();
    write_dispatch(root.path());
    write_direct_registration(root.path());

    let cold_state = new_state(root.path());
    let cold = entities_in_file(&cold_state, root.path(), "module.ts");

    let warm_state = new_state(root.path());
    publish_all(
        &warm_state,
        root.path(),
        &["reducer.ts", "actions.ts", "dispatch.ts"],
    );
    let warm = entities_in_file(&warm_state, root.path(), "module.ts");

    assert_eq!(cold["coverage"], warm["coverage"]);
    assert_eq!(
        cold["entities"], warm["entities"],
        "cold/warm truth diverged"
    );
    assert!(has_role(
        &cold,
        "StoreRegistration",
        "StoreModule.forFeature"
    ));
}

#[test]
fn issue_152_shorthand_member_and_bare_case_complete_the_authority_chain() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("actions.ts"),
        r#"
export const ACTION_1 = 'v1';
export const ActionObjectA = { ACTION_1 };
"#,
    )
    .unwrap();
    write_bare_reducer(root.path());
    write_dispatch(root.path());
    write_direct_registration(root.path());
    let state = new_state(root.path());
    publish_all(
        &state,
        root.path(),
        &["module.ts", "dispatch.ts", "actions.ts", "reducer.ts"],
    );

    let action = find(&state, root.path(), "ActionObjectA.ACTION_1");
    assert!(
        has_role(&action, "Action", "ActionObjectA.ACTION_1"),
        "{action}"
    );
    let edges = forward(&state, root.path(), "ActionObjectA.ACTION_1");
    assert!(
        edges["edges"].as_array().unwrap().iter().any(|edge| {
            edge["relation"] == "TriggersReducer" && edge["object"]["name"] == "ReducerA"
        }),
        "{edges}"
    );
    let actions = entities_in_file(&state, root.path(), "actions.ts");
    assert!(
        actions["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| {
                entity["domain"] == "typescript"
                    && entity["entity_type"] == "ObjectMember"
                    && entity["name"] == "ActionObjectA.ACTION_1"
            }),
        "{actions}"
    );
    assert!(
        actions["coverage"].get("semantic_omissions").is_none(),
        "{actions}"
    );
    let reducer = entities_in_file(&state, root.path(), "reducer.ts");
    assert!(
        reducer["coverage"].get("semantic_omissions").is_none(),
        "{reducer}"
    );
}

#[test]
fn issue_152_reference_valued_member_keeps_qualified_action_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("actions.ts"),
        r#"
export const ACTION_1 = 'v1';
export const ActionObjectA = { ACTION_1: ACTION_1 };
"#,
    )
    .unwrap();
    write_dispatch(root.path());
    let state = new_state(root.path());
    publish_all(&state, root.path(), &["dispatch.ts", "actions.ts"]);
    let action = find(&state, root.path(), "ActionObjectA.ACTION_1");
    assert!(
        has_role(&action, "Action", "ActionObjectA.ACTION_1"),
        "{action}"
    );
    assert!(!has_role(
        &find(&state, root.path(), "ACTION_1"),
        "Action",
        "ACTION_1"
    ));
}

#[test]
fn issue_152_registered_reducer_map_authorizes_members_not_shape_alone() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("actions.ts"),
        r#"
export const ACTION_1 = 'v1';
export const ActionObjectA = { ACTION_1 };
"#,
    )
    .unwrap();
    write_bare_reducer(root.path());
    write_dispatch(root.path());
    fs::write(
        root.path().join("reducers.ts"),
        r#"
import { ReducerA } from './reducer';
export const reducers = { feature: ReducerA };
"#,
    )
    .unwrap();
    fs::write(
        root.path().join("module.ts"),
        r#"
import { StoreModule } from '@ngrx/store';
import { reducers } from './reducers';
StoreModule.forRoot(reducers);
"#,
    )
    .unwrap();
    let state = new_state(root.path());
    publish_all(
        &state,
        root.path(),
        &[
            "module.ts",
            "reducers.ts",
            "reducer.ts",
            "dispatch.ts",
            "actions.ts",
        ],
    );
    assert!(has_role(
        &find(&state, root.path(), "ReducerA"),
        "Reducer",
        "ReducerA"
    ));
    let edges = forward(&state, root.path(), "ActionObjectA.ACTION_1");
    assert!(
        edges["edges"].as_array().unwrap().iter().any(|edge| {
            edge["relation"] == "TriggersReducer" && edge["object"]["name"] == "ReducerA"
        }),
        "{edges}"
    );

    let unregistered = tempfile::tempdir().unwrap();
    fs::write(
        unregistered.path().join("reducer.ts"),
        "export function ReducerA(state: unknown) { return state; }",
    )
    .unwrap();
    fs::write(
        unregistered.path().join("reducers.ts"),
        "import { ReducerA } from './reducer'; export const reducers = { feature: ReducerA };",
    )
    .unwrap();
    let state = new_state(unregistered.path());
    publish_all(&state, unregistered.path(), &["reducers.ts", "reducer.ts"]);
    assert!(!has_role(
        &find(&state, unregistered.path(), "ReducerA"),
        "Reducer",
        "ReducerA"
    ));
}
