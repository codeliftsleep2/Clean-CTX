use crate::layers::meta::semantic::SemanticRelation;
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
        &json!(8),
        tool,
        &json!({ "arguments": arguments }),
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
        json!({
            "workspaceRoot": root,
            "type": "find_entities",
            "name": name
        }),
    );
    assert!(response.get("error").is_none(), "{response}");
}

fn refresh(state: &crate::mcp::McpState, root: &Path) {
    let response = call(state, "refresh_workspace", json!({ "workspaceRoot": root }));
    assert_eq!(response["result"]["structuredContent"]["refreshed"], true);
}

fn targets(state: &crate::mcp::McpState, hub: &str) -> Vec<String> {
    state
        .workspace_index_read()
        .forward_edges_by_identity("dotnet", "Hub", hub)
        .into_iter()
        .filter(|edge| edge.relation == SemanticRelation::HubMethodTargets)
        .map(|edge| edge.object.name.clone())
        .collect()
}

fn method_contexts(state: &crate::mcp::McpState, file: &Path, method_name: &str) -> Vec<String> {
    let owner = state.semantic_owner_path(&file.to_string_lossy());
    let alias = state.get_or_create_alias(owner);
    let context = state.ir_context_read();
    let Some(instructions) = context.get_ir(&alias) else {
        return Vec::new();
    };
    let Some(method_id) = instructions.iter().find_map(|tuple| {
        (tuple.first().is_some_and(|opcode| opcode == "DEF_M")
            && tuple.get(3).is_some_and(|name| name == method_name))
        .then(|| tuple[2].clone())
    }) else {
        return Vec::new();
    };
    instructions
        .iter()
        .filter(|tuple| {
            tuple.first().is_some_and(|opcode| opcode == "CTX") && tuple.get(1) == Some(&method_id)
        })
        .filter_map(|tuple| tuple.get(2).cloned())
        .collect()
}

#[test]
fn r8_signalr_cross_file_ancestry_and_method_lifecycle_converge() {
    let root = tempfile::TempDir::new().expect("workspace root");
    let base = root.path().join("BaseHub.cs");
    let derived = root.path().join("DerivedHub.cs");
    std::fs::write(
        &base,
        "using Microsoft.AspNetCore.SignalR; public class BaseHub : Hub<IClient> {}\n",
    )
    .expect("base hub fixture");
    std::fs::write(
        &derived,
        "public class DerivedHub : BaseHub { public async Task Broadcast() {} }\n",
    )
    .expect("derived hub fixture");
    let state = state();

    hydrate(&state, root.path(), "BaseHub");
    hydrate(&state, root.path(), "DerivedHub");
    assert_eq!(targets(&state, "DerivedHub"), vec!["Broadcast"]);
    assert_eq!(
        method_contexts(&state, &derived, "Broadcast"),
        vec!["async", "realtime"]
    );

    std::fs::write(
        &base,
        "public class PlainBase {} public class BaseHub : PlainBase {}\n",
    )
    .expect("remove SignalR ancestry");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "DerivedHub");
    assert!(
        targets(&state, "DerivedHub").is_empty(),
        "upstream authority removal must retract hub targets"
    );
    assert_eq!(
        method_contexts(&state, &derived, "Broadcast"),
        vec!["async"],
        "SignalR retraction must preserve independent async authority"
    );

    std::fs::write(
        &base,
        "using Microsoft.AspNetCore.SignalR; public class BaseHub : Hub<IClient> {}\n",
    )
    .expect("restore SignalR ancestry");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "BaseHub");
    hydrate(&state, root.path(), "DerivedHub");
    assert_eq!(targets(&state, "DerivedHub"), vec!["Broadcast"]);
    assert_eq!(
        method_contexts(&state, &derived, "Broadcast"),
        vec!["async", "realtime"]
    );

    std::fs::remove_file(&derived).expect("delete derived hub");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "BaseHub");
    assert!(
        targets(&state, "DerivedHub").is_empty(),
        "owner deletion must retract hub targets"
    );
    assert!(
        method_contexts(&state, &derived, "Broadcast").is_empty(),
        "owner deletion must retract canonical execution contexts"
    );
}
