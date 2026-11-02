//! Generation-77 retained transport, relation identity and currentness.
use super::angular_guard_kinds_controls_tests::{KINDS, answer, count, kinds};
use super::typescript_guard_kinds_field_support::{call, capture, setup};
use serde_json::json;
fn facts(state: &crate::mcp::McpState, p: &std::path::Path, wanted: usize) {
    let observed = answer(state, p, "same");
    for (_, _, name) in KINDS {
        assert_eq!(count(&observed, name), wanted, "public {name}");
    }
    assert_eq!(
        count(&observed, "GuardedBy"),
        0,
        "no umbrella/weak downgrade on candidate objects"
    );
    assert_eq!(observed["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(observed["coverage"]["source_complete"], false);
}
#[test]
fn issue_162_guard77_controls_sqlite_complete_index_before_queries_and_public_scope() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("namespace");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let a = capture(&state, p, "one.ts");
    let expected = kinds(&a);
    assert_eq!(expected.len(), 32);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("one.ts")}),
    );
    drop(state);
    let state = crate::mcp::McpState::new(config);
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
        let edges = index.forward_edges_by_identity("angular", "Route", "same");
        for (_, relation, _) in KINDS {
            assert_eq!(edges.iter().filter(|e| e.relation == relation).count(), 4);
        }
    }
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
    for (_, _, name) in KINDS {
        assert_eq!(count(&scoped, name), 4);
    }
    facts(&state, p, 8);
    let owner = std::fs::read(p.join("one.ts")).unwrap();
    std::fs::remove_file(p.join("one.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, 4);
    std::fs::write(p.join("one.ts"), owner).unwrap();
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
fn issue_162_guard77_controls_slot_field_kind_child_provider_root_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("barrel");
    let p = root.path();
    let original = std::fs::read_to_string(p.join("one.ts")).unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    let baseline = capture(&state, p, "one.ts");
    let span = super::typescript_route_guard_field_support::retained_field(&baseline, "canMatch")
        [0]
    .object;
    drop(state);
    let mut removed_object = original.clone();
    assert_eq!(&removed_object[span.1..span.1 + 1], ",");
    removed_object.replace_range(span.0..span.1 + 1, "");
    let changes = [
        (removed_object, 24),
        (original.replace("[A,B,A]", "[A,B]"), 32),
        (original.replace("[A,B,A]", "[B]"), 28),
        (original.replace("canMatch:", "other:"), 24),
        (original.replace("providers:[Token]", "providers:[]"), 32),
        (
            original.replace("path:'parent',children:", "path:'parent',other:"),
            24,
        ),
        (
            "import type {Routes} from '@angular/router'; export const routes:Routes=[];"
                .to_owned(),
            0,
        ),
    ];
    for (source, wanted) in changes {
        assert_ne!(source, original);
        std::fs::write(p.join("one.ts"), source).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        assert_eq!(kinds(&capture(&state, p, "one.ts")).len(), wanted);
        drop(state);
        std::fs::write(p.join("one.ts"), &original).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        assert_eq!(kinds(&capture(&state, p, "one.ts")).len(), 32);
    }
}
#[test]
fn issue_162_guard77_controls_unchanged_owner_target_barrel_package_config_stale_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..5 {
        let (root, config) = setup("barrel");
        let p = root.path();
        let owner = std::fs::read(p.join("one.ts")).unwrap();
        std::fs::write(p.join("foreign-router.ts"), "export type Routes=unknown[];").unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let before = capture(&state, p, "one.ts");
        assert_eq!(kinds(&before).len(), 32);
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
        let source=match mode {0=>"export {Gate as A} from './guard-b'; export {Gate as B} from './guard-b';".to_owned(),1=>"export {Gate as B} from './guard-b';".to_owned(),2=>"export const Gate=42;".to_owned(),3=>{let mut v:serde_json::Value=serde_json::from_slice(&original).unwrap();v["version"]=json!("20.0.1");v.to_string()},_=>r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"paths":{"@angular/router":["./foreign-router.ts"]}}}"#.to_owned()};
        std::fs::write(&dependency, source).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let refused = super::angular_bootstrap_inline_controls_tests::raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("one.ts")}),
        );
        assert!(refused.get("error").is_some(), "stale restore {mode}");
        let now = capture(&state, p, "one.ts");
        assert_eq!(now.source_hash, before.source_hash);
        let edges = kinds(&now);
        assert_eq!(
            edges.len(),
            match mode {
                0 => 28,
                1 => 16,
                4 => 0,
                _ => 32,
            }
        );
        if mode == 0 {
            assert!(
                edges
                    .iter()
                    .all(|e| e.object.file.as_ref().unwrap().ends_with("guard-b.ts"))
            );
        }
        assert_eq!(std::fs::read(p.join("one.ts")).unwrap(), owner);
        drop(state);
        std::fs::write(dependency, original).unwrap();
        let state = crate::mcp::McpState::new(config);
        assert_eq!(kinds(&capture(&state, p, "one.ts")).len(), 32);
    }
}
