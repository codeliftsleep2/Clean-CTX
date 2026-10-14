use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

const ACTIONS: &str = r#"
import { Action } from '@ngrx/store';
export const SET_AUDIT = '[Audit] Set';
export function auditReducer(state: any, action: Action) {
  switch (action.type) {
    case SET_AUDIT: return state;
    default: return state;
  }
}
"#;

const DISPATCH: &str = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { SET_AUDIT } from './legacy.actions';
@Component({selector: 'audit-dispatch', template: ''})
export class DispatchProbe {
  constructor(private store: Store<unknown>) {}
  send() { this.store.dispatch({type: SET_AUDIT, payload: 1}); }
}
"#;

fn call(state: &crate::mcp::McpState, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(140), tool, &json!({"arguments": arguments}), state);
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("MCP response");
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");
    response["result"]["structuredContent"].clone()
}

fn publish(state: &crate::mcp::McpState, root: &std::path::Path, path: &std::path::Path) {
    call(
        state,
        "provide_code_context",
        json!({"workspaceRoot": root, "filePath": path, "fidelity": "high"}),
    );
}

fn dispatch_targets(state: &crate::mcp::McpState, root: &std::path::Path) -> Vec<String> {
    call(
        state,
        "workspace_query",
        json!({
            "workspaceRoot": root, "type": "forward_edges",
            "domain": "angular", "entity_type": "Component", "name": "DispatchProbe"
        }),
    )["edges"]
        .as_array()
        .expect("edges")
        .iter()
        .filter(|edge| edge["relation"] == "Dispatches")
        .map(|edge| {
            edge["object"]["name"]
                .as_str()
                .expect("target name")
                .to_owned()
        })
        .collect()
}

fn state(root: &std::path::Path) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn refresh(state: &crate::mcp::McpState, root: &std::path::Path) {
    call(state, "refresh_workspace", json!({"workspaceRoot": root}));
}

#[test]
fn object_dispatch_requires_the_exact_imported_discriminant_declaration() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let actions = root.path().join("legacy.actions.ts");
    let foreign = root.path().join("foreign.ts");
    let dispatch = root.path().join("dispatch.component.ts");
    std::fs::write(&actions, ACTIONS).expect("actions fixture");
    std::fs::write(&foreign, "export const SET_AUDIT = '[Foreign] Set';").expect("foreign fixture");
    std::fs::write(&dispatch, DISPATCH).expect("dispatch fixture");

    let state = state(root.path());
    for path in [&actions, &foreign, &dispatch] {
        publish(&state, root.path(), path);
    }
    assert_eq!(dispatch_targets(&state, root.path()), ["SET_AUDIT"]);

    std::fs::write(
        &dispatch,
        DISPATCH.replace("'./legacy.actions'", "'./foreign'"),
    )
    .expect("foreign dispatch fixture");
    call(
        &state,
        "refresh_workspace",
        json!({"workspaceRoot": root.path()}),
    );
    publish(&state, root.path(), &dispatch);
    assert!(
        dispatch_targets(&state, root.path()).is_empty(),
        "same spelling from a non-authoritative module must not dispatch an Action"
    );
}

#[test]
fn exact_source_selection_shadowing_and_unsupported_imports_fail_closed() {
    let _serial = crate::protocol::handler_response_serial();
    for (name, dispatch_source, expected) in [
        ("direct", DISPATCH.to_owned(), true),
        (
            "shadow",
            DISPATCH.replace("send() {", "send() { const SET_AUDIT = '[Local] Set';"),
            false,
        ),
        (
            "alias",
            DISPATCH
                .replace("{ SET_AUDIT }", "{ SET_AUDIT as SET }")
                .replace("type: SET_AUDIT", "type: SET"),
            false,
        ),
        (
            "namespace",
            DISPATCH
                .replace("{ SET_AUDIT }", "* as Actions")
                .replace("type: SET_AUDIT", "type: Actions.SET_AUDIT"),
            false,
        ),
        (
            "path-alias",
            DISPATCH.replace("'./legacy.actions'", "'@app/legacy.actions'"),
            false,
        ),
        (
            "missing-module",
            DISPATCH.replace("'./legacy.actions'", "'./missing'"),
            false,
        ),
        (
            "missing-export",
            DISPATCH.replace("{ SET_AUDIT }", "{ OTHER_AUDIT as SET_AUDIT }"),
            false,
        ),
        (
            "barrel",
            DISPATCH.replace("'./legacy.actions'", "'./barrel'"),
            false,
        ),
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let actions = root.path().join("legacy.actions.ts");
        let dispatch = root.path().join(format!("{name}.component.ts"));
        std::fs::write(&actions, ACTIONS).expect("actions fixture");
        if name == "barrel" {
            std::fs::write(
                root.path().join("barrel.ts"),
                "export { SET_AUDIT } from './legacy.actions';",
            )
            .expect("barrel fixture");
        }
        std::fs::write(&dispatch, dispatch_source).expect("dispatch fixture");
        let state = state(root.path());
        publish(&state, root.path(), &actions);
        publish(&state, root.path(), &dispatch);
        assert_eq!(
            !dispatch_targets(&state, root.path()).is_empty(),
            expected,
            "{name}"
        );
    }
}

#[test]
fn multiple_direct_imports_resolve_independently() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let audit = root.path().join("audit.ts");
    let theme = root.path().join("theme.ts");
    let dispatch = root.path().join("dispatch.component.ts");
    std::fs::write(&audit, ACTIONS).expect("audit action");
    std::fs::write(
        &theme,
        ACTIONS
            .replace("SET_AUDIT", "SET_THEME")
            .replace("auditReducer", "themeReducer"),
    )
    .expect("theme action");
    let source = DISPATCH
        .replace("'./legacy.actions'", "'./audit'")
        .replace(
            "@Component",
            "import { SET_THEME } from './theme';\n@Component",
        )
        .replace(
            "this.store.dispatch({type: SET_AUDIT, payload: 1});",
            "this.store.dispatch({type: SET_AUDIT, payload: 1}); this.store.dispatch({type: SET_THEME});",
        );
    std::fs::write(&dispatch, source).expect("dispatch fixture");
    let state = state(root.path());
    for path in [&dispatch, &theme, &audit] {
        publish(&state, root.path(), path);
    }
    let mut targets = dispatch_targets(&state, root.path());
    targets.sort();
    assert_eq!(targets, ["SET_AUDIT", "SET_THEME"]);
}

#[test]
fn same_name_files_and_multiple_actions_resolve_by_exact_module() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let first = root.path().join("first.ts");
    let second = root.path().join("second.ts");
    let dispatch = root.path().join("dispatch.component.ts");
    std::fs::write(&first, ACTIONS).expect("first action");
    std::fs::write(&second, ACTIONS.replace("auditReducer", "otherReducer"))
        .expect("second action");
    std::fs::write(
        &dispatch,
        DISPATCH.replace("'./legacy.actions'", "'./second'"),
    )
    .expect("dispatch fixture");
    let state = state(root.path());
    for path in [&first, &second, &dispatch] {
        publish(&state, root.path(), path);
    }
    assert_eq!(dispatch_targets(&state, root.path()), ["SET_AUDIT"]);
    let edge = call(
        &state,
        "workspace_query",
        json!({"workspaceRoot":root.path(),"type":"forward_edges","domain":"angular","entity_type":"Component","name":"DispatchProbe"}),
    )["edges"]
        .as_array()
        .unwrap()
        .iter()
        .find(|edge| edge["relation"] == "Dispatches")
        .unwrap()
        .clone();
    assert_eq!(edge["object"]["file"], second.to_string_lossy().as_ref());
}

#[test]
fn destination_authority_and_file_order_converge_through_refresh() {
    let _serial = crate::protocol::handler_response_serial();
    for dispatch_first in [false, true] {
        let root = tempfile::tempdir().expect("workspace");
        let actions = root.path().join("legacy.actions.ts");
        let dispatch = root.path().join("dispatch.component.ts");
        std::fs::write(&actions, ACTIONS).expect("actions fixture");
        std::fs::write(&dispatch, DISPATCH).expect("dispatch fixture");
        let state = state(root.path());
        let order = if dispatch_first {
            [&dispatch, &actions]
        } else {
            [&actions, &dispatch]
        };
        for path in order {
            publish(&state, root.path(), path);
        }
        assert_eq!(dispatch_targets(&state, root.path()), ["SET_AUDIT"]);

        std::fs::write(
            &dispatch,
            DISPATCH.replace("'./legacy.actions'", "'./fake.actions'"),
        )
        .expect("retarget import");
        std::fs::write(
            root.path().join("fake.actions.ts"),
            "export const SET_AUDIT = '[Audit] Set';",
        )
        .expect("ordinary target");
        refresh(&state, root.path());
        for path in [&actions, &dispatch] {
            publish(&state, root.path(), path);
        }
        assert!(dispatch_targets(&state, root.path()).is_empty());

        std::fs::write(&dispatch, DISPATCH).expect("restore import");
        refresh(&state, root.path());
        for path in [&dispatch, &actions] {
            publish(&state, root.path(), path);
        }
        assert_eq!(dispatch_targets(&state, root.path()), ["SET_AUDIT"]);

        std::fs::write(
            &actions,
            ACTIONS.replace("case SET_AUDIT: return state;", ""),
        )
        .expect("remove authority");
        refresh(&state, root.path());
        for path in [&dispatch, &actions] {
            publish(&state, root.path(), path);
        }
        assert!(dispatch_targets(&state, root.path()).is_empty());

        std::fs::write(&actions, ACTIONS).expect("restore authority");
        refresh(&state, root.path());
        for path in [&actions, &dispatch] {
            publish(&state, root.path(), path);
        }
        assert_eq!(dispatch_targets(&state, root.path()), ["SET_AUDIT"]);

        std::fs::write(
            &actions,
            ACTIONS
                .replace("SET_AUDIT", "SET_RENAMED")
                .replace("auditReducer", "renamedReducer"),
        )
        .expect("rename destination");
        std::fs::write(&dispatch, DISPATCH.replace("SET_AUDIT", "SET_RENAMED"))
            .expect("rename import and use");
        refresh(&state, root.path());
        for path in [&actions, &dispatch] {
            publish(&state, root.path(), path);
        }
        assert_eq!(dispatch_targets(&state, root.path()), ["SET_RENAMED"]);

        std::fs::remove_file(&dispatch).expect("delete dispatch");
        refresh(&state, root.path());
        assert!(dispatch_targets(&state, root.path()).is_empty());

        std::fs::write(&dispatch, DISPATCH).expect("restore dispatch");
        std::fs::write(&actions, ACTIONS).expect("restore destination");
        refresh(&state, root.path());
        for path in [&dispatch, &actions] {
            publish(&state, root.path(), path);
        }

        std::fs::remove_file(&actions).expect("delete destination");
        refresh(&state, root.path());
        publish(&state, root.path(), &dispatch);
        assert!(dispatch_targets(&state, root.path()).is_empty());
    }
}

#[test]
fn import_refinement_advances_only_angular_semantics() {
    assert_eq!(
        crate::layers::meta::ANGULAR_SEMANTIC_PRODUCER_GENERATION,
        15
    );
    assert_eq!(crate::layers::meta::ANGULAR_MARKER_PRODUCER_GENERATION, 5);
    assert_eq!(crate::ir::pipeline::CANONICAL_PIPELINE_GENERATION, 1);
}
