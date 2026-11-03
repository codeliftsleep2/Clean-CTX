//! Generation-78 exact unqualified written support; key-specific answers remain open.
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
                    let joins = a
                        .resolve_mappings
                        .iter()
                        .filter(|m| m.value == (value.start, value.end))
                        .collect::<Vec<_>>();
                    assert_eq!(joins.len(), 1, "stage=resolver-exact-mapping-prerequisite");
                    let (mapping, target) = a
                        .resolve_mapping_at(joins[0].pair)
                        .expect("stage=resolver-key-token-prerequisite");
                    assert_eq!(mapping.name, *key);
                    assert_eq!(target, id);
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
    let duplicates = false;
    let (root, config) = fixture(form);
    let p = root.path();
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
            "stage=accepted-guard-kinds-sibling"
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
    let reverse_right = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":"Resolver","name":"RightGate"}),
    );
    let observed = json!({"forward":forward,"reverse":[reverse,reverse_right]});
    eprintln!("resolver-semantic-observed form={form} public={observed}");
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
    let mut supports = Vec::new();
    let mut slots = Vec::new();
    for a in &authorities {
        let mut groups = std::collections::BTreeMap::new();
        for m in &a.resolve_mappings {
            groups.entry(m.owner).or_insert_with(Vec::new).push(m);
        }
        assert_eq!(groups.len(), 7);
        assert_eq!(a.resolve_mappings.len(), 9);
        for ((_, mappings), label) in groups.into_iter().zip([
            "same",
            "same",
            "same",
            "",
            "",
            "parent/child",
            "parent/child",
        ]) {
            let mut distinct = std::collections::BTreeSet::new();
            for m in mappings {
                let (_, id) = a
                    .resolve_mapping_at(m.pair)
                    .expect("stage=resolver-complete-selection-prerequisite");
                let relative = id.source.strip_prefix(p).unwrap().to_str().unwrap();
                let independent = capture(&state, p, relative);
                assert!(
                    independent.declarations.contains(id),
                    "stage=resolver-independent-selected-declaration-prerequisite"
                );
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
        let select = |r: &&Row| direction == "reverse" || !r.3.is_empty();
        let lower = counts(supports.iter().filter(select).cloned());
        let actual = counts(rows(observed[direction].as_array().unwrap()).into_iter());
        assert_eq!(
            actual.keys().collect::<Vec<_>>(),
            lower.keys().collect::<Vec<_>>(),
            "stage=resolve-association direction={direction} form={form} public={observed}"
        );
        assert_eq!(
            actual, lower,
            "stage=resolve-support-multiplicity form={form} direction={direction}"
        );
    }
    let repeat = observe(&state, p);
    for direction in ["forward", "reverse"] {
        for (first, next) in observed[direction]
            .as_array()
            .unwrap()
            .iter()
            .zip(repeat[direction].as_array().unwrap())
        {
            assert_eq!(
                first["edges"], next["edges"],
                "stage=resolver-repeated-query"
            );
        }
    }
}

pub(super) fn fixture(form: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = setup(form);
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
            source.push_str("resolve:{");
            for (index, value) in values[..end].split(',').enumerate() {
                source.push_str(&format!("data{index}:{value},"));
            }
            source.push('}');
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
    (root, config)
}

type Row = (String, String, String, String);
fn key(p: &std::path::Path) -> String {
    crate::dictionary::path::canonical_identity_key(&p.to_string_lossy())
}
fn counts(rows: impl Iterator<Item = Row>) -> std::collections::BTreeMap<Row, usize> {
    let mut out = std::collections::BTreeMap::new();
    for row in rows {
        *out.entry(row).or_default() += 1;
    }
    out
}
fn rows(answers: &[Value]) -> Vec<Row> {
    answers
        .iter()
        .flat_map(|a| a["edges"].as_array().unwrap())
        .filter(|e| {
            e["relation"] == "ResolvedBy"
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
    let forward:Vec<_>=["same","parent/child","controls"].into_iter().map(|name|call(state,p,"workspace_query",json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}))).collect();
    let reverse:Vec<_>=["Gate","RightGate"].into_iter().map(|name|call(state,p,"workspace_query",json!({"type":"reverse_edges","domain":"angular","entity_type":"Resolver","name":name}))).collect();
    json!({"forward":forward,"reverse":reverse})
}
#[test]
fn issue_162_resolver_semantic_direct() {
    question("direct");
}
#[test]
fn issue_162_resolver_semantic_alias() {
    question("alias");
}
#[test]
fn issue_162_resolver_semantic_barrel() {
    question("barrel");
}
#[test]
fn issue_162_resolver_semantic_namespace() {
    question("namespace");
}
#[test]
fn issue_162_resolver_semantic_local() {
    question("local");
}
#[test]
fn issue_162_resolver_semantic_key_only_public_collision() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let before = capture(&state, p, "one.ts");
    assert_eq!(before.resolve_mappings.len(), 9);
    for m in &before.resolve_mappings {
        assert!(before.resolve_mapping_at(m.pair).is_some());
    }
    let first = observe(&state, p);
    eprintln!("resolver-key-observed before={first}");
    drop(state);
    for owner in ["one.ts", "two.ts"] {
        let source = std::fs::read_to_string(p.join(owner)).unwrap();
        std::fs::write(p.join(owner), source.replace("data0:", "other:")).unwrap();
    }
    let state = crate::mcp::McpState::new(config);
    let after = capture(&state, p, "one.ts");
    assert_ne!(before.source_hash, after.source_hash);
    assert_eq!(after.resolve_mappings.len(), 9);
    assert!(after.resolve_mappings.iter().any(|m| m.name == "other"));
    for m in &after.resolve_mappings {
        assert!(after.resolve_mapping_at(m.pair).is_some());
    }
    let second = observe(&state, p);
    eprintln!("resolver-key-observed after={second}");
    for direction in ["forward", "reverse"] {
        for (a, b) in first[direction]
            .as_array()
            .unwrap()
            .iter()
            .zip(second[direction].as_array().unwrap())
        {
            let sorted = |v: &Value| {
                let mut rows = v["edges"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|e| e["relation"] == "ResolvedBy")
                    .map(|e| serde_json::to_string(e).unwrap())
                    .collect::<Vec<_>>();
                rows.sort();
                rows
            };
            assert_eq!(
                sorted(a),
                sorted(b),
                "current key-only public collision observation"
            );
        }
    }
}
