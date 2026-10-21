//! Issue #157 public-path candidates. Runtime RED requires operator execution.
use crate::mcp::McpState;
use serde_json::{Value, json};
use std::path::Path;

fn state() -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    McpState::new(config)
}

fn call(state: &McpState, root: &Path, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(157),
        "workspace_query",
        &json!({"arguments": args}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    response["result"]["structuredContent"].clone()
}

fn admit(state: &McpState, root: &Path, file: &str) {
    call(
        state,
        root,
        json!({"type":"entities_in_file", "file_path":file, "fidelity":"high"}),
    );
}

fn target() -> Value {
    json!({"type":"forward_edges", "domain":"angular",
        "entity_type":"Module", "name":"AppModule"})
}

fn facts(result: &Value) -> Vec<(String, String)> {
    let mut facts: Vec<_> = result["edges"]
        .as_array()
        .expect("edge response")
        .iter()
        .filter(|edge| edge["relation"] == "DeclaresInModule")
        .map(|edge| {
            (
                edge["object"]["name"].as_str().unwrap().to_string(),
                edge["object"]["entity_type"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    facts.sort();
    facts.dedup();
    facts
}

fn expect(state: &McpState, root: &Path, expected: &[(&str, &str)]) {
    let result = call(state, root, target());
    let mut wanted: Vec<_> = expected
        .iter()
        .map(|(name, kind)| (name.to_string(), kind.to_string()))
        .collect();
    wanted.sort();
    assert_eq!(
        facts(&result),
        wanted,
        "module contributor truth: {result:#}"
    );
}

fn write(root: &Path, file: &str, source: &str) {
    std::fs::write(root.join(file), source).unwrap();
}

fn declaration(kind: &str) -> String {
    format!(
        "import {{ {kind} }} from '@angular/core';\n@{kind}({{selector:'[shared]',name:'shared'}}) export class Shared {{}}"
    )
}

fn fixture(root: &Path, spread: bool) {
    write(root, "leaf.ts", &declaration("Directive"));
    write(root, "unrelated.ts", "export class Unrelated {}");
    let (binding, expression) = if spread {
        write(
            root,
            "shared.ts",
            "import { Shared } from './leaf'; export const SHARED = [Shared];",
        );
        ("import { SHARED } from './shared';", "...SHARED")
    } else {
        ("import { Shared } from './leaf';", "Shared")
    };
    write(
        root,
        "app.module.ts",
        &format!(
            "import {{ NgModule }} from '@angular/core';\n{binding}\n@NgModule({{declarations:[{expression}]}}) export class AppModule {{}}"
        ),
    );
}

#[test]
fn issue_157_direct_cold_warm_and_admission_orders() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), false);
    for order in ["cold", "contributor_query", "entities", "unrelated"] {
        let state = state();
        match order {
            "contributor_query" => {
                call(
                    &state,
                    root.path(),
                    json!({"type":"forward_edges", "domain":"angular",
                        "entity_type":"Directive", "name":"Shared"}),
                );
            }
            "entities" => admit(&state, root.path(), "leaf.ts"),
            "unrelated" => admit(&state, root.path(), "unrelated.ts"),
            _ => {}
        }
        expect(&state, root.path(), &[("Shared", "Directive")]);
        expect(&state, root.path(), &[("Shared", "Directive")]);
    }
}

#[test]
fn issue_157_direct_contributor_content_mutation_retracts_old_kind() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), false);
    let warm = state();
    expect(&warm, root.path(), &[("Shared", "Directive")]);
    write(root.path(), "leaf.ts", &declaration("Pipe"));
    // A fresh control proves the changed relationship is supported before
    // comparing an unchanged target's cached result.
    expect(&state(), root.path(), &[("Shared", "Pipe")]);
    expect(&warm, root.path(), &[("Shared", "Pipe")]);
}

#[test]
fn issue_157_direct_contributor_deletion_and_restoration() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), false);
    let warm = state();
    expect(&warm, root.path(), &[("Shared", "Directive")]);
    std::fs::remove_file(root.path().join("leaf.ts")).unwrap();
    expect(&state(), root.path(), &[]);
    expect(&warm, root.path(), &[]);
    write(root.path(), "leaf.ts", &declaration("Pipe"));
    expect(&state(), root.path(), &[("Shared", "Pipe")]);
    expect(&warm, root.path(), &[("Shared", "Pipe")]);
}

#[test]
fn issue_157_imported_array_tracks_leaf_and_array_content() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), true);
    let warm = state();
    expect(&warm, root.path(), &[("Shared", "Directive")]);
    write(root.path(), "leaf.ts", &declaration("Pipe"));
    expect(&state(), root.path(), &[("Shared", "Pipe")]);
    expect(&warm, root.path(), &[("Shared", "Pipe")]);
    write(root.path(), "shared.ts", "export const SHARED = [];");
    expect(&state(), root.path(), &[]);
    expect(&warm, root.path(), &[]);
}

#[test]
fn issue_157_reversed_batch_after_contributor_mutation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), false);
    for reverse in [false, true] {
        write(root.path(), "leaf.ts", &declaration("Directive"));
        let warm = state();
        expect(&warm, root.path(), &[("Shared", "Directive")]);
        write(root.path(), "leaf.ts", &declaration("Pipe"));
        expect(&state(), root.path(), &[("Shared", "Pipe")]);
        let mut module = target();
        module["id"] = json!("module");
        let leaf = json!({"id":"leaf", "type":"entities_in_file",
            "file_path":"leaf.ts", "fidelity":"high"});
        let queries = if reverse {
            vec![leaf, module]
        } else {
            vec![module, leaf]
        };
        let result = call(&warm, root.path(), json!({"queries": queries}));
        let module = result["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == "module")
            .unwrap();
        assert_eq!(module["status"], "ok", "{result:#}");
        assert_eq!(
            facts(&module["result"]),
            vec![("Shared".to_string(), "Pipe".to_string())],
            "surviving batch truth: {result:#}"
        );
    }
}
