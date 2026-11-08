//! Agent MaxHeadRoom: test-only retrieval design comparison, not production closure.
use super::overload_equivalence::original;
use crate::compression::Fidelity;
use crate::diff::snapshot::DeclarationEvidence;
use crate::diff::{
    CapturedClass, CapturedMethod, CapturedStructure, DiffAction, diff_summary, format_diff,
};
use crate::work_observations_af_perf_002 as work;

#[path = "overload_alternative_comparator_af_perf_002.rs"]
mod candidate;
#[path = "rejected_fifo_comparator_af_perf_002.rs"]
mod fifo_reference;
#[path = "overload_observed_original_af_perf_002.rs"]
mod observed_original;
#[path = "rejected_fifo_lookup_af_perf_002.rs"]
mod rejected_fifo;
#[path = "overload_alternative_lookup_af_perf_002.rs"]
pub(crate) mod retrieval;
use retrieval::Design;

#[path = "overload_integrated_af_perf_002.rs"]
mod integrated;
#[path = "overload_refinement_af_perf_002.rs"]
mod refinement;
#[path = "overload_source_forms_af_perf_002.rs"]
mod source_forms;

fn method(i: usize) -> CapturedMethod {
    let mut m = CapturedMethod {
        sig: "Run(α)".into(),
        declaration: None,
        markers: vec![],
        body: None,
    };
    match i {
        0 => {}
        1 => m.body = Some(String::new()),
        2 => m.body = Some("共有x".into()),
        3 => m.body = Some("共有y".into()),
        4 => {
            m.declaration = Some(DeclarationEvidence {
                return_type: "int".into(),
                attributes: vec!["A".into(), "A".into()],
                modifiers: vec!["public".into()],
            })
        }
        5 => m.markers = vec!["b".into(), "a".into()],
        6 => m.markers = vec!["a".into(), "b".into()],
        7 => m.sig = "Run(other)".into(),
        _ => {
            m.declaration = Some(DeclarationEvidence {
                return_type: String::new(),
                attributes: vec![],
                modifiers: vec![],
            })
        }
    }
    m
}
fn snap(methods: Vec<CapturedMethod>, orphan: bool) -> CapturedStructure {
    if orphan {
        CapturedStructure {
            orphan_methods: methods,
            ..Default::default()
        }
    } else {
        CapturedStructure {
            classes: vec![CapturedClass {
                name: "Probe".into(),
                class_meta: String::new(),
                fields: vec![],
                methods,
            }],
            ..Default::default()
        }
    }
}
fn check(base: &CapturedStructure, cur: &CapturedStructure, design: Design) {
    work::reset();
    let expected = original::diff_snapshots(base, cur);
    let pairs = work::snapshot().overload.pairs;
    work::reset();
    let actual = retrieval::with_design(design, || candidate::diff_snapshots(base, cur));
    assert_eq!(
        work::snapshot().overload.pairs,
        pairs,
        "selected original indices: {design:?}"
    );
    assert_eq!(actual, expected, "complete ordered actions: {design:?}");
    assert_eq!(
        observed_original::diff_snapshots(base, cur),
        expected,
        "detailed observed original equivalence"
    );
    assert_eq!(diff_summary(&actual), diff_summary(&expected));
    for f in [
        Fidelity::Low,
        Fidelity::Medium,
        Fidelity::High,
        Fidelity::Edit,
        Fidelity::Verbatim,
    ] {
        assert_eq!(format_diff(&actual, f), format_diff(&expected, f));
    }
}
#[test]
fn af002_overload_alternative_semantics() {
    semantics(&[
        Design::Partition,
        Design::Sorted,
        Design::Hash,
        Design::Collisions,
    ]);
}
fn semantics(designs: &[Design]) {
    let mut sequences = vec![vec![]];
    for i in 0..9 {
        sequences.push(vec![method(i)]);
        for j in 0..9 {
            sequences.push(vec![method(i), method(j)]);
        }
    }
    for &d in designs {
        for orphan in [false, true] {
            for b in &sequences {
                for c in &sequences {
                    check(&snap(b.clone(), orphan), &snap(c.clone(), orphan), d);
                }
            }
        }
        check(
            &snap(vec![method(2), method(3)], false),
            &snap(vec![method(3), method(1)], false),
            d,
        );
        for (n, w, mode, payload) in CASES {
            let (b, c) = fixture(*n, *w, mode, *payload);
            check(&b, &c, d);
        }
    }
}

fn cs_source(methods: usize, width: usize, mode: &str, changed: bool) -> String {
    let mut source = format!(
        "// phase {}\nusing System;\nclass Subject {{\n",
        if changed { "new" } else { "old" }
    );
    let mut order: Vec<_> = (0..methods).collect();
    if changed && mode == "reordered" {
        order.reverse();
    }
    for i in order {
        let value = if changed && (mode == "all-body" || (mode == "one-body" && i == 0)) {
            2
        } else {
            1
        };
        source.push_str(&format!(
            " public int Run{}(Type{i} value) {{ return {value}; }}\n",
            i / width
        ));
    }
    source.push_str("}\n");
    source
}
#[test]
fn af002_overload_alternative_public_oracle() {
    public_oracle(&[
        Design::Partition,
        Design::Sorted,
        Design::Hash,
        Design::Collisions,
    ]);
}
fn public_oracle(designs: &[Design]) {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/overload_original_public_rows.json")).unwrap();
    assert_eq!(rows.len(), 45);
    for row in rows {
        let n = row["axes"][0].as_u64().unwrap() as usize;
        let w = row["axes"][1].as_u64().unwrap() as usize;
        let mode = row["axes"][2].as_str().unwrap();
        let fidelity: Fidelity = serde_json::from_value(row["fidelity"].clone()).unwrap();
        let before = cs_source(n, w, mode, false);
        let after = cs_source(n, w, mode, true);
        let b = crate::diff::build_snapshot(&before, fidelity).unwrap();
        let c = crate::diff::build_snapshot(&after, fidelity).unwrap();
        assert_eq!(format!("{c:?}"), row["snapshot"].as_str().unwrap());
        assert_eq!(
            crate::cache::content_hash(after.as_bytes()),
            row["hash"].as_str().unwrap()
        );
        for &design in designs {
            work::reset();
            let actions = retrieval::with_design(design, || candidate::diff_snapshots(&b, &c));
            assert_eq!(serde_json::to_value(&actions).unwrap(), row["actions"]);
            assert_eq!(
                serde_json::to_value(work::snapshot().overload.pairs).unwrap(),
                row["pairs"]
            );
            let (a, r, m, u) = diff_summary(&actions);
            let text = format!(
                "// --- AST Diff: <FILE> ---\n// +{a} -{r} ~{m} ={u} (classes/methods/fields/imports)\n{}",
                format_diff(&actions, fidelity)
            );
            let response = serde_json::json!({"id":1,"jsonrpc":"2.0","result":{"content":[{"type":"text","text":text}]}});
            assert_eq!(
                response, row["response"],
                "complete frozen public response projection"
            );
        }
    }
}

const CASES: &[(usize, usize, &str, usize)] = &[
    (0, 1, "exact", 0),
    (1, 1, "exact", 0),
    (8, 8, "body", 0),
    (128, 1, "body", 0),
    (128, 8, "body", 0),
    (128, 128, "body", 0),
    (128, 128, "exact", 0),
    (128, 128, "duplicates", 0),
    (128, 128, "same-signature", 0),
    (128, 128, "reordered", 0),
    (128, 128, "sparse", 0),
    (64, 64, "duplicates", 4096),
    (64, 64, "same-signature", 4096),
    (512, 1, "exact", 0),
];
fn fixture(
    n: usize,
    width: usize,
    mode: &str,
    payload: usize,
) -> (CapturedStructure, CapturedStructure) {
    let methods: Vec<_> = (0..n)
        .map(|i| CapturedMethod {
            sig: format!(
                "Run{}({})",
                i / width,
                if mode == "duplicates" || mode == "same-signature" {
                    0
                } else {
                    i
                }
            ),
            declaration: None,
            markers: vec![],
            body: Some(format!(
                "{}{}",
                "共有".repeat(payload),
                if mode == "duplicates" { 0 } else { i }
            )),
        })
        .collect();
    let mut changed = methods.clone();
    match mode {
        "body" | "same-signature" => {
            for m in &mut changed {
                m.body.as_mut().unwrap().push('!');
            }
        }
        "reordered" => changed.reverse(),
        "sparse" => {
            for m in changed.iter_mut().skip(1) {
                m.sig.push_str("other");
            }
        }
        _ => {}
    }
    (snap(methods, true), snap(changed, true))
}
fn compare(which: usize, b: &CapturedStructure, c: &CapturedStructure) -> Vec<DiffAction> {
    match which {
        0 => observed_original::diff_snapshots(b, c),
        1 => fifo_reference::diff_snapshots(b, c),
        _ => candidate::diff_snapshots(b, c),
    }
}
fn measured(which: usize, b: &CapturedStructure, c: &CapturedStructure) -> u128 {
    let start = std::time::Instant::now();
    let out = compare(which, b, c);
    std::hint::black_box(&out);
    drop(out);
    start.elapsed().as_nanos()
}
#[test]
fn af002_overload_alternative_costs() {
    eprintln!("AF002_ALT_LAYOUT {}", retrieval::layout());
    for &(n, w, mode, payload) in CASES {
        let (b, c) = fixture(n, w, mode, payload);
        let designs = [Design::Partition, Design::Sorted, Design::Hash];
        let mut observations = Vec::new();
        for which in 0usize..5 {
            let d = designs[which.saturating_sub(2).min(2)];
            work::reset();
            retrieval::with_design(d, || {
                compare(which, &b, &c);
            });
            let observation = work::snapshot();
            observations
                .push(serde_json::json!({"original":observation.diff,"new":observation.overload}));
        }
        let mut samples: [Vec<u128>; 5] = std::array::from_fn(|_| Vec::new());
        work::unobserved(|| {
            for round in 0usize..27 {
                for offset in 0usize..5 {
                    let which = (round + offset) % 5;
                    let d = designs[which.saturating_sub(2).min(2)];
                    let ns = retrieval::with_design(d, || measured(which, &b, &c));
                    if round >= 3 {
                        samples[which].push(ns);
                    }
                }
            }
        });
        let medians: Vec<_> = samples
            .iter_mut()
            .map(|s| {
                s.sort_unstable();
                s[12]
            })
            .collect();
        eprintln!(
            "AF002_ALT_COST {}",
            serde_json::json!({"axes":[n,w,mode,payload],
            "order":["original","unaccepted_fifo","signature_partition","sorted_exact","collision_checked_hash"],
            "median_ns":medians,"work":observations,"scope":"test-only full comparator setup/queries/actions/drop; common input preparation and design selection outside timer; all implementations retain disabled diagnostic probes; no allocator/public-latency claim"})
        );
    }
}
