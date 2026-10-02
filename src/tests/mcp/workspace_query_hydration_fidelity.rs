use crate::layers::meta::semantic::SemanticRelation;
use crate::mcp::McpState;
use crate::mcp::tools::dispatch_tools_call;
use crate::workspace::index::SemanticFidelity;
use serde_json::{Value, json};

const SOURCE: &str = r#"
import { Injectable } from '@angular/core';

@Injectable()
export class Consumer {
    constructor(private foo: FooService) {}
}
"#;

fn state() -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}

fn response() -> Value {
    crate::protocol::captured_responses()
        .pop()
        .expect("handler response")
}

fn query_entities(state: &McpState, root: &std::path::Path) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({
            "arguments": {
                "type": "entities_in_file",
                "file_path": "consumer.service.ts",
                "workspaceRoot": root.to_string_lossy(),
                "fidelity": "low"
            }
        }),
        state,
    );
    response()
}

fn query_consumer_edges(state: &McpState, root: &std::path::Path) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(2),
        "workspace_query",
        &json!({
            "arguments": {
                "type": "forward_edges",
                "domain": "angular",
                "entity_type": "Service",
                "name": "Consumer",
                "workspaceRoot": root.to_string_lossy()
            }
        }),
        state,
    );
    response()
}

fn find_consumer(state: &McpState, root: &std::path::Path) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(3),
        "workspace_query",
        &json!({
            "arguments": {
                "type": "find_entities",
                "name": "Consumer",
                "workspaceRoot": root.to_string_lossy()
            }
        }),
        state,
    );
    response()
}

fn assert_one_injects_foo(response: &Value) {
    assert!(
        response.get("error").is_none(),
        "forward_edges must succeed: {response}"
    );
    let edges = response["result"]["structuredContent"]["edges"]
        .as_array()
        .expect("edge array");
    let injects: Vec<_> = edges
        .iter()
        .filter(|edge| edge["relation"] == json!(SemanticRelation::Injects))
        .collect();
    assert_eq!(
        injects.len(),
        1,
        "expected exactly one Injects edge: {response}"
    );
    assert_eq!(injects[0]["subject"]["name"], "Consumer");
    assert_eq!(injects[0]["object"]["name"], "FooService");
}

fn current_at(state: &McpState, path: &std::path::Path, fidelity: SemanticFidelity) -> bool {
    let canonical = crate::dictionary::path::canonical_identity_key(&path.to_string_lossy());
    let source = state
        .read_source(&path.to_string_lossy())
        .expect("fixture source");
    let source_hash = state.cache_read().compute_hash(source.as_bytes());
    state
        .workspace_index_read()
        .has_current_semantic_projection(&canonical, fidelity, &source_hash)
}

#[test]
fn current_low_projection_upgrades_for_angular_injects_query() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("consumer.service.ts");
    std::fs::write(&path, SOURCE).expect("fixture");
    let state = state();

    let low = query_entities(&state, root.path());
    assert!(low.get("error").is_none(), "Low projection: {low}");
    assert!(current_at(&state, &path, SemanticFidelity::Low));
    assert!(!current_at(&state, &path, SemanticFidelity::High));

    let edges = query_consumer_edges(&state, root.path());
    assert_one_injects_foo(&edges);
    assert!(
        current_at(&state, &path, SemanticFidelity::High),
        "hydration must replace Low coverage with current High coverage"
    );
}

#[test]
fn initially_unindexed_angular_injects_query_compiles_at_high_coverage() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("consumer.service.ts");
    std::fs::write(&path, SOURCE).expect("fixture");
    let state = state();

    let edges = query_consumer_edges(&state, root.path());
    assert_one_injects_foo(&edges);
    assert!(current_at(&state, &path, SemanticFidelity::High));
}

#[test]
fn legacy_discovery_completion_does_not_suppress_high_angular_hydration() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("consumer.service.ts");
    std::fs::write(&path, SOURCE).expect("fixture");
    let state = state();

    let legacy = find_consumer(&state, root.path());
    assert!(
        legacy.get("error").is_none(),
        "legacy discovery must complete: {legacy}"
    );
    assert!(
        !current_at(&state, &path, SemanticFidelity::High),
        "legacy/Edit hydration must not claim High semantic coverage"
    );

    let edges = query_consumer_edges(&state, root.path());
    assert_one_injects_foo(&edges);
    assert!(
        current_at(&state, &path, SemanticFidelity::High),
        "the distinct High discovery entry must permit coverage upgrade"
    );
}
