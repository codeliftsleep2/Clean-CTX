// src/tests/cbm/query_wire.rs
//
// WIRE CONTRACT: CBM 0.8.1 `query_graph` (the typed `graph_query` path).
//
// Live-testing defect fixed 2026-08-24: `GraphBridge::query_graph` read only
// column 0 of each result row into nodes while `edges` was a literal empty
// vec — so every relationship-returning Cypher (e.g. RETURN a.name, type(r),
// b.name) collapsed to "N node(s), 0 edge(s)" even though CBM returned the
// full rows matrix and `cbm_proxy(query_graph)` surfaced it intact.
//
// Verified wire contract (verbatim raw captures, fresh subprocess,
// 2026-08-24): responses are `{columns, rows, total}` where `columns` echo
// the projection expressions verbatim (`"a.name"`, `"type(r)"`), cells are
// JSON strings, and undirected `-[r]-` patterns are supported (returning
// every relationship type: DEFINES / DECORATES / USAGE / CALLS mixed).
//
// Fix convention — STRICT POSITIONAL (decision: option B):
//   exactly three cells per row, uniform across all rows
//     => `[from, relationship-type, to]` -> GraphEdge{from, to, label}
//   every other shape => legacy column-0 node mapping, no edges.
// Deliberately excluded (separate findings): node deduplication, file-path
// population, endpoint normalization. Duplicates pass through untouched.
//
// Layers, mirroring trace_wire.rs:
//   1. Deterministic pins against VERBATIM raw captures taken 2026-08-24.
//   2. Fresh-process live probes (`serial(cbm_live)`) over a SYNTHETIC
//      temp-dir fixture repo proving typed edges end-to-end.

use serial_test::serial;

use crate::cbm::bridge::{GraphBridge, convert_query_rows};
use crate::cbm::config::CbmConfig;

// ── Verbatim raw row captures (fresh subprocess, 2026-08-24) ─────────

/// Directed CALLS projection, bare names — inner `rows` array of the captured
/// `{"columns":["a.name","type(r)","b.name"],"rows":[…],"total":5}` body.
const GQ_DIRECTED_CALLS_ROWS: &str = r#"[
 ["bridge_detect_changes_returns_none_when_no_client","CALLS","try_create"],
 ["bridge_detect_changes_returns_none_when_no_client","CALLS","new"],
 ["bridge_disabled_is_unavailable","CALLS","try_create"],
 ["bridge_disabled_is_unavailable","CALLS","new"],
 ["cbm_project_slug_matches_verified_cbm_wire_contract","CALLS","new"]
]"#;

/// UNDIRECTED `-[]-` projection: works on CBM 0.8.1 and returns EVERY
/// relationship type mixed (DEFINES / DECORATES / USAGE here).
const GQ_UNDIRECTED_MIXED_ROWS: &str = r#"[
 ["AST-level diff (track changes over time)","DEFINES","README.md"],
 ["Adding a Language","DEFINES","CONTRIBUTING.md"],
 ["Adding a Tool","DEFINES","CONTRIBUTING.md"],
 ["AffectedSymbol","DECORATES","derive"],
 ["AffectedSymbol","USAGE","derive"]
]"#;

/// Qualified-name endpoints flow through untouched — graph_query has no
/// target matching, so no normalization applies (unlike graph_trace M-01).
const GQ_QUALIFIED_ROWS: &str = r#"[
 ["C-Users-MNasty-Desktop-RustContextLayerAI.src.tests.cbm.regression.bridge_detect_changes_returns_none_when_no_client","CALLS","C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.try_create"],
 ["C-Users-MNasty-Desktop-RustContextLayerAI.src.tests.cbm.regression.bridge_detect_changes_returns_none_when_no_client","CALLS","C-Users-MNasty-Desktop-RustContextLayerAI.src.mcp.state.McpState.new"],
 ["C-Users-MNasty-Desktop-RustContextLayerAI.src.tests.cbm.regression.bridge_disabled_is_unavailable","CALLS","C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.try_create"]
]"#;

/// Node-only control (two columns): must keep the legacy mapping.
const GQ_NODE_ONLY_ROWS: &str = r#"[
 ["cbm_binary_exists","src/tests/cbm/e2e.rs"],
 ["shared_live_state","src/tests/cbm/e2e.rs"],
 ["wait_for_indexing_complete","src/tests/cbm/e2e.rs"]
]"#;

fn rows_of(value: &str) -> Vec<Vec<serde_json::Value>> {
    serde_json::from_str(value).expect("verbatim wire capture must parse")
}

// ── Deterministic pins ────────────────────────────────────────────────

#[test]
fn directed_calls_capture_becomes_positional_edges() {
    let qr = convert_query_rows(&rows_of(GQ_DIRECTED_CALLS_ROWS));

    assert!(
        qr.nodes.is_empty(),
        "relationship-shaped projection IS its edges"
    );
    assert_eq!(qr.edges.len(), 5, "{:?}", qr.edges);
    assert_eq!(
        qr.edges[0].from,
        "bridge_detect_changes_returns_none_when_no_client"
    );
    assert_eq!(qr.edges[0].to, "try_create");
    assert_eq!(qr.edges[0].label, "CALLS");
    assert_eq!(
        qr.edges[4].from,
        "cbm_project_slug_matches_verified_cbm_wire_contract"
    );
    assert_eq!(qr.edges[4].to, "new", "CBM emission order preserved");
}

#[test]
fn undirected_mixed_types_keep_their_labels() {
    let qr = convert_query_rows(&rows_of(GQ_UNDIRECTED_MIXED_ROWS));

    assert_eq!(qr.edges.len(), 5);
    let labels: Vec<&str> = qr.edges.iter().map(|e| e.label.as_str()).collect();
    assert_eq!(
        labels,
        ["DEFINES", "DEFINES", "DEFINES", "DECORATES", "USAGE"],
        "every relationship type passes through verbatim"
    );
}

#[test]
fn qualified_endpoints_flow_through_untouched() {
    let qr = convert_query_rows(&rows_of(GQ_QUALIFIED_ROWS));

    assert_eq!(qr.edges.len(), 3);
    assert!(
        qr.edges[0]
            .from
            .ends_with(".bridge_detect_changes_returns_none_when_no_client"),
        "qualified identity preserved as-is: {}",
        qr.edges[0].from
    );
    assert_eq!(
        qr.edges[0].to,
        "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.try_create"
    );
}

#[test]
fn node_only_shape_keeps_legacy_mapping() {
    let qr = convert_query_rows(&rows_of(GQ_NODE_ONLY_ROWS));

    assert!(
        qr.edges.is_empty(),
        "non-triple shape must produce no edges"
    );
    assert_eq!(qr.nodes.len(), 3);
    assert_eq!(qr.nodes[0].name, "cbm_binary_exists");
    assert_eq!(
        qr.nodes[0].id, "cbm_binary_exists",
        "column 0 maps to id+name"
    );
}

// ── Policy pins (synthetic inputs pin the convention boundaries) ─────

/// Build typed rows (`Vec<Vec<Value>>`) from string triples/slices.
fn vrows(rows: &[&[&str]]) -> Vec<Vec<serde_json::Value>> {
    rows.iter()
        .map(|r| r.iter().map(|s| serde_json::Value::from(*s)).collect())
        .collect()
}

#[test]
fn non_triple_arity_falls_back_to_nodes() {
    let one_col = vrows(&[&["a"], &["b"]]);
    let four_col = vrows(&[&["a", "p", "1", "x"], &["b", "q", "2", "y"]]);
    for rows in [&one_col, &four_col] {
        let qr = convert_query_rows(rows);
        assert!(!qr.nodes.is_empty(), "legacy node mapping preserved");
        assert!(qr.edges.is_empty());
    }
}

#[test]
fn mixed_arity_never_becomes_edges() {
    let rows = vrows(&[&["a", "CALLS", "b"], &["c", "d"], &["e", "CALLS", "f"]]);
    let qr = convert_query_rows(&rows);
    assert!(
        qr.edges.is_empty(),
        "uniform arity is required — one deviant row downgrades the whole set"
    );
    assert_eq!(qr.nodes.len(), 3, "all first-cells still map to nodes");
}

#[test]
fn empty_result_stays_empty_without_error() {
    let qr = convert_query_rows(&[]);
    assert!(qr.nodes.is_empty() && qr.edges.is_empty());
}

#[test]
fn duplicate_rows_pass_through_untouched() {
    // Rider-out pin: dedupe is a SEPARATE finding and deliberately not done.
    let rows = vrows(&[&["a", "CALLS", "b"], &["a", "CALLS", "b"]]);
    let qr = convert_query_rows(&rows);
    assert_eq!(qr.edges.len(), 2, "no silent collapsing in this fix");
}

// ── Fresh-process live probes over the SYNTHETIC fixture repo ────────

fn cbm_binary_exists() -> bool {
    let name = if cfg!(windows) {
        "codebase-memory-mcp.exe"
    } else {
        "codebase-memory-mcp"
    };
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}

fn live_config() -> CbmConfig {
    CbmConfig {
        enabled: true,
        ..Default::default()
    }
}

fn wait_ready(bridge: &mut GraphBridge) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        match bridge.ensure_indexed() {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => return,
            Ok(crate::cbm::bridge::IndexingStatus::StillIndexing { .. }) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "timed out waiting for CBM indexing of the query fixture"
                );
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            Err(e) => {
                eprintln!("Indexing failed (continuing): {e}");
                return;
            }
        }
    }
}

/// caller → callee is the ONLY relationship; nothing derives from this repo.
const FIXTURE_RS: &str = r#"
pub fn tw_probe_callee() -> i32 { 7 }
pub fn tw_probe_caller() -> i32 { tw_probe_callee() + 1 }
"#;

fn fresh_fixture_bridge() -> (tempfile::TempDir, GraphBridge) {
    let root = tempfile::Builder::new()
        .prefix("cleanctx_query_wire_")
        .tempdir()
        .expect("fixture tempdir");
    std::fs::write(root.path().join("query_fixture.rs"), FIXTURE_RS)
        .expect("write query fixture source");
    let mut bridge = GraphBridge::try_create(&live_config(), root.path());
    wait_ready(&mut bridge);
    (root, bridge)
}

/// THE regression: typed `graph_query` now surfaces relationship rows AS
/// EDGES. Pre-fix this exact projection collapsed to duplicated column-0
/// nodes with `"edges":[]`.
#[serial(cbm_live)]
#[test]
fn live_typed_graph_query_returns_edges_for_relationship_projection() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge) = fresh_fixture_bridge();

    let qr = bridge.query_graph(
        "MATCH (a)-[r:CALLS]->(b) WHERE a.name = 'tw_probe_caller' \
         RETURN a.name, type(r), b.name",
    );
    assert!(bridge.take_last_error().is_none(), "query must succeed");
    assert_eq!(
        qr.edges.len(),
        1,
        "relationship rows become edges: {:?}",
        qr.edges
    );
    assert_eq!(qr.edges[0].from, "tw_probe_caller");
    // Endpoint identity is EXACTLY what the projection asks for: `.name`
    // yields the bare symbol (matches the directed-CALLS raw capture where
    // b.name arrived bare, e.g. "try_create").
    assert_eq!(qr.edges[0].to, "tw_probe_callee");
    assert_eq!(qr.edges[0].label, "CALLS");
    assert!(qr.nodes.is_empty(), "strict positional: no invented nodes");

    // Same relationship, QUALIFIED projection: endpoints become the fully
    // qualified wire identities, verbatim.
    let qqr = bridge.query_graph(
        "MATCH (a)-[r:CALLS]->(b) WHERE a.name = 'tw_probe_caller' \
         RETURN a.qualified_name, type(r), b.qualified_name",
    );
    assert!(bridge.take_last_error().is_none());
    assert_eq!(qqr.edges.len(), 1, "{:?}", qqr.edges);
    assert!(
        qqr.edges[0].from.ends_with(".tw_probe_caller"),
        "{}",
        qqr.edges[0].from
    );
    assert!(
        qqr.edges[0].to.ends_with(".tw_probe_callee"),
        "{}",
        qqr.edges[0].to
    );
    assert_eq!(qqr.edges[0].label, "CALLS");
}

/// Preserved behavior: node-only projections keep producing nodes.
#[serial(cbm_live)]
#[test]
fn live_node_only_projection_still_returns_nodes() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge) = fresh_fixture_bridge();

    let qr = bridge.query_graph("MATCH (f:Function) RETURN f.name LIMIT 5");
    assert!(bridge.take_last_error().is_none());
    assert!(!qr.nodes.is_empty(), "node-only projections unchanged");
    assert!(qr.edges.is_empty());
    assert!(
        qr.nodes.iter().any(|n| n.name == "tw_probe_caller"),
        "fixture functions visible: {:?}",
        qr.nodes
    );
}
