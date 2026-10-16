//! Structural characterization of current semantic-query reconstruction work.
//!
//! Counts intentionally describe the uncached Phase 2 baseline. They are not
//! future cache targets and do not participate in semantic authority.

use super::{angular_templates, work_metrics};
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};
use std::path::Path;

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
            "fidelity": "high"
        }),
    );
    assert!(
        response.get("error").is_none(),
        "fixture publication must succeed: {response}"
    );
}

fn batch_results(response: &Value) -> &Vec<Value> {
    response["result"]["structuredContent"]["results"]
        .as_array()
        .expect("batch results")
}

#[test]
fn equivalent_csharp_batch_items_reuse_one_request_local_catalog() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = root.path().join("Derived.cs");
    std::fs::write(&file, "class Base {} class Derived : Base {}").expect("C# fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(&state, root.path(), &file);

    work_metrics::reset();
    let response = call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                { "id": "first", "type": "forward_edges", "domain": "builtin", "entity_type": "Class", "name": "Derived" },
                { "id": "second", "type": "forward_edges", "domain": "builtin", "entity_type": "Class", "name": "Derived" }
            ]
        }),
    );

    let results = batch_results(&response);
    assert_eq!(results.len(), 2, "{response}");
    assert_eq!(results[0]["status"], "ok", "{response}");
    assert_eq!(results[1]["status"], "ok", "{response}");
    assert_eq!(results[0]["result"], results[1]["result"]);

    let counts = work_metrics::snapshot();
    assert_eq!(
        counts.csharp_catalog_builds, 1,
        "equivalent consumers over unchanged request evidence must share one catalog"
    );
    assert_eq!(counts.csharp_files_considered, 1);
    assert_eq!(counts.csharp_files_parsed, 1);
}

#[test]
fn repeated_angular_catalog_builds_reprocess_inline_and_external_templates() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let external_component = root.path().join("ExternalHost.ts");
    let inline_component = root.path().join("InlineHost.ts");
    let template = root.path().join("external.html");
    std::fs::write(&template, "<child-view></child-view>").expect("external template");
    std::fs::write(
        &external_component,
        "import { Component } from '@angular/core'; @Component({ selector: 'external-host', templateUrl: './external.html' }) export class ExternalHost {}",
    )
    .expect("external component");
    std::fs::write(
        &inline_component,
        "import { Component } from '@angular/core'; @Component({ selector: 'inline-host', template: '<child-view></child-view>' }) export class InlineHost {}",
    )
    .expect("inline component");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(&state, root.path(), &external_component);
    publish(&state, root.path(), &inline_component);

    work_metrics::reset();
    let index = state.workspace_index_read();
    let first = angular_templates::Catalog::build(&state, &index, None, true);
    let second = angular_templates::Catalog::build(&state, &index, None, true);
    assert_eq!(first.selectors(), second.selectors());
    assert_eq!(
        first.forward("angular", "Component", "ExternalHost"),
        second.forward("angular", "Component", "ExternalHost")
    );
    drop(index);

    let counts = work_metrics::snapshot();
    assert_eq!(counts.angular_catalog_builds, 2);
    assert_eq!(
        counts.angular_components_considered, 8,
        "each build currently considers two declaration identities and their two selector endpoint identities"
    );
    assert_eq!(counts.angular_template_sources_processed, 4);
    assert_eq!(counts.angular_inline_templates_processed, 2);
    assert_eq!(counts.angular_external_templates_processed, 2);
}

#[test]
fn equivalent_angular_batch_items_reuse_compatible_request_local_catalogs() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let external_component = root.path().join("ExternalHost.ts");
    let inline_component = root.path().join("InlineHost.ts");
    std::fs::write(
        root.path().join("external.html"),
        "<child-view></child-view>",
    )
    .expect("external template");
    std::fs::write(
        &external_component,
        "import { Component } from '@angular/core'; @Component({ selector: 'external-host', templateUrl: './external.html' }) export class ExternalHost {}",
    )
    .expect("external component");
    std::fs::write(
        &inline_component,
        "import { Component } from '@angular/core'; @Component({ selector: 'inline-host', template: '<child-view></child-view>' }) export class InlineHost {}",
    )
    .expect("inline component");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(&state, root.path(), &external_component);
    publish(&state, root.path(), &inline_component);

    work_metrics::reset();
    let response = call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                { "id": "first", "type": "forward_edges", "domain": "angular", "entity_type": "Component", "name": "ExternalHost" },
                { "id": "second", "type": "forward_edges", "domain": "angular", "entity_type": "Component", "name": "ExternalHost" }
            ]
        }),
    );

    let results = batch_results(&response);
    assert_eq!(results.len(), 2, "{response}");
    assert_eq!(results[0]["status"], "ok", "{response}");
    assert_eq!(results[1]["status"], "ok", "{response}");
    assert_eq!(results[0]["result"], results[1]["result"]);
    let counts = work_metrics::snapshot();
    assert_eq!(counts.angular_catalog_builds, 2);
    assert_eq!(counts.angular_components_considered, 8);
    assert_eq!(counts.angular_template_sources_processed, 4);
    assert_eq!(counts.angular_inline_templates_processed, 2);
    assert_eq!(counts.angular_external_templates_processed, 2);
}

#[test]
fn csharp_catalog_build_reuse_invalidate_rebuild_reuse_sequence() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let derived = root.path().join("Derived.cs");
    let added = root.path().join("Added.cs");
    std::fs::write(&derived, "class Base {} class Derived : Base {}").expect("derived fixture");
    std::fs::write(&added, "class Added {}").expect("added fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(&state, root.path(), &derived);

    work_metrics::reset();
    let response = call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                { "id": "before-first", "type": "forward_edges", "domain": "builtin", "entity_type": "Class", "name": "Derived" },
                { "id": "before-second", "type": "forward_edges", "domain": "builtin", "entity_type": "Class", "name": "Derived" },
                { "id": "publish", "type": "entities_in_file", "file_path": "Added.cs", "fidelity": "high" },
                { "id": "after-first", "type": "forward_edges", "domain": "builtin", "entity_type": "Class", "name": "Derived" },
                { "id": "after-second", "type": "forward_edges", "domain": "builtin", "entity_type": "Class", "name": "Derived" }
            ]
        }),
    );

    let results = batch_results(&response);
    assert!(
        results.iter().all(|item| item["status"] == "ok"),
        "{response}"
    );
    assert_eq!(results[0]["result"], results[1]["result"]);
    assert_eq!(results[0]["result"], results[3]["result"]);
    assert_eq!(results[0]["result"], results[4]["result"]);
    let counts = work_metrics::snapshot();
    assert_eq!(
        counts.csharp_catalog_builds, 2,
        "publication must invalidate the catalog built from earlier evidence"
    );
    assert_eq!(
        counts.csharp_catalog_reuses, 6,
        "one reuse must occur before publication, one after rebuilding, and four in the second preparation pass"
    );
    assert_eq!(
        counts.derived_catalog_invalidations, 1,
        "the direct publication must invalidate the one non-empty request-local derived state"
    );
    assert_eq!(counts.csharp_files_considered, 3);
    assert_eq!(counts.csharp_files_parsed, 3);
}
