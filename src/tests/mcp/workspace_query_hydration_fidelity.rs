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
    let canonical = state.semantic_owner_path(&path.to_string_lossy());
    let source = state
        .read_source(&path.to_string_lossy())
        .expect("fixture source");
    let source_hash = state.cache_read().compute_hash(source.as_bytes());
    state
        .workspace_index_read()
        .has_current_semantic_projection(&canonical, fidelity, &source_hash)
}

#[cfg(any(windows, unix))]
fn assert_hard_link_provide_and_hydration_publish_one_alias_and_occurrence() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let owner = root.path().join("consumer.service.ts");
    let alternate = root.path().join("consumer.alias.ts");
    std::fs::write(&owner, SOURCE).expect("fixture");
    std::fs::hard_link(&owner, &alternate).expect("hard-link fixture");
    let provide_state = state();
    let root_text = root.path().to_string_lossy().into_owned();
    let owner_text = owner.to_string_lossy().into_owned();
    let alternate_text = alternate.to_string_lossy().into_owned();

    for (id, file) in [(10, &owner_text), (11, &alternate_text)] {
        crate::protocol::captured_responses().clear();
        dispatch_tools_call(
            &json!(id),
            "provide_code_context",
            &json!({ "arguments": {
                "filePath": file,
                "workspaceRoot": root_text.clone(),
                "fidelity": "low"
            }}),
            &provide_state,
        );
        let provided = response();
        assert!(provided.get("error").is_none(), "{provided}");
    }

    assert_eq!(
        provide_state.alias_for_path(&owner_text),
        provide_state.alias_for_path(&alternate_text),
        "provide calls must share one session alias"
    );

    let hydration_state = state();
    let hydrated = find_consumer(&hydration_state, root.path());
    assert!(hydrated.get("error").is_none(), "{hydrated}");
    let entities = hydrated["result"]["structuredContent"]["entities"]
        .as_array()
        .expect("entity results");
    let consumers: Vec<_> = entities
        .iter()
        .filter(|entity| entity["name"] == "Consumer")
        .collect();
    assert!(
        !consumers.is_empty(),
        "Consumer must be published: {hydrated}"
    );
    let owner_files: std::collections::HashSet<_> = consumers
        .iter()
        .filter_map(|entity| entity["file"].as_str())
        .collect();
    assert_eq!(
        owner_files.len(),
        1,
        "hard-link discovery must publish one physical-owner spelling: {hydrated}"
    );
    let occurrence_keys: std::collections::HashSet<_> = consumers
        .iter()
        .map(|entity| {
            (
                entity["domain"].as_str(),
                entity["entity_type"].as_str(),
                entity["name"].as_str(),
                entity["file"].as_str(),
            )
        })
        .collect();
    assert_eq!(
        occurrence_keys.len(),
        consumers.len(),
        "hard-link discovery must not duplicate a semantic occurrence: {hydrated}"
    );
}

#[cfg(windows)]
#[test]
fn windows_hard_link_provide_and_hydration_publish_one_alias_and_occurrence() {
    assert_hard_link_provide_and_hydration_publish_one_alias_and_occurrence();
}

#[cfg(unix)]
#[test]
fn unix_hard_link_provide_and_hydration_publish_one_alias_and_occurrence() {
    assert_hard_link_provide_and_hydration_publish_one_alias_and_occurrence();
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
