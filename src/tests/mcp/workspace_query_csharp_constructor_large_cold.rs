// Issue #133 acceptance regression: a first cold reverse query must discover
// every C# constructor consumer across a large, multi-directory workspace.

use super::*;
use crate::mcp::tool_handlers::hydration::TEST_PROJECT_HYDRATION_SERIALIZE;
use std::collections::BTreeSet;

const CONTRACT: &str = "ILargeColdContract";
const CONSUMER_COUNT: usize = 52;
const DIRECTORY_COUNT: usize = 4;

fn dispatch(
    state: &crate::mcp::McpState,
    tool: &str,
    arguments: serde_json::Value,
) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), tool, &json!({ "arguments": arguments }), state);
    pop_response()
}

#[test]
fn issue_133_large_cold_reverse_query_finds_every_constructor_consumer() {
    let _hydration_serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    std::fs::create_dir(root.path().join(".git")).expect("Git root marker");
    std::fs::write(
        root.path().join(format!("{CONTRACT}.cs")),
        format!("public interface {CONTRACT} {{}}\n"),
    )
    .expect("interface fixture");
    std::fs::write(
        root.path().join("LargeColdContract.cs"),
        format!("public sealed class LargeColdContract : {CONTRACT} {{}}\n"),
    )
    .expect("matching-stem implementation control");

    let mut expected = BTreeSet::new();
    for index in 0..CONSUMER_COUNT {
        let directory = root
            .path()
            .join(format!("group{}", index % DIRECTORY_COUNT));
        std::fs::create_dir_all(&directory).expect("consumer directory");
        let name = format!("ColdConsumer{index:03}");
        std::fs::write(
            directory.join(format!("{name}.cs")),
            format!("public sealed class {name} {{ public {name}({CONTRACT} dependency) {{}} }}\n"),
        )
        .expect("consumer fixture");
        expected.insert(name);
    }

    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;

    let refreshed = dispatch(
        &state,
        "refresh_workspace",
        json!({ "workspaceRoot": root.path().to_string_lossy() }),
    );
    assert!(
        refreshed["result"]["structuredContent"]["refreshed"] == true,
        "refresh must establish the cold workspace boundary: {refreshed}"
    );

    // This is intentionally the first semantic query after refresh. No consumer
    // file is precompiled or published by test setup.
    let response = dispatch(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot": root.path().to_string_lossy(),
            "type": "reverse_edges",
            "domain": "builtin",
            "entity_type": "TypeRef",
            "name": CONTRACT
        }),
    );
    let result = response["result"]
        .as_object()
        .unwrap_or_else(|| panic!("workspace query must succeed: {response}"));
    assert_valid_mcp_envelope(result);
    let all_edges = result["structuredContent"]["edges"]
        .as_array()
        .expect("reverse edge array");

    assert!(all_edges.iter().any(|edge| {
        edge["relation"] == "HasBaseType"
            && edge["subject"]["name"] == "LargeColdContract"
            && edge["object"]["name"] == CONTRACT
    }));
    let constructor_edges: Vec<_> = all_edges
        .iter()
        .filter(|edge| edge["relation"] == "HasConstructorParameterType")
        .collect();

    assert_eq!(
        constructor_edges.len(),
        CONSUMER_COUNT,
        "{constructor_edges:#?}"
    );
    assert!(constructor_edges.iter().all(|edge| {
        edge["relation"] == "HasConstructorParameterType"
            && edge["subject"]["domain"] == "builtin"
            && edge["subject"]["entity_type"] == "Class"
            && edge["object"]["domain"] == "builtin"
            && edge["object"]["entity_type"] == "TypeRef"
            && edge["object"]["name"] == CONTRACT
            && edge["layer"] == "builtin"
    }));
    let actual: BTreeSet<String> = constructor_edges
        .iter()
        .map(|edge| {
            edge["subject"]["name"]
                .as_str()
                .expect("consumer subject name")
                .to_string()
        })
        .collect();
    assert_eq!(actual, expected);
    assert!(!actual.contains("LargeColdContract"));
}
