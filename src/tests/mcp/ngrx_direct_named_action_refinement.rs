use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

const ACTION: &str = r#"
import { createAction } from '@ngrx/store';
export const saveThing = createAction('[Thing] Save');
"#;

const CONSUMER: &str = r#"
import { Component } from '@angular/core';
import { Store, createReducer, on } from '@ngrx/store';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { saveThing } from './actions';
@Component({selector: 'probe', template: ''})
export class ProbeComponent {
  effect = createEffect(() => this.actions.pipe(ofType(saveThing)));
  constructor(private store: Store<unknown>, private actions: Actions) {}
  send() { this.store.dispatch(saveThing()); }
}
export const reducer = createReducer({}, on(saveThing, state => state));
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

fn edges(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    domain: &str,
    entity_type: &str,
    name: &str,
    file: Option<&std::path::Path>,
) -> Vec<Value> {
    let mut arguments = json!({
        "workspaceRoot": root,
        "type": "forward_edges",
        "domain": domain,
        "entity_type": entity_type,
        "name": name
    });
    if let Some(file) = file {
        arguments["file_path"] = json!(file);
    }
    call(state, "workspace_query", arguments)["edges"]
        .as_array()
        .expect("edges")
        .clone()
}

#[test]
fn all_three_relations_use_the_exact_action_declaration_file() {
    let _serial = crate::protocol::handler_response_serial();
    for consumer_first in [false, true] {
        let root = tempfile::tempdir().expect("workspace");
        let action = root.path().join("actions.ts");
        let consumer = root.path().join("consumer.ts");
        std::fs::write(&action, ACTION).expect("action");
        std::fs::write(&consumer, CONSUMER).expect("consumer");
        let state = state(root.path());
        let order = if consumer_first {
            [&consumer, &action]
        } else {
            [&action, &consumer]
        };
        for path in order {
            publish(&state, root.path(), path);
        }
        let expected = action.to_string_lossy();
        let dispatches = edges(
            &state,
            root.path(),
            "angular",
            "Component",
            "ProbeComponent",
            Some(&consumer),
        );
        let handles = edges(
            &state,
            root.path(),
            "ngrx",
            "Effect",
            "effect",
            Some(&consumer),
        );
        let triggers = edges(
            &state,
            root.path(),
            "ngrx",
            "Action",
            "saveThing",
            Some(&action),
        );
        assert!(dispatches.iter().any(|edge| {
            edge["relation"] == "Dispatches" && edge["object"]["file"] == json!(expected.as_ref())
        }));
        assert!(handles.iter().any(|edge| {
            edge["relation"] == "HandlesAction"
                && edge["object"]["file"] == json!(expected.as_ref())
        }));
        assert!(
            triggers
                .iter()
                .any(|edge| edge["relation"] == "TriggersReducer")
        );
    }
}

#[test]
fn ordinary_destination_and_unsupported_imports_never_gain_action_authority() {
    let _serial = crate::protocol::handler_response_serial();
    for (name, declaration, consumer) in [
        (
            "ordinary",
            "export const saveThing = 123;",
            CONSUMER.to_owned(),
        ),
        (
            "missing-export",
            ACTION,
            CONSUMER.replace("{ saveThing }", "{ missing as saveThing }"),
        ),
        (
            "alias",
            ACTION,
            CONSUMER
                .replace("{ saveThing }", "{ saveThing as save }")
                .replace("saveThing())", "save())")
                .replace("saveThing, state", "save, state")
                .replace("ofType(saveThing)", "ofType(save)"),
        ),
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let action = root.path().join("actions.ts");
        let consumer_path = root.path().join(format!("{name}.ts"));
        std::fs::write(&action, declaration).expect("declaration");
        std::fs::write(&consumer_path, consumer).expect("consumer");
        let state = state(root.path());
        publish(&state, root.path(), &action);
        publish(&state, root.path(), &consumer_path);
        assert!(
            edges(
                &state,
                root.path(),
                "ngrx",
                "Action",
                "saveThing",
                Some(&action)
            )
            .is_empty(),
            "{name}"
        );
    }
}

#[test]
fn authority_revoke_restore_and_import_retarget_rebuild_all_three_relations() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let action = root.path().join("actions.ts");
    let alternate = root.path().join("alternate.ts");
    let consumer = root.path().join("consumer.ts");
    std::fs::write(&action, ACTION).expect("action");
    std::fs::write(&alternate, "export const saveThing = 123;").expect("alternate");
    std::fs::write(&consumer, CONSUMER).expect("consumer");
    let state = state(root.path());
    for path in [&consumer, &action, &alternate] {
        publish(&state, root.path(), path);
    }
    assert_eq!(
        edges(
            &state,
            root.path(),
            "angular",
            "Component",
            "ProbeComponent",
            Some(&consumer),
        )
        .iter()
        .filter(|edge| edge["relation"] == "Dispatches")
        .count(),
        1
    );

    std::fs::write(&consumer, CONSUMER.replace("'./actions'", "'./alternate'"))
        .expect("retarget consumer");
    refresh(&state, root.path());
    for path in [&action, &alternate, &consumer] {
        publish(&state, root.path(), path);
    }
    assert!(
        edges(
            &state,
            root.path(),
            "angular",
            "Component",
            "ProbeComponent",
            Some(&consumer),
        )
        .iter()
        .all(|edge| edge["relation"] != "Dispatches")
    );

    std::fs::write(&consumer, CONSUMER).expect("restore consumer");
    std::fs::write(&action, "export const saveThing = 123;").expect("revoke authority");
    refresh(&state, root.path());
    for path in [&consumer, &action] {
        publish(&state, root.path(), path);
    }
    assert!(
        edges(
            &state,
            root.path(),
            "angular",
            "Component",
            "ProbeComponent",
            Some(&consumer),
        )
        .iter()
        .all(|edge| edge["relation"] != "Dispatches")
    );

    std::fs::write(&action, ACTION).expect("restore authority");
    refresh(&state, root.path());
    for path in [&action, &consumer] {
        publish(&state, root.path(), path);
    }
    assert_eq!(
        edges(
            &state,
            root.path(),
            "angular",
            "Component",
            "ProbeComponent",
            Some(&consumer),
        )
        .iter()
        .filter(|edge| edge["relation"] == "Dispatches")
        .count(),
        1
    );
}

#[test]
fn namespace_members_refine_all_three_relations_to_the_exact_action_declaration() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let action = root.path().join("actions.ts");
    let consumer = root.path().join("namespace-consumer.ts");
    let namespace_consumer = CONSUMER
        .replace("{ saveThing }", "* as ActionsApi")
        .replace("saveThing())", "ActionsApi.saveThing())")
        .replace("saveThing, state", "ActionsApi.saveThing, state")
        .replace("ofType(saveThing)", "ofType(ActionsApi.saveThing)");
    std::fs::write(&action, ACTION).expect("action");
    std::fs::write(&consumer, namespace_consumer).expect("consumer");
    let state = state(root.path());
    publish(&state, root.path(), &consumer);
    publish(&state, root.path(), &action);

    let expected = json!(action.to_string_lossy().as_ref());
    let dispatches = edges(
        &state,
        root.path(),
        "angular",
        "Component",
        "ProbeComponent",
        Some(&consumer),
    );
    let handles = edges(
        &state,
        root.path(),
        "ngrx",
        "Effect",
        "effect",
        Some(&consumer),
    );
    let triggers = edges(
        &state,
        root.path(),
        "ngrx",
        "Action",
        "saveThing",
        Some(&action),
    );
    assert!(dispatches.iter().any(|edge| {
        edge["relation"] == "Dispatches"
            && edge["object"]["name"] == "saveThing"
            && edge["object"]["file"] == expected
    }));
    assert!(handles.iter().any(|edge| {
        edge["relation"] == "HandlesAction"
            && edge["object"]["name"] == "saveThing"
            && edge["object"]["file"] == expected
    }));
    assert!(triggers.iter().any(|edge| {
        edge["relation"] == "TriggersReducer"
            && edge["subject"]["name"] == "saveThing"
            && edge["subject"]["file"] == expected
    }));
}

fn namespace_consumer(module: &str) -> String {
    CONSUMER
        .replace("{ saveThing }", "* as ActionsApi")
        .replace("'./actions'", &format!("'{module}'"))
        .replace("saveThing())", "ActionsApi.saveThing())")
        .replace("saveThing, state", "ActionsApi.saveThing, state")
        .replace("ofType(saveThing)", "ofType(ActionsApi.saveThing)")
}

#[test]
fn namespace_member_resolution_is_exact_and_unsupported_authority_fails_closed() {
    let _serial = crate::protocol::handler_response_serial();
    for (case, destination, consumer) in [
        (
            "ordinary",
            "export const saveThing = 123;",
            namespace_consumer("./actions"),
        ),
        (
            "missing",
            "export const anotherThing = 123;",
            namespace_consumer("./actions"),
        ),
        (
            "barrel",
            "export { saveThing } from './real-actions';",
            namespace_consumer("./actions"),
        ),
        (
            "shadow",
            ACTION,
            namespace_consumer("./actions")
                .replace("send() {", "send(ActionsApi: { saveThing(): unknown }) {"),
        ),
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let action = root.path().join("actions.ts");
        let consumer_path = root.path().join(format!("{case}.ts"));
        std::fs::write(&action, destination).expect("destination");
        std::fs::write(&consumer_path, consumer).expect("consumer");
        if case == "barrel" {
            std::fs::write(root.path().join("real-actions.ts"), ACTION).expect("real action");
        }
        let state = state(root.path());
        refresh(&state, root.path());
        publish(&state, root.path(), &action);
        if case == "barrel" {
            publish(&state, root.path(), &root.path().join("real-actions.ts"));
        }
        publish(&state, root.path(), &consumer_path);
        assert!(
            edges(
                &state,
                root.path(),
                "angular",
                "Component",
                "ProbeComponent",
                Some(&consumer_path),
            )
            .iter()
            .all(|edge| edge["relation"] != "Dispatches"),
            "{case}"
        );
    }

    let root = tempfile::tempdir().expect("workspace");
    let first = root.path().join("first.ts");
    let second = root.path().join("second.ts");
    let consumer = root.path().join("consumer.ts");
    std::fs::write(&first, ACTION).expect("first");
    std::fs::write(&second, ACTION).expect("second");
    std::fs::write(&consumer, namespace_consumer("./second")).expect("consumer");
    let state = state(root.path());
    refresh(&state, root.path());
    for path in [&first, &second, &consumer] {
        publish(&state, root.path(), path);
    }
    let dispatches = edges(
        &state,
        root.path(),
        "angular",
        "Component",
        "ProbeComponent",
        Some(&consumer),
    );
    assert!(dispatches.iter().any(|edge| {
        edge["relation"] == "Dispatches"
            && edge["object"]["file"] == json!(second.to_string_lossy().as_ref())
    }));
    assert!(dispatches.iter().all(|edge| {
        edge["relation"] != "Dispatches"
            || edge["object"]["file"] != json!(first.to_string_lossy().as_ref())
    }));
}

#[test]
fn namespace_member_retarget_revoke_and_restore_rebuilds_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let action = root.path().join("actions.ts");
    let alternate = root.path().join("alternate.ts");
    let consumer = root.path().join("consumer.ts");
    std::fs::write(&action, ACTION).expect("action");
    std::fs::write(&alternate, "export const saveThing = 123;").expect("alternate");
    std::fs::write(&consumer, namespace_consumer("./actions")).expect("consumer");
    let state = state(root.path());
    refresh(&state, root.path());
    for path in [&action, &alternate, &consumer] {
        publish(&state, root.path(), path);
    }
    let dispatch_count = || {
        edges(
            &state,
            root.path(),
            "angular",
            "Component",
            "ProbeComponent",
            Some(&consumer),
        )
        .iter()
        .filter(|edge| edge["relation"] == "Dispatches")
        .count()
    };
    assert_eq!(dispatch_count(), 1);

    std::fs::write(&consumer, namespace_consumer("./alternate")).expect("retarget");
    refresh(&state, root.path());
    for path in [&action, &alternate, &consumer] {
        publish(&state, root.path(), path);
    }
    assert_eq!(dispatch_count(), 0);

    std::fs::write(&consumer, namespace_consumer("./actions")).expect("restore import");
    std::fs::write(&action, "export const saveThing = 123;").expect("revoke");
    refresh(&state, root.path());
    for path in [&action, &alternate, &consumer] {
        publish(&state, root.path(), path);
    }
    assert_eq!(dispatch_count(), 0);

    std::fs::write(&action, ACTION).expect("restore authority");
    refresh(&state, root.path());
    for path in [&action, &alternate, &consumer] {
        publish(&state, root.path(), path);
    }
    assert_eq!(dispatch_count(), 1);
}
