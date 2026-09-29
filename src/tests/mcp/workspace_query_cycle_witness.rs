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

#[test]
fn has_cycle_no_cycle_response_is_empty_index_only_and_side_effect_free() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let before_stats = state.session_stats_lock().summary();
    let before_cache_len = state.source_cache_lock().len();

    let response = call_has_cycle(
        &state,
        json!({
            "type": "has_cycle",
            "kind": "dependency",
            "workspaceRoot": "C:/workspace-that-need-not-exist"
        }),
    );
    let structured = &response["result"]["structuredContent"];
    assert_eq!(structured["has_cycle"], false);
    assert_eq!(structured["cycle"], json!([]));
    assert_eq!(structured["coverage"]["status"], "indexed_evidence_only");
    assert_eq!(structured["coverage"]["source_complete"], false);

    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("model-facing content");
    assert!(text.contains("\"authority\": \"workspace_index\""));
    assert!(text.contains("\"status\": \"indexed_evidence_only\""));
    assert!(
        text.contains("\"zero_result\": true"),
        "a false cycle result with an empty witness is a zero result: {text}"
    );
    assert!(!text.contains("authoritative_index_snapshot_for_effective_scope"));

    let after_stats = state.session_stats_lock().summary();
    assert_eq!(after_stats.total_files, before_stats.total_files);
    assert_eq!(
        after_stats.full_compress_count,
        before_stats.full_compress_count
    );
    assert_eq!(state.source_cache_lock().len(), before_cache_len);
    assert!(state.workspace_index_read().is_empty());
}
