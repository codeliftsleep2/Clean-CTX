//! Issue #156 candidates: existing EF HasEntity truth across fresh admission orders.
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

fn state() -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    crate::mcp::McpState::new(config)
}

fn query(state: &crate::mcp::McpState, root: &Path, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(156),
        "workspace_query",
        &json!({"arguments": args}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    response["result"]["structuredContent"].clone()
}

fn fixture(root: &Path, model: &str) {
    for (file, source) in [
        ("App.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\" />"),
        ("Record.cs", "namespace App; public class Record {}"),
        (
            "Mapping.cs",
            "using Microsoft.EntityFrameworkCore; namespace App; public class Mapping : IEntityTypeConfiguration<Record> {}",
        ),
        (
            "Bridge.cs",
            "namespace App; public class Bridge : Microsoft.EntityFrameworkCore.DbContext {}",
        ),
        (
            "Lookalike.cs",
            "namespace Other; public class Record {} public class Mapping {}",
        ),
        ("Unrelated.cs", "public class Unrelated {}"),
    ] {
        std::fs::write(root.join(file), source).unwrap();
    }
    std::fs::write(
        root.join("Target.cs"),
        format!(
            "using Microsoft.EntityFrameworkCore; namespace App; public class Target : Bridge {{ protected override void OnModelCreating(ModelBuilder builder) {{ {model} }} }}"
        ),
    )
    .unwrap();
}

fn target(state: &crate::mcp::McpState, root: &Path) -> Value {
    query(
        state,
        root,
        json!({"type":"forward_edges", "domain":"dotnet", "entity_type":"DbContext", "name":"Target"}),
    )
}

fn facts(answer: &Value) -> BTreeSet<String> {
    answer["edges"]
        .as_array()
        .expect("edge response")
        .iter()
        .filter(|edge| edge["relation"] == "HasEntity")
        .map(|edge| serde_json::to_string(edge).unwrap())
        .collect()
}

fn assert_supported_fact(answer: &Value, root: &Path) {
    let edges = answer["edges"].as_array().expect("edge response");
    let owned: Vec<_> = edges
        .iter()
        .filter(|edge| edge["relation"] == "HasEntity")
        .collect();
    assert_eq!(
        owned.len(),
        1,
        "warm control must prove one supported relationship: {answer:#}"
    );
    let edge = owned[0];
    assert_eq!(edge["subject"]["name"], "Target", "{answer:#}");
    assert_eq!(edge["object"]["name"], "Record", "{answer:#}");
    let expected = std::fs::canonicalize(root.join("Record.cs")).unwrap();
    let actual = Path::new(
        edge["object"]["file"]
            .as_str()
            .expect("resolved entity file"),
    );
    assert_eq!(std::fs::canonicalize(actual).unwrap(), expected);
}

fn convergence(model: &str) {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), model);
    // Legitimate public queries establish the positive control before cold assertions.
    // No direct index mutation or compilation/publication helper is used.
    let control = state();
    for file in ["Lookalike.cs", "Record.cs", "Mapping.cs"] {
        query(
            &control,
            root.path(),
            json!({"type":"entities_in_file", "file_path":file, "fidelity":"high"}),
        );
    }
    let expected = target(&control, root.path());
    assert_supported_fact(&expected, root.path());
    let expected_facts = facts(&expected);
    let mut failures = Vec::new();
    for order in [
        "cold",
        "dependency_names",
        "dependency_files",
        "unrelated",
        "repeat",
    ] {
        let fresh = state();
        match order {
            "dependency_names" => {
                for name in ["Record", "Mapping"] {
                    query(
                        &fresh,
                        root.path(),
                        json!({"type":"find_entities", "name":name}),
                    );
                }
            }
            "dependency_files" => {
                for file in ["Lookalike.cs", "Record.cs", "Mapping.cs"] {
                    query(
                        &fresh,
                        root.path(),
                        json!({"type":"entities_in_file", "file_path":file, "fidelity":"high"}),
                    );
                }
            }
            "unrelated" => {
                query(
                    &fresh,
                    root.path(),
                    json!({"type":"find_entities", "name":"Unrelated"}),
                );
            }
            "repeat" => {
                let first = target(&fresh, root.path());
                if facts(&first) != expected_facts {
                    failures.push(format!("repeat/first: {first:#}"));
                }
            }
            _ => {}
        }
        let actual = target(&fresh, root.path());
        if facts(&actual) != expected_facts {
            failures.push(format!("{order}: {actual:#}"));
        }
    }
    assert!(
        failures.is_empty(),
        "EF admission-order divergence:\n{}",
        failures.join("\n")
    );
}

#[test]
#[serial_test::serial]
fn issue_156_direct_model_entity_admission_converges() {
    convergence("builder.Entity<App.Record>();");
}

#[test]
#[serial_test::serial]
fn issue_156_explicit_configuration_admission_converges() {
    convergence("builder.ApplyConfiguration(new App.Mapping());");
}

#[test]
#[serial_test::serial]
fn issue_156_local_project_assembly_admission_converges() {
    convergence("builder.ApplyConfigurationsFromAssembly(typeof(Target).Assembly);");
}
