use super::*;

#[path = "pcg_knowledge_schema.rs"]
mod schema_interoperability;

fn knowledge_call(state: &McpState, args: Value) -> Value {
    protocol::captured_responses().clear();
    crate::mcp::router::dispatch(
        crate::protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(9)),
            method: "tools/call".into(),
            params: Some(json!({"name":"pcg_knowledge_v1","arguments":args})),
        },
        state,
    );
    protocol::captured_responses().pop().unwrap()
}

#[test]
fn registered_knowledge_is_conditional_closed_and_production_reachable() {
    let f = Fixture::new();
    let source = "<!-- pcg:A:begin -->\nExact pipeline statement.\n<!-- pcg:A:end -->\n\
```architecture-graph\nnode: DOC-001\nkind: INVARIANT\nstatus: IMPLEMENTED\ntitle: First\ntrack: FOUNDATION\nartifact: docs/a.md\nknowledge_scope: invariant\nknowledge_content:\n  - statement self#A\n```\n";
    std::fs::write(f.root.join("docs/a.md"), source).unwrap();
    let args = json!({"workspaceRoot":f.root,"mode":"get","id":"DOC-001"});
    assert_eq!(
        knowledge_call(&state(None), args.clone())["error"]["code"],
        -32601
    );
    let enabled = state(Some(&f));
    let catalog = crate::mcp::tools::tool_list_for_state(&enabled);
    let descriptor = catalog
        .iter()
        .find(|d| d["name"] == "pcg_knowledge_v1")
        .unwrap();
    assert_eq!(descriptor["inputSchema"]["additionalProperties"], false);
    assert_eq!(descriptor["annotations"]["readOnlyHint"], false);
    let response = knowledge_call(&enabled, args.clone());
    assert!(response.get("error").is_none(), "{response}");
    let payload: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["contract"], "pcg_knowledge_v1");
    assert!(payload["units"][0]["components"][0].get("text").is_none());
    let acquired = knowledge_call(&enabled, payload["units"][0]["acquire_request"].clone());
    let acquired: Value =
        serde_json::from_str(acquired["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(
        acquired["acquisitions"][0]["components"][0]["text"],
        "Exact pipeline statement.\n"
    );
    let reference = payload["units"][0]["source_request"].clone();
    let original = knowledge_call(&enabled, reference);
    let original: Value =
        serde_json::from_str(original["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(original["sources"][0]["text"], source);
    let mut unknown = args.clone();
    unknown["tokenBudget"] = 1.into();
    assert_eq!(knowledge_call(&enabled, unknown)["error"]["code"], -32602);
    let mut invalid = args;
    invalid["role"] = Value::Null;
    assert_eq!(knowledge_call(&enabled, invalid)["error"]["code"], -32602);
}

#[test]
fn registered_knowledge_rejects_stale_boundary_without_publishing_partial_source() {
    let f = Fixture::new();
    std::fs::write(f.root.join("docs/a.md"),"```architecture-graph\nnode: DOC-001\nkind: MISSION\nstatus: INVESTIGATING\ntitle: First\ntrack: FOUNDATION\nknowledge_scope: proposition\nknowledge_content:\n  - statement self#ABSENT\n```\n").unwrap();
    let enabled = state(Some(&f));
    let response = knowledge_call(
        &enabled,
        json!({"workspaceRoot":f.root,"mode":"get","id":"DOC-001"}),
    );
    let payload: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["result_state"], "blocked");
    assert_eq!(payload["units"], json!([]));
    assert_eq!(payload["diagnostics"][0]["code"], "E_KNOWLEDGE_BOUNDARY");
}
