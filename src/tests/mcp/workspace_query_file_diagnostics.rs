use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn query(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    file: &std::path::Path,
    within: Option<&str>,
) -> Value {
    crate::protocol::captured_responses().clear();
    let mut arguments = json!({
        "type": "entities_in_file",
        "workspaceRoot": root,
        "file_path": file,
    });
    if let Some(within) = within {
        arguments["withinPath"] = json!(within);
    }
    dispatch_tools_call(
        &json!(135),
        "workspace_query",
        &json!({ "arguments": arguments }),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}

fn reason(response: &Value) -> Option<&str> {
    response["error"]["data"]["reason"].as_str()
}

#[test]
fn file_queries_distinguish_empty_scope_security_type_and_lifecycle() {
    let _serial = crate::protocol::handler_response_serial();
    let base = tempfile::tempdir().unwrap();
    let root = base.path().join("repo");
    let included = root.join("src/included");
    let excluded = root.join("src/excluded");
    std::fs::create_dir_all(&included).unwrap();
    std::fs::create_dir_all(&excluded).unwrap();
    let empty = included.join("empty.ts");
    let declared = included.join("declared.ts");
    let outside_within = excluded.join("outside.ts");
    let unsupported = included.join("notes.txt");
    std::fs::write(&empty, "// intentionally empty").unwrap();
    std::fs::write(&declared, "export class Present {}").unwrap();
    std::fs::write(&outside_within, "export class Excluded {}").unwrap();
    std::fs::write(&unsupported, "not source").unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let empty_answer = query(&state, &root, &empty, None);
    assert_eq!(empty_answer["result"]["structuredContent"]["count"], 0);
    let declared_answer = query(&state, &root, &declared, None);
    assert!(
        declared_answer["result"]["structuredContent"]["count"]
            .as_u64()
            .is_some_and(|count| count > 0)
    );
    assert_eq!(
        reason(&query(&state, &root, &outside_within, Some("src/included"))),
        Some("excluded_by_within_path")
    );
    assert_eq!(
        reason(&query(&state, &root, &unsupported, None)),
        Some("unsupported_file_type")
    );

    let missing = included.join("deleted.ts");
    assert_eq!(
        reason(&query(&state, &root, &missing, None)),
        Some("file_not_found")
    );
    let other = tempfile::tempdir().unwrap();
    let unauthorized = other.path().join("secret.ts");
    std::fs::write(&unauthorized, "export class Secret {}").unwrap();
    assert_eq!(
        reason(&query(&state, &root, &unauthorized, None)),
        Some("outside_authorized_roots")
    );

    let warm = query(&state, &root, &declared, None);
    assert_eq!(
        warm["result"]["structuredContent"], declared_answer["result"]["structuredContent"],
        "query history must not change the admitted file answer"
    );
}
