//! Public MCP RED contract for exact direct-relative Selector refinement.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

const SELECTOR: &str = r#"
import { createSelector } from '@ngrx/store';
const selectState = (state: unknown) => state;
export const selectThing = createSelector(selectState, state => state);
"#;

const CONSUMER: &str = r#"
import { Component } from '@angular/core';
import { Store, select } from '@ngrx/store';
import { selectThing } from './selectors';
@Component({selector: 'probe', template: ''})
export class ProbeComponent {
  constructor(private store: Store<unknown>) {}
  read() { return this.store.pipe(select(selectThing)); }
}
"#;

const SERVICE: &str = r#"
import { Injectable } from '@angular/core';
import { Store, select } from '@ngrx/store';
import { selectThing } from './selectors';
@Injectable()
export class ProbeService {
  constructor(private store: Store<unknown>) {}
  read() { return this.store.pipe(select(selectThing)); }
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

fn selects(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    consumer: &std::path::Path,
    entity_type: &str,
    name: &str,
) -> Vec<Value> {
    call(
        state,
        "workspace_query",
        json!({
            "workspaceRoot": root,
            "type": "forward_edges",
            "domain": "angular",
            "entity_type": entity_type,
            "name": name,
            "file_path": consumer
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
fn direct_named_selector_import_refines_to_the_exact_declaration() {
    let _serial = crate::protocol::handler_response_serial();
    for consumer_first in [false, true] {
        let root = tempfile::tempdir().expect("workspace");
        let selector = root.path().join("selectors.ts");
        let consumer = root.path().join("consumer.ts");
        std::fs::write(&selector, SELECTOR).expect("selector");
        std::fs::write(&consumer, CONSUMER).expect("consumer");
        let mut config = crate::tests::test_config();
        config
            .additional_roots
            .push(root.path().to_string_lossy().into_owned());
        let state = crate::mcp::McpState::new(config);
        let order = if consumer_first {
            [&consumer, &selector]
        } else {
            [&selector, &consumer]
        };
        for path in order {
            publish(&state, root.path(), path);
        }
        let edges = selects(
            &state,
            root.path(),
            &consumer,
            "Component",
            "ProbeComponent",
        );
        assert_eq!(edges.len(), 1, "{edges:?}");
        assert_eq!(edges[0]["object"]["name"], "selectThing");
        assert_eq!(
            edges[0]["object"]["file"],
            json!(selector.to_string_lossy())
        );
    }
}

#[test]
fn direct_named_selector_import_preserves_exact_service_ownership() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let selector = root.path().join("selectors.ts");
    let service = root.path().join("service.ts");
    std::fs::write(&selector, SELECTOR).expect("selector");
    std::fs::write(&service, SERVICE).expect("service");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    publish(&state, root.path(), &service);
    publish(&state, root.path(), &selector);
    let edges = selects(&state, root.path(), &service, "Service", "ProbeService");
    assert_eq!(edges.len(), 1, "{edges:?}");
    assert_eq!(
        edges[0]["object"]["file"],
        json!(selector.to_string_lossy())
    );
}

#[test]
fn unsupported_selector_imports_and_destinations_fail_closed() {
    let _serial = crate::protocol::handler_response_serial();
    for (case, destination, consumer) in [
        (
            "ordinary",
            "export const selectThing = 123;",
            CONSUMER.to_owned(),
        ),
        (
            "alias",
            SELECTOR,
            CONSUMER
                .replace("{ selectThing }", "{ selectThing as chosen }")
                .replace("select(selectThing)", "select(chosen)"),
        ),
        (
            "local-shadow",
            SELECTOR,
            CONSUMER.replace(
                "read() { return",
                "read() { const selectThing = 123; return",
            ),
        ),
        (
            "foreign-operator",
            SELECTOR,
            CONSUMER.replace(
                "import { Store, select } from '@ngrx/store';",
                "import { Store } from '@ngrx/store';\nimport { select } from './operators';",
            ),
        ),
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let selector = root.path().join("selectors.ts");
        let consumer_path = root.path().join("consumer.ts");
        std::fs::write(&selector, destination).expect("selector");
        std::fs::write(&consumer_path, consumer).expect("consumer");
        std::fs::write(
            root.path().join("operators.ts"),
            "export const select = (x: unknown) => x;",
        )
        .expect("operator");
        let mut config = crate::tests::test_config();
        config
            .additional_roots
            .push(root.path().to_string_lossy().into_owned());
        let state = crate::mcp::McpState::new(config);
        publish(&state, root.path(), &selector);
        publish(&state, root.path(), &consumer_path);
        assert!(
            selects(
                &state,
                root.path(),
                &consumer_path,
                "Component",
                "ProbeComponent",
            )
            .is_empty(),
            "{case}",
        );
    }

    let root = tempfile::tempdir().expect("workspace");
    let selector = root.path().join("selectors.ts");
    let barrel = root.path().join("index.ts");
    let consumer = root.path().join("consumer.ts");
    std::fs::write(&selector, SELECTOR).expect("selector");
    std::fs::write(&barrel, "export { selectThing } from './selectors';").expect("barrel");
    std::fs::write(&consumer, CONSUMER.replace("'./selectors'", "'./index'")).expect("consumer");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for path in [&selector, &barrel, &consumer] {
        publish(&state, root.path(), path);
    }
    assert!(
        selects(
            &state,
            root.path(),
            &consumer,
            "Component",
            "ProbeComponent",
        )
        .is_empty(),
        "barrels remain outside Slice 3",
    );
}

#[test]
fn selector_authority_retargets_revokes_restores_and_deletes() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let selector = root.path().join("selectors.ts");
    let alternate = root.path().join("alternate.ts");
    let consumer = root.path().join("consumer.ts");
    std::fs::write(&selector, SELECTOR).expect("selector");
    std::fs::write(&alternate, SELECTOR).expect("alternate");
    std::fs::write(&consumer, CONSUMER).expect("consumer");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for path in [&consumer, &selector, &alternate] {
        publish(&state, root.path(), path);
    }
    let query = || {
        selects(
            &state,
            root.path(),
            &consumer,
            "Component",
            "ProbeComponent",
        )
    };
    assert_eq!(
        query()[0]["object"]["file"],
        json!(selector.to_string_lossy())
    );

    std::fs::write(
        &consumer,
        CONSUMER.replace("'./selectors'", "'./alternate'"),
    )
    .expect("retarget");
    publish(&state, root.path(), &consumer);
    assert_eq!(
        query()[0]["object"]["file"],
        json!(alternate.to_string_lossy())
    );

    std::fs::write(&alternate, "export const selectThing = 123;").expect("revoke");
    publish(&state, root.path(), &alternate);
    assert!(query().is_empty());

    std::fs::write(&alternate, SELECTOR).expect("restore");
    publish(&state, root.path(), &alternate);
    assert_eq!(query().len(), 1);

    std::fs::remove_file(&alternate).expect("delete");
    call(
        &state,
        "refresh_workspace",
        json!({"workspaceRoot": root.path()}),
    );
    assert!(query().is_empty());
}
