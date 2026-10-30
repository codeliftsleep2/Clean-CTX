//! Generation-74 production lazy proof, replacement and mixed-origin controls.
use super::typescript_lazy_selection_support::*;
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use serde_json::{Value, json};
fn lazy(a: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    crate::angular_meta::route_lazy_components::project(a)
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
fn issue_162_lazy_controls_forged_native_field_and_selection_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, root.path(), "one.ts");
    let edges = lazy(&a);
    assert_eq!(edges.len(), 7);
    assert!(a.admits_relation_evidence(&edges));
    for mode in 0..8 {
        let mut edge = edges[0].clone();
        match mode {
            0 => edge.object = edges[1].object.clone(),
            1 => edge.subject.canonical = edges[1].subject.canonical.clone(),
            2 => edge.object.file = edge.subject.file.clone(),
            3 => edge.subject.name = "borrowed".into(),
            4 => edge.object.name = "borrowed".into(),
            5 => edge.subject.canonical = None,
            6 => edge.object.canonical = None,
            _ => edge.layer = "typescript",
        }
        assert!(!a.admits_relation_evidence(&[edge]), "forged edge {mode}");
    }
    for mode in 0..6 {
        let mut bad = a.clone();
        match mode {
            0 => bad.arrow_selections.clear(),
            1 => bad.source_hash.push('x'),
            2 => bad.demanded_occurrences.clear(),
            3 => {
                let target = bad.arrow_selections[0].target.clone();
                bad.declaration_facets.retain(|f| f.declaration != target);
            }
            4 => bad.arrow_selections[0].target = bad.arrow_selections[1].target.clone(),
            _ => bad.module_requests.clear(),
        }
        assert!(
            !bad.admits_relation_evidence(&edges[..1]),
            "missing/crossed/native {mode}"
        );
    }
    assert!(
        !crate::mcp::semantic_compilation::retained::RetainedAuthority::default()
            .admits_relation_evidence(&edges)
    );
}
#[test]
fn issue_162_lazy_controls_owned_unsupported_and_duplicate_no_weak_fallback() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(p.join("ordinary.ts"), "export class Screen {}").unwrap();
    std::fs::write(
        p.join("private.ts"),
        "import {Component} from '@angular/core'; @Component({}) class Screen {}",
    )
    .unwrap();
    std::fs::write(
        p.join("type-only.ts"),
        "export type {Screen} from './screen-a';",
    )
    .unwrap();
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from './screen-a'; export * from './screen-b';",
    )
    .unwrap();
    for callback in [
        "()=>import('./screen-a').then(m=>other.Screen)",
        "()=>import('./screen-a').then(m=>m['Screen'])",
        "()=>import('./screen-a').then((m,x)=>m.Screen)",
        "async()=>import('./screen-a').then(m=>m.Screen)",
        "()=>{return import('./screen-a').then(m=>m.Screen);}",
        "()=>import('./ordinary').then(m=>m.Screen)",
        "()=>import('./private').then(m=>m.Screen)",
        "()=>import('./type-only').then(m=>m.Screen)",
        "()=>import('./ambiguous').then(m=>m.Screen)",
        "()=>import('./missing').then(m=>m.Screen)",
        "()=>import('./screen-a').then(m=>m.Missing)",
        "()=>import('./screen-a').then(m=>m.Screen),loadComponent:()=>import('./screen-b').then(m=>m.Screen)",
    ] {
        rewrite(p, callback);
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "one.ts");
        assert!(lazy(&a).is_empty(), "{callback}");
        assert_eq!(
            count(&answer(&state, p, "lazy"), "RouteMapsTo"),
            0,
            "no owned legacy fallback: {callback}"
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
fn issue_162_lazy_controls_outside_ownership_and_load_children_preserved() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    for (file, declaration, object) in [
        (
            "untyped.ts",
            "const legacyRoutes",
            "{path:'legacy',loadComponent:()=>import('./screen-a').then(m=>m.Screen)}",
        ),
        (
            "mutable.ts",
            "let routes:Routes",
            "{path:'legacy',loadComponent:()=>import('./screen-a').then(m=>m.Screen)}",
        ),
        (
            "spread.ts",
            "const routes:Routes",
            "{path:'legacy',...other,loadComponent:()=>import('./screen-a').then(m=>m.Screen)}",
        ),
    ] {
        std::fs::write(
            p.join(file),
            format!(
                "import type {{Routes}} from '@angular/router'; export {declaration}=[{object}];"
            ),
        )
        .unwrap();
    }
    std::fs::write(
        p.join("child-routes.ts"),
        "import {Routes} from '@angular/router'; export const childRoutes:Routes=[];",
    )
    .unwrap();
    std::fs::write(p.join("children.ts"),"import type {Routes} from '@angular/router'; export const routes:Routes=[{path:'children',loadChildren:()=>import('./child-routes').then(m=>m.childRoutes)}];").unwrap();
    let state = crate::mcp::McpState::new(config);
    for file in ["untyped.ts", "mutable.ts", "spread.ts"] {
        assert!(lazy(&capture(&state, p, file)).is_empty());
    }
    assert_eq!(count(&answer(&state, p, "legacy"), "RouteMapsTo"), 3);
    assert_eq!(count(&answer(&state, p, "children"), "LoadsChildren"), 1);
}
#[test]
fn issue_162_lazy_controls_mixed_eager_lazy_provider_independent_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    for (fields, eager, lazy_count, public, providers) in [
        (
            "component:A,loadComponent:()=>import('./screen-b').then(m=>m.Screen),providers:[Token]",
            1,
            1,
            2,
            1,
        ),
        (
            "loadComponent:()=>import('./screen-b').then(m=>m.Screen),providers:[Token]",
            0,
            1,
            1,
            1,
        ),
        ("component:A,providers:[Token]", 1, 0, 1, 1),
        (
            "component:A,loadComponent:()=>import('./screen-b').then(m=>m.Screen)",
            1,
            1,
            2,
            0,
        ),
        (
            "component:A,loadComponent:()=>import('./screen-a').then(m=>m.Screen),providers:[Token]",
            1,
            1,
            1,
            1,
        ),
        (
            "component:A,loadComponent:async()=>import('./screen-a').then(m=>m.Screen),providers:[Token]",
            1,
            0,
            1,
            1,
        ),
    ] {
        std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Screen as A}} from './screen-a'; import {{Token}} from './left'; export const routes:Routes=[{{path:'mixed',{fields}}}];")).unwrap();
        let a = capture(&state, p, "one.ts");
        assert_eq!(
            crate::angular_meta::route_components::project(&a).len(),
            eager
        );
        assert_eq!(lazy(&a).len(), lazy_count);
        assert_eq!(
            count(&answer(&state, p, "mixed"), "RouteMapsTo"),
            public,
            "one canonical relation per exact object/target; independent eager/lazy supports"
        );
        assert_eq!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                .count(),
            providers
        );
    }
}
