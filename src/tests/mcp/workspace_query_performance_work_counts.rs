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

#[test]
fn equivalent_ef_batch_items_reuse_one_request_local_catalog() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = root.path().join("EfModel.cs");
    std::fs::write(
        &file,
        r#"using Microsoft.EntityFrameworkCore;
public class Entity {}
public class Config : IEntityTypeConfiguration<Entity> {}
public class AppDbContext : DbContext {
 protected override void OnModelCreating(ModelBuilder builder) {
  builder.ApplyConfiguration(new Config());
 }
}"#,
    )
    .expect("EF fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(&state, root.path(), &file);

    work_metrics::reset();
    let response = call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                { "id": "first", "type": "forward_edges", "domain": "dotnet", "entity_type": "DbContext", "name": "AppDbContext" },
                { "id": "second", "type": "forward_edges", "domain": "dotnet", "entity_type": "DbContext", "name": "AppDbContext" }
            ]
        }),
    );

    let results = batch_results(&response);
    assert_eq!(results.len(), 2, "{response}");
    assert!(
        results.iter().all(|item| item["status"] == "ok"),
        "{response}"
    );
    assert_eq!(results[0]["result"], results[1]["result"]);
    let counts = work_metrics::snapshot();
    eprintln!("post-reuse EF work counts: {counts:#?}");
    assert_eq!(counts.csharp_catalog_builds, 1);
    assert_eq!(
        counts.ef_catalog_builds, 1,
        "equivalent items across both preparation passes must share EF derivation"
    );
    assert_eq!(counts.ef_catalog_reuses, 3);
    assert!(counts.ef_configuration_declarations_examined > 0);
    assert!(counts.ef_configuration_reference_comparisons > 0);
    assert!(counts.ef_context_declarations_examined > 0);
    assert!(counts.ef_context_reference_comparisons > 0);
    assert!(counts.ef_owner_reference_comparisons > 0);
    assert!(counts.ef_synthetic_type_parses > 0);
    assert_eq!(counts.ef_model_occurrences_processed, 1);
    assert_eq!(counts.ef_assembly_configuration_comparisons, 0);
    assert_eq!(counts.ef_project_root_lookups, 0);
}

#[test]
fn ef_catalog_build_reuse_invalidate_rebuild_reuse_sequence() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let model = root.path().join("EfModel.cs");
    let added = root.path().join("Added.cs");
    std::fs::write(
        root.path().join("App.csproj"),
        "<Project Sdk=\"Microsoft.NET.Sdk\" />",
    )
    .expect("project fixture");
    std::fs::write(
        &model,
        "using Microsoft.EntityFrameworkCore; public class Entity {} public class Config : IEntityTypeConfiguration<Entity> {} public class AppDbContext : DbContext { protected override void OnModelCreating(ModelBuilder builder) { builder.ApplyConfigurationsFromAssembly(typeof(AppDbContext).Assembly); } }",
    )
    .expect("EF fixture");
    std::fs::write(
        &added,
        "using Microsoft.EntityFrameworkCore; public class AddedEntity {} public class AddedConfig : IEntityTypeConfiguration<AddedEntity> {}",
    )
    .expect("added fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(&state, root.path(), &model);

    work_metrics::reset();
    let response = call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot": root.path(),
            "queries": [
                { "id": "before-first", "type": "forward_edges", "domain": "dotnet", "entity_type": "DbContext", "name": "AppDbContext" },
                { "id": "before-second", "type": "forward_edges", "domain": "dotnet", "entity_type": "DbContext", "name": "AppDbContext" },
                { "id": "publish", "type": "entities_in_file", "file_path": "Added.cs", "fidelity": "high" },
                { "id": "after-first", "type": "forward_edges", "domain": "dotnet", "entity_type": "DbContext", "name": "AppDbContext" },
                { "id": "after-second", "type": "forward_edges", "domain": "dotnet", "entity_type": "DbContext", "name": "AppDbContext" }
            ]
        }),
    );

    let results = batch_results(&response);
    assert!(
        results.iter().all(|item| item["status"] == "ok"),
        "{response}"
    );
    for result in [&results[0], &results[1], &results[3], &results[4]] {
        let rendered = result["result"].to_string();
        assert!(rendered.contains("Entity"), "{response}");
        assert!(
            rendered.contains("AddedEntity"),
            "publication must be visible after invalidation and second-pass rebuild: {response}"
        );
    }
    let counts = work_metrics::snapshot();
    assert_eq!(counts.ef_catalog_builds, 2);
    assert_eq!(counts.ef_catalog_reuses, 6);
    assert_eq!(counts.derived_catalog_invalidations, 1);
}

fn measure_ef_assembly_work(configuration_count: usize) -> work_metrics::QueryWorkCounts {
    let root = tempfile::tempdir().expect("workspace");
    std::fs::write(
        root.path().join("App.csproj"),
        "<Project Sdk=\"Microsoft.NET.Sdk\" />",
    )
    .expect("project fixture");
    let mut configurations = String::from("using Microsoft.EntityFrameworkCore; namespace App;\n");
    for index in 0..configuration_count {
        configurations.push_str(&format!(
            "public class Entity{index} {{}} public class Config{index} : IEntityTypeConfiguration<Entity{index}> {{}}\n"
        ));
    }
    let configuration_file = root.path().join("Configurations.cs");
    let context_file = root.path().join("AppDbContext.cs");
    std::fs::write(&configuration_file, configurations).expect("configuration fixture");
    std::fs::write(
        &context_file,
        r#"using Microsoft.EntityFrameworkCore;
namespace App;
public class AppDbContext : DbContext {
 protected override void OnModelCreating(ModelBuilder builder) {
  builder.ApplyConfigurationsFromAssembly(typeof(AppDbContext).Assembly);
 }
}"#,
    )
    .expect("context fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(&state, root.path(), &configuration_file);
    publish(&state, root.path(), &context_file);

    work_metrics::reset();
    let response = call(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot": root.path(),
            "type": "forward_edges",
            "domain": "dotnet",
            "entity_type": "DbContext",
            "name": "AppDbContext"
        }),
    );
    assert!(response.get("error").is_none(), "{response}");
    let edges = response["result"]["structuredContent"]["edges"]
        .as_array()
        .expect("EF edges");
    assert_eq!(
        edges
            .iter()
            .filter(|edge| edge["relation"] == "HasEntity")
            .count(),
        configuration_count,
        "{response}"
    );
    work_metrics::snapshot()
}

#[test]
fn ef_reference_scans_scale_multiplicatively_after_catalog_reuse() {
    let _serial = crate::protocol::handler_response_serial();
    let small = measure_ef_assembly_work(4);
    let large = measure_ef_assembly_work(16);
    eprintln!("scaled EF work counts: small={small:#?} large={large:#?}");

    assert_eq!(small.csharp_catalog_builds, 1);
    assert_eq!(small.ef_catalog_builds, 1);
    assert_eq!(large.csharp_catalog_builds, 1);
    assert_eq!(large.ef_catalog_builds, 1);
    assert_eq!(
        small.ef_configuration_reference_comparisons, 5,
        "each class declaration must inspect only its ordered owner bucket"
    );
    assert_eq!(large.ef_configuration_reference_comparisons, 17);
    assert_eq!(small.ef_context_reference_comparisons, 5);
    assert_eq!(large.ef_context_reference_comparisons, 17);
    assert_eq!(small.ef_owner_reference_comparisons, 1);
    assert_eq!(large.ef_owner_reference_comparisons, 1);
    assert_eq!(small.reference_candidate_index_builds, 1);
    assert_eq!(large.reference_candidate_index_builds, 1);
    assert_eq!(small.reference_candidate_index_keys, 5);
    assert_eq!(large.reference_candidate_index_keys, 17);
    assert_eq!(small.reference_candidate_indexed_references, 5);
    assert_eq!(large.reference_candidate_indexed_references, 17);
    assert_eq!(small.reference_candidate_max_bucket_size, 1);
    assert_eq!(large.reference_candidate_max_bucket_size, 1);
    assert_eq!(
        large.ef_assembly_configuration_comparisons,
        small.ef_assembly_configuration_comparisons * 4
    );
    assert_eq!(
        large.ef_synthetic_type_parses,
        small.ef_synthetic_type_parses + 12
    );
    assert_eq!(
        (small.ef_project_root_lookups, large.ef_project_root_lookups),
        (5, 17),
        "logical assembly project-membership work remains unchanged"
    );
    assert_eq!(
        (
            small.ef_project_root_filesystem_resolutions,
            large.ef_project_root_filesystem_resolutions,
        ),
        (2, 2),
        "each distinct participating file must resolve its project only once per EF build"
    );
    assert_eq!(
        (
            small.ef_project_root_cache_hits,
            large.ef_project_root_cache_hits,
        ),
        (3, 15),
        "repeated configuration-file lookups must reuse the build-local result"
    );
}
