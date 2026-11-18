use super::{CbmClient, CbmError, a2_support as fixture};
use serde_json::{Value, json};

fn with_client(mode: &str, body: impl FnOnce(&mut CbmClient, &std::path::Path)) {
    let root = tempfile::tempdir().unwrap();
    let log = root.path().join("wire.json");
    let mut client = fixture::launch_for_admission(root.path(), &log, mode);
    client.admit().unwrap();
    body(&mut client, &log);
}
fn args(log: &std::path::Path) -> Value {
    std::fs::read_to_string(log.with_extension("requests"))
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .rfind(|v| v["method"] == "tools/call")
        .unwrap()["params"]["arguments"]
        .clone()
}

#[test]
fn cbm_current_wire_search_requests_json_and_preserves_filters() {
    with_client("b1-search", |c, log| {
        let _ = c.search_graph("run", "fixture", Some("Function"));
        assert_eq!(
            args(log),
            json!({"project":"fixture","name_pattern":"run","label":"Function","format":"json"})
        );
    });
}
#[test]
fn cbm_current_wire_grouped_search_preserves_identity_file_and_zero() {
    with_client("b1-search", |c, _| {
        let rows = c.search_graph("run", "fixture", None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["qualified_name"], "fixture.src.api.run");
        assert_eq!(rows[0]["file_path"], "src/api.ts");
        assert_eq!(rows[0]["out_degree"], 0);
    });
}
#[test]
fn cbm_current_wire_empty_grouped_search_is_valid() {
    with_client("b1-empty-search", |c, _| {
        assert!(c.search_graph("x", "fixture", None).unwrap().is_empty())
    });
}
#[test]
fn cbm_current_wire_missing_search_groups_is_not_empty_success() {
    with_client("b1-missing-group", |c, _| {
        assert!(matches!(
            c.search_graph("x", "fixture", None),
            Err(CbmError::ParseError(_))
        ))
    });
}
#[test]
fn cbm_current_wire_malformed_search_row_is_not_empty_success() {
    with_client("b1-bad-search-row", |c, _| {
        assert!(matches!(
            c.search_graph("x", "fixture", None),
            Err(CbmError::ParseError(_))
        ))
    });
}
#[test]
fn cbm_current_wire_query_requests_json_and_preserves_cells() {
    with_client("b1-query", |c, log| {
        let table = c.query_graph("RETURN 0 AS alias", "fixture").unwrap();
        assert_eq!(table.columns, ["alias", "flag", "missing"]);
        assert_eq!(
            table.rows,
            vec![vec![json!("0"), json!(false), Value::Null]]
        );
        assert_eq!(args(log)["format"], "json");
    });
}
#[test]
fn cbm_current_wire_query_nonstring_columns_are_rejected() {
    with_client("b1-bad-query-column", |c, _| {
        assert!(matches!(
            c.query_graph("query", "fixture"),
            Err(CbmError::ParseError(_))
        ))
    });
}
#[test]
fn cbm_current_wire_query_nonarray_rows_are_rejected() {
    with_client("b1-bad-query-row", |c, _| {
        assert!(matches!(
            c.query_graph("query", "fixture"),
            Err(CbmError::ParseError(_))
        ))
    });
}
#[test]
fn cbm_current_wire_query_missing_tables_are_rejected() {
    with_client("b1-missing-query-table", |c, _| {
        assert!(matches!(
            c.query_graph("query", "fixture"),
            Err(CbmError::ParseError(_))
        ))
    });
}
#[test]
fn cbm_current_wire_trace_normalizes_only_proven_direct_hops() {
    with_client("b1-trace", |c, log| {
        let edges = c
            .trace_path("fixture.run", "both", "fixture", Some(3))
            .unwrap();
        assert_eq!(
            edges,
            vec![
                json!({"from":"fixture.caller","to":"fixture.run","label":"calls"}),
                json!({"from":"fixture.run","to":"fixture.leaf","label":"calls"})
            ]
        );
        assert_eq!(args(log)["format"], "json");
    });
}
#[test]
fn cbm_current_wire_trace_missing_leg_is_rejected() {
    with_client("b1-missing-trace-leg", |c, _| {
        assert!(matches!(
            c.trace_path("fixture.run", "both", "fixture", None),
            Err(CbmError::ParseError(_))
        ))
    });
}
#[test]
fn cbm_current_wire_architecture_normalizes_consumed_sections() {
    with_client("b1-architecture", |c, log| {
        let arch = c.get_architecture("fixture").unwrap();
        assert_eq!(arch["packages"][0]["node_count"], 4);
        assert_eq!(arch["boundaries"][0]["to"], "data");
        assert_eq!(args(log)["format"], "json");
        assert_eq!(args(log)["aspects"], json!(["packages", "boundaries"]));
    });
}
#[test]
fn cbm_current_wire_plain_text_soft_error_stays_tool_failure() {
    with_client("b1-soft-text", |c, _| {
        assert!(matches!(c.search_graph("x", "fixture", None),
        Err(CbmError::ToolError {message,..}) if message == "project not indexed"))
    });
}
#[test]
fn cbm_current_wire_raw_proxy_preserves_caller_format() {
    with_client("b1-search", |c, log| {
        c.call_tool_raw("search_graph", json!({"project":"fixture","format":"tree"}))
            .unwrap();
        assert_eq!(args(log)["format"], "tree");
    });
}
