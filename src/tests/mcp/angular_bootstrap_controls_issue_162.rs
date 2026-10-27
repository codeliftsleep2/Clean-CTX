//! Integrated bootstrap structural proof, independent arguments and lifecycle.
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::path::Path;

const PREFIX: &str = "import {bootstrapApplication as launch} from './api'; import {Root} from './root'; import {appConfig} from './config';";
const CALL: &str = "launch(Root,appConfig);";
const ROOT: &str = "import {Component} from '@angular/core'; @Component({}) export class Root {}";
const CONFIG: &str = "import type {ApplicationConfig} from '@angular/core'; export const appConfig:ApplicationConfig={providers:[]};";
const MANIFEST: &str =
    r#"{"name":"@angular/platform-browser","version":"20.0.0","types":"index.d.ts"}"#;
fn setup(body: &str, neighbor: bool) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"),"export declare function Component(metadata:unknown):any; export interface ApplicationConfig {providers:unknown[];}").unwrap();
    let platform = p.join("node_modules/@angular/platform-browser");
    std::fs::create_dir_all(&platform).unwrap();
    std::fs::write(platform.join("package.json"), MANIFEST).unwrap();
    std::fs::write(
        platform.join("index.d.ts"),
        "export declare function bootstrapApplication(root:unknown,config?:unknown):unknown;",
    )
    .unwrap();
    for (file, source) in [
        ("root.ts", ROOT),
        ("config.ts", CONFIG),
        (
            "api.ts",
            "export {bootstrapApplication} from '@angular/platform-browser';",
        ),
        (
            "fake.ts",
            "export function bootstrapApplication(root:unknown,config?:unknown){}",
        ),
        ("other.ts", "export class Root {}"),
        (
            "fake-config.ts",
            "export interface ApplicationConfig {providers:unknown[];}",
        ),
    ] {
        std::fs::write(p.join(file), source).unwrap();
    }
    std::fs::write(p.join("main.ts"), format!("{PREFIX}{body}")).unwrap();
    if neighbor {
        std::fs::write(p.join("neighbor.ts"), format!("{PREFIX}{CALL}")).unwrap();
    }
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("proof.db").to_string_lossy().into_owned();
    (root, config)
}
fn raw(state: &crate::mcp::McpState, p: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(p);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}
fn call(state: &crate::mcp::McpState, p: &Path, tool: &str, args: Value) -> Value {
    let r = raw(state, p, tool, args);
    assert!(r.get("error").is_none(), "{r}");
    assert_ne!(r["result"]["isError"], true, "{r}");
    r["result"]["structuredContent"].clone()
}
fn capture(state: &crate::mcp::McpState, p: &Path, file: &str) -> TypeScriptAuthority {
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
fn facts(state: &crate::mcp::McpState, p: &Path, expected: &[(&str, usize, usize)]) {
    let answer = call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapApplication"}),
    );
    let mut actual = Vec::new();
    for edge in answer["edges"].as_array().unwrap() {
        if !matches!(
            edge["relation"].as_str(),
            Some("DeclaresBootstrapRoot" | "UsesApplicationConfig")
        ) {
            continue;
        }
        assert!(
            edge["subject"].get("canonical").is_none() && edge["object"].get("canonical").is_none()
        );
        actual.push((
            crate::dictionary::path::canonical_identity_key(
                edge["subject"]["file"].as_str().unwrap(),
            ),
            edge["relation"].as_str().unwrap().to_owned(),
        ));
    }
    let mut wanted = Vec::new();
    for (file, roots, configs) in expected {
        for (count, relation) in [
            (*roots, "DeclaresBootstrapRoot"),
            (*configs, "UsesApplicationConfig"),
        ] {
            for _ in 0..count {
                wanted.push((
                    crate::dictionary::path::canonical_identity_key(
                        &p.join(file).to_string_lossy(),
                    ),
                    relation.to_owned(),
                ));
            }
        }
    }
    actual.sort();
    wanted.sort();
    assert_eq!(actual, wanted, "{answer}");
}
#[test]
fn issue_162_bootstrap_controls_independent_local_scoped_and_unsupported_arguments() {
    let _serial = crate::protocol::handler_response_serial();
    let body = "import type {ApplicationConfig as C} from '@angular/core'; import {Root as Other} from './other'; const local:C={providers:[]}; launch(Root,local); if(true){launch(Root,appConfig);} function run(){launch(Root,appConfig);} launch(Root,{}); launch(Other,appConfig); launch(Root,...[appConfig]); launch(...[Root],appConfig);";
    let (root, config) = setup(body, false);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, p, "main.ts");
    let edges = crate::angular_meta::bootstrap_references::project(&authority);
    assert!(authority.admits_relation_evidence(&edges));
    assert_eq!(
        edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
            .count(),
        5,
        "{edges:?}"
    );
    assert_eq!(
        edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::UsesApplicationConfig)
            .count(),
        4,
        "{edges:?}"
    );
    facts(&state, p, &[("main.ts", 5, 4)]);
}
#[test]
fn issue_162_bootstrap_controls_missing_crossed_duplicate_and_endpoint_refusal() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(&CALL.repeat(2), false);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, p, "main.ts");
    let edges = crate::angular_meta::bootstrap_references::project(&authority);
    assert_eq!(edges.len(), 4);
    assert!(authority.admits_relation_evidence(&edges));
    for mode in 0..17 {
        let mut bad = authority.clone();
        match mode {
            0=>bad.invocation_facets.clear(),
            1=>bad.demanded_occurrences.clear(),
            2=>bad.reference_sites.clear(),
            3=>bad.package_exports.clear(),
            4=>bad.invocation_facets[0].source_hash.push('x'),
            5=>bad.invocation_facets[0].source.push("crossed.ts"),
            6=>bad.invocation_facets[0].expression.start+=1,
            7=>bad.invocation_facets[0].expression.end-=1,
            8=>{let duplicate=bad.invocation_facets[0].clone();bad.invocation_facets.push(duplicate);},
            9=>bad.declaration_facets.clear(),
            10=>bad.demanded_occurrences.retain(|v|!matches!(v.value,crate::ir::typescript_resolution::demanded_occurrences::DemandedValue::TypeReference(_))),
            _=>{
                let ExpressionValue::Call(callee,args)=&mut bad.invocation_facets[0].expression.value else {panic!()};
                match mode {
                    11=>callee.start+=1,
                    12=>args.swap(0,1),
                    13=>args[0].start+=1,
                    14=>{let crossed=args[1].value.clone();args[0].value=crossed;},
                    15=>{let crossed=args[0].value.clone();args[1].value=crossed;},
                    _=>args.clear(),
                }
            }
        }
        assert!(!bad.admits_relation_evidence(&edges), "forged mode={mode}");
    }
    let restored: TypeScriptAuthority =
        serde_json::from_value(serde_json::to_value(&authority).unwrap()).unwrap();
    assert!(restored.admits_relation_evidence(&edges));
    for mode in 0..5 {
        let mut forged = edges[0].clone();
        match mode {
            0 => forged.subject.canonical = None,
            1 => forged.subject.entity_type = "Component",
            2 => forged.object.canonical = edges[1].object.canonical.clone(),
            3 => forged.relation = SemanticRelation::RegistersProvider,
            _ => forged.subject.canonical = edges[2].subject.canonical.clone(),
        }
        if mode == 4 {
            // Same target from the other real call is a valid independent association.
            assert!(authority.admits_relation_evidence(&[forged]));
        } else {
            assert!(
                !authority.admits_relation_evidence(&[forged]),
                "endpoint mode={mode}"
            );
        }
    }
    let mut historical = serde_json::to_value(&authority).unwrap();
    historical
        .as_object_mut()
        .unwrap()
        .remove("invocation_facets");
    let historical: TypeScriptAuthority = serde_json::from_value(historical).unwrap();
    assert!(!historical.admits_relation_evidence(&edges));
}
#[test]
fn issue_162_bootstrap_controls_complete_sqlite_index_restore_and_multiplicity() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(&CALL.repeat(2), true);
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let mut expected = Vec::new();
    for file in ["main.ts", "neighbor.ts"] {
        capture(&state, p, file);
        let alias = state
            .alias_for_path(&p.join(file).to_string_lossy())
            .unwrap();
        let snapshot = state.semantic_snapshot(&alias).unwrap();
        let durable: Vec<_> = snapshot
            .edges
            .iter()
            .map(crate::mcp::state::durable_semantics::DurableSemanticEdge::from)
            .collect();
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
                |row| row.get(0),
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
        let index = state.workspace_index_read();
        let proof = index
            .retained_typescript_authority(&owner)
            .expect("before-query retained index proof");
        assert_eq!(serde_json::to_value(proof).unwrap(), wanted["proof"]);
    }
    facts(&state, p, &[("main.ts", 2, 2), ("neighbor.ts", 1, 1)]);
}
#[test]
fn issue_162_bootstrap_controls_unchanged_caller_dependency_mutation_and_stale_restore() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..6 {
        let (root, config) = setup(CALL, false);
        let p = root.path();
        let state = crate::mcp::McpState::new(config.clone());
        capture(&state, p, "main.ts");
        facts(&state, p, &[("main.ts", 1, 1)]);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("main.ts")}),
        );
        drop(state);
        let caller = std::fs::read(p.join("main.ts")).unwrap();
        let (file, replacement, roots, configs) = match mode {
            0 => ("root.ts", "export class Root {}", 0, 1),
            1 => ("config.ts", "export const appConfig={providers:[]};", 1, 0),
            2 => (
                "api.ts",
                "export {bootstrapApplication} from './fake';",
                0,
                0,
            ),
            3 => (
                "node_modules/@angular/platform-browser/package.json",
                r#"{"name":"@angular/platform-browser","version":"20.0.0","exports":{".":null}}"#,
                0,
                0,
            ),
            4 => (
                "tsconfig.json",
                r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"paths":{"@angular/platform-browser":["./fake.ts"]}}}"#,
                0,
                0,
            ),
            _ => (
                "node_modules/@angular/core/index.d.ts",
                "export interface Component {} export interface ApplicationConfig {providers:unknown[];}",
                0,
                1,
            ),
        };
        let original = std::fs::read(p.join(file)).unwrap();
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
            "stale restore before query mode={mode}: {rejected}"
        );
        assert!(
            state
                .alias_for_path(&p.join("main.ts").to_string_lossy())
                .is_none()
        );
        facts(&state, p, &[("main.ts", roots, configs)]);
        std::fs::write(p.join(file), original).unwrap();
        facts(&state, p, &[("main.ts", 1, 1)]);
        assert_eq!(std::fs::read(p.join("main.ts")).unwrap(), caller);
    }
}
#[test]
fn issue_162_bootstrap_controls_independent_argument_call_and_source_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(&CALL.repeat(2), true);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    capture(&state, p, "main.ts");
    capture(&state, p, "neighbor.ts");
    facts(&state, p, &[("main.ts", 2, 2), ("neighbor.ts", 1, 1)]);
    for (body, roots, configs) in [
        (CALL.to_owned(), 1, 1),
        (format!("{CALL}launch(Root);"), 2, 1),
        (format!("{CALL}launch(undefined,appConfig);"), 1, 2),
        ("export class Replaced {}".into(), 0, 0),
    ] {
        std::fs::write(p.join("main.ts"), format!("{PREFIX}{body}")).unwrap();
        facts(
            &state,
            p,
            &[("main.ts", roots, configs), ("neighbor.ts", 1, 1)],
        );
    }
    std::fs::write(p.join("main.ts"), format!("{PREFIX}{CALL}")).unwrap();
    capture(&state, p, "main.ts");
    facts(&state, p, &[("main.ts", 1, 1), ("neighbor.ts", 1, 1)]);
    std::fs::remove_file(p.join("main.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &[("neighbor.ts", 1, 1)]);
}

#[test]
fn issue_162_bootstrap_controls_exact_public_targets_and_independent_replacement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(&CALL.repeat(2), true);
    let p = root.path();
    std::fs::write(p.join("other-root.ts"), ROOT).unwrap();
    std::fs::write(p.join("other-config.ts"), CONFIG).unwrap();
    let state = crate::mcp::McpState::new(config);
    capture(&state, p, "main.ts");
    capture(&state, p, "neighbor.ts");
    let identity = |file: &str| {
        crate::dictionary::path::canonical_identity_key(&p.join(file).to_string_lossy())
    };
    for (body, root_file, config_file) in [
        (CALL.repeat(2), "root.ts", "config.ts"),
        ("import {Root as Next} from './other-root'; launch(Next,appConfig); launch(Root,appConfig);".into(), "other-root.ts", "config.ts"),
        ("import {Root as Next} from './other-root'; import {appConfig as NextConfig} from './other-config'; launch(Next,NextConfig); launch(Root,appConfig);".into(), "other-root.ts", "other-config.ts"),
    ] {
        std::fs::write(p.join("main.ts"), format!("{PREFIX}{body}")).unwrap();
        let mut wanted = Vec::new();
        for (owner, target_root, target_config) in [
            ("main.ts", root_file, config_file),
            ("main.ts", "root.ts", "config.ts"),
            ("neighbor.ts", "root.ts", "config.ts"),
        ] {
            wanted.push((identity(owner), "DeclaresBootstrapRoot".to_owned(), identity(target_root)));
            wanted.push((identity(owner), "UsesApplicationConfig".to_owned(), identity(target_config)));
        }
        wanted.sort();
        for _ in 0..2 {
            for reverse in [false, true] {
                let mut actual = Vec::new();
                let selectors = if reverse {
                    vec![("Component", "Root"), ("ApplicationConfig", "appConfig")]
                } else {
                    vec![("ApplicationBootstrap", "bootstrapApplication")]
                };
                for (kind, name) in selectors {
                    let answer = call(&state, p, "workspace_query", json!({
                        "type": if reverse {"reverse_edges"} else {"forward_edges"},
                        "domain":"angular", "entity_type":kind, "name":name
                    }));
                    for edge in answer["edges"].as_array().unwrap() {
                        let Some(relation @ ("DeclaresBootstrapRoot" | "UsesApplicationConfig")) = edge["relation"].as_str() else { continue };
                        assert_eq!(edge["subject"]["entity_type"], "ApplicationBootstrap");
                        assert_eq!(edge["subject"]["name"], "bootstrapApplication");
                        assert_eq!(edge["object"]["entity_type"], if relation == "DeclaresBootstrapRoot" {"Component"} else {"ApplicationConfig"});
                        assert!(edge["subject"].get("canonical").is_none());
                        assert!(edge["object"].get("canonical").is_none());
                        actual.push((
                            crate::dictionary::path::canonical_identity_key(edge["subject"]["file"].as_str().unwrap()),
                            relation.to_owned(),
                            crate::dictionary::path::canonical_identity_key(edge["object"]["file"].as_str().unwrap()),
                        ));
                    }
                }
                actual.sort();
                assert_eq!(actual, wanted, "reverse={reverse}");
            }
        }
    }
}

#[test]
fn issue_162_bootstrap_controls_refresh_rediscovers_unchanged_barrel_caller() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(CALL, true);
    let p = root.path();
    // Match the binary route: the caller itself has no native API spelling.
    std::fs::write(
        p.join("api.ts"),
        "export {bootstrapApplication,bootstrapApplication as start} from '@angular/platform-browser';",
    ).unwrap();
    std::fs::write(
        p.join("neighbor.ts"),
        format!(
            "{}{CALL}",
            PREFIX.replace("bootstrapApplication as launch", "start as launch")
        ),
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config);
    for file in ["main.ts", "neighbor.ts"] {
        let authority = capture(&state, p, file);
        let edges = crate::angular_meta::bootstrap_references::project(&authority);
        assert_eq!(edges.len(), 2, "native call prerequisites: {file}");
        assert!(authority.admits_relation_evidence(&edges));
    }
    facts(&state, p, &[("main.ts", 1, 1), ("neighbor.ts", 1, 1)]);
    let neighbor = std::fs::read(p.join("neighbor.ts")).unwrap();
    std::fs::remove_file(p.join("main.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    assert_eq!(std::fs::read(p.join("neighbor.ts")).unwrap(), neighbor);
    // First public query after refresh must rediscover the proved barrel caller.
    // No explicit capture or reverse target query may rescue this assertion.
    facts(&state, p, &[("neighbor.ts", 1, 1)]);
}

#[test]
fn issue_162_bootstrap_controls_cold_discovery_routes_and_query_boundary() {
    let _serial = crate::protocol::handler_response_serial();
    for (import, callee) in [
        (
            "import {bootstrapApplication} from '@angular/platform-browser';",
            "bootstrapApplication",
        ),
        (
            "import {bootstrapApplication as launch} from '@angular/platform-browser';",
            "launch",
        ),
        ("import {start as launch} from './api';", "launch"),
        (
            "import * as platform from '@angular/platform-browser';",
            "platform.bootstrapApplication",
        ),
    ] {
        let (root, config) = setup(CALL, false);
        let p = root.path();
        std::fs::write(p.join("api.ts"), "export {bootstrapApplication,bootstrapApplication as start} from '@angular/platform-browser';").unwrap();
        std::fs::write(p.join("neighbor.ts"), format!("{import} import {{Root}} from './root'; import {{appConfig}} from './config'; {callee}(Root,appConfig);")).unwrap();
        let state = crate::mcp::McpState::new(config);
        call(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"AbsentUnrelatedName"}),
        );
        assert!(
            state
                .alias_for_path(&p.join("neighbor.ts").to_string_lossy())
                .is_none(),
            "generic query must not acquire all TS owners"
        );
        // No source capture precedes the first bootstrap answer.
        facts(&state, p, &[("main.ts", 1, 1), ("neighbor.ts", 1, 1)]);
        std::fs::remove_file(p.join("main.ts")).unwrap();
        call(&state, p, "refresh_workspace", json!({}));
        facts(&state, p, &[("neighbor.ts", 1, 1)]);
        let answer = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapApplication"}),
        );
        assert_eq!(answer["coverage"]["result_semantics"], "lower_bound");
        assert_eq!(answer["coverage"]["source_complete"], false);
    }
}
