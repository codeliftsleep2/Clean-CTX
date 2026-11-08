//! Agent MaxHeadRoom: complete comparison cost and finite scan/phase protection.
use super::overload_equivalence::original;
use super::*;
use crate::work_observations_af_perf_002 as work;

fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn protected(source: &str, helper: &str) -> bool {
    let Some(a) = source.find("fn diff_methods(") else {
        return false;
    };
    let Some(b) = source[a..].find("fn push_method_comparison(") else {
        return false;
    };
    let block = compact(&source[a..a + b]);
    let h = compact(helper);
    let create = block.find("letmutlookup=MethodLookup::new(g2);");
    let exact = block.find("lookup.exact(base_method,&cur_used)");
    let phase = block.find("//Completeglobalexactphasebeforeanysignatureconsumption.");
    let skip = block.find("ifbase_used[base_idx]{continue;}");
    let sig = block.find("lookup.signature(&base_method.sig,&cur_used)");
    let drop_lookup = block.find("drop(lookup);");
    matches!((create,exact,phase,skip,sig,drop_lookup),(Some(a),Some(b),Some(c),Some(d),Some(e),Some(f)) if a<b&&b<c&&c<d&&d<e&&e<f)
        && block
            .matches("for(base_idx,base_method)ing1.iter().enumerate()")
            .count()
            == 2
        && !block.contains(".find(")
        && h.contains("ifcurrent.len()==1{None}")
        && h.contains("Vec::with_capacity(current.len())")
        && h.contains("indices.extend(0..current.len())")
        && h.contains("signature_cmp(&current[a].sig,&current[b].sig).then(a.cmp(&b))")
        && h.contains("index.indices[a..b].iter().find(|&&i|")
        && h.contains("}).copied()")
        && h.contains("!used[i]&&method_equal(self.current[i],base)")
        && h.contains("!used[0]&&method_equal(self.current[0],base)")
        && h.contains("!used[0]&&signature_equal(&self.current[0].sig,signature)")
        && h.contains("if!used[i]{returnSome(i);}")
        && h.contains("index.cursor[a]+=1")
        && h.contains("#[cfg(not(test))]{a==b}")
        && !h.contains("HashMap")
        && !h.contains("BTreeMap")
}
#[test]
fn afperf_overload_static() {
    let source = include_str!("../../diff/differ.rs");
    let helper = include_str!("../../diff/overload_lookup.rs");
    assert!(
        protected(source, helper),
        "Overload retrieval static protection"
    );
    assert!(!protected(
        &source.replace(
            "// Complete global exact phase before any signature consumption.",
            ""
        ),
        helper
    ));
    assert!(!protected(
        source,
        &helper.replace("a == b", "a.sig == b.sig")
    ));
    assert!(!protected(
        source,
        &helper.replace("current.len() == 1", "current.len() <= 8")
    ));
    assert!(!protected(
        &source.replace(
            "lookup.exact(base_method, &cur_used)",
            "g2.iter().enumerate().find(|_| true)"
        ),
        helper
    ));
    assert!(!protected(
        &source.replace(
            "lookup.signature(&base_method.sig, &cur_used)",
            "g2.iter().enumerate().find(|_| true)"
        ),
        helper
    ));
}
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
    (
        CapturedStructure {
            orphan_methods: methods,
            ..Default::default()
        },
        CapturedStructure {
            orphan_methods: changed,
            ..Default::default()
        },
    )
}
fn sample(base: &CapturedStructure, cur: &CapturedStructure, old: bool) -> u128 {
    let start = std::time::Instant::now();
    let output = if old {
        original::diff_snapshots(base, cur)
    } else {
        diff_snapshots(base, cur)
    };
    std::hint::black_box(&output);
    drop(output);
    start.elapsed().as_nanos()
}
#[test]
fn afperf_overload_paired_cost() {
    for (n, width, mode, payload) in [
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
    ] {
        let (base, cur) = fixture(n, width, mode, payload);
        work::reset();
        let expected = original::diff_snapshots(&base, &cur);
        let old = work::snapshot();
        work::reset();
        let actual = diff_snapshots(&base, &cur);
        let new = work::snapshot();
        assert_eq!(actual, expected);
        assert_eq!(new.overload.pairs, old.overload.pairs);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        work::unobserved(|| {
            for _ in 0..3 {
                sample(&base, &cur, true);
                sample(&base, &cur, false);
            }
            for iteration in 0..24 {
                if iteration % 2 == 0 {
                    a.push(sample(&base, &cur, true));
                    b.push(sample(&base, &cur, false));
                } else {
                    b.push(sample(&base, &cur, false));
                    a.push(sample(&base, &cur, true));
                }
            }
        });
        a.sort_unstable();
        b.sort_unstable();
        eprintln!(
            "AF002_OVERLOAD_COST {}",
            serde_json::json!({"axes":[n,width,mode,payload],
            "original_median_ns":a[12],"replacement_median_ns":b[12],"original_work":old.diff,
            "replacement_work":new.overload,"scope":"debug full snapshot comparator setup/query/actions/drop; common input preparation excluded; both original predicate and replacement observer calls remain with updates disabled; no allocator or public-latency attribution"})
        );
    }
}
