//! Existing serde/SQLite/witness lifecycle for neutral guard fields and slots.
use super::typescript_route_guard_field_support::*;
use serde_json::json;
#[test]
fn issue_162_guard_field_sqlite_before_queries_and_owner_retirement_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let before = capture(&state, p, "one.ts");
    let expected = retained(&before);
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
    assert_eq!(
        retained(snapshot.authority.typescript_authority.as_ref().unwrap()),
        expected,
        "restored fields/slots before consumer queries"
    );
    let owner = std::fs::read(p.join("one.ts")).unwrap();
    drop(state);
    std::fs::write(
        p.join("one.ts"),
        "import type {Routes} from '@angular/router'; export const routes:Routes=[];",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    let changed = capture(&state, p, "one.ts");
    assert_ne!(changed.source_hash, snapshot.source_hash);
    assert!(!snapshot.authority.is_current(&changed.source_hash));
    assert!(retained(&changed).is_empty());
    drop(state);
    std::fs::write(p.join("one.ts"), owner).unwrap();
    let state = crate::mcp::McpState::new(config);
    let repaired = capture(&state, p, "one.ts");
    assert_eq!(retained(&repaired), expected);
    assert_eq!(repaired.source_hash, before.source_hash);
}
#[test]
fn issue_162_guard_field_dependency_barrel_package_config_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..4 {
        let (root, config) = setup("barrel");
        let p = root.path();
        let owner = std::fs::read(p.join("one.ts")).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let before = capture(&state, p, "one.ts");
        let expected = retained(&before);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("one.ts")}),
        );
        drop(state);
        let dependency = p.join(match mode {
            0 => "guard-a.ts",
            1 => "guard-api.ts",
            2 => "node_modules/@angular/router/package.json",
            _ => "tsconfig.json",
        });
        let original = std::fs::read(&dependency).unwrap();
        let changed=match mode {
   0=>"import type {CanActivateFn} from '@angular/router'; export const Gate:CanActivateFn=()=>false;".to_owned(),
   1=>"export {Gate as A} from './guard-b'; export {Gate as B} from './guard-b';".to_owned(),
   2=>{let mut value:serde_json::Value=serde_json::from_slice(&original).unwrap();value["version"]=json!("20.0.1");value.to_string()},
   _=>{let mut value:serde_json::Value=serde_json::from_slice(&original).unwrap();value["compilerOptions"]["baseUrl"]=json!(".");value.to_string()},
  };
        std::fs::write(&dependency, changed).unwrap();
        assert!(
            !before
                .witnesses
                .iter()
                .all(crate::ir::typescript_resolution::Witness::current),
            "consumed dependency must invalidate: {mode}"
        );
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
        let now = capture(&state, p, "one.ts");
        let fields = retained(&now);
        assert_eq!(fields.len(), 7);
        assert_eq!(now.source_hash, before.source_hash);
        if mode == 0 {
            assert_ne!(
                fields[0].elements[0].reference,
                expected[0].elements[0].reference
            );
        }
        if mode == 1 {
            assert!(fields.iter().flat_map(|f|&f.elements).all(|e|matches!(&e.reference,crate::ir::typescript_resolution::Resolution::Resolved(id) if id.source.ends_with("guard-b.ts"))));
        }
        assert_eq!(std::fs::read(p.join("one.ts")).unwrap(), owner);
        drop(state);
        std::fs::write(dependency, original).unwrap();
        let state = crate::mcp::McpState::new(config);
        assert_eq!(retained(&capture(&state, p, "one.ts")), expected);
    }
}
