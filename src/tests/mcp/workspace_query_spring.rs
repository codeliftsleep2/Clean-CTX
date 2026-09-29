//! Spring Boot semantic facts through the real MCP provide/query lifecycle.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;

fn pop_response() -> serde_json::Value {
    crate::protocol::captured_responses()
        .pop()
        .expect("captured MCP response")
}

#[test]
fn spring_provide_publishes_autowired_dependency_for_workspace_query() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("UserController.java");
    std::fs::write(
        &path,
        r#"import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    private UserService userService;
}
"#,
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(dir.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let root = dir.path().to_string_lossy().into_owned();

    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({ "arguments": {
            "filePath": path.to_string_lossy(),
            "workspaceRoot": root,
            "fidelity": "high"
        }}),
        &state,
    );
    let provide = pop_response();
    assert!(provide.get("result").is_some(), "provide failed: {provide}");

    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(2),
        "workspace_query",
        &json!({ "arguments": {
            "type": "reverse_edges",
            "domain": "spring",
            "entity_type": "Service",
            "name": "UserService",
            "workspaceRoot": dir.path().to_string_lossy()
        }}),
        &state,
    );
    let query = pop_response();
    let edges = query["result"]["structuredContent"]["edges"]
        .as_array()
        .unwrap_or_else(|| panic!("workspace_query failed: {query}"));
    assert!(
        edges.iter().any(|edge| {
            edge["relation"] == "Autowired"
                && edge["subject"]["name"] == "UserController"
                && edge["object"]["name"] == "UserService"
        }),
        "workspace_query must consume the dependency published by provide: {query}"
    );
}
