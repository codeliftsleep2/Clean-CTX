//! Characterize enclosing public-query work, preserving exact-call identity.
use crate::mcp::tools::dispatch_tools_call;
use crate::work_observations_af_perf_002 as work;
use serde_json::{Value, json};

#[path = "csharp_a3_budget_af_perf_002.rs"]
mod a3_budget;

#[path = "artifact_owner_class_af_perf_002.rs"]
mod a2_growth;

fn query(state: &crate::mcp::McpState, root: &std::path::Path, kind: &str, name: &str) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({"arguments":{
            "type":"reverse_edges", "domain":"builtin", "entity_type":kind,
            "name":name, "workspaceRoot":root
        }}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("query response");
    assert!(response.get("error").is_none(), "{response}");
    response
}

fn assert_save_call(response: &Value) {
    let edges = response["result"]["structuredContent"]["edges"]
        .as_array()
        .expect("edges");
    assert!(
        edges
            .iter()
            .any(|edge| edge["subject"]["name"] == "Run" && edge["object"]["name"] == "Save"),
        "{response}"
    );
}

fn assert_published_exact_call(state: &crate::mcp::McpState, root: &std::path::Path) {
    let caller_file = state.semantic_owner_path(&root.join("Caller.cs").to_string_lossy());
    let index = state.workspace_index_read();
    let edges = index.reverse_edges_by_identity("csharp", "Method", "Target::Save()->void");
    assert!(
        edges.iter().any(|edge| {
            edge.layer == "csharp-exact-call"
                && edge.relation == crate::layers::meta::semantic::SemanticRelation::Calls
                && edge.subject.domain == "csharp"
                && edge.subject.name == "Caller::Run(Target)->void"
                && edge.object.domain == "csharp"
                && edge.object.name == "Target::Save()->void"
                && edge.subject.file.as_deref() == Some(caller_file.as_str())
                && edge.call_evidence.is_some_and(|evidence| {
                    evidence.explicit_arg_count == 0 && !evidence.has_spread
                })
        }),
        "public trigger must publish the independently exact selected call; generic names alone cannot qualify"
    );
}

#[test]
fn af002_artifact_public_query_call_catalog_growth() {
    let _serial = crate::protocol::handler_response_serial();
    // Independent axes plus one joint point, not a quadratic test-case campaign.
    let dimensions = [
        (1, 1),
        (2, 1),
        (4, 1),
        (8, 1),
        (16, 1),
        (64, 1),
        (256, 1),
        (1, 2),
        (1, 4),
        (1, 8),
        (1, 16),
        (1, 64),
        (1, 256),
        (16, 16),
    ];
    for (calls, extra_declarations) in dimensions {
        let root = tempfile::tempdir().unwrap();
        let body = "target.Save();".repeat(calls);
        std::fs::write(
            root.path().join("Caller.cs"),
            format!("class Caller {{ void Run(Target target) {{ {body} }} }}"),
        )
        .unwrap();
        let methods: String = (0..extra_declarations)
            .map(|i| format!("public void Other{i}() {{}} "))
            .collect();
        std::fs::write(
            root.path().join("Target.cs"),
            format!("class Target {{ public void Save() {{}} {methods} }}"),
        )
        .unwrap();
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        let state = crate::mcp::McpState::new(config);
        *state.graph_bridge_lock() = None;
        query(&state, root.path(), "TypeRef", "Target");
        // Explicit existing dirty lifecycle gives a reproducible reconciliation
        // boundary; it does not claim every ordinary query starts dirty.
        state.mark_csharp_authority_dirty();
        state.reset_test_csharp_reconciliation_count();
        state.reset_test_full_semantic_parses();
        work::reset();
        let started = std::time::Instant::now();
        let first = query(&state, root.path(), "Method", "Save");
        assert_save_call(&first);
        assert_published_exact_call(&state, root.path());
        let counts = work::snapshot();
        assert!(
            counts.artifact_projection_owners > 0,
            "fixture must reach retained generation projection"
        );
        assert!(
            counts.csharp_declaration_probes[0] > 0,
            "fixture must reach actual declaration selection"
        );
        eprintln!(
            "AF002_ARTIFACT_POPULATION requested_calls={calls} requested_extra_declarations={extra_declarations} actual_resolution_calls={} actual_catalog_population_max={}",
            counts.csharp_resolution_calls, counts.csharp_catalog_population_max
        );
        eprintln!(
            "AF002_ARTIFACT calls={calls} extra_declarations={extra_declarations} phase=dirty projection_owners={} resolutions={} declaration_probes={:?} ir_instructions_cloned={} publication_clones={:?} reconciliations={} full_parses={} us={}",
            counts.artifact_projection_owners,
            counts.csharp_resolution_calls,
            counts.csharp_declaration_probes,
            counts.artifact_ir_instruction_clones,
            counts.artifact_publication_clones,
            state.test_csharp_reconciliation_count(),
            state.test_full_semantic_parses(),
            started.elapsed().as_micros()
        );
        work::reset();
        state.reset_test_csharp_reconciliation_count();
        state.reset_test_full_semantic_parses();
        let started = std::time::Instant::now();
        let repeated = query(&state, root.path(), "Method", "Save");
        assert_save_call(&repeated);
        assert_published_exact_call(&state, root.path());
        let warm = work::snapshot();
        eprintln!(
            "AF002_ARTIFACT_WARM_PARSE full_parses={}",
            state.test_full_semantic_parses()
        );
        eprintln!(
            "AF002_ARTIFACT calls={calls} extra_declarations={extra_declarations} phase=warm projection_owners={} resolutions={} declaration_probes={:?} ir_instructions_cloned={} publication_clones={:?} reconciliations={} us={}",
            warm.artifact_projection_owners,
            warm.csharp_resolution_calls,
            warm.csharp_declaration_probes,
            warm.artifact_ir_instruction_clones,
            warm.artifact_publication_clones,
            state.test_csharp_reconciliation_count(),
            started.elapsed().as_micros()
        );
    }
}

#[path = "compression_token_work_af_perf_002.rs"]
mod compression_token_work;

#[path = "c1_public_response_af_perf_002.rs"]
mod c1_public_response;

#[path = "c2_public_alias_work_af_perf_002.rs"]
mod c2_public_alias_work;
