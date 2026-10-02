//! Exact-identity zero-result truthfulness through registered MCP dispatch.

use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::mcp::tools::{dispatch_tools_call, tool_list};
use crate::tests::assert_valid_mcp_envelope;
use serde_json::{Value, json};

fn registration(entity_type: &'static str, name: &str, file: &str) -> SemanticEdge {
    let entity = EntityRef::new("builtin", entity_type, name).with_file(file.to_string());
    SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: entity.clone(),
        object: entity,
        layer: "builtin",
        call_evidence: None,
    }
}

fn constructor_edge(subject: &str, target: &str, file: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::HasConstructorParameterType,
        subject: EntityRef::new("builtin", "Class", subject).with_file(file.to_string()),
        object: EntityRef::new("builtin", "TypeRef", target).with_file(file.to_string()),
        layer: "builtin",
        call_evidence: None,
    }
}

fn state() -> crate::mcp::McpState {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let mut index = state.workspace_index_lock();
    index.add_edges(
        "contracts.cs",
        vec![
            registration("Interface", "IFooService", "contracts.cs"),
            registration("TypeRef", "IUnusedService", "contracts.cs"),
        ],
    );
    index.add_edges(
        "consumer.cs",
        vec![constructor_edge(
            "BarController",
            "IFooService",
            "consumer.cs",
        )],
    );
    drop(index);
    state
}

fn exact(name: &str, entity_type: &str) -> Value {
    json!({
        "type": "reverse_edges",
        "domain": "builtin",
        "entity_type": entity_type,
        "name": name,
    })
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
        .expect("workspace_query response")
}

fn structured(response: &Value) -> &Value {
    let result = response["result"].as_object().expect("MCP result");
    assert_valid_mcp_envelope(result);
    &response["result"]["structuredContent"]
}

fn assert_coverage(
    result: &Value,
    status: &str,
    identity_indexed: bool,
    capability_established: bool,
) {
    assert_eq!(result["coverage"]["status"], status, "{result}");
    assert_eq!(
        result["coverage"]["identity_indexed"], identity_indexed,
        "{result}"
    );
    assert_eq!(
        result["coverage"]["capability_established"], capability_established,
        "{result}"
    );
    assert_eq!(result["coverage"]["source_complete"], false, "{result}");
}

#[test]
fn workspace_query_exact_identity_coverage_single_results() {
    let _serial = crate::protocol::handler_response_serial();
    let state = state();

    let bogus_response = dispatch(&state, exact("DefinitelyMissing", "DefinitelyUnsupported"));
    let bogus = structured(&bogus_response);
    assert_eq!(bogus["count"], 0);
    assert_coverage(bogus, "identity_not_indexed", false, false);

    let unsupported_response = dispatch(&state, exact("IFooService", "Interface"));
    let unsupported = structured(&unsupported_response);
    assert_eq!(unsupported["count"], 0);
    assert_coverage(unsupported, "capability_not_established", true, false);

    let positive_response = dispatch(&state, exact("IFooService", "TypeRef"));
    let positive = structured(&positive_response);
    assert_eq!(positive["count"], 1);
    assert_coverage(positive, "established_indexed_capability", true, true);

    let unused_response = dispatch(&state, exact("IUnusedService", "TypeRef"));
    let unused = structured(&unused_response);
    assert_eq!(unused["count"], 0);
    assert_coverage(unused, "established_indexed_capability", true, true);

    let text = unused_response["result"]["content"][0]["text"]
        .as_str()
        .expect("model-facing content");
    assert!(text.contains("\"status\": \"established_indexed_capability\""));
    assert!(!text.contains("authoritative_index_snapshot_for_effective_scope"));

    let traversal_response = dispatch(
        &state,
        json!({
            "type": "transitive_dependencies",
            "domain": "builtin",
            "entity_type": "Class",
            "name": "BarController",
            "depth": 1,
        }),
    );
    let traversal = structured(&traversal_response);
    assert_coverage(traversal, "established_indexed_capability", true, true);
}

#[test]
fn workspace_query_exact_identity_coverage_batch_results() {
    let _serial = crate::protocol::handler_response_serial();
    let state = state();
    let response = dispatch(
        &state,
        json!({
            "queries": [
                { "id": "bogus", "type": "reverse_edges", "domain": "builtin", "entity_type": "DefinitelyUnsupported", "name": "DefinitelyMissing" },
                { "id": "unsupported", "type": "reverse_edges", "domain": "builtin", "entity_type": "Interface", "name": "IFooService" },
                { "id": "positive", "type": "reverse_edges", "domain": "builtin", "entity_type": "TypeRef", "name": "IFooService" },
                { "id": "unused", "type": "reverse_edges", "domain": "builtin", "entity_type": "TypeRef", "name": "IUnusedService" }
            ]
        }),
    );
    let results = structured(&response)["results"]
        .as_array()
        .expect("batch results");
    let expected = [
        ("identity_not_indexed", false, false, 0),
        ("capability_not_established", true, false, 0),
        ("established_indexed_capability", true, true, 1),
        ("established_indexed_capability", true, true, 0),
    ];
    for (item, (status, identity, capability, count)) in results.iter().zip(expected) {
        assert_eq!(item["status"], "ok");
        assert_eq!(item["result"]["count"], count);
        assert_coverage(&item["result"], status, identity, capability);
    }
}

#[test]
fn workspace_query_exact_identity_coverage_schema() {
    let workspace_query = tool_list()
        .into_iter()
        .find(|tool| tool["name"] == "workspace_query")
        .expect("workspace_query schema");
    let coverage = &workspace_query["outputSchema"]["properties"]["coverage"];
    let statuses = coverage["properties"]["status"]["enum"]
        .as_array()
        .expect("coverage status enum");
    for status in [
        "established_indexed_capability",
        "identity_not_indexed",
        "capability_not_established",
        "indexed_evidence_only",
    ] {
        assert!(statuses.iter().any(|candidate| candidate == status));
    }
    for field in [
        "identity_indexed",
        "capability_established",
        "source_complete",
    ] {
        assert_eq!(coverage["properties"][field]["type"], "boolean");
    }
}
