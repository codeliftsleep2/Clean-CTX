//! Integrated occurrence-owner controls; no effective application provider model.
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::path::Path;
pub(super) const PREFIX: &str = "import {start as launch} from './api'; import {Root} from './root'; import {GROUP} from './group';";
pub(super) const CALL: &str = "launch(Root,{providers:[...GROUP]});";
const ROOT: &str = "import {Component} from '@angular/core'; @Component({}) export class Root {}";
const GROUP: &str = "import {Token as Left} from './left'; import {Token as Right} from './right'; export const GROUP=[Left,{provide:Right,useValue:1},Left];";
const API: &str = "export {bootstrapApplication as start} from '@angular/platform-browser';";
pub(super) fn setup(body: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    let package = p.join("node_modules/@angular/platform-browser");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@angular/platform-browser","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("index.d.ts"),
        "export declare function bootstrapApplication(root:unknown,config?:unknown):unknown;",
    )
    .unwrap();
    for (file, text) in [
        ("root.ts", ROOT),
        ("group.ts", GROUP),
        ("api.ts", API),
        ("left.ts", "export class Token {}"),
        ("right.ts", "export class Token {}"),
        (
            "fake.ts",
            "export function bootstrapApplication(root:unknown,config?:unknown){}",
        ),
    ] {
        std::fs::write(p.join(file), text).unwrap();
    }
    std::fs::write(p.join("main.ts"), format!("{PREFIX}{body}")).unwrap();
    std::fs::write(p.join("neighbor.ts"), format!("{PREFIX}{CALL}")).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("proof.db").to_string_lossy().into_owned();
    (root, config)
}
pub(super) fn raw(state: &crate::mcp::McpState, p: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(p);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}
pub(super) fn call(state: &crate::mcp::McpState, p: &Path, tool: &str, args: Value) -> Value {
    let r = raw(state, p, tool, args);
    assert!(r.get("error").is_none(), "{r}");
    assert_ne!(r["result"]["isError"], true, "{r}");
    r["result"]["structuredContent"].clone()
}
pub(super) fn capture(state: &crate::mcp::McpState, p: &Path, file: &str) -> TypeScriptAuthority {
    call(
        state,
        p,
        "compress_code_context",
        json!({"filePath":p.join(file),"fidelity":"high"}),
    );
    let alias = state
        .alias_for_path(&p.join(file).to_string_lossy())
        .unwrap();
    state
        .semantic_snapshot(&alias)
        .unwrap()
        .authority
        .typescript_authority
        .unwrap()
}
pub(super) fn facts(state: &crate::mcp::McpState, p: &Path, expected: &[(&str, usize, usize)]) {
    let key = |file: &str| {
        crate::dictionary::path::canonical_identity_key(&p.join(file).to_string_lossy())
    };
    let mut wanted = Vec::new();
    for (file, left, right) in expected {
        for (count, token) in [(*left, "left.ts"), (*right, "right.ts")] {
            for _ in 0..count {
                wanted.push((key(file), key(token)));
            }
        }
    }
    wanted.sort();
    for reverse in [false, true] {
        let answer = call(
            state,
            p,
            "workspace_query",
            json!({"type":if reverse {"reverse_edges"}else{"forward_edges"},"domain":"angular","entity_type":if reverse{"ProviderToken"}else{"ApplicationBootstrap"},"name":if reverse{"Token"}else{"bootstrapApplication"}}),
        );
        let selected = answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| {
                e["relation"] == "RegistersProvider"
                    && e["subject"]["entity_type"] == "ApplicationBootstrap"
                    && e["object"]["name"] == "Token"
            })
            .collect::<Vec<_>>();
        let mut actual = selected
            .iter()
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
        assert_eq!(actual, wanted, "reverse={reverse}; {answer}");
        assert!(
            selected
                .iter()
                .all(|e| e["subject"].get("canonical").is_none()
                    && e["object"].get("canonical").is_none())
        );
    }
}
#[test]
fn issue_162_bootstrap_inline_controls_composition_strategy_and_refusal_siblings() {
    let _serial = crate::protocol::handler_response_serial();
    let body = "import {Token as Left} from './left'; import {Token as Right} from './right'; const LOCAL=[Left]; const CYCLE=[...CYCLE,Right]; launch(Root,{providers:[...LOCAL,[...GROUP],...CYCLE,{provide:Left,useClass:Right},{provide:Right,useExisting:Left},{provide:Left,useFactory:make},{provide:'named',useValue:Left}]}); function make(){return {};}; launch(Root,{providers:[],providers:[Left]}); launch(Root,{providers:[Left],...{providers:[Right]}});";
    let (root, config) = setup(body);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, p, "main.ts");
    let edges = crate::angular_meta::bootstrap_references::project(&authority);
    assert!(authority.admits_relation_evidence(&edges));
    assert_eq!(
        edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .map(|e| serde_json::to_string(&e.object.canonical).unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3,
        "{edges:?}"
    );
    for relation in [
        SemanticRelation::ProviderUsesClass,
        SemanticRelation::ProviderUsesFactory,
        SemanticRelation::ProviderUsesExisting,
        SemanticRelation::ProviderUsesValue,
    ] {
        assert!(
            edges.iter().any(|e| e.relation == relation),
            "{relation:?}: {edges:?}"
        );
    }
    assert_eq!(
        edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
            .count(),
        3
    );
    facts(&state, p, &[("main.ts", 1, 1), ("neighbor.ts", 1, 1)]);
    for reverse in [false, true] {
        let answer = call(
            &state,
            p,
            "workspace_query",
            json!({
                "type":if reverse {"reverse_edges"}else{"forward_edges"},
                "domain":"angular", "entity_type":if reverse {"ProviderToken"}else{"ApplicationBootstrap"},
                "name":if reverse {"\"named\""}else{"bootstrapApplication"}
            }),
        );
        let named = answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "RegistersProvider" && e["object"]["name"] == "\"named\"")
            .collect::<Vec<_>>();
        assert_eq!(named.len(), 1, "{answer}");
        for endpoint in ["subject", "object"] {
            assert_eq!(
                crate::dictionary::path::canonical_identity_key(
                    named[0][endpoint]["file"].as_str().unwrap()
                ),
                crate::dictionary::path::canonical_identity_key(
                    &p.join("main.ts").to_string_lossy()
                )
            );
        }
    }
}
#[test]
fn issue_162_bootstrap_inline_controls_missing_crossed_field_entry_and_serde_refusal() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(CALL);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, p, "main.ts");
    let registrations = crate::angular_meta::bootstrap_references::project(&authority)
        .into_iter()
        .filter(|e| e.relation == SemanticRelation::RegistersProvider)
        .collect::<Vec<_>>();
    assert_eq!(registrations.len(), 3, "retained repeated Left entry");
    assert_eq!(
        registrations
            .iter()
            .map(|e| serde_json::to_string(&e.object.canonical).unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        2
    );
    assert!(authority.admits_relation_evidence(&registrations));
    for mode in 0..12 {
        let mut bad = authority.clone();
        match mode {
            0 => bad.invocation_facets.clear(),
            1 => bad.demanded_occurrences.clear(),
            2 => bad.reference_sites.clear(),
            3 => bad.package_exports.clear(),
            4 => bad.invocation_facets[0].source_hash.push('x'),
            5 => bad.invocation_facets[0].source.push("crossed.ts"),
            6 => bad.invocation_facets[0].expression.start += 1,
            7 => {
                let duplicate = bad.invocation_facets[0].clone();
                bad.invocation_facets.push(duplicate);
            }
            _ => {
                let ExpressionValue::Call(_, args) = &mut bad.invocation_facets[0].expression.value
                else {
                    panic!()
                };
                match mode {
                    8 => args.swap(0, 1),
                    9 => args[1].start += 1,
                    10 => {
                        let ExpressionValue::Object(fields) = &mut args[1].value else {
                            panic!()
                        };
                        fields[0].0 = "viewProviders".into();
                    }
                    _ => {
                        let ExpressionValue::Object(fields) = &mut args[1].value else {
                            panic!()
                        };
                        fields.push(fields[0].clone());
                    }
                }
            }
        }
        assert!(
            !bad.admits_relation_evidence(&registrations),
            "forged mode={mode}"
        );
    }
    let restored: TypeScriptAuthority =
        serde_json::from_value(serde_json::to_value(&authority).unwrap()).unwrap();
    assert!(restored.admits_relation_evidence(&registrations));
    let mut forged = registrations[0].clone();
    forged.object = crate::angular_meta::bootstrap_references::project(&authority)
        .into_iter()
        .find(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
        .unwrap()
        .object;
    assert!(!authority.admits_relation_evidence(&[forged]));
}
#[test]
fn issue_162_bootstrap_inline_controls_sqlite_complete_before_query_and_multiplicity() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(&CALL.repeat(2));
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let mut expected = Vec::new();
    for file in ["main.ts", "neighbor.ts"] {
        capture(&state, p, file);
        let alias = state
            .alias_for_path(&p.join(file).to_string_lossy())
            .unwrap();
        let snapshot = state.semantic_snapshot(&alias).unwrap();
        let durable = snapshot
            .edges
            .iter()
            .map(crate::mcp::state::durable_semantics::DurableSemanticEdge::from)
            .collect::<Vec<_>>();
        expected.push(json!({"proof":snapshot.authority.typescript_authority,"edges":durable}));
        call(&state, p, "save_context", json!({"filePath":p.join(file)}));
    }
    drop(state);
    let db = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    for (file, wanted) in ["main.ts", "neighbor.ts"].into_iter().zip(&expected) {
        let encoded: String = db
            .query_row(
                "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path=?1",
                [p.join(file).to_string_lossy().into_owned()],
                |r| r.get(0),
            )
            .unwrap();
        let stored: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(stored["typescript_authority"], wanted["proof"]);
        assert_eq!(stored["edges"], wanted["edges"]);
    }
    drop(db);
    let state = crate::mcp::McpState::new(config);
    for (file, wanted) in ["main.ts", "neighbor.ts"].into_iter().zip(expected) {
        call(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join(file)}),
        );
        let owner = state.semantic_owner_path(&p.join(file).to_string_lossy());
        assert_eq!(
            serde_json::to_value(
                state
                    .workspace_index_read()
                    .retained_typescript_authority(&owner)
                    .unwrap()
            )
            .unwrap(),
            wanted["proof"]
        );
    }
    facts(&state, p, &[("main.ts", 2, 2), ("neighbor.ts", 1, 1)]);
}
#[test]
fn issue_162_bootstrap_inline_controls_stale_contributor_token_api_package_config_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for (file, replacement, left, right) in [
        ("group.ts", "export const GROUP=[];", 0, 0),
        ("left.ts", "export interface Token {}", 0, 1),
        (
            "api.ts",
            "export {bootstrapApplication as start} from './fake';",
            0,
            0,
        ),
        (
            "node_modules/@angular/platform-browser/package.json",
            r#"{"name":"@angular/platform-browser","version":"20.0.0","exports":{".":null}}"#,
            0,
            0,
        ),
        (
            "tsconfig.json",
            r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"paths":{"@angular/platform-browser":["./fake.ts"]}}}"#,
            0,
            0,
        ),
    ] {
        let (root, config) = setup(CALL);
        let p = root.path();
        let state = crate::mcp::McpState::new(config.clone());
        capture(&state, p, "main.ts");
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("main.ts")}),
        );
        drop(state);
        let original = std::fs::read(p.join(file)).unwrap();
        let caller = std::fs::read(p.join("main.ts")).unwrap();
        std::fs::write(p.join(file), replacement).unwrap();
        let state = crate::mcp::McpState::new(config);
        let rejected = raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("main.ts")}),
        );
        assert!(
            rejected.get("error").is_some() || rejected["result"]["isError"] == true,
            "{rejected}"
        );
        facts(
            &state,
            p,
            &[("main.ts", left, right), ("neighbor.ts", left, right)],
        );
        std::fs::write(p.join(file), original).unwrap();
        facts(&state, p, &[("main.ts", 1, 1), ("neighbor.ts", 1, 1)]);
        assert_eq!(std::fs::read(p.join("main.ts")).unwrap(), caller);
    }
}
#[test]
fn issue_162_bootstrap_inline_controls_entry_field_call_source_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(&CALL.repeat(2));
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    facts(&state, p, &[("main.ts", 2, 2), ("neighbor.ts", 1, 1)]);
    for (body, left, right) in [
        (CALL, 1, 1),
        (
            "import {Token} from './left'; launch(Root,{providers:[Token]});",
            1,
            0,
        ),
        ("launch(Root,{other:1});", 0, 0),
        ("export class Replaced {}", 0, 0),
    ] {
        std::fs::write(p.join("main.ts"), format!("{PREFIX}{body}")).unwrap();
        facts(
            &state,
            p,
            &[("main.ts", left, right), ("neighbor.ts", 1, 1)],
        );
    }
    std::fs::remove_file(p.join("main.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &[("neighbor.ts", 1, 1)]);
}
