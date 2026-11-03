//! Generation-78 exact unqualified written resolver use and refusal policy.
pub(super) use super::angular_route_resolver_semantic_tests::fixture;
pub(super) use super::typescript_route_guard_field_support::{call, capture};
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
pub(super) fn answer(s: &crate::mcp::McpState, p: &std::path::Path, name: &str) -> Value {
    call(
        s,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}),
    )
}
pub(super) fn count(v: &Value) -> usize {
    v["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "ResolvedBy")
        .count()
}
pub(super) fn write_routes(p: &std::path::Path, body: &str) {
    let source = std::fs::read_to_string(p.join("one.ts")).unwrap();
    let prefix = source.split("export const routes:Routes=").next().unwrap();
    std::fs::write(
        p.join("one.ts"),
        format!("{prefix} export const routes:Routes=[{body}];"),
    )
    .unwrap();
}
#[test]
fn issue_162_resolver78_controls_exact_admission_forged_and_borrowed() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    let edges = crate::angular_meta::route_resolvers::project(&a);
    assert_eq!(edges.len(), 8);
    assert!(a.admits_relation_evidence(&edges));
    let restored: crate::ir::typescript_resolution::TypeScriptAuthority =
        serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
    assert!(restored.admits_relation_evidence(&edges));
    for mode in 0..10 {
        let mut b = a.clone();
        let mut edge = edges[2].clone();
        match mode {
            0 => edge.object = edges[0].object.clone(),
            1 => edge.object.file = edge.subject.file.clone(),
            2 => edge.subject.name = "borrowed".into(),
            3 => edge.object.name = "borrowed".into(),
            4 => edge.subject.canonical = None,
            5 => edge.object.canonical = None,
            6 => edge.layer = "typescript",
            7 => b.resolve_mappings.clear(),
            8 => b.declaration_facets.clear(),
            _ => b.capture_current = false,
        }
        assert!(
            !b.admits_relation_evidence(&[edge]),
            "forged membership mode {mode}"
        );
    }
    let mut duplicate = a.clone();
    duplicate
        .resolve_mappings
        .push(duplicate.resolve_mappings[0].clone());
    assert!(
        crate::angular_meta::route_resolvers::project(&duplicate)
            .iter()
            .any(|e| e.subject.canonical == edges[0].subject.canonical
                && e.object.canonical == edges[0].object.canonical)
    );
    let mut cross = crate::angular_meta::route_guards::project(&a)[0].clone();
    cross.relation = SemanticRelation::ResolvedBy;
    cross.object.entity_type = "Resolver";
    assert!(
        !a.admits_relation_evidence(&[cross]),
        "guard endpoint cannot prove resolve membership"
    );
}
#[test]
fn issue_162_resolver78_controls_broad_partial_duplicate_keys_and_owned_no_fallback() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    std::fs::write(p.join("values.ts"),"export const Numeric=42;export class Plain{}export function Ordinary(){}export const Typed:unknown=1;").unwrap();
    write_routes(
        p,
        "{path:'partial',resolve:{num:Numeric,plain:Plain,ordinary:Ordinary,typed:Typed,missing:Missing,typeonly:T}},{path:'duplicate-key',resolve:{dup:Gate,'d\\u0075p':Right,good:Right}},{path:'unsupported-duplicate',resolve:{dup:Gate,dup:()=>Right,good:Right}},{path:'outer',resolve:{a:Gate},resolve:{b:Right}},{path:'unsupported',resolve:{a:Gate,...Other}},{path:'repeat',resolve:{a:Gate,b:Gate}},{path:'sibling',resolve:{bad:Missing},canActivate:[Gate],canMatch:[Gate],canLoad:[Gate],canDeactivate:[Gate],canActivateChild:[Gate],component:Eager,loadComponent:()=>import('./screen-b').then(m=>m.Screen),loadChildren:()=>import('./feature-a').then(m=>m.Screen),providers:[Token]}",
    );
    let owner = p.join("one.ts");
    let source = std::fs::read_to_string(&owner).unwrap();
    std::fs::write(owner,format!("import {{Numeric,Plain,Ordinary,Typed}} from './values';import type {{Gate as T}} from './guard-b';{source}")).unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    for (name, n) in [
        ("partial", 4),
        ("duplicate-key", 1),
        ("unsupported-duplicate", 1),
        ("outer", 0),
        ("unsupported", 0),
        ("repeat", 1),
        ("sibling", 0),
    ] {
        assert_eq!(count(&answer(&state, p, name)), n, "{name}");
    }
    let expected = crate::angular_meta::route_resolvers::project(&a);
    assert_eq!(expected.len(), 7);
    assert!(a.admits_relation_evidence(&expected));
    let sibling = answer(&state, p, "sibling");
    for relation in [
        "GuardedBy",
        "GuardedByCanMatch",
        "GuardedByCanLoad",
        "GuardedByCanDeactivate",
        "GuardedByCanActivateChild",
        "LoadsChildren",
        "RegistersProvider",
    ] {
        assert_eq!(
            sibling["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["relation"] == relation)
                .count(),
            1,
            "independent {relation}"
        );
    }
    assert_eq!(
        sibling["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "RouteMapsTo")
            .count(),
        2
    );
}
#[test]
fn issue_162_resolver78_controls_outside_legacy_and_key_support_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    std::fs::write(p.join("legacy.ts"),"import type {Routes} from '@angular/router';import {Gate} from './guard-a';export let routes:Routes=[{path:'legacy',resolve:{data:Gate}}];").unwrap();
    write_routes(
        p,
        "{path:'key-support',resolve:{a:Gate,b:Gate}},{path:'kind',resolve:{a:Gate},canActivate:[Gate]}",
    );
    let original = std::fs::read_to_string(p.join("one.ts")).unwrap();
    let state = crate::mcp::McpState::new(config);
    assert_eq!(count(&answer(&state, p, "legacy")), 1);
    assert_eq!(count(&answer(&state, p, "key-support")), 1);
    for (from, to, wanted) in [
        ("a:Gate,b:Gate", "a:Gate", 1),
        ("a:Gate,b:Gate", "", 0),
        (
            "resolve:{a:Gate},canActivate",
            "resolve:{a:Missing},canActivate",
            1,
        ),
    ] {
        std::fs::write(p.join("one.ts"), original.replace(from, to)).unwrap();
        assert_eq!(count(&answer(&state, p, "key-support")), wanted);
        let kind = answer(&state, p, "kind");
        assert_eq!(
            count(&kind),
            if from.starts_with("resolve:") { 0 } else { 1 }
        );
        assert_eq!(
            kind["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["relation"] == "GuardedBy")
                .count(),
            1
        );
    }
    std::fs::write(p.join("one.ts"), original).unwrap();
    assert_eq!(count(&answer(&state, p, "key-support")), 1);
}
