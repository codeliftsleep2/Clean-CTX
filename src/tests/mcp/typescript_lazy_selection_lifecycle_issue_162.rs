//! Exact neutral retention/currentness; runtime and lazy relation remain operator-owned.
use super::typescript_lazy_selection_support::*;
use serde_json::json;
#[test]
fn issue_162_lazy_selection_sqlite_before_consumer_and_dependency_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..6 {
        let (root, config) = setup("alias");
        let p = root.path();
        let state = crate::mcp::McpState::new(config.clone());
        let before = capture(&state, p, "one.ts");
        let expected = before.arrow_selections.clone();
        proofs(&before);
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
        let restored = snapshot.authority.typescript_authority.as_ref().unwrap();
        assert_eq!(restored.arrow_selections, expected);
        proofs(restored);
        drop(state);
        let dependency = p.join(match mode {
            0 | 1 => "alias-a.ts",
            2 => "screen-a.ts",
            3 => "node_modules/@angular/core/package.json",
            4 => "tsconfig.json",
            _ => "unrelated.ts",
        });
        let original = std::fs::read(&dependency).ok();
        let changed=match mode{
            0=>"export {Screen as PublicScreen} from './screen-b';".to_owned(),1=>"export {};".to_owned(),2=>"export class Other {}".to_owned(),
            3=>{let mut value:serde_json::Value=serde_json::from_slice(original.as_ref().unwrap()).unwrap();value["version"]=json!("20.0.1");value.to_string()},
            4=>r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"baseUrl":"."}}"#.to_owned(),_=>"export class Unrelated {}".to_owned(),
        };
        std::fs::write(&dependency, changed).unwrap();
        if mode != 5 {
            assert!(
                before.arrow_selection_at(expected[0].outer).is_none(),
                "stale dependency {mode}"
            );
        }
        let state = crate::mcp::McpState::new(config.clone());
        let now = capture(&state, p, "one.ts");
        let selected: Vec<_> = now
            .arrow_selections
            .iter()
            .filter_map(|proof| now.arrow_selection_at(proof.outer))
            .collect();
        match mode {
            0 => {
                assert_eq!(selected.len(), 7);
                assert!(
                    selected
                        .iter()
                        .all(|proof| proof.target.source.ends_with("screen-b.ts"))
                );
            }
            1 | 2 => assert_eq!(selected.len(), 3),
            _ => assert_eq!(selected.len(), 7),
        }
        drop(state);
        if let Some(bytes) = original {
            std::fs::write(&dependency, bytes).unwrap();
        } else {
            std::fs::remove_file(&dependency).unwrap();
        }
        let state = crate::mcp::McpState::new(config);
        let repaired = capture(&state, p, "one.ts");
        proofs(&repaired);
        assert_eq!(repaired.arrow_selections, expected);
    }
}
#[test]
fn issue_162_lazy_selection_owner_member_and_field_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(
        p.join("both.ts"),
        "export {Screen as First} from './screen-a'; export {Screen as Second} from './screen-b';",
    )
    .unwrap();
    let mut original: Option<crate::ir::typescript_resolution::TypeScriptAuthority> = None;
    for (callback, wanted) in [
        ("()=>import('./both').then(m=>m.First)", Some("screen-a.ts")),
        (
            "()=>import('./both').then(m=>m.Second)",
            Some("screen-b.ts"),
        ),
        ("()=>import('./both').then(m=>m.Missing)", None),
    ] {
        rewrite(p, callback);
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "one.ts");
        if let Some(wanted) = wanted {
            let proof = &a.arrow_selections[0];
            assert!(
                a.arrow_selection_at(proof.outer)
                    .unwrap()
                    .target
                    .source
                    .ends_with(wanted)
            );
        } else {
            assert!(a.arrow_selections.is_empty());
        }
        if let Some(old) = original.as_ref() {
            assert_ne!(old.source_hash, a.source_hash);
            let authority = crate::mcp::semantic_compilation::retained::RetainedAuthority {
                typescript_authority: Some(old.clone()),
            };
            assert!(!authority.is_current(&a.source_hash));
        } else {
            original = Some(a.clone());
        }
        assert_eq!(crate::angular_meta::route_components::project(&a).len(), 1);
        assert!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .any(|e| e.relation
                    == crate::layers::meta::semantic::SemanticRelation::RegistersProvider)
        );
    }
    std::fs::write(p.join("one.ts"),"import type {Routes} from '@angular/router'; import {Screen} from './screen-a'; import {Token} from './left'; export const routes:Routes=[{path:'eager',component:Screen,providers:[Token]}];").unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    assert!(a.arrow_selections.is_empty());
    assert_eq!(crate::angular_meta::route_components::project(&a).len(), 1);
}
