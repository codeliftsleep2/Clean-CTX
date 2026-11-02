//! Canonical canActivate snapshot/index/currentness and public lifecycle.
use super::angular_route_guard_controls_tests::{answer, count, guards};
use super::typescript_route_guard_field_support::{call, capture, setup};
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;

fn facts(state: &crate::mcp::McpState, p: &std::path::Path, wanted: usize) {
    let result = answer(state, p, "same");
    assert_eq!(count(&result), wanted);
    assert_eq!(result["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(result["coverage"]["source_complete"], false);
}
#[test]
fn issue_162_guard_controls_sqlite_snapshot_index_before_queries_and_target_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("barrel");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let before = capture(&state, p, "one.ts");
    let expected = guards(&before);
    assert_eq!(expected.len(), 8);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("one.ts")}),
    );
    drop(state);
    let state = crate::mcp::McpState::new(config.clone());
    call(
        &state,
        p,
        "restore_context",
        json!({"filePath":p.join("one.ts")}),
    );
    let alias = state
        .alias_for_path(&p.join("one.ts").to_string_lossy())
        .unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    assert!(snapshot.authority.admits_relation_evidence(&expected));
    for edge in &expected {
        assert_eq!(
            snapshot
                .edges
                .iter()
                .filter(|e| e.relation == edge.relation
                    && e.subject.canonical == edge.subject.canonical
                    && e.object.canonical == edge.object.canonical)
                .count(),
            1
        );
    }
    {
        let index = state.workspace_index_read();
        assert_eq!(
            index
                .forward_edges_by_identity("angular", "Route", "same")
                .iter()
                .filter(|e| e.relation == SemanticRelation::GuardedBy)
                .count(),
            4
        );
    }
    facts(&state, p, 8);
    drop(state);
    let original = std::fs::read(p.join("guard-a.ts")).unwrap();
    std::fs::write(p.join("guard-a.ts"), "export const Other=123;").unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    let changed = capture(&state, p, "one.ts");
    assert_eq!(guards(&changed).len(), 4);
    facts(&state, p, 4);
    drop(state);
    std::fs::write(p.join("guard-a.ts"), original).unwrap();
    let state = crate::mcp::McpState::new(config);
    assert_eq!(guards(&capture(&state, p, "one.ts")).len(), 8);
    facts(&state, p, 8);
}
#[test]
fn issue_162_guard_controls_public_cold_warm_refresh_scope_delete_recreate() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("namespace");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    facts(&state, p, 8);
    facts(&state, p, 8);
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, 8);
    let scoped = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"same","withinPath":p.join("one.ts")}),
    );
    assert_eq!(count(&scoped), 4);
    facts(&state, p, 8);
    let original = std::fs::read(p.join("one.ts")).unwrap();
    std::fs::remove_file(p.join("one.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, 4);
    std::fs::write(p.join("one.ts"), original).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, 8);
    let target = std::fs::read(p.join("guard-a.ts")).unwrap();
    std::fs::remove_file(p.join("guard-a.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, 4);
    std::fs::write(p.join("guard-a.ts"), target).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, 8);
}
#[test]
fn issue_162_guard_controls_owner_slot_field_object_child_provider_root_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let original = std::fs::read_to_string(p.join("one.ts")).unwrap();
    let changes = [
        (original.replace("[Gate,Right,Gate]", "[Gate,Right]"), 8),
        (original.replace("[Gate,Right,Gate]", "[Right]"), 7),
        (
            original.replace("canActivate:[Gate,Right,Gate]", "other:[Gate,Right,Gate]"),
            6,
        ),
        (
            original.replace(
                "{path:'same',canActivate:[Gate,Right,Gate],providers:[Token]},",
                "",
            ),
            6,
        ),
        (
            original.replace(",{path:'child',canActivate:[Right]}", ""),
            7,
        ),
        (original.replace("providers:[Token]", "providers:[]"), 8),
        (
            "import type {Routes} from '@angular/router'; export const routes:Routes=[];"
                .to_owned(),
            0,
        ),
    ];
    for (changed, wanted) in changes {
        assert_ne!(
            changed, original,
            "mutation must touch actual fixture bytes"
        );
        std::fs::write(p.join("one.ts"), changed).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "one.ts");
        assert_eq!(guards(&a).len(), wanted);
        drop(state);
        std::fs::write(p.join("one.ts"), &original).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        assert_eq!(guards(&capture(&state, p, "one.ts")).len(), 8);
    }
}
#[test]
fn issue_162_guard_controls_unchanged_owner_export_api_package_config_stale_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..5 {
        let (root, config) = setup("barrel");
        let p = root.path();
        let owner = std::fs::read(p.join("one.ts")).unwrap();
        std::fs::write(p.join("foreign-router.ts"), "export type Routes=unknown[];").unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let before = capture(&state, p, "one.ts");
        assert_eq!(guards(&before).len(), 8);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("one.ts")}),
        );
        drop(state);
        let dependency = p.join(match mode {
            0 | 1 => "guard-api.ts",
            2 => "guard-a.ts",
            3 => "node_modules/@angular/router/package.json",
            _ => "tsconfig.json",
        });
        let original = std::fs::read(&dependency).unwrap();
        let changed=match mode {
            0=>"export {Gate as A} from './guard-b'; export {Gate as B} from './guard-b';".to_owned(),
            1=>"export {Gate as B} from './guard-b';".to_owned(),
            2=>"export const Gate=42;".to_owned(),
            3=>{let mut v:serde_json::Value=serde_json::from_slice(&original).unwrap();v["version"]=json!("20.0.1");v.to_string()},
            _=>r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"paths":{"@angular/router":["./foreign-router.ts"]}}}"#.to_owned(),
        };
        std::fs::write(&dependency, changed).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let refused = super::angular_bootstrap_inline_controls_tests::raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("one.ts")}),
        );
        assert!(
            refused.get("error").is_some(),
            "stale restore {mode}: {refused}"
        );
        let a = capture(&state, p, "one.ts");
        assert_eq!(a.source_hash, before.source_hash);
        let edges = guards(&a);
        assert_eq!(
            edges.len(),
            match mode {
                0 => 7,
                1 => 4,
                4 => 0,
                _ => 8,
            },
            "dependency {mode}"
        );
        if mode == 0 {
            assert!(
                edges
                    .iter()
                    .all(|e| e.object.file.as_ref().unwrap().ends_with("guard-b.ts"))
            );
        }
        if mode == 2 {
            assert!(
                edges
                    .iter()
                    .filter(|e| e.object.file.as_ref().unwrap().ends_with("guard-a.ts"))
                    .all(|e| e.object.name == "Gate"),
                "noncallable exact runtime value remains written-use eligible"
            );
        }
        assert_eq!(std::fs::read(p.join("one.ts")).unwrap(), owner);
        drop(state);
        std::fs::write(dependency, original).unwrap();
        let state = crate::mcp::McpState::new(config);
        assert_eq!(guards(&capture(&state, p, "one.ts")).len(), 8);
    }
}
