//! Public MCP RED contract for exact modern Service dispatch ownership.

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

fn forward(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    path: &std::path::Path,
    entity_type: &str,
    name: &str,
) -> Vec<Value> {
    call(
        state,
        "workspace_query",
        json!({
            "workspaceRoot": root, "type": "forward_edges",
            "domain": "angular", "entity_type": entity_type,
            "name": name, "file_path": path
        }),
    )["edges"]
        .as_array()
        .expect("edges")
        .clone()
}

#[test]
fn nested_property_arrows_publish_exact_service_dispatch_without_fallback() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let actions = root.path().join("actions.ts");
    let service = root.path().join("service.ts");
    std::fs::write(
        &actions,
        "import { createAction } from '@ngrx/store';\n\
         export const changeHour = createAction('[Clock] Change Hour');",
    )
    .expect("actions");
    std::fs::write(
        &service,
        "import { Injectable } from '@angular/core';\n\
         import { Store } from '@ngrx/store';\n\
         import { changeHour } from './actions';\n\
         @Injectable()\n\
         export class ClockEffects {\n\
           tick = runOutside(() => setInterval(() => {\n\
             this.store.dispatch(changeHour());\n\
           }, 60_000));\n\
           constructor(private store: Store<unknown>) {}\n\
         }",
    )
    .expect("service");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    publish(&state, root.path(), &service);
    publish(&state, root.path(), &actions);

    let service_edges = forward(&state, root.path(), &service, "Service", "ClockEffects");
    assert!(
        service_edges.iter().any(|edge| {
            edge["relation"] == "Dispatches"
                && edge["object"]["name"] == "changeHour"
                && edge["object"]["file"] == json!(actions.to_string_lossy())
        }),
        "{service_edges:?}"
    );

    let fallback = forward(&state, root.path(), &service, "DispatchSite", "store");
    assert!(
        fallback.iter().all(|edge| edge["relation"] != "Dispatches"),
        "{fallback:?}",
    );
}

#[test]
fn property_initializer_service_dispatch_refuses_inexact_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let cases = [
        (
            "static-initializer",
            "import { Injectable } from '@angular/core';\n\
             import { Store } from '@ngrx/store';\n\
             import { changeHour } from './actions';\n\
             @Injectable() export class ClockEffects {\n\
               static tick = this.store.dispatch(changeHour());\n\
               constructor(private store: Store<unknown>) {}\n\
             }",
        ),
        (
            "undecorated-owner",
            "import { Store } from '@ngrx/store';\n\
             import { changeHour } from './actions';\n\
             export class ClockEffects {\n\
               tick = this.store.dispatch(changeHour());\n\
               constructor(private store: Store<unknown>) {}\n\
             }",
        ),
        (
            "foreign-store",
            "import { Injectable } from '@angular/core';\n\
             import { Store } from './foreign';\n\
             import { changeHour } from './actions';\n\
             @Injectable() export class ClockEffects {\n\
               tick = this.store.dispatch(changeHour());\n\
               constructor(private store: Store<unknown>) {}\n\
             }",
        ),
        (
            "other-class-property",
            "import { Injectable } from '@angular/core';\n\
             import { Store } from '@ngrx/store';\n\
             import { changeHour } from './actions';\n\
             class Other { constructor(private store: Store<unknown>) {} }\n\
             @Injectable() export class ClockEffects {\n\
               tick = this.store.dispatch(changeHour());\n\
             }",
        ),
        (
            "ordinary-function",
            "import { Injectable } from '@angular/core';\n\
             import { Store } from '@ngrx/store';\n\
             import { changeHour } from './actions';\n\
             @Injectable() export class ClockEffects {\n\
               tick = (function () { return this.store.dispatch(changeHour()); })();\n\
               constructor(private store: Store<unknown>) {}\n\
             }",
        ),
    ];

    for (label, source) in cases {
        let root = tempfile::tempdir().expect("workspace");
        let actions = root.path().join("actions.ts");
        let service = root.path().join("service.ts");
        std::fs::write(
            &actions,
            "import { createAction } from '@ngrx/store';\n\
             export const changeHour = createAction('[Clock] Change Hour');",
        )
        .expect("actions");
        std::fs::write(&service, source).expect("service");
        let mut config = crate::tests::test_config();
        config
            .additional_roots
            .push(root.path().to_string_lossy().into_owned());
        let state = crate::mcp::McpState::new(config);
        publish(&state, root.path(), &service);
        publish(&state, root.path(), &actions);

        let edges = forward(&state, root.path(), &service, "Service", "ClockEffects");
        assert!(
            edges.iter().all(|edge| edge["relation"] != "Dispatches"),
            "{label}: {edges:?}",
        );
    }
}
