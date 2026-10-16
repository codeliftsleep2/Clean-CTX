//! Legacy declaration authority must exist independently of dispatch/linkage.
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn call(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(140), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"].clone()
}

#[test]
fn ngrx_legacy_declaration_class_without_uses_is_publicly_reachable() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("actions.ts");
    let declaration = "export class NoSuffixRecord implements Action { readonly type = '[Audit] Set Audit'; constructor(public payload: string) {} }";
    let source = format!(
        "import {{ Action }} from '@ngrx/store';\n{declaration}\n{}",
        "// unchanged economics padding\n".repeat(100)
    );
    crate::edit::apply::verify_syntax(&source, "ts").unwrap();
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let captures = crate::compression::capture_pipeline::run_capture_pipeline_nodes(
        language,
        query,
        &source,
        crate::compression::Fidelity::High,
        |_, raw, _| Some(raw.into()),
    )
    .unwrap();
    assert!(captures.iter().any(|capture| capture.name == "class.root"
        && capture.raw_text == declaration.strip_prefix("export ").unwrap()));
    std::fs::write(&path, &source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let context = call(
        &state,
        "provide_code_context",
        json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":"high"}),
    );
    assert!(
        context["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("C NoSuffixRecord")
    );
    let entities = call(
        &state,
        "workspace_query",
        json!({"workspaceRoot":root.path(),"type":"entities_in_file","file_path":path}),
    );
    println!("primary entities: {entities}");
    let actions: Vec<_> = entities["structuredContent"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Action")
        .collect();
    assert_eq!(
        actions.len(),
        1,
        "source-proven Action declaration missing: {entities}"
    );
    assert_eq!(actions[0]["name"], "NoSuffixRecord");
    assert_eq!(actions[0]["file"], path.to_string_lossy().as_ref());
    let edges = call(
        &state,
        "workspace_query",
        json!({"workspaceRoot":root.path(),"type":"forward_edges","domain":"ngrx","entity_type":"Action","name":"NoSuffixRecord"}),
    );
    assert!(
        edges["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .is_empty(),
        "declaration must not fabricate handler/reference relationships: {edges}"
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}
