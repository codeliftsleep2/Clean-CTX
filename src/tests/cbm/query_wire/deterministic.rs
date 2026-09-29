use super::*;

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
    crate::cbm::bridge::test_helpers::seed_active_cache(
        &bridge,
        format!("cypher:{cypher}"),
        CachedGraphData {
            data: serde_json::to_value(&stale).expect("serialize stale"),
            expires_at,
        },
    );
    // 0.5.1 entry under the new namespace — must win.
    crate::cbm::bridge::test_helpers::seed_active_cache(
        &bridge,
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
