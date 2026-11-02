//! Approved written canActivate investigation only; no capture/role/semantic change.
use super::typescript_children_selection_support::{call, capture};
use super::typescript_shared_selection_roles_tests::setup_role;
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};

fn fixture(form: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = setup_role("direct", "module");
    let p = root.path();
    let router = p.join("node_modules/@angular/router/index.d.ts");
    let mut declarations = std::fs::read_to_string(&router).unwrap();
    // Minimal native fixture type, not a claim about runtime callable eligibility.
    declarations.push_str("\nexport interface Route {canActivate?:unknown[];} export type CanActivateFn=(route:unknown,state:unknown)=>boolean;\n");
    std::fs::write(router, declarations).unwrap();
    for side in ["a", "b"] {
        std::fs::write(p.join(format!("guard-{side}.ts")),"import type {CanActivateFn} from '@angular/router'; export const Gate:CanActivateFn=()=>true;").unwrap();
    }
    std::fs::write(
        p.join("guard-api.ts"),
        "export {Gate as A} from './guard-a'; export {Gate as B} from './guard-b';",
    )
    .unwrap();
    let (imports, a, b) = match form {
        "direct" => (
            "import {Gate} from './guard-a'; import {Gate as Right} from './guard-b';",
            "Gate",
            "Right",
        ),
        "alias" => (
            "import {Gate as A} from './guard-a'; import {Gate as B} from './guard-b';",
            "A",
            "B",
        ),
        "barrel" => ("import {A,B} from './guard-api';", "A", "B"),
        "namespace" => (
            "import * as left from './guard-a'; import * as right from './guard-b';",
            "left.Gate",
            "right.Gate",
        ),
        _ => (
            "function Gate(){return true;} function RightGate(){return false;}",
            "Gate",
            "RightGate",
        ),
    };
    let body = format!(
        "import type {{Routes}} from '@angular/router'; import {{Screen as Eager}} from './screen-a'; import {{Token}} from './left'; {imports} export const routes:Routes=[{{path:'controls',component:Eager,loadComponent:()=>import('./screen-b').then(m=>m.Screen),loadChildren:()=>import('./feature-a').then(m=>m.Screen),providers:[Token]}},{{path:'same',canActivate:[{a},{b},{a}],providers:[Token]}},{{path:'same',canActivate:[{b}]}},{{path:'same',canActivate:[{a}]}},{{path:'',canActivate:[{a}]}},{{path:'',canActivate:[{b}]}},{{path:'parent',children:[{{path:'child',canActivate:[{a}]}},{{path:'child',canActivate:[{b}]}}]}}];"
    );
    for owner in ["one.ts", "two.ts"] {
        std::fs::write(p.join(owner), &body).unwrap();
    }
    (root, config)
}
fn observe(state: &crate::mcp::McpState, p: &std::path::Path) -> Value {
    let forward:Vec<_>=["same","parent/child"].into_iter().map(|name|call(state,p,"workspace_query",json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}))).collect();
    let reverse = call(
        state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":"Guard","name":"Gate"}),
    );
    json!({"forward":forward,"reverse":reverse})
}
fn membership(
    a: &TypeScriptAuthority,
    v: &ExpressionFacet,
    objects: &mut usize,
    slots: &mut usize,
    observed: &Value,
) {
    assert!(
        a.admits_demanded_structure(&a.source, &a.source_hash, v),
        "stage=route-object-structure-prerequisite public={observed}"
    );
    match &v.value {
        ExpressionValue::Array(values) => {
            for v in values {
                membership(a, v, objects, slots, observed);
            }
        }
        ExpressionValue::Object(entries) => {
            if entries.iter().any(|(k, _)| k == "component") {
                return;
            }
            if let Some((_, children)) = entries.iter().find(|(k, _)| k == "children") {
                membership(a, children, objects, slots, observed);
                return;
            }
            let fields: Vec<_> = entries.iter().filter(|(k, _)| k == "canActivate").collect();
            assert_eq!(
                fields.len(),
                1,
                "stage=canActivate-field-prerequisite object={}..{} public={observed}",
                v.start,
                v.end
            );
            let array = &fields[0].1;
            assert!(
                v.start <= array.start && array.start < array.end && array.end <= v.end,
                "stage=guard-field-containment"
            );
            assert!(
                a.admits_demanded_structure(&a.source, &a.source_hash, array),
                "stage=canActivate-array-prerequisite public={observed}"
            );
            let ExpressionValue::Array(values) = &array.value else {
                panic!("stage=canActivate-array-prerequisite {array:?}")
            };
            for element in values {
                assert!(
                    array.start <= element.start
                        && element.start < element.end
                        && element.end <= array.end,
                    "stage=guard-element-containment"
                );
                let ExpressionValue::Reference(Resolution::Resolved(target)) = &element.value
                else {
                    panic!("stage=guard-reference-prerequisite {element:?}; public={observed}")
                };
                assert_eq!(
                    a.demanded_reference_at(&a.source, &a.source_hash, element.start, element.end),
                    Some(target),
                    "stage=exact-guard-reference-prerequisite public={observed}"
                );
                assert!(
                    !target.source_hash.is_empty(),
                    "stage=guard-target-source-prerequisite"
                );
                *slots += 1;
            }
            *objects += 1;
        }
        _ => panic!("stage=guard-container-prerequisite {v:?}"),
    }
}
fn question(form: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture(form);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    // Independent runtime identity controls; these do not prove Array membership
    // or assign an intrinsic Angular Guard role.
    for file in if form == "local" {
        ["one.ts", "two.ts"]
    } else {
        ["guard-a.ts", "guard-b.ts"]
    } {
        let a = capture(&state, p, file);
        let id = a
            .declarations
            .iter()
            .find(|id| id.name == "Gate")
            .expect("stage=independent-runtime-target-prerequisite");
        assert_eq!(id.source, a.source);
        assert_eq!(id.source_hash, a.source_hash);
        assert_eq!(
            id.kind,
            if form == "local" {
                "function_declaration"
            } else {
                "variable_declarator"
            }
        );
    }
    let mut authorities = Vec::new();
    for owner in ["one.ts", "two.ts"] {
        let a = capture(&state, p, owner);
        assert_eq!(
            crate::angular_meta::route_components::project(&a).len(),
            1,
            "stage=eager-control"
        );
        assert_eq!(
            crate::angular_meta::route_lazy_components::project(&a).len(),
            1,
            "stage=loadComponent-control"
        );
        assert_eq!(
            crate::angular_meta::route_load_children::project(&a).len(),
            1,
            "stage=LoadsChildren-control"
        );
        let providers = crate::angular_meta::route_providers::project(&a);
        let registrations: Vec<_> = providers
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .collect();
        assert_eq!(registrations.len(), 2, "stage=provider-control");
        assert!(
            registrations.iter().all(|e| matches!(
                e.subject.canonical,
                Some(CanonicalEndpoint::RouteOccurrence { .. })
            )),
            "stage=native-root-object-control"
        );
        authorities.push(a);
    }
    let observed = observe(&state, p);
    eprintln!("route-guard-observed form={form} public={observed}");
    for a in &authorities {
        let root = a
            .declaration_facets
            .iter()
            .find(|f| f.declaration.source == a.source && f.declaration.name == "routes")
            .unwrap();
        let mut objects = 0;
        let mut slots = 0;
        membership(
            a,
            root.initializer.as_ref().unwrap(),
            &mut objects,
            &mut slots,
            &observed,
        );
        assert_eq!(objects, 7);
        assert_eq!(slots, 9);
    }
    // Acquisition probe only. Nine slots per root are eight distinct object/target
    // pairs; no public multiplicity or target-eligibility policy is assumed here.
}
#[test]
fn issue_162_route_guard_direct_prerequisites() {
    question("direct");
}
#[test]
fn issue_162_route_guard_alias_prerequisites() {
    question("alias");
}
#[test]
fn issue_162_route_guard_barrel_prerequisites() {
    question("barrel");
}
#[test]
fn issue_162_route_guard_namespace_prerequisites() {
    question("namespace");
}
#[test]
fn issue_162_route_guard_local_prerequisites() {
    question("local");
}
#[test]
fn issue_162_route_guard_unsupported_and_root_observations() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    std::fs::write(p.join("guard-values.ts"),"export const NotCallable=123; export class PlainClass {} export function Ordinary(){return true;}").unwrap();
    std::fs::write(
        p.join("guard-ambiguous.ts"),
        "export * from './guard-a'; export * from './guard-b';",
    )
    .unwrap();
    for (prefix, decl, object, registrations) in [
        (
            "import type {Routes} from '@angular/router';",
            "const routes:Routes",
            "{path:'refused',canActivate:[Missing],providers:[Token]}",
            1,
        ),
        (
            "import type {Routes} from '@angular/router';",
            "const routes:Routes",
            "{path:'refused',canActivate:[A,...other],providers:[Token]}",
            1,
        ),
        (
            "import type {Routes} from '@angular/router';",
            "const routes:Routes",
            "{path:'refused',canActivate:[A],canActivate:[B],providers:[Token]}",
            1,
        ),
        (
            "import type {Routes} from '@angular/router';",
            "let routes:Routes",
            "{path:'refused',canActivate:[A],providers:[Token]}",
            0,
        ),
        (
            "import {Component} from '@angular/core'; @Component({}) class Anchor{} type Routes=unknown[];",
            "const routes:Routes",
            "{path:'refused',canActivate:[A],providers:[Token]}",
            0,
        ),
        (
            "import type {Routes} from '@angular/router';",
            "const routes",
            "{path:'refused',canActivate:[A],providers:[Token]}",
            0,
        ),
    ] {
        std::fs::write(p.join("guard-refused.ts"),format!("{prefix} import {{Gate as A}} from './guard-a'; import {{Gate as B}} from './guard-b'; import {{Token}} from './left'; export {decl}=[{object}];")).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "guard-refused.ts");
        assert_eq!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                .count(),
            registrations
        );
        let observed = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"refused"}),
        );
        eprintln!(
            "route-guard-refusal-observed declaration={decl} object={object} public={observed}"
        );
    }
    // Observe target categories and type-only/ambiguous references without imposing
    // a new eligibility or canonical refusal policy on the legacy producer.
    std::fs::write(p.join("guard-categories.ts"),"import type {Routes} from '@angular/router'; import {NotCallable,PlainClass,Ordinary} from './guard-values'; import type {Gate as TypeOnly} from './guard-a'; import {Gate as Ambiguous} from './guard-ambiguous'; import {Token} from './left'; export const routes:Routes=[{path:'categories',canActivate:[NotCallable,PlainClass,Ordinary,TypeOnly,Ambiguous],providers:[Token]}];").unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "guard-categories.ts");
    assert_eq!(
        crate::angular_meta::route_providers::project(&a)
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .count(),
        1
    );
    let observed = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"categories"}),
    );
    eprintln!("route-guard-categories-observed public={observed}");
}
