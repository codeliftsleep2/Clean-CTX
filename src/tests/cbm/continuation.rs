use super::{CbmClient, a2_support as fixture};
use serde_json::{Value, json};
fn with_client(mode: &str, body: impl FnOnce(&mut CbmClient, &std::path::Path)) {
    let root = tempfile::tempdir().unwrap();
    let log = root.path().join("wire.json");
    let mut client = fixture::launch_for_admission(root.path(), &log, mode);
    client.admit().unwrap();
    body(&mut client, &log);
}
fn requests(log: &std::path::Path) -> Vec<Value> {
    std::fs::read_to_string(log.with_extension("requests"))
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .filter(|v| v["method"] == "tools/call")
        .collect()
}
#[test]
fn cbm_continuation_query_collects_duplicate_rows_and_fixed_projection() {
    with_client("b3-query", |c, log| {
        let table = c.query_graph("query", "fixture").unwrap();
        assert_eq!(
            table.rows,
            vec![vec![json!("same"), json!(false), Value::Null]; 2]
        );
        let r = requests(log);
        assert_eq!(r.len(), 2);
        assert_eq!(
            r[1]["params"]["arguments"],
            json!({"project":"fixture","query":"query","format":"json","cursor":"opaque-page-two"})
        );
    });
}
#[test]
fn cbm_continuation_search_uses_offset_and_preserves_filter() {
    with_client("b3-search", |c, log| {
        let rows = c
            .search_graph("pattern", "fixture", Some("Function"))
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1]["qualified_name"], "fixture.second");
        let r = requests(log);
        assert_eq!(r.len(), 2);
        assert_eq!(r[1]["params"]["arguments"]["offset"], 1);
        assert_eq!(r[1]["params"]["arguments"]["label"], "Function");
    });
}
#[test]
fn cbm_continuation_trace_collects_without_inventing_deeper_edges() {
    with_client("b3-trace", |c, _| {
        assert_eq!(
            c.trace_path("fixture.run", "both", "fixture", Some(3))
                .unwrap(),
            vec![json!({"from":"fixture.caller","to":"fixture.run","label":"calls"})]
        );
    });
}
#[test]
fn cbm_continuation_coverage_collects_exact_requested_paths() {
    with_client("b3-coverage", |c, log| {
        let root = log.parent().unwrap();
        let paths = (0..21).map(|i| format!("file{i}.ts")).collect::<Vec<_>>();
        c.observe_candidate_paths("fixture", root, &paths).unwrap();
        let r = requests(log);
        assert_eq!(r.len(), 3);
        assert_eq!(r[2]["params"]["arguments"]["paths"], json!(paths));
        assert_eq!(r[2]["params"]["arguments"]["path_offset"], 20);
    });
}
#[test]
fn cbm_continuation_repeated_cursor_stops_without_third_request() {
    with_client("b3-repeat", |c, log| {
        assert!(c.query_graph("query", "fixture").is_err());
        assert_eq!(requests(log).len(), 2);
    });
}
#[test]
fn cbm_continuation_changed_columns_fail_without_partial_success() {
    with_client("b3-columns", |c, log| {
        assert!(c.query_graph("query", "fixture").is_err());
        assert_eq!(requests(log).len(), 2);
    });
}
#[test]
fn cbm_continuation_stale_cursor_is_not_restarted_as_offset() {
    with_client("b3-stale", |c, log| {
        let e = c.query_graph("query", "fixture").unwrap_err();
        assert!(e.to_string().contains("stale cursor"));
        assert_eq!(requests(log).len(), 2);
    });
}
#[test]
fn cbm_continuation_aggregate_payload_budget_stops_collection() {
    with_client("b3-budget", |c, _| {
        c.timeout = std::time::Duration::from_secs(1);
        let e = c.query_graph("query", "fixture").unwrap_err();
        assert!(e.to_string().contains("aggregate"), "{e}");
    });
}
#[test]
fn cbm_continuation_shared_deadline_does_not_reset_per_page() {
    with_client("b3-deadline", |c, log| {
        c.timeout = std::time::Duration::from_millis(250);
        let start = std::time::Instant::now();
        let error = c.query_graph("query", "fixture").unwrap_err();
        assert!(matches!(error, super::CbmError::Timeout(_)), "{error}");
        assert!(start.elapsed() < std::time::Duration::from_millis(600));
        assert_eq!(requests(log).len(), 2);
    });
}
#[test]
fn cbm_continuation_raw_proxy_does_not_collect_pages() {
    with_client("b3-query", |c, log| {
        c.call_tool_raw(
            "query_graph",
            json!({"project":"fixture","query":"query","format":"json"}),
        )
        .unwrap();
        assert_eq!(requests(log).len(), 1);
    });
}

#[test]
fn cbm_continuation_retry_budget_is_shared_across_pages() {
    with_client("b3-retry", |c, log| {
        assert!(c.query_graph("query", "fixture").is_err());
        assert_eq!(
            requests(log).len(),
            3,
            "one retry on page one leaves no retry for page two"
        );
    });
}
#[test]
fn cbm_continuation_empty_page_with_cursor_stops_without_retry() {
    with_client("b3-empty", |c, log| {
        assert!(c.query_graph("query", "fixture").is_err());
        assert_eq!(requests(log).len(), 1);
    });
}
#[test]
fn cbm_continuation_changed_total_is_not_offset_fallback() {
    with_client("b3-total", |c, log| {
        assert!(c.query_graph("query", "fixture").is_err());
        assert_eq!(requests(log).len(), 2);
    });
}
#[test]
fn cbm_continuation_deadline_retains_partial_evidence_after_retirement() {
    with_client("b3-deadline", |c, _| {
        c.timeout = std::time::Duration::from_millis(250);
        assert!(c.query_graph("query", "fixture").is_err());
        let observation = c.provider_observation("fixture", "query_graph").unwrap();
        assert_eq!(observation.metadata["pages_consumed"], 1);
        assert_eq!(
            observation.metadata["page_observations"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(observation.failure.is_some());
    });
}
