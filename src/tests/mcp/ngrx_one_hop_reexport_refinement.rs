//! Public MCP RED contract for exact one-hop NgRx named re-exports.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

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

#[test]
fn one_hop_named_reexports_refine_actions_and_selectors_to_original_declarations() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let action = root.path().join("actions.ts");
    let selector = root.path().join("selectors.ts");
    let barrel = root.path().join("barrel.ts");
    let consumer = root.path().join("consumer.ts");
    std::fs::write(
        &action,
        "import { createAction } from '@ngrx/store';\n\
         export const saveThing = createAction('[Thing] Save');",
    )
    .expect("action");
    std::fs::write(
        &selector,
        "import { createSelector } from '@ngrx/store';\n\
         const state = (value: unknown) => value;\n\
         export const selectThing = createSelector(state, value => value);",
    )
    .expect("selector");
    std::fs::write(
        &barrel,
        "import { saveThing } from './actions';\n\
         import { selectThing } from './selectors';\n\
         export { saveThing, selectThing };",
    )
    .expect("barrel");
    std::fs::write(
        &consumer,
        "import { Component } from '@angular/core';\n\
         import { Store, select } from '@ngrx/store';\n\
         import { saveThing, selectThing } from './barrel';\n\
         @Component({selector: 'probe', template: ''})\n\
         export class ProbeComponent {\n\
           selected = this.store.pipe(select(selectThing));\n\
           constructor(private store: Store<unknown>) {}\n\
           save() { this.store.dispatch(saveThing()); }\n\
         }",
    )
    .expect("consumer");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for path in [&consumer, &barrel, &selector, &action] {
        publish(&state, root.path(), path);
    }
    let result = call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot": root.path(), "type": "forward_edges",
            "domain": "angular", "entity_type": "Component",
            "name": "ProbeComponent", "file_path": consumer
        }),
    );
    let edges = result["edges"].as_array().expect("edges");
    assert!(
        edges.iter().any(|edge| {
            edge["relation"] == "Dispatches"
                && edge["object"]["name"] == "saveThing"
                && edge["object"]["file"] == json!(action.to_string_lossy())
        }),
        "{edges:?}"
    );
    assert!(
        edges.iter().any(|edge| {
            edge["relation"] == "Selects"
                && edge["object"]["name"] == "selectThing"
                && edge["object"]["file"] == json!(selector.to_string_lossy())
        }),
        "{edges:?}"
    );
}

fn selector_edges_for_barrel(
    barrel_source: &str,
    selector_source: &str,
    inner_source: Option<&str>,
) -> Vec<Value> {
    let root = tempfile::tempdir().expect("workspace");
    let selector = root.path().join("selectors.ts");
    let barrel = root.path().join("barrel.ts");
    let consumer = root.path().join("consumer.ts");
    std::fs::write(&selector, selector_source).expect("selector");
    std::fs::write(&barrel, barrel_source).expect("barrel");
    if let Some(inner) = inner_source {
        std::fs::write(root.path().join("inner.ts"), inner).expect("inner");
    }
    std::fs::write(
        &consumer,
        "import { Component } from '@angular/core';\n\
         import { Store, select } from '@ngrx/store';\n\
         import { selectThing } from './barrel';\n\
         @Component({selector: 'probe', template: ''})\n\
         export class ProbeComponent {\n\
           selected = this.store.pipe(select(selectThing));\n\
           constructor(private store: Store<unknown>) {}\n\
         }",
    )
    .expect("consumer");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for path in [&consumer, &barrel, &selector] {
        publish(&state, root.path(), path);
    }
    if inner_source.is_some() {
        publish(&state, root.path(), &root.path().join("inner.ts"));
    }
    call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot": root.path(), "type": "forward_edges",
            "domain": "angular", "entity_type": "Component",
            "name": "ProbeComponent", "file_path": consumer
        }),
    )["edges"]
        .as_array()
        .expect("edges")
        .iter()
        .filter(|edge| edge["relation"] == "Selects")
        .cloned()
        .collect()
}

#[test]
fn unsupported_reexport_shapes_and_destinations_fail_closed() {
    let _serial = crate::protocol::handler_response_serial();
    let selector = "import { createSelector } from '@ngrx/store';\n\
                    const state = (value: unknown) => value;\n\
                    export const selectThing = createSelector(state, value => value);";
    for (case, barrel, destination, inner) in [
        (
            "import-alias",
            "import { selectThing as chosen } from './selectors'; export { chosen as selectThing };",
            selector,
            None,
        ),
        (
            "export-alias",
            "import { selectThing } from './selectors'; export { selectThing as chosen };",
            selector,
            None,
        ),
        ("wildcard", "export * from './selectors';", selector, None),
        (
            "ordinary-destination",
            "import { selectThing } from './selectors'; export { selectThing };",
            "export const selectThing = 123;",
            None,
        ),
        (
            "second-hop",
            "import { selectThing } from './inner'; export { selectThing };",
            selector,
            Some("import { selectThing } from './selectors'; export { selectThing };"),
        ),
        (
            "ambiguous-import",
            "import { selectThing } from './selectors';\n\
             import { selectThing } from './inner';\n\
             export { selectThing };",
            selector,
            Some("export const selectThing = 123;"),
        ),
    ] {
        assert!(
            selector_edges_for_barrel(barrel, destination, inner).is_empty(),
            "{case}",
        );
    }
}
