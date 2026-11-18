use super::*;

fn compile(dir: &tempfile::TempDir, state: &crate::mcp::McpState, source: &str) {
    std::fs::write(dir.path().join("Dependencies.cs"), source).unwrap();
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({ "arguments": {
            "filePath": "Dependencies.cs",
            "fidelity": "high",
            "workspaceRoot": dir.path().to_string_lossy()
        }}),
        state,
    );
    assert!(pop_response().get("result").is_some());
}

fn query(
    dir: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    query_type: &str,
    extra: serde_json::Value,
) -> serde_json::Value {
    let mut arguments = serde_json::json!({
        "type": query_type,
        "workspaceRoot": dir.path().to_string_lossy()
    });
    arguments
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(2),
        "workspace_query",
        &json!({ "arguments": arguments }),
        state,
    );
    let response = pop_response();
    assert!(response.get("error").is_none(), "{response:?}");
    response["result"]["structuredContent"].clone()
}

#[test]
fn exact_csharp_dependencies_respect_depth_and_exclude_unresolved_decoys() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    compile(
        &dir,
        &state,
        r#"
namespace Product;
public class A : B { public A(Missing guessed) {} }
public class B { public B(C value) {} }
public class C : D {}
public class D {}
"#,
    );

    for (depth, expected) in [
        (1, vec!["B"]),
        (2, vec!["B", "C"]),
        (3, vec!["B", "C", "D"]),
        (0, vec!["B", "C", "D"]),
    ] {
        let result = query(
            &dir,
            &state,
            "transitive_dependencies",
            json!({ "domain": "builtin", "entity_type": "Class", "name": "A", "depth": depth }),
        );
        let names: Vec<_> = result["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|dependency| dependency[2].as_str().unwrap())
            .collect();
        assert_eq!(names, expected, "{result:?}");
    }
}

#[test]
fn exact_constructor_cycle_returns_a_closed_deterministic_witness() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    compile(
        &dir,
        &state,
        "public class P { public P(Q q) {} } public class Q { public Q(P p) {} }",
    );

    let first = query(&dir, &state, "has_cycle", json!({ "kind": "dependency" }));
    let second = query(&dir, &state, "has_cycle", json!({ "kind": "dependency" }));
    assert_eq!(first, second);
    assert_eq!(first["has_cycle"], true);
    let cycle = first["cycle"].as_array().unwrap();
    assert_eq!(cycle.len(), 2, "{first:?}");
    assert_eq!(cycle[0]["subject"]["name"], "P");
    assert_eq!(cycle[0]["object"]["name"], "Q");
    assert_eq!(cycle[1]["subject"]["name"], "Q");
    assert_eq!(cycle[1]["object"]["name"], "P");
    assert!(
        cycle
            .iter()
            .all(|step| step["relation"] == "HasConstructorParameterType")
    );
}

#[test]
fn ambiguous_or_missing_csharp_targets_do_not_enter_exact_traversal() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    compile(
        &dir,
        &state,
        r#"
namespace One { public class Target {} }
namespace Two { public class Target {} }
public class Root { public Root(Target ambiguous, Missing absent) {} }
"#,
    );
    let result = query(
        &dir,
        &state,
        "transitive_dependencies",
        json!({ "domain": "builtin", "entity_type": "Class", "name": "Root", "depth": 0 }),
    );
    assert_eq!(result["dependencies"], json!([]), "{result:?}");
}
