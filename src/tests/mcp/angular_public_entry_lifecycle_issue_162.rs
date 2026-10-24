use serde_json::{Value, json};
use std::path::Path;

const SOURCE: &str = r#"
import * as api from './api';
import type {Intercept} from './api';
import {AuditApi} from './tokens';
const misleading="@angular/common/http exports FakeApi";
function noise<T>(AuditApi:T){return AuditApi;}
export const intercept:Intercept=()=>api.obtain(AuditApi);
"#;

fn manifest(version: &str, enabled: bool) -> String {
    json!({"name":"@angular/common","version":version,"types":"index.d.ts",
        "exports":{".":{"types":"./index.d.ts"},"./http":if enabled {json!({"types":"./http/index.d.ts"})} else {Value::Null}}}).to_string()
}

fn call(state: &crate::mcp::McpState, root: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}

fn proof(state: &crate::mcp::McpState, file: &str) -> Value {
    let alias = state.alias_for_path(file).unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let endpoints: Vec<_> = snapshot
        .edges
        .iter()
        .filter(|edge| {
            edge.subject.entity_type == "Interceptor"
                && edge.relation == crate::layers::meta::semantic::SemanticRelation::Injects
        })
        .map(|edge| (&edge.subject.canonical, &edge.object.canonical))
        .collect();
    assert_eq!(endpoints.len(), 1, "nonempty canonical HTTP dependency");
    assert!(
        endpoints
            .iter()
            .all(|(owner, token)| owner.is_some() && token.is_some())
    );
    let result = json!({"authority":snapshot.authority,"source_hash":snapshot.source_hash,"endpoints":endpoints});
    let packages = result["authority"]["typescript_authority"]["package_exports"]
        .as_array()
        .unwrap();
    assert!(
        packages
            .iter()
            .any(|package| package["name"] == "@angular/common"
                && package["public_entries"]
                    .as_array()
                    .is_some_and(|entries| entries.iter().any(
                        |entry| entry["route"]["specifier"] == "@angular/common/http"
                            && entry["export_name"] == "HttpInterceptorFn"
                    ))),
        "owning package and consumed HTTP public entry remain distinct"
    );
    result
}

fn setup() -> (tempfile::TempDir, crate::config::CleanCtxConfig, Vec<Value>) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(
        p.join("node_modules/@angular/core/index.d.ts"),
        "export declare function inject(token:unknown):unknown;",
    )
    .unwrap();
    std::fs::create_dir_all(p.join("node_modules/@angular/common/http")).unwrap();
    std::fs::write(
        p.join("node_modules/@angular/common/package.json"),
        manifest("20.0.0", true),
    )
    .unwrap();
    std::fs::write(
        p.join("node_modules/@angular/common/index.d.ts"),
        "export interface UnrelatedRootApi {}",
    )
    .unwrap();
    std::fs::write(
        p.join("node_modules/@angular/common/http/index.d.ts"),
        "export type HttpInterceptorFn=(...args:unknown[])=>unknown;",
    )
    .unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export type HttpInterceptorFn=(...args:unknown[])=>unknown;",
    )
    .unwrap();
    std::fs::write(p.join("tokens.ts"), "export class AuditApi {}").unwrap();
    std::fs::write(p.join("api.ts"),"export {inject as obtain} from '@angular/core'; export type {HttpInterceptorFn as Intercept} from '@angular/common/http';").unwrap();
    for name in ["app.ts", "neighbor.ts"] {
        std::fs::write(p.join(name), SOURCE).unwrap();
    }
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("entries.db").to_string_lossy().into_owned();
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
    for (name, expected) in ["app.ts", "neighbor.ts"].iter().zip(&expected) {
        let encoded: String = database
            .query_row(
                "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1",
                [p.join(name).to_string_lossy().into_owned()],
                |row| row.get(0),
            )
            .unwrap();
        let durable: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            durable["typescript_authority"], expected["authority"]["typescript_authority"],
            "SQLite retains complete entry/route/witness proof"
        );
        let endpoints: Vec<_> = durable["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| {
                edge["subject"]["entity_type"] == "Interceptor" && edge["relation"] == "Injects"
            })
            .map(|edge| json!([edge["subject"]["canonical"], edge["object"]["canonical"]]))
            .collect();
        assert_eq!(json!(endpoints), expected["endpoints"]);
    }
    drop(database);
    (root, config, expected)
}

fn answers(state: &crate::mcp::McpState, root: &Path, count: usize) {
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
        json!({"type":"forward_edges","domain":"angular","entity_type":"Interceptor","name":"intercept"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    let selected: Vec<_> = response["result"]["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .collect();
    assert_eq!(
        selected.len(),
        count,
        "exact current HTTP dependencies: {selected:?}"
    );
    let origins: std::collections::BTreeSet<_> = selected
        .iter()
        .map(|edge| {
            assert_eq!(edge["object"]["name"], "AuditApi");
            assert_eq!(
                crate::dictionary::path::canonical_identity_key(
                    edge["object"]["file"].as_str().unwrap()
                ),
                crate::dictionary::path::canonical_identity_key(
                    &root.join("tokens.ts").to_string_lossy()
                )
            );
            crate::dictionary::path::canonical_identity_key(
                edge["subject"]["file"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        origins.len(),
        count,
        "independent source-owned interceptor facts"
    );
}

#[test]
fn issue_162_public_entry_lifecycle_exact_restart_precedes_reacquisition() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, expected) = setup();
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    for (name, expected) in ["app.ts", "neighbor.ts"].iter().zip(expected) {
        let file = p.join(name).to_string_lossy().into_owned();
        let response = call(&state, p, "restore_context", json!({"filePath":file}));
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(
            proof(&state, &file),
            expected,
            "before any reacquisition query"
        );
        let owner = state.semantic_owner_path(&file);
        let index = state.workspace_index_read();
        assert_eq!(
            serde_json::to_value(index.retained_typescript_authority(&owner).unwrap()).unwrap(),
            expected["authority"]["typescript_authority"]
        );
    }
    answers(&state, p, 2);
}

#[test]
fn issue_162_public_entry_lifecycle_stale_exports_config_and_version_refuse_restore() {
    let _serial = crate::protocol::handler_response_serial();
    for mutation in ["exports", "config", "version"] {
        let (root, config, _) = setup();
        let p = root.path();
        let original_config = std::fs::read(p.join("tsconfig.json")).unwrap();
        match mutation {
            "exports" => std::fs::write(p.join("node_modules/@angular/common/package.json"),manifest("20.0.0",false)).unwrap(),
            "version" => std::fs::write(p.join("node_modules/@angular/common/package.json"),manifest("20.0.1",true)).unwrap(),
            _ => std::fs::write(p.join("tsconfig.json"),json!({"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"paths":{"@angular/common/http":["./foreign.ts"]}}}).to_string()).unwrap(),
        }
        let state = crate::mcp::McpState::new(config);
        for name in ["app.ts", "neighbor.ts"] {
            let file = p.join(name).to_string_lossy().into_owned();
            let response = call(&state, p, "restore_context", json!({"filePath":file}));
            assert!(
                response.get("error").is_some(),
                "stale {mutation} evidence adopted: {response}"
            );
            assert!(state.alias_for_path(&file).is_none());
        }
        answers(&state, p, if mutation == "version" { 2 } else { 0 });
        std::fs::write(p.join("tsconfig.json"), original_config).unwrap();
        std::fs::write(
            p.join("node_modules/@angular/common/package.json"),
            manifest("20.0.0", true),
        )
        .unwrap();
        answers(&state, p, 2);
        for name in ["app.ts", "neighbor.ts"] {
            assert_eq!(std::fs::read_to_string(p.join(name)).unwrap(), SOURCE);
        }
    }
}
