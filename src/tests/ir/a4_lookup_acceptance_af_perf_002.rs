//! Finite known-retrieval AST gate and paired construction-inclusive cost evidence.
use super::a4_lookup_tests::original;
use super::*;
use tree_sitter::{Node, Parser};

fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn inspect(source: &str, lookup: &str) -> Vec<String> {
    let mut parser = Parser::new();
    parser
        .set_language(&crate::compression::language::safe_rust_language().unwrap())
        .unwrap();
    let tree = parser.parse(source, None).unwrap();
    let mut issues = Vec::new();
    if tree.root_node().has_error() {
        return vec!["A4 AST parse error".into()];
    }
    let mut calls = BTreeMap::<String, usize>::new();
    fn walk(
        node: Node<'_>,
        source: &str,
        owner: &str,
        calls: &mut BTreeMap<String, usize>,
        issues: &mut Vec<String>,
    ) {
        let owner = if node.kind() == "function_item" {
            &source[node.child_by_field_name("name").unwrap().byte_range()]
        } else {
            owner
        };
        if node.kind() == "identifier"
            && ["collect_obligations", "visit"].contains(&owner)
            && ["units", "original_units"].contains(&&source[node.byte_range()])
        {
            issues.push("A4 restored full-input retrieval".into());
        }
        if node.kind() == "call_expression" {
            let function =
                compact(&source[node.child_by_field_name("function").unwrap().byte_range()]);
            if function.starts_with("inputs.") || function == "CSharpSourceSymbolIndex::build" {
                *calls.entry(function).or_default() += 1;
            }
        }
        for child in node.named_children(&mut node.walk()) {
            walk(child, source, owner, calls, issues);
        }
    }
    walk(tree.root_node(), source, "", &mut calls, &mut issues);
    let expected: BTreeMap<_, _> = [
        ("inputs.insert", 2),
        ("inputs.observe_storage", 2),
        ("inputs.seeds", 1),
        ("inputs.family", 2),
        ("inputs.family(key).iter", 1),
        ("inputs.family(key).iter().copied", 1),
        ("inputs.family(key).iter().copied().find", 1),
        ("CSharpSourceSymbolIndex::build", 2),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    if calls != expected {
        issues.push(format!("A4 unclassified retrieval/build calls: {calls:?}"));
    }
    let source = compact(source);
    if !source.contains("input.part.base_references.contains(written)") {
        issues.push("A4 original exact base predicate missing".into());
    }
    let lookup_tree = parser.parse(lookup, None).unwrap();
    if lookup_tree.root_node().has_error() {
        issues.push("A4 lookup AST parse error".into());
    }
    let lookup = compact(lookup);
    for required in [
        "Vec<&'aClassInput>",
        ".entry(Key(&input.part.key)).or_default().push(input)",
        ".get(&Key(key)).map_or(&[],Vec::as_slice)",
        ".entry(Key(input.part.file.as_str())).or_default().insert(Key(&input.part.key))",
    ] {
        if !lookup.contains(required) {
            issues.push(format!(
                "A4 occurrence/order lookup contract missing: {required}"
            ));
        }
    }
    issues
}
#[test]
fn afperf_a4_static_original_input_retrieval() {
    let source = include_str!("../../ir/csharp_resolution/compilation/dependencies.rs");
    let lookup = include_str!("../../ir/csharp_resolution/compilation/dependency_inputs.rs");
    let issues = inspect(source, lookup);
    assert!(
        issues.is_empty(),
        "A4 retrieval static protection: {issues:?}"
    );
    let changed = source.replace(
        "for key in inputs.seeds(seed)",
        "for key in units.seeds(seed)",
    );
    assert!(!inspect(&changed, lookup).is_empty());
    let changed = lookup.replace(".push(input)", ".push(&input.clone())");
    assert!(!inspect(source, &changed).is_empty());
    let changed = source.replace(".iter().copied().find", ".iter().rev().copied().find");
    assert!(!inspect(&changed, lookup).is_empty());
}

fn units(count: usize, density: usize, depth: usize) -> Vec<CSharpCompilationUnit> {
    let classes = (0..count)
        .map(|i| ClassInput {
            part: CSharpDeclarationPart {
                key: CSharpSymbolKey {
                    namespace_path: vec!["Example".into()],
                    containing_type_path: vec![],
                    name: format!("Type{}", i / density),
                    generic_arity: 0,
                },
                kind: super::super::super::CSharpDeclarationKind::Class,
                is_partial: true,
                file: format!("Owner{}.cs", i % 8),
                span: CSharpSourceSpan {
                    start_byte: 0,
                    end_byte: 10,
                },
                base_references: if i < depth {
                    vec![format!("Type{}", i / density + 1)]
                } else {
                    vec![]
                },
            },
            scope: Scope {
                namespace_path: vec!["Example".into()],
                containing_type_path: vec![],
                ordinary_imports: vec![],
                aliases: CSharpAliasContext::default(),
            },
            raw_class: String::new(),
            method_ids: vec![],
            methods: vec![],
            attributes: vec![],
        })
        .collect();
    vec![CSharpCompilationUnit { classes }]
}
#[test]
fn afperf_a4_paired_construction_cost() {
    use crate::work_observations_af_perf_002 as work;
    for (count, density, depth) in [
        (0, 1, 0),
        (1, 1, 0),
        (8, 1, 0),
        (64, 1, 0),
        (256, 1, 0),
        (64, 8, 0),
        (64, 64, 0),
        (256, 4, 16),
    ] {
        let units = units(count, density, depth);
        let names: Vec<_> = (0..8).map(|i| format!("Owner{i}.cs")).collect();
        let seeds: Vec<_> = names.iter().map(String::as_str).collect();
        // Also keep the frozen single-owner entry point live in this test module.
        let old_single = original::ancestry_obligations(&units, "Missing.cs", &[]);
        let new_single = ancestry_obligations(&units, "Missing.cs", &[]);
        assert_eq!(old_single.discovery_terms, new_single.discovery_terms);
        assert_eq!(old_single.owners, new_single.owners);
        assert_eq!(old_single.reasons, new_single.reasons);
        assert!(old_single.requirements.is_empty() && new_single.requirements.is_empty());
        let mut old_times = Vec::new();
        let mut new_times = Vec::new();
        for round in 0..25 {
            let mut old_result = None;
            let mut new_result = None;
            for old in if round % 2 == 0 {
                [true, false]
            } else {
                [false, true]
            } {
                let start = std::time::Instant::now();
                if old {
                    let value = work::unobserved(|| {
                        original::ancestry_obligations_for_files(&units, &seeds, &[])
                    });
                    let elapsed = start.elapsed().as_nanos();
                    if round > 0 {
                        old_times.push(elapsed);
                    }
                    old_result=Some(value.into_iter().map(|(f,v)|(f,format!("{:?}",
                        (v.discovery_terms,v.owners,v.reasons,v.requirements.iter().map(original::CSharpAncestryObligation::identity_material).collect::<Vec<_>>())))).collect::<BTreeMap<_,_>>());
                } else {
                    let value =
                        work::unobserved(|| ancestry_obligations_for_files(&units, &seeds, &[]));
                    let elapsed = start.elapsed().as_nanos();
                    if round > 0 {
                        new_times.push(elapsed);
                    }
                    new_result = Some(
                        value
                            .into_iter()
                            .map(|(f, v)| {
                                (
                                    f,
                                    format!(
                                        "{:?}",
                                        (
                                            v.discovery_terms,
                                            v.owners,
                                            v.reasons,
                                            v.requirements
                                                .iter()
                                                .map(CSharpAncestryObligation::identity_material)
                                                .collect::<Vec<_>>()
                                        )
                                    ),
                                )
                            })
                            .collect::<BTreeMap<_, _>>(),
                    );
                }
            }
            assert_eq!(new_result, old_result, "paired semantic identity");
        }
        old_times.sort_unstable();
        new_times.sort_unstable();
        eprintln!(
            "AF002_A4_PAIRED {}",
            serde_json::json!({"classes":count,"density":density,"depth":depth,
            "samples":24,"old_median_ns":old_times[12],"new_median_ns":new_times[12],
            "old_min_ns":old_times[0],"new_min_ns":new_times[0],
            "note":"debug helper, full construction and output; observation updates disabled; not public latency or allocator bytes"})
        );
    }
}
