//! Phase 6 RED regressions for Angular-template diagnostics and completeness.

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
            "workspaceRoot": root,
            "filePath": file,
            "fidelity": "edit"
        }),
    );
    assert!(
        response.get("error").is_none(),
        "publication must succeed: {response}"
    );
}

fn query(state: &crate::mcp::McpState, root: &Path, entity_type: &str, name: &str) -> Value {
    let response = call(
        state,
        "workspace_query",
        json!({
            "type": "forward_edges",
            "workspaceRoot": root,
            "domain": "angular",
            "entity_type": entity_type,
            "name": name
        }),
    );
    assert!(
        response.get("error").is_none(),
        "query must succeed for angular/{entity_type}/{name}: {response}"
    );
    response["result"]["structuredContent"].clone()
}

fn canonical(path: &Path) -> String {
    crate::dictionary::path::canonical_identity_key(&path.to_string_lossy())
}

fn write_component(path: &Path, class_name: &str, selector: &str, metadata: &str) {
    std::fs::write(
        path,
        format!(
            "import {{ Component }} from '@angular/core';
\
             @Component({{ selector: '{selector}', {metadata} }})
\
             export class {class_name} {{}}
"
        ),
    )
    .expect("component fixture");
}

fn diagnostic_codes(result: &Value) -> Vec<&str> {
    result["template_diagnostics"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry["code"].as_str())
        .collect()
}

#[test]
fn angular_template_fidelities_disclose_intentional_omissions() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let template = root.path().join("host.component.html");
    let source = format!(
        "{}
@if (ready) {{ <child-row>{{{{ title }}}}</child-row> }}",
        "<!-- compression padding -->
"
        .repeat(300)
    );
    std::fs::write(&template, source).expect("template fixture");
    let state = state();

    let low = call(
        &state,
        "provide_code_context",
        json!({
            "workspaceRoot": root.path(),
            "filePath": template,
            "fidelity": "low"
        }),
    );
    assert!(
        low.get("error").is_none(),
        "Low template retrieval must succeed: {low}"
    );
    assert_eq!(
        low["result"]["_meta"]["degradation"]["ordering"], "aggregate_unordered",
        "Low must not imply containment or source ordering: {low}"
    );
    assert!(
        low["result"]["_meta"]["degradation"]["omits"]
            .as_array()
            .is_some_and(|omits| !omits.is_empty()),
        "Low must enumerate its intentional omissions: {low}"
    );

    let medium = call(
        &state,
        "provide_code_context",
        json!({
            "workspaceRoot": root.path(),
            "filePath": template,
            "fidelity": "medium"
        }),
    );
    assert!(
        medium.get("error").is_none(),
        "Medium template retrieval must succeed: {medium}"
    );
    assert_eq!(
        medium["result"]["_meta"]["degradation"]["ordering"], "source_ordered",
        "Medium preserves ordered structure: {medium}"
    );
    assert!(
        medium["result"]["_meta"]["degradation"]["omits"]
            .as_array()
            .is_some_and(|omits| omits.iter().any(|item| item == "comments")),
        "Medium must disclose its deliberate comment omission: {medium}"
    );
}

#[test]
fn missing_linked_template_retains_reference_and_reports_partiality() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let component = root.path().join("missing-host.ts");
    let missing = root.path().join("missing-view.html");
    write_component(
        &component,
        "MissingHost",
        "missing-host",
        "templateUrl: './missing-view.html'",
    );
    let state = state();
    publish(&state, root.path(), &component);

    let result = query(&state, root.path(), "Component", "MissingHost");
    assert!(
        result["edges"]
            .as_array()
            .is_some_and(|edges| edges.iter().any(|edge| {
                edge["relation"] == "HasTemplate" && edge["object"]["name"] == canonical(&missing)
            })),
        "written HasTemplate evidence must remain: {result}"
    );
    assert!(
        diagnostic_codes(&result).contains(&"linked_template_missing"),
        "missing target must be distinguishable from no relationship: {result}"
    );
    assert_eq!(result["coverage"]["source_complete"], false);
    assert_eq!(result["coverage"]["omitted_possible"], true);
    assert!(
        result["coverage"]["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons
                .iter()
                .any(|reason| reason["code"] == "linked_template_missing")),
        "producer-proven partiality must propagate into coverage reasons: {result}"
    );
}

#[test]
fn selector_resolution_diagnostics_are_explicit_and_neutral() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let parent = root.path().join("parent.ts");
    let template = root.path().join("parent.html");
    std::fs::write(
        &template,
        "<missing-child></missing-child><duplicate-child></duplicate-child>",
    )
    .expect("template fixture");
    write_component(
        &parent,
        "Parent",
        "parent-host",
        "templateUrl: './parent.html'",
    );
    for (file, class_name) in [("a.ts", "DuplicateA"), ("b.ts", "DuplicateB")] {
        let child = root.path().join(file);
        write_component(&child, class_name, "duplicate-child", "template: ''");
        publish(&state, root.path(), &child);
    }
    publish(&state, root.path(), &parent);

    let result = query(&state, root.path(), "Template", &canonical(&template));
    let codes = diagnostic_codes(&result);
    assert!(
        codes.contains(&"selector_unresolved"),
        "zero candidates must remain neutral and explicit: {result}"
    );
    assert!(
        codes.contains(&"selector_ambiguous"),
        "multiple candidates must remain neutral and explicit: {result}"
    );
    assert!(
        result["edges"].as_array().is_some_and(|edges| {
            edges
                .iter()
                .filter(|edge| edge["relation"] == "UsesSelector")
                .count()
                == 2
                && edges.iter().all(|edge| edge["relation"] != "UsesComponent")
        }),
        "diagnostics must not discard or fabricate semantic evidence: {result}"
    );
}

#[test]
fn reverse_component_use_does_not_overstate_parent_template_discovery() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let parent = root.path().join("reverse-parent.ts");
    let child = root.path().join("reverse-child.ts");
    let template = root.path().join("reverse-parent.html");
    std::fs::write(&template, "<reverse-child></reverse-child>").expect("template fixture");
    write_component(
        &parent,
        "ReverseParent",
        "reverse-parent",
        "templateUrl: './reverse-parent.html'",
    );
    write_component(&child, "ReverseChild", "reverse-child", "template: ''");
    publish(&state, root.path(), &child);
    publish(&state, root.path(), &parent);

    let response = call(
        &state,
        "workspace_query",
        json!({
            "type": "reverse_edges",
            "workspaceRoot": root.path(),
            "domain": "angular",
            "entity_type": "Component",
            "name": "ReverseChild"
        }),
    );
    assert!(response.get("error").is_none(), "{response}");
    let result = &response["result"]["structuredContent"];
    assert!(
        result["edges"]
            .as_array()
            .is_some_and(|edges| { edges.iter().any(|edge| edge["relation"] == "UsesComponent") }),
        "current indexed evidence may answer the relationship: {result}"
    );
    assert_eq!(
        result["coverage"]["source_complete"], false,
        "reverse Component hydration cannot prove every parent template was discovered: {result}"
    );
    assert_eq!(result["coverage"]["omitted_possible"], true);
}

#[test]
fn complete_template_discovery_establishes_exact_capability_and_completeness() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state();
    let parent = root.path().join("complete-parent.ts");
    let child = root.path().join("complete-child.ts");
    let template = root.path().join("complete.html");
    std::fs::write(&template, "<complete-child></complete-child>").expect("template fixture");
    write_component(
        &parent,
        "CompleteParent",
        "complete-parent",
        "templateUrl: './complete.html'",
    );
    write_component(&child, "CompleteChild", "complete-child", "template: ''");
    publish(&state, root.path(), &child);
    publish(&state, root.path(), &parent);

    let result = query(&state, root.path(), "Template", &canonical(&template));
    assert_eq!(
        result["coverage"]["capability_established"], true,
        "{result}"
    );
    assert_eq!(
        result["coverage"]["source_complete"], true,
        "completed authorized Component and Template discovery may establish completeness: {result}"
    );
    assert_eq!(result["coverage"]["status"], "authoritative_complete");
    assert_eq!(result["coverage"]["result_semantics"], "exact");
    assert_eq!(result["coverage"]["reasons"], json!([]));
    assert_eq!(result["coverage"]["omitted_possible"], false, "{result}");
    assert!(
        result["coverage"].get("alternative_query").is_none(),
        "successful exact capability needs no fallback guidance: {result}"
    );
}
