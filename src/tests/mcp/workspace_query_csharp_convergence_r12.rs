use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

fn state() -> crate::mcp::McpState {
    state_with_roots(&[])
}

fn state_with_roots(additional: &[&Path]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional
        .iter()
        .map(|root| root.to_string_lossy().into_owned())
        .collect();
    crate::mcp::McpState::new(config)
}

fn call(state: &crate::mcp::McpState, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(12),
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
    assert!(response.get("error").is_none(), "{response:#}");
}

fn refresh(state: &crate::mcp::McpState, root: &Path) {
    let response = call(state, "refresh_workspace", json!({ "workspaceRoot": root }));
    assert_eq!(response["result"]["structuredContent"]["refreshed"], true);
}

fn framework_facts(state: &crate::mcp::McpState) -> BTreeSet<(String, String, String)> {
    let index = state.workspace_index_read();
    let mut facts = BTreeSet::new();
    for (entity_type, name) in [
        ("DbContext", "AppContext"),
        ("MapperProfile", "Maps"),
        ("Hub", "Events"),
        ("Controller", "OrdersEndpoint"),
        ("TestClass", "OrdersTests"),
    ] {
        for edge in index.forward_edges_by_identity("dotnet", entity_type, name) {
            facts.insert((
                format!("{entity_type}/{name}"),
                format!("{:?}", edge.relation),
                format!("{}/{}", edge.object.entity_type, edge.object.name),
            ));
        }
        if index
            .find_entities_by_name(name)
            .iter()
            .any(|entity| entity.domain == "dotnet" && entity.entity_type == entity_type)
        {
            facts.insert((
                format!("{entity_type}/{name}"),
                "Role".into(),
                format!("{entity_type}/{name}"),
            ));
        }
    }
    facts
}

fn hydrate_all(state: &crate::mcp::McpState, root: &Path) {
    for name in [
        "AppContext",
        "Maps",
        "Events",
        "OrdersEndpoint",
        "OrdersTests",
    ] {
        hydrate(state, root, name);
    }
}

#[test]
fn r12_workspace_generation_converges_and_consumer_retraction_is_isolated() {
    let root = tempfile::TempDir::new().expect("workspace root");
    let profile = root.path().join("Maps.cs");
    let hub = root.path().join("Events.cs");
    std::fs::write(
        root.path().join("AppContext.cs"),
        "public class AppContext : global::Microsoft.EntityFrameworkCore.DbContext {}\n",
    )
    .expect("EF fixture");
    std::fs::write(
        &profile,
        "public class Maps : global::AutoMapper.Profile { public Maps() { CreateMap<Source, Destination>(); } }\n",
    )
    .expect("AutoMapper fixture");
    std::fs::write(
        &hub,
        "public class Events : global::Microsoft.AspNetCore.SignalR.Hub { public void Broadcast() {} }\n",
    )
    .expect("SignalR fixture");
    std::fs::write(
        root.path().join("OrdersEndpoint.cs"),
        "[global::Microsoft.AspNetCore.Mvc.ApiControllerAttribute] [global::Microsoft.AspNetCore.Mvc.RouteAttribute(\"orders\")] public class OrdersEndpoint { [global::Microsoft.AspNetCore.Mvc.HttpGetAttribute] public void List() {} }\n",
    )
    .expect("ASP.NET fixture");
    std::fs::write(
        root.path().join("OrdersTests.cs"),
        "[global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute] public class OrdersTests {}\n",
    )
    .expect("MSTest fixture");
    let state = state();

    hydrate_all(&state, root.path());
    let cold = framework_facts(&state);
    assert!(cold.iter().any(|fact| fact.0 == "DbContext/AppContext"));
    assert!(cold.iter().any(|fact| fact.0 == "MapperProfile/Maps"));
    assert!(cold.iter().any(|fact| fact.0 == "Hub/Events"));
    assert!(
        cold.iter()
            .any(|fact| fact.0 == "Controller/OrdersEndpoint")
    );
    assert!(cold.iter().any(|fact| fact.0 == "TestClass/OrdersTests"));

    hydrate_all(&state, root.path());
    assert_eq!(
        framework_facts(&state),
        cold,
        "warm truth must equal cold truth"
    );

    std::fs::write(
        &profile,
        "namespace Local { public class Profile {} } public class Maps : Local.Profile { public Maps() { CreateMap<Source, Destination>(); } }\n",
    )
    .expect("replace profile authority");
    refresh(&state, root.path());
    hydrate_all(&state, root.path());
    let without_profile = framework_facts(&state);
    assert!(
        without_profile
            .iter()
            .all(|fact| fact.0 != "MapperProfile/Maps")
    );
    assert_eq!(
        without_profile
            .iter()
            .filter(|fact| fact.0 != "MapperProfile/Maps")
            .collect::<Vec<_>>(),
        cold.iter()
            .filter(|fact| fact.0 != "MapperProfile/Maps")
            .collect::<Vec<_>>(),
        "one consumer's authority removal cannot disturb another consumer"
    );

    std::fs::remove_file(&hub).expect("delete hub owner");
    refresh(&state, root.path());
    hydrate_all(&state, root.path());
    let after_deletion = framework_facts(&state);
    assert!(after_deletion.iter().all(|fact| fact.0 != "Hub/Events"));
    assert!(
        after_deletion
            .iter()
            .any(|fact| fact.0 == "DbContext/AppContext")
    );
    assert!(
        after_deletion
            .iter()
            .any(|fact| fact.0 == "Controller/OrdersEndpoint")
    );
    assert!(
        after_deletion
            .iter()
            .any(|fact| fact.0 == "TestClass/OrdersTests")
    );
    assert!(
        after_deletion
            .iter()
            .all(|fact| { fact.1 != format!("{:?}", SemanticRelation::HubMethodTargets) })
    );
}

#[test]
fn r12_cross_root_truth_is_independent_of_consumer_query_order() {
    let primary = tempfile::TempDir::new().expect("primary root");
    let additional = tempfile::TempDir::new().expect("additional root");
    std::fs::write(
        additional.path().join("FrameworkBases.cs"),
        "public class RealtimeBase : global::Microsoft.AspNetCore.SignalR.Hub {}\n",
    )
    .expect("cross-root base fixture");
    std::fs::write(
        additional.path().join("Events.cs"),
        "public class Events : RealtimeBase { public void Broadcast() {} }\n",
    )
    .expect("additional-root derived fixture");
    std::fs::write(
        primary.path().join("OrdersEndpoint.cs"),
        "[global::Microsoft.AspNetCore.Mvc.ApiControllerAttribute] public class OrdersEndpoint {}\n",
    )
    .expect("controller fixture");

    let signalr_first = state_with_roots(&[additional.path()]);
    hydrate(&signalr_first, primary.path(), "Events");
    hydrate(&signalr_first, primary.path(), "OrdersEndpoint");
    let first = framework_facts(&signalr_first);

    let controller_first = state_with_roots(&[additional.path()]);
    hydrate(&controller_first, primary.path(), "OrdersEndpoint");
    hydrate(&controller_first, primary.path(), "RealtimeBase");
    hydrate(&controller_first, primary.path(), "Events");
    let second = framework_facts(&controller_first);

    assert_eq!(first, second, "query history cannot alter resolved truth");
    assert!(first.iter().any(|fact| {
        fact.0 == "Hub/Events"
            && fact.1 == format!("{:?}", SemanticRelation::HubMethodTargets)
            && fact.2 == "HubMethod/Broadcast"
    }));
    assert!(
        first
            .iter()
            .any(|fact| fact.0 == "Controller/OrdersEndpoint" && fact.1 == "Role")
    );
}
