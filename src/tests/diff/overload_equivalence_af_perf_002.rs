//! Agent MaxHeadRoom: frozen original pairing and complete output equivalence.
use super::*;
use crate::compression::Fidelity;
use crate::diff::snapshot::DeclarationEvidence;
use crate::diff::{diff_summary, format_diff};
use crate::work_observations_af_perf_002 as work;

#[path = "original_matcher_af_perf_002.rs"]
pub(crate) mod original;

fn method(id: usize) -> CapturedMethod {
    let mut m = CapturedMethod {
        sig: "Run(x)".into(),
        declaration: None,
        markers: vec![],
        body: None,
    };
    match id {
        0 => {}
        1 => m.body = Some(String::new()),
        2 => m.body = Some("α return 1;".into()),
        3 => m.body = Some("α return 2;".into()),
        4 => m.markers = vec!["loop".into(), "guard".into()],
        5 => {
            m.declaration = Some(DeclarationEvidence {
                return_type: String::new(),
                attributes: vec![],
                modifiers: vec![],
            })
        }
        6 => {
            m.declaration = Some(DeclarationEvidence {
                return_type: "int".into(),
                attributes: vec!["A".into(), "A".into()],
                modifiers: vec!["public".into()],
            })
        }
        7 => m.sig = "Run(other)".into(),
        _ => m.sig = "Other()".into(),
    }
    m
}
fn snapshot(methods: Vec<CapturedMethod>, orphan: bool) -> CapturedStructure {
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
fn check(base: &CapturedStructure, cur: &CapturedStructure) {
    work::reset();
    let expected = original::diff_snapshots(base, cur);
    let pairs = work::snapshot().overload.pairs;
    work::reset();
    let actual = diff_snapshots(base, cur);
    assert_eq!(
        work::snapshot().overload.pairs,
        pairs,
        "selected occurrence-index trace"
    );
    assert_eq!(
        actual, expected,
        "complete ordered actions/reasons/previous details"
    );
    assert_eq!(diff_summary(&actual), diff_summary(&expected));
    for fidelity in [
        Fidelity::Low,
        Fidelity::Medium,
        Fidelity::High,
        Fidelity::Edit,
        Fidelity::Verbatim,
    ] {
        assert_eq!(
            format_diff(&actual, fidelity),
            format_diff(&expected, fidelity)
        );
    }
}
fn sequences(alphabet: usize, depth: usize) -> Vec<Vec<CapturedMethod>> {
    let mut all = vec![vec![]];
    let mut level = vec![vec![]];
    for _ in 0..depth {
        let mut next = Vec::new();
        for prefix in &level {
            for id in 0..alphabet {
                let mut sequence = prefix.clone();
                sequence.push(method(id));
                next.push(sequence);
            }
        }
        all.extend(next.iter().cloned());
        level = next;
    }
    all
}
#[test]
fn af002_overload_frozen_equivalence() {
    for (alphabet, depth) in [(4, 3), (9, 2)] {
        let sequences = sequences(alphabet, depth);
        for orphan in [false, true] {
            for b in &sequences {
                for c in &sequences {
                    check(&snapshot(b.clone(), orphan), &snapshot(c.clone(), orphan));
                }
            }
        }
    }
    for n in [1, 8, 32, 128] {
        for orphan in [false, true] {
            let base: Vec<_> = (0..n)
                .map(|i| {
                    let mut m = method(i % 9);
                    m.body = Some(format!("{}{}", "共有".repeat(1024), i % 3));
                    m
                })
                .collect();
            let mut cur = base.clone();
            cur.reverse();
            cur.push(method(3));
            check(&snapshot(base, orphan), &snapshot(cur, orphan));
        }
    }
    // Exact reservation must precede any same-signature pairing.
    check(
        &snapshot(vec![method(2), method(3)], false),
        &snapshot(vec![method(3), method(1)], false),
    );
    work::reset();
    diff_snapshots(
        &snapshot(vec![method(2), method(3)], false),
        &snapshot(vec![method(3), method(1)], false),
    );
    assert_eq!(
        work::snapshot().overload.pairs,
        vec![("Run".into(), 0, 1, 0), ("Run".into(), 1, 0, 1)]
    );
}
