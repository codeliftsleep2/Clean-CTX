use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{json, Value};
use std::path::Path;

fn state() -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    crate::mcp::McpState::new(config)
}

fn call(state: &crate::mcp::McpState, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(9),
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

fn controller_facts(
    state: &crate::mcp::McpState,
    controller: &str,
) -> Vec<(SemanticRelation, String)> {
    state
        .workspace_index_read()
        .forward_edges_by_identity("dotnet", "Controller", controller)
        .into_iter()
        .filter(|edge| {
            matches!(
                edge.relation,
                SemanticRelation::HasRoute | SemanticRelation::ControllerAction
            )
        })
        .map(|edge| (edge.relation, edge.object.name.clone()))
        .collect()
}

#[test]
fn r9_aspnet_cross_file_ancestry_and_fact_lifecycle_converge() {
    let root = tempfile::TempDir::new().expect("workspace root");
    let base = root.path().join("BaseController.cs");
    let derived = root.path().join("OrdersEndpoint.cs");
    std::fs::write(
        &base,
        "using Microsoft.AspNetCore.Mvc; public class BaseController : ControllerBase {}\n",
    )
    .expect("base controller fixture");
    std::fs::write(
        &derived,
        "using Microsoft.AspNetCore.Mvc; [Route(\"orders\")] public class OrdersEndpoint : BaseController { [HttpGet] public void List() {} }\n",
    )
    .expect("derived controller fixture");
    let state = state();

    hydrate(&state, root.path(), "BaseController");
    hydrate(&state, root.path(), "OrdersEndpoint");
    assert_eq!(
        controller_facts(&state, "OrdersEndpoint"),
        vec![
            (SemanticRelation::HasRoute, "orders".into()),
            (SemanticRelation::ControllerAction, "List".into()),
        ]
    );

    std::fs::write(
        &base,
        "public class PlainBase {} public class BaseController : PlainBase {}\n",
    )
    .expect("remove ASP.NET ancestry");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "OrdersEndpoint");
    assert!(
        controller_facts(&state, "OrdersEndpoint").is_empty(),
        "upstream authority removal must retract controller facts"
    );

    std::fs::write(
        &base,
        "using Microsoft.AspNetCore.Mvc; public class BaseController : ControllerBase {}\n",
    )
    .expect("restore ASP.NET ancestry");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "BaseController");
    hydrate(&state, root.path(), "OrdersEndpoint");
    assert_eq!(controller_facts(&state, "OrdersEndpoint").len(), 2);

    std::fs::remove_file(&derived).expect("delete derived controller");
    refresh(&state, root.path());
    assert!(
        controller_facts(&state, "OrdersEndpoint").is_empty(),
        "owner deletion must retract controller facts"
    );
}

#[test]
fn r9_partial_controller_publishes_one_symbol_role_with_cross_part_payload() {
    let root = tempfile::TempDir::new().expect("workspace root");
    std::fs::write(
        root.path().join("PartialBase.cs"),
        "using Microsoft.AspNetCore.Mvc; public partial class PartialEndpoint : ControllerBase {}\n",
    )
    .expect("partial base fixture");
    std::fs::write(
        root.path().join("PartialPayload.cs"),
        "using Microsoft.AspNetCore.Mvc; [Route(\"partial\")] public partial class PartialEndpoint { [HttpGet] public void List() {} }\n",
    )
    .expect("partial payload fixture");
    let state = state();

    hydrate(&state, root.path(), "PartialEndpoint");
    let index = state.workspace_index_read();
    let roles = index
        .find_entities_by_name("PartialEndpoint")
        .into_iter()
        .filter(|entity| entity.domain == "dotnet" && entity.entity_type == "Controller")
        .collect::<Vec<_>>();
    assert_eq!(roles.len(), 1, "a composed partial symbol owns one role");
    drop(index);
    assert_eq!(
        controller_facts(&state, "PartialEndpoint"),
        vec![
            (SemanticRelation::HasRoute, "partial".into()),
            (SemanticRelation::ControllerAction, "List".into()),
        ],
        "payload from the non-role-owning part must remain projected"
    );
}
