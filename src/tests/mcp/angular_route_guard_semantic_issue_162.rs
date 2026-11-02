//! Investigation hypothesis only: written use, never intrinsic Guard eligibility.
use super::typescript_route_guard_field_support::{call, capture, retained, setup};
use crate::ir::typescript_resolution::Resolution;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

// Public coordinates cannot expose private object spans. Counts retain distinct
// objects; repeated slots contribute support, not duplicate public relationships.
type Row = (String, String, String, String);
fn key(path: &std::path::Path) -> String {
    crate::dictionary::path::canonical_identity_key(&path.to_string_lossy())
}
fn counts(rows: &[Row]) -> BTreeMap<Row, usize> {
    let mut result = BTreeMap::new();
    for row in rows {
        *result.entry(row.clone()).or_default() += 1;
    }
    result
}
fn rows(answers: &[Value]) -> Vec<Row> {
    answers
        .iter()
        .flat_map(|a| a["edges"].as_array().unwrap())
        .filter(|e| e["relation"] == "GuardedBy")
        .map(|e| {
            (
                key(std::path::Path::new(e["subject"]["file"].as_str().unwrap())),
                key(std::path::Path::new(e["object"]["file"].as_str().unwrap())),
                e["object"]["name"].as_str().unwrap().to_owned(),
                e["subject"]["name"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}
fn observe(state: &crate::mcp::McpState, p: &std::path::Path) -> Value {
    let forward: Vec<_> = ["same", "parent/child"].into_iter().map(|name|
        call(state,p,"workspace_query",json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}))).collect();
    let reverse: Vec<_> = ["Gate", "RightGate"].into_iter().map(|name|
        call(state,p,"workspace_query",json!({"type":"reverse_edges","domain":"angular","entity_type":"Guard","name":name}))).collect();
    json!({"forward":forward,"reverse":reverse})
}
fn question(form: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(form);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let mut supports = Vec::new();
    let mut slots = Vec::new();
    let mut checked_targets = BTreeSet::new();
    for owner in ["one.ts", "two.ts"] {
        let a = capture(&state, p, owner);
        assert_eq!(
            crate::angular_meta::route_components::project(&a).len(),
            1,
            "stage=eager-prerequisite"
        );
        assert_eq!(
            crate::angular_meta::route_lazy_components::project(&a).len(),
            1,
            "stage=lazy-prerequisite"
        );
        assert_eq!(
            crate::angular_meta::route_load_children::project(&a).len(),
            1,
            "stage=children-prerequisite"
        );
        assert_eq!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                .count(),
            2,
            "stage=provider-root-prerequisite"
        );
        let fields = retained(&a);
        assert_eq!(fields.len(), 7, "stage=guard-fields-prerequisite");
        assert_eq!(fields.iter().map(|f| f.elements.len()).sum::<usize>(), 9);
        for (i, field) in fields.iter().enumerate() {
            // Fixture order supplies the display oracle only, never production identity.
            let path = if i < 3 {
                "same"
            } else if i < 5 {
                ""
            } else {
                "parent/child"
            };
            let mut distinct = BTreeSet::new();
            for element in &field.elements {
                let Resolution::Resolved(id) = &element.reference else {
                    panic!("stage=guard-reference-prerequisite form={form}: {element:?}")
                };
                if checked_targets.insert(id.clone()) {
                    let target = capture(&state, p, id.source.to_str().unwrap());
                    assert!(
                        target.declarations.contains(id),
                        "stage=guard-target-declaration-prerequisite {id:?}"
                    );
                    assert_eq!(target.source_hash, id.source_hash);
                }
                let row = (
                    key(&a.source),
                    key(&id.source),
                    id.name.clone(),
                    path.to_owned(),
                );
                slots.push(row.clone());
                // One support per exact object/target; separate objects remain separate.
                if distinct.insert(id.clone()) {
                    supports.push(row);
                }
            }
        }
    }
    assert_eq!(supports.len(), 16);
    assert_eq!(slots.len(), 18);
    let observed = observe(&state, p);
    eprintln!("guard-semantic-observed form={form} public={observed}");
    for direction in ["forward", "reverse"] {
        let select = |row: &&Row| direction == "reverse" || !row.3.is_empty();
        let lower = counts(&supports.iter().filter(select).cloned().collect::<Vec<_>>());
        let actual = counts(&rows(observed[direction].as_array().unwrap()));
        assert_eq!(
            actual.keys().collect::<Vec<_>>(),
            lower.keys().collect::<Vec<_>>(),
            "stage=canActivate-association direction={direction} form={form} public={observed}"
        );
        assert_eq!(
            actual, lower,
            "stage=canActivate-association direction={direction} form={form} support multiplicity public={observed}"
        );
    }
    let repeated = observe(&state, p);
    for direction in ["forward", "reverse"] {
        for (first, next) in observed[direction]
            .as_array()
            .unwrap()
            .iter()
            .zip(repeated[direction].as_array().unwrap())
        {
            assert_eq!(first["edges"], next["edges"], "stage=guard-repeated-query");
        }
    }
}
#[test]
fn issue_162_guard_semantic_direct() {
    question("direct");
}
#[test]
fn issue_162_guard_semantic_alias() {
    question("alias");
}
#[test]
fn issue_162_guard_semantic_barrel() {
    question("barrel");
}
#[test]
fn issue_162_guard_semantic_namespace() {
    question("namespace");
}
#[test]
fn issue_162_guard_semantic_local() {
    question("local");
}
