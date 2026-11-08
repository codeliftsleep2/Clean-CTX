//! Agent MaxHeadRoom: one finite source-form investigation; production bytes untouched.
use super::*;
#[path = "overload_form_combined_comparator.rs"]
mod combined;
#[path = "overload_form_control_comparator.rs"]
mod control;
#[path = "overload_form_find_copied_comparator.rs"]
mod find_copied;
#[path = "overload_form_combined_helper.rs"]
mod helper_combined;
#[path = "overload_form_control_helper.rs"]
mod helper_control;
#[path = "overload_form_find_copied_helper.rs"]
mod helper_find_copied;
#[path = "overload_form_inline_helper.rs"]
mod helper_inline;
#[path = "overload_form_inline_comparator.rs"]
mod inline;

fn source_body(source: &str) -> String {
    let normalized = source.replace("\r\n", "\n");
    normalized
        .split("#[cfg(test)]\n#[path")
        .next()
        .unwrap()
        .to_string()
}
fn compact(source: &str) -> String {
    // rustfmt may reorder top-level imports after changing module paths.
    // Normalize only import statement order, never function/control-flow order.
    let mut imports = Vec::new();
    let mut pending = String::new();
    let mut body = String::new();
    for line in source.lines() {
        if !pending.is_empty() || line.starts_with("use ") {
            pending.push_str(line);
            if line.contains(';') {
                imports.push(std::mem::take(&mut pending));
            }
        } else {
            body.push_str(line);
        }
    }
    assert!(pending.is_empty());
    for statement in &mut imports {
        *statement = statement
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .replace(",};", "};"); // Optional trailing comma in a rustfmt use list only.
    }
    imports.sort();
    for statement in imports {
        body.push_str(&statement);
    }
    body.chars().filter(|c| !c.is_whitespace()).collect()
}
#[test]
fn af002_overload_source_forms_identity() {
    let helper = compact(
        &source_body(include_str!(
            "fixtures/overload_source_forms_original_helper.txt"
        ))
        .replace("use super::snapshot::", "use crate::diff::snapshot::"),
    );
    let comparator = source_body(include_str!(
        "fixtures/overload_source_forms_original_comparator.txt"
    ))
    .replace("use super::action::", "use crate::diff::action::")
    .replace("use super::keys::", "use crate::diff::keys::")
    .replace("use super::snapshot::", "use crate::diff::snapshot::");
    for (name, helper_copy, comparator_copy, iterator, inlining) in [
        (
            "control",
            include_str!("overload_form_control_helper.rs"),
            include_str!("overload_form_control_comparator.rs"),
            false,
            false,
        ),
        (
            "find_copied",
            include_str!("overload_form_find_copied_helper.rs"),
            include_str!("overload_form_find_copied_comparator.rs"),
            true,
            false,
        ),
        (
            "inline",
            include_str!("overload_form_inline_helper.rs"),
            include_str!("overload_form_inline_comparator.rs"),
            false,
            true,
        ),
        (
            "combined",
            include_str!("overload_form_combined_helper.rs"),
            include_str!("overload_form_combined_comparator.rs"),
            true,
            true,
        ),
    ] {
        let mut expected = helper.clone();
        if iterator {
            expected = expected
                .replace(
                    "index.indices[a..b].iter().copied().find(|&i|{",
                    "index.indices[a..b].iter().find(|&&i|{",
                )
                .replace(
                    "})}pub(super)fnsignature(",
                    "}).copied()}pub(super)fnsignature(",
                );
        }
        if inlining {
            for name in ["signature_cmp", "method_equal", "signature_equal"] {
                expected = expected.replace(
                    &format!("fn{name}("),
                    &format!("#[inline(always)]fn{name}("),
                );
            }
        }
        assert_eq!(
            compact(helper_copy),
            expected,
            "only authorized helper source substitution: {name}"
        );
        let expected = comparator.replace(
            "use super::overload_lookup::MethodLookup;",
            &format!("use super::helper_{name}::MethodLookup;"),
        );
        assert_eq!(
            compact(comparator_copy),
            compact(&expected),
            "identical complete comparator: {name}"
        );
    }
}

fn variant(which: usize, b: &CapturedStructure, c: &CapturedStructure) -> Vec<DiffAction> {
    match which {
        0 => observed_original::diff_snapshots(b, c),
        1 => candidate::diff_snapshots(b, c), // fixed Design::Refined, direct loop reference
        2 => super::super::diff_snapshots(b, c),
        3 => control::diff_snapshots(b, c),
        4 => find_copied::diff_snapshots(b, c),
        5 => inline::diff_snapshots(b, c),
        6 => combined::diff_snapshots(b, c),
        _ => unreachable!(),
    }
}
fn verify(b: &CapturedStructure, c: &CapturedStructure) {
    work::reset();
    let expected = original::diff_snapshots(b, c);
    let pairs = work::snapshot().overload.pairs;
    work::reset();
    variant(2, b, c);
    let actual_work = work::snapshot().overload;
    for which in 0..7 {
        work::reset();
        let actions = retrieval::with_design(Design::Refined, || variant(which, b, c));
        let observed = work::snapshot().overload;
        assert_eq!(
            observed.pairs, pairs,
            "source form {which} occurrence indices"
        );
        assert_eq!(actions, expected, "source form {which} complete actions");
        assert_eq!(diff_summary(&actions), diff_summary(&expected));
        for f in [
            Fidelity::Low,
            Fidelity::Medium,
            Fidelity::High,
            Fidelity::Edit,
            Fidelity::Verbatim,
        ] {
            assert_eq!(format_diff(&actions, f), format_diff(&expected, f));
        }
        if which >= 3 {
            assert_eq!(
                observed, actual_work,
                "source form {which} identical enabled selection/equality/signature/payload/capacity observations"
            );
        }
    }
}
#[test]
fn af002_overload_source_forms_semantics() {
    let mut sequences = vec![vec![]];
    for i in 0..9 {
        sequences.push(vec![method(i)]);
        for j in 0..9 {
            sequences.push(vec![method(i), method(j)]);
        }
    }
    for orphan in [false, true] {
        for b in &sequences {
            for c in &sequences {
                verify(&snap(b.clone(), orphan), &snap(c.clone(), orphan));
            }
        }
    }
    for &(n, w, mode, payload) in CASES {
        let (b, c) = fixture(n, w, mode, payload);
        verify(&b, &c);
    }
}
#[test]
fn af002_overload_source_forms_public_oracle() {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/overload_original_public_rows.json")).unwrap();
    assert_eq!(rows.len(), 45);
    for row in rows {
        let n = row["axes"][0].as_u64().unwrap() as usize;
        let w = row["axes"][1].as_u64().unwrap() as usize;
        let mode = row["axes"][2].as_str().unwrap();
        let f: Fidelity = serde_json::from_value(row["fidelity"].clone()).unwrap();
        let b = crate::diff::build_snapshot(&cs_source(n, w, mode, false), f).unwrap();
        let after = cs_source(n, w, mode, true);
        let c = crate::diff::build_snapshot(&after, f).unwrap();
        assert_eq!(format!("{c:?}"), row["snapshot"].as_str().unwrap());
        assert_eq!(
            crate::cache::content_hash(after.as_bytes()),
            row["hash"].as_str().unwrap()
        );
        for which in 0..7 {
            work::reset();
            let actions = retrieval::with_design(Design::Refined, || variant(which, &b, &c));
            assert_eq!(serde_json::to_value(&actions).unwrap(), row["actions"]);
            assert_eq!(
                serde_json::to_value(work::snapshot().overload.pairs).unwrap(),
                row["pairs"]
            );
            let (a, r, m, u) = diff_summary(&actions);
            let text = format!(
                "// --- AST Diff: <FILE> ---\n// +{a} -{r} ~{m} ={u} (classes/methods/fields/imports)\n{}",
                format_diff(&actions, f)
            );
            assert_eq!(
                serde_json::json!({"id":1,"jsonrpc":"2.0","result":{"content":[{"type":"text","text":text}]}}),
                row["response"]
            );
        }
    }
}
fn timed(which: usize, b: &CapturedStructure, c: &CapturedStructure) -> u128 {
    let start = std::time::Instant::now();
    let actions = variant(which, b, c);
    std::hint::black_box(&actions);
    drop(actions);
    start.elapsed().as_nanos()
}
#[test]
fn af002_overload_source_forms_cost() {
    for &(n, w, mode, payload) in CASES {
        let (b, c) = fixture(n, w, mode, payload);
        verify(&b, &c);
        let mut observations = Vec::new();
        for which in 0..7 {
            work::reset();
            retrieval::with_design(Design::Refined, || {
                variant(which, &b, &c);
            });
            let counts = work::snapshot();
            observations
                .push(serde_json::json!({"original":counts.diff,"selection":counts.overload}));
        }
        // Two fixed rotations retain 24 samples each to inspect within-run repeatability.
        let mut rounds = Vec::new();
        for block in 0usize..2 {
            let mut samples: [Vec<u128>; 7] = std::array::from_fn(|_| Vec::new());
            work::unobserved(|| {
                retrieval::with_design(Design::Refined, || {
                    for round in 0usize..27 {
                        for offset in 0usize..7 {
                            let which = (round + offset + block * 3) % 7;
                            let ns = timed(which, &b, &c);
                            if round >= 3 {
                                samples[which].push(ns);
                            }
                        }
                    }
                })
            });
            let medians: Vec<_> = samples
                .iter_mut()
                .map(|s| {
                    s.sort_unstable();
                    s[12]
                })
                .collect();
            rounds.push(serde_json::json!({"median_ns":medians,"samples_ns":samples}));
        }
        eprintln!(
            "AF002_SOURCE_FORM_COST {}",
            serde_json::json!({"axes":[n,w,mode,payload],
            "order":["observed_original","approved_loop_refinement","active_production","identical_source_control","find_then_copied","inline_wrappers","find_then_copied_inline_wrappers"],
            "blocks":rounds,"work":observations,"scope":"test-only source-equivalent helper/comparator copies; full setup/query/actions/drop; exact work equality enforced; two finite rotating blocks of 24 debug samples; no release/public or allocator claims"})
        );
    }
}
