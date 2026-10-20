//! Issue #155: source-equivalent cold requests must converge before answering.
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
        &json!(155),
        "workspace_query",
        &json!({"arguments": args}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    response["result"]["structuredContent"].clone()
}

fn fixture(root: &Path, role: &str) {
    let (external, body) = match role {
        "Hub" => (
            "global::Microsoft.AspNetCore.SignalR.Hub",
            "public void Broadcast() {}",
        ),
        "MapperProfile" => (
            "global::AutoMapper.Profile",
            "public Target() { CreateMap<Source, Destination>(); }",
        ),
        "DbContext" => (
            "global::Microsoft.EntityFrameworkCore.DbContext",
            "public global::Microsoft.EntityFrameworkCore.DbSet<Row> Rows { get; set; }",
        ),
        "Controller" => (
            "global::Microsoft.AspNetCore.Mvc.ControllerBase",
            "[global::Microsoft.AspNetCore.Mvc.HttpGetAttribute] public int Get() { return 1; }",
        ),
        _ => unreachable!(),
    };
    for (file, source) in [
        (
            "Foundation.cs",
            format!("public class Foundation : {external} {{}}"),
        ),
        ("Bridge.cs", "public class Bridge : Foundation {}".into()),
        (
            "Target.cs",
            format!("public class Target : Bridge {{ {body} }}"),
        ),
        ("Unrelated.cs", "public class Unrelated {}".into()),
    ] {
        std::fs::write(root.join(file), source).unwrap();
    }
}

fn target(role: &str, surface: &str) -> Value {
    if surface == "entities_in_file" {
        json!({"type":surface, "file_path":"Target.cs", "fidelity":"high"})
    } else {
        json!({"type":surface, "domain":"dotnet", "entity_type":role, "name":"Target"})
    }
}

fn facts(result: &Value, surface: &str) -> BTreeSet<String> {
    let key = if surface == "entities_in_file" {
        "entities"
    } else {
        "edges"
    };
    result[key]
        .as_array()
        .expect("query facts")
        .iter()
        .map(|fact| serde_json::to_string(fact).unwrap())
        .collect()
}

fn assert_role(result: &Value, role: &str, surface: &str) {
    if surface == "entities_in_file" {
        assert!(
            result["entities"].as_array().unwrap().iter().any(|entity| {
                entity["domain"] == "dotnet"
                    && entity["entity_type"] == role
                    && entity["name"] == "Target"
            }),
            "missing {role} in file response: {result:#}"
        );
        assert_eq!(result["coverage"]["source_complete"], true, "{result:#}");
        assert_eq!(
            result["coverage"]["result_semantics"], "exact",
            "{result:#}"
        );
    } else {
        let relation = match role {
            "Hub" => "HubMethodTargets",
            "MapperProfile" => "MapsFrom",
            "DbContext" => "HasEntity",
            "Controller" => "ControllerAction",
            _ => unreachable!(),
        };
        assert!(
            result["edges"]
                .as_array()
                .unwrap()
                .iter()
                .any(|edge| edge["relation"] == relation),
            "missing {relation}: {result:#}"
        );
        assert_ne!(
            result["coverage"]["status"], "identity_not_indexed",
            "{result:#}"
        );
    }
}

fn convergence(role: &str, surface: &str) {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), role);
    // The control explicitly acquires both bases. Each order below gets a fresh state.
    let control = state();
    for name in ["Foundation", "Bridge"] {
        query(
            &control,
            root.path(),
            json!({"type":"find_entities", "name":name}),
        );
    }
    let expected = query(&control, root.path(), target(role, surface));
    assert_role(&expected, role, surface);
    for order in 0..5 {
        let state = state();
        match order {
            1 => {
                for name in ["Foundation", "Bridge"] {
                    query(
                        &state,
                        root.path(),
                        json!({"type":"find_entities", "name":name}),
                    );
                }
            }
            2 => {
                for file in ["Foundation.cs", "Bridge.cs"] {
                    query(
                        &state,
                        root.path(),
                        json!({"type":"entities_in_file", "file_path":file}),
                    );
                }
            }
            3 => {
                query(
                    &state,
                    root.path(),
                    json!({"type":"find_entities", "name":"Unrelated"}),
                );
            }
            4 => {
                query(&state, root.path(), target(role, surface));
            }
            _ => {}
        }
        let actual = query(&state, root.path(), target(role, surface));
        assert_role(&actual, role, surface);
        assert_eq!(
            facts(&actual, surface),
            facts(&expected, surface),
            "{role}/{surface}, order {order}: {actual:#}"
        );
    }
}

macro_rules! case {
    ($name:ident, $role:literal, $surface:literal) => {
        #[test]
        fn $name() {
            convergence($role, $surface);
        }
    };
}

case!(issue_155_hub_edges, "Hub", "forward_edges");
case!(issue_155_profile_edges, "MapperProfile", "forward_edges");
case!(issue_155_context_edges, "DbContext", "forward_edges");
case!(issue_155_controller_edges, "Controller", "forward_edges");
case!(issue_155_hub_file, "Hub", "entities_in_file");
case!(issue_155_profile_file, "MapperProfile", "entities_in_file");
case!(issue_155_context_file, "DbContext", "entities_in_file");
case!(issue_155_controller_file, "Controller", "entities_in_file");
