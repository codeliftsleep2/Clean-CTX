//! Bounded remaining-field acquisition probes, not semantic/public kind acceptance.
use super::typescript_route_guard_field_support::{call, capture, setup};
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};

fn walk(
    a: &TypeScriptAuthority,
    v: &ExpressionFacet,
    field: &str,
    counts: &mut (usize, usize),
    observed: &Value,
) {
    assert!(
        a.admits_demanded_structure(&a.source, &a.source_hash, v),
        "stage=guard-kind-container-prerequisite field={field}"
    );
    match &v.value {
        ExpressionValue::Array(values) => {
            for value in values {
                walk(a, value, field, counts, observed);
            }
        }
        ExpressionValue::Object(entries) => {
            if entries.iter().any(|(name, _)| name == "component") {
                return;
            }
            if let Some((_, children)) = entries.iter().find(|(name, _)| name == "children") {
                walk(a, children, field, counts, observed);
                return;
            }
            let fields = entries
                .iter()
                .filter(|(name, _)| name == field)
                .collect::<Vec<_>>();
            assert_eq!(
                fields.len(),
                1,
                "stage=guard-kind-field-prerequisite field={field} object={}..{} public={observed}",
                v.start,
                v.end
            );
            let array = &fields[0].1;
            assert!(v.start <= array.start && array.start < array.end && array.end <= v.end);
            assert!(
                a.admits_demanded_structure(&a.source, &a.source_hash, array),
                "stage=guard-kind-array-prerequisite field={field} public={observed}"
            );
            let ExpressionValue::Array(values) = &array.value else {
                panic!("stage=guard-kind-array-prerequisite field={field}")
            };
            for slot in values {
                assert!(
                    array.start <= slot.start && slot.start < slot.end && slot.end <= array.end
                );
                let ExpressionValue::Reference(Resolution::Resolved(id)) = &slot.value else {
                    panic!(
                        "stage=guard-kind-reference-prerequisite field={field} slot={slot:?} public={observed}"
                    )
                };
                assert_eq!(
                    a.demanded_reference_at(&a.source, &a.source_hash, slot.start, slot.end),
                    Some(id),
                    "stage=guard-kind-exact-reference-prerequisite field={field}"
                );
                assert!(!id.source_hash.is_empty());
                counts.1 += 1;
            }
            counts.0 += 1;
        }
        _ => panic!("stage=guard-kind-container-prerequisite field={field}"),
    }
}
fn question(field: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    // Preserve canonical canActivate on a distinct control object. Its public
    // GuardedBy answer must not be mistaken for membership in the candidate field.
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
    for file in ["guard-a.ts", "guard-b.ts"] {
        let target = capture(&state, p, file);
        assert!(
            target.declarations.iter().any(|id| id.name == "Gate"
                && id.source == target.source
                && id.source_hash == target.source_hash),
            "stage=guard-kind-independent-target-prerequisite"
        );
    }
    let mut authorities = Vec::new();
    for file in ["one.ts", "two.ts"] {
        let a = capture(&state, p, file);
        assert_eq!(
            crate::angular_meta::route_guards::project(&a).len(),
            1,
            "stage=canActivate-sibling"
        );
        assert_eq!(
            crate::angular_meta::route_components::project(&a).len(),
            1,
            "stage=eager-sibling"
        );
        assert_eq!(
            crate::angular_meta::route_lazy_components::project(&a).len(),
            1,
            "stage=lazy-sibling"
        );
        assert_eq!(
            crate::angular_meta::route_load_children::project(&a).len(),
            1,
            "stage=children-sibling"
        );
        assert_eq!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                .count(),
            2,
            "stage=provider-root-sibling"
        );
        authorities.push(a);
    }
    let forward:Vec<_>=["same","parent/child","controls"].into_iter().map(|name|
        call(&state,p,"workspace_query",json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}))).collect();
    let reverse = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":"Guard","name":"Gate"}),
    );
    let observed = json!({"forward":forward,"reverse":reverse});
    eprintln!("guard-kind-observed field={field} public={observed}");
    for a in &authorities {
        let root = a
            .declaration_facets
            .iter()
            .find(|f| f.declaration.source == a.source && f.declaration.name == "routes")
            .unwrap();
        let mut counts = (0, 0);
        walk(
            a,
            root.initializer.as_ref().unwrap(),
            field,
            &mut counts,
            &observed,
        );
        assert_eq!(counts, (7, 9));
    }
    // No field-specific public kind selector exists. Complete prerequisites must
    // precede any proposed exact association/kind/multiplicity/replacement oracle.
}
#[test]
fn issue_162_guard_kind_can_match_prerequisites() {
    question("canMatch");
}
#[test]
fn issue_162_guard_kind_can_deactivate_prerequisites() {
    question("canDeactivate");
}
#[test]
fn issue_162_guard_kind_can_load_prerequisites() {
    question("canLoad");
}
#[test]
fn issue_162_guard_kind_can_activate_child_prerequisites() {
    question("canActivateChild");
}
