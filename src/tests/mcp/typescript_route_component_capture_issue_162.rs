//! Neutral component-field capture, not eager route semantic admission.
use super::angular_bootstrap_inline_controls_tests::{call, capture};
use super::angular_config_composition_controls_tests::setup as base_setup;
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{DeclarationId, Resolution, TypeScriptAuthority};
use serde_json::json;
const TARGET: &str = "import {Component} from '@angular/core'; import {Token} from './left'; @Component({providers:[Token]}) export class Screen {}";
fn setup() -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = base_setup("Base,Feature");
    let p = root.path();
    let router = p.join("node_modules/@angular/router");
    std::fs::create_dir_all(&router).unwrap();
    std::fs::write(
        router.join("package.json"),
        r#"{"name":"@angular/router","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(router.join("index.d.ts"),"export interface Route {path?:string;component?:unknown;providers?:unknown[];children?:Route[];} export type Routes=Route[];").unwrap();
    std::fs::write(p.join("target-a.ts"), TARGET).unwrap();
    std::fs::write(p.join("target-b.ts"), TARGET).unwrap();
    std::fs::write(
        p.join("target-api.ts"),
        "export {Screen as A} from './target-a'; export {Screen as B} from './target-b';",
    )
    .unwrap();
    for (i,(import,a,b)) in [
        ("import {Screen} from './target-a'; import {Screen as B} from './target-b';","Screen","B"),
        ("import {Screen as A} from './target-a'; import {Screen as B} from './target-b';","A","B"),
        ("import {A,B} from './target-api';","A","B"),
        ("import * as ns from './target-api';","ns.A","ns.B"),
        ("import {Component} from '@angular/core'; import {Screen as B} from './target-b'; @Component({providers:[Token]}) class Screen {}","Screen","B"),
    ].into_iter().enumerate() {
        std::fs::write(p.join(format!("capture-{i}.ts")),format!("import type {{Routes}} from '@angular/router'; import {{Token}} from './left'; {import} export const routes:Routes=[{{path:'same',component:{a},providers:[Token]}},{{path:'same',component:{a}}},{{path:'',component:{b}}},{{path:'parent',children:[{{path:'child',component:{b}}}]}}];")).unwrap();
    }
    (root, config)
}
fn values<'a>(value: &'a ExpressionFacet, output: &mut Vec<&'a ExpressionFacet>) {
    match &value.value {
        ExpressionValue::Array(items) => {
            for item in items {
                values(item, output);
            }
        }
        ExpressionValue::Object(fields) => {
            for (name, value) in fields {
                if name == "component" {
                    output.push(value);
                } else if name == "children" {
                    values(value, output);
                }
            }
        }
        _ => {}
    }
}
fn targets(a: &TypeScriptAuthority) -> Vec<DeclarationId> {
    let root = a
        .declaration_facets
        .iter()
        .find(|f| f.declaration.source == a.source && f.declaration.name == "routes")
        .unwrap();
    let mut fields = Vec::new();
    values(root.initializer.as_ref().unwrap(), &mut fields);
    assert_eq!(
        fields.len(),
        4,
        "exact provider/free/repeated/empty/child fields"
    );
    fields
        .into_iter()
        .map(|field| {
            let ExpressionValue::Reference(Resolution::Resolved(id)) = &field.value else {
                panic!("{field:?}")
            };
            assert_eq!(
                a.demanded_reference_at(&a.source, &a.source_hash, field.start, field.end),
                Some(id)
            );
            assert_eq!(
                crate::angular_meta::class_role_authority::declaration_role(a, id),
                Some(crate::angular_meta::decorators::ClassKind::Component)
            );
            id.clone()
        })
        .collect()
}
#[test]
fn issue_162_route_component_capture_exact_routes_same_names_and_serde() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup();
    let state = crate::mcp::McpState::new(config);
    for i in 0..5 {
        let a = capture(&state, root.path(), &format!("capture-{i}.ts"));
        assert!(
            a.capture_current
                && a.witnesses
                    .iter()
                    .all(crate::ir::typescript_resolution::Witness::current)
        );
        let ids = targets(&a);
        assert_eq!(ids[0], ids[1]);
        assert_eq!(ids[2], ids[3]);
        assert_ne!(ids[0], ids[2]);
        assert_eq!(ids[0].name, "Screen");
        assert_eq!(ids[2].name, "Screen");
        let restored: TypeScriptAuthority =
            serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
        assert_eq!(targets(&restored), ids);
    }
}
#[test]
fn issue_162_route_component_capture_unresolved_typeonly_duplicate_and_metadata() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..4 {
        let (root, config) = setup();
        let p = root.path();
        let source = match mode {
            0 => {
                "import type {Routes} from '@angular/router'; import {Token} from './left'; export const routes:Routes=[{path:'ok',component:Missing,providers:[Token]}];"
            }
            1 => {
                "import type {Routes} from '@angular/router'; import type {Screen} from './target-a'; export const routes:Routes=[{path:'x',component:Screen}];"
            }
            2 => {
                "import type {Routes} from '@angular/router'; import {Screen as A} from './target-a'; import {Screen as B} from './target-b'; export const routes:Routes=[{component:A,component:B}];"
            }
            3 => {
                "import {Component} from '@angular/core'; import {Screen} from './target-a'; @Component({component:Screen}) export class Host {}"
            }
            _ => unreachable!(),
        };
        std::fs::write(p.join("probe.ts"), source).unwrap();
        let state = crate::mcp::McpState::new(config);
        let a = capture(&state, p, "probe.ts");
        if mode == 3 {
            assert!(crate::angular_meta::route_providers::project(&a).is_empty());
            continue;
        }
        let root = a
            .declaration_facets
            .iter()
            .find(|f| f.declaration.name == "routes" && f.declaration.source == a.source)
            .unwrap();
        let mut fields = Vec::new();
        values(root.initializer.as_ref().unwrap(), &mut fields);
        assert_eq!(fields.len(), if mode == 2 { 2 } else { 1 });
        if mode < 2 {
            assert!(
                !matches!(
                    &fields[0].value,
                    ExpressionValue::Reference(Resolution::Resolved(_))
                ),
                "mode={mode} no spelling fallback"
            );
        } else {
            let ExpressionValue::Reference(Resolution::Resolved(left)) = &fields[0].value else {
                panic!("left target")
            };
            let ExpressionValue::Reference(Resolution::Resolved(right)) = &fields[1].value else {
                panic!("right target")
            };
            assert_ne!(left, right);
        }
    }
}
#[test]
fn issue_162_route_component_capture_sqlite_before_query_and_dependency_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup();
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let a = capture(&state, p, "capture-2.ts");
    let expected = targets(&a);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("capture-2.ts")}),
    );
    drop(state);
    let restored = crate::mcp::McpState::new(config.clone());
    call(
        &restored,
        p,
        "restore_context",
        json!({"filePath":p.join("capture-2.ts")}),
    );
    let alias = restored
        .alias_for_path(&p.join("capture-2.ts").to_string_lossy())
        .unwrap();
    let snap = restored.semantic_snapshot(&alias).unwrap();
    assert_eq!(
        targets(snap.authority.typescript_authority.as_ref().unwrap()),
        expected
    );
    drop(restored);
    std::fs::write(p.join("target-a.ts"), "export class Screen {}").unwrap();
    assert!(
        !a.witnesses
            .iter()
            .all(crate::ir::typescript_resolution::Witness::current)
    );
    let next = crate::mcp::McpState::new(config.clone());
    let changed = capture(&next, p, "capture-2.ts");
    let root = changed
        .declaration_facets
        .iter()
        .find(|f| f.declaration.name == "routes" && f.declaration.source == changed.source)
        .unwrap();
    let mut fields = Vec::new();
    values(root.initializer.as_ref().unwrap(), &mut fields);
    let ExpressionValue::Reference(Resolution::Resolved(id)) = &fields[0].value else {
        panic!("exact class reference remains independently resolved")
    };
    assert_eq!(
        crate::angular_meta::class_role_authority::declaration_role(&changed, id),
        None
    );
    drop(next);
    std::fs::write(p.join("target-a.ts"), TARGET).unwrap();
    let repaired = crate::mcp::McpState::new(config);
    assert_eq!(targets(&capture(&repaired, p, "capture-2.ts")), expected);
}
