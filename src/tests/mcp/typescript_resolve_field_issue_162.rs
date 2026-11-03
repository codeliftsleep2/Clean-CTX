//! Field transport preservation after approved bounded mapping capture.
use super::typescript_guard_kinds_field_support::{call, capture};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use serde_json::json;

pub(super) fn setup(form: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = super::typescript_guard_kinds_field_support::setup(form);
    for owner in ["one.ts", "two.ts"] {
        let original = std::fs::read_to_string(root.path().join(owner)).unwrap();
        let mut source = String::new();
        for (i, part) in original.split("canMatch:[").enumerate() {
            if i == 0 {
                source.push_str(part);
                continue;
            }
            let reference = part.split([',', ']']).next().unwrap();
            source.push_str(&format!("resolve:{{data0:{reference}}},canMatch:[{part}"));
        }
        std::fs::write(root.path().join(owner), source).unwrap();
    }
    (root, config)
}
fn fields(a: &TypeScriptAuthority) -> Vec<ExpressionFacet> {
    fn walk(a: &TypeScriptAuthority, v: &ExpressionFacet, out: &mut Vec<ExpressionFacet>) {
        match &v.value {
            ExpressionValue::Array(values) => {
                for v in values {
                    walk(a, v, out);
                }
            }
            ExpressionValue::Object(entries) => {
                assert!(a.admits_demanded_structure(&a.source, &a.source_hash, v));
                for (name, value) in entries {
                    if name == "resolve" {
                        assert!(
                            v.start <= value.start && value.start < value.end && value.end <= v.end
                        );
                        assert!(a.admits_demanded_structure(&a.source, &a.source_hash, value));
                        assert!(matches!(&value.value, ExpressionValue::Object(_)));
                        out.push(value.clone());
                    } else if name == "children" {
                        walk(a, value, out);
                    }
                }
            }
            _ => panic!("native route Array/Object required"),
        }
    }
    let root = a
        .declaration_facets
        .iter()
        .find(|f| f.declaration.source == a.source && f.declaration.name == "routes")
        .unwrap();
    let mut out = Vec::new();
    walk(a, root.initializer.as_ref().unwrap(), &mut out);
    out
}
fn siblings(a: &TypeScriptAuthority) {
    assert_eq!(crate::angular_meta::route_guards::project(a).len(), 1);
    assert_eq!(
        crate::angular_meta::route_guards::project_kinds(a).len(),
        32
    );
    assert_eq!(crate::angular_meta::route_components::project(a).len(), 1);
    assert_eq!(
        crate::angular_meta::route_lazy_components::project(a).len(),
        1
    );
    assert_eq!(
        crate::angular_meta::route_load_children::project(a).len(),
        1
    );
    assert_eq!(
        crate::angular_meta::route_providers::project(a)
            .iter()
            .filter(|e| e.relation
                == crate::layers::meta::semantic::SemanticRelation::RegistersProvider)
            .count(),
        2
    );
}
#[test]
fn issue_162_resolve_field_forms_duplicates_serde() {
    let _serial = crate::protocol::handler_response_serial();
    for form in ["direct", "alias", "barrel", "namespace", "local"] {
        let (root, config) = setup(form);
        let p = root.path();
        let state = crate::mcp::McpState::new(config.clone());
        for owner in ["one.ts", "two.ts"] {
            let a = capture(&state, p, owner);
            let retained = fields(&a);
            assert_eq!(retained.len(), 7);
            siblings(&a);
            // The approved immediate mapping is retained without assigning resolver meaning.
            assert!(retained.iter().all(
                |v| matches!(&v.value, ExpressionValue::Object(entries) if entries.len() == 1)
            ));
            let restored: TypeScriptAuthority =
                serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
            assert_eq!(fields(&restored), retained);
        }
        drop(state);
        let owner = p.join("one.ts");
        let source = std::fs::read_to_string(&owner).unwrap();
        let changed = source.replacen("resolve:{", "resolve:{},resolve:{", 1);
        std::fs::write(&owner, changed).unwrap();
        let state = crate::mcp::McpState::new(config);
        let a = capture(&state, p, "one.ts");
        let retained = fields(&a);
        assert_eq!(retained.len(), 8);
        assert_ne!(retained[0].start, retained[1].start);
        siblings(&a);
    }
}
#[test]
fn issue_162_resolve_field_sqlite_before_queries_owner_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let before = capture(&state, p, "one.ts");
    let expected = fields(&before);
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
        fields(snapshot.authority.typescript_authority.as_ref().unwrap()),
        expected
    );
    let original = std::fs::read(p.join("one.ts")).unwrap();
    drop(state);
    std::fs::write(
        p.join("one.ts"),
        "import type {Routes} from '@angular/router'; export const routes:Routes=[];",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    let changed = capture(&state, p, "one.ts");
    assert_ne!(changed.source_hash, before.source_hash);
    assert!(!snapshot.authority.is_current(&changed.source_hash));
    assert!(fields(&changed).is_empty());
    drop(state);
    std::fs::write(p.join("one.ts"), original).unwrap();
    let state = crate::mcp::McpState::new(config);
    let repaired = capture(&state, p, "one.ts");
    assert_eq!(fields(&repaired), expected);
    siblings(&repaired);
}
#[test]
fn issue_162_resolve_field_dependency_package_config_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..4 {
        let (root, config) = setup("barrel");
        let p = root.path();
        let owner = std::fs::read(p.join("one.ts")).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let before = capture(&state, p, "one.ts");
        let expected = fields(&before);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("one.ts")}),
        );
        drop(state);
        // Dependencies are independently consumed by accepted guard siblings;
        // Mapping dependencies share the existing consumed witness lifecycle.
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
            2=>{let mut v:serde_json::Value=serde_json::from_slice(&original).unwrap();v["version"]=json!("20.0.1");v.to_string()},
            _=>{let mut v:serde_json::Value=serde_json::from_slice(&original).unwrap();v["compilerOptions"]["baseUrl"]=json!(".");v.to_string()},
        };
        std::fs::write(&dependency, changed).unwrap();
        assert!(
            !before
                .witnesses
                .iter()
                .all(crate::ir::typescript_resolution::Witness::current)
        );
        let state = crate::mcp::McpState::new(config.clone());
        let refused = super::angular_bootstrap_inline_controls_tests::raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("one.ts")}),
        );
        assert!(refused.get("error").is_some());
        let now = capture(&state, p, "one.ts");
        assert_eq!(now.source_hash, before.source_hash);
        assert_eq!(
            fields(&now)
                .iter()
                .map(|v| (v.start, v.end))
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|v| (v.start, v.end))
                .collect::<Vec<_>>()
        );
        assert_eq!(std::fs::read(p.join("one.ts")).unwrap(), owner);
        drop(state);
        std::fs::write(dependency, original).unwrap();
        let state = crate::mcp::McpState::new(config);
        let repaired = capture(&state, p, "one.ts");
        assert_eq!(fields(&repaired), expected);
        siblings(&repaired);
    }
}
