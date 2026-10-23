use serde_json::{Value, json};
use std::path::Path;

const API: &str = "export {input as read, output as emit} from '@angular/core';";
const FOREIGN: &str = "export declare function input<T>(value:T):unknown; export declare function output<T>():unknown;";
const SOURCE: &str = r#"
import {Component as View} from '@angular/core';
import * as api from './api';
import {read as local} from './api';
const misleading = "@View({}) class Panel {fake=api.read('');}";
function noise<T>(input:T) { return input; }
@View({}) export class Panel {
  title=local(''); changed=api.emit<string>();
  method(input:()=>unknown) { return input(); }
}
"#;

fn call(state: &crate::mcp::McpState, root: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}

fn proof(state: &crate::mcp::McpState, file: &str) -> Value {
    let alias = state.alias_for_path(file).expect("source-owned alias");
    let snapshot = state.semantic_snapshot(&alias).expect("source snapshot");
    json!({"authority":snapshot.authority,"source_hash":snapshot.source_hash})
}

fn setup() -> (tempfile::TempDir, crate::config::CleanCtxConfig, Vec<Value>) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"),
        "export declare function Component(metadata:unknown):any; export declare function input<T>(value:T):unknown; export declare function output<T>():unknown;").unwrap();
    std::fs::write(p.join("foreign.ts"), FOREIGN).unwrap();
    std::fs::write(p.join("api.ts"), API).unwrap();
    // Complete project membership before capturing either owner's witnesses.
    for name in ["app.ts", "neighbor.ts"] {
        std::fs::write(p.join(name), SOURCE).unwrap();
    }
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("members.db").to_string_lossy().into_owned();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config.clone());
    let mut expected = Vec::new();
    for name in ["app.ts", "neighbor.ts"] {
        let file = p.join(name).to_string_lossy().into_owned();
        let response = call(
            &state,
            p,
            "compress_code_context",
            json!({"filePath":file,"fidelity":"high"}),
        );
        assert!(response.get("error").is_none(), "{response}");
        expected.push(proof(&state, &file));
    }
    state.flush_persistence();
    (root, config, expected)
}

fn answers(state: &crate::mcp::McpState, root: &Path) -> Vec<Value> {
    for name in ["app.ts", "neighbor.ts"] {
        let response = call(
            state,
            root,
            "workspace_query",
            json!({"type":"entities_in_file","file_path":root.join(name),"fidelity":"high"}),
        );
        assert!(response.get("error").is_none(), "{response}");
    }
    let response = call(
        state,
        root,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"Panel"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| matches!(edge["relation"].as_str(), Some("HasInput" | "HasOutput")))
        .cloned()
        .collect()
}

fn assert_members(edges: &[Value], expected_owners: usize) {
    assert_eq!(edges.len(), expected_owners * 2, "{edges:?}");
    let mut origins = std::collections::BTreeMap::<String, Vec<(String, String)>>::new();
    for edge in edges {
        origins
            .entry(crate::dictionary::path::canonical_identity_key(
                edge["subject"]["file"].as_str().unwrap(),
            ))
            .or_default()
            .push((
                edge["relation"].as_str().unwrap().into(),
                edge["object"]["name"].as_str().unwrap().into(),
            ));
    }
    assert_eq!(origins.len(), expected_owners);
    for members in origins.values_mut() {
        members.sort();
        assert_eq!(
            *members,
            vec![
                ("HasInput".into(), "title".into()),
                ("HasOutput".into(), "changed".into())
            ]
        );
    }
}

#[test]
fn issue_162_member_lifecycle_restart_restores_authority_before_queries() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, expected) = setup();
    let state = crate::mcp::McpState::new(config);
    for (name, expected) in ["app.ts", "neighbor.ts"].iter().zip(expected) {
        let file = root.path().join(name).to_string_lossy().into_owned();
        let response = call(
            &state,
            root.path(),
            "restore_context",
            json!({"filePath":file}),
        );
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(proof(&state, &file), expected, "before query reacquisition");
        let owner = state.semantic_owner_path(&file);
        let index = state.workspace_index_read();
        assert_eq!(
            serde_json::to_value(
                index
                    .retained_typescript_authority(&owner)
                    .expect("restored index authority")
            )
            .unwrap(),
            expected["authority"]["typescript_authority"]
        );
    }
    assert_members(&answers(&state, root.path()), 2);
}

#[test]
fn issue_162_member_lifecycle_stale_barrel_refusal_mutation_and_source_replacement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, _) = setup();
    let p = root.path();
    std::fs::write(
        p.join("api.ts"),
        "export {input as read, output as emit} from './foreign';",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config);
    // Attempt every stale restore before any query can acquire new authority.
    for name in ["app.ts", "neighbor.ts"] {
        let file = p.join(name).to_string_lossy().into_owned();
        let response = call(&state, p, "restore_context", json!({"filePath":file}));
        assert!(
            response.get("error").is_some(),
            "stale API route adopted: {response}"
        );
        assert!(state.alias_for_path(&file).is_none());
    }
    assert_members(&answers(&state, p), 0);
    std::fs::write(p.join("api.ts"), API).unwrap();
    assert_members(&answers(&state, p), 2);
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), SOURCE);
    assert_eq!(
        std::fs::read_to_string(p.join("neighbor.ts")).unwrap(),
        SOURCE
    );
    std::fs::write(p.join("app.ts"), "export class Panel {}").unwrap();
    let remaining = answers(&state, p);
    assert_members(&remaining, 1);
    assert!(remaining.iter().all(|edge| {
        edge["subject"]["file"]
            .as_str()
            .unwrap()
            .ends_with("neighbor.ts")
    }));
}
