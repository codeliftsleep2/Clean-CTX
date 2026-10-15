use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn call(state: &crate::mcp::McpState, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(140), tool, &json!({ "arguments": arguments }), state);
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("MCP response");
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"].clone()
}

fn public_selects(store_module: &str, property: &str) -> Vec<Value> {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("receiver.ts");
    let source = format!(
        "import {{ createSelector }} from '@ngrx/store';\n\
         import {{ Store }} from '{store_module}';\n\
         const base = (state: unknown) => state;\n\
         export const selectAudit = createSelector(base, state => state);\n\
         export class AuditService {{\n\
           constructor(private {property}: Store<AppState>) {{}}\n\
           load() {{ return this.{property}.select(selectAudit); }}\n\
         }}"
    );
    std::fs::write(&path, source).expect("fixture");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    call(
        &state,
        "provide_code_context",
        json!({"workspaceRoot": root.path(), "filePath": path, "fidelity": "high"}),
    );
    let result = call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot": root.path(), "type": "reverse_edges",
            "domain": "ngrx", "entity_type": "Selector", "name": "selectAudit"
        }),
    );
    result["edges"]
        .as_array()
        .expect("edges")
        .iter()
        .filter(|edge| edge["relation"] == "Selects")
        .cloned()
        .collect()
}

#[test]
fn public_semantics_require_exact_ngrx_store_receiver_authority() {
    let valid = public_selects("@ngrx/store", "state");
    assert_eq!(valid.len(), 1, "{valid:?}");
    assert_eq!(valid[0]["object"]["name"], "selectAudit");
    assert!(public_selects("./my-own-store", "store").is_empty());
}

#[test]
fn receiver_qualification_advances_only_angular_semantics() {
    assert_eq!(
        crate::layers::meta::ANGULAR_SEMANTIC_PRODUCER_GENERATION,
        21
    );
    assert_eq!(crate::layers::meta::ANGULAR_MARKER_PRODUCER_GENERATION, 5);
    assert_eq!(crate::ir::pipeline::CANONICAL_PIPELINE_GENERATION, 1);
}
