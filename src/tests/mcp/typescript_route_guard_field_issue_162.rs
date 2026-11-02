//! Neutral canActivate field/Array/slot/reference retention only.
use super::typescript_route_guard_field_support::*;
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;
#[test]
fn issue_162_guard_field_exact_forms_slots_serde_and_legacy_preservation() {
    let _serial = crate::protocol::handler_response_serial();
    for form in ["direct", "alias", "barrel", "namespace", "local"] {
        let (root, config) = setup(form);
        let p = root.path();
        let state = crate::mcp::McpState::new(config);
        for owner in ["one.ts", "two.ts"] {
            let a = capture(&state, p, owner);
            let fields = retained(&a);
            assert_eq!(fields.len(), 7);
            let mut objects = fields.iter().map(|f| f.object).collect::<Vec<_>>();
            objects.sort();
            objects.dedup();
            assert_eq!(objects.len(), 7);
            let slots = fields.iter().flat_map(|f| &f.elements).collect::<Vec<_>>();
            assert_eq!(slots.len(), 9);
            let mut spans = slots.iter().map(|e| (e.start, e.end)).collect::<Vec<_>>();
            spans.sort();
            spans.dedup();
            assert_eq!(spans.len(), 9);
            assert_eq!(fields[0].elements.len(), 3);
            assert_eq!(
                fields[0].elements[0].reference,
                fields[0].elements[2].reference
            );
            assert_ne!(fields[0].elements[0].start, fields[0].elements[2].start);
            assert_ne!(
                fields[0].elements[0].reference,
                fields[0].elements[1].reference
            );
            for element in slots {
                let Resolution::Resolved(id) = &element.reference else {
                    panic!("stage=neutral-runtime-reference-prerequisite form={form}: {element:?}")
                };
                assert_eq!(
                    id.kind,
                    if form == "local" {
                        "function_declaration"
                    } else {
                        "variable_declarator"
                    }
                );
                if form == "local" {
                    assert_eq!(id.source, a.source);
                } else {
                    assert!(id.source.ends_with("guard-a.ts") || id.source.ends_with("guard-b.ts"));
                }
            }
            let roundtrip: TypeScriptAuthority =
                serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
            assert_eq!(retained(&roundtrip), fields);
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
        }
        for name in ["same", "parent/child"] {
            let old = call(
                &state,
                p,
                "workspace_query",
                json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}),
            );
            assert_eq!(
                old["edges"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|e| e["relation"] == "GuardedBy")
                    .count(),
                if name == "same" { 8 } else { 4 },
                "approved canonical guard support must remain: {form}/{name}"
            );
        }
        let reverse = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"reverse_edges","domain":"angular","entity_type":"Guard","name":"Gate"}),
        );
        assert_eq!(
            reverse["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["relation"] == "GuardedBy")
                .count(),
            if form == "local" { 8 } else { 16 }
        );
    }
}
#[test]
fn issue_162_guard_field_duplicate_mappings_and_nonroute_metadata_neutrality() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(p.join("one.ts"),"import type {Routes} from '@angular/router'; import {Gate as A} from './guard-a'; import {Gate as B} from './guard-b'; export const routes:Routes=[{path:'duplicate',canActivate:[A,A],canActivate:[B]}];").unwrap();
    std::fs::write(p.join("metadata.ts"),"import {Component} from '@angular/core'; import {Gate} from './guard-a'; @Component({canActivate:[Gate,Gate]}) export class Other {}").unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    let fields = retained(&a);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].object, fields[1].object);
    assert_ne!(fields[0].span, fields[1].span);
    assert_eq!(fields[0].elements.len(), 2);
    assert_eq!(fields[1].elements.len(), 1);
    assert_eq!(
        fields[0].elements[0].reference,
        fields[0].elements[1].reference
    );
    assert_ne!(fields[0].elements[0].start, fields[0].elements[1].start);
    let metadata = capture(&state, p, "metadata.ts");
    assert!(crate::angular_meta::route_components::project(&metadata).is_empty());
    assert!(crate::angular_meta::route_providers::project(&metadata).is_empty());
    assert!(crate::angular_meta::route_load_children::project(&metadata).is_empty());
    let facet = metadata
        .declaration_facets
        .iter()
        .find(|f| f.declaration.source == metadata.source && f.declaration.name == "Other")
        .unwrap();
    let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Object(entries) =
        &facet.decorators[0].arguments[0].value
    else {
        panic!("metadata Object")
    };
    let arrays = entries
        .iter()
        .filter(|(name, _)| name == "canActivate")
        .collect::<Vec<_>>();
    assert_eq!(arrays.len(), 1);
    let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Array(values) =
        &arrays[0].1.value
    else {
        panic!("metadata Array")
    };
    assert_eq!(values.len(), 2);
    assert_ne!(values[0].start, values[1].start);
}
#[test]
fn issue_162_guard_field_unproved_references_and_plain_runtime_values() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from './guard-a'; export * from './guard-b';",
    )
    .unwrap();
    std::fs::write(
        p.join("values.ts"),
        "export const Numeric=123; export class Plain {} export function Ordinary(){return true;}",
    )
    .unwrap();
    std::fs::write(p.join("one.ts"),"import type {Routes} from '@angular/router'; import type {Gate as T} from './guard-a'; import {Gate as Ambiguous} from './ambiguous'; import {Numeric,Plain,Ordinary} from './values'; import {Token} from './left'; export const routes:Routes=[{path:'neutral',canActivate:[Missing,T,Ambiguous,Numeric,Plain,Ordinary],providers:[Token]}];").unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    let fields = retained(&a);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].elements.len(), 6);
    assert!(
        fields[0].elements[..3]
            .iter()
            .all(|e| !matches!(e.reference, Resolution::Resolved(_))),
        "unproved runtime references cannot be guessed"
    );
    assert!(
        fields[0].elements[3..]
            .iter()
            .all(|e| matches!(e.reference, Resolution::Resolved(_))),
        "runtime value identity grants no guard eligibility"
    );
    assert_eq!(
        crate::angular_meta::route_providers::project(&a)
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .count(),
        1
    );
}
