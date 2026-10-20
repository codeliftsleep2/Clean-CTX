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
        &json!(11),
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

fn role_count(state: &crate::mcp::McpState, name: &str) -> usize {
    state
        .workspace_index_read()
        .find_entities_by_name(name)
        .into_iter()
        .filter(|entity| entity.domain == "dotnet" && entity.entity_type == "Controller")
        .count()
}

fn facts(state: &crate::mcp::McpState, name: &str) -> Vec<(SemanticRelation, String)> {
    state
        .workspace_index_read()
        .forward_edges_by_identity("dotnet", "Controller", name)
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

const ROLE: &str = r#"
using ControllerMarker = Microsoft.AspNetCore.Mvc.ControllerAttribute;
[ControllerMarker] public partial class OrdersEndpoint {}
"#;

const PAYLOAD: &str = r#"
using RouteMarker = Microsoft.AspNetCore.Mvc.RouteAttribute;
using GetMarker = Microsoft.AspNetCore.Mvc.HttpGetAttribute;
[RouteMarker("orders")] public partial class OrdersEndpoint {
    [GetMarker] public void List() {}
}
"#;

#[test]
fn r11_aspnet_attribute_parts_reconcile_role_route_action_and_owner_lifecycle() {
    let root = tempfile::TempDir::new().expect("workspace root");
    let role = root.path().join("ControllerRole.cs");
    let payload = root.path().join("ControllerPayload.cs");
    std::fs::write(&role, ROLE).expect("role fixture");
    std::fs::write(&payload, PAYLOAD).expect("payload fixture");
    let state = state();

    hydrate(&state, root.path(), "OrdersEndpoint");
    assert_eq!(role_count(&state, "OrdersEndpoint"), 1);
    assert_eq!(
        facts(&state, "OrdersEndpoint"),
        vec![
            (SemanticRelation::HasRoute, "orders".into()),
            (SemanticRelation::ControllerAction, "List".into()),
        ]
    );

    std::fs::write(
        &payload,
        r#"
using RouteMarker = Other.RouteAttribute;
using GetMarker = Other.HttpGetAttribute;
namespace Other {
    public class RouteAttribute : System.Attribute { public RouteAttribute(string value) {} }
    public class HttpGetAttribute : System.Attribute {}
}
[RouteMarker("orders")] public partial class OrdersEndpoint {
    [GetMarker] public void List() {}
}
"#,
    )
    .expect("retarget payload aliases");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "OrdersEndpoint");
    assert_eq!(role_count(&state, "OrdersEndpoint"), 1);
    assert!(facts(&state, "OrdersEndpoint").is_empty());

    std::fs::write(&payload, PAYLOAD).expect("restore payload aliases");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "OrdersEndpoint");
    assert_eq!(facts(&state, "OrdersEndpoint").len(), 2);

    std::fs::write(
        &role,
        "using ControllerMarker = Other.ControllerAttribute; namespace Other { public class ControllerAttribute : System.Attribute {} } [ControllerMarker] public partial class OrdersEndpoint {}\n",
    )
    .expect("retarget role alias");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "OrdersEndpoint");
    assert_eq!(role_count(&state, "OrdersEndpoint"), 0);
    assert!(facts(&state, "OrdersEndpoint").is_empty());

    std::fs::write(&role, ROLE).expect("restore role alias");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "OrdersEndpoint");
    assert_eq!(role_count(&state, "OrdersEndpoint"), 1);
    assert_eq!(facts(&state, "OrdersEndpoint").len(), 2);

    std::fs::remove_file(&payload).expect("delete payload owner");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "OrdersEndpoint");
    assert_eq!(role_count(&state, "OrdersEndpoint"), 1);
    assert!(facts(&state, "OrdersEndpoint").is_empty());

    std::fs::remove_file(&role).expect("delete role owner");
    refresh(&state, root.path());
    assert_eq!(role_count(&state, "OrdersEndpoint"), 0);
}
