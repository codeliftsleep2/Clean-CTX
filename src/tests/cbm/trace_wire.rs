// src/tests/cbm/trace_wire.rs
//
// WIRE CONTRACT: CBM 0.8.1 `trace_path` (the typed `graph_trace` path).
//
// Live-testing defects fixed 2026-08-24:
//   1. CbmClient::trace_path parsed a phantom `inner["edges"]` key — CBM
//      actually represents relationships through `callers` / `callees`
//      arrays whose entries carry `name` / `qualified_name` / `hop` — so
//      every typed graph_trace silently collapsed to zero edges while the
//      raw cbm_proxy path worked fine.
//   2. GraphBridge::trace_path hardcoded OUTBOUND whenever both endpoints
//      were supplied, making inbound-only relationships undiscoverable.
//
// Layers, mirroring the established CBM audit suites:
//   1. Deterministic pins — VERBATIM raw captures taken 2026-08-24 from a
//      fresh CBM 0.8.1 subprocess (probe transcript preserved in the
//      constants below), exercised through the pure parser and the F11
//      soft-error gate.
//   2. Fresh-process live probes (`#[serial(cbm_live)]`) against a
//      SYNTHETIC temp-dir fixture repo — caller → callee is the only
//      relationship and NOTHING depends on Clean-CTX's own graph —
//      covering: typed graph_query CALLS rows, the outbound trace,
//      the inbound trace, and the two-endpoint inbound-only discovery
//      (the regression that motivated this suite).

use serial_test::serial;

use crate::cbm::bridge::{GraphBridge, filter_trace_edges};
use crate::cbm::client::{check_soft_error, extract_trace_edges};
use crate::cbm::config::CbmConfig;

// ── Verbatim raw wire captures (fresh subprocess, 2026-08-24) ────────

/// `direction=inbound` — relationships arrive as a `callers` array keyed by
/// `name` / `qualified_name` / `hop`. Note the `__file__` module pseudo-node:
/// CBM reports module-level code as a genuine caller.
const TRACE_INBOUND_WIRE_CAPTURE: &str = r#"{"function":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.query_graph","direction":"inbound","callers":[{"name":"src/cbm/bridge.rs","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.__file__","hop":1},{"name":"handle_graph_query","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.handle_graph_query","hop":1}]}"#;

/// `direction=outbound` — symmetric `callees` array. Bare-name lookup works
/// when unambiguous; entries carry exactly the three keys pinned here.
const TRACE_OUTBOUND_WIRE_CAPTURE: &str = r#"{"function":"C-Users-MNasty-Desktop-RustContextLayerAI.src.tests.cbm.regression.bridge_disabled_is_unavailable","direction":"outbound","callees":[{"name":"new","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.mcp.state.McpState.new","hop":1},{"name":"try_create","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.try_create","hop":1}]}"#;

/// `direction=both` — BOTH arrays coexist in one body; an empty half is a
/// real empty ARRAY (never a missing key). Hop-2 callers are flat BFS
/// discoveries with no parent linkage.
const TRACE_BOTH_WIRE_CAPTURE: &str = r#"{"function":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.ensure_indexed","direction":"both","callees":[],"callers":[{"name":"src/cbm/bridge.rs","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.__file__","hop":1},{"name":"ensure_indexed_or_error","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.ensure_indexed_or_error","hop":1},{"name":"handle_graph_search","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.handle_graph_search","hop":2},{"name":"handle_graph_query","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.handle_graph_query","hop":2},{"name":"handle_graph_trace","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.handle_graph_trace","hop":2},{"name":"handle_get_architecture","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.handle_get_architecture","hop":2}]}"#;

/// `depth=3` outbound — pins the depth semantics that govern conversion:
/// entries are FLAT hop-tagged BFS discoveries with NO parent linkage.
/// `status` appears at BOTH hop 1 (direct) and hop 2 (transitive); hop ≥ 2
/// entries cannot be attributed to an intermediate caller and must never be
/// turned into invented edges.
const TRACE_DEEP_OUTBOUND_WIRE_CAPTURE: &str = r#"{"function":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.handle_graph_query","direction":"outbound","callees":[{"name":"status","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.status","hop":1},{"name":"take_last_error","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.take_last_error","hop":1},{"name":"query_graph","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.query_graph","hop":1},{"name":"with_bridge","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.with_bridge","hop":1},{"name":"set_project_from_params","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.set_project_from_params","hop":1},{"name":"ensure_indexed_or_error","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.ensure_indexed_or_error","hop":1},{"name":"graph_bridge_lock","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.mcp.state.McpState.graph_bridge_lock","hop":2},{"name":"status","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.status","hop":2},{"name":"check_cbm_healthy","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.check_cbm_healthy","hop":2},{"name":"new","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.mcp.state.McpState.new","hop":2},{"name":"set_project","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.set_project","hop":2},{"name":"set_workspace_root","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.set_workspace_root","hop":2},{"name":"ensure_indexed","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.ensure_indexed","hop":2},{"name":"send_indexing_gate","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.send_indexing_gate","hop":2},{"name":"is_available","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.is_available","hop":3}]}"#;

/// Function-not-found: CBM signals tool failure INSIDE a successful JSON-RPC
/// result (`result.isError=true` + an `"error"` key in the content text).
/// Full JSON-RPC result envelope, verbatim — the F11 soft-error gate must
/// classify this as [`crate::cbm::client::CbmError::ToolError`] BEFORE any
/// edge parsing runs, so failure is never a valid empty result.
const TRACE_NOT_FOUND_RESULT_ENVELOPE: &str = r#"{"content":[{"type":"text","text":"{\"error\":\"function not found\",\"function_name\":\"compile_inner\",\"hint\":\"Use search_graph(name_pattern=\\\".*compile_inner.*\\\") to find the exact name, then pass it to trace_path.\"}"}],"isError":true}"#;

// ── Deterministic pins: pure parser against verbatim captures ────────

fn capture(value: &str) -> serde_json::Value {
    serde_json::from_str(value).expect("verbatim wire capture must parse")
}

#[test]
fn inbound_capture_yields_caller_to_function_edges() {
    let inner = capture(TRACE_INBOUND_WIRE_CAPTURE);
    let edges = extract_trace_edges(
        &inner,
        "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.query_graph",
    );

    assert_eq!(edges.len(), 2, "both hop-1 callers convert: {edges:?}");
    // CBM emission order preserved: __file__ first, then handle_graph_query.
    assert_eq!(
        edges[0]["from"],
        "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.__file__"
    );
    assert_eq!(
        edges[1]["from"],
        "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.handle_graph_query"
    );
    for e in &edges {
        assert_eq!(
            e["to"],
            "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.query_graph"
        );
        assert_eq!(
            e["label"], "calls",
            "caller → callee orientation is uniform"
        );
    }
}

#[test]
fn outbound_capture_yields_function_to_callee_edges() {
    let inner = capture(TRACE_OUTBOUND_WIRE_CAPTURE);
    let fn_q = "C-Users-MNasty-Desktop-RustContextLayerAI.src.tests.cbm.regression.bridge_disabled_is_unavailable";
    let edges = extract_trace_edges(&inner, fn_q);

    assert_eq!(edges.len(), 2, "both hop-1 callees convert: {edges:?}");
    assert_eq!(edges[0]["from"], fn_q);
    assert_eq!(
        edges[0]["to"],
        "C-Users-MNasty-Desktop-RustContextLayerAI.src.mcp.state.McpState.new"
    );
    assert_eq!(
        edges[1]["to"],
        "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.try_create"
    );
}

#[test]
fn both_capture_merges_arrays_with_correct_directions() {
    let inner = capture(TRACE_BOTH_WIRE_CAPTURE);
    let fn_q =
        "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge.ensure_indexed";
    let edges = extract_trace_edges(&inner, fn_q);

    // callees=[] contributes nothing; the two hop-1 callers do.
    assert_eq!(edges.len(), 2, "{edges:?}");
    assert_eq!(
        edges[0]["from"],
        "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.__file__"
    );
    assert_eq!(
        edges[1]["from"],
        "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.ensure_indexed_or_error"
    );
}

#[test]
fn deep_capture_keeps_only_hop1_entries_never_invents_edges() {
    let inner = capture(TRACE_DEEP_OUTBOUND_WIRE_CAPTURE);
    let fn_q = "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.handle_graph_query";
    let edges = extract_trace_edges(&inner, fn_q);

    // Exactly the six hop-1 direct callees; the nine hop ≥ 2 entries are
    // flat discoveries with unidentifiable parents and are NOT converted.
    assert_eq!(
        edges.len(),
        6,
        "hop ≥ 2 entries must be dropped, not mis-attributed: {edges:?}"
    );
    let tos: Vec<&str> = edges.iter().filter_map(|e| e["to"].as_str()).collect();
    for expected in [
        ".GraphBridge.status",
        ".GraphBridge.take_last_error",
        ".GraphBridge.query_graph",
        ".handlers.with_bridge",
        ".handlers.set_project_from_params",
        ".handlers.ensure_indexed_or_error",
    ] {
        assert!(
            tos.iter().any(|t| t.ends_with(expected)),
            "missing hop-1 callee {expected} in {tos:?}"
        );
    }
    // Transitive-only symbols stay out entirely.
    for transitive in [
        ".graph_bridge_lock",
        ".check_cbm_healthy",
        ".is_available",
        ".send_indexing_gate",
    ] {
        assert!(
            !tos.iter().any(|t| t.ends_with(transitive)),
            "transitive hop must not become an edge: {tos:?}"
        );
    }
}

#[test]
fn function_not_found_envelope_maps_to_tool_error_before_parsing() {
    let result = capture(TRACE_NOT_FOUND_RESULT_ENVELOPE);
    let err = check_soft_error("trace_path", &result)
        .expect_err("isError envelope must map to CbmError::ToolError (F11)");
    match &err {
        crate::cbm::client::CbmError::ToolError { tool, message } => {
            assert_eq!(tool, "trace_path");
            assert_eq!(message, "function not found");
        }
        other => panic!("expected ToolError, got {other:?}"),
    }
    assert_eq!(err.to_string(), "CBM trace_path failed: function not found");
}

// ── Synthetic policy pins (hand-built inputs; fixtures above pin reality,
//    these pin the conversion POLICY at the boundaries) ─────────────────

#[test]
fn missing_qualified_name_falls_back_to_bare_name() {
    let inner = serde_json::json!({
        "callers": [{"name": "plain_caller", "hop": 1}]
    });
    let edges = extract_trace_edges(&inner, "target_fn");
    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0]["from"], "plain_caller",
        "qualified_name → name fallback"
    );
    assert_eq!(edges[0]["to"], "target_fn");
}

#[test]
fn exact_duplicate_edges_dedupe_preserving_first_order() {
    let inner = serde_json::json!({
        "callees": [
            {"name": "dup", "qualified_name": "p.dup", "hop": 1},
            {"name": "mid", "qualified_name": "p.mid", "hop": 1},
            {"name": "dup", "qualified_name": "p.dup", "hop": 1}
        ]
    });
    let edges = extract_trace_edges(&inner, "f");
    assert_eq!(edges.len(), 2, "exact duplicate edge collapses: {edges:?}");
    assert_eq!(edges[0]["to"], "p.dup");
    assert_eq!(edges[1]["to"], "p.mid");
}

#[test]
fn absent_arrays_and_non_direct_hops_yield_no_invented_edges() {
    assert!(extract_trace_edges(&serde_json::json!({}), "f").is_empty());
    let hops = serde_json::json!({
        "callers": [{"name": "deep", "qualified_name": "p.deep", "hop": 2}],
        "callees": [{"name": "deeper", "qualified_name": "p.deeper", "hop": 3}]
    });
    assert!(
        extract_trace_edges(&hops, "f").is_empty(),
        "non-direct hops are never converted into relationships"
    );
}

// ── Deterministic pins: M-01 target filter under qualified identity ──

fn ge(from: &str, to: &str) -> serde_json::Value {
    serde_json::json!({"from": from, "to": to, "label": "calls"})
}

#[test]
fn target_filter_matches_bare_and_qualified_endpoints() {
    let edges = vec![
        ge("caller", "p.mod.GraphBridge.query_graph"), // qualified `to`
        ge("p.mod.GraphBridge.query_graph", "x"),      // qualified `from`
        ge("bare_exact", "y"),                         // exact bare match
        ge("a", "p.unrelated.symbol"),                 // touches nothing
    ];

    let kept = filter_trace_edges(&edges, &Some("query_graph".to_string()));
    assert_eq!(
        kept.len(),
        2,
        "qualified-suffix matches on either endpoint: {kept:?}"
    );

    let bare = filter_trace_edges(&edges, &Some("bare_exact".to_string()));
    assert_eq!(bare.len(), 1, "exact bare-name match still works");

    let none = filter_trace_edges(&edges, &None);
    assert_eq!(
        none.len(),
        4,
        "no target → M-01 filter passes everything through"
    );
}

#[test]
fn target_filter_preserves_wire_order() {
    let edges = vec![ge("f", "p.b"), ge("f", "p.a"), ge("f", "p.c")];
    let kept = filter_trace_edges(&edges, &None);
    assert_eq!(kept[0].to, "p.b");
    assert_eq!(kept[1].to, "p.a");
    assert_eq!(kept[2].to, "p.c");
}

/// REGRESSION (2026-08-24): bare `to` name + QUALIFIED wire endpoint ⇒ the
/// edge is RETAINED. Live shape: `graph_trace(callee, caller)` sends a bare
/// target while CBM reports the caller under its qualified name; the former
/// equality-only M-01 check discarded every such edge, so even correctly
/// parsed traces collapsed to zero results.
#[test]
fn regression_bare_to_name_with_qualified_endpoint_is_retained() {
    let edges = vec![ge(
        "tw_probe_callee",
        "C-Users-MNasty-AppData-Local-Temp-cleanctx_trace_wire_x.src.trace_fixture.tw_probe_caller",
    )];
    let kept = filter_trace_edges(&edges, &Some("tw_probe_caller".to_string()));
    assert_eq!(
        kept.len(),
        1,
        "bare to-name must retain the qualified wire edge, got {kept:?}"
    );
    assert_eq!(
        kept[0].to, edges[0]["to"],
        "wire identity preserved untouched"
    );
}

/// The boundary contract is EXACT-OR-FINAL-SEGMENT, nothing looser:
/// partial and multi-segment targets match nothing.
#[test]
fn target_filter_rejects_partial_and_multisegment_targets() {
    let edges = vec![ge("f", "p.mod.tw_probe_caller")];
    for target in ["probe_caller", "mod.tw_probe_caller"] {
        let kept = filter_trace_edges(&edges, &Some(target.to_string()));
        assert!(
            kept.is_empty(),
            "non-contract target '{target}' must not match"
        );
    }
    // Fully qualified target matches exactly.
    let exact = filter_trace_edges(&edges, &Some("p.mod.tw_probe_caller".to_string()));
    assert_eq!(exact.len(), 1);
}

// ── Fresh-process live probes against a SYNTHETIC fixture repo ───────
//
// Each probe owns a fresh GraphBridge ⇒ fresh CBM subprocess + fresh index
// of a throwaway single-file crate. No assertion depends on any symbol,
// file, or relationship inside the Clean-CTX repository itself.

/// Whether the CBM binary is present (without launching a second copy).
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

/// Wait until the fixture project's index is Ready (each `try_create`
/// triggers a background re-index; querying mid-rebuild observes partial
/// graphs). Mirrors graph_intel.rs's per-bridge gating.
fn wait_ready(bridge: &mut GraphBridge) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        match bridge.ensure_indexed() {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => return,
            Ok(crate::cbm::bridge::IndexingStatus::StillIndexing { .. }) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "timed out waiting for CBM indexing of the trace fixture"
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

/// caller → callee is the ONLY call relationship; `isolated` touches nothing.
const FIXTURE_RS: &str = r#"
pub fn tw_probe_callee() -> i32 { 7 }
pub fn tw_probe_caller() -> i32 { tw_probe_callee() + 1 }
pub fn tw_probe_isolated() -> i32 { 3 }
"#;

/// Spawn a fresh bridge over a throwaway fixture repo. The TempDir MUST be
/// kept alive by the caller for the bridge's lifetime.
fn fresh_fixture_bridge() -> (tempfile::TempDir, GraphBridge, String) {
    let root = tempfile::Builder::new()
        .prefix("cleanctx_trace_wire_")
        .tempdir()
        .expect("fixture tempdir");
    std::fs::write(root.path().join("trace_fixture.rs"), FIXTURE_RS)
        .expect("write trace fixture source");
    let canon = root
        .path()
        .canonicalize()
        .unwrap_or_else(|_| root.path().to_path_buf());
    let project = crate::cbm::bridge::cbm_project_slug(&canon);
    let mut bridge = GraphBridge::try_create(&live_config(), root.path());
    wait_ready(&mut bridge);
    (root, bridge, project)
}

/// L1 — typed `graph_query` surfaces caller/callee CALLS rows.
#[serial(cbm_live)]
#[test]
fn live_typed_graph_query_returns_caller_callee_rows() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, bridge, project) = fresh_fixture_bridge();
    let cypher = "MATCH (a:Function)-[:CALLS]->(b:Function) WHERE a.name = 'tw_probe_caller' RETURN a.name, b.name".to_string();
    let rows = {
        let mut guard = bridge.client.lock().unwrap_or_else(|p| p.into_inner());
        let client = guard.as_mut().expect("live CBM client");
        client
            .query_graph(&cypher, &project)
            .expect("typed query_graph must execute on the live CBM")
            .rows
    };
    let hit = rows.iter().any(|r| {
        r.first().and_then(|v| v.as_str()) == Some("tw_probe_caller")
            && r.get(1).and_then(|v| v.as_str()) == Some("tw_probe_callee")
    });
    assert!(
        hit,
        "typed graph_query must surface the caller→callee CALLS row; got {rows:?}"
    );
}

/// L2 — outbound-reachable pair resolves via the FIRST (outbound) attempt,
/// exactly as before the fix; unrelated pairs yield zero invented edges.
#[serial(cbm_live)]
#[test]
fn live_two_endpoint_outbound_trace_is_preserved() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge, _project) = fresh_fixture_bridge();

    let edges = bridge.trace_path("tw_probe_caller", "tw_probe_callee");
    assert_eq!(
        edges.len(),
        1,
        "direct outbound edge must be found: {edges:?}"
    );
    assert_eq!(
        edges[0].from, "tw_probe_caller",
        "traced side keeps the caller's identity"
    );
    assert!(
        edges[0].to.ends_with(".tw_probe_callee"),
        "discovered side uses CBM's qualified wire identity: {}",
        edges[0].to
    );
    assert_eq!(edges[0].label, "calls");
    assert!(
        bridge.take_last_error().is_none(),
        "success must clear last_error"
    );

    let none = bridge.trace_path("tw_probe_caller", "tw_probe_isolated");
    assert!(
        none.is_empty(),
        "unrelated pair must yield zero edges — never invented ones: {none:?}"
    );
    assert!(bridge.take_last_error().is_none());
}

/// L3 — single-endpoint trace discovers the inbound caller of a leaf.
#[serial(cbm_live)]
#[test]
fn live_single_endpoint_trace_discovers_inbound_caller() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge, _project) = fresh_fixture_bridge();

    // Empty `to` → pre-existing "both" sweep semantics (unchanged by the fix).
    let edges = bridge.trace_path("tw_probe_callee", "");
    assert_eq!(
        edges.len(),
        1,
        "leaf function has exactly one caller: {edges:?}"
    );
    assert!(
        edges[0].from.ends_with(".tw_probe_caller"),
        "caller arrives with its qualified wire identity: {}",
        edges[0].from
    );
    assert_eq!(edges[0].to, "tw_probe_callee");
    assert!(bridge.take_last_error().is_none());
}

/// L4 — THE REGRESSION: both endpoints supplied, only an INBOUND
/// relationship exists. Pre-fix this returned zero edges forever (direction
/// hardcoded outbound AND the phantom `edges` key collapsed every response).
/// The fallback must discover callee ← caller exactly once.
#[serial(cbm_live)]
#[test]
fn live_two_endpoint_trace_discovers_inbound_only_relationship() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge, _project) = fresh_fixture_bridge();

    // Outbound from `tw_probe_callee` finds nothing touching the target;
    // the single inbound attempt then finds caller → callee.
    let edges = bridge.trace_path("tw_probe_callee", "tw_probe_caller");
    assert_eq!(
        edges.len(),
        1,
        "inbound-only relationship must be discovered via the inbound fallback: {edges:?}"
    );
    assert!(
        edges[0].from.ends_with(".tw_probe_caller"),
        "edge origin is the qualified caller: {}",
        edges[0].from
    );
    assert_eq!(edges[0].to, "tw_probe_callee");
    assert_eq!(edges[0].label, "calls");
    assert!(
        bridge.take_last_error().is_none(),
        "discovery is success, not error"
    );

    // Symmetry: tracing the same pair in the natural call direction still
    // resolves through the FIRST attempt (no behavioral change).
    let forward = bridge.trace_path("tw_probe_caller", "tw_probe_callee");
    assert_eq!(forward.len(), 1, "{forward:?}");
}
