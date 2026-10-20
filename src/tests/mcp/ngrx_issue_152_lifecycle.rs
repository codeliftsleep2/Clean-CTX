use serde_json::{Value, json};
use std::fs;

fn call(state: &crate::mcp::McpState, query: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(152),
        "workspace_query",
        &json!({"arguments": query}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("MCP response");
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"].clone()
}

fn state(root: &std::path::Path) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn publish(state: &crate::mcp::McpState, root: &std::path::Path, file: &str) -> Value {
    call(
        state,
        json!({
            "type":"entities_in_file", "file_path":root.join(file),
            "workspaceRoot":root, "fidelity":"high"
        }),
    )
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

fn triggers(answer: &Value, reducer: &str) -> bool {
    answer["edges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|edge| edge["relation"] == "TriggersReducer" && edge["object"]["name"] == reducer)
}

fn write_fixture(root: &std::path::Path) {
    fs::write(
        root.join("actions.ts"),
        "export const ACTION_1 = 'v1'; export const ActionObjectA = { ACTION_1 };",
    )
    .unwrap();
    fs::write(
        root.join("dispatch.ts"),
        r#"import { Store } from '@ngrx/store';
import { ActionObjectA as AliasA } from './actions';
export class C { constructor(private store: Store<unknown>) {}
send() { this.store.dispatch({ type: AliasA.ACTION_1 }); } }"#,
    )
    .unwrap();
    fs::write(
        root.join("reducer.ts"),
        r#"import { Action } from '@ngrx/store';
import { ACTION_1 } from './actions';
export function ReducerA(state: unknown, action: Action) {
switch (action.type) { case ACTION_1: return state; default: return state; } }"#,
    )
    .unwrap();
    fs::write(
        root.join("reducers.ts"),
        "import { ReducerA } from './reducer'; export const reducers = { feature: ReducerA };",
    )
    .unwrap();
    fs::write(
        root.join("module.ts"),
        r#"import { StoreModule } from '@ngrx/store';
import { reducers } from './reducers'; StoreModule.forRoot(reducers);"#,
    )
    .unwrap();
}

fn semantic_snapshot(state: &crate::mcp::McpState, root: &std::path::Path) -> (bool, bool, bool) {
    (
        has_role(
            &find(state, root, "ActionObjectA.ACTION_1"),
            "Action",
            "ActionObjectA.ACTION_1",
        ),
        has_role(&find(state, root, "ReducerA"), "Reducer", "ReducerA"),
        triggers(&forward(state, root, "ActionObjectA.ACTION_1"), "ReducerA"),
    )
}

#[test]
fn issue_152_publication_order_matrix_converges() {
    let _serial = crate::protocol::handler_response_serial();
    let orders = [
        ["actions.ts", "reducer.ts", "module.ts", "dispatch.ts"],
        ["dispatch.ts", "actions.ts", "module.ts", "reducer.ts"],
        ["module.ts", "reducer.ts", "actions.ts", "dispatch.ts"],
        ["reducer.ts", "module.ts", "dispatch.ts", "actions.ts"],
    ];
    for order in orders {
        let root = tempfile::tempdir().unwrap();
        write_fixture(root.path());
        let state = state(root.path());
        for file in order {
            publish(&state, root.path(), file);
        }
        assert_eq!(semantic_snapshot(&state, root.path()), (true, true, true));
    }
}

#[test]
fn issue_152_shared_constant_ambiguity_and_equal_literals_fail_closed() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("actions.ts"),
        "export const X = 'same'; export const Y = 'same'; export const A = { X }; export const B = { X }; export const C = { Y };",
    )
    .unwrap();
    fs::write(
        root.path().join("dispatch.ts"),
        r#"import { Store } from '@ngrx/store'; import { A, B, C } from './actions';
export class D { constructor(private store: Store<unknown>) {}
send() { this.store.dispatch({type:A.X}); this.store.dispatch({type:B.X}); this.store.dispatch({type:C.Y}); } }"#,
    )
    .unwrap();
    fs::write(
        root.path().join("reducer.ts"),
        r#"import { Action } from '@ngrx/store'; import { X, Y } from './actions';
export function R1(s: unknown, a: Action) { switch(a.type) { case X: return s; default:return s; } }
export function R2(s: unknown, a: Action) { switch(a.type) { case Y: return s; default:return s; } }"#,
    )
    .unwrap();
    fs::write(
        root.path().join("module.ts"),
        r#"import { StoreModule } from '@ngrx/store'; import { R1, R2 } from './reducer';
StoreModule.forFeature('one', R1); StoreModule.forFeature('two', R2);"#,
    )
    .unwrap();
    let state = state(root.path());
    for file in ["module.ts", "dispatch.ts", "actions.ts", "reducer.ts"] {
        publish(&state, root.path(), file);
    }
    assert!(!triggers(&forward(&state, root.path(), "A.X"), "R1"));
    assert!(!triggers(&forward(&state, root.path(), "B.X"), "R1"));
    assert!(triggers(&forward(&state, root.path(), "C.Y"), "R2"));
    assert!(!triggers(&forward(&state, root.path(), "A.X"), "R2"));
}

#[test]
fn issue_152_map_registration_and_dispatch_authority_retract_and_rebuild() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    write_fixture(root.path());
    let state = state(root.path());
    for file in [
        "module.ts",
        "reducers.ts",
        "reducer.ts",
        "dispatch.ts",
        "actions.ts",
    ] {
        publish(&state, root.path(), file);
    }
    assert_eq!(semantic_snapshot(&state, root.path()), (true, true, true));

    fs::write(
        root.path().join("reducers.ts"),
        "export const reducers = {};",
    )
    .unwrap();
    publish(&state, root.path(), "reducers.ts");
    assert_eq!(semantic_snapshot(&state, root.path()), (true, false, false));

    fs::write(
        root.path().join("reducers.ts"),
        "import { ReducerA } from './reducer'; export const reducers = { feature: ReducerA };",
    )
    .unwrap();
    publish(&state, root.path(), "reducers.ts");
    assert_eq!(semantic_snapshot(&state, root.path()), (true, true, true));

    fs::write(
        root.path().join("dispatch.ts"),
        "export const removed = true;",
    )
    .unwrap();
    publish(&state, root.path(), "dispatch.ts");
    assert_eq!(semantic_snapshot(&state, root.path()), (false, true, false));
}
