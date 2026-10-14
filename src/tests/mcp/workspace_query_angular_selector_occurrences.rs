//! Phase 5 RED regressions for ephemeral Angular selector occurrences.
//! Occurrence-derived facts are query-time results, never durable index state.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};
use std::path::Path;

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

fn publish(state: &crate::mcp::McpState, root: &Path, file: &Path) {
    let response = call(
        state,
        "provide_code_context",
        json!({
            "workspaceRoot": root, "filePath": file, "fidelity": "edit"
        }),
    );
    assert!(
        response.get("error").is_none(),
        "publication must succeed: {response}"
    );
}

fn refresh(state: &crate::mcp::McpState, root: &Path) {
    let response = call(state, "refresh_workspace", json!({ "workspaceRoot": root }));
    assert_eq!(
        response["result"]["structuredContent"]["refreshed"], true,
        "refresh must succeed: {response}"
    );
}

fn query(state: &crate::mcp::McpState, root: &Path, entity_type: &str, name: &str) -> Value {
    let response = call(
        state,
        "workspace_query",
        json!({
            "type": "forward_edges", "workspaceRoot": root,
            "domain": "angular", "entity_type": entity_type, "name": name
        }),
    );
    assert!(
        response.get("error").is_none(),
        "forward query must succeed for angular/{entity_type}/{name}: {response}"
    );
    response["result"]["structuredContent"].clone()
}

fn reverse_query(
    state: &crate::mcp::McpState,
    root: &Path,
    entity_type: &str,
    name: &str,
) -> Value {
    let response = call(
        state,
        "workspace_query",
        json!({
            "type": "reverse_edges", "workspaceRoot": root,
            "domain": "angular", "entity_type": entity_type, "name": name
        }),
    );
    assert!(
        response.get("error").is_none(),
        "reverse query must succeed for angular/{entity_type}/{name}: {response}"
    );
    response["result"]["structuredContent"].clone()
}

fn canonical(path: &Path) -> String {
    crate::dictionary::path::canonical_identity_key(&path.to_string_lossy())
}

fn inline_identity(component: &Path, class_name: &str) -> String {
    format!("inline:{}#component={class_name}", canonical(component))
}

fn write_component(path: &Path, class_name: &str, selector: &str, field: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("component directory");
    }
    std::fs::write(
        path,
        format!(
            "import {{ Component }} from '@angular/core';\n\
         @Component({{ selector: '{selector}', {field} }})\n\
         export class {class_name} {{}}\n"
        ),
    )
    .expect("component fixture");
}

fn relation_edges<'a>(result: &'a Value, relation: &str) -> Vec<&'a Value> {
    result["edges"]
        .as_array()
        .expect("edges array")
        .iter()
        .filter(|edge| edge["relation"] == relation)
        .collect()
}

fn selector_edge<'a>(edges: &'a [&Value], selector: &str) -> &'a Value {
    edges
        .iter()
        .copied()
        .find(|edge| edge["object"]["name"] == selector)
        .unwrap_or_else(|| panic!("missing selector {selector}: {edges:?}"))
}

fn opening_spans(source: &str, prefix: &str) -> Vec<(usize, usize)> {
    source
        .match_indices(prefix)
        .map(|(start, _)| {
            let end = source[start..]
                .find('>')
                .map(|offset| start + offset + 1)
                .expect("opening tag end");
            (start, end)
        })
        .collect()
}

fn assert_occurrence(edge: &Value, span: (usize, usize)) {
    let evidence = &edge["evidence"]["template_occurrence"];
    assert!(
        evidence.is_object(),
        "occurrence evidence is required: {edge}"
    );
    assert_eq!(evidence["span"]["start_byte"], span.0);
    assert_eq!(evidence["span"]["end_byte"], span.1);
    assert!(
        edge["occurrence"].is_u64(),
        "occurrence handle is required: {edge}"
    );
}

fn ancestry_has(edge: &Value, syntax: &str, kind: &str, expression: &str) -> bool {
    edge["evidence"]["template_occurrence"]["control_ancestry"]
        .as_array()
        .is_some_and(|frames| {
            frames.iter().any(|frame| {
                frame["syntax"] == syntax
                    && frame["kind"] == kind
                    && frame["written_header"]
                        .as_str()
                        .is_some_and(|header| header.contains(expression))
            })
        })
}

#[test]
fn repeated_modern_uses_keep_distinct_neutral_and_typed_occurrences() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let parent = root.path().join("parent.ts");
    let child = root.path().join("child.ts");
    let template = root.path().join("parent.html");
    let html = "@if (showFirst) {\n<child-widget [value]=\"first\"></child-widget>\n}\n\
                @if (showSecond) {\n<child-widget [value]=\"second\"></child-widget>\n}\n";
    std::fs::write(&template, html).expect("template");
    write_component(
        &parent,
        "ParentComponent",
        "parent-host",
        "templateUrl: './parent.html'",
    );
    write_component(&child, "ChildComponent", "child-widget", "template: ''");
    publish(&state, root.path(), &child);
    publish(&state, root.path(), &parent);

    let neutral_result = query(&state, root.path(), "Template", &canonical(&template));
    let neutral = relation_edges(&neutral_result, "UsesSelector");
    let spans = opening_spans(html, "<child-widget");
    assert_eq!(
        neutral.len(),
        2,
        "both written uses remain neutral: {neutral_result}"
    );
    for (index, edge) in neutral.iter().enumerate() {
        assert_eq!(edge["object"]["entity_type"], "SelectorRef");
        assert_eq!(edge["object"]["name"], "child-widget");
        assert_occurrence(edge, spans[index]);
    }
    assert!(ancestry_has(neutral[0], "Modern", "If", "showFirst"));
    assert!(ancestry_has(neutral[1], "Modern", "If", "showSecond"));

    let typed_result = query(&state, root.path(), "Component", "ParentComponent");
    let typed = relation_edges(&typed_result, "UsesComponent");
    assert_eq!(
        typed.len(),
        2,
        "typed knowledge is per occurrence: {typed_result}"
    );
    for (index, edge) in typed.iter().enumerate() {
        assert_eq!(edge["object"]["name"], "ChildComponent");
        assert_occurrence(edge, spans[index]);
    }

    let reverse_neutral = reverse_query(&state, root.path(), "SelectorRef", "child-widget");
    assert_eq!(
        relation_edges(&reverse_neutral, "UsesSelector").len(),
        2,
        "reverse neutral query retains both occurrences"
    );
    let reverse_typed = reverse_query(&state, root.path(), "Component", "ChildComponent");
    assert_eq!(
        relation_edges(&reverse_typed, "UsesComponent").len(),
        2,
        "reverse typed query retains both occurrences"
    );

    let neutral_again = query(&state, root.path(), "Template", &canonical(&template));
    assert_eq!(
        relation_edges(&neutral_again, "UsesSelector").len(),
        2,
        "typed refinement never replaces neutral evidence"
    );
}

#[test]
fn inline_mixed_legacy_and_modern_controls_share_one_occurrence_model() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let parent = root.path().join("mixed.ts");
    let child = root.path().join("child.ts");
    let html = "<section *ngIf=\"ready\"><child-widget></child-widget>\
                @if (showModern) { <child-widget /> }</section>";
    let field = format!("template: '{}'", html);
    write_component(&parent, "MixedParent", "mixed-parent", &field);
    write_component(&child, "MixedChild", "child-widget", "template: ''");
    publish(&state, root.path(), &child);
    publish(&state, root.path(), &parent);

    let result = query(
        &state,
        root.path(),
        "Template",
        &inline_identity(&parent, "MixedParent"),
    );
    let neutral = relation_edges(&result, "UsesSelector");
    let spans = opening_spans(html, "<child-widget");
    assert_eq!(neutral.len(), 2, "mixed syntax keeps both uses: {result}");
    assert_occurrence(neutral[0], spans[0]);
    assert_occurrence(neutral[1], spans[1]);
    assert!(ancestry_has(neutral[0], "Legacy", "If", "ready"));
    assert!(ancestry_has(neutral[1], "Legacy", "If", "ready"));
    assert!(ancestry_has(neutral[1], "Modern", "If", "showModern"));
}

#[test]
fn unique_resolution_is_additive_while_missing_and_ambiguous_stay_neutral() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let parent = root.path().join("resolution-parent.ts");
    let template = root.path().join("resolution.html");
    let html = "<unique-child></unique-child><missing-child></missing-child>\
                <duplicate-child></duplicate-child>";
    std::fs::write(&template, html).expect("template");
    write_component(
        &parent,
        "ResolutionParent",
        "resolution-parent",
        "templateUrl: './resolution.html'",
    );
    let declarations = [
        ("unique.ts", "UniqueChild", "unique-child"),
        ("duplicate-a.ts", "DuplicateA", "duplicate-child"),
        ("duplicate-b.ts", "DuplicateB", "duplicate-child"),
    ];
    for (file, class_name, selector) in declarations {
        let path = root.path().join(file);
        write_component(&path, class_name, selector, "template: ''");
        publish(&state, root.path(), &path);
    }
    publish(&state, root.path(), &parent);

    let neutral_result = query(&state, root.path(), "Template", &canonical(&template));
    let neutral = relation_edges(&neutral_result, "UsesSelector");
    assert_eq!(neutral.len(), 3, "all written evidence remains visible");
    assert_eq!(
        selector_edge(&neutral, "unique-child")["evidence"]["resolution"]["status"],
        "unique"
    );
    assert_eq!(
        selector_edge(&neutral, "missing-child")["evidence"]["resolution"]["status"],
        "unresolved"
    );
    let ambiguous = selector_edge(&neutral, "duplicate-child");
    assert_eq!(ambiguous["evidence"]["resolution"]["status"], "ambiguous");
    assert_eq!(
        ambiguous["evidence"]["resolution"]["candidates"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    let typed_result = query(&state, root.path(), "Component", "ResolutionParent");
    let typed = relation_edges(&typed_result, "UsesComponent");
    assert_eq!(
        typed.len(),
        1,
        "only a unique match may type: {typed_result}"
    );
    assert_eq!(typed[0]["object"]["name"], "UniqueChild");
    assert!(
        neutral
            .iter()
            .any(|edge| edge["object"]["name"] == "unique-child"),
        "neutral evidence remains beside typed knowledge"
    );
}

#[test]
fn refresh_and_selector_changes_do_not_reuse_stale_results() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let parent = root.path().join("lifecycle-parent.ts");
    let child = root.path().join("lifecycle-child.ts");
    let template = root.path().join("lifecycle.html");
    std::fs::write(&template, "<old-widget></old-widget>").expect("template");
    write_component(
        &parent,
        "LifecycleParent",
        "lifecycle-parent",
        "templateUrl: './lifecycle.html'",
    );
    write_component(&child, "LifecycleChild", "old-widget", "template: ''");
    publish(&state, root.path(), &child);
    publish(&state, root.path(), &parent);
    let initial = query(&state, root.path(), "Component", "LifecycleParent");
    assert_eq!(relation_edges(&initial, "UsesComponent").len(), 1);

    std::fs::write(&template, "<new-widget></new-widget>").expect("replace");
    refresh(&state, root.path());
    let neutral = query(&state, root.path(), "Template", &canonical(&template));
    let selector_edges = relation_edges(&neutral, "UsesSelector");
    assert_eq!(selector_edges.len(), 1);
    assert_eq!(selector_edges[0]["object"]["name"], "new-widget");
    let untyped = query(&state, root.path(), "Component", "LifecycleParent");
    assert!(
        relation_edges(&untyped, "UsesComponent").is_empty(),
        "old typed result retracts after template replacement"
    );

    write_component(&child, "LifecycleChild", "new-widget", "template: ''");
    publish(&state, root.path(), &child);
    let retyped = query(&state, root.path(), "Component", "LifecycleParent");
    assert_eq!(relation_edges(&retyped, "UsesComponent").len(), 1);

    std::fs::remove_file(&template).expect("delete template");
    refresh(&state, root.path());
    let missing = query(&state, root.path(), "Component", "LifecycleParent");
    assert!(relation_edges(&missing, "UsesComponent").is_empty());
    std::fs::write(&template, "<new-widget></new-widget>").expect("restore");
    refresh(&state, root.path());
    let restored = query(&state, root.path(), "Component", "LifecycleParent");
    assert_eq!(relation_edges(&restored, "UsesComponent").len(), 1);
}

#[test]
fn exact_selector_binding_advances_only_the_angular_semantic_generation() {
    assert_eq!(
        crate::layers::meta::ANGULAR_SEMANTIC_PRODUCER_GENERATION,
        15
    );
    assert_eq!(crate::layers::meta::ANGULAR_MARKER_PRODUCER_GENERATION, 5);
}
