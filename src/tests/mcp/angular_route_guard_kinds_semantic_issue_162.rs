//! Generation-77 exact kind-specific written membership through registered queries.
use super::typescript_route_guard_field_support::{call, capture, retained_field, setup};
use crate::ir::invocation_evidence::SourceSpan;
use crate::ir::typescript_resolution::Resolution;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
type Row = (String, String, String, String);
fn key(p: &std::path::Path) -> String {
    crate::dictionary::path::canonical_identity_key(&p.to_string_lossy())
}
fn counts(rows: &[Row]) -> BTreeMap<Row, usize> {
    let mut result = BTreeMap::new();
    for row in rows {
        *result.entry(row.clone()).or_default() += 1;
    }
    result
}
fn rows(answers: &[Value], relation: &str) -> Vec<Row> {
    answers
        .iter()
        .flat_map(|r| r["edges"].as_array().unwrap())
        .filter(|e| {
            e["relation"] == relation
                && ["same", "parent/child", ""].contains(&e["subject"]["name"].as_str().unwrap())
        })
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
    let forward:Vec<_>=["same","parent/child"].into_iter().map(|name|call(state,p,"workspace_query",json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}))).collect();
    let reverse = call(
        state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":"Guard","name":"Gate"}),
    );
    let control = call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"controls"}),
    );
    json!({"forward":forward,"reverse":reverse,"control":control})
}
fn question(field: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    for file in ["one.ts", "two.ts"] {
        let source = std::fs::read_to_string(p.join(file))
            .unwrap()
            .replace("canActivate:", &format!("{field}:"))
            .replace(
                "path:'controls',component:",
                "path:'controls',canActivate:[Gate],component:",
            );
        std::fs::write(p.join(file), source).unwrap();
    }
    let state = crate::mcp::McpState::new(config);
    let mut authorities = Vec::new();
    for owner in ["one.ts", "two.ts"] {
        let a = capture(&state, p, owner);
        assert_eq!(
            crate::angular_meta::route_guards::project(&a).len(),
            1,
            "stage=accepted-canActivate-control"
        );
        assert_eq!(crate::angular_meta::route_components::project(&a).len(), 1);
        assert_eq!(
            crate::angular_meta::route_lazy_components::project(&a).len(),
            1
        );
        assert_eq!(
            crate::angular_meta::route_load_children::project(&a).len(),
            1
        );
        assert_eq!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                .count(),
            2
        );
        authorities.push(a);
    }
    let relation = relation(field);
    let observed = observe(&state, p);
    eprintln!("guard-kind-semantic-observed field={field} public={observed}");
    let controls = observed["control"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "GuardedBy")
        .collect::<Vec<_>>();
    assert_eq!(
        controls.len(),
        2,
        "stage=accepted-public-canActivate-control"
    );
    assert!(controls.iter().all(|e| e["object"]["name"] == "Gate"
        && key(std::path::Path::new(e["object"]["file"].as_str().unwrap()))
            == key(&p.join("guard-a.ts"))));
    let mut supports = Vec::new();
    let mut slots = Vec::new();
    let mut checked = BTreeSet::new();
    for a in &authorities {
        let fields = retained_field(a, field);
        assert_eq!(
            fields.len(),
            7,
            "stage=guard-kind-field-prerequisite field={field}"
        );
        assert_eq!(fields.iter().map(|f| f.elements.len()).sum::<usize>(), 9);
        for (i, f) in fields.iter().enumerate() {
            let label = if i < 3 {
                "same"
            } else if i < 5 {
                ""
            } else {
                "parent/child"
            };
            let mut distinct = BTreeSet::new();
            for slot in &f.elements {
                let Resolution::Resolved(id) = &slot.reference else {
                    panic!(
                        "stage=guard-kind-runtime-reference-prerequisite field={field}: {slot:?}"
                    )
                };
                assert_eq!(
                    a.value_reference_at(SourceSpan::new(slot.start, slot.end).unwrap()),
                    Some(id),
                    "stage=guard-kind-runtime-occurrence-prerequisite field={field} public={observed}"
                );
                if checked.insert(id.clone()) {
                    let target = capture(&state, p, id.source.to_str().unwrap());
                    assert!(
                        target.declarations.contains(id),
                        "stage=guard-kind-selected-declaration-prerequisite field={field}"
                    );
                    assert_eq!(target.source_hash, id.source_hash);
                }
                let row = (
                    key(&a.source),
                    key(&id.source),
                    id.name.clone(),
                    label.to_owned(),
                );
                slots.push(row.clone());
                if distinct.insert(id.clone()) {
                    supports.push(row);
                }
            }
        }
    }
    assert_eq!(supports.len(), 16);
    assert_eq!(slots.len(), 18);
    for direction in ["forward", "reverse"] {
        let select = |row: &&Row| direction == "reverse" || !row.3.is_empty();
        let lower = counts(&supports.iter().filter(select).cloned().collect::<Vec<_>>());
        let answers = if direction == "forward" {
            observed["forward"].as_array().unwrap().clone()
        } else {
            vec![observed["reverse"].clone()]
        };
        let actual = counts(&rows(&answers, relation));
        assert_eq!(
            actual.keys().collect::<Vec<_>>(),
            lower.keys().collect::<Vec<_>>(),
            "stage=guard-kind-association field={field} direction={direction} public={observed}"
        );
        assert_eq!(
            actual, lower,
            "stage=guard-kind-association field={field} direction={direction} exact support policy public={observed}"
        );
    }
    let repeated = observe(&state, p);
    for (a, b) in observed["forward"]
        .as_array()
        .unwrap()
        .iter()
        .zip(repeated["forward"].as_array().unwrap())
    {
        assert_eq!(a["edges"], b["edges"]);
    }
    assert_eq!(observed["reverse"]["edges"], repeated["reverse"]["edges"]);
}
#[test]
fn issue_162_guard_kinds_semantic_can_match() {
    question("canMatch");
}
#[test]
fn issue_162_guard_kinds_semantic_can_deactivate() {
    question("canDeactivate");
}
#[test]
fn issue_162_guard_kinds_semantic_can_load() {
    question("canLoad");
}
#[test]
fn issue_162_guard_kinds_semantic_can_activate_child() {
    question("canActivateChild");
}

#[test]
fn issue_162_guard_kinds_semantic_distinct_public_kind_and_can_activate() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    for field in ["canMatch", "canDeactivate", "canLoad", "canActivateChild"] {
        std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Gate}} from './guard-a'; export const routes:Routes=[{{path:'kind-collision',canActivate:[Gate],{field}:[Gate,Gate]}}];")).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "one.ts");
        assert_eq!(retained_field(&a, field).len(), 1);
        assert_eq!(crate::angular_meta::route_guards::project(&a).len(), 1);
        let observed = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"kind-collision"}),
        );
        let mut actual = observed["edges"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["relation"].as_str().unwrap())
            .collect::<Vec<_>>();
        actual.sort();
        let mut expected = vec!["GuardedBy", relation(field)];
        expected.sort();
        assert_eq!(
            actual, expected,
            "exact field relation, independent canActivate, no summary or weak duplicate: {observed}"
        );
    }
}

fn relation(field: &str) -> &'static str {
    match field {
        "canMatch" => "GuardedByCanMatch",
        "canDeactivate" => "GuardedByCanDeactivate",
        "canLoad" => "GuardedByCanLoad",
        "canActivateChild" => "GuardedByCanActivateChild",
        _ => panic!("unexpected guard field"),
    }
}
