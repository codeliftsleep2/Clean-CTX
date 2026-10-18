//! Issue #144 RED characterization of redundant reverse-TypeRef work.
//!
//! Wall-clock timing is deliberately excluded. The production-path query must
//! parse exactly the K owners reported as compiled, must not reconcile the
//! unrelated exact-Calls capability, and must do no work when repeated warm.

use crate::mcp::semantic_work_metrics;
use crate::mcp::tools::dispatch_tools_call;
use crate::tests::assert_valid_mcp_envelope;
use serde_json::{Value, json};
use std::path::Path;

fn write_fixture(root: &Path, consumer_count: usize) {
    std::fs::create_dir(root.join(".git")).expect("Git root marker");
    std::fs::write(
        root.join("ContextA.cs"),
        "public sealed class ContextA {}\n",
    )
    .expect("target source");
    for index in 0..consumer_count {
        std::fs::write(
            root.join(format!("Consumer{index:02}.cs")),
            format!(
                "public sealed class Consumer{index:02} {{ public Consumer{index:02}(ContextA value) {{}} }}\n"
            ),
        )
        .expect("consumer source");
    }
}

fn reverse_type_ref(state: &crate::mcp::McpState, root: &Path) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(144),
        "workspace_query",
        &json!({
            "arguments": {
                "workspaceRoot": root,
                "type": "reverse_edges",
                "domain": "builtin",
                "entity_type": "TypeRef",
                "name": "ContextA"
            }
        }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("workspace_query response")
}

fn structured(response: &Value) -> &Value {
    let result = response["result"]
        .as_object()
        .unwrap_or_else(|| panic!("workspace_query must succeed: {response}"));
    assert_valid_mcp_envelope(result);
    &response["result"]["structuredContent"]
}

#[test]
fn issue_144_performance_work_counts_compile_only_k_and_skip_unrelated_calls() {
    let _response_serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path(), 8);
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);

    semantic_work_metrics::reset();
    state.reset_test_csharp_reconciliation_count();
    let cold = reverse_type_ref(&state, root.path());
    let cold = structured(&cold);
    let compiled = cold["discovery"]["compiled"]
        .as_u64()
        .expect("cold query must report compiled K") as usize;
    let cold_parses = semantic_work_metrics::full_semantic_parses();
    let cold_reconciliations = state.test_csharp_reconciliation_count();

    semantic_work_metrics::reset();
    state.reset_test_csharp_reconciliation_count();
    let warm = reverse_type_ref(&state, root.path());
    let warm = structured(&warm);
    let warm_parses = semantic_work_metrics::full_semantic_parses();
    let warm_reconciliations = state.test_csharp_reconciliation_count();

    assert_eq!(cold["coverage"]["source_complete"], true, "{cold}");
    assert_eq!(warm["coverage"]["source_complete"], true, "{warm}");
    assert_eq!(cold["edges"], warm["edges"], "cold/warm truth differs");
    assert_eq!(
        (cold_parses, cold_reconciliations),
        (compiled, 0),
        "reverse TypeRef must parse exactly K inadequate owners and must not prepare exact Calls"
    );
    assert_eq!(
        (warm_parses, warm_reconciliations),
        (0, 0),
        "unchanged warm reverse TypeRef must perform no semantic compilation or exact-Calls reconciliation"
    );
}
