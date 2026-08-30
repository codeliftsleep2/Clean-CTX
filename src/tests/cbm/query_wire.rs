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
// Verified wire contract (verbatim raw captures, fresh subprocesses,
// 2026-08-24): responses are `{columns, rows, total}` where `columns` echo
// the RETURN expressions VERBATIM — including inner whitespace
// (`"type( r )"`), cells are JSON strings (numeric projections arrive
// stringly, e.g. in_degree `"10"`), and undirected `-[r]-` patterns are
// supported (returning every relationship type mixed: DEFINES / DECORATES /
// USAGE / CALLS).
//
// ALIAS PIN (captured live): an `AS` alias REPLACES the whole expression in
// the echoed columns — `type(r) AS rel_kind` echoes as `"rel_kind"`, plain
// `a.name AS caller` echoes as `"caller"`. An aliased type() projection is
// therefore INTENTIONALLY indistinguishable from an ordinary scalar at the
// typed layer. Do NOT reverse-engineer aliases into relationship semantics
// CBM no longer provides; such projections fall back to nodes by design.
//
// Fix convention — COLUMN-SHAPE DRIVEN (arity rule retired):
//   exactly one echoed literal `type(...)` column (whitespace-tolerant),
//   >= 3 columns, and every row aligned with the echoed columns
//     => endpoints = FIRST and LAST non-type columns (projection order
//        rules), type cell -> GraphEdge.label, every other projected
//        column -> GraphEdge.properties keyed by echoed column text.
//     Scrambled 5/6/N-column orders work purely from column metadata.
//   anything else => legacy column-0 node mapping, no edges — REGARDLESS
//        of column count. Arbitrary uniform triples (e.g.
//        [name, in_degree, out_degree]) must NEVER fabricate edges; the
//        retired arity rule produced a fake edge labelled `"10"` for
//        exactly that shape.
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

// ── Shape-audit captures (fresh subprocess, 2026-08-24, probe P1–P8) ────

/// Echoed columns for the captures above (unaliased `type(r)` echoes
/// verbatim).
const GQ_TYPE_MIDDLE_COLS: &[&str] = &["a.name", "type(r)", "b.name"];
const GQ_QUALIFIED_COLS: &[&str] = &["a.qualified_name", "type(r)", "b.qualified_name"];
const GQ_NODE_ONLY_COLS: &[&str] = &["f.name", "f.file_path"];

/// P2 — ALIAS ERASES THE MARKER: `type(r) AS rel_kind` echoes as
/// `"rel_kind"`. Intentionally indistinguishable from an ordinary scalar.
const GQ_ALIASED_TYPE_COLS: &[&str] = &["a.name", "rel_kind", "b.name"];
const GQ_ALIASED_TYPE_ROWS: &str = r#"[
 ["bridge_detect_changes_returns_none_when_no_client","CALLS","try_create"],
 ["bridge_detect_changes_returns_none_when_no_client","CALLS","new"],
 ["bridge_disabled_is_unavailable","CALLS","try_create"]
]"#;

/// P4 — 5-column relationship projection, `type(r)` mid-projection, extra
/// qualified-name columns (first two rows of the captured body).
const GQ_FIVE_COL_COLS: &[&str] = &[
    "a.name",
    "type(r)",
    "b.name",
    "a.qualified_name",
    "b.qualified_name",
];
const GQ_FIVE_COL_ROWS: &str = r#"[
 ["bridge_detect_changes_returns_none_when_no_client","CALLS","try_create","C-Users-MNasty-Desktop-RustContextLayerAI.src.tests.cbm.regression.bridge_detect_changes_returns_none_when_no_client","C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.try_create"],
 ["bridge_detect_changes_returns_none_when_no_client","CALLS","new","C-Users-MNasty-Desktop-RustContextLayerAI.src.tests.cbm.regression.bridge_detect_changes_returns_none_when_no_client","C-Users-MNasty-Desktop-RustContextLayerAI.src.mcp.state.McpState.new"]
]"#;

/// P5 — 6-column SCRAMBLED projection: `type(r)` at index 2, extra props on
/// both sides, trailing file_path (both captured rows verbatim).
const GQ_SIX_SCRAMBLED_COLS: &[&str] = &[
    "b.name",
    "b.qualified_name",
    "type(r)",
    "a.name",
    "a.qualified_name",
    "a.file_path",
];
const GQ_SIX_SCRAMBLED_ROWS: &str = r#"[
 ["try_create","C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.try_create","CALLS","bridge_detect_changes_returns_none_when_no_client","C-Users-MNasty-Desktop-RustContextLayerAI.src.tests.cbm.regression.bridge_detect_changes_returns_none_when_no_client","src/tests/cbm/regression.rs"],
 ["new","C-Users-MNasty-Desktop-RustContextLayerAI.src.mcp.state.McpState.new","CALLS","bridge_detect_changes_returns_none_when_no_client","C-Users-MNasty-Desktop-RustContextLayerAI.src.tests.cbm.regression.bridge_detect_changes_returns_none_when_no_client","src/tests/cbm/regression.rs"]
]"#;

/// P6 — `type(r)` FIRST: endpoint identification cannot assume middle
/// position.
const GQ_TYPE_FIRST_COLS: &[&str] = &["type(r)", "a.name", "b.name"];
const GQ_TYPE_FIRST_ROWS: &str = r#"[
 ["CALLS","bridge_detect_changes_returns_none_when_no_client","try_create"],
 ["CALLS","bridge_detect_changes_returns_none_when_no_client","new"]
]"#;

/// P7 — THE KILLER PROOF against the arity rule: a uniform numeric triple
/// `[name, in_degree, out_degree]` would have become a fabricated edge with
/// label `"10"`. Under the shape rule it is a plain node projection.
const GQ_NUMERIC_TRIPLE_COLS: &[&str] = &["f.name", "f.in_degree", "f.out_degree"];
const GQ_NUMERIC_TRIPLE_ROWS: &str = r#"[
 ["cbm_binary_exists","10","0"],
 ["shared_live_state","10","2"],
 ["cbm_project_slug","9","0"]
]"#;

/// P8 — inner whitespace survives the echo verbatim: `"type( r )"`.
const GQ_WHITESPACE_TYPE_COLS: &[&str] = &["a.name", "type( r )", "b.name"];
const GQ_WHITESPACE_TYPE_ROWS: &str = r#"[
 ["CONTRIBUTING.md","DEFINES","CONTRIBUTING.md"],
 ["CONTRIBUTING.md","DEFINES","Contributing to Clean-CTX"]
]"#;

fn cols(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| (*s).to_string()).collect()
}

fn rows_of(value: &str) -> Vec<Vec<serde_json::Value>> {
    serde_json::from_str(value).expect("verbatim wire capture must parse")
}

// ── Deterministic pins ────────────────────────────────────────────────

#[test]
fn directed_calls_capture_becomes_shape_driven_edges() {
    let qr = convert_query_rows(&cols(GQ_TYPE_MIDDLE_COLS), &rows_of(GQ_DIRECTED_CALLS_ROWS));

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
    assert!(
        qr.edges[0].properties.is_empty(),
        "no extra projected columns => no properties"
    );
    assert_eq!(
        qr.edges[4].from,
        "cbm_project_slug_matches_verified_cbm_wire_contract"
    );
    assert_eq!(qr.edges[4].to, "new", "CBM emission order preserved");
}

#[test]
fn undirected_mixed_types_keep_their_labels() {
    let qr = convert_query_rows(
        &cols(GQ_TYPE_MIDDLE_COLS),
        &rows_of(GQ_UNDIRECTED_MIXED_ROWS),
    );

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
    let qr = convert_query_rows(&cols(GQ_QUALIFIED_COLS), &rows_of(GQ_QUALIFIED_ROWS));

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
    let qr = convert_query_rows(&cols(GQ_NODE_ONLY_COLS), &rows_of(GQ_NODE_ONLY_ROWS));

    assert!(
        qr.edges.is_empty(),
        "no type(...) column must produce no edges"
    );
    assert_eq!(qr.nodes.len(), 3);
    assert_eq!(qr.nodes[0].name, "cbm_binary_exists");
    assert_eq!(
        qr.nodes[0].id, "cbm_binary_exists",
        "column 0 maps to id+name"
    );
    // 0.5.1 data-fidelity fix: the projected `f.file_path` cell (present in
    // the verbatim raw capture) populates `GraphNode.file` instead of being
    // silently discarded.
    assert_eq!(
        qr.nodes[0].file, "src/tests/cbm/e2e.rs",
        "projected f.file_path must populate GraphNode.file"
    );
    assert!(
        qr.nodes[0].label.is_empty(),
        "no label projection -> legacy empty label"
    );
    assert!(
        qr.nodes[0].properties.is_empty(),
        "consumed columns do not leak into properties"
    );
}

// ── 0.5.1 data-fidelity pins ───────────────────────────────────────
//
// Node-shaped projections previously read only column 0 and hard-coded
// GraphNode `file`/`label` to empty strings. The verbatim raw captures below
// prove CBM *does* return `f.file_path` (and any extra projected columns);
// the conversion must now surface them instead of discarding them.

/// Node-only projection with a recognized `label` column and an extra scalar.
/// CBM echoes these verbatim exactly like the `f.name, f.file_path` control.
const GQ_NODE_FULL_COLS: &[&str] = &["f.name", "f.label", "f.file_path", "f.in_degree"];
const GQ_NODE_FULL_ROWS: &str = r#"[
 ["cbm_binary_exists","Function","src/tests/cbm/e2e.rs","10"],
 ["shared_live_state","Function","src/tests/cbm/e2e.rs","6"]
]"#;

/// 0.5.1 data-fidelity fix: additional node projections are preserved. A
/// clearly-recognizable `label` column populates `GraphNode.label`, and every
/// non-consumed column lands in `GraphNode.properties` keyed by the echoed
/// column text (mirroring the relationship-shaped property rule).
#[test]
fn node_projection_preserves_extra_properties_and_known_label() {
    let qr = convert_query_rows(&cols(GQ_NODE_FULL_COLS), &rows_of(GQ_NODE_FULL_ROWS));

    assert!(
        qr.edges.is_empty(),
        "no type(...) column must produce no edges"
    );
    assert_eq!(qr.nodes.len(), 2);
    assert_eq!(qr.nodes[0].name, "cbm_binary_exists");
    assert_eq!(qr.nodes[0].id, "cbm_binary_exists");
    assert_eq!(
        qr.nodes[0].label, "Function",
        "label projection populates label"
    );
    assert_eq!(
        qr.nodes[0].file, "src/tests/cbm/e2e.rs",
        "file_path projection populates file"
    );
    assert_eq!(
        qr.nodes[0]
            .properties
            .get("f.in_degree")
            .and_then(serde_json::Value::as_str),
        Some("10"),
        "non-consumed projected columns are preserved verbatim in properties"
    );
    for consumed in ["f.name", "f.label", "f.file_path"] {
        assert!(
            !qr.nodes[0].properties.contains_key(consumed),
            "{consumed} is consumed by GraphNode fields, not a property"
        );
    }
}

/// 0.5.1 cache-namespace bump: `query_graph` keys move from `cypher:` to
/// `cypher2:` so results cached before the node-mapping fix (with always-empty
/// `file` cells) can never survive the upgrade. A pre-0.5.1 `cypher:` entry
/// must be treated as a miss even when a newer `cypher2:` entry is present.
#[test]
fn query_cache_key_namespace_bumps_stale_cypher_entries() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use crate::cbm::bridge::{CachedGraphData, GraphNode, QUERY_CACHE_KEY_NAMESPACE, QueryResult};
    use std::collections::HashMap;

    let mut bridge = new_mock_empty();
    let cypher = "MATCH (f:Function) RETURN f.name, f.file_path LIMIT 5";
    let expires_at = std::time::Instant::now() + std::time::Duration::from_secs(3600);

    let stale: QueryResult = QueryResult {
        nodes: vec![GraphNode {
            id: "stale-id".into(),
            label: String::new(),
            name: "stale-name".into(),
            file: String::new(),
            properties: HashMap::new(),
        }],
        edges: vec![],
    };
    let fresh: QueryResult = QueryResult {
        nodes: vec![GraphNode {
            id: "cbm_binary_exists".into(),
            label: String::new(),
            name: "cbm_binary_exists".into(),
            file: "src/tests/cbm/e2e.rs".into(),
            properties: HashMap::new(),
        }],
        edges: vec![],
    };

    // Pre-0.5.1 entry under the old namespace — must be treated as a miss.
    bridge.cache.insert(
        format!("cypher:{cypher}"),
        CachedGraphData {
            data: serde_json::to_value(&stale).expect("serialize stale"),
            expires_at,
        },
    );
    // 0.5.1 entry under the new namespace — must win.
    bridge.cache.insert(
        format!("{QUERY_CACHE_KEY_NAMESPACE}:{cypher}"),
        CachedGraphData {
            data: serde_json::to_value(&fresh).expect("serialize fresh"),
            expires_at,
        },
    );

    let r = bridge.query_graph(cypher);

    assert_eq!(QUERY_CACHE_KEY_NAMESPACE, "cypher2", "namespace bump pin");
    assert_eq!(r.nodes.len(), 1, "exactly one entry may win");
    assert_eq!(
        r.nodes[0].file, "src/tests/cbm/e2e.rs",
        "fresh 0.5.1-namespace entry is served"
    );
    assert!(
        r.nodes[0].name != "stale-name",
        "pre-0.5.1 cypher: entry must NOT be reused"
    );
}

// ── Shape-audit pins (verbatim captures P2–P8) ─────────────────────────

/// P4: extra projected columns land in `GraphEdge.properties`, keyed by the
/// echoed column text, values preserved exactly as projected.
#[test]
fn five_column_projection_maps_extras_into_properties() {
    let qr = convert_query_rows(&cols(GQ_FIVE_COL_COLS), &rows_of(GQ_FIVE_COL_ROWS));

    assert!(qr.nodes.is_empty());
    assert_eq!(qr.edges.len(), 2, "{:?}", qr.edges);
    assert_eq!(
        qr.edges[0].from,
        "bridge_detect_changes_returns_none_when_no_client"
    );
    // Projection order rules: LAST non-type column wins as `to` — here that
    // is the trailing b.qualified_name column, not b.name.
    assert_eq!(
        qr.edges[0].to,
        "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.try_create"
    );
    assert_eq!(qr.edges[0].label, "CALLS");
    assert_eq!(
        qr.edges[0]
            .properties
            .get("b.name")
            .and_then(serde_json::Value::as_str),
        Some("try_create"),
        "the middle non-type column demotes into properties"
    );
    assert_eq!(
        qr.edges[0]
            .properties
            .get("a.qualified_name")
            .and_then(serde_json::Value::as_str),
        Some(
            "C-Users-MNasty-Desktop-RustContextLayerAI.src.tests.cbm.regression.bridge_detect_changes_returns_none_when_no_client"
        ),
        "remaining projected columns map into properties keyed by echoed column text"
    );
}

/// P5: a scrambled 6-column projection resolves endpoints purely from the
/// column metadata — first non-type column is `from`, LAST non-type column
/// is `to` (projection order rules), everything else becomes properties.
#[test]
fn scrambled_six_column_projection_follows_column_metadata() {
    let qr = convert_query_rows(
        &cols(GQ_SIX_SCRAMBLED_COLS),
        &rows_of(GQ_SIX_SCRAMBLED_ROWS),
    );

    assert!(qr.nodes.is_empty());
    assert_eq!(qr.edges.len(), 2, "{:?}", qr.edges);
    // type(r) sits at index 2; first non-type col (b.name) => from,
    // last non-type col (a.file_path) => to.
    assert_eq!(qr.edges[0].from, "try_create");
    assert_eq!(qr.edges[0].to, "src/tests/cbm/regression.rs");
    assert_eq!(qr.edges[0].label, "CALLS");
    let mut keys: Vec<&String> = qr.edges[0].properties.keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        ["a.name", "a.qualified_name", "b.qualified_name"],
        "exactly the middle non-type columns become properties"
    );
    assert_eq!(
        qr.edges[0].properties["a.name"],
        "bridge_detect_changes_returns_none_when_no_client"
    );
}

/// P6: `type(r)` FIRST — detection is position-independent.
#[test]
fn type_first_projection_still_identifies_endpoints() {
    let qr = convert_query_rows(&cols(GQ_TYPE_FIRST_COLS), &rows_of(GQ_TYPE_FIRST_ROWS));

    assert!(qr.nodes.is_empty());
    assert_eq!(qr.edges.len(), 2);
    assert_eq!(
        qr.edges[0].from,
        "bridge_detect_changes_returns_none_when_no_client"
    );
    assert_eq!(qr.edges[0].to, "try_create");
    assert_eq!(qr.edges[0].label, "CALLS");
}

/// P8: inner whitespace survives CBM's verbatim echo; detection tolerates it.
#[test]
fn whitespace_tolerant_type_detection() {
    let qr = convert_query_rows(
        &cols(GQ_WHITESPACE_TYPE_COLS),
        &rows_of(GQ_WHITESPACE_TYPE_ROWS),
    );

    assert_eq!(qr.edges.len(), 2, "{:?}", qr.edges);
    assert_eq!(qr.edges[1].label, "DEFINES");
    assert_eq!(qr.edges[1].from, "CONTRIBUTING.md");
    assert_eq!(qr.edges[1].to, "Contributing to Clean-CTX");
}

// ── Policy pins (synthetic inputs pin the convention boundaries) ─────

/// Build typed rows (`Vec<Vec<Value>>`) from string slices.
fn vrows(rows: &[&[&str]]) -> Vec<Vec<serde_json::Value>> {
    rows.iter()
        .map(|r| r.iter().map(|s| serde_json::Value::from(*s)).collect())
        .collect()
}

/// P7 — THE regression that retired the arity rule: a uniform triple of
/// `[name, in_degree, out_degree]` must NEVER become an edge. The retired
/// positional rule fabricated `cbm_binary_exists -10-> cbm_binary_exists`
/// here — semantically invented data, not merely incomplete data.
#[test]
fn numeric_triple_without_type_column_is_never_an_edge() {
    let qr = convert_query_rows(
        &cols(GQ_NUMERIC_TRIPLE_COLS),
        &rows_of(GQ_NUMERIC_TRIPLE_ROWS),
    );

    assert!(
        qr.edges.is_empty(),
        "arity must never fabricate edges: {:?}",
        qr.edges
    );
    assert!(
        !qr.edges.iter().any(|e| e.label == "10"),
        "the retired rule labelled a fake edge with the stringly in_degree"
    );
    assert_eq!(qr.nodes.len(), 3, "column-0 node mapping preserved");
    assert_eq!(qr.nodes[0].name, "cbm_binary_exists");
}

/// P2 pin: an ALIASED type() projection is intentionally indistinguishable
/// from ordinary scalars at the typed layer — CBM erases the semantic marker
/// from the echoed columns (`rel_kind` carries no trace of `type(r)`).
/// Falling back to nodes is the contract; reverse-engineering aliases into
/// relationship semantics is explicitly forbidden (CBM-WIRE-002).
#[test]
fn aliased_type_projection_is_intentionally_node_shaped() {
    let qr = convert_query_rows(&cols(GQ_ALIASED_TYPE_COLS), &rows_of(GQ_ALIASED_TYPE_ROWS));

    assert!(
        qr.edges.is_empty(),
        "aliased rel_kind must NOT be interpreted as a relationship type: {:?}",
        qr.edges
    );
    assert_eq!(qr.nodes.len(), 3, "legacy column-0 mapping applies");
}

/// Ambiguity guard: more than one type(...) column means the projection's
/// semantics are unclear — refuse to guess, fall back to nodes.
#[test]
fn multiple_type_columns_refuse_to_guess() {
    let columns = cols(&["type(r)", "a.name", "type(r)"]);
    let rows = vrows(&[&["CALLS", "a", "DEFINES"]]);
    let qr = convert_query_rows(&columns, &rows);

    assert!(qr.edges.is_empty(), "{:?}", qr.edges);
    assert_eq!(qr.nodes.len(), 1);
}

/// Defensive alignment guard: rows that do not line up with the echoed
/// columns mean the projection metadata cannot be trusted.
#[test]
fn row_length_mismatch_against_columns_falls_back() {
    let columns = cols(GQ_TYPE_MIDDLE_COLS);
    let rows = vrows(&[&["a", "CALLS", "b"], &["c", "CALLS"]]);
    let qr = convert_query_rows(&columns, &rows);

    assert!(qr.edges.is_empty(), "{:?}", qr.edges);
    assert_eq!(qr.nodes.len(), 2, "all first-cells still map to nodes");
}

#[test]
fn empty_result_stays_empty_without_error() {
    let qr = convert_query_rows(&[], &[]);
    assert!(qr.nodes.is_empty() && qr.edges.is_empty());
}

#[test]
fn duplicate_rows_pass_through_untouched() {
    // Rider-out pin: dedupe is a SEPARATE finding and deliberately not done.
    let columns = cols(GQ_TYPE_MIDDLE_COLS);
    let rows = vrows(&[&["a", "CALLS", "b"], &["a", "CALLS", "b"]]);
    let qr = convert_query_rows(&columns, &rows);
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
    assert!(qr.nodes.is_empty(), "shape conversion synthesizes no nodes");

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

/// Wide SCRAMBLED relationship projection (5 columns, type() mid-projection):
/// endpoints and properties resolve purely from the echoed column metadata.
/// Projection order rules: first non-type column => `from`, LAST non-type
/// column => `to` — here the projection asks for callee-first.
#[serial(cbm_live)]
#[test]
fn live_wide_scrambled_projection_maps_columns_by_shape() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge) = fresh_fixture_bridge();

    let qr = bridge.query_graph(
        "MATCH (a)-[r:CALLS]->(b) WHERE a.name = 'tw_probe_caller' \
         RETURN b.name, b.qualified_name, type(r), a.name",
    );
    assert!(bridge.take_last_error().is_none(), "query must succeed");
    assert_eq!(qr.edges.len(), 1, "{:?}", qr.edges);
    let edge = &qr.edges[0];
    assert_eq!(edge.from, "tw_probe_callee", "first non-type column");
    assert_eq!(edge.to, "tw_probe_caller", "LAST non-type column");
    assert_eq!(edge.label, "CALLS", "type column becomes the label");
    let qn = edge
        .properties
        .get("b.qualified_name")
        .and_then(serde_json::Value::as_str)
        .expect("middle projected column becomes a property");
    assert!(
        qn.ends_with(".tw_probe_callee"),
        "property value preserved as projected: {qn}"
    );
}

/// The two fabrication guards, live: an ALIASED type() projection (marker
/// erased by CBM) and a numeric triple must both stay node-shaped — never
/// edges.
#[serial(cbm_live)]
#[test]
fn live_non_relationship_projections_never_become_edges() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge) = fresh_fixture_bridge();

    // Aliased type(): CBM echoes only `caller`/`rel_kind`/`callee`.
    let aliased = bridge.query_graph(
        "MATCH (a)-[r:CALLS]->(b) WHERE a.name = 'tw_probe_caller' \
         RETURN a.name AS caller, type(r) AS rel_kind, b.name AS callee",
    );
    assert!(bridge.take_last_error().is_none());
    assert!(
        aliased.edges.is_empty(),
        "aliased type() is intentionally undetectable: {:?}",
        aliased.edges
    );
    assert!(!aliased.nodes.is_empty(), "falls back to nodes");

    // Numeric triple — the exact shape the arity rule fabricated
    // a `"10"`-labelled edge from.
    let nums = bridge.query_graph("MATCH (f:Function) RETURN f.name, f.in_degree, f.out_degree");
    assert!(bridge.take_last_error().is_none());
    assert!(
        nums.edges.is_empty(),
        "numeric triples must never become edges: {:?}",
        nums.edges
    );
    assert!(
        nums.nodes.iter().any(|n| n.name == "tw_probe_callee"),
        "node mapping preserved: {:?}",
        nums.nodes
    );
}
