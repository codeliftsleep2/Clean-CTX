//! Generation-76 written-use eligibility, exact replay and selective compatibility.
use super::typescript_route_guard_field_support::{call, capture, retained, setup};
use crate::ir::typescript_resolution::{TypeScriptAuthority, demanded_occurrences::DemandedValue};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use serde_json::{Value, json};

pub(super) fn guards(a: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    crate::angular_meta::route_guards::project(a)
}
pub(super) fn answer(state: &crate::mcp::McpState, p: &std::path::Path, name: &str) -> Value {
    call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}),
    )
}
pub(super) fn count(answer: &Value) -> usize {
    answer["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "GuardedBy")
        .count()
}
#[test]
fn issue_162_guard_controls_exact_admission_forged_crossed_and_missing_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    let edges = guards(&a);
    assert_eq!(edges.len(), 8);
    assert!(a.admits_relation_evidence(&edges));
    let roundtrip: TypeScriptAuthority =
        serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
    assert!(roundtrip.admits_relation_evidence(&edges));
    for mode in 0..10 {
        let mut edge = edges[3].clone();
        match mode {
            0 => edge.object = edges[1].object.clone(), // B is valid elsewhere, absent here.
            1 => edge.object.file = edge.subject.file.clone(),
            2 => edge.subject.name = "borrowed".into(),
            3 => edge.object.name = "borrowed".into(),
            4 => edge.subject.canonical = None,
            5 => edge.object.canonical = None,
            6 => edge.object.entity_type = "Service",
            7 => edge.layer = "typescript",
            8 => {
                let Some(CanonicalEndpoint::RouteOccurrence { route_occurrence }) =
                    &mut edge.subject.canonical
                else {
                    panic!()
                };
                route_occurrence.start += 1;
            }
            _ => {
                let Some(CanonicalEndpoint::RouteOccurrence { route_occurrence }) =
                    &mut edge.subject.canonical
                else {
                    panic!()
                };
                route_occurrence.root.source_hash.push('x');
            }
        }
        assert!(
            !a.admits_relation_evidence(&[edge]),
            "forged endpoint {mode}"
        );
    }
    let fields = retained(&a);
    let slot = &fields[2].elements[0];
    for mode in 0..9 {
        let mut bad = a.clone();
        match mode {
            0 => bad.demanded_occurrences.clear(),
            1 => bad.reference_sites.clear(),
            2 => bad.source_hash.push('x'),
            3 => bad.declaration_facets.clear(),
            4 => bad.witnesses.clear(),
            5 => bad
                .demanded_occurrences
                .retain(|s| !(s.source == a.source && s.start == slot.start && s.end == slot.end)),
            6 => {
                let site = bad
                    .demanded_occurrences
                    .iter()
                    .find(|s| s.source == a.source && s.start == slot.start && s.end == slot.end)
                    .unwrap()
                    .clone();
                bad.demanded_occurrences.push(site);
            }
            7 => {
                let site = bad
                    .reference_sites
                    .iter()
                    .find(|(s, e, _)| *s == slot.start && *e == slot.end)
                    .unwrap()
                    .clone();
                bad.reference_sites.push(site);
            }
            _ => {
                let site = bad
                    .demanded_occurrences
                    .iter_mut()
                    .find(|s| s.source == a.source && s.start == slot.start && s.end == slot.end)
                    .unwrap();
                site.source_hash.push('x');
            }
        }
        assert!(
            !bad.admits_relation_evidence(&edges[3..4]),
            "missing/crossed retained proof {mode}"
        );
    }
    // Changing only a retained field name or slot container cannot borrow an endpoint.
    let mut bad = a.clone();
    for facet in &mut bad.declaration_facets {
        if facet.declaration.source == a.source && facet.declaration.name == "routes" {
            let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Array(
                objects,
            ) = &mut facet.initializer.as_mut().unwrap().value
            else {
                panic!()
            };
            let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Object(
                fields,
            ) = &mut objects[3].value
            else {
                panic!()
            };
            fields
                .iter_mut()
                .find(|(name, _)| name == "canActivate")
                .unwrap()
                .0 = "canLoad".into();
        }
    }
    assert!(!bad.admits_relation_evidence(&edges[3..4]));
    // Root proof cannot be downgraded to a generic canonical endpoint pair.
    let mut bad = a.clone();
    for site in &mut bad.demanded_occurrences {
        if let DemandedValue::Declaration { is_const, .. } = &mut site.value {
            *is_const = false;
        }
    }
    assert!(!bad.admits_relation_evidence(&edges));
}
#[test]
fn issue_162_guard_controls_broad_runtime_values_partial_slots_and_no_fallback() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(
        p.join("values.ts"),
        "export const Numeric=123; export class Plain {} export function Ordinary(){return true;}",
    )
    .unwrap();
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from './guard-a'; export * from './guard-b';",
    )
    .unwrap();
    std::fs::write(p.join("one.ts"),"import type {Routes} from '@angular/router'; import {Gate as A} from './guard-a'; import type {Gate as TypeOnly} from './guard-b'; import {Gate as Ambiguous} from './ambiguous'; import {Numeric,Plain,Ordinary} from './values'; import {Token} from './left'; export const routes:Routes=[{path:'values',canActivate:[A,Numeric,Plain,Ordinary,Missing,TypeOnly,Ambiguous,A],providers:[Token]},{path:'duplicate',canActivate:[A],canActivate:[Numeric],providers:[Token]},{path:'unsupported',canActivate:()=>A,providers:[Token]}];").unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    let edges = guards(&a);
    assert_eq!(edges.len(), 4);
    assert!(a.admits_relation_evidence(&edges));
    let names = edges
        .iter()
        .map(|e| e.object.name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        names,
        std::collections::BTreeSet::from(["Gate", "Numeric", "Ordinary", "Plain"])
    );
    assert_eq!(count(&answer(&state, p, "values")), 4);
    assert_eq!(count(&answer(&state, p, "duplicate")), 0);
    assert_eq!(count(&answer(&state, p, "unsupported")), 0);
    assert_eq!(
        crate::angular_meta::route_providers::project(&a)
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .count(),
        3
    );
}
#[test]
fn issue_162_guard_controls_other_kinds_same_target_and_outside_legacy() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(p.join("one.ts"),"import type {Routes} from '@angular/router'; import {Gate as A} from './guard-a'; import {Gate as B} from './guard-b'; export const routes:Routes=[{path:'mixed',canActivate:[A,B,A],canLoad:[A],canDeactivate:[A],canMatch:[B]}];").unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    let a = capture(&state, p, "one.ts");
    assert_eq!(guards(&a).len(), 2);
    let alias = state
        .alias_for_path(&p.join("one.ts").to_string_lossy())
        .unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let rows = snapshot
        .edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::GuardedBy)
        .collect::<Vec<_>>();
    assert_eq!(
        rows.iter()
            .filter(|e| e.subject.canonical.is_some())
            .count(),
        2
    );
    assert_eq!(
        rows.iter()
            .filter(|e| e.subject.canonical.is_none())
            .count(),
        3,
        "all three other-kind contributions survive, even equal target labels"
    );
    // Index support dedup is independent of producer preservation of other kinds.
    assert_eq!(count(&answer(&state, p, "mixed")), 4);
    drop(state);
    // Legacy root discovery tests text in the declaration itself; import text
    // elsewhere does not make an untyped or differently named annotation a root.
    for (declaration, wanted) in [
        ("let routes:Routes", 1),
        ("const routes", 0),
        ("const routes:Foreign", 0),
    ] {
        std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Gate}} from './guard-a'; type Foreign=unknown[]; export {declaration}=[{{path:'legacy',canActivate:[Gate]}}];")).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "one.ts");
        assert!(guards(&a).is_empty());
        let observed = answer(&state, p, "legacy");
        let source = std::fs::read_to_string(p.join("one.ts")).unwrap();
        let legacy = crate::angular_meta::routing::extract_route_shape(
            &source,
            crate::compression::Fidelity::Low,
        )
        .map(|shape| {
            shape
                .to_semantic_edges()
                .iter()
                .filter(|e| e.relation == SemanticRelation::GuardedBy)
                .count()
        })
        .unwrap_or(0);
        assert_eq!(legacy, wanted, "unmodified legacy emission: {declaration}");
        assert_eq!(count(&observed), wanted, "outside legacy: {declaration}");
    }
}
#[test]
fn issue_162_guard_controls_unsupported_slots_shadowing_and_independent_objects() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(p.join("one.ts"),"import type {Routes} from '@angular/router'; import {Gate as A} from './guard-a'; import * as ns from './guard-b'; const extra=[]; const Gate=42; export const routes:Routes=[{path:'partial',canActivate:[A,...extra,ns['Gate'],()=>true,Gate]},{path:'sibling',canActivate:[A]},{path:'spread',...extra,canActivate:[A]}];").unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    let edges = guards(&a);
    assert_eq!(
        edges.len(),
        3,
        "two exact partial slots and an independent object; spread object stays outside ownership"
    );
    assert!(
        edges
            .iter()
            .any(|e| e.object.name == "Gate" && e.object.file.as_deref() == a.source.to_str()),
        "local binding must not become imported Gate"
    );
    assert_eq!(count(&answer(&state, p, "partial")), 2);
    assert_eq!(count(&answer(&state, p, "sibling")), 1);
    assert_eq!(
        count(&answer(&state, p, "spread")),
        1,
        "outside-envelope legacy remains"
    );
}
