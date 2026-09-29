//! RED batch-contract regressions for heterogeneous `workspace_query` calls.
//!
//! These tests intentionally exercise the registered MCP boundary. They must
//! fail behaviorally against the legacy single-query-only handler, then remain
//! byte-identical while the production batch path is implemented.

use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::mcp::tools::{dispatch_tools_call, tool_list};
use crate::tests::assert_valid_mcp_envelope;
use serde_json::{Value, json};

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

fn batch_results(response: &Value) -> &Vec<Value> {
    let result = response["result"].as_object().expect("batch MCP result");
    assert_valid_mcp_envelope(result);
    response["result"]["structuredContent"]["results"]
        .as_array()
        .expect("ordered batch results")
}

fn seeded_state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let a_file = root.path().join("A.rs").to_string_lossy().into_owned();
    let b_file = root.path().join("B.rs").to_string_lossy().into_owned();
    std::fs::write(&a_file, "struct A;").expect("A fixture");
    std::fs::write(&b_file, "struct B;").expect("B fixture");
    state.workspace_index_lock().add_edges(
        &a_file,
        vec![SemanticEdge {
            relation: SemanticRelation::ImportsModule,
            subject: EntityRef::new("builtin", "Class", "A").with_file(a_file.clone()),
            object: EntityRef::new("builtin", "Class", "B").with_file(b_file.clone()),
            layer: "test",
            call_evidence: None,
        }],
    );
    state.workspace_index_lock().add_edges(
        &b_file,
        vec![SemanticEdge {
            relation: SemanticRelation::ImportsModule,
            subject: EntityRef::new("builtin", "Class", "B").with_file(b_file.clone()),
            object: EntityRef::new("builtin", "Class", "A").with_file(a_file),
            layer: "test",
            call_evidence: None,
        }],
    );
    state
}

fn exact_edge_item(id: &str, query_type: &str, name: &str) -> Value {
    json!({
        "id": id,
        "type": query_type,
        "domain": "builtin",
        "entity_type": "Class",
        "name": name,
    })
}

#[test]
fn red_batch_heterogeneous_operations_preserve_order_and_result_boundaries() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = seeded_state(&root);
    let response = dispatch(
        &state,
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                exact_edge_item("outgoing", "forward_edges", "A"),
                exact_edge_item("incoming", "reverse_edges", "A"),
                { "id": "cycle", "type": "has_cycle" },
            ],
        }),
    );

    let results = batch_results(&response);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0]["id"], "outgoing");
    assert_eq!(results[0]["type"], "forward_edges");
    assert_eq!(results[0]["status"], "ok");
    assert_eq!(results[0]["result"]["count"], 1);
    assert_eq!(results[1]["id"], "incoming");
    assert_eq!(results[1]["type"], "reverse_edges");
    assert_eq!(results[1]["status"], "ok");
    assert_eq!(results[1]["result"]["count"], 1);
    assert_eq!(results[2]["id"], "cycle");
    assert_eq!(results[2]["type"], "has_cycle");
    assert_eq!(results[2]["status"], "ok");
    assert_eq!(results[2]["result"]["has_cycle"], true);
}

#[test]
fn red_batch_item_failure_is_isolated_between_successful_siblings() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = seeded_state(&root);
    let response = dispatch(
        &state,
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                exact_edge_item("before", "forward_edges", "A"),
                { "id": "bad-cycle", "type": "has_cycle", "kind": "all" },
                exact_edge_item("after", "reverse_edges", "A"),
            ],
        }),
    );

    assert!(response.get("error").is_none(), "{response}");
    let results = batch_results(&response);
    assert_eq!(results[0]["status"], "ok");
    assert_eq!(results[1]["status"], "error");
    assert_eq!(results[1]["error"]["code"], -32602);
    assert_eq!(results[2]["status"], "ok");
}

#[test]
fn red_batch_item_validation_error_does_not_become_empty_success() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = seeded_state(&root);
    let response = dispatch(
        &state,
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                exact_edge_item("valid", "forward_edges", "A"),
                { "id": "missing-name", "type": "find_entities" },
            ],
        }),
    );

    let results = batch_results(&response);
    assert_eq!(results[0]["status"], "ok");
    assert_eq!(results[1]["status"], "error");
    assert!(results[1].get("result").is_none(), "{response}");
    assert_eq!(results[1]["error"]["code"], -32602);
}

#[test]
fn red_batch_shared_scope_rejection_happens_before_item_results() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let outside = tempfile::tempdir().expect("outside");
    let state = seeded_state(&root);
    let response = dispatch(
        &state,
        json!({
            "workspaceRoot": root.path(),
            "withinPath": outside.path(),
            "queries": [exact_edge_item("q", "forward_edges", "A")],
        }),
    );

    assert_eq!(response["error"]["code"], -32602, "{response}");
    assert!(
        response["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("withinPath")),
        "the shared-scope authority must classify the rejection: {response}"
    );
    assert!(response.get("result").is_none(), "{response}");
}

#[test]
fn red_batch_structure_validation_rejects_the_whole_request() {
    let _serial = crate::protocol::handler_response_serial();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let thirty_three: Vec<_> = (0..33)
        .map(|index| json!({ "id": format!("q{index}"), "type": "has_cycle" }))
        .collect();
    let cases = [
        (json!({ "queries": [] }), "non-empty"),
        (json!({ "queries": {} }), "array"),
        (json!({ "queries": [{ "type": "has_cycle" }] }), "id"),
        (
            json!({ "queries": [{ "id": "", "type": "has_cycle" }] }),
            "id",
        ),
        (
            json!({ "queries": [{ "id": "same", "type": "has_cycle" }, { "id": "same", "type": "has_cycle" }] }),
            "duplicate",
        ),
        (
            json!({ "type": "has_cycle", "queries": [{ "id": "q", "type": "has_cycle" }] }),
            "mutually exclusive",
        ),
        (
            json!({ "queries": [{ "id": "q", "type": "has_cycle", "workspaceRoot": "item-scope" }] }),
            "workspaceRoot",
        ),
        (json!({ "queries": thirty_three }), "32"),
    ];

    for (arguments, reason) in cases {
        let response = dispatch(&state, arguments);
        assert_eq!(response["error"]["code"], -32602, "{response}");
        assert!(
            response["error"]["message"]
                .as_str()
                .is_some_and(|message| message.contains(reason)),
            "batch rejection must report {reason:?}: {response}"
        );
        assert!(response.get("result").is_none(), "{response}");
    }
}

#[test]
fn red_batch_schema_declares_heterogeneous_input_and_ordered_outcomes() {
    let workspace_query = tool_list()
        .into_iter()
        .find(|tool| tool["name"] == "workspace_query")
        .expect("workspace_query tool");
    let input = &workspace_query["inputSchema"];
    let branches = input["oneOf"].as_array().expect("request-form union");
    let batch = branches
        .iter()
        .find(|branch| {
            branch["required"]
                .as_array()
                .is_some_and(|required| required.iter().any(|field| field == "queries"))
        })
        .expect("batch request branch");
    let items = &batch["properties"]["queries"]["items"];
    assert!(items["properties"].get("id").is_some());
    assert!(items["oneOf"].as_array().is_some());

    let results = &workspace_query["outputSchema"]["properties"]["results"];
    assert_eq!(results["type"], "array");
    let item = &results["items"];
    for field in ["id", "type", "status", "result", "error"] {
        assert!(item["properties"].get(field).is_some(), "missing {field}");
    }
}

#[test]
fn red_batch_content_uses_a_distinct_authoritative_envelope() {
    let _serial = crate::protocol::handler_response_serial();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let response = dispatch(
        &state,
        json!({ "queries": [{ "id": "cycle", "type": "has_cycle" }] }),
    );
    let result = response["result"].as_object().expect("batch MCP result");
    assert_valid_mcp_envelope(result);
    let text = result["content"][0]["text"]
        .as_str()
        .expect("batch text content");
    assert!(
        text.starts_with("// WORKSPACE-QUERY-BATCH v1; structuredContent remains authoritative")
    );
    assert!(text.contains("clean-ctx/workspace-query-batch-answer"));
    assert_eq!(result["structuredContent"]["batch"], true);
}
