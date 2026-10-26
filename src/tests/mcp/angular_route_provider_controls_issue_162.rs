use crate::layers::meta::semantic::SemanticRelation;

#[path = "angular_route_provider_exit_issue_162.rs"]
mod exit_cases;
use serde_json::{Value, json};
use std::path::Path;

const PREFIX: &str = "import type {Routes as NativeRoutes} from '@angular/router'; import {Token as Left} from './left'; import {Token as Right} from './right'; import {GROUP} from './group';";
const GROUP: &str = "import {Token as Left} from './left'; import {Token as Right} from './right'; export const GROUP=[Left,{provide:Right,useValue:1}];";
const MANIFEST: &str = r#"{"name":"@angular/router","version":"20.0.0","types":"index.d.ts"}"#;

fn setup(source: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    let router = p.join("node_modules/@angular/router");
    std::fs::create_dir_all(&router).unwrap();
    std::fs::write(router.join("package.json"), MANIFEST).unwrap();
    std::fs::write(router.join("index.d.ts"), "export type Routes=unknown[];").unwrap();
    for file in ["left.ts", "right.ts"] {
        std::fs::write(p.join(file), "export class Token {}").unwrap();
    }
    std::fs::write(p.join("group.ts"), GROUP).unwrap();
    std::fs::write(p.join("routes.ts"), format!("{PREFIX}{source}")).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("authority.db").to_string_lossy().into_owned();
    (root, config)
}

fn raw(state: &crate::mcp::McpState, p: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(p);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}
fn call(state: &crate::mcp::McpState, p: &Path, tool: &str, args: Value) -> Value {
    let response = raw(state, p, tool, args);
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");
    response["result"]["structuredContent"].clone()
}
fn capture(state: &crate::mcp::McpState, p: &Path) {
    call(
        state,
        p,
        "compress_code_context",
        json!({"filePath":p.join("routes.ts"),"fidelity":"high"}),
    );
}
fn answers(state: &crate::mcp::McpState, p: &Path, name: &str, count: usize) {
    let result = call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}),
    );
    let edges: Vec<_> = result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "RegistersProvider")
        .collect();
    assert_eq!(edges.len(), count, "{result}");
    for edge in edges {
        assert_eq!(edge["subject"]["file"], json!(p.join("routes.ts")));
        assert!(edge["subject"].get("canonical").is_none());
        assert!(edge["object"].get("canonical").is_none());
        assert_eq!(edge["object"]["name"], "Token");
    }
}
fn authority(
    state: &crate::mcp::McpState,
    p: &Path,
) -> crate::ir::typescript_resolution::TypeScriptAuthority {
    let alias = state
        .alias_for_path(&p.join("routes.ts").to_string_lossy())
        .unwrap();
    state
        .semantic_snapshot(&alias)
        .unwrap()
        .authority
        .typescript_authority
        .as_ref()
        .unwrap()
        .clone()
}

#[test]
fn issue_162_route_provider_native_root_children_same_path_and_refusals() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "export const routes:NativeRoutes=[{path:'same',providers:[Left,Left],children:[{path:'child',providers:[Right]}]},{path:'same',providers:[Left]}]; export const other:NativeRoutes=[{path:'same',providers:[Right]}];";
    let (root, config) = setup(source);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    capture(&state, p);
    answers(&state, p, "same", 3);
    answers(&state, p, "same/child", 1);
    for replacement in [
        source.replace(":NativeRoutes", ""),
        source
            .replace("const routes", "let routes")
            .replace("const other", "let other"),
        source.replace(":NativeRoutes", ":FakeRoutes") + " type FakeRoutes=unknown[];",
        "const unrelated={path:'same',providers:[Left]};".into(),
    ] {
        std::fs::write(p.join("routes.ts"), format!("{PREFIX}{replacement}")).unwrap();
        answers(&state, p, "same", 0);
    }
    std::fs::write(p.join("routes.ts"), format!("{PREFIX}{source}")).unwrap();
    answers(&state, p, "same", 3);
    std::fs::write(
        p.join("node_modules/@angular/router/package.json"),
        MANIFEST.replace("@angular/router", "foreign-router"),
    )
    .unwrap();
    answers(&state, p, "same", 0);
    std::fs::write(
        p.join("node_modules/@angular/router/package.json"),
        MANIFEST,
    )
    .unwrap();
    answers(&state, p, "same", 3);
}

#[test]
fn issue_162_route_provider_proof_forgery_serde_and_shared_strategy_policies() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "function make(){return {}} const cycle=[...cycle,Right]; export const routes:NativeRoutes=[{path:'same',providers:[...GROUP,...cycle,{provide:'str',useClass:Left},{provide:Left,useFactory:make},{provide:Right,useExisting:Left},{provide:Left,useValue:Right}]}];";
    let (root, config) = setup(source);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    capture(&state, p);
    let proof = authority(&state, p);
    let edges = crate::angular_meta::route_providers::project(&proof);
    let registrations: Vec<_> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::RegistersProvider)
        .cloned()
        .collect();
    assert!(registrations.len() >= 7, "{edges:?}");
    assert!(proof.admits_relation_evidence(&edges));
    for relation in [
        SemanticRelation::ProviderUsesClass,
        SemanticRelation::ProviderUsesFactory,
        SemanticRelation::ProviderUsesExisting,
    ] {
        assert!(edges.iter().any(|e| e.relation == relation), "{edges:?}");
    }
    let roundtrip = serde_json::from_value(serde_json::to_value(&proof).unwrap()).unwrap();
    let replay = crate::angular_meta::route_providers::project(&roundtrip);
    assert_eq!(replay.len(), edges.len());
    for mutation in 0..7 {
        let mut bad = proof.clone();
        match mutation {
            0 => bad.type_reference_sites.clear(),
            1 => bad.package_exports.clear(),
            2 => bad.demanded_occurrences.clear(),
            3 => bad.source_hash.push('x'),
            4 => {
                let f = bad
                    .declaration_facets
                    .iter_mut()
                    .find(|f| f.declaration.name == "routes")
                    .unwrap();
                f.type_annotation = None;
            }
            5 => {
                let duplicate = bad
                    .declaration_facets
                    .iter()
                    .find(|f| f.declaration.name == "routes")
                    .unwrap()
                    .clone();
                bad.declaration_facets.push(duplicate);
            }
            _ => {
                let start = bad
                    .declaration_facets
                    .iter()
                    .find(|f| f.declaration.name == "routes")
                    .unwrap()
                    .initializer
                    .as_ref()
                    .unwrap()
                    .start;
                let site = bad
                    .demanded_occurrences
                    .iter_mut()
                    .find(|s| s.start == start && s.source == bad.source)
                    .unwrap();
                site.start += 1;
            }
        }
        assert!(
            !bad.admits_relation_evidence(&registrations),
            "mutation={mutation}"
        );
    }
    let mut forged = registrations[0].clone();
    if let Some(crate::layers::meta::canonical_endpoint::CanonicalEndpoint::RouteOccurrence {
        route_occurrence,
    }) = &mut forged.subject.canonical
    {
        route_occurrence.start += 1;
    }
    assert!(!proof.admits_relation_evidence(&[forged]));
}

#[test]
fn issue_162_route_provider_complete_sqlite_restore_same_path_empty_path_and_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "export const routes:NativeRoutes=[{path:'same',providers:GROUP},{path:'same',providers:GROUP},{path:'',providers:[Left]},{path:'',providers:[Left]}];";
    let (root, config) = setup(source);
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    capture(&state, p);
    let expected = serde_json::to_value(authority(&state, p)).unwrap();
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("routes.ts")}),
    );
    drop(state);
    let db = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let encoded: String = db
        .query_row(
            "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path=?1",
            [p.join("routes.ts").to_string_lossy().into_owned()],
            |r| r.get(0),
        )
        .unwrap();
    let durable: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(durable["typescript_authority"], expected);
    assert!(
        durable["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["subject"]["canonical"].get("route_occurrence").is_some())
    );
    drop(db);
    let state = crate::mcp::McpState::new(config);
    call(
        &state,
        p,
        "restore_context",
        json!({"filePath":p.join("routes.ts")}),
    );
    assert_eq!(
        serde_json::to_value(authority(&state, p)).unwrap(),
        expected,
        "before queries"
    );
    let owner = state.semantic_owner_path(&p.join("routes.ts").to_string_lossy());
    let index = state.workspace_index_read();
    assert_eq!(
        serde_json::to_value(index.retained_typescript_authority(&owner).unwrap()).unwrap(),
        expected
    );
    let same = index.forward_edges_by_identity("angular", "Route", "same");
    assert_eq!(
        same.iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .count(),
        4
    );
    let empty = index.forward_edges_by_identity("angular", "Route", "");
    assert_eq!(
        empty
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .count(),
        2
    );
    drop(index);
    answers(&state, p, "same", 4);
    let reverse = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":"ProviderToken","name":"Token"}),
    );
    assert_eq!(
        reverse["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "RegistersProvider" && e["subject"]["name"] == "")
            .count(),
        2,
        "empty-path occurrences remain externally visible"
    );
    let reduced = source.replacen("{path:'same',providers:GROUP},", "", 1);
    std::fs::write(p.join("routes.ts"), format!("{PREFIX}{reduced}")).unwrap();
    answers(&state, p, "same", 2);
    std::fs::write(p.join("routes.ts"), format!("{PREFIX}{source}")).unwrap();
    answers(&state, p, "same", 4);
    std::fs::write(
        p.join("routes.ts"),
        format!("{PREFIX}export const routes:NativeRoutes=[];"),
    )
    .unwrap();
    answers(&state, p, "same", 0);
}

#[test]
fn issue_162_route_provider_stale_contributor_token_package_refusal_and_unchanged_owner_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for (file, replacement, count) in [
        ("group.ts", GROUP.replace("const GROUP", "let GROUP"), 0),
        ("right.ts", "export interface Token {}".into(), 1),
        (
            "node_modules/@angular/router/package.json",
            MANIFEST.replace("20.0.0", "20.0.1"),
            2,
        ),
    ] {
        let source = "export const routes:NativeRoutes=[{path:'same',providers:GROUP}];";
        let (root, config) = setup(source);
        let p = root.path();
        let state = crate::mcp::McpState::new(config.clone());
        capture(&state, p);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("routes.ts")}),
        );
        drop(state);
        let original = std::fs::read(p.join(file)).unwrap();
        std::fs::write(p.join(file), replacement).unwrap();
        let state = crate::mcp::McpState::new(config);
        let response = raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("routes.ts")}),
        );
        assert!(
            response.get("error").is_some() || response["result"]["isError"] == true,
            "{response}"
        );
        answers(&state, p, "same", count);
        std::fs::write(p.join(file), original).unwrap();
        answers(&state, p, "same", 2);
        assert_eq!(
            std::fs::read_to_string(p.join("routes.ts")).unwrap(),
            format!("{PREFIX}{source}")
        );
    }
}
