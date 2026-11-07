//! Finite exact-end retrieval protection and original two-join kernel comparisons.
use super::*;
use crate::work_observations_af_perf_002 as work;
use tree_sitter::{Node, Parser as AstParser};
type Declaration = (u64, u64, Option<u64>);
type Joins = (Vec<Option<Declaration>>, Vec<bool>);
fn unit(start: u64, end: u64) -> UnitRecord {
    UnitRecord {
        qualified_name: "Subject.m0".into(),
        method_id: "M0".into(),
        name: "m0".into(),
        class_name: Some("Subject".into()),
        start_byte: start,
        end_byte: end,
        text: String::new(),
        declaration_span: None,
        insert_before_byte: None,
        declaration_blocker: Some("unavailable".into()),
        fingerprint: "Subject()".into(),
    }
}
// Frozen predicates/tie selection, independently copied from original production.
fn original(declarations: Vec<Declaration>, units: &[UnitRecord], syntax: &[(u64, u64)]) -> Joins {
    (
        units
            .iter()
            .map(|unit| {
                declarations
                    .iter()
                    .copied()
                    .filter(|(start, end, _)| {
                        // Match the frozen production observer site as well as
                        // its predicate; disabled probes still cost a call.
                        work::update(|c| c.edit.attachment_predicates += 1);
                        *start <= unit.start_byte && *end == unit.end_byte
                    })
                    .min_by_key(|(start, end, _)| end - start)
            })
            .collect(),
        syntax
            .iter()
            .map(|(start, end)| {
                units.iter().any(|unit| {
                    work::update(|c| c.edit.editable_predicates += 1);
                    unit.end_byte == *end && unit.start_byte >= *start
                })
            })
            .collect(),
    )
}
fn corrected(declarations: Vec<Declaration>, units: &[UnitRecord], syntax: &[(u64, u64)]) -> Joins {
    let join = EndJoin::new(declarations, units);
    (
        units.iter().map(|unit| join.declaration(unit)).collect(),
        syntax
            .iter()
            .map(|(start, end)| join.editable(*start, *end))
            .collect(),
    )
}
#[test]
fn afperf_edit_end_raw_order_and_metadata() {
    let cases = vec![
        vec![],
        vec![(0, 10, None)],
        vec![(0, 10, Some(0)), (0, 10, None)],
        vec![(0, 10, None), (0, 10, Some(0))],
        vec![
            (0, 10, Some(0)),
            (3, 10, Some(3)),
            (3, 10, None),
            (4, 11, Some(4)),
        ],
        vec![(0, 0, None), (0, 0, Some(0)), (0, 9, Some(0))],
    ];
    let units = vec![
        unit(0, 10),
        unit(3, 10),
        unit(9, 10),
        unit(0, 0),
        unit(4, 11),
        unit(0, 99),
        unit(12, 10),
    ];
    let syntax: Vec<_> = [0, 9, 10, 11, 99]
        .into_iter()
        .flat_map(|end| (0..=12).map(move |start| (start, end)))
        .collect();
    for declarations in cases {
        for reversed in [false, true] {
            let mut declarations = declarations.clone();
            let mut units = units.clone();
            if reversed {
                declarations.reverse();
                units.reverse();
            }
            assert_eq!(
                corrected(declarations.clone(), &units, &syntax),
                original(declarations.clone(), &units, &syntax)
            );
            assert_eq!(
                corrected(declarations.clone(), &[], &syntax),
                original(declarations.clone(), &[], &syntax)
            );
            assert_eq!(
                corrected(declarations.clone(), &units, &[]),
                original(declarations, &units, &[])
            );
        }
    }
    let target = unit(3, 10);
    assert_eq!(
        EndJoin::new(vec![(3, 10, None), (3, 10, Some(3))], &[]).declaration(&target),
        Some((3, 10, None))
    );
    assert_eq!(
        EndJoin::new(vec![(3, 10, Some(3)), (3, 10, None)], &[]).declaration(&target),
        Some((3, 10, Some(3)))
    );
}
#[test]
fn afperf_edit_end_construction_cost() {
    for (count, density, misses) in [
        (0, 1, 1),
        (1, 1, 1),
        (8, 1, 1),
        (64, 1, 1),
        (256, 1, 1),
        (64, 8, 16),
        (64, 64, 16),
        (256, 256, 16),
    ] {
        let declarations: Vec<_> = (0..count)
            .map(|i| {
                let end = (i / density + 1) as u64 * 100;
                (end - 50, end, Some(end - 50))
            })
            .collect();
        let units: Vec<_> = declarations
            .iter()
            .map(|(_, end, _)| unit(end - 25, *end))
            .collect();
        let mut syntax: Vec<_> = declarations
            .iter()
            .map(|(start, end, _)| (*start, *end))
            .collect();
        syntax.extend((0..misses).map(|i| (0, 1_000_000 + i as u64)));
        let expected = original(declarations.clone(), &units, &syntax);
        work::reset();
        let actual = corrected(declarations.clone(), &units, &syntax);
        assert_eq!(actual, expected);
        let counts = work::snapshot().edit;
        assert_eq!(counts.end_setup_declarations, count);
        assert_eq!(counts.end_setup_units, count);
        assert_eq!(counts.end_retained_declarations, count);
        assert_eq!(counts.end_attachment_lookups, count);
        assert_eq!(counts.end_editability_lookups, syntax.len());
        let mut before = Vec::new();
        let mut after = Vec::new();
        for round in 0..25 {
            for old in if round % 2 == 0 {
                [true, false]
            } else {
                [false, true]
            } {
                // Both receive the same already-owned input outside timing.
                let input = declarations.clone();
                let start = std::time::Instant::now();
                let result = work::unobserved(|| {
                    if old {
                        original(input, &units, &syntax)
                    } else {
                        corrected(input, &units, &syntax)
                    }
                });
                let elapsed = start.elapsed().as_nanos();
                assert_eq!(result, expected);
                if round > 0 {
                    if old {
                        before.push(elapsed)
                    } else {
                        after.push(elapsed)
                    }
                }
            }
        }
        before.sort_unstable();
        after.sort_unstable();
        eprintln!(
            "AF002_EDIT_END_COST {}",
            serde_json::json!({"declarations":count,"units":count,"same_end_density":density,
            "missing_ends":misses,"work":counts,"tuple_size":std::mem::size_of::<Declaration>(),"samples":24,
            "old_median_ns":before[12],"new_median_ns":after[12],"old_min_ns":before[0],"new_min_ns":after[0],
            "scope":"two-join kernel including map construction/drop and output vectors; input preparation excluded; debug; not public latency or allocator bytes"})
        );
    }
}
fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn inspect(source: &str, helper: &str) -> Vec<String> {
    let mut parser = AstParser::new();
    parser
        .set_language(&crate::compression::language::safe_rust_language().unwrap())
        .unwrap();
    let tree = parser.parse(source, None).unwrap();
    let helper_tree = parser.parse(helper, None).unwrap();
    let mut issues = Vec::new();
    if tree.root_node().has_error() || helper_tree.root_node().has_error() {
        return vec!["end-join AST parse failure".into()];
    }
    let mut calls = std::collections::BTreeMap::<String, usize>::new();
    fn walk(
        node: Node<'_>,
        source: &str,
        owner: &str,
        calls: &mut std::collections::BTreeMap<String, usize>,
        issues: &mut Vec<String>,
    ) {
        let owner = if node.kind() == "function_item" {
            &source[node.child_by_field_name("name").unwrap().byte_range()]
        } else {
            owner
        };
        if owner == "from_instructions_with_declarations" && node.kind() == "call_expression" {
            let function =
                compact(&source[node.child_by_field_name("function").unwrap().byte_range()]);
            if function.starts_with("end_join.") || function == "EndJoin::new" {
                *calls.entry(function.clone()).or_default() += 1;
            }
            if function.starts_with("declarations.iter")
                || function.starts_with("table.units.iter")
                || function == "declarations.clone"
            {
                issues.push("restored full-input join or declaration cloning".into());
            }
            let args =
                compact(&source[node.child_by_field_name("arguments").unwrap().byte_range()]);
            if function == "EndJoin::new" && args != "(declarations,&table.units)" {
                issues.push("declarations must move into one local join".into());
            }
        }
        for child in node.named_children(&mut node.walk()) {
            walk(child, source, owner, calls, issues);
        }
    }
    walk(tree.root_node(), source, "", &mut calls, &mut issues);
    let expected = std::collections::BTreeMap::from([
        ("EndJoin::new".to_owned(), 1),
        ("end_join.declaration".to_owned(), 1),
        ("end_join.editable".to_owned(), 1),
    ]);
    if calls != expected {
        issues.push(format!("unclassified join calls {calls:?}"));
    }
    let helper = compact(helper);
    for required in [
        "declarations:Vec<Declaration>",
        ".declarations.push(declaration)",
        "*start<=unit.start_byte&&*end==unit.end_byte",
        ".min_by_key(|(start,end,_)|end-start)",
        ".get(&End(unit.end_byte))",
        ".get(&End(end))",
        ".and_then(|bucket|bucket.max_start).is_some_and(|maximum|maximum>=start)",
    ] {
        if !helper.contains(required) {
            issues.push(format!(
                "end-join predicate/order contract missing: {required}"
            ));
        }
    }
    issues
}
#[test]
fn afperf_edit_end_static_known_joins() {
    let source = include_str!("../../edit/locate.rs");
    let helper = include_str!("../../edit/locate/end_join.rs");
    let issues = inspect(source, helper);
    assert!(
        issues.is_empty(),
        "Edit exact-end static protection: {issues:?}"
    );
    assert!(
        !inspect(
            &source.replace(
                "end_join.editable(start, end)",
                "table.units.iter().any(|_| true)"
            ),
            helper
        )
        .is_empty()
    );
    assert!(!inspect(source, &helper.replace(".min_by_key(", ".max_by_key(")).is_empty());
}
