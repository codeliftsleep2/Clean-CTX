use serde_json::{Value, json};
use std::path::Path;

fn state() -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    crate::mcp::McpState::new(config)
}

fn call(state: &crate::mcp::McpState, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(10),
        tool,
        &json!({"arguments": arguments}),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("tool response")
}

fn hydrate(state: &crate::mcp::McpState, root: &Path, name: &str) {
    let response = call(
        state,
        "workspace_query",
        json!({"workspaceRoot": root, "type": "find_entities", "name": name}),
    );
    assert!(response.get("error").is_none(), "{response:#}");
}

fn refresh(state: &crate::mcp::McpState, root: &Path) {
    let response = call(state, "refresh_workspace", json!({"workspaceRoot": root}));
    assert_eq!(response["result"]["structuredContent"]["refreshed"], true);
}

fn test_class_count(state: &crate::mcp::McpState, name: &str) -> usize {
    state
        .workspace_index_read()
        .find_entities_by_name(name)
        .into_iter()
        .filter(|entity| entity.domain == "dotnet" && entity.entity_type == "TestClass")
        .count()
}

fn tests_edge_count(state: &crate::mcp::McpState, root: &Path, name: &str) -> usize {
    let response = call(
        state,
        "workspace_query",
        json!({
            "workspaceRoot": root,
            "type": "forward_edges",
            "domain": "dotnet",
            "entity_type": "TestClass",
            "name": name
        }),
    );
    assert!(response.get("error").is_none(), "{response:#}");
    response["result"]["structuredContent"]["edges"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|edge| edge["relation"] == "Tests")
        .count()
}

#[test]
fn r10_partial_testclass_composes_one_durable_role_across_parts() {
    let root = tempfile::TempDir::new().expect("workspace root");
    std::fs::write(
        root.path().join("PartialBody.cs"),
        "public partial class PartialTests { public void Works() {} }\n",
    )
    .expect("partial body fixture");
    std::fs::write(
        root.path().join("PartialMarker.cs"),
        "using Microsoft.VisualStudio.TestTools.UnitTesting; [TestClass] public partial class PartialTests {}\n",
    )
    .expect("partial marker fixture");
    let state = state();

    hydrate(&state, root.path(), "PartialTests");
    assert_eq!(
        test_class_count(&state, "PartialTests"),
        1,
        "one composed source symbol must publish one TestClass role"
    );
}

#[test]
fn r10_alias_retarget_attribute_removal_restoration_and_deletion_reconcile() {
    let root = tempfile::TempDir::new().expect("workspace root");
    let tests = root.path().join("OrderServiceTests.cs");
    std::fs::write(
        root.path().join("OrderService.cs"),
        "public class OrderService {}\n",
    )
    .expect("subject fixture");
    let resolved = "using Marker = Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute; [Marker] public class OrderServiceTests {}\n";
    std::fs::write(&tests, resolved).expect("resolved alias fixture");
    let state = state();

    hydrate(&state, root.path(), "OrderServiceTests");
    assert_eq!(test_class_count(&state, "OrderServiceTests"), 1);
    assert_eq!(
        tests_edge_count(&state, root.path(), "OrderServiceTests"),
        1
    );

    std::fs::write(
        &tests,
        "using Marker = Other.TestClassAttribute; namespace Other { public class TestClassAttribute : System.Attribute {} } [Marker] public class OrderServiceTests {}\n",
    )
    .expect("retarget alias");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "OrderServiceTests");
    assert_eq!(
        test_class_count(&state, "OrderServiceTests"),
        0,
        "alias retargeting must retract MSTest authority"
    );
    assert_eq!(
        tests_edge_count(&state, root.path(), "OrderServiceTests"),
        0
    );

    std::fs::write(&tests, resolved).expect("restore alias");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "OrderServiceTests");
    assert_eq!(test_class_count(&state, "OrderServiceTests"), 1);
    assert_eq!(
        tests_edge_count(&state, root.path(), "OrderServiceTests"),
        1
    );

    std::fs::remove_file(&tests).expect("delete test class");
    refresh(&state, root.path());
    assert_eq!(
        test_class_count(&state, "OrderServiceTests"),
        0,
        "owner deletion must retract MSTest authority"
    );
}
