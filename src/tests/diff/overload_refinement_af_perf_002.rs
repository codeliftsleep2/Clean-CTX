//! Agent MaxHeadRoom: exact capacity and one-current-occurrence investigation only.
use super::*;

#[test]
fn af002_overload_refinement_semantics() {
    semantics(&[Design::Capacity, Design::Refined]);
    // First baseline fails equality, later baseline reserves the sole exact current.
    for orphan in [false, true] {
        check(
            &snap(vec![method(2), method(3)], orphan),
            &snap(vec![method(3)], orphan),
            Design::Refined,
        );
        check(
            &snap(vec![method(2), method(2), method(3)], orphan),
            &snap(vec![method(3)], orphan),
            Design::Refined,
        );
        check(
            &snap(vec![method(2), method(1)], orphan),
            &snap(vec![method(3)], orphan),
            Design::Refined,
        );
    }
}
#[test]
fn af002_overload_refinement_public_oracle() {
    public_oracle(&[Design::Capacity, Design::Refined]);
}
#[test]
fn af002_overload_refinement_costs() {
    eprintln!("AF002_REFINE_LAYOUT {}", retrieval::layout());
    for &(n, w, mode, payload) in CASES {
        let (b, c) = fixture(n, w, mode, payload);
        let mut observations = Vec::new();
        for which in 0usize..5 {
            let design = match which {
                3 => Design::Capacity,
                4 => Design::Refined,
                _ => Design::Partition,
            };
            work::reset();
            let actual = retrieval::with_design(design, || compare(which, &b, &c));
            let observed = work::snapshot();
            let expected = work::unobserved(|| original::diff_snapshots(&b, &c));
            assert_eq!(actual, expected);
            observations.push(
                serde_json::json!({"original":observed.diff,"replacement":observed.overload}),
            );
        }
        let mut samples: [Vec<u128>; 5] = std::array::from_fn(|_| Vec::new());
        work::unobserved(|| {
            for round in 0usize..27 {
                for offset in 0usize..5 {
                    let which = (round + offset) % 5;
                    let design = match which {
                        3 => Design::Capacity,
                        4 => Design::Refined,
                        _ => Design::Partition,
                    };
                    let ns = retrieval::with_design(design, || measured(which, &b, &c));
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
            "AF002_REFINE_COST {}",
            serde_json::json!({"axes":[n,w,mode,payload],
            "order":["observed_original","unaccepted_fifo","signature_partition","exact_capacity_only","refined_partition"],
            "median_ns":medians,"samples_ns":samples,"work":observations,
            "scope":"debug complete comparator setup/query/actions/drop; common input preparation and design selection outside timing; all original/candidate probes retained with updates disabled; capacities are not allocator events"})
        );
    }
}
