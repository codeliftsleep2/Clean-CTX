//! Permanent receiver-versus-token regression at the registered MCP boundary.
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn call(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(138), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"].clone()
}

#[test]
fn angular_di_inject_token_receiver_is_not_the_dependency() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("consumer.ts");
    let source = format!(
        "import {{ Injectable, inject }} from '@angular/core';\n\
         @Injectable()\nexport class AuditService {{}}\n\
         @Injectable()\nexport class AuditConsumer {{\n\
         private readonly completelyDifferentName = inject(AuditService);\n}}\n{}",
        "// unchanged economics padding\n".repeat(100)
    );
    crate::edit::apply::verify_syntax(&source, "ts").unwrap();
    let evidence = crate::ir::invocation_evidence::collect_typescript_source(&source).unwrap();
    let view = evidence.view("standalone-typescript", &source).unwrap();
    let inject = view
        .occurrences()
        .iter()
        .find(|call| {
            matches!(call.callee(), crate::ir::invocation_evidence::WrittenCallee::Free { terminal }
            if view.slice(*terminal).unwrap() == "inject")
        })
        .unwrap();
    let receiver = inject.declaration_context().unwrap();
    assert_eq!(
        view.slice(receiver.written_name().unwrap()).unwrap(),
        "completelyDifferentName"
    );
    assert_eq!(
        view.slice(inject.arguments()[0].span()).unwrap(),
        "AuditService"
    );
    std::fs::write(&path, &source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let context = call(
        &state,
        "provide_code_context",
        json!({
            "workspaceRoot":root.path(),"filePath":path,"fidelity":"high"
        }),
    );
    assert!(
        context["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("C AuditConsumer")
    );
    let entities = call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot":root.path(),"type":"entities_in_file","file_path":path
        }),
    );
    for name in ["AuditConsumer", "AuditService"] {
        assert_eq!(
            entities["structuredContent"]["entities"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|entity| entity["domain"] == "angular"
                    && entity["entity_type"] == "Service"
                    && entity["name"] == name
                    && entity["file"] == path.to_string_lossy().as_ref())
                .count(),
            1
        );
    }
    let answer = call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot":root.path(),"type":"forward_edges","domain":"angular",
            "entity_type":"Service","name":"AuditConsumer"
        }),
    );
    println!("primary MCP result: {answer}");
    let edges: Vec<_> = answer["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .collect();
    let targets: Vec<_> = edges
        .iter()
        .map(|edge| edge["object"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        targets,
        ["AuditService"],
        "receiver must never establish dependency identity: {answer}"
    );
    assert_eq!(edges[0]["subject"]["name"], "AuditConsumer");
    assert_eq!(edges[0]["object"]["domain"], "angular");
    assert_eq!(edges[0]["object"]["entity_type"], "Service");
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}
