//! Generation-77 independent relation/field replay, no umbrella GuardedBy.
use super::typescript_route_guard_field_support::{call, capture, retained_field, setup};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use serde_json::{Value, json};
pub(super) const KINDS: [(&str, SemanticRelation, &str); 4] = [
    (
        "canMatch",
        SemanticRelation::GuardedByCanMatch,
        "GuardedByCanMatch",
    ),
    (
        "canDeactivate",
        SemanticRelation::GuardedByCanDeactivate,
        "GuardedByCanDeactivate",
    ),
    (
        "canLoad",
        SemanticRelation::GuardedByCanLoad,
        "GuardedByCanLoad",
    ),
    (
        "canActivateChild",
        SemanticRelation::GuardedByCanActivateChild,
        "GuardedByCanActivateChild",
    ),
];
pub(super) fn kinds(a: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    crate::angular_meta::route_guards::project_kinds(a)
}
pub(super) fn answer(state: &crate::mcp::McpState, p: &std::path::Path, name: &str) -> Value {
    call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}),
    )
}
pub(super) fn count(v: &Value, relation: &str) -> usize {
    v["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == relation)
        .count()
}

#[test]
fn issue_162_guard77_controls_cross_kind_admission_and_named_json_compatibility() {
    let _serial = crate::protocol::handler_response_serial();
    assert_eq!(
        serde_json::to_value(SemanticRelation::GuardedBy).unwrap(),
        json!("GuardedBy")
    );
    assert_eq!(SemanticRelation::GuardedBy as usize, 22);
    assert_eq!(SemanticRelation::HasConstructorParameterType as usize, 50);
    for (field, relation, name) in KINDS {
        assert_eq!(serde_json::to_value(relation).unwrap(), json!(name));
        assert_eq!(
            serde_json::from_value::<SemanticRelation>(json!(name)).unwrap(),
            relation
        );
        let (root, config) = setup("direct");
        let p = root.path();
        let source = std::fs::read_to_string(p.join("one.ts"))
            .unwrap()
            .replace("canActivate:", &format!("{field}:"))
            .replace(
                "path:'controls',component:",
                "path:'controls',canActivate:[Gate],component:",
            );
        std::fs::write(p.join("one.ts"), source).unwrap();
        let state = crate::mcp::McpState::new(config);
        let a = capture(&state, p, "one.ts");
        let edges = kinds(&a);
        assert_eq!(edges.len(), 8);
        assert!(edges.iter().all(|e| e.relation == relation));
        assert!(a.admits_relation_evidence(&edges));
        let old = crate::angular_meta::route_guards::project(&a);
        assert_eq!(old.len(), 1);
        let mut borrowed = edges[3].clone();
        borrowed.relation = SemanticRelation::GuardedBy;
        assert!(
            !a.admits_relation_evidence(&[borrowed]),
            "candidate field cannot authorize canActivate"
        );
        let restored: TypeScriptAuthority =
            serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
        assert!(restored.admits_relation_evidence(&edges));
        for (_, other, _) in KINDS {
            let mut borrowed = old[0].clone();
            borrowed.relation = other;
            assert!(
                !a.admits_relation_evidence(&[borrowed]),
                "canActivate cannot authorize {other:?}"
            );
            if other != relation {
                let mut borrowed = edges[3].clone();
                borrowed.relation = other;
                assert!(
                    !a.admits_relation_evidence(&[borrowed]),
                    "cross-kind field borrowing"
                );
            }
        }
        for mode in 0..8 {
            let mut edge = edges[3].clone();
            match mode {
                0 => edge.object = edges[1].object.clone(),
                1 => edge.object.file = edge.subject.file.clone(),
                2 => edge.subject.name = "borrowed".into(),
                3 => edge.object.name = "borrowed".into(),
                4 => edge.subject.canonical = None,
                5 => edge.object.canonical = None,
                6 => {
                    edge.subject.canonical = None;
                    edge.object.canonical = None;
                }
                _ => edge.layer = "typescript",
            }
            assert!(
                !a.admits_relation_evidence(&[edge]),
                "forged {field}/{mode}"
            );
        }
    }
}
#[test]
fn issue_162_guard77_controls_missing_competing_crossed_retained_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = super::typescript_guard_kinds_field_support::setup("direct");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    let edges = kinds(&a);
    assert_eq!(edges.len(), 32);
    for (field, relation, _) in KINDS {
        let fields = retained_field(&a, field);
        let slot = &fields[2].elements[0];
        let edge=edges.iter().find(|e|e.relation==relation&&matches!(&e.subject.canonical,Some(crate::layers::meta::canonical_endpoint::CanonicalEndpoint::RouteOccurrence{route_occurrence}) if route_occurrence.start==fields[2].object.0)).unwrap();
        for mode in 0..8 {
            let mut bad = a.clone();
            match mode {
                0 => bad.demanded_occurrences.clear(),
                1 => bad.reference_sites.clear(),
                2 => bad.source_hash.push('x'),
                3 => bad.declaration_facets.clear(),
                4 => bad.witnesses.clear(),
                5 => bad.demanded_occurrences.retain(|s| {
                    !(s.source == a.source && s.start == slot.start && s.end == slot.end)
                }),
                6 => {
                    let site = bad
                        .demanded_occurrences
                        .iter()
                        .find(|s| {
                            s.source == a.source && s.start == slot.start && s.end == slot.end
                        })
                        .unwrap()
                        .clone();
                    bad.demanded_occurrences.push(site);
                }
                _ => {
                    let site = bad
                        .reference_sites
                        .iter()
                        .find(|(s, e, _)| *s == slot.start && *e == slot.end)
                        .unwrap()
                        .clone();
                    bad.reference_sites.push(site);
                }
            }
            assert!(
                !bad.admits_relation_evidence(std::slice::from_ref(edge)),
                "retained proof {field}/{mode}"
            );
        }
    }
}
#[test]
fn issue_162_guard77_controls_partial_broad_values_duplicate_fields_and_owned_no_fallback() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(
        p.join("values.ts"),
        "export const Numeric=42; export class Plain {} export function Ordinary(){return true;}",
    )
    .unwrap();
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from './guard-a'; export * from './guard-b';",
    )
    .unwrap();
    let valid=KINDS.iter().map(|(field,_,_)|format!("{field}:[A,Numeric,Plain,Ordinary,Missing,T,Ambiguous,A,...extra,ns['Gate'],()=>true]")).collect::<Vec<_>>().join(",");
    let mixed = KINDS
        .iter()
        .map(|(field, _, _)| {
            if *field == "canMatch" {
                format!("{field}:[A],{field}:[B]")
            } else {
                format!("{field}:[B]")
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    let duplicate = KINDS
        .iter()
        .map(|(field, _, _)| format!("{field}:[A],{field}:[B]"))
        .collect::<Vec<_>>()
        .join(",");
    std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Gate as A}} from './guard-a'; import {{Gate as B}} from './guard-b'; import type {{Gate as T}} from './guard-a'; import {{Gate as Ambiguous}} from './ambiguous'; import {{Numeric,Plain,Ordinary}} from './values'; import {{Token}} from './left'; import * as ns from './guard-b'; const extra=[]; export const routes:Routes=[{{path:'values',canActivate:[A],{valid},providers:[Token]}},{{path:'duplicate',{duplicate},providers:[Token]}},{{path:'independent',canActivate:[A],{mixed},providers:[Token]}}];")).unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    assert_eq!(kinds(&a).len(), 19);
    assert!(a.admits_relation_evidence(&kinds(&a)));
    let values = answer(&state, p, "values");
    assert_eq!(count(&values, "GuardedBy"), 1);
    let refused = answer(&state, p, "duplicate");
    assert!(
        refused["edges"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| !e["relation"].as_str().unwrap().starts_with("GuardedBy")),
        "no generic weak downgrade on duplicate candidate field"
    );
    for (_, _, name) in KINDS {
        assert_eq!(count(&values, name), 4);
    }
    let independent = answer(&state, p, "independent");
    assert_eq!(count(&independent, "GuardedBy"), 1);
    for (field, _, name) in KINDS {
        assert_eq!(
            count(&independent, name),
            if field == "canMatch" { 0 } else { 1 },
            "a refused field cannot suppress valid other kinds"
        );
    }
    assert_eq!(
        crate::angular_meta::route_providers::project(&a)
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .count(),
        3
    );
}
#[test]
fn issue_162_guard77_controls_same_target_distinct_relations_and_outside_legacy() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let fields = KINDS
        .iter()
        .map(|(field, _, _)| format!("{field}:[Gate,Gate]"))
        .collect::<Vec<_>>()
        .join(",");
    std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Gate}} from './guard-a'; export const routes:Routes=[{{path:'kinds',canActivate:[Gate,Gate],{fields}}}];")).unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    let a = capture(&state, p, "one.ts");
    assert_eq!(kinds(&a).len(), 4);
    assert_eq!(crate::angular_meta::route_guards::project(&a).len(), 1);
    let observed = answer(&state, p, "kinds");
    assert_eq!(observed["edges"].as_array().unwrap().len(), 5);
    assert_eq!(count(&observed, "GuardedBy"), 1);
    for (_, _, name) in KINDS {
        assert_eq!(count(&observed, name), 1);
    }
    drop(state);
    for (declaration, expected) in [
        ("let routes:Routes", 1),
        ("const routes", 0),
        ("const routes:Foreign", 0),
    ] {
        std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Gate}} from './guard-a'; type Foreign=unknown[]; export {declaration}=[{{path:'legacy',{fields}}}];")).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        assert!(kinds(&capture(&state, p, "one.ts")).is_empty());
        let observed = answer(&state, p, "legacy");
        assert_eq!(count(&observed, "GuardedBy"), expected);
        for (_, _, name) in KINDS {
            assert_eq!(count(&observed, name), 0);
        }
    }
}
