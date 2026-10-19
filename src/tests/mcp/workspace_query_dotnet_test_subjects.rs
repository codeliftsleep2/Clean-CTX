//! LGR-001: .NET Tests relationships require exact Class declarations.

use super::*;

fn write(root: &tempfile::TempDir, file: &str, source: &str) {
    let source = if source.contains("[TestClass]")
        && !source.contains("Microsoft.VisualStudio.TestTools.UnitTesting")
    {
        format!("using Microsoft.VisualStudio.TestTools.UnitTesting;\n{source}")
    } else {
        source.to_string()
    };
    std::fs::write(root.path().join(file), source).unwrap();
}

fn edges(
    root: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    direction: &str,
    domain: &str,
    entity_type: &str,
    name: &str,
) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(134),
        "workspace_query",
        &json!({"arguments":{
            "type":direction, "domain":domain, "entity_type":entity_type,
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

fn test_edges(value: &serde_json::Value) -> Vec<&serde_json::Value> {
    value["edges"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|edge| edge["relation"] == "Tests")
        .collect()
}

fn assert_edge(value: &serde_json::Value, test: &str, subject: &str, file: &str) {
    let matching: Vec<_> = test_edges(value)
        .into_iter()
        .filter(|edge| {
            edge["subject"]["domain"] == "dotnet"
                && edge["subject"]["entity_type"] == "TestClass"
                && edge["subject"]["name"] == test
                && edge["object"]["domain"] == "builtin"
                && edge["object"]["entity_type"] == "Class"
                && edge["object"]["name"] == subject
                && edge["object"]["file"] == file
        })
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "expected one exact Tests edge: {value:?}"
    );
}

fn state() -> crate::mcp::McpState {
    crate::mcp::McpState::new(crate::tests::test_config())
}

#[test]
fn ghost_and_endpoint_occurrence_without_declaration_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root,
        "GhostTests.cs",
        "[TestClass] public class GhostTests { }",
    );
    write(
        &root,
        "UsesGhost.cs",
        "public class UsesGhost { public UsesGhost(Ghost value) {} }",
    );
    let state = state();
    state.workspace_index_lock().add_edges(
        &root.path().join("Endpoint.cs").to_string_lossy(),
        vec![crate::layers::meta::semantic::SemanticEdge {
            relation: crate::layers::meta::semantic::SemanticRelation::Selects,
            subject: crate::layers::meta::semantic::EntityRef::new(
                "example",
                "Owner",
                "EndpointOnly",
            ),
            object: crate::layers::meta::semantic::EntityRef::new("builtin", "Class", "Ghost"),
            layer: "test",
            call_evidence: None,
        }],
    );
    let result = edges(
        &root,
        &state,
        "forward_edges",
        "dotnet",
        "TestClass",
        "GhostTests",
    );
    assert!(
        test_edges(&result).is_empty(),
        "an endpoint is not a declaration: {result:?}"
    );
}

#[test]
fn conventional_subject_requires_and_preserves_exact_declaration_provenance() {
    let root = tempfile::tempdir().unwrap();
    write(&root, "OrderService.cs", "public class OrderService { }");
    write(
        &root,
        "OrderServiceTests.cs",
        "[TestClass] public class OrderServiceTests { }",
    );
    let state = state();
    let result = edges(
        &root,
        &state,
        "forward_edges",
        "dotnet",
        "TestClass",
        "OrderServiceTests",
    );
    let target = root
        .path()
        .join("OrderService.cs")
        .to_string_lossy()
        .into_owned();
    assert_edge(&result, "OrderServiceTests", "OrderService", &target);
    let repeated = edges(
        &root,
        &state,
        "forward_edges",
        "dotnet",
        "TestClass",
        "OrderServiceTests",
    );
    assert_edge(&repeated, "OrderServiceTests", "OrderService", &target);
    assert_eq!(test_edges(&repeated).len(), 1);
}

#[test]
fn persistent_constructed_subject_outranks_conflicting_suffix() {
    let root = tempfile::tempdir().unwrap();
    write(&root, "Billing.cs", "public class Billing { }");
    write(
        &root,
        "RealBillingService.cs",
        "public class RealBillingService { }",
    );
    write(
        &root,
        "BillingTests.cs",
        r#"[TestClass]
public class BillingTests {
    private RealBillingService _sut;
    [TestInitialize] public void Setup() { _sut = new RealBillingService(); }
}"#,
    );
    let result = edges(
        &root,
        &state(),
        "forward_edges",
        "dotnet",
        "TestClass",
        "BillingTests",
    );
    let target = root
        .path()
        .join("RealBillingService.cs")
        .to_string_lossy()
        .into_owned();
    assert_edge(&result, "BillingTests", "RealBillingService", &target);
    assert!(
        test_edges(&result)
            .iter()
            .all(|edge| edge["object"]["name"] != "Billing")
    );
}

#[test]
fn ambiguous_terminal_and_multiple_structured_candidates_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root,
        "A.cs",
        "namespace A { public class Shared {} } public class First {} ",
    );
    write(
        &root,
        "B.cs",
        "namespace B { public class Shared {} } public class Second {} ",
    );
    write(
        &root,
        "SharedTests.cs",
        "[TestClass] public class SharedTests { }",
    );
    write(
        &root,
        "MultiTests.cs",
        r#"[TestClass] public class MultiTests {
private First _first; private Second _second;
[TestInitialize] public void Setup() { _first = new First(); _second = new Second(); }
}"#,
    );
    let state = state();
    for name in ["SharedTests", "MultiTests"] {
        let result = edges(&root, &state, "forward_edges", "dotnet", "TestClass", name);
        assert!(
            test_edges(&result).is_empty(),
            "ambiguous evidence must fail closed: {result:?}"
        );
    }
}

#[test]
fn transient_construction_does_not_manufacture_a_subject() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root,
        "RealBillingService.cs",
        "public class RealBillingService { }",
    );
    write(
        &root,
        "BehaviorChecks.cs",
        "[TestClass] public class BehaviorChecks { public void Check() { var local = new RealBillingService(); } }",
    );
    let result = edges(
        &root,
        &state(),
        "forward_edges",
        "dotnet",
        "TestClass",
        "BehaviorChecks",
    );
    assert!(test_edges(&result).is_empty());
}

#[test]
fn removed_or_replaced_target_retracts_the_ephemeral_relationship() {
    let root = tempfile::tempdir().unwrap();
    write(&root, "Target.cs", "public class Target { }");
    write(
        &root,
        "TargetTests.cs",
        "[TestClass] public class TargetTests { }",
    );
    let state = state();
    let target = root.path().join("Target.cs").to_string_lossy().into_owned();
    assert_edge(
        &edges(
            &root,
            &state,
            "forward_edges",
            "dotnet",
            "TestClass",
            "TargetTests",
        ),
        "TargetTests",
        "Target",
        &target,
    );
    std::fs::write(
        root.path().join("Target.cs"),
        "public class Replacement { }",
    )
    .unwrap();
    crate::mcp::tool_handlers::hydration::reconcile_external_refresh_for_root(
        &state,
        &root.path().to_string_lossy(),
    );
    let current = edges(
        &root,
        &state,
        "forward_edges",
        "dotnet",
        "TestClass",
        "TargetTests",
    );
    assert!(
        test_edges(&current).is_empty(),
        "stale target survived: {current:?}"
    );
}

#[test]
fn reverse_query_exposes_the_same_declaration_proven_relationship() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root,
        "RealBillingService.cs",
        "public class RealBillingService { }",
    );
    write(
        &root,
        "BillingTests.cs",
        "[TestClass] public class BillingTests { private RealBillingService _sut; public BillingTests() { _sut = new RealBillingService(); } }",
    );
    let result = edges(
        &root,
        &state(),
        "reverse_edges",
        "builtin",
        "Class",
        "RealBillingService",
    );
    let target = root
        .path()
        .join("RealBillingService.cs")
        .to_string_lossy()
        .into_owned();
    assert_edge(&result, "BillingTests", "RealBillingService", &target);
}
