//! Generation-75 exact LoadsChildren proof, refusal and independent compatibility.
use super::typescript_children_selection_support::{call, capture, rewrite};
use super::typescript_shared_selection_roles_tests::setup_role;
use crate::ir::typescript_resolution::{TypeScriptAuthority, demanded_occurrences::DemandedValue};
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use serde_json::{Value, json};
fn children(a: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    crate::angular_meta::route_load_children::project(a)
}
fn answer(state: &crate::mcp::McpState, p: &std::path::Path, name: &str) -> Value {
    call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}),
    )
}
fn count(v: &Value, relation: &str) -> usize {
    v["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == relation)
        .count()
}
#[test]
fn issue_162_children_controls_forged_selection_and_separate_target_roles() {
    let _serial = crate::protocol::handler_response_serial();
    for role in ["module", "routes"] {
        let (root, config) = setup_role("direct", role);
        let p = root.path();
        let state = crate::mcp::McpState::new(config);
        let a = capture(&state, p, "one.ts");
        let edges = children(&a);
        assert_eq!(edges.len(), 7);
        assert!(a.admits_relation_evidence(&edges));
        for mode in 0..9 {
            let mut edge = edges[0].clone();
            match mode {
                0 => edge.object = edges[1].object.clone(),
                1 => edge.subject.canonical = edges[1].subject.canonical.clone(),
                2 => edge.object.file = edge.subject.file.clone(),
                3 => edge.subject.name = "borrowed".into(),
                4 => {
                    edge.object.entity_type = if role == "module" {
                        "RouteConfig"
                    } else {
                        "Module"
                    }
                }
                5 => edge.subject.canonical = None,
                6 => edge.object.canonical = None,
                7 => edge.layer = "typescript",
                _ => edge.object.name = "borrowed".into(),
            }
            assert!(!a.admits_relation_evidence(&[edge]), "forged {role}/{mode}");
        }
        let target = a.arrow_selections[0].target.clone();
        for mode in 0..7 {
            let mut bad = a.clone();
            match mode {
                0 => bad.arrow_selections.clear(),
                1 => bad.source_hash.push('x'),
                2 => bad.demanded_occurrences.clear(),
                3 => bad.declaration_facets.retain(|f| f.declaration != target),
                4 => bad.arrow_selections[0].target = bad.arrow_selections[1].target.clone(),
                5 => bad.module_requests.clear(),
                _ => bad.package_exports.clear(),
            }
            assert!(
                !bad.admits_relation_evidence(&edges[..1]),
                "missing/native {role}/{mode}"
            );
        }
        if role == "routes" {
            for mode in 0..3 {
                let mut bad = a.clone();
                if mode == 0 {
                    for site in &mut bad.demanded_occurrences {
                        if let DemandedValue::Declaration { id, is_const, .. } = &mut site.value
                            && *id == target
                        {
                            *is_const = false;
                        }
                    }
                } else {
                    let facet = bad
                        .declaration_facets
                        .iter_mut()
                        .find(|f| f.declaration == target)
                        .unwrap();
                    if mode == 1 {
                        facet.type_annotation = None;
                    } else {
                        facet.initializer = None;
                    }
                }
                assert!(
                    !bad.admits_relation_evidence(&edges[..1]),
                    "Routes declaration proof {mode}"
                );
            }
        }
    }
}
#[test]
fn issue_162_children_controls_owned_refusals_never_legacy_fallback() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup_role("direct", "module");
    let p = root.path();
    for (file, text) in [
        ("plain.ts", "export class Screen {}"),
        (
            "foreign-module.ts",
            "function NgModule(_:unknown){return (_:unknown)=>{};} @NgModule({}) export class Screen {}",
        ),
        (
            "competing.ts",
            "import {NgModule,Component} from '@angular/core'; @NgModule({}) @Component({}) export class Screen {}",
        ),
        ("untyped.ts", "export const Screen=[];"),
        (
            "mutable.ts",
            "import {Routes} from '@angular/router'; export let Screen:Routes=[];",
        ),
        (
            "foreign.ts",
            "type Routes=unknown[]; export const Screen:Routes=[];",
        ),
        (
            "private.ts",
            "import {NgModule} from '@angular/core'; @NgModule({}) class Screen {}",
        ),
        ("type-only.ts", "export type {Screen} from './feature-a';"),
        (
            "ambiguous.ts",
            "export * from './feature-a'; export * from './feature-b';",
        ),
    ] {
        std::fs::write(p.join(file), text).unwrap();
    }
    for callback in [
        "()=>import('./feature-a').then(m=>other.Screen)",
        "()=>import('./feature-a').then(m=>m['Screen'])",
        "()=>import(specifier).then(m=>m.Screen)",
        "async()=>import('./feature-a').then(m=>m.Screen)",
        "()=>flag?import('./feature-a').then(m=>m.Screen):other",
        "()=>import('./missing').then(m=>m.Screen)",
        "()=>import('./feature-a').then(m=>m.Missing)",
        "()=>import('./plain').then(m=>m.Screen)",
        "()=>import('./foreign-module').then(m=>m.Screen)",
        "()=>import('./competing').then(m=>m.Screen)",
        "()=>import('./untyped').then(m=>m.Screen)",
        "()=>import('./mutable').then(m=>m.Screen)",
        "()=>import('./foreign').then(m=>m.Screen)",
        "()=>import('./private').then(m=>m.Screen)",
        "()=>import('./type-only').then(m=>m.Screen)",
        "()=>import('./ambiguous').then(m=>m.Screen)",
        "()=>import('./feature-a').then(m=>m.Screen),loadChildren:()=>import('./feature-b').then(m=>m.Screen)",
    ] {
        rewrite(p, callback);
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "one.ts");
        assert!(children(&a).is_empty(), "{callback}");
        assert_eq!(
            count(&answer(&state, p, "lazy"), "LoadsChildren"),
            0,
            "owned fallback: {callback}"
        );
        assert_eq!(crate::angular_meta::route_components::project(&a).len(), 1);
        assert!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .any(|e| e.relation == SemanticRelation::RegistersProvider)
        );
    }
}
#[test]
fn issue_162_children_controls_empty_provider_free_config_and_native_module() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup_role("direct", "routes");
    let p = root.path();
    for target in [
        "import {Routes} from '@angular/router'; export const Screen:Routes=[];",
        "import {Routes} from '@angular/router'; export const Screen:Routes=[{path:'empty'}];",
        "import {NgModule} from '@angular/core'; @NgModule({}) export class Screen {}",
    ] {
        std::fs::write(p.join("feature-a.ts"), target).unwrap();
        rewrite(p, "()=>import('./feature-a').then(m=>m.Screen)");
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "one.ts");
        let edges = children(&a);
        assert_eq!(edges.len(), 1);
        assert_eq!(
            edges[0].object.entity_type,
            if target.contains("NgModule") {
                "Module"
            } else {
                "RouteConfig"
            }
        );
        assert!(a.admits_relation_evidence(&edges));
        assert_eq!(count(&answer(&state, p, "lazy"), "LoadsChildren"), 1);
    }
}
#[test]
fn issue_162_children_controls_outside_ownership_and_independent_fields() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup_role("direct", "module");
    let p = root.path();
    for (file, decl, object) in [
        (
            "legacy-untyped.ts",
            "const legacyRoutes",
            "{path:'legacy',loadChildren:()=>import('./feature-a').then(m=>m.Screen)}",
        ),
        (
            "legacy-mutable.ts",
            "let routes:Routes",
            "{path:'legacy',loadChildren:()=>import('./feature-a').then(m=>m.Screen)}",
        ),
        (
            "legacy-spread.ts",
            "const routes:Routes",
            "{path:'legacy',...other,loadChildren:()=>import('./feature-a').then(m=>m.Screen)}",
        ),
    ] {
        std::fs::write(
            p.join(file),
            format!("import type {{Routes}} from '@angular/router'; export {decl}=[{object}];"),
        )
        .unwrap();
    }
    let state = crate::mcp::McpState::new(config);
    for file in ["legacy-untyped.ts", "legacy-mutable.ts", "legacy-spread.ts"] {
        assert!(children(&capture(&state, p, file)).is_empty());
    }
    assert_eq!(count(&answer(&state, p, "legacy"), "LoadsChildren"), 3);
    for (fields, eager, lazy, child, provider) in [
        (
            "component:A,loadComponent:()=>import('./screen-b').then(m=>m.Screen),loadChildren:()=>import('./feature-a').then(m=>m.Screen),providers:[Token]",
            1,
            1,
            1,
            1,
        ),
        (
            "loadComponent:()=>import('./screen-b').then(m=>m.Screen),loadChildren:()=>import('./feature-a').then(m=>m.Screen),providers:[Token]",
            0,
            1,
            1,
            1,
        ),
        (
            "component:A,loadChildren:()=>import('./feature-a').then(m=>m.Screen),providers:[Token]",
            1,
            0,
            1,
            1,
        ),
        (
            "component:A,loadComponent:()=>import('./screen-b').then(m=>m.Screen),providers:[Token]",
            1,
            1,
            0,
            1,
        ),
        (
            "component:A,loadComponent:()=>import('./screen-b').then(m=>m.Screen),loadChildren:()=>import('./feature-a').then(m=>m.Screen)",
            1,
            1,
            1,
            0,
        ),
        (
            "component:A,loadComponent:()=>import('./screen-b').then(m=>m.Screen),loadChildren:async()=>import('./feature-a').then(m=>m.Screen),providers:[Token]",
            1,
            1,
            0,
            1,
        ),
    ] {
        std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Screen as A}} from './screen-a'; import {{Token}} from './left'; export const routes:Routes=[{{path:'mixed',{fields}}}];")).unwrap();
        let a = capture(&state, p, "one.ts");
        assert_eq!(
            crate::angular_meta::route_components::project(&a).len(),
            eager
        );
        assert_eq!(
            crate::angular_meta::route_lazy_components::project(&a).len(),
            lazy
        );
        assert_eq!(children(&a).len(), child);
        assert_eq!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                .count(),
            provider
        );
        let public = answer(&state, p, "mixed");
        assert_eq!(count(&public, "RouteMapsTo"), eager + lazy);
        assert_eq!(count(&public, "LoadsChildren"), child);
    }
}

#[test]
fn issue_162_children_controls_object_child_field_and_empty_root_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for role in ["module", "routes"] {
        let (root, config) = setup_role("direct", role);
        let p = root.path();
        let state = crate::mcp::McpState::new(config);
        let object = "{path:'exit',loadChildren:()=>import('./feature-a').then(m=>m.Screen)}";
        let child = "{path:'parent',children:[{path:'child',loadChildren:()=>import('./feature-b').then(m=>m.Screen)}]}";
        let sibling = "{path:'eager',component:A,providers:[Token]}";
        for (objects, wanted, same, nested, eager) in [
            (format!("{sibling},{object},{object},{child}"), 3, 2, 1, 1),
            (format!("{sibling},{object},{child}"), 2, 1, 1, 1),
            (format!("{sibling},{object}"), 1, 1, 0, 1),
            (format!("{sibling},{{path:'exit'}}"), 0, 0, 0, 1),
            (String::new(), 0, 0, 0, 0),
            (format!("{sibling},{object},{object},{child}"), 3, 2, 1, 1),
        ] {
            std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Screen as A}} from './screen-a'; import {{Token}} from './left'; export const routes:Routes=[{objects}];")).unwrap();
            let a = capture(&state, p, "one.ts");
            assert_eq!(children(&a).len(), wanted);
            assert!(a.admits_relation_evidence(&children(&a)));
            assert_eq!(count(&answer(&state, p, "exit"), "LoadsChildren"), same);
            assert_eq!(
                count(&answer(&state, p, "parent/child"), "LoadsChildren"),
                nested
            );
            assert_eq!(
                crate::angular_meta::route_components::project(&a).len(),
                eager
            );
            assert_eq!(
                crate::angular_meta::route_providers::project(&a)
                    .iter()
                    .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                    .count(),
                eager
            );
        }
    }
}
