//! Public context_stats format work; no persistence/proxy traffic in growth fixture.
use crate::work_observations_af_perf_002 as work;
use serde_json::{Value, json};
#[test]
fn af002_stats_format_and_file_growth() {
    let _serial = crate::protocol::handler_response_serial();
    let mut axes: Vec<_> = [1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|n| (n, 1))
        .collect();
    axes.extend([2, 4, 8, 16, 64, 256].into_iter().map(|n| (1, n)));
    axes.push((16, 16));
    for (files, invocations) in axes {
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        config.persistence.enabled = false;
        let mut state = crate::mcp::McpState::new(config);
        state.proxy_port = 0;
        *state.graph_bridge_lock() = None;
        {
            let mut stats = state.session_stats_lock();
            for i in 0..files {
                stats.record_compression(
                    &format!("stats://file-{i}"),
                    100,
                    50,
                    "high",
                    false,
                    "full",
                    None,
                    "compression",
                );
            }
        }
        let before_files =
            serde_json::to_value(state.session_stats_lock().all_file_stats()).unwrap();
        let before_domains =
            serde_json::to_value(state.session_stats_lock().domain_breakdown()).unwrap();
        for format in ["json", "text"] {
            work::reset();
            let mut total_us = 0;
            for _ in 0..invocations {
                crate::protocol::captured_responses().clear();
                let start = std::time::Instant::now();
                crate::mcp::tools::dispatch_tools_call(
                    &json!(1),
                    "context_stats",
                    &json!({"arguments":{"format":format}}),
                    &state,
                );
                total_us += start.elapsed().as_micros();
                let response = crate::protocol::captured_responses().pop().unwrap();
                assert!(response.get("error").is_none(), "{response}");
                let content = response["result"]["content"][0]["text"].as_str().unwrap();
                if format == "json" {
                    let body: Value = serde_json::from_str(content).unwrap();
                    for dashboard in [
                        &body,
                        &body["accounting"]["session"],
                        &body["accounting"]["combined"],
                    ] {
                        assert_eq!(dashboard["session"]["total_files"], files);
                        assert_eq!(dashboard["session"]["total_raw_tokens"], 100 * files);
                        assert_eq!(dashboard["session"]["total_compressed_tokens"], 50 * files);
                        let actual = dashboard["files"].as_array().unwrap();
                        assert_eq!(actual.len(), files);
                        let actual_paths: std::collections::BTreeSet<_> = actual
                            .iter()
                            .map(|f| f["file_path"].as_str().unwrap())
                            .collect();
                        let expected_paths: std::collections::BTreeSet<_> = before_files
                            .as_object()
                            .unwrap()
                            .keys()
                            .map(String::as_str)
                            .collect();
                        assert_eq!(actual_paths, expected_paths);
                        assert!(actual.iter().all(|f| f["raw_tokens"] == 100
                            && f["compressed_tokens"] == 50
                            && f["fidelity"] == "high"));
                    }
                    assert_eq!(body["persistence"]["enabled"], false);
                } else {
                    assert!(content.contains("Current session:") && content.contains("Combined:"));
                    for i in 0..files {
                        assert!(content.contains(&format!("stats://file-{i}")));
                    }
                }
            }
            let c = work::snapshot();
            assert_eq!(
                serde_json::to_value(state.session_stats_lock().all_file_stats()).unwrap(),
                before_files
            );
            assert_eq!(
                serde_json::to_value(state.session_stats_lock().domain_breakdown()).unwrap(),
                before_domains
            );
            eprintln!(
                "AF002_STATS files={files} invocations={invocations} format={format} initial_text_renders={} text_input_files={} initial_text_bytes={} us={total_us} boundary=public-context-stats",
                c.stats_initial_text_renders,
                c.stats_initial_text_input_files,
                c.stats_initial_text_bytes
            );
        }
        work::reset();
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(2),
            "context_stats",
            &json!({"arguments":{"format":"json","filePath":"stats://file-0"}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none());
        let file: Value =
            serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap())
                .unwrap();
        assert_eq!(file["raw_tokens"], 100);
        assert_eq!(file["compressed_tokens"], 50);
        eprintln!(
            "AF002_STATS_FILE files={files} initial_text_renders={}",
            work::snapshot().stats_initial_text_renders
        );
    }
}
