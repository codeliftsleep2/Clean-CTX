//! Generation-78 resolver production transport, currentness and retirement.
use super::angular_resolver78_controls_tests::{answer, call, capture, count, fixture};
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;
fn same(s: &crate::mcp::McpState, p: &std::path::Path, n: usize) {
    let v = answer(s, p, "same");
    assert_eq!(count(&v), n);
    assert_eq!(v["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(v["coverage"]["source_complete"], false);
}
#[test]
fn issue_162_resolver78_lifecycle_sqlite_index_before_query_scope_refresh_delete_recreate() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("namespace");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let a = capture(&state, p, "one.ts");
    let expected = crate::angular_meta::route_resolvers::project(&a);
    assert_eq!(expected.len(), 8);
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
                .filter(|e| e.relation == SemanticRelation::ResolvedBy
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
                .filter(|e| e.relation == SemanticRelation::ResolvedBy)
                .count(),
            4
        );
    }
    same(&state, p, 8);
    same(&state, p, 8);
    call(&state, p, "refresh_workspace", json!({}));
    same(&state, p, 8);
    let scoped = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"same","withinPath":p.join("one.ts")}),
    );
    assert_eq!(count(&scoped), 4);
    same(&state, p, 8);
    let owner = std::fs::read(p.join("one.ts")).unwrap();
    std::fs::remove_file(p.join("one.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    same(&state, p, 4);
    std::fs::write(p.join("one.ts"), owner).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    same(&state, p, 8);
    let target = std::fs::read(p.join("guard-a.ts")).unwrap();
    std::fs::remove_file(p.join("guard-a.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    same(&state, p, 4);
    std::fs::write(p.join("guard-a.ts"), target).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    same(&state, p, 8);
}
#[test]
fn issue_162_resolver78_lifecycle_key_field_object_child_provider_root_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let source = std::fs::read_to_string(p.join("one.ts")).unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    let span = a.resolve_mappings[0].owner;
    let mut removed = source.clone();
    removed.replace_range(span.0..span.1 + 1, "");
    let first = "resolve:{data0:Gate,data1:Right,data2:Gate,},";
    assert!(source.contains(first));
    let changes = [
        (source.replacen("data2:Gate,", "", 1), 8, 4),
        (source.replacen(first, "", 1), 6, 4),
        (removed, 6, 4),
        (source.replace("resolve:{data0:Gate,}", "resolve:{}"), 7, 3),
        (
            source.replacen("providers:[Token]", "providers:[]", 1),
            8,
            4,
        ),
        (
            "import type {Routes} from '@angular/router';export const routes:Routes=[];".into(),
            4,
            2,
        ),
    ];
    for (changed, wanted, child) in changes {
        assert_ne!(changed, source);
        std::fs::write(p.join("one.ts"), changed).unwrap();
        same(&state, p, wanted);
        assert_eq!(count(&answer(&state, p, "parent/child")), child);
    }
    std::fs::write(p.join("one.ts"), source).unwrap();
    same(&state, p, 8);
    assert_eq!(count(&answer(&state, p, "parent/child")), 4);
}
#[test]
fn issue_162_resolver78_lifecycle_unchanged_owner_target_barrel_package_config_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..4 {
        let (root, config) = fixture("barrel");
        let p = root.path();
        let owner = std::fs::read(p.join("one.ts")).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "one.ts");
        let edges = crate::angular_meta::route_resolvers::project(&a);
        assert_eq!(edges.len(), 8);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("one.ts")}),
        );
        drop(state);
        let dep = p.join(match mode {
            0 => "guard-a.ts",
            1 => "guard-api.ts",
            2 => "node_modules/@angular/router/package.json",
            _ => "tsconfig.json",
        });
        let bytes = std::fs::read(&dep).unwrap();
        let changed = match mode {
            0 => "export const Other=42;".into(),
            1 => "export {Gate as A} from './guard-b';export {Gate as B} from './guard-b';".into(),
            2 => {
                let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                v["version"] = json!("20.0.1");
                v.to_string()
            }
            _ => {
                let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                v["compilerOptions"]["baseUrl"] = json!(".");
                v.to_string()
            }
        };
        std::fs::write(&dep, changed).unwrap();
        assert!(!a.admits_relation_evidence(&edges));
        let state = crate::mcp::McpState::new(config.clone());
        let refused = super::angular_bootstrap_inline_controls_tests::raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("one.ts")}),
        );
        assert!(refused.get("error").is_some());
        same(
            &state,
            p,
            if mode == 0 {
                4
            } else if mode == 1 {
                6
            } else {
                8
            },
        );
        let now = capture(&state, p, "one.ts");
        assert_eq!(now.source_hash, a.source_hash);
        assert_eq!(std::fs::read(p.join("one.ts")).unwrap(), owner);
        drop(state);
        std::fs::write(dep, bytes).unwrap();
        let state = crate::mcp::McpState::new(config);
        same(&state, p, 8);
    }
}
