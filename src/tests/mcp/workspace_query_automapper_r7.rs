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
        &json!(7),
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

fn mappings(state: &crate::mcp::McpState, profile: &str) -> Vec<(SemanticRelation, String)> {
    state
        .workspace_index_read()
        .forward_edges_by_identity("dotnet", "MapperProfile", profile)
        .into_iter()
        .filter(|edge| {
            matches!(
                edge.relation,
                SemanticRelation::MapsFrom | SemanticRelation::MapsTo
            )
        })
        .map(|edge| (edge.relation, edge.object.name.clone()))
        .collect()
}

#[test]
fn r7_automapper_cross_file_ancestry_and_mapping_lifecycle_converge() {
    let root = tempfile::TempDir::new().expect("workspace root");
    let base = root.path().join("BaseProfile.cs");
    let derived = root.path().join("DerivedProfile.cs");
    std::fs::write(
        &base,
        "using AutoMapper; public class BaseProfile : Profile {}\n",
    )
    .expect("base profile fixture");
    std::fs::write(
        &derived,
        "public class DerivedProfile : BaseProfile { void Configure() { CreateMap<Entity, EntityDto>(); } }\n",
    )
    .expect("derived profile fixture");
    let state = state();

    hydrate(&state, root.path(), "BaseProfile");
    hydrate(&state, root.path(), "DerivedProfile");
    let initial = mappings(&state, "DerivedProfile");
    assert_eq!(
        initial.len(),
        2,
        "resolved ancestry must authorize both mappings"
    );
    assert!(initial.contains(&(SemanticRelation::MapsFrom, "Entity".into())));
    assert!(initial.contains(&(SemanticRelation::MapsTo, "EntityDto".into())));

    std::fs::write(
        &base,
        "public class PlainBase {} public class BaseProfile : PlainBase {}\n",
    )
    .expect("remove AutoMapper ancestry");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "DerivedProfile");
    assert!(
        mappings(&state, "DerivedProfile").is_empty(),
        "upstream authority removal must retract mappings"
    );

    std::fs::write(
        &base,
        "using AutoMapper; public class BaseProfile : Profile {}\n",
    )
    .expect("restore AutoMapper ancestry");
    refresh(&state, root.path());
    hydrate(&state, root.path(), "BaseProfile");
    hydrate(&state, root.path(), "DerivedProfile");
    assert_eq!(mappings(&state, "DerivedProfile").len(), 2);

    std::fs::remove_file(derived).expect("delete derived profile");
    refresh(&state, root.path());
    assert!(
        mappings(&state, "DerivedProfile").is_empty(),
        "owner deletion must retract resolved mappings"
    );
}
