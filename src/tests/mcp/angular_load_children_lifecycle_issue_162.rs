//! Canonical lazy persistence, dependency currentness and public acquisition.
use super::typescript_children_selection_support::{call, capture};
use super::typescript_shared_selection_roles_tests::setup_role;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;
fn lazy(
    a: &crate::ir::typescript_resolution::TypeScriptAuthority,
) -> Vec<crate::layers::meta::semantic::SemanticEdge> {
    crate::angular_meta::route_load_children::project(a)
}
fn facts(state: &crate::mcp::McpState, p: &std::path::Path, wanted: usize) {
    let a = call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"same"}),
    );
    assert_eq!(
        a["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "LoadsChildren")
            .count(),
        wanted
    );
    assert_eq!(a["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(a["coverage"]["source_complete"], false);
}
#[test]
fn issue_162_children_controls_sqlite_snapshot_index_before_queries_and_role_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for role in ["module", "routes"] {
        let (root, config) = setup_role("alias", role);
        let p = root.path();
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "one.ts");
        let expected = lazy(&a);
        assert_eq!(expected.len(), 7);
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
            let edges = index.forward_edges_by_identity("angular", "Route", "same");
            assert_eq!(
                edges
                    .iter()
                    .filter(|e| e.relation == SemanticRelation::LoadsChildren)
                    .count(),
                3
            );
        }
        facts(&state, p, 6);
        drop(state);
        let original = std::fs::read(p.join("feature-a.ts")).unwrap();
        std::fs::write(p.join("feature-a.ts"), "export class Screen {}").unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let changed = capture(&state, p, "one.ts");
        assert_eq!(lazy(&changed).len(), 3);
        facts(&state, p, 2);
        drop(state);
        std::fs::write(p.join("feature-a.ts"), original).unwrap();
        let state = crate::mcp::McpState::new(config);
        assert_eq!(lazy(&capture(&state, p, "one.ts")).len(), 7);
        facts(&state, p, 6);
    }
}
#[test]
fn issue_162_children_controls_public_cold_warm_refresh_scope_and_root_target_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    for role in ["module", "routes"] {
        for form in ["direct", "reexport", "alias"] {
            let (root, config) = setup_role(form, role);
            let p = root.path();
            let state = crate::mcp::McpState::new(config);
            facts(&state, p, 6);
            facts(&state, p, 6);
            call(&state, p, "refresh_workspace", json!({}));
            facts(&state, p, 6);
            let scoped = call(
                &state,
                p,
                "workspace_query",
                json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"same","withinPath":p.join("one.ts")}),
            );
            assert_eq!(
                scoped["edges"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|e| e["relation"] == "LoadsChildren")
                    .count(),
                3
            );
            facts(&state, p, 6);
            let original = std::fs::read(p.join("one.ts")).unwrap();
            std::fs::remove_file(p.join("one.ts")).unwrap();
            call(&state, p, "refresh_workspace", json!({}));
            facts(&state, p, 3);
            std::fs::write(p.join("one.ts"), original).unwrap();
            call(&state, p, "refresh_workspace", json!({}));
            facts(&state, p, 6);
            let target = std::fs::read(p.join("feature-a.ts")).unwrap();
            std::fs::remove_file(p.join("feature-a.ts")).unwrap();
            call(&state, p, "refresh_workspace", json!({}));
            facts(&state, p, 2);
            std::fs::write(p.join("feature-a.ts"), target).unwrap();
            call(&state, p, "refresh_workspace", json!({}));
            facts(&state, p, 6);
        }
    }
}

#[test]
fn issue_162_children_controls_export_barrel_package_config_stale_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for role in ["module", "routes"] {
        for mode in 0..5 {
            let (root, config) = setup_role("alias", role);
            let p = root.path();
            std::fs::write(
                p.join("foreign-core.ts"),
                "export function Component(_:unknown){return (_:unknown)=>{};}",
            )
            .unwrap();
            std::fs::write(p.join("foreign-router.ts"), "export type Routes=unknown[];").unwrap();
            let owner = std::fs::read(p.join("one.ts")).unwrap();
            let state = crate::mcp::McpState::new(config.clone());
            assert_eq!(lazy(&capture(&state, p, "one.ts")).len(), 7);
            call(
                &state,
                p,
                "save_context",
                json!({"filePath":p.join("one.ts")}),
            );
            drop(state);
            let dependency = p.join(match mode {
                0 | 1 => "alias-a.ts",
                2 => "feature-a.ts",
                3 => "node_modules/@angular/core/package.json",
                _ => "tsconfig.json",
            });
            let original = std::fs::read(&dependency).unwrap();
            let changed=match mode{0=>"export {Screen as PublicScreen} from './feature-b';".to_owned(),1=>"export {};".to_owned(),2=>"export class Other {}".to_owned(),3=>{let mut m:serde_json::Value=serde_json::from_slice(&original).unwrap();m["version"]=json!("20.0.1");m.to_string()},_=>r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"paths":{"@angular/core":["./foreign-core.ts"],"@angular/router":["./foreign-router.ts"]}}}"#.to_owned()};
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
                "stale proof must not restore: {refused}"
            );
            let a = capture(&state, p, "one.ts");
            let projected = lazy(&a);
            let wanted = match mode {
                1 | 2 => 3,
                4 => 0,
                _ => 7,
            };
            assert_eq!(projected.len(), wanted, "dependency {mode}");
            if mode == 0 {
                assert!(
                    projected.iter().all(|e| e
                        .object
                        .file
                        .as_ref()
                        .unwrap()
                        .ends_with("feature-b.ts"))
                );
            }
            facts(
                &state,
                p,
                match mode {
                    1 | 2 => 2,
                    4 => 0,
                    _ => 6,
                },
            );
            drop(state);
            std::fs::write(&dependency, original).unwrap();
            let state = crate::mcp::McpState::new(config);
            assert_eq!(lazy(&capture(&state, p, "one.ts")).len(), 7);
            facts(&state, p, 6);
            assert_eq!(std::fs::read(p.join("one.ts")).unwrap(), owner);
        }
    }
}
