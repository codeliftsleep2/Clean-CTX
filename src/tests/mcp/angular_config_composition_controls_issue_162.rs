//! Pair-level config composition controls; no evaluated provider union.
use super::angular_bootstrap_inline_controls_tests::{call, capture, raw, setup as base_setup};
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use serde_json::json;
pub(super) const NATIVE: &str = "import type {ApplicationConfig} from '@angular/core'; import {mergeApplicationConfig as combine} from '@angular/core'; import {leafConfig as Base} from './base'; import {leafConfig as Feature} from './feature';";
pub(super) fn setup(args: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = base_setup("");
    let p = root.path();
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"),"export declare function Component(metadata:unknown):any; export interface ApplicationConfig {providers:unknown[];} export declare function mergeApplicationConfig(...configs:ApplicationConfig[]):ApplicationConfig;").unwrap();
    for name in ["base", "feature"] {
        std::fs::write(p.join(format!("{name}.ts")),"import type {ApplicationConfig} from '@angular/core'; import {Token} from './left'; export const leafConfig:ApplicationConfig={providers:[Token]};").unwrap();
    }
    std::fs::write(
        p.join("result.ts"),
        format!("{NATIVE} export const appConfig:ApplicationConfig=combine({args});"),
    )
    .unwrap();
    std::fs::write(p.join("caller.ts"),"import {bootstrapApplication} from '@angular/platform-browser'; import {Root} from './root'; import {appConfig} from './result'; bootstrapApplication(Root,appConfig);").unwrap();
    std::fs::write(
        p.join("fake-merge.ts"),
        "export function mergeApplicationConfig(...configs:unknown[]){return {};}",
    )
    .unwrap();
    (root, config)
}
pub(super) fn facts(state: &crate::mcp::McpState, p: &std::path::Path, inputs: &[&str]) {
    let identity = |file: &str| {
        crate::dictionary::path::canonical_identity_key(&p.join(file).to_string_lossy())
    };
    for reverse in [false, true] {
        let mut answer = call(
            state,
            p,
            "workspace_query",
            json!({"type":if reverse{"reverse_edges"}else{"forward_edges"},"domain":"angular","entity_type":"ApplicationConfig","name":if reverse{"leafConfig"}else{"appConfig"}}),
        );
        if reverse && inputs.contains(&"result.ts") {
            // This fixture's newly admitted merged input is named nested, not leafConfig.
            let nested = call(
                state,
                p,
                "workspace_query",
                json!({"type":"reverse_edges","domain":"angular","entity_type":"ApplicationConfig","name":"nested"}),
            );
            answer["edges"]
                .as_array_mut()
                .unwrap()
                .extend(nested["edges"].as_array().unwrap().iter().cloned());
        }
        let mut actual = answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| {
                e["relation"] == "IncludesApplicationConfig" && e["subject"]["name"] == "appConfig"
            })
            .map(|e| {
                (
                    crate::dictionary::path::canonical_identity_key(
                        e["subject"]["file"].as_str().unwrap(),
                    ),
                    crate::dictionary::path::canonical_identity_key(
                        e["object"]["file"].as_str().unwrap(),
                    ),
                )
            })
            .collect::<Vec<_>>();
        actual.sort();
        let mut wanted = inputs
            .iter()
            .map(|f| (identity("result.ts"), identity(f)))
            .collect::<Vec<_>>();
        wanted.sort();
        assert_eq!(actual, wanted, "{answer}");
        assert!(
            !answer["edges"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |e| e["relation"] == "RegistersProvider" && e["subject"]["name"] == "appConfig"
                )
        );
    }
}
#[test]
fn issue_162_config_composition_controls_standalone_cold_pair_support_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("Base,Base");
    let p = root.path();
    std::fs::remove_file(p.join("caller.ts")).unwrap();
    let state = crate::mcp::McpState::new(config);
    facts(&state, p, &["base.ts"]);
    let authority = capture(&state, p, "result.ts");
    let facet = authority
        .declaration_facets
        .iter()
        .find(|f| f.declaration.name == "appConfig")
        .unwrap();
    let ExpressionValue::Call(_, args) = &facet.initializer.as_ref().unwrap().value else {
        panic!()
    };
    assert_eq!(args.len(), 2);
    assert_ne!(args[0].start, args[1].start);
    for (args, inputs) in [
        ("Base", vec!["base.ts"]),
        ("", vec![]),
        ("Base,Feature", vec!["base.ts", "feature.ts"]),
    ] {
        std::fs::write(
            p.join("result.ts"),
            format!("{NATIVE} export const appConfig:ApplicationConfig=combine({args});"),
        )
        .unwrap();
        facts(&state, p, &inputs);
    }
}
#[test]
fn issue_162_config_composition_controls_native_and_result_input_refusals() {
    let _serial = crate::protocol::handler_response_serial();
    for (initializer, declaration, expected) in [
        (
            "combine(Base,{})",
            "const appConfig:ApplicationConfig",
            vec!["base.ts"],
        ),
        (
            "combine(Base,...[Feature])",
            "const appConfig:ApplicationConfig",
            vec!["base.ts"],
        ),
        (
            "combine(Base,Missing)",
            "const appConfig:ApplicationConfig",
            vec!["base.ts"],
        ),
        ("combine(Base)", "let appConfig:ApplicationConfig", vec![]),
        ("combine(Base)", "const appConfig", vec![]),
        (
            "({providers:[]})",
            "const appConfig:ApplicationConfig",
            vec![],
        ),
        ("foreign(Base)", "const appConfig:ApplicationConfig", vec![]),
    ] {
        let (root, config) = setup("Base,Feature");
        let p = root.path();
        std::fs::write(p.join("result.ts"),format!("{NATIVE} import {{mergeApplicationConfig as foreign}} from './fake-merge'; export {declaration}={initializer};")).unwrap();
        let state = crate::mcp::McpState::new(config);
        facts(&state, p, &expected);
    }
}
#[test]
fn issue_162_config_composition_controls_missing_crossed_proof_and_serde() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("Base,Feature");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, p, "result.ts");
    let edges = crate::angular_meta::config_composition::project(&authority);
    assert_eq!(edges.len(), 2);
    assert!(authority.admits_relation_evidence(&edges));
    let result = authority
        .declaration_facets
        .iter()
        .position(|f| f.declaration.name == "appConfig")
        .unwrap();
    for mode in 0..9 {
        let mut bad = authority.clone();
        match mode {
            0 => bad.demanded_occurrences.clear(),
            1 => bad.package_exports.clear(),
            2 => bad.declaration_facets[result]
                .declaration
                .source_hash
                .push('x'),
            3 => {
                bad.declaration_facets[result]
                    .initializer
                    .as_mut()
                    .unwrap()
                    .start += 1
            }
            _ => {
                let ExpressionValue::Call(callee, args) = &mut bad.declaration_facets[result]
                    .initializer
                    .as_mut()
                    .unwrap()
                    .value
                else {
                    panic!()
                };
                match mode {
                    4 => callee.start += 1,
                    5 => args.swap(0, 1),
                    6 => args[0].start += 1,
                    7 => args[0].value = args[1].value.clone(),
                    _ => args.clear(),
                }
            }
        }
        assert!(!bad.admits_relation_evidence(&edges), "mode={mode}");
    }
    let restored: crate::ir::typescript_resolution::TypeScriptAuthority =
        serde_json::from_value(serde_json::to_value(&authority).unwrap()).unwrap();
    assert!(restored.admits_relation_evidence(&edges));
    let mut forged = edges[0].clone();
    forged.subject.canonical = edges[1].object.canonical.clone();
    assert!(!authority.admits_relation_evidence(&[forged]));
}
#[test]
fn issue_162_config_composition_controls_sqlite_before_query_and_stale_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("Base,Feature");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let authority = capture(&state, p, "result.ts");
    let alias = state
        .alias_for_path(&p.join("result.ts").to_string_lossy())
        .unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let wanted = snapshot
        .edges
        .iter()
        .map(crate::mcp::state::durable_semantics::DurableSemanticEdge::from)
        .collect::<Vec<_>>();
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("result.ts")}),
    );
    drop(state);
    let db = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let encoded: String = db
        .query_row(
            "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path=?1",
            [p.join("result.ts").to_string_lossy().into_owned()],
            |r| r.get(0),
        )
        .unwrap();
    let stored: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        stored["typescript_authority"],
        serde_json::to_value(&authority).unwrap()
    );
    assert_eq!(stored["edges"], serde_json::to_value(wanted).unwrap());
    drop(db);
    let state = crate::mcp::McpState::new(config.clone());
    call(
        &state,
        p,
        "restore_context",
        json!({"filePath":p.join("result.ts")}),
    );
    let owner = state.semantic_owner_path(&p.join("result.ts").to_string_lossy());
    assert_eq!(
        serde_json::to_value(
            state
                .workspace_index_read()
                .retained_typescript_authority(&owner)
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(&authority).unwrap()
    );
    facts(&state, p, &["base.ts", "feature.ts"]);
    drop(state);
    let original = std::fs::read(p.join("feature.ts")).unwrap();
    let result = std::fs::read(p.join("result.ts")).unwrap();
    std::fs::write(
        p.join("feature.ts"),
        "export const leafConfig={providers:[]};",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config);
    let rejected = raw(
        &state,
        p,
        "restore_context",
        json!({"filePath":p.join("result.ts")}),
    );
    assert!(
        rejected.get("error").is_some() || rejected["result"]["isError"] == true,
        "{rejected}"
    );
    facts(&state, p, &["base.ts"]);
    std::fs::write(p.join("feature.ts"), original).unwrap();
    facts(&state, p, &["base.ts", "feature.ts"]);
    assert_eq!(std::fs::read(p.join("result.ts")).unwrap(), result);
}
