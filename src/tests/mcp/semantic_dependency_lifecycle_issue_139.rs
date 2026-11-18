//! Issue #139 cross-file semantic dependency lifecycle regressions.

use crate::layers::meta::semantic::{
    EntityRef, SemanticDependency, SemanticEdge, SemanticRelation,
};
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn route_edge() -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::RouteMapsTo,
        subject: EntityRef::new("angular", "Route", "lazy"),
        object: EntityRef::new("angular", "Component", "LazyComponent"),
        layer: "angular",
        call_evidence: None,
    }
}

fn assert_route_edge(edges: &[SemanticEdge]) {
    assert_eq!(edges.len(), 1);
    let edge = &edges[0];
    assert_eq!(edge.relation, SemanticRelation::RouteMapsTo);
    assert_eq!(edge.subject.domain, "angular");
    assert_eq!(edge.subject.entity_type, "Route");
    assert_eq!(edge.subject.name, "lazy");
    assert_eq!(edge.object.domain, "angular");
    assert_eq!(edge.object.entity_type, "Component");
    assert_eq!(edge.object.name, "LazyComponent");
}
fn state() -> crate::mcp::McpState {
    crate::mcp::McpState::new(crate::tests::test_config())
}

fn remember_external(state: &crate::mcp::McpState, alias: &str, dependency: &std::path::Path) {
    state.remember_semantic_projection(
        alias,
        vec![route_edge()],
        vec![SemanticDependency::current(
            dependency.to_string_lossy().into_owned(),
        )],
    );
}

#[test]
fn unchanged_external_target_keeps_cached_semantics_valid() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("lazy.component.ts");
    std::fs::write(&target, "export class LazyComponent {}").unwrap();
    let state = state();
    remember_external(&state, "routes", &target);

    let snapshot = state.semantic_snapshot("routes").expect("current snapshot");
    assert_route_edge(&snapshot.edges);
    assert_eq!(snapshot.dependencies.len(), 1);
}

#[test]
fn changed_external_target_invalidates_cached_semantics() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("lazy.component.ts");
    std::fs::write(&target, "export class LazyComponent {}").unwrap();
    let state = state();
    remember_external(&state, "routes", &target);

    std::fs::write(&target, "export class ReplacementComponent {}").unwrap();
    assert!(state.semantic_snapshot("routes").is_none());
}

#[test]
fn deleted_external_target_invalidates_cached_semantics() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("lazy.component.ts");
    std::fs::write(&target, "export class LazyComponent {}").unwrap();
    let state = state();
    remember_external(&state, "routes", &target);

    std::fs::remove_file(&target).unwrap();
    assert!(state.semantic_snapshot("routes").is_none());
}

#[test]
fn restored_external_target_can_publish_current_semantics_again() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("lazy.component.ts");
    std::fs::write(&target, "export class LazyComponent {}").unwrap();
    let state = state();
    remember_external(&state, "routes", &target);
    std::fs::remove_file(&target).unwrap();
    assert!(state.semantic_snapshot("routes").is_none());

    std::fs::write(&target, "export class LazyComponent { restored = true; }").unwrap();
    remember_external(&state, "routes", &target);
    assert!(state.semantic_snapshot("routes").is_some());
}

#[test]
fn purely_local_semantic_snapshot_retains_existing_reuse() {
    let state = state();
    state.remember_semantic_edges("local", vec![route_edge()]);

    let snapshot = state.semantic_snapshot("local").expect("local snapshot");
    assert_route_edge(&snapshot.edges);
    assert!(snapshot.dependencies.is_empty());
}

fn call(state: &crate::mcp::McpState, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(139), tool, &json!({"arguments": arguments}), state);
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("tool response");
    assert!(response.get("error").is_none(), "{tool}: {response}");
    response["result"].clone()
}

fn publish(state: &crate::mcp::McpState, root: &std::path::Path, route: &std::path::Path) {
    call(
        state,
        "provide_code_context",
        json!({"workspaceRoot": root, "filePath": route, "fidelity": "high"}),
    );
}

fn lazy_edges(state: &crate::mcp::McpState) -> Vec<SemanticEdge> {
    state
        .workspace_index_lock()
        .forward_edges_by_identity("angular", "Route", "lazy")
        .into_iter()
        .cloned()
        .collect()
}

#[test]
fn refresh_retraction_then_republish_cannot_resurrect_invalid_cross_file_evidence() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let route = root.path().join("app.routes.ts");
    let target = root.path().join("lazy.component.ts");
    let padding = "// semantic lifecycle padding\n".repeat(100);
    std::fs::write(
        &route,
        format!(
            "import {{ Routes }} from '@angular/router';\nexport const routes: Routes = [{{ path: 'lazy', loadComponent: () => import('./lazy.component').then(m => m.LazyComponent) }}];\n{padding}"
        ),
    )
    .unwrap();
    let component = "import { Component } from '@angular/core';\n@Component({selector:'lazy-probe', template:''}) export class LazyComponent {}";
    std::fs::write(&target, component).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;

    publish(&state, root.path(), &route);
    assert!(lazy_edges(&state).iter().any(|edge| {
        edge.relation == SemanticRelation::RouteMapsTo && edge.object.name == "LazyComponent"
    }));

    std::fs::remove_file(&target).unwrap();
    call(
        &state,
        "refresh_workspace",
        json!({"workspaceRoot": root.path()}),
    );
    publish(&state, root.path(), &route);
    assert!(
        lazy_edges(&state).is_empty(),
        "deleted authority was resurrected"
    );

    std::fs::write(&target, component).unwrap();
    call(
        &state,
        "refresh_workspace",
        json!({"workspaceRoot": root.path()}),
    );
    publish(&state, root.path(), &route);
    assert!(lazy_edges(&state).iter().any(|edge| {
        edge.relation == SemanticRelation::RouteMapsTo && edge.object.name == "LazyComponent"
    }));
}
