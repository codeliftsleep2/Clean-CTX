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

#[path = "query_wire/deterministic.rs"]
mod deterministic;
#[path = "query_wire/live.rs"]
mod live;
