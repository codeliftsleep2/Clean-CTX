// src/tests/mcp/d4_advisory_meta.rs
//
// Phase D4 — retrieval-only CBM advisory metadata.
//
// Verifies that `provide_code_context` attaches the bounded, advisory
// `_meta.cbm` block (derived from the already-collected D1/D2
// `CbmIntelligence`) without changing compilation, the MCP envelope, or the
// authority boundary.
//
// Coverage:
//   - pure `cbm_advisory_meta` renderer (shape, omission, determinism, bounds)
//   - `inject_cbm_advisory_meta` (additive, preserves `decision_summary`)
//   - raw-passthrough / token-economics branch
//   - real handler dispatch (full + delta branches)
//   - wire vs internal distinction (D4 consumes the reduced `DataFlowContext`,
//     never the CBM grouped `{cols, groups}` wire JSON)

use crate::cbm::bridge::DataFlowContext;
use crate::cbm::{GraphBridge, SymbolImportance};
use crate::intelligence::fidelity::CbmIntelligence;
use crate::mcp::tool_handlers::core::{
    cbm_advisory_meta, inject_cbm_advisory_meta, maybe_economics_fallback,
};
use crate::mcp::tools::dispatch_tools_call;
use crate::tests::assert_valid_mcp_envelope;
use serde_json::json;
use std::collections::{HashMap, HashSet};

// ── Helpers ────────────────────────────────────────────────────────────

/// Build a `DataFlowContext` from string-vector inputs (HashSet insertion
/// order is irrelevant — D4 must sort before emission).
fn mk_df(symbols: Vec<&str>, files: Vec<&str>) -> DataFlowContext {
    DataFlowContext {
        symbols: symbols.into_iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
        files: files.into_iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
    }
}

/// Build a request-scoped `CbmIntelligence` with the two advisory
/// capabilities, exactly as `CbmIntelligence` is carried by
/// `ContextDecision` after a D1/D2 consultation.
fn make_intel(
    df_ctx: Option<DataFlowContext>,
    cs_ctx: Option<DataFlowContext>,
) -> CbmIntelligence {
    CbmIntelligence {
        importance: HashMap::new(),
        skip_set: HashSet::new(),
        data_flow: df_ctx,
        cross_service: cs_ctx,
    }
}

/// A mock GraphBridge that consults as "available" and serves a seeded
/// importance map plus seeded data-flow/cross-service traces for the
/// requested fixture file (score 0.5 → `NoRecommendation`, so the
/// consultation does not change the fidelity decision).
fn make_seeded_bridge() -> GraphBridge {
    use crate::cbm::bridge::test_helpers::{new_mock, seed_cross_service, seed_data_flow};
    let mut data = HashMap::new();
    data.insert(
        "CriticalAPI".to_string(),
        SymbolImportance {
            symbol: "CriticalAPI".into(),
            score: 0.5,
            file: "greeter.ts".into(),
        },
    );
    let bridge = new_mock(data);
    seed_data_flow(&bridge, "CriticalAPI", 2, mk_df(vec!["helper", "gateway"], vec!["svc.ts"]));
    seed_cross_service(&bridge, "CriticalAPI", 2, mk_df(vec!["remote_helper"], vec![]));
    bridge
}

const TS_FIXTURE: &str = "export class Greeter {\n    private prefix: string;\n    constructor(prefix: string) {\n        this.prefix = prefix;\n    }\n    greet(name: string): string {\n        return this.prefix + ', ' + name;\n    }\n}\n";

struct D4Fixture {
    _dir: tempfile::TempDir,
    path: String,
    root: String,
}

fn d4_temp_fixture() -> D4Fixture {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let file = dir.path().join("greeter.ts");
    std::fs::write(&file, TS_FIXTURE).expect("write fixture");
    let root = dir.path().to_string_lossy().to_string();
    let path = file.to_string_lossy().to_string();
    D4Fixture {
        _dir: dir,
        path,
        root,
    }
}

// ══════════════════════════════════════════════════════════════════════
// Test 1 — Data-flow advisory metadata
// ══════════════════════════════════════════════════════════════════════

#[test]
fn d4_data_flow_advisory_meta_emitted() {
    let intel = make_intel(Some(mk_df(vec!["b_sym", "a_sym"], vec!["b.rs", "a.rs"])), None);
    let cbm = cbm_advisory_meta(&intel);
    let obj = cbm.as_object().expect("cbm object");
    assert_eq!(obj["advisory"].as_bool(), Some(true));
    let dfo = obj["data_flow"].as_object().expect("data_flow object");
    let syms = dfo["symbols"].as_array().expect("symbols array");
    assert_eq!(
        syms.iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>(),
        vec!["a_sym", "b_sym"],
        "symbols sorted deterministically"
    );
    let files = dfo["files"].as_array().expect("files array");
    assert_eq!(
        files.iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>(),
        vec!["a.rs", "b.rs"],
        "files sorted deterministically"
    );
    assert!(
        !obj.contains_key("cross_service"),
        "cross_service omitted entirely when None (absence ≠ empty)"
    );

    // Injection is additive: `decision_summary` (with the existing cbm_df=
    // /cbm_cs= diagnostics) is preserved byte-for-byte.
    let mut response = json!({
        "jsonrpc": "2.0", "id": 1, "result": {
            "content": [{ "type": "text", "text": "ctx" }],
            "_meta": {
                "decision_summary": "fidelity=medium, cbm_df=2sym/2files, cbm_cs=0sym/0files"
            }
        }
    });
    inject_cbm_advisory_meta(&mut response, Some(&cbm));
    let meta = response["result"]["_meta"].as_object().expect("_meta");
    assert_eq!(
        meta["decision_summary"].as_str(),
        Some("fidelity=medium, cbm_df=2sym/2files, cbm_cs=0sym/0files"),
        "decision_summary unchanged"
    );
    assert_eq!(meta["cbm"]["advisory"].as_bool(), Some(true), "block attached under _meta.cbm");
}

// ══════════════════════════════════════════════════════════════════════
// Test 2 — Cross-service advisory metadata
// ══════════════════════════════════════════════════════════════════════

#[test]
fn d4_cross_service_advisory_meta_emitted() {
    let intel = make_intel(None, Some(mk_df(vec!["handler", "gateway"], vec!["svc.ts"])));
    let cbm = cbm_advisory_meta(&intel);
    let obj = cbm.as_object().expect("cbm object");
    assert_eq!(obj["advisory"].as_bool(), Some(true));
    let cso = obj["cross_service"].as_object().expect("cross_service object");
    let syms = cso["symbols"].as_array().expect("symbols array");
    let syms_vec = syms.iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>();
    assert!(syms_vec.contains(&"gateway") && syms_vec.contains(&"handler"));
    assert!(
        !obj.contains_key("data_flow"),
        "data_flow omitted entirely when None (absence ≠ empty)"
    );
}

// ══════════════════════════════════════════════════════════════════════
// Test 3 — Both capabilities
// ══════════════════════════════════════════════════════════════════════

#[test]
fn d4_both_capabilities_emitted() {
    let intel = make_intel(
        Some(mk_df(vec!["a"], vec!["a.rs"])),
        Some(mk_df(vec!["b"], vec!["b.rs"])),
    );
    let cbm = cbm_advisory_meta(&intel);
    let obj = cbm.as_object().expect("cbm object");
    assert_eq!(obj["advisory"].as_bool(), Some(true));
    assert!(obj.contains_key("data_flow"), "data_flow present when Some");
    assert!(obj.contains_key("cross_service"), "cross_service present when Some");
    assert_eq!(obj["data_flow"]["symbols"].as_array().expect("symbols").len(), 1);
    assert_eq!(obj["cross_service"]["symbols"].as_array().expect("symbols").len(), 1);
}

// ══════════════════════════════════════════════════════════════════════
// Test 4 — No CBM intelligence → no `_meta.cbm`
// ══════════════════════════════════════════════════════════════════════

#[test]
fn d4_no_cbm_intelligence_omits_meta_block() {
    // Pure: injecting `None` is a no-op.
    let mut response = json!({
        "jsonrpc": "2.0", "id": 1, "result": {
            "content": [{ "type": "text", "text": "ctx" }],
            "_meta": { "decision_summary": "fidelity=medium" }
        }
    });
    inject_cbm_advisory_meta(&mut response, None);
    let meta = response["result"]["_meta"].as_object().expect("_meta");
    assert!(!meta.contains_key("cbm"), "no _meta.cbm when there is no intelligence");

    // Handler: no bridge → no CBM consultation → the response carries no
    // `_meta.cbm` (absence means "no advisory result", not "CBM found none").
    let fx = d4_temp_fixture();
    let _serial = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({ "arguments": { "filePath": fx.path, "workspaceRoot": fx.root } }),
        &state,
    );
    let resp = crate::protocol::captured_responses().pop().expect("response");
    assert!(resp.get("error").is_none(), "no error: {resp}");
    let meta = resp["result"]["_meta"].as_object().expect("_meta");
    assert!(!meta.contains_key("cbm"), "no `_meta.cbm` without CBM consultation");
}

// ══════════════════════════════════════════════════════════════════════
// Test 5 — Explicit fidelity / intent prevent consultation → no metadata
// ══════════════════════════════════════════════════════════════════════

#[test]
fn d4_explicit_fidelity_and_intent_prevent_advisory_meta() {
    use crate::mcp::heuristics;
    let config = crate::config::CleanCtxConfig::default();
    let source = "export class Greeter { greet(): string { return 'hi'; } }";

    // Explicit fidelity → no CBM consultation → no advisory intelligence.
    let mut bridge = make_seeded_bridge();
    let decision = heuristics::decide(
        "/project/src/greeter.ts",
        Some("medium"),
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();
    assert!(decision.cbm_intelligence.is_none(), "explicit fidelity → no consultation");

    // Explicit intent → no CBM consultation → no advisory intelligence.
    let mut bridge = make_seeded_bridge();
    let decision = heuristics::decide(
        "/project/src/greeter.ts",
        None,
        Some("refactor"),
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();
    assert!(decision.cbm_intelligence.is_none(), "explicit intent → no consultation");
}

// ══════════════════════════════════════════════════════════════════════
// Test 6 — Deterministic ordering
// ══════════════════════════════════════════════════════════════════════

#[test]
fn d4_advisory_meta_deterministic_ordering() {
    // Intentionally unordered inputs (HashSet iteration order is arbitrary).
    let intel = make_intel(
        Some(mk_df(vec!["zeta", "alpha", "mid"], vec!["z.rs", "a.rs", "m.rs"])),
        Some(mk_df(vec!["omega", "beta"], vec![])),
    );
    let cbm = cbm_advisory_meta(&intel);
    let dfo = cbm["data_flow"].as_object().expect("data_flow");
    assert_eq!(
        dfo["symbols"].as_array().expect("symbols").iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>(),
        vec!["alpha", "mid", "zeta"],
        "symbols must be sorted"
    );
    assert_eq!(
        dfo["files"].as_array().expect("files").iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>(),
        vec!["a.rs", "m.rs", "z.rs"],
        "files must be sorted"
    );
    let cso = cbm["cross_service"].as_object().expect("cross_service");
    assert_eq!(
        cso["symbols"].as_array().expect("symbols").iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>(),
        vec!["beta", "omega"],
        "cross-service symbols must be sorted"
    );
}

// ══════════════════════════════════════════════════════════════════════
// Test 7 — Bounds preserved (D1/D2 caps are not exceeded at the boundary)
// ══════════════════════════════════════════════════════════════════════

#[test]
fn d4_advisory_meta_bounds_preserved() {
    // D1/D2 defaults: data_flow = 12 symbols / 6 files; cross_service = 20 / 8.
    let df_syms = (0..12).map(|i| format!("sym{i}")).collect::<Vec<_>>();
    let df_files = (0..6).map(|i| format!("file{i}.rs")).collect::<Vec<_>>();
    let cs_syms = (0..20).map(|i| format!("cs{i}")).collect::<Vec<_>>();
    let cs_files = (0..8).map(|i| format!("cfile{i}.rs")).collect::<Vec<_>>();
    let intel = make_intel(
        Some(DataFlowContext {
            symbols: df_syms.into_iter().collect::<HashSet<_>>(),
            files: df_files.into_iter().collect::<HashSet<_>>(),
        }),
        Some(DataFlowContext {
            symbols: cs_syms.into_iter().collect::<HashSet<_>>(),
            files: cs_files.into_iter().collect::<HashSet<_>>(),
        }),
    );
    let cbm = cbm_advisory_meta(&intel);
    let dfo = cbm["data_flow"].as_object().expect("data_flow");
    let cso = cbm["cross_service"].as_object().expect("cross_service");
    assert_eq!(dfo["symbols"].as_array().expect("symbols").len(), 12);
    assert_eq!(dfo["files"].as_array().expect("files").len(), 6);
    assert_eq!(cso["symbols"].as_array().expect("symbols").len(), 20);
    assert_eq!(cso["files"].as_array().expect("files").len(), 8);
}

// ══════════════════════════════════════════════════════════════════════
// Test 8 — Authority separation: compiled content + envelope unchanged,
// CBM data confined to the advisory `_meta.cbm` block.
// ══════════════════════════════════════════════════════════════════════

#[test]
fn d4_authority_separation_content_and_envelope_unchanged() {
    let fx = d4_temp_fixture();
    let _serial = crate::protocol::handler_response_serial();

    // Baseline: no CBM consultation → capture compiled content.
    crate::protocol::captured_responses().clear();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({ "arguments": { "filePath": fx.path, "workspaceRoot": fx.root } }),
        &state,
    );
    let baseline = crate::protocol::captured_responses().pop().expect("baseline response");
    let baseline_text = baseline["result"]["content"][0]["text"]
        .as_str()
        .expect("baseline content");

    // Seeded bridge → advisory block present; compiled content unchanged and
    // the response still validates as a canonical MCP envelope.
    crate::protocol::captured_responses().clear();
    let state2 = crate::mcp::McpState::new(crate::tests::test_config());
    {
        let mut guard = state2.graph_bridge_lock();
        *guard = Some(make_seeded_bridge());
    }
    dispatch_tools_call(
        &json!(2),
        "provide_code_context",
        &json!({ "arguments": { "filePath": fx.path, "workspaceRoot": fx.root } }),
        &state2,
    );
    let resp2 = crate::protocol::captured_responses().pop().expect("seeded response");
    assert!(resp2.get("error").is_none(), "no error: {resp2}");
    let result2 = resp2["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result2);
    let seeded_text = result2["content"][0]["text"].as_str().expect("seeded content");
    assert_eq!(
        seeded_text,
        baseline_text,
        "advisory metadata must not alter compiled content"
    );

    let meta2 = result2["_meta"].as_object().expect("_meta");
    let cbm = meta2["cbm"].as_object().expect("_meta.cbm present");
    assert_eq!(cbm["advisory"].as_bool(), Some(true));
    let dfo = cbm["data_flow"].as_object().expect("data_flow advisory");
    let syms = dfo["symbols"].as_array().expect("symbols array");
    let syms_vec = syms.iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>();
    assert!(
        syms_vec.contains(&"helper") && syms_vec.contains(&"gateway"),
        "seeded advisory symbols exposed"
    );

    // No CBM-derived relationship / identity fields leak into the advisory
    // block, and no CBM data appears outside `_meta`.
    for banned in ["hop", "qn_prefix", "from", "to", "pairs", "entities", "dependencies"] {
        assert!(
            !cbm.contains_key(banned),
            "no '{banned}' inside _meta.cbm: {resp2}"
        );
    }
    assert!(
        !result2.contains_key("structuredContent"),
        "no structuredContent introduced by D4"
    );
    assert!(
        !result2.contains_key("entities") && !result2.contains_key("dependencies"),
        "no CBM fields at the result level"
    );
}

// ══════════════════════════════════════════════════════════════════════
// Test 9 — Response-branch coverage (raw_passthrough + full + delta)
// ══════════════════════════════════════════════════════════════════════

#[test]
fn d4_response_branch_coverage() {
    let fx = d4_temp_fixture();
    let _serial = crate::protocol::handler_response_serial();

    // (a) Raw-passthrough / token-economics fallback branch.
    let cbm = cbm_advisory_meta(&make_intel(Some(mk_df(vec!["helper"], vec!["svc.ts"])), None));
    crate::protocol::captured_responses().clear();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let fell_back = maybe_economics_fallback(
        &json!(7),
        "hello world",
        100,
        200,
        &state,
        "/test/file.ts",
        false,
        crate::compression::Fidelity::Edit,
        "test",
        Some(&cbm),
    );
    assert!(fell_back, "candidate worse than raw must fall back");
    let resp = crate::protocol::captured_responses().pop().expect("fallback response");
    let meta = resp["result"]["_meta"].as_object().expect("_meta");
    assert_eq!(meta["decision_summary"].as_str(), Some("test"));
    assert!(meta.contains_key("cbm"), "raw_passthrough carries _meta.cbm");
    assert_eq!(meta["cbm"]["advisory"].as_bool(), Some(true));

    // (b) Real handler: first dispatch (full) and second dispatch (delta)
    // both carry the block through their response branches.
    crate::protocol::captured_responses().clear();
    let state2 = crate::mcp::McpState::new(crate::tests::test_config());
    {
        let mut guard = state2.graph_bridge_lock();
        *guard = Some(make_seeded_bridge());
    }
    dispatch_tools_call(
        &json!(8),
        "provide_code_context",
        &json!({ "arguments": { "filePath": fx.path, "workspaceRoot": fx.root } }),
        &state2,
    );
    let first = crate::protocol::captured_responses().pop().expect("first response");
    let meta1 = first["result"]["_meta"].as_object().expect("_meta (first)");
    assert!(meta1.contains_key("cbm"), "first (full) dispatch carries _meta.cbm");

    dispatch_tools_call(
        &json!(9),
        "provide_code_context",
        &json!({ "arguments": { "filePath": fx.path, "workspaceRoot": fx.root } }),
        &state2,
    );
    let second = crate::protocol::captured_responses().pop().expect("second response");
    let meta2 = second["result"]["_meta"].as_object().expect("_meta (second)");
    assert!(meta2.contains_key("cbm"), "second (delta) dispatch carries _meta.cbm");
}

// ══════════════════════════════════════════════════════════════════════
// Test 10 — Wire vs internal distinction: D4 consumes the reduced
// `DataFlowContext` produced by the D2-Wire grouped parser, not the raw
// CBM `{cols, groups}` JSON.
// ══════════════════════════════════════════════════════════════════════

#[test]
fn d4_wire_internal_distinction() {
    use crate::cbm::bridge::test_helpers::{new_mock, trace_context_from_wire};

    let mut data = HashMap::new();
    data.insert(
        "main".to_string(),
        SymbolImportance {
            symbol: "main".into(),
            score: 0.9,
            file: "api.rs".into(),
        },
    );
    data.insert(
        "route".to_string(),
        SymbolImportance {
            symbol: "route".into(),
            score: 0.7,
            file: "routes.rs".into(),
        },
    );
    // "handler" deliberately absent from importance → unresolvable endpoint.
    let mut bridge = new_mock(data);

    // CBM 0.10.8 grouped `{cols, groups}` wire shape (same parser as D2-Wire).
    let body = json!({
        "function": "proj.main",
        "direction": "both",
        "mode": "cross_service",
        "callees_total": 1,
        "callees": {
            "cols": ["name", "hop"],
            "groups": [{ "qn_prefix": "proj.svc", "rows": [["handler", 1]] }]
        },
        "callers_total": 1,
        "callers": {
            "cols": ["name", "hop"],
            "groups": [{ "qn_prefix": "proj.routes", "rows": [["route", 1]] }]
        }
    });

    // Grouped wire → parser → GraphEdge → reduced DataFlowContext.
    let ctx = trace_context_from_wire(&mut bridge, &body, "proj.main");
    let cbm = cbm_advisory_meta(&make_intel(Some(ctx), None));
    let dfo = cbm["data_flow"].as_object().expect("data_flow object");

    // D4 emits the REDUCED representation: bare symbol names only.
    let syms = dfo["symbols"].as_array().expect("symbols array");
    let syms_vec = syms.iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>();
    for expected in ["main", "handler", "route"] {
        assert!(syms_vec.contains(&expected), "bare symbol '{expected}' exposed");
    }
    assert!(
        syms_vec.iter().all(|s| !s.contains(".")),
        "no reconstructed qualified names manufactured"
    );

    // The grouped wire information intentionally lost before D4 must not be
    // reconstructed or exposed.
    for banned in ["hop", "qn_prefix", "from", "to"] {
        assert!(
            !dfo.contains_key(banned),
            "no '{banned}' in the advisory capability"
        );
    }

    // Files resolve via the current-project importance map only; the
    // unresolvable endpoint contributes no file.
    let files = dfo["files"].as_array().expect("files array");
    let files_vec = files.iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>();
    assert!(
        files_vec.contains(&"api.rs") && files_vec.contains(&"routes.rs"),
        "importance-resolved files exposed"
    );
    assert!(
        files_vec.iter().all(|f| !f.contains("handler")),
        "unresolvable endpoint contributes no file"
    );
}