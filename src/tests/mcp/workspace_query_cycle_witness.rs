use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn dependency_edge(from: &str, to: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", from),
        object: EntityRef::new("angular", "Service", to),
        layer: "angular",
        call_evidence: None,
    }
}

fn pop_response() -> Value {
    crate::protocol::captured_responses()
        .pop()
        .expect("handler must send one response")
}

fn call_has_cycle(state: &crate::mcp::McpState, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({ "arguments": arguments }),
        state,
    );
    pop_response()
}

#[test]
fn has_cycle_returns_a_closed_dependency_witness_with_honest_coverage() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    {
        let mut index = state.workspace_index_lock();
        index.add_edges(
            "src/alpha.service.ts",
            vec![dependency_edge("AlphaService", "BetaService")],
        );
        index.add_edges(
            "src/beta.service.ts",
            vec![dependency_edge("BetaService", "AlphaService")],
        );
    }

    let response = call_has_cycle(&state, json!({ "type": "has_cycle", "kind": "dependency" }));
    let structured = &response["result"]["structuredContent"];
    assert_eq!(
        structured["has_cycle"], true,
        "dependency cycle must remain true"
    );

    let cycle = structured["cycle"]
        .as_array()
        .expect("has_cycle must expose an ordered cycle witness");
    assert_eq!(cycle.len(), 2, "the two-edge cycle must have two steps");
    assert_eq!(cycle[0]["object"], cycle[1]["subject"]);
    assert_eq!(cycle[1]["object"], cycle[0]["subject"]);
    assert_eq!(cycle[0]["relation"], "Injects");
    assert_eq!(cycle[1]["relation"], "Injects");
    assert!(
        cycle.iter().all(|step| step["asserting_file"].is_string()),
        "every witness edge must retain its asserting-file provenance"
    );

    assert_eq!(structured["coverage"]["status"], "indexed_evidence_only");
    assert_eq!(structured["coverage"]["source_complete"], false);
    assert_eq!(structured["identity_model"], "semantic_tuple");
}

#[test]
fn has_cycle_rejects_an_unknown_cycle_kind() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let response = call_has_cycle(
        &state,
        json!({ "type": "has_cycle", "kind": "arbitrary_graph" }),
    );

    assert_eq!(response["error"]["code"], -32602);
    assert!(
        response["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("kind")),
        "invalid-params response must identify the unsupported kind: {response:?}"
    );
}

#[test]
fn has_cycle_discloses_semantic_identity_collisions_in_its_witness() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    {
        let mut index = state.workspace_index_lock();
        index.add_edges(
            "src/alpha-one.service.ts",
            vec![dependency_edge("AlphaService", "BetaService")],
        );
        index.add_edges(
            "src/alpha-two.service.ts",
            vec![dependency_edge("AlphaService", "BetaService")],
        );
        index.add_edges(
            "src/beta.service.ts",
            vec![dependency_edge("BetaService", "AlphaService")],
        );
    }

    let response = call_has_cycle(&state, json!({ "type": "has_cycle", "kind": "dependency" }));
    let structured = &response["result"]["structuredContent"];
    assert_eq!(structured["has_cycle"], true);
    assert_eq!(
        structured["identity_ambiguous"], true,
        "a semantic tuple occurring in multiple files must not look physically unique"
    );
    let ambiguities = structured["identity_ambiguities"]
        .as_array()
        .expect("collision disclosure must list ambiguous witness identities");
    assert!(!ambiguities.is_empty());
    assert!(ambiguities.iter().all(|ambiguity| {
        ambiguity["occurrence_files"]
            .as_array()
            .is_some_and(|files| files.len() > 1)
    }));
}
