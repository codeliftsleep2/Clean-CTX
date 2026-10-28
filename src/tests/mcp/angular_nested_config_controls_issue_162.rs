//! Direct merged-input role controls, never recursive configuration evaluation.
use super::angular_bootstrap_inline_controls_tests::{call, capture, raw};
use super::angular_config_composition_controls_tests::{NATIVE, setup as base_setup};
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use serde_json::{Value, json};
use std::path::Path;
const INNER: &str = "export const inner:ApplicationConfig=combine(Base,Feature);";
fn outer(args: &str) -> String {
    format!(
        "{NATIVE} import {{inner as Selected}} from './inner'; export const outer:ApplicationConfig=combine({args});"
    )
}
fn setup(args: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = base_setup("Base,Feature");
    let p = root.path();
    std::fs::write(p.join("inner.ts"), format!("{NATIVE}{INNER}")).unwrap();
    std::fs::write(p.join("outer.ts"), outer(args)).unwrap();
    std::fs::write(p.join("nested-caller.ts"),"import {bootstrapApplication} from '@angular/platform-browser'; import {Root} from './root'; import {outer} from './outer'; bootstrapApplication(Root,outer);").unwrap();
    (root, config)
}
fn facts(state: &crate::mcp::McpState, p: &Path, inner: bool, base: bool) {
    let identity = |file: &str| {
        crate::dictionary::path::canonical_identity_key(&p.join(file).to_string_lossy())
    };
    let mut wanted = Vec::new();
    if inner {
        wanted.push((identity("outer.ts"), identity("inner.ts")));
    }
    if base {
        wanted.push((identity("outer.ts"), identity("base.ts")));
    }
    wanted.sort();
    let forward = call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":"outer"}),
    );
    let mut actual = forward["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "IncludesApplicationConfig")
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
    assert_eq!(actual, wanted, "{forward}");
    assert_eq!(forward["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(forward["coverage"]["source_complete"], false);
    let reverse = call(
        state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":"ApplicationConfig","name":"inner"}),
    );
    let selected = reverse["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "IncludesApplicationConfig" && e["subject"]["name"] == "outer")
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), usize::from(inner), "{reverse}");
    assert!(
        !forward["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["relation"] == "RegistersProvider"),
        "no provider inheritance"
    );
}
#[test]
fn issue_162_nested_config_controls_missing_borrowed_and_crossed_inner_role() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("Selected,Base");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, p, "outer.ts");
    let edges = crate::angular_meta::config_composition::project(&authority);
    assert_eq!(edges.len(), 2);
    assert!(edges.iter().all(|e| e.subject.name == "outer"));
    assert!(authority.admits_relation_evidence(&edges));
    let inner = authority
        .declaration_facets
        .iter()
        .position(|f| f.declaration.name == "inner")
        .unwrap();
    let owner = authority
        .declaration_facets
        .iter()
        .position(|f| f.declaration.name == "outer")
        .unwrap();
    for mode in 0..9 {
        let mut bad = authority.clone();
        match mode {
            0 => bad.declaration_facets[inner].initializer = None,
            1 => bad.declaration_facets[inner].type_annotation = None,
            2 => bad.declaration_facets[inner]
                .declaration
                .source_hash
                .push('x'),
            3 => {
                let duplicate = bad.declaration_facets[inner].clone();
                bad.declaration_facets.push(duplicate);
            }
            4 => {
                bad.declaration_facets[inner]
                    .initializer
                    .as_mut()
                    .unwrap()
                    .start += 1
            }
            _ => {
                let ExpressionValue::Call(callee, args) = &mut bad.declaration_facets[owner]
                    .initializer
                    .as_mut()
                    .unwrap()
                    .value
                else {
                    panic!()
                };
                match mode {
                    5 => callee.start += 1,
                    6 => args[0].start += 1,
                    7 => args[0].value = args[1].value.clone(),
                    _ => args.swap(0, 1),
                }
            }
        }
        assert!(!bad.admits_relation_evidence(&edges), "mode={mode}");
    }
    let restored: crate::ir::typescript_resolution::TypeScriptAuthority =
        serde_json::from_value(serde_json::to_value(&authority).unwrap()).unwrap();
    assert!(restored.admits_relation_evidence(&edges));
}
#[test]
fn issue_162_nested_config_controls_inner_refusals_and_valid_siblings() {
    let _serial = crate::protocol::handler_response_serial();
    for (declaration, initializer) in [
        ("const inner", "combine(Base,Feature)"),
        ("let inner:ApplicationConfig", "combine(Base,Feature)"),
        ("const inner:ApplicationConfig", "foreign(Base)"),
        ("const inner:ApplicationConfig", "Missing(Base)"),
    ] {
        let (root, config) = setup("Selected,Base");
        let p = root.path();
        std::fs::write(p.join("inner.ts"),format!("{NATIVE} import {{mergeApplicationConfig as foreign}} from './fake-merge'; export {declaration}={initializer};")).unwrap();
        let state = crate::mcp::McpState::new(config);
        facts(&state, p, false, true);
    }
    let (root, config) = setup("Selected,...[Selected],{},Alias,combine(Base),Base");
    let p = root.path();
    let text = std::fs::read_to_string(p.join("outer.ts")).unwrap();
    std::fs::write(
        p.join("outer.ts"),
        text.replace(
            "export const outer",
            "const Alias=Selected; export const outer",
        ),
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config);
    facts(&state, p, true, true);
}
#[test]
fn issue_162_nested_config_controls_sqlite_before_queries_and_owned_assertion() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("Selected,Selected,Base");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let mut expected = Vec::new();
    for file in ["inner.ts", "outer.ts"] {
        let proof = capture(&state, p, file);
        let own = crate::angular_meta::config_composition::project(&proof);
        assert!(own.iter().all(|e|e.subject.file.as_deref()==Some(proof.source.to_string_lossy().as_ref())),"imported role proof must not republish inner edges");
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
    for (file, wanted) in ["inner.ts", "outer.ts"].into_iter().zip(&expected) {
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
    for (file, wanted) in ["inner.ts", "outer.ts"].into_iter().zip(expected) {
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
    facts(&state, p, true, true);
}
#[test]
fn issue_162_nested_config_controls_stale_inner_api_package_and_unchanged_outer_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..4 {
        let (root, config) = setup("Selected,Base");
        let p = root.path();
        std::fs::write(
            p.join("inner-api.ts"),
            "export {mergeApplicationConfig as combine} from '@angular/core';",
        )
        .unwrap();
        let inner = format!(
            "{} {INNER}",
            NATIVE.replace(
                "import {mergeApplicationConfig as combine} from '@angular/core';",
                "import {combine} from './inner-api';"
            )
        );
        std::fs::write(p.join("inner.ts"), inner).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        capture(&state, p, "outer.ts");
        facts(&state, p, true, true);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("outer.ts")}),
        );
        drop(state);
        let (file, replacement, base) = match mode {
            0 => ("inner.ts", "export const inner={providers:[]};", true),
            1 => (
                "inner-api.ts",
                "export {mergeApplicationConfig as combine} from './fake-merge';",
                true,
            ),
            2 => (
                "node_modules/@angular/core/package.json",
                r#"{"name":"@angular/core","version":"20.0.0","exports":{".":null}}"#,
                false,
            ),
            _ => (
                "tsconfig.json",
                r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"paths":{"@angular/core":["./fake-merge.ts"]}}}"#,
                false,
            ),
        };
        let original = std::fs::read(p.join(file)).unwrap();
        let outer = std::fs::read(p.join("outer.ts")).unwrap();
        std::fs::write(p.join(file), replacement).unwrap();
        let state = crate::mcp::McpState::new(config);
        let rejected = raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("outer.ts")}),
        );
        assert!(
            rejected.get("error").is_some() || rejected["result"]["isError"] == true,
            "{rejected}"
        );
        facts(&state, p, false, base);
        std::fs::write(p.join(file), original).unwrap();
        facts(&state, p, true, true);
        assert_eq!(std::fs::read(p.join("outer.ts")).unwrap(), outer);
    }
}
#[test]
fn issue_162_nested_config_controls_support_and_independent_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("Selected,Selected,Base");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    facts(&state, p, true, true);
    for args in ["Selected,Base", "Base"] {
        std::fs::write(p.join("outer.ts"), outer(args)).unwrap();
        facts(&state, p, args.starts_with("Selected"), true);
    }
    std::fs::write(p.join("outer.ts"), outer("Selected,Base")).unwrap();
    facts(&state, p, true, true);
    std::fs::remove_file(p.join("nested-caller.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, true, true);
    let inner = std::fs::read(p.join("inner.ts")).unwrap();
    std::fs::remove_file(p.join("inner.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, false, true);
    std::fs::write(p.join("inner.ts"), inner).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, true, true);
    std::fs::write(p.join("outer.ts"), "export const Replaced=1;").unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, false, false);
}

#[test]
fn issue_162_nested_config_controls_cycle_cold_warm_without_bootstrap() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("Selected,Base");
    let p = root.path();
    std::fs::remove_file(p.join("nested-caller.ts")).unwrap();
    std::fs::write(p.join("inner.ts"), format!("{NATIVE} import {{outer}} from './outer'; export const inner:ApplicationConfig=combine(outer,Base);")).unwrap();
    let state = crate::mcp::McpState::new(config);
    // No explicit capture, restore or bootstrap query precedes the cold answer.
    let cold = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":"outer"}),
    );
    facts(&state, p, true, true);
    let warm = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":"outer"}),
    );
    assert_eq!(cold["edges"], warm["edges"]);
    let inner = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":"inner"}),
    );
    let mut targets = inner["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "IncludesApplicationConfig")
        .map(|edge| edge["object"]["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    targets.sort();
    assert_eq!(targets, vec!["leafConfig", "outer"]);
}
