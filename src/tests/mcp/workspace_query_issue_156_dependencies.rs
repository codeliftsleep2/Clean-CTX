//! Supplemental EF obligations; the original observed RED is immutable.
#[path = "workspace_query_issue_156_coverage.rs"]
mod coverage;

use crate::mcp::McpState;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

fn state(roots: &[&Path]) -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = roots
        .iter()
        .map(|root| root.to_string_lossy().into_owned())
        .collect();
    McpState::new(config)
}

fn write(root: &Path, file: &str, source: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

fn query(state: &McpState, root: &Path, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(1561),
        "workspace_query",
        &json!({"arguments":args}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    response["result"]["structuredContent"].clone()
}

fn target_args() -> Value {
    json!({"type":"forward_edges", "domain":"dotnet", "entity_type":"DbContext", "name":"Target"})
}

fn target(state: &McpState, root: &Path) -> Value {
    query(state, root, target_args())
}

fn names(result: &Value) -> BTreeSet<String> {
    result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "HasEntity")
        .map(|edge| edge["object"]["name"].as_str().unwrap().to_string())
        .collect()
}

fn expect(result: &Value, expected: &[&str]) {
    assert_eq!(
        names(result),
        expected.iter().map(|name| name.to_string()).collect(),
        "{result:#}"
    );
    assert_eq!(result["coverage"]["source_complete"], false, "{result:#}");
    assert_eq!(
        result["coverage"]["result_semantics"], "lower_bound",
        "{result:#}"
    );
}

fn fixture(root: &Path, model: &str) {
    write(root, "App.csproj", "<Project />");
    write(
        root,
        "Rows.cs",
        "namespace App; public class First {} public class Second {}",
    );
    write(
        root,
        "Mapping.cs",
        "using Microsoft.EntityFrameworkCore; namespace App; public class Mapping : IEntityTypeConfiguration<First> {}",
    );
    write(
        root,
        "Target.cs",
        &format!(
            "using Microsoft.EntityFrameworkCore; namespace App; public class Target : DbContext {{ protected override void OnModelCreating(ModelBuilder builder) {{ {model} }} }}"
        ),
    );
}

#[test]
#[serial_test::serial]
fn issue_156_alias_and_namespace_targets_exclude_lookalikes() {
    let root = tempfile::tempdir().unwrap();
    fixture(
        root.path(),
        "builder.Entity<Row>(); builder.ApplyConfiguration(new Map());",
    );
    write(
        root.path(),
        "Target.cs",
        "using Microsoft.EntityFrameworkCore; using Row = App.Second; using Map = App.Mapping; namespace App; public class Target : DbContext { protected override void OnModelCreating(ModelBuilder builder) { builder.Entity<Row>(); builder.ApplyConfiguration(new Map()); } }",
    );
    write(
        root.path(),
        "Lookalike.cs",
        "namespace Other; public class First {} public class Second {} public class Mapping {}",
    );
    let state = state(&[]);
    let result = target(&state, root.path());
    expect(&result, &["First", "Second"]);
    for edge in result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "HasEntity")
    {
        assert!(
            edge["object"]["file"]
                .as_str()
                .unwrap()
                .ends_with("Rows.cs"),
            "{result:#}"
        );
    }
}

#[test]
#[serial_test::serial]
fn issue_156_missing_entity_stays_unproven_and_matches_schema() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), "builder.Entity<Missing>();");
    let result = target(&state(&[]), root.path());
    expect(&result, &[]);
    assert_eq!(result["coverage"]["dependency_proof"]["status"], "unproven");
    let tools = crate::mcp::tools::tool_list();
    let schema = &tools
        .iter()
        .find(|tool| tool["name"] == "workspace_query")
        .unwrap()["outputSchema"]["properties"]["coverage"]["properties"];
    assert!(
        schema["status"]["enum"]
            .as_array()
            .unwrap()
            .contains(&result["coverage"]["status"])
    );
    let strings = schema["reasons"]["items"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|schema| schema["type"] == "string")
        .unwrap();
    for reason in result["coverage"]["reasons"].as_array().unwrap() {
        assert!(
            strings["enum"].as_array().unwrap().contains(reason),
            "{reason:#}"
        );
    }
}

#[test]
#[serial_test::serial]
fn issue_156_duplicate_and_import_ambiguous_entities_fail_closed() {
    for duplicate in [false, true] {
        let root = tempfile::tempdir().unwrap();
        fixture(root.path(), "builder.Entity<First>();");
        if duplicate {
            write(
                root.path(),
                "Duplicate.cs",
                "namespace App; public class First {}",
            );
        } else {
            write(
                root.path(),
                "Rows.cs",
                "namespace Left { public class First {} } namespace Right { public class First {} }",
            );
            write(
                root.path(),
                "Target.cs",
                "using Microsoft.EntityFrameworkCore; using Left; using Right; public class Target : DbContext { protected override void OnModelCreating(ModelBuilder builder) { builder.Entity<First>(); } }",
            );
        }
        let result = target(&state(&[]), root.path());
        expect(&result, &[]);
        assert_eq!(
            result["coverage"]["dependency_proof"]["status"], "unproven",
            "{result:#}"
        );
    }
}

#[test]
#[serial_test::serial]
fn issue_156_partial_entity_and_configuration_use_composed_identity() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), "builder.ApplyConfiguration(new Mapping());");
    write(
        root.path(),
        "Rows.cs",
        "namespace App; public partial class First {}",
    );
    write(
        root.path(),
        "MoreRows.cs",
        "namespace App; public partial class First { public int Id { get; set; } }",
    );
    write(
        root.path(),
        "Mapping.cs",
        "using Microsoft.EntityFrameworkCore; namespace App; public partial class Mapping : IEntityTypeConfiguration<First> {}",
    );
    write(
        root.path(),
        "AMapping.cs",
        "namespace App; public partial class Mapping {}",
    );
    write(
        root.path(),
        "Target.cs",
        "using Microsoft.EntityFrameworkCore; namespace App; public partial class Target : DbContext {}",
    );
    write(
        root.path(),
        "TargetModel.cs",
        "using Microsoft.EntityFrameworkCore; namespace App; public partial class Target { protected override void OnModelCreating(ModelBuilder builder) { builder.ApplyConfiguration(new Mapping()); } }",
    );
    expect(&target(&state(&[]), root.path()), &["First"]);
}

#[test]
#[serial_test::serial]
fn issue_156_direct_obligation_does_not_compile_unrelated_workspace_sources() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), "builder.Entity<App.First>();");
    for index in 0..24 {
        write(
            root.path(),
            &format!("Noise{index}.cs"),
            &format!("namespace App; public class Noise{index} {{}}"),
        );
    }
    let state = state(&[]);
    expect(&target(&state, root.path()), &["First"]);
    for index in 0..24 {
        let owner = state.semantic_owner_path(
            &root
                .path()
                .join(format!("Noise{index}.cs"))
                .to_string_lossy(),
        );
        assert!(
            state.csharp_call_authority.artifact(&owner).is_none(),
            "unrelated source compiled: {owner}"
        );
    }
}

#[test]
#[serial_test::serial]
fn issue_156_single_flight_followers_establish_their_own_ef_evidence() {
    use std::sync::Arc;
    struct Release(Arc<McpState>);
    impl Drop for Release {
        fn drop(&mut self) {
            self.0.hydration_flights.test_release();
        }
    }
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), "builder.ApplyConfiguration(new Mapping());");
    let state = Arc::new(state(&[]));
    query(
        &state,
        root.path(),
        json!({"type":"entities_in_file", "file_path":"Target.cs"}),
    );
    let owner = state.semantic_owner_path(&root.path().join("Target.cs").to_string_lossy());
    let workspace = root.path().to_string_lossy().into_owned();
    let _release = Release(Arc::clone(&state));
    state.hydration_flights.test_arm();
    let spawn = || {
        let state = Arc::clone(&state);
        let owner = owner.clone();
        let workspace = workspace.clone();
        std::thread::spawn(move || {
            crate::mcp::tool_handlers::query::csharp_resolution::prepare_ef_dependencies(
                &state,
                &[owner],
                Some(&workspace),
                None,
                false,
                None,
                "issue-156-follower",
            )
            .unwrap()
        })
    };
    let first = spawn();
    state.hydration_flights.test_wait_for_leader();
    let second = spawn();
    state.hydration_flights.test_wait_for_followers(1);
    state.hydration_flights.test_release();
    for proof in [first.join().unwrap(), second.join().unwrap()] {
        let index = state.workspace_index_read();
        assert!(proof.current(&index));
        let mut coverage = json!({"dependency_proof":{"status":"complete"}});
        proof.apply(&index, &mut coverage);
        assert_eq!(
            coverage["dependency_proof"]["status"], "complete",
            "{coverage:#}"
        );
    }
    expect(&target(&state, root.path()), &["First"]);
}

#[test]
#[serial_test::serial]
fn issue_156_root_order_cannot_admit_a_foreign_entity_identity() {
    let root = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let empty = tempfile::tempdir().unwrap();
    fixture(root.path(), "builder.Entity<App.First>();");
    write(other.path(), "Other.csproj", "<Project />");
    write(
        other.path(),
        "Rows.cs",
        "namespace Other; public class First {}",
    );
    for roots in [[other.path(), empty.path()], [empty.path(), other.path()]] {
        let state = state(&roots);
        query(
            &state,
            root.path(),
            json!({"type":"entities_in_file", "file_path":other.path().join("Rows.cs")}),
        );
        let result = target(&state, root.path());
        expect(&result, &["First"]);
        let file = result["edges"][0]["object"]["file"].as_str().unwrap();
        assert!(
            std::fs::canonicalize(file)
                .unwrap()
                .starts_with(std::fs::canonicalize(root.path()).unwrap()),
            "{result:#}"
        );
    }
}

#[test]
#[serial_test::serial]
fn issue_156_cold_reverse_acquires_the_model_authority() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), "builder.ApplyConfiguration(new Mapping());");
    let result = query(
        &state(&[]),
        root.path(),
        json!({"type":"reverse_edges", "domain":"dotnet", "entity_type":"Entity", "name":"First"}),
    );
    assert!(
        result["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| { edge["relation"] == "HasEntity" && edge["subject"]["name"] == "Target" }),
        "{result:#}"
    );
    assert_eq!(result["coverage"]["source_complete"], false);
}

#[test]
#[serial_test::serial]
fn issue_156_warm_queries_reuse_compilation_and_reconciliation() {
    let root = tempfile::tempdir().unwrap();
    fixture(
        root.path(),
        "builder.ApplyConfigurationsFromAssembly(typeof(Target).Assembly);",
    );
    let state = state(&[]);
    expect(&target(&state, root.path()), &["First"]);
    state.reset_test_full_semantic_parses();
    state.reset_test_csharp_reconciliation_count();
    for _ in 0..2 {
        expect(&target(&state, root.path()), &["First"]);
    }
    assert_eq!(state.test_full_semantic_parses(), 0);
    assert_eq!(state.test_csharp_reconciliation_count(), 0);
}

#[path = "workspace_query_issue_156_lifecycle.rs"]
mod lifecycle;

#[test]
#[serial_test::serial]
fn issue_156_duplicate_configurations_cannot_establish_ownership() {
    for model in [
        "builder.ApplyConfiguration(new Mapping());",
        "builder.ApplyConfigurationsFromAssembly(typeof(Target).Assembly);",
    ] {
        let root = tempfile::tempdir().unwrap();
        fixture(root.path(), model);
        write(
            root.path(),
            "DuplicateMapping.cs",
            "using Microsoft.EntityFrameworkCore; namespace App; public class Mapping : IEntityTypeConfiguration<First> {}",
        );
        let result = target(&state(&[]), root.path());
        expect(&result, &[]);
        assert_eq!(
            result["coverage"]["dependency_proof"]["status"], "unproven",
            "{result:#}"
        );
    }
}
