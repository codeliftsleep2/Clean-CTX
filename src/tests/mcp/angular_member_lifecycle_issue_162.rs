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
    let endpoints: Vec<_> = snapshot
        .edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.relation,
                crate::layers::meta::semantic::SemanticRelation::HasInput
                    | crate::layers::meta::semantic::SemanticRelation::HasOutput
            )
        })
        .map(|edge| (&edge.subject.canonical, &edge.object.canonical))
        .collect();
    assert_eq!(endpoints.len(), 2);
    assert!(
        endpoints
            .iter()
            .all(|(owner, member)| owner.is_some() && member.is_some())
    );
    json!({"authority":snapshot.authority,"source_hash":snapshot.source_hash,"endpoints":endpoints})
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
    let database = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    for (name, proof) in ["app.ts", "neighbor.ts"].iter().zip(&expected) {
        let file = p.join(name).to_string_lossy().into_owned();
        let encoded: String = database
            .query_row(
                "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1",
                [&file],
                |row| row.get(0),
            )
            .unwrap();
        let durable: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            durable["typescript_authority"],
            proof["authority"]["typescript_authority"]
        );
        let endpoints: Vec<_> = durable["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| matches!(edge["relation"].as_str(), Some("HasInput" | "HasOutput")))
            .map(|edge| json!([edge["subject"]["canonical"], edge["object"]["canonical"]]))
            .collect();
        assert_eq!(
            json!(endpoints),
            proof["endpoints"],
            "SQLite retains complete member endpoints"
        );
    }
    drop(database);
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

#[test]
fn issue_162_member_identity_endpoints_require_retained_owner_and_field_declarations() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, _) = setup();
    let state = crate::mcp::McpState::new(config);
    // Establish nonempty public behavior first; endpoint evidence must come from
    // the actual source-owned producer, never names manufactured by this test.
    assert_members(&answers(&state, root.path()), 2);
    for name in ["app.ts", "neighbor.ts"] {
        let file = root.path().join(name).to_string_lossy().into_owned();
        let response = call(
            &state,
            root.path(),
            "compress_code_context",
            json!({"filePath":file,"fidelity":"high"}),
        );
        assert!(response.get("error").is_none(), "{response}");
        let alias = state.alias_for_path(&file).unwrap();
        let snapshot = state.semantic_snapshot(&alias).unwrap();
        let selected: Vec<_> = snapshot
            .edges
            .iter()
            .filter(|edge| {
                matches!(
                    edge.relation,
                    crate::layers::meta::semantic::SemanticRelation::HasInput
                        | crate::layers::meta::semantic::SemanticRelation::HasOutput
                )
            })
            .collect();
        assert_eq!(selected.len(), 2, "nonempty source-owned member facts");
        let authority = serde_json::to_value(&snapshot.authority).unwrap();
        let declarations = authority["typescript_authority"]["declarations"]
            .as_array()
            .unwrap();
        for edge in selected {
            for entity in [&edge.subject, &edge.object] {
                let endpoint = entity
                    .canonical
                    .as_ref()
                    .expect("member relation must retain typed declaration identity");
                let declaration = endpoint
                    .declaration()
                    .expect("members are declaration identities");
                assert_eq!(declaration.name, entity.name);
                assert_eq!(
                    crate::dictionary::path::canonical_identity_key(
                        &declaration.source.to_string_lossy()
                    ),
                    crate::dictionary::path::canonical_identity_key(&file)
                );
                assert_eq!(declaration.source_hash, snapshot.source_hash);
                assert!(
                    declarations.iter().any(|id| id["name"] == declaration.name
                        && id["start"] == declaration.start
                        && id["end"] == declaration.end
                        && id["kind"] == declaration.kind),
                    "endpoint must match retained declaration evidence"
                );
                let spelling = &SOURCE[declaration.start..declaration.end];
                assert!(spelling.contains(&entity.name));
            }
        }
    }
}

#[test]
fn issue_162_member_lifecycle_span_replacement_and_external_deletion_retire_exact_owner() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, expected) = setup();
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    assert_members(&answers(&state, p), 2);
    let file = p.join("app.ts").to_string_lossy().into_owned();
    std::fs::write(&file, format!("// shift all declaration spans\n{SOURCE}")).unwrap();
    let response = call(
        &state,
        p,
        "compress_code_context",
        json!({"filePath":file,"fidelity":"high"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    let shifted = proof(&state, &file);
    assert_ne!(shifted["source_hash"], expected[0]["source_hash"]);
    assert_ne!(
        shifted["endpoints"], expected[0]["endpoints"],
        "new source replaces old declaration identities"
    );
    for _ in 0..2 {
        assert_members(&answers(&state, p), 2);
    }
    std::fs::remove_file(&file).unwrap();
    let response = call(&state, p, "refresh_workspace", json!({}));
    assert!(response.get("error").is_none(), "{response}");
    // Do not query the deleted file or supply editor bytes to recreate it.
    for _ in 0..2 {
        let response = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges",
            "domain":"angular","entity_type":"Component","name":"Panel"}),
        );
        assert!(response.get("error").is_none(), "{response}");
        let edges: Vec<_> = response["result"]["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| matches!(edge["relation"].as_str(), Some("HasInput" | "HasOutput")))
            .cloned()
            .collect();
        assert_members(&edges, 1);
        assert!(edges.iter().all(|edge| {
            edge["subject"]["file"]
                .as_str()
                .unwrap()
                .ends_with("neighbor.ts")
        }));
    }
    assert_eq!(
        std::fs::read_to_string(p.join("neighbor.ts")).unwrap(),
        SOURCE
    );
}
