//! Characterization of exact raw-source count requests at two actual production boundaries.
use crate::work_observations_af_perf_002 as work;
use serde_json::json;

#[test]
fn af002_compression_public_raw_count_growth() {
    let _serial = crate::protocol::handler_response_serial();
    for n in [1, 2, 4, 8, 16, 64, 256] {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("workload.ts");
        let source = format!(
            "// {}\nexport class Workload {{ run() {{ return 1; }} }}",
            "ordinary source words ".repeat(n)
        );
        std::fs::write(&file, &source).unwrap();
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        let state = crate::mcp::McpState::new(config);
        *state.graph_bridge_lock() = None;
        work::reset();
        crate::protocol::captured_responses().clear();
        let start = std::time::Instant::now();
        crate::mcp::tools::dispatch_tools_call(
            &json!(1),
            "compress_code_context",
            &json!({"arguments": {
                "filePath": file, "workspaceRoot": root.path(), "fidelity": "high"
            }}),
            &state,
        );
        let response = crate::protocol::captured_responses()
            .pop()
            .expect("actual public response");
        assert!(response.get("error").is_none(), "{response}");
        assert!(response.get("result").is_some(), "{response}");
        let counts = work::snapshot();
        let hash = crate::cache::content_hash(source.as_bytes());
        let helper = counts.token_count_inputs[0]
            .get(&hash)
            .copied()
            .unwrap_or(0);
        let economic = counts.token_count_inputs[1]
            .get(&hash)
            .copied()
            .unwrap_or(0);
        assert!(helper > 0, "public handler raw-count prerequisite");
        assert_eq!(
            economic, 0,
            "corrected primary IR path reuses the handler raw count"
        );
        eprintln!(
            "AF002_COMPRESSION_RAW n={n} bytes={} helper_raw_requests={helper} economics_raw_requests={economic} all_boundary_bytes={:?} us={} accounting=test-observation-hash-cost-included",
            source.len(),
            counts.token_count_bytes,
            start.elapsed().as_micros()
        );
    }
}

#[test]
fn c1_budget_public_raw_count_once() {
    let _serial = crate::protocol::handler_response_serial();
    let mut failures = Vec::new();
    for n in [1, 2, 4, 8, 16, 64, 256] {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("workload.ts");
        let source = format!(
            "// {}\nexport class Workload {{ run() {{ return 1; }} }}",
            "ordinary source words ".repeat(n)
        );
        std::fs::write(&file, &source).unwrap();
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        let state = crate::mcp::McpState::new(config);
        *state.graph_bridge_lock() = None;
        work::reset();
        crate::protocol::captured_responses().clear();
        let start = std::time::Instant::now();
        crate::mcp::tools::dispatch_tools_call(
            &json!(1),
            "compress_code_context",
            &json!({"arguments": {
                "filePath": file, "workspaceRoot": root.path(), "fidelity": "high"
            }}),
            &state,
        );
        let response = crate::protocol::captured_responses()
            .pop()
            .expect("actual public response");
        assert!(response.get("error").is_none(), "{response}");
        assert!(response.get("result").is_some(), "{response}");
        let counts = work::snapshot();
        let hash = crate::cache::content_hash(source.as_bytes());
        let helper = counts.token_count_inputs[0]
            .get(&hash)
            .copied()
            .unwrap_or(0);
        let economic = counts.token_count_inputs[1]
            .get(&hash)
            .copied()
            .unwrap_or(0);
        assert!(helper > 0, "public handler raw-count prerequisite");
        if helper != 1 || economic != 0 {
            failures.push((n, helper, economic));
        }
        let root_text = root.path().to_string_lossy();
        let normalized = serde_json::to_string(&response["result"]).unwrap().replace(
            &serde_json::to_string(root_text.as_ref()).unwrap()
                [1..serde_json::to_string(root_text.as_ref()).unwrap().len() - 1],
            "<ROOT>",
        );
        eprintln!("C1_PUBLIC_RESPONSE n={n} result={normalized}");
        eprintln!(
            "AF002_COMPRESSION_RAW n={n} bytes={} helper_raw_requests={helper} economics_raw_requests={economic} all_boundary_bytes={:?} us={} accounting=test-observation-hash-cost-included",
            source.len(),
            counts.token_count_bytes,
            start.elapsed().as_micros()
        );
    }
    assert!(
        failures.is_empty(),
        "C1 redundant raw recount: {failures:?}"
    );
}
