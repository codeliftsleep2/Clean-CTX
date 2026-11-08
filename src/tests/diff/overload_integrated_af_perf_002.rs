//! Agent MaxHeadRoom: integrated public identity and construction-inclusive costs.
use super::*;
use crate::mcp::{McpState, tools::dispatch_tools_call};
use serde_json::{Value, json};

fn dispatch(state: &McpState, file: &str, root: &tempfile::TempDir, fidelity: Fidelity) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "diff_code_context",
        &json!({"arguments":{
        "filePath":file,"workspaceRoot":root.path().to_string_lossy(),"fidelity":fidelity}}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response
}
#[test]
fn af002_overload_integrated_frozen_public() {
    let _serial = crate::protocol::handler_response_serial();
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/overload_original_public_rows.json")).unwrap();
    assert_eq!(rows.len(), 45);
    for row in rows {
        let n = row["axes"][0].as_u64().unwrap() as usize;
        let w = row["axes"][1].as_u64().unwrap() as usize;
        let mode = row["axes"][2].as_str().unwrap();
        let fidelity: Fidelity = serde_json::from_value(row["fidelity"].clone()).unwrap();
        let before = cs_source(n, w, mode, false);
        let after = cs_source(n, w, mode, true);
        let expected = crate::diff::build_snapshot(&after, fidelity).unwrap();
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("subject.cs");
        std::fs::write(&path, &before).unwrap();
        let file = std::fs::canonicalize(&path)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        config.persistence.enabled = false;
        config
            .additional_roots
            .push(root.path().to_string_lossy().into_owned());
        let state = McpState::new(config);
        *state.graph_bridge_lock() = None;
        let cold = dispatch(&state, &file, &root, fidelity);
        assert_eq!(cold, row["cold"], "complete cold response and metadata");
        std::fs::write(&path, &after).unwrap();
        state.invalidate_source_cache(&file);
        work::reset();
        let mut response = dispatch(&state, &file, &root, fidelity);
        let pairs = work::snapshot().overload.pairs;
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .replace(&file, "<FILE>");
        response["result"]["content"][0]["text"] = json!(text);
        assert_eq!(
            response, row["response"],
            "complete registered response and metadata"
        );
        assert_eq!(serde_json::to_value(pairs).unwrap(), row["pairs"]);
        let key = format!("{file}::{}", fidelity as u8);
        let cache = state.cache_read();
        assert_eq!(cache.get_baseline(&key), Some(&expected));
        assert_eq!(cache.get_baseline_hash(&key), row["hash"].as_str());
        assert_eq!(format!("{expected:?}"), row["snapshot"].as_str().unwrap());
    }
}
fn run(which: usize, b: &CapturedStructure, c: &CapturedStructure) -> Vec<DiffAction> {
    if which == 4 {
        super::super::diff_snapshots(b, c)
    } else {
        compare(if which == 3 { 2 } else { which }, b, c)
    }
}
fn sample(which: usize, b: &CapturedStructure, c: &CapturedStructure) -> u128 {
    let start = std::time::Instant::now();
    let actions = run(which, b, c);
    std::hint::black_box(&actions);
    drop(actions);
    start.elapsed().as_nanos()
}
#[test]
fn af002_overload_integrated_cost() {
    eprintln!(
        "AF002_INTEGRATED_LAYOUT {}",
        json!({"lookup_header":size_of::<crate::diff::overload_lookup::MethodLookup<'_, '_>>(),
        "index":size_of::<usize>(),"scope":"type sizes only; no allocator event/peak claim"})
    );
    for &(n, w, mode, payload) in CASES {
        let (b, c) = fixture(n, w, mode, payload);
        let mut observations = Vec::new();
        work::reset();
        let expected = original::diff_snapshots(&b, &c);
        let expected_pairs = work::snapshot().overload.pairs;
        for which in 0usize..5 {
            let design = if which == 3 {
                Design::Refined
            } else {
                Design::Partition
            };
            work::reset();
            let actual = retrieval::with_design(design, || run(which, &b, &c));
            let observation = work::snapshot();
            assert_eq!(actual, expected);
            assert_eq!(observation.overload.pairs, expected_pairs);
            observations
                .push(json!({"original":observation.diff,"replacement":observation.overload}));
        }
        let mut samples: [Vec<u128>; 5] = std::array::from_fn(|_| Vec::new());
        work::unobserved(|| {
            for round in 0usize..27 {
                for offset in 0usize..5 {
                    let which = (round + offset) % 5;
                    let design = if which == 3 {
                        Design::Refined
                    } else {
                        Design::Partition
                    };
                    let ns = retrieval::with_design(design, || sample(which, &b, &c));
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
            "AF002_INTEGRATED_COST {}",
            json!({"axes":[n,w,mode,payload],
            "order":["observed_original","rejected_fifo","signature_arrays","approved_test_refinement","integrated_production"],
            "median_ns":medians,"samples_ns":samples,"work":observations,
            "scope":"debug complete comparator grouping/setup/query/actions/drop; shared field observers and disabled updates; no public/release/allocator claim"})
        );
    }
}
