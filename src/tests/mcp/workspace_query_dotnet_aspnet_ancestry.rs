//! LGR-004: ASP.NET controller meaning follows exact declared ancestry.

use super::*;

fn write(root: &tempfile::TempDir, file: &str, source: &str) {
    std::fs::write(root.path().join(file), source).unwrap();
}

fn state() -> crate::mcp::McpState {
    crate::mcp::McpState::new(crate::tests::test_config())
}

fn publish(root: &tempfile::TempDir, state: &crate::mcp::McpState, file: &str) {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(401),
        "provide_code_context",
        &json!({"arguments":{
            "workspaceRoot":root.path(), "filePath":file, "fidelity":"edit"
        }}),
        state,
    );
    let response = pop_response();
    assert!(
        response.get("error").is_none(),
        "publication failed: {response:?}"
    );
}

fn query(
    root: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    query_type: &str,
    entity_type: &str,
    name: &str,
) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(402),
        "workspace_query",
        &json!({"arguments":{
            "type":query_type, "domain":"dotnet", "entity_type":entity_type,
            "name":name, "workspaceRoot":root.path()
        }}),
        state,
    );
    let response = pop_response();
    assert!(
        response.get("error").is_none(),
        "query failed: {response:?}"
    );
    response["result"]["structuredContent"].clone()
}

fn publish_all(root: &tempfile::TempDir, state: &crate::mcp::McpState, files: &[&str]) {
    for file in files {
        publish(root, state, file);
    }
}

fn controller_count(result: &serde_json::Value, name: &str) -> usize {
    result["entities"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|entity| {
            entity["domain"] == "dotnet"
                && entity["entity_type"] == "Controller"
                && entity["name"] == name
        })
        .count()
}

#[test]
fn exact_multi_hop_ancestry_projects_controller_route_and_actions() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root,
        "SharedHttpBase.cs",
        "using Microsoft.AspNetCore.Mvc; namespace Api; public class SharedHttpBase : ControllerBase {}",
    );
    write(
        &root,
        "SecondHttpBase.cs",
        "namespace Api; public class SecondHttpBase : SharedHttpBase {}",
    );
    write(
        &root,
        "OrdersEndpoint.cs",
        "using Microsoft.AspNetCore.Mvc; namespace Api; [Route(\"orders\")] public class OrdersEndpoint : SecondHttpBase { [HttpGet(\"one\")] public int GetOne() { return 1; } [HttpPost] public void Add() {} }",
    );
    let state = state();
    publish_all(
        &root,
        &state,
        &[
            "SharedHttpBase.cs",
            "SecondHttpBase.cs",
            "OrdersEndpoint.cs",
        ],
    );

    let entities = query(
        &root,
        &state,
        "find_entities",
        "Controller",
        "OrdersEndpoint",
    );
    assert_eq!(controller_count(&entities, "OrdersEndpoint"), 1);
    let edges = query(
        &root,
        &state,
        "forward_edges",
        "Controller",
        "OrdersEndpoint",
    );
    let edges = edges["edges"].as_array().unwrap();
    assert_eq!(
        edges
            .iter()
            .filter(|edge| edge["relation"] == "HasRoute")
            .count(),
        1
    );
    assert_eq!(
        edges
            .iter()
            .filter(|edge| edge["relation"] == "ControllerAction")
            .count(),
        2
    );
}

#[test]
fn project_owned_controllerbase_name_is_not_framework_authority() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root,
        "Fake.cs",
        "namespace Local; public class ControllerBase {}",
    );
    write(
        &root,
        "Endpoint.cs",
        "using Microsoft.AspNetCore.Mvc; namespace Local; public class Endpoint : ControllerBase { [HttpGet] public void Get() {} }",
    );
    let state = state();
    publish_all(&root, &state, &["Fake.cs", "Endpoint.cs"]);
    let result = query(&root, &state, "find_entities", "Controller", "Endpoint");
    assert_eq!(controller_count(&result, "Endpoint"), 0);
}

#[test]
fn missing_ambiguous_and_cyclic_hops_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root,
        "Missing.cs",
        "public class MissingEndpoint : UnknownBase {}",
    );
    write(
        &root,
        "A.cs",
        "namespace One; public class SharedBase : ControllerBase {}",
    );
    write(
        &root,
        "B.cs",
        "namespace Two; public class SharedBase : ControllerBase {}",
    );
    write(
        &root,
        "Ambiguous.cs",
        "public class AmbiguousEndpoint : SharedBase {}",
    );
    write(&root, "CycleA.cs", "public class CycleA : CycleB {}");
    write(&root, "CycleB.cs", "public class CycleB : CycleA {}");
    let state = state();
    publish_all(
        &root,
        &state,
        &[
            "Missing.cs",
            "A.cs",
            "B.cs",
            "Ambiguous.cs",
            "CycleA.cs",
            "CycleB.cs",
        ],
    );
    for name in ["MissingEndpoint", "AmbiguousEndpoint", "CycleA"] {
        let result = query(&root, &state, "find_entities", "Controller", name);
        assert_eq!(controller_count(&result, name), 0, "unexpected: {result:?}");
    }
}

#[test]
fn exact_namespace_import_selects_one_project_base() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root,
        "Good.cs",
        "using Microsoft.AspNetCore.Mvc; namespace Good; public class SharedBase : ControllerBase {}",
    );
    write(
        &root,
        "Other.cs",
        "namespace Other; public class SharedBase {}",
    );
    write(
        &root,
        "Endpoint.cs",
        "using Good; public class QualifiedEndpoint : SharedBase {}",
    );
    let state = state();
    publish_all(&root, &state, &["Good.cs", "Other.cs", "Endpoint.cs"]);
    let result = query(
        &root,
        &state,
        "find_entities",
        "Controller",
        "QualifiedEndpoint",
    );
    assert_eq!(controller_count(&result, "QualifiedEndpoint"), 1);
}

#[test]
fn replacing_framework_base_retracts_projection() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root,
        "Base.cs",
        "using Microsoft.AspNetCore.Mvc; public class ProjectBase : ControllerBase {}",
    );
    write(
        &root,
        "Endpoint.cs",
        "public class Endpoint : ProjectBase {}",
    );
    let state = state();
    publish_all(&root, &state, &["Base.cs", "Endpoint.cs"]);
    assert_eq!(
        controller_count(
            &query(&root, &state, "find_entities", "Controller", "Endpoint"),
            "Endpoint",
        ),
        1
    );
    std::fs::write(root.path().join("Base.cs"), "public class Replacement {}").unwrap();
    crate::mcp::tool_handlers::hydration::reconcile_external_refresh_for_root(
        &state,
        &root.path().to_string_lossy(),
    );
    let result = query(&root, &state, "find_entities", "Controller", "Endpoint");
    assert_eq!(controller_count(&result, "Endpoint"), 0);
}
