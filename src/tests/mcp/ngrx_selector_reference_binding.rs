//! Public MCP proof for exact same-file NgRx Selector reference binding.

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

fn publish_and_selects(source: &str, component: &str) -> Vec<Value> {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("selector-reference.ts");
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
            "domain": "angular", "entity_type": "Component", "name": component
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

fn source(body: &str) -> String {
    format!(
        "import {{ Component }} from '@angular/core';\n\
         import {{ createSelector, Store }} from '@ngrx/store';\n\
         const selectState = (state: unknown) => state;\n\
         export const selectAudit = createSelector(selectState, state => state);\n\
         @Component({{ selector: 'audit-view' }})\n\
         export class AuditComponent {{\n\
           constructor(private store: Store) {{}}\n\
           consume() {{ {body} }}\n\
         }}"
    )
}

#[test]
fn public_relationship_requires_exact_authoritative_declaration() {
    let direct = publish_and_selects(
        &source("return this.store.select(selectAudit);"),
        "AuditComponent",
    );
    assert_eq!(direct.len(), 1, "{direct:?}");
    assert_eq!(direct[0]["object"]["name"], "selectAudit");

    let shadowed = publish_and_selects(
        &source("const selectAudit = 123; return this.store.select(selectAudit);"),
        "AuditComponent",
    );
    assert!(shadowed.is_empty(), "{shadowed:?}");
}

#[test]
fn exact_binding_changes_only_the_angular_semantic_generation() {
    assert_eq!(
        crate::layers::meta::ANGULAR_SEMANTIC_PRODUCER_GENERATION,
        25
    );
    assert_eq!(crate::layers::meta::ANGULAR_MARKER_PRODUCER_GENERATION, 6);
    assert_eq!(crate::ir::pipeline::CANONICAL_PIPELINE_GENERATION, 1);
}
