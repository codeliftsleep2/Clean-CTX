//! RED preparation/snapshot regressions for heterogeneous workspace batches.

use crate::cbm::GraphBridge;
use crate::cbm::bridge::cbm_project_slug;
use crate::mcp::tool_handlers::hydration::{
    TestDiscoveryKind, TestProjectReadiness, clear_test_project_search_results, discovery_calls,
    set_test_inbound_project_search_results, set_test_project_readiness,
    set_test_project_search_results,
};
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;

static TEST_SERIALIZE: &std::sync::Mutex<()> =
    &crate::mcp::tool_handlers::hydration::TEST_PROJECT_HYDRATION_SERIALIZE;

fn state(root: &Path) -> (String, crate::mcp::McpState) {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create(&bridge_config, root));
    let project = cbm_project_slug(&root.canonicalize().expect("canonical root"));
    (project, state)
}

fn configure(project: &str) {
    set_test_project_search_results(HashMap::from([(project.to_string(), Ok(Vec::new()))]));
    set_test_inbound_project_search_results(HashMap::from([(project.to_string(), Ok(Vec::new()))]));
    set_test_project_readiness(HashMap::from([(
        project.to_string(),
        TestProjectReadiness::Ready,
    )]));
}

fn dispatch(state: &crate::mcp::McpState, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({ "arguments": arguments }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered workspace_query response")
}

fn results(response: &Value) -> &Vec<Value> {
    response["result"]["structuredContent"]["results"]
        .as_array()
        .expect("batch results")
}

#[test]
fn red_batch_deduplicates_equivalent_hydration_requirements() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let (project, state) = state(root.path());
    configure(&project);

    let response = dispatch(
        &state,
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                { "id": "find", "type": "find_entities", "name": "Shared" },
                { "id": "forward", "type": "forward_edges", "domain": "builtin", "entity_type": "Class", "name": "Shared" },
            ],
        }),
    );
    let calls = discovery_calls();
    clear_test_project_search_results();

    assert_eq!(results(&response).len(), 2, "{response}");
    assert_eq!(
        calls.len(),
        1,
        "equivalent declaration discovery: {calls:?}"
    );
    assert_eq!(calls[0].1, TestDiscoveryKind::Declaration);
}

#[test]
fn red_batch_runs_distinct_names_and_discovery_modes_independently() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let (project, state) = state(root.path());
    configure(&project);

    let response = dispatch(
        &state,
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                { "id": "find", "type": "find_entities", "name": "Alpha" },
                { "id": "reverse", "type": "reverse_edges", "domain": "builtin", "entity_type": "Class", "name": "Beta" },
                { "id": "cycle", "type": "has_cycle" },
            ],
        }),
    );
    let calls = discovery_calls();
    clear_test_project_search_results();

    assert_eq!(results(&response).len(), 3, "{response}");
    assert_eq!(calls.len(), 2, "distinct preparation keys: {calls:?}");
    assert!(
        calls
            .iter()
            .any(|(_, kind)| *kind == TestDiscoveryKind::Declaration)
    );
    assert!(
        calls
            .iter()
            .any(|(_, kind)| *kind == TestDiscoveryKind::InboundReference)
    );
}

#[test]
fn red_batch_index_queries_share_the_final_post_preparation_snapshot() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    std::fs::write(
        root.path().join("Snapshot.ts"),
        "export class SnapshotOnlyEntity {}\n",
    )
    .expect("fixture");
    let (project, state) = state(root.path());
    configure(&project);

    let response = dispatch(
        &state,
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                { "id": "find-first", "type": "find_entities", "name": "SnapshotOnlyEntity" },
                { "id": "prepare-second", "type": "entities_in_file", "file_path": "Snapshot.ts", "fidelity": "high" },
            ],
        }),
    );
    clear_test_project_search_results();

    let outcomes = results(&response);
    assert_eq!(outcomes[0]["status"], "ok", "{response}");
    assert_eq!(outcomes[0]["result"]["count"], 1, "{response}");
    assert_eq!(outcomes[1]["status"], "ok", "{response}");
    assert_eq!(outcomes[1]["result"]["count"], 1, "{response}");
}

#[test]
fn red_batch_calls_in_file_stays_fresh_and_unpublished() {
    let _response_serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("Worker.ts");
    std::fs::write(
        &path,
        "export class Worker { run(): void { target(); target(); } }\n",
    )
    .expect("fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let response = dispatch(
        &state,
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                { "id": "cycle", "type": "has_cycle" },
                {
                    "id": "calls",
                    "type": "calls_in_file",
                    "filePath": "Worker.ts",
                    "owner": { "kind": "class", "name": "Worker" },
                    "method": { "name": "run" }
                },
            ],
        }),
    );

    let outcomes = results(&response);
    assert_eq!(outcomes[0]["status"], "ok", "{response}");
    assert_eq!(outcomes[1]["status"], "ok", "{response}");
    assert_eq!(outcomes[1]["result"]["overload_count"], 1);
    assert_eq!(outcomes[1]["result"]["count"], 2);
    let canonical = crate::dictionary::path::canonical_identity_key(&path.to_string_lossy());
    assert!(
        state
            .workspace_index_read()
            .entities_in_file(&canonical)
            .is_empty(),
        "calls_in_file must not publish WorkspaceIndex facts"
    );
}
