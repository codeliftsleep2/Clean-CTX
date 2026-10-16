//! Phase 4 RED regressions for authoritative Angular Component/Template linkage.
//!
//! These tests use the production MCP publication, query, and refresh paths.
//! The declaring TypeScript file owns HasTemplate; external Template identity is
//! the canonical physical/semantic-owner path, never a basename or workspace-
//! relative presentation path.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn state() -> crate::mcp::McpState {
    crate::mcp::McpState::new(crate::tests::test_config())
}

fn call(state: &crate::mcp::McpState, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("MCP response")
}

fn publish(state: &crate::mcp::McpState, root: &Path, component: &Path) {
    let response = call(
        state,
        "provide_code_context",
        json!({
            "workspaceRoot": root,
            "filePath": component,
            "fidelity": "edit"
        }),
    );
    assert!(
        response.get("error").is_none(),
        "component publication must succeed: {response}"
    );
}

fn refresh(state: &crate::mcp::McpState, root: &Path) {
    let response = call(state, "refresh_workspace", json!({ "workspaceRoot": root }));
    assert_eq!(
        response["result"]["structuredContent"]["refreshed"], true,
        "refresh must succeed: {response}"
    );
}

fn query_edges(
    state: &crate::mcp::McpState,
    root: &Path,
    query_type: &str,
    domain: &str,
    entity_type: &str,
    name: &str,
) -> Value {
    let response = call(
        state,
        "workspace_query",
        json!({
            "type": query_type,
            "workspaceRoot": root,
            "domain": domain,
            "entity_type": entity_type,
            "name": name
        }),
    );
    assert!(
        response.get("error").is_none(),
        "{query_type} must succeed for {domain}/{entity_type}/{name}: {response}"
    );
    response["result"]["structuredContent"]["edges"].clone()
}

fn canonical(path: &Path) -> String {
    crate::dictionary::path::canonical_identity_key(&path.to_string_lossy())
}

fn inline_identity(component: &Path, class_name: &str) -> String {
    format!("inline:{}#component={class_name}", canonical(component))
}

fn component_source(class_name: &str, metadata: &str) -> String {
    format!(
        "import {{ Component }} from '@angular/core';\n\
         @Component({{ selector: 'x-host', {metadata} }})\n\
         export class {class_name} {{}}\n"
    )
}

fn write_component(path: &Path, class_name: &str, metadata: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("component directory");
    }
    std::fs::write(path, component_source(class_name, metadata)).expect("component fixture");
}

fn has_template_to(edges: &Value, template_name: &str) -> bool {
    edges.as_array().is_some_and(|edges| {
        edges.iter().any(|edge| {
            edge["relation"] == "HasTemplate"
                && edge["object"]["domain"] == "angular"
                && edge["object"]["entity_type"] == "Template"
                && edge["object"]["name"] == template_name
        })
    })
}

fn has_template_from(edges: &Value, component_name: &str) -> bool {
    edges.as_array().is_some_and(|edges| {
        edges.iter().any(|edge| {
            edge["relation"] == "HasTemplate"
                && edge["subject"]["domain"] == "angular"
                && edge["subject"]["entity_type"] == "Component"
                && edge["subject"]["name"] == component_name
        })
    })
}

fn has_template_edges(edges: &Value) -> Vec<&Value> {
    edges
        .as_array()
        .expect("edges array")
        .iter()
        .filter(|edge| edge["relation"] == "HasTemplate")
        .collect()
}

#[test]
fn external_template_variants_use_canonical_identity_forward_and_reverse() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let fixtures = [
        (
            "ConventionalHost",
            PathBuf::from("conventional.component.ts"),
            PathBuf::from("conventional.component.html"),
        ),
        (
            "ArbitraryHost",
            PathBuf::from("arbitrary.ts"),
            PathBuf::from("detail-page.html"),
        ),
        (
            "NestedHost",
            PathBuf::from("features/host/nested.ts"),
            PathBuf::from("features/templates/nested-view.html"),
        ),
    ];

    for (class_name, component_rel, template_rel) in fixtures {
        let component = root.path().join(&component_rel);
        let template = root.path().join(&template_rel);
        std::fs::create_dir_all(template.parent().expect("template parent"))
            .expect("template directory");
        std::fs::write(&template, "<child-view></child-view>").expect("template fixture");
        let written =
            relative_fixture_path(&template, component.parent().expect("component parent"))
                .expect("relative template path")
                .to_string_lossy()
                .replace('\\', "/");
        write_component(
            &component,
            class_name,
            &format!("templateUrl: './{written}'"),
        );
        publish(&state, root.path(), &component);

        let template_name = canonical(&template);
        let forward = query_edges(
            &state,
            root.path(),
            "forward_edges",
            "angular",
            "Component",
            class_name,
        );
        assert!(
            has_template_to(&forward, &template_name),
            "{class_name} must link to canonical Template {template_name}: {forward}"
        );
        assert!(
            !has_template_to(&forward, &template_rel.to_string_lossy()),
            "workspace-relative presentation path must not become identity: {forward}"
        );

        let reverse = query_edges(
            &state,
            root.path(),
            "reverse_edges",
            "angular",
            "Template",
            &template_name,
        );
        assert!(
            has_template_from(&reverse, class_name),
            "reverse Template query must find {class_name}: {reverse}"
        );
    }
}

#[test]
fn inline_template_identity_is_deterministic_and_never_aliases_external_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let component = root.path().join("inline-host.ts");
    write_component(
        &component,
        "InlineHost",
        r#"template: '<child-view></child-view>'"#,
    );
    publish(&state, root.path(), &component);

    let expected = inline_identity(&component, "InlineHost");
    let forward = query_edges(
        &state,
        root.path(),
        "forward_edges",
        "angular",
        "Component",
        "InlineHost",
    );
    assert!(
        has_template_to(&forward, &expected),
        "inline Template must use the deterministic component-scoped identity: {forward}"
    );
    assert_ne!(expected, canonical(&component));
}

#[test]
fn republish_replacement_and_external_inline_transitions_retract_old_edges() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let component = root.path().join("transition-host.ts");
    let first = root.path().join("first.html");
    let second = root.path().join("second.html");
    std::fs::write(&first, "<p>first</p>").expect("first template");
    std::fs::write(&second, "<p>second</p>").expect("second template");

    write_component(&component, "TransitionHost", "templateUrl: './first.html'");
    publish(&state, root.path(), &component);
    publish(&state, root.path(), &component);

    let repeated = query_edges(
        &state,
        root.path(),
        "forward_edges",
        "angular",
        "Component",
        "TransitionHost",
    );
    assert_eq!(
        has_template_edges(&repeated).len(),
        1,
        "unchanged republication must not duplicate HasTemplate: {repeated}"
    );

    write_component(&component, "TransitionHost", "templateUrl: './second.html'");
    refresh(&state, root.path());
    publish(&state, root.path(), &component);
    let replaced = query_edges(
        &state,
        root.path(),
        "forward_edges",
        "angular",
        "Component",
        "TransitionHost",
    );
    assert!(
        has_template_to(&replaced, &canonical(&second)),
        "{replaced}"
    );
    assert!(
        !has_template_to(&replaced, &canonical(&first)),
        "{replaced}"
    );

    write_component(&component, "TransitionHost", r#"template: '<p>inline</p>'"#);
    refresh(&state, root.path());
    publish(&state, root.path(), &component);
    let inline = query_edges(
        &state,
        root.path(),
        "forward_edges",
        "angular",
        "Component",
        "TransitionHost",
    );
    assert!(
        has_template_to(&inline, &inline_identity(&component, "TransitionHost")),
        "{inline}"
    );
    assert!(!has_template_to(&inline, &canonical(&second)), "{inline}");

    write_component(&component, "TransitionHost", "templateUrl: './first.html'");
    refresh(&state, root.path());
    publish(&state, root.path(), &component);
    let external_again = query_edges(
        &state,
        root.path(),
        "forward_edges",
        "angular",
        "Component",
        "TransitionHost",
    );
    assert!(
        has_template_to(&external_again, &canonical(&first)),
        "{external_again}"
    );
    assert!(
        !has_template_to(
            &external_again,
            &inline_identity(&component, "TransitionHost")
        ),
        "{external_again}"
    );
}

#[test]
fn deletion_restoration_and_missing_external_target_follow_typescript_ownership() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let component = root.path().join("lifecycle-host.ts");
    let template = root.path().join("lifecycle-view.html");
    std::fs::write(&template, "<p>present</p>").expect("template fixture");
    write_component(
        &component,
        "LifecycleHost",
        "templateUrl: './lifecycle-view.html'",
    );
    publish(&state, root.path(), &component);

    std::fs::remove_file(&template).expect("delete external template");
    refresh(&state, root.path());
    publish(&state, root.path(), &component);
    let missing_target = query_edges(
        &state,
        root.path(),
        "forward_edges",
        "angular",
        "Component",
        "LifecycleHost",
    );
    assert!(
        has_template_to(&missing_target, &canonical(&template)),
        "written templateUrl remains neutral evidence while target is missing: {missing_target}"
    );

    std::fs::remove_file(&component).expect("delete component");
    refresh(&state, root.path());
    assert!(
        state
            .workspace_index_read()
            .forward_edges_by_identity("angular", "Component", "LifecycleHost")
            .is_empty(),
        "deleting the TypeScript assertion owner must retract HasTemplate"
    );

    write_component(
        &component,
        "LifecycleHost",
        "templateUrl: './lifecycle-view.html'",
    );
    std::fs::write(&template, "<p>restored</p>").expect("restore external template");
    refresh(&state, root.path());
    publish(&state, root.path(), &component);
    let restored = query_edges(
        &state,
        root.path(),
        "forward_edges",
        "angular",
        "Component",
        "LifecycleHost",
    );
    assert!(
        has_template_to(&restored, &canonical(&template)),
        "restoring the assertion owner must restore HasTemplate: {restored}"
    );
}

fn relative_fixture_path(target: &Path, base: &Path) -> Option<PathBuf> {
    let target_parts = target.components().collect::<Vec<_>>();
    let base_parts = base.components().collect::<Vec<_>>();
    let shared = target_parts
        .iter()
        .zip(&base_parts)
        .take_while(|(target_part, base_part)| target_part == base_part)
        .count();

    if shared == 0 {
        return None;
    }

    let mut relative = PathBuf::new();
    for _ in shared..base_parts.len() {
        relative.push("..");
    }
    for part in &target_parts[shared..] {
        relative.push(part.as_os_str());
    }
    Some(relative)
}
