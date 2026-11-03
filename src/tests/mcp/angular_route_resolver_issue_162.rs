//! Approved resolve investigation only: no acquisition or semantic repair.
use super::typescript_route_guard_field_support::{call, capture, setup};
use crate::ir::invocation_evidence::SourceSpan;
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
    duplicates: bool,
) {
    assert!(
        a.admits_demanded_structure(&a.source, &a.source_hash, v),
        "stage=resolver-container-prerequisite field={field}"
    );
    match &v.value {
        ExpressionValue::Array(values) => {
            for value in values {
                walk(a, value, field, counts, observed, duplicates);
            }
        }
        ExpressionValue::Object(entries) => {
            if entries.iter().any(|(name, _)| name == "component") {
                return;
            }
            if let Some((_, children)) = entries.iter().find(|(name, _)| name == "children") {
                walk(a, children, field, counts, observed, duplicates);
                return;
            }
            let fields = entries
                .iter()
                .filter(|(name, _)| name == field)
                .collect::<Vec<_>>();
            assert_eq!(
                fields.len(),
                if duplicates { 2 } else { 1 },
                "stage=resolver-field-prerequisite field={field} object={}..{} public={observed}",
                v.start,
                v.end
            );
            for (_, nested) in fields {
                assert!(
                    v.start <= nested.start && nested.start < nested.end && nested.end <= v.end
                );
                assert!(
                    a.admits_demanded_structure(&a.source, &a.source_hash, nested),
                    "stage=resolver-nested-object-prerequisite public={observed}"
                );
                let ExpressionValue::Object(mappings) = &nested.value else {
                    panic!("stage=resolver-nested-object-prerequisite public={observed}")
                };
                assert!(
                    !mappings.is_empty(),
                    "stage=resolver-mapping-prerequisite public={observed}"
                );
                for (index, (key, value)) in mappings.iter().enumerate() {
                    assert_eq!(
                        key,
                        &format!(
                            "data{}",
                            if duplicates && index + 1 == mappings.len() {
                                0
                            } else {
                                index
                            }
                        ),
                        "stage=resolver-key-value-prerequisite public={observed}"
                    );
                    assert!(
                        nested.start <= value.start
                            && value.start < value.end
                            && value.end <= nested.end
                    );
                    let ExpressionValue::Reference(Resolution::Resolved(id)) = &value.value else {
                        panic!(
                            "stage=resolver-reference-prerequisite value={value:?} public={observed}"
                        )
                    };
                    assert_eq!(
                        a.demanded_reference_at(&a.source, &a.source_hash, value.start, value.end),
                        Some(id),
                        "stage=resolver-demanded-reference-prerequisite"
                    );
                    assert_eq!(
                        a.value_reference_at(SourceSpan::new(value.start, value.end).unwrap()),
                        Some(id),
                        "stage=resolver-runtime-reference-prerequisite"
                    );
                    assert!(!id.source_hash.is_empty());
                    counts.1 += 1;
                }
            }
            counts.0 += 1;
        }
        _ => panic!("stage=resolver-container-prerequisite field={field}"),
    }
}
fn question(form: &str) {
    let field = "resolve";
    let _serial = crate::protocol::handler_response_serial();
    let duplicates = form == "duplicates";
    let (root, config) = setup(if duplicates { "direct" } else { form });
    let p = root.path();
    // Preserve canonical canActivate on a distinct control object. Its public
    // GuardedBy answer must not be mistaken for membership in the candidate field.
    for file in ["one.ts", "two.ts"] {
        let original = std::fs::read_to_string(p.join(file)).unwrap();
        let mut source = String::new();
        let mut rest = original.as_str();
        while let Some(start) = rest.find("canActivate:[") {
            source.push_str(&rest[..start]);
            let values = &rest[start + "canActivate:[".len()..];
            let end = values.find(']').unwrap();
            let mapping_start = source.len();
            source.push_str("resolve:{");
            for (index, value) in values[..end].split(',').enumerate() {
                source.push_str(&format!("data{index}:{value},"));
            }
            if duplicates {
                source.push_str(&format!(
                    "data0:{},",
                    values[..end].split(',').next().unwrap()
                ));
            }
            source.push('}');
            if duplicates {
                let mapping = source[mapping_start..].to_owned();
                source.push(',');
                source.push_str(&mapping);
            }
            rest = &values[end + 1..];
        }
        source.push_str(rest);
        let target = match form {
            "alias" | "barrel" => "A",
            "namespace" => "left.Gate",
            _ => "Gate",
        };
        source = source.replace("path:'controls',component:", &format!("path:'controls',canActivate:[{target}],canMatch:[{target}],canDeactivate:[{target}],canLoad:[{target}],canActivateChild:[{target}],component:"));
        std::fs::write(p.join(file), source).unwrap();
    }
    let state = crate::mcp::McpState::new(config);
    for file in ["guard-a.ts", "guard-b.ts"] {
        let target = capture(&state, p, file);
        assert!(
            target.declarations.iter().any(|id| id.name == "Gate"
                && id.source == target.source
                && id.source_hash == target.source_hash),
            "stage=resolver-independent-target-prerequisite"
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
            crate::angular_meta::route_guards::project_kinds(&a).len(),
            4,
            "stage=accepted-resolvers-sibling"
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
        json!({"type":"reverse_edges","domain":"angular","entity_type":"Resolver","name":"Gate"}),
    );
    let observed = json!({"forward":forward,"reverse":reverse});
    eprintln!("resolver-observed field={field} public={observed}");
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
            duplicates,
        );
        assert_eq!(counts, (7, if duplicates { 32 } else { 9 }));
    }
    // Mapping labels/value joins do not certify key-token spans or public key reachability.
    // Return these separately before any association oracle or semantic disposition.
}
#[test]
fn issue_162_resolver_direct_prerequisites() {
    question("direct");
}
#[test]
fn issue_162_resolver_alias_prerequisites() {
    question("alias");
}
#[test]
fn issue_162_resolver_barrel_prerequisites() {
    question("barrel");
}
#[test]
fn issue_162_resolver_namespace_prerequisites() {
    question("namespace");
}
#[test]
fn issue_162_resolver_local_prerequisites() {
    question("local");
}

#[test]
fn issue_162_resolver_duplicate_mappings_prerequisites() {
    question("duplicates");
}
