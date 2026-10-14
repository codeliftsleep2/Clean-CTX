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

fn public_service_selects(decorator: &str, store_module: &str, selector: &str) -> Vec<Value> {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("service-select.ts");
    let source = format!(
        "import {{ Injectable }} from '@angular/core';\n\
         import {{ createSelector }} from '@ngrx/store';\n\
         import {{ Store }} from '{store_module}';\n\
         const base = (state: unknown) => state;\n\
         export const selectAudit = createSelector(base, state => state);\n\
         {decorator}\n\
         export class AuditService {{\n\
           constructor(private state: Store<AppState>) {{}}\n\
           load() {{ return this.state.select({selector}); }}\n\
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
            "workspaceRoot": root.path(), "type": "forward_edges",
            "domain": "angular", "entity_type": "Service", "name": "AuditService"
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
fn public_service_select_requires_owner_operation_and_target_authority() {
    let valid = public_service_selects("@Injectable()", "@ngrx/store", "selectAudit");
    assert_eq!(valid.len(), 1, "{valid:?}");
    assert_eq!(valid[0]["object"]["name"], "selectAudit");
    assert!(public_service_selects("", "@ngrx/store", "selectAudit").is_empty());
    assert!(public_service_selects("@Injectable()", "./foreign", "selectAudit").is_empty());
    assert!(public_service_selects("@Injectable()", "@ngrx/store", "ordinary").is_empty());
}

#[test]
fn service_select_ownership_advances_only_angular_semantics() {
    assert_eq!(
        crate::layers::meta::ANGULAR_SEMANTIC_PRODUCER_GENERATION,
        17
    );
    assert_eq!(crate::layers::meta::ANGULAR_MARKER_PRODUCER_GENERATION, 5);
    assert_eq!(crate::ir::pipeline::CANONICAL_PIPELINE_GENERATION, 1);
}
