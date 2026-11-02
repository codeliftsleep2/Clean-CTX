//! Four approved neutral demand names; no candidate GuardedBy meaning.
use super::typescript_guard_kinds_field_support::*;
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;

#[test]
fn issue_162_guard_kinds_field_exact_forms_independent_origins_slots_serde() {
    let _serial = crate::protocol::handler_response_serial();
    for form in ["direct", "alias", "barrel", "namespace", "local"] {
        let (root, config) = setup(form);
        let p = root.path();
        let state = crate::mcp::McpState::new(config);
        let mut independent = std::collections::BTreeSet::new();
        if form != "local" {
            for target in ["guard-a.ts", "guard-b.ts"] {
                let target = capture(&state, p, target);
                let id = target
                    .declarations
                    .iter()
                    .find(|id| {
                        id.name == "Gate"
                            && id.source == target.source
                            && id.source_hash == target.source_hash
                    })
                    .unwrap();
                independent.insert(id.clone());
            }
            assert_eq!(independent.len(), 2);
        }
        for file in ["one.ts", "two.ts"] {
            let a = capture(&state, p, file);
            let groups = all(&a);
            assert_eq!(groups.len(), 4);
            let mut arrays = std::collections::BTreeSet::new();
            let mut slots = std::collections::BTreeSet::new();
            for (field, fields) in &groups {
                assert_eq!(fields.len(), 7, "field={field} form={form}");
                assert_eq!(fields.iter().map(|f| f.elements.len()).sum::<usize>(), 9);
                assert_eq!(
                    fields
                        .iter()
                        .map(|f| f.object)
                        .collect::<std::collections::BTreeSet<_>>()
                        .len(),
                    7
                );
                assert_eq!(
                    fields[0].elements[0].reference,
                    fields[0].elements[2].reference
                );
                assert_ne!(fields[0].elements[0].start, fields[0].elements[2].start);
                assert_ne!(
                    fields[0].elements[0].reference,
                    fields[0].elements[1].reference
                );
                for field in fields {
                    assert!(
                        arrays.insert(field.span),
                        "different field origins cannot share an Array occurrence"
                    );
                    for element in &field.elements {
                        assert!(slots.insert((element.start, element.end)));
                        let Resolution::Resolved(id) = &element.reference else {
                            panic!(
                                "stage=neutral-kind-reference-prerequisite form={form} {element:?}"
                            )
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
                            assert!(a.declarations.contains(id));
                        } else {
                            assert!(independent.contains(id));
                        }
                    }
                }
            }
            assert_eq!(arrays.len(), 28);
            assert_eq!(slots.len(), 36);
            let restored: TypeScriptAuthority =
                serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
            assert_eq!(all(&restored), groups);
            assert_eq!(
                crate::angular_meta::route_guards::project(&a).len(),
                1,
                "only accepted canActivate control publishes canonical guard use"
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
        }
        for name in ["same", "parent/child"] {
            let observed = call(
                &state,
                p,
                "workspace_query",
                json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}),
            );
            assert_eq!(
                observed["edges"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|e| e["relation"] == "GuardedBy")
                    .count(),
                if form == "namespace" { 0 } else { 4 },
                "field-only acquisition must preserve old merged labels: {form}/{name}"
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
            if matches!(form, "direct" | "local") {
                8
            } else {
                2
            }
        );
    }
}
#[test]
fn issue_162_guard_kinds_field_duplicate_unproved_slots_and_nonroute_neutrality() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from './guard-a'; export * from './guard-b';",
    )
    .unwrap();
    let fields = FIELDS
        .iter()
        .map(|field| format!("{field}:[A,A],{field}:[B,Missing,T,Ambiguous]"))
        .collect::<Vec<_>>()
        .join(",");
    std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Gate as A}} from './guard-a'; import {{Gate as B}} from './guard-b'; import type {{Gate as T}} from './guard-a'; import {{Gate as Ambiguous}} from './ambiguous'; import {{Token}} from './left'; export const routes:Routes=[{{path:'duplicate',{fields},providers:[Token]}}];")).unwrap();
    let metadata = FIELDS
        .iter()
        .map(|field| format!("{field}:[Gate,Gate]"))
        .collect::<Vec<_>>()
        .join(",");
    std::fs::write(p.join("metadata.ts"),format!("import {{Component}} from '@angular/core'; import {{Gate}} from './guard-a'; @Component({{{metadata}}}) export class Other {{}}")).unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    for (field, fields) in all(&a) {
        assert_eq!(fields.len(), 2, "duplicate mapping {field}");
        assert_eq!(fields[0].object, fields[1].object);
        assert_ne!(fields[0].span, fields[1].span);
        assert_eq!(fields[0].elements.len(), 2);
        assert_eq!(fields[1].elements.len(), 4);
        assert_eq!(
            fields[0].elements[0].reference,
            fields[0].elements[1].reference
        );
        assert_ne!(fields[0].elements[0].start, fields[0].elements[1].start);
        assert!(matches!(
            fields[1].elements[0].reference,
            Resolution::Resolved(_)
        ));
        assert!(
            fields[1].elements[1..]
                .iter()
                .all(|e| !matches!(e.reference, Resolution::Resolved(_))),
            "no guessed missing/type-only/ambiguous runtime reference"
        );
    }
    assert!(crate::angular_meta::route_guards::project(&a).is_empty());
    assert_eq!(
        crate::angular_meta::route_providers::project(&a)
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .count(),
        1
    );
    let a = capture(&state, p, "metadata.ts");
    assert!(crate::angular_meta::route_guards::project(&a).is_empty());
    assert!(crate::angular_meta::route_providers::project(&a).is_empty());
    let facet = a
        .declaration_facets
        .iter()
        .find(|f| f.declaration.source == a.source && f.declaration.name == "Other")
        .unwrap();
    let object = &facet.decorators[0].arguments[0];
    assert!(a.admits_demanded_structure(&a.source, &a.source_hash, object));
    let ExpressionValue::Object(entries) = &object.value else {
        panic!("neutral metadata Object")
    };
    for field in FIELDS {
        let fields = entries
            .iter()
            .filter(|(name, _)| name == field)
            .collect::<Vec<_>>();
        assert_eq!(fields.len(), 1);
        let array = &fields[0].1;
        assert!(a.admits_demanded_structure(&a.source, &a.source_hash, array));
        let ExpressionValue::Array(values) = &array.value else {
            panic!("neutral metadata Array")
        };
        assert_eq!(values.len(), 2);
        assert_ne!(values[0].start, values[1].start);
    }
}
