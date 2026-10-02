use super::*;

fn compile_csharp(
    dir: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    relative_path: &str,
    source: &str,
) {
    std::fs::write(dir.path().join(relative_path), source).unwrap();
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({
            "arguments": {
                "filePath": relative_path,
                "fidelity": "high",
                "workspaceRoot": dir.path().to_string_lossy()
            }
        }),
        state,
    );
    let response = pop_response();
    assert!(response.get("result").is_some(), "{response:?}");
}

fn query_edges(
    state: &crate::mcp::McpState,
    query_type: &str,
    entity_type: &str,
    name: &str,
) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(2),
        "workspace_query",
        &json!({
            "arguments": {
                "type": query_type,
                "domain": "builtin",
                "entity_type": entity_type,
                "name": name
            }
        }),
        state,
    );
    let response = pop_response();
    let result = response["result"].as_object().expect("query result");
    assert_valid_mcp_envelope(result);
    result["structuredContent"]["edges"].clone()
}

#[test]
fn csharp_constructor_type_consumers_reach_workspace_queries() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let source = r#"
public interface IFooService
{
    void Handle();
}

public sealed class BarController
{
    public BarController(IFooService fooService) {}
    public void Handle(IFooService foo) {}
}

public sealed class BazService
{
    public BazService(IFooService fooService) {}
}

public sealed class ImplementsFoo : IFooService
{
    public void Handle() {}
}

public sealed class CallOnly
{
    public void Run()
    {
        Handle();
    }
}
"#;
    compile_csharp(&dir, &state, "Consumers.cs", source);

    let reverse = query_edges(&state, "reverse_edges", "TypeRef", "IFooService");
    let reverse = reverse.as_array().expect("reverse edge array");
    assert_eq!(reverse.len(), 2, "{reverse:?}");
    let subjects: Vec<&str> = reverse
        .iter()
        .map(|edge| edge["subject"]["name"].as_str().expect("subject name"))
        .collect();
    assert_eq!(subjects, vec!["BarController", "BazService"]);
    assert!(reverse.iter().all(|edge| {
        edge["relation"].as_str() == Some("HasConstructorParameterType")
            && edge["object"]["entity_type"].as_str() == Some("TypeRef")
            && edge["object"]["name"].as_str() == Some("IFooService")
            && edge["layer"].as_str() == Some("builtin")
            && edge["subject"]["file"].as_str().is_some()
    }));

    let forward = query_edges(&state, "forward_edges", "Class", "BarController");
    let forward = forward.as_array().expect("forward edge array");
    assert_eq!(forward.len(), 1, "{forward:?}");
    assert_eq!(
        forward[0]["relation"].as_str(),
        Some("HasConstructorParameterType")
    );
    assert_eq!(forward[0]["object"]["name"].as_str(), Some("IFooService"));
}
