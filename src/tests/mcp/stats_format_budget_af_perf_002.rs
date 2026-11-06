//! Approved format-only repair: public operation budget and frozen response rows.
use crate::work_observations_af_perf_002 as work;
use serde_json::{Value, json};

fn normalize_json(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            if let Some(duration) = fields.get_mut("duration_secs") {
                assert!(duration.is_u64());
                *duration = json!(0);
            }
            for (key, child) in fields {
                if key == "files" {
                    let files = child.as_array_mut().unwrap();
                    files.sort_by_key(|file| file["file_path"].as_str().unwrap().to_owned());
                }
                normalize_json(child);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(normalize_json),
        _ => {}
    }
}

#[test]
fn af002_stats_json_zero_unused_text_budget() {
    let _serial = crate::protocol::handler_response_serial();
    let mut axes: Vec<_> = [0, 1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|files| (files, 1))
        .collect();
    axes.extend([2, 4, 8, 16, 64, 256].into_iter().map(|calls| (1, calls)));
    axes.push((16, 16));
    let mut violations = Vec::new();
    for (files, calls) in axes {
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        config.persistence.enabled = false;
        let mut state = crate::mcp::McpState::new(config);
        state.proxy_port = 0;
        *state.graph_bridge_lock() = None;
        for i in 0..files {
            state.session_stats_lock().record_compression(
                &format!("stats://budget-{i:03}"),
                100 + i,
                50,
                "high",
                false,
                "full",
                None,
                "compression",
            );
        }
        let original = serde_json::to_value(state.session_stats_lock().all_file_stats()).unwrap();
        for format in ["json", "text"] {
            work::reset();
            let start = std::time::Instant::now();
            for invocation in 0..calls {
                crate::protocol::captured_responses().clear();
                crate::mcp::tools::dispatch_tools_call(
                    &json!(1),
                    "context_stats",
                    &json!({"arguments":{"format":format}}),
                    &state,
                );
                let mut response = crate::protocol::captured_responses().pop().unwrap();
                assert!(response.get("error").is_none(), "{response}");
                let text = response["result"]["content"][0]["text"].as_str().unwrap();
                let normalized = if format == "json" {
                    let mut body: Value = serde_json::from_str(text).unwrap();
                    assert_eq!(body["session"]["total_files"], files);
                    assert_eq!(body["persistence"]["enabled"], false);
                    normalize_json(&mut body);
                    serde_json::to_string_pretty(&body).unwrap()
                } else {
                    assert!(text.contains("Accounting scope: session"));
                    text.split_inclusive('\n')
                        .map(|line| {
                            if line.starts_with("  Session Duration: ") {
                                assert!(line.ends_with('\n'));
                                "  Session Duration: <elapsed>\n"
                            } else {
                                line
                            }
                        })
                        .collect::<String>()
                };
                response["result"]["content"][0]["text"] = json!(normalized);
                if invocation == 0 {
                    let frozen: Vec<Value> =
                        serde_json::from_str(include_str!("fixtures/stats_format_responses.json"))
                            .unwrap();
                    let expected = frozen
                        .iter()
                        .find(|row| {
                            row["files"] == files
                                && row["calls"] == calls
                                && row["format"] == format
                        })
                        .unwrap();
                    assert_eq!(response, expected["response"], "frozen complete response");
                }
                if invocation == 0 {
                    eprintln!(
                        "AF002_STATS_RESPONSE {}",
                        json!({"files":files,"calls":calls,"format":format,"response":response})
                    );
                }
            }
            let elapsed = start.elapsed().as_micros();
            let c = work::snapshot();
            eprintln!(
                "AF002_STATS_BUDGET files={files} calls={calls} format={format} renders={} input_files={} bytes={} us={elapsed}",
                c.stats_initial_text_renders,
                c.stats_initial_text_input_files,
                c.stats_initial_text_bytes
            );
            let expected = if format == "json" { 0 } else { calls };
            if c.stats_initial_text_renders != expected {
                violations.push((files, calls, format, c.stats_initial_text_renders));
            }
            if format == "json"
                && (c.stats_initial_text_input_files != 0 || c.stats_initial_text_bytes != 0)
            {
                violations.push((files, calls, format, c.stats_initial_text_bytes));
            }
            assert_eq!(
                serde_json::to_value(state.session_stats_lock().all_file_stats()).unwrap(),
                original
            );
        }
    }
    assert!(
        violations.is_empty(),
        "unused stats text render budget: {violations:?}"
    );
}
