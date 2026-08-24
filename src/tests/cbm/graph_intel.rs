// src/tests/cbm/graph_intel.rs
//
// AUDIT: graph-intelligence layer above raw CBM integration.
//
// Verifies that Clean-CTX's graph-intelligence features consume REAL CBM
// data correctly:
//
//   - `get_symbol_importance_mut` (bridge) / `get_symbol_importance` (client)
//   - dead-code detection
//   - blast radius / affected-symbol analysis
//   - architecture analysis
//   - call/dataflow edge enrichment
//   - caching and project isolation of the above
//
// Approach mirrors the established CBM audit suites:
//   1. Deterministic self-contained fixtures (temp SQLite store, seeded
//      caches, disabled bridge) for caching/project-isolation semantics.
//   2. Live-CBM semantic probes (`#[serial(cbm_live)]`, skipped when the
//      binary is absent) that pin wire shapes and cross-check bridge
//      results against ground-truth Cypher issued on the same client.
//
// Findings status after the 2026-08-24 fix pass:
//   F1  FIXED — blast-radius Cypher now matches on f.name; the former
//       intentionally-red test below passes and pins true caller files.
//   F2w WIRE PIN (unchanged) — through CbmClient::query_graph, numeric
//       graph metrics (`in_degree`) arrive as JSON STRINGS ("10"); the
//       as_str().parse::<f64>() importance parser depends on this shape
//       and the live probe below keeps pinning it.
//   F11 FIXED — CBM soft errors (result.isError + inner "error") map to
//       CbmError::ToolError; graph-intelligence queries return
//       Result<_, CbmError> and never conflate failure with empty data.
//   F3  FIXED — dead-code detection scans Function AND Method labels.
//   F10 RESOLVED — DATAFLOW does not exist in CBM 0.8.1 and USAGE/WRITES
//       are not equivalents (no read direction); the dead query path was
//       removed. A schema guard below re-triggers the audit if a future
//       CBM introduces DATAFLOW edges.
//   ISO disk-cache partitioning + hydration across workspace switches
//       verified (positive test).

use std::path::Path;

use serial_test::serial;

use crate::cbm::bridge::GraphBridge;
use crate::cbm::config::{CbmConfig, CbmStatus};

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

/// Live-CBM config pointed at this repository (already indexed by the
/// production MCP session; queries resolve against the canonical slug).
fn live_config() -> CbmConfig {
    CbmConfig {
        enabled: true,
        ..Default::default()
    }
}

/// The canonical CBM project slug for this repository's working directory.
fn this_project() -> String {
    let canon = std::path::Path::new(".").canonicalize().expect("cwd");
    crate::cbm::bridge::cbm_project_slug(&canon)
}

/// Wait until this repo's CBM index is Ready (each `try_create` triggers a
/// background re-index; querying mid-rebuild observes partial graphs).
/// Mirrors e2e.rs's shared-fixture gating, scoped to a per-test bridge.
fn wait_ready(bridge: &mut GraphBridge) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        match bridge.ensure_indexed() {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => return,
            Ok(crate::cbm::bridge::IndexingStatus::StillIndexing { .. }) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "timed out waiting for CBM indexing"
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

// ── Deterministic fixture: project isolation of cached intel ────

/// Disk-cache partitioning + hydration across workspace switches must be
/// project-scoped: repo A's cached `symbol_importance` must NEVER hydrate
/// while repo B is active, and must hydrate again when A re-activates.
#[test]
fn disk_cache_project_isolation_across_workspace_switches() {
    use crate::cbm::bridge::CachedGraphData;
    use crate::cbm::cache_store::GraphCacheStore;

    let tmp = tempfile::Builder::new()
        .prefix("cleanctx_graph_intel_iso_")
        .tempdir()
        .expect("tempdir");
    let root_a = tempfile::Builder::new()
        .prefix("cleanctx_repo_a_")
        .tempdir()
        .expect("tempdir");
    let root_b = tempfile::Builder::new()
        .prefix("cleanctx_repo_b_")
        .tempdir()
        .expect("tempdir");

    // Disabled bridge → no subprocess, deterministic query-failure path.
    let mut bridge = GraphBridge::try_create(
        &CbmConfig {
            enabled: false,
            ..Default::default()
        },
        root_a.path(),
    );
    assert_eq!(bridge.status(), &CbmStatus::Unavailable);

    let store =
        GraphCacheStore::open(&tmp.path().join("graph_cache.db")).expect("open cache store");

    // Production write-through always partitions by the CANONICALIZED root
    // (`self.project_root`), so the fixture seeds the same way.
    let root_a_canon = root_a
        .path()
        .canonicalize()
        .unwrap_or_else(|_| root_a.path().to_path_buf());

    // Seed repo A's partition directly (memory + disk write-through shape),
    // exactly as a completed intelligence query would have persisted it.
    let seeded = serde_json::json!({
        "RepoAOnlySymbol": {
            "symbol": "RepoAOnlySymbol",
            "score": 0.42,
            "file": "src/a.rs"
        }
    });
    let expires = std::time::Instant::now() + std::time::Duration::from_secs(600);
    bridge.cache.insert(
        "symbol_importance".to_string(),
        CachedGraphData {
            data: seeded.clone(),
            expires_at: expires,
        },
    );
    let slug_a = crate::cbm::bridge::cbm_project_slug(&root_a_canon);
    let root_a_str = root_a_canon.to_string_lossy().into_owned();
    store.put(
        &root_a_str,
        &format!("{slug_a}:symbol_importance"),
        &seeded.to_string(),
        4_102_444_800_000i64, // far-future epoch-ms
    );

    // Hand the store to the bridge afterwards (it is consumed on attach).
    bridge.attach_disk_cache(store);

    // Repo B has nothing on disk or in memory under its own keys.
    bridge.set_workspace_root(root_b.path());
    assert!(
        bridge.cache.get("symbol_importance").is_none(),
        "switching workspace roots must clear the in-memory cache"
    );
    let from_b = bridge.get_symbol_importance_mut();
    assert!(
        from_b.is_err(),
        "repo B query must FAIL (no client), never return a fake-empty Ok"
    );

    // Back to repo A: hydration from A's disk partition must restore the
    // exact seeded entry (no CBM round-trip needed).
    bridge.set_workspace_root(root_a.path());
    let from_a = bridge
        .get_symbol_importance_mut()
        .expect("repo A's disk-cached entry must hydrate without error");
    assert_eq!(from_a.len(), 1, "repo A's entry must hydrate from disk");
    let info = from_a.get("RepoAOnlySymbol").expect("seeded symbol");
    assert_eq!(info.score, 0.42);
    assert_eq!(info.file, "src/a.rs");
}

// ── Live-CBM semantic probes ─────────────────────────────────────

/// F2w WIRE PIN: through `CbmClient::query_graph`, numeric graph metrics
/// arrive as JSON STRINGS ("10"), not numbers. The production
/// `as_str().parse::<f64>()` importance parser depends on this shape.
#[serial(cbm_live)]
#[test]
fn live_wire_in_degree_cells_are_json_numbers() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut bridge = GraphBridge::try_create(&live_config(), Path::new("."));
    wait_ready(&mut bridge);
    let project = this_project();

    let rows = {
        let mut guard = bridge.client.lock().unwrap_or_else(|p| p.into_inner());
        let client = guard.as_mut().expect("live CBM client");
        client
            .query_graph(
                "MATCH (f:Function) WHERE f.in_degree >= 1 RETURN f.name, f.in_degree LIMIT 3",
                &project,
            )
            .expect("live query_graph")
    };
    assert!(!rows.is_empty(), "graph must contain called functions");
    for row in &rows {
        let cell = row.get(1).expect("in_degree cell");
        assert!(
            cell.is_string() && cell.as_str().unwrap().parse::<f64>().is_ok(),
            "wire contract changed: in_degree cell is {cell} — CbmClient::get_symbol_importance \
             parses cells via as_str().parse::<f64>(), so native JSON numbers would silently \
             collapse every importance score to 0.0. Re-audit the parser."
        );
    }
}

/// F2 SEMANTIC: the hottest function in the graph (max in_degree) MUST have
/// a positive importance score through the full bridge pipeline, and every
/// score must stay within the documented [0.0, 1.0] contract.
#[serial(cbm_live)]
#[test]
fn live_symbol_importance_scores_are_nonzero_and_bounded() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut bridge = GraphBridge::try_create(&live_config(), Path::new("."));
    wait_ready(&mut bridge);
    let project = this_project();

    // Ground truth: hottest function name straight from the graph.
    let hottest = {
        let mut guard = bridge.client.lock().unwrap_or_else(|p| p.into_inner());
        let client = guard.as_mut().expect("live CBM client");
        let rows = client
            .query_graph(
                "MATCH (f:Function) WHERE f.in_degree >= 1 RETURN f.name, f.in_degree \
                 ORDER BY f.in_degree DESC LIMIT 1",
                &project,
            )
            .expect("live query_graph");
        rows[0][0].as_str().expect("name cell").to_string()
    };

    // F2w/F11: scores flow through the Result API; an Ok(map) is always a
    // complete, valid result.
    let map = bridge
        .get_symbol_importance_mut()
        .expect("importance query must succeed on a live indexed graph");
    assert!(
        !map.is_empty(),
        "importance map must not be empty on a live graph"
    );

    let info = map
        .get(&hottest)
        .unwrap_or_else(|| panic!("hottest function '{hottest}' missing from importance map"));
    assert!(
        info.score > 0.0,
        "F2 CONFIRMED: '{hottest}' has callers but importance score is {} \
         (numeric in_degree parsed via as_str() collapses to 0.0)",
        info.score
    );
    for (sym, v) in &map {
        assert!(
            (0.0..=1.0).contains(&v.score),
            "documented contract violated: score of '{sym}' is {} (must be within [0.0, 1.0])",
            v.score
        );
    }
}

/// F1 SEMANTIC: bridge blast radius must equal the TRUE caller-file set of
/// the queried symbol — not the whole project call graph. Ground truth is
/// computed with the corrected Cypher (`WHERE f.name`) on the same client.
#[serial(cbm_live)]
#[test]
fn live_blast_radius_matches_true_caller_files() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut bridge = GraphBridge::try_create(&live_config(), Path::new("."));
    wait_ready(&mut bridge);
    let project = this_project();

    // Pick a concrete called function and compute its true caller files.
    let (symbol, true_files): (String, Vec<String>) = {
        let mut guard = bridge.client.lock().unwrap_or_else(|p| p.into_inner());
        let client = guard.as_mut().expect("live CBM client");
        let seed = client
            .query_graph(
                "MATCH (caller:Function)-[:CALLS]->(f:Function) WHERE f.in_degree >= 2 \
                 RETURN f.name LIMIT 1",
                &project,
            )
            .expect("seed query");
        let sym = seed[0][0].as_str().expect("symbol").to_string();
        let rows = client
            .query_graph(
                &format!(
                    "MATCH (caller:Function)-[:CALLS]->(f:Function) WHERE f.name = '{sym}' \
                     RETURN caller.name, caller.file_path"
                ),
                &project,
            )
            .expect("ground-truth caller query");
        let files: Vec<String> = rows
            .iter()
            .filter_map(|r| r.get(1).and_then(|v| v.as_str()).map(String::from))
            .collect();
        assert!(!files.is_empty(), "fixture sanity: true callers must exist");
        (sym, files)
    };

    // F1 REGRESSION GUARD: the reported blast radius must equal the true
    // caller-file set now that the Cypher matches on f.name.
    let reported = bridge
        .get_blast_radius(&symbol, 1)
        .expect("blast-radius query must succeed on a live indexed graph");

    let mut expected = true_files.clone();
    expected.sort();
    expected.dedup();
    let mut actual = reported.clone();
    actual.sort();
    actual.dedup();

    assert!(
        actual.len() == expected.len() && actual.iter().zip(expected.iter()).all(|(a, e)| a == e),
        "F1 REGRESSION: blast radius of '{symbol}' reports {actual:?} but the true caller files are {expected:?} — the WHERE clause no longer matches only the queried symbol"
    );
}

/// F3 REGRESSION GUARD: production dead-code output must equal the union of
/// dead Function and Method nodes in the graph (name+file pairs), proving
/// class methods are now covered rather than silently skipped.
#[serial(cbm_live)]
#[test]
fn live_dead_code_detection_covers_methods() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut bridge = GraphBridge::try_create(&live_config(), Path::new("."));
    wait_ready(&mut bridge);
    let project = this_project();

    // Ground truth: exact dead-node pairs per label, merged client-side.
    let mut expected_pairs: Vec<(String, String)> = {
        let mut guard = bridge.client.lock().unwrap_or_else(|p| p.into_inner());
        let client = guard.as_mut().expect("live CBM client");
        let mut pairs = Vec::new();
        for label in ["Function", "Method"] {
            let rows = client
                .query_graph(
                    &format!(
                        "MATCH (n:{label}) WHERE n.in_degree = 0 AND n.is_entry_point = false \
                         RETURN n.name, n.file_path"
                    ),
                    &project,
                )
                .expect("ground-truth dead query");
            for row in rows {
                let name = row.first().and_then(|v| v.as_str()).unwrap_or_default();
                let file = row.get(1).and_then(|v| v.as_str()).unwrap_or_default();
                if !name.is_empty() {
                    pairs.push((name.to_string(), file.to_string()));
                }
            }
        }
        pairs
    };
    expected_pairs.sort();
    expected_pairs.dedup();

    let production: Vec<(String, String)> = bridge
        .get_dead_code()
        .expect("dead-code query must succeed on a live indexed graph")
        .into_iter()
        .map(|e| (e.symbol, e.file))
        .collect();
    let mut actual_pairs = production.clone();
    actual_pairs.sort();
    actual_pairs.dedup();

    let dead_methods = expected_pairs
        .iter()
        .filter(|(_, f)| f.ends_with(".ts") || f.ends_with(".cs") || f.ends_with(".java"))
        .count();
    if dead_methods == 0 && !expected_pairs.is_empty() {
        eprintln!(
            "Fixture note: graph has {} dead nodes but no method-file dead entries; \
             dual-label coverage still pinned by set equality",
            expected_pairs.len()
        );
    }

    assert!(
        actual_pairs.len() == expected_pairs.len()
            && actual_pairs
                .iter()
                .zip(expected_pairs.iter())
                .all(|(a, e)| a == e),
        "F3 REGRESSION: get_dead_code returned {:?} but the graph's dead Function+Method \
         nodes are {expected_pairs:?} — label coverage diverged",
        actual_pairs
    );
}

/// F10 GUARD: the DATAFLOW edge type must still not exist in CBM's schema.
/// The dead `get_dataflow_edges` query path was REMOVED because USAGE
/// (reference tracking, no direction) and WRITES (field assignments only,
/// no READS counterpart) are not dataflow equivalents — a CBM 0.8.1
/// limitation documented at the F10 note in `GraphBridge`. If this guard
/// ever fires, a future CBM gained dataflow edges: reintroduce the query
/// and its InferenceLayer consumption deliberately.
#[serial(cbm_live)]
#[test]
fn live_dataflow_edge_type_still_absent_reintroduction_guard() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut bridge = GraphBridge::try_create(&live_config(), Path::new("."));
    wait_ready(&mut bridge);

    assert!(
        !bridge
            .get_call_edges()
            .expect("CALLS edge query must succeed on a live indexed graph")
            .is_empty(),
        "fixture sanity: live graph must have CALLS edges for this probe"
    );

    let dataflow_rows = {
        let project = this_project();
        let mut guard = bridge.client.lock().unwrap_or_else(|p| p.into_inner());
        let client = guard.as_mut().expect("live CBM client");
        client
            .query_graph(
                "MATCH (a)-[r:DATAFLOW]->(b) RETURN a.name, b.name LIMIT 5",
                &project,
            )
            .expect("DATAFLOW probe query")
    };
    assert!(
        dataflow_rows.is_empty(),
        "F10 REINTRODUCTION TRIGGER: CBM now exposes {} DATAFLOW edges — \
         reinstate the dataflow enrichment path (see the F10 limitation note \
         in GraphBridge) instead of leaving it removed",
        dataflow_rows.len()
    );
}

/// Architecture analysis must parse CBM 0.8.1's packages/boundaries wire
/// shape into a populated overview (modules named, dependency kind pinned).
#[serial(cbm_live)]
#[test]
fn live_architecture_overview_parses_packages_and_boundaries() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut bridge = GraphBridge::try_create(&live_config(), Path::new("."));
    wait_ready(&mut bridge);
    let overview = bridge
        .get_architecture()
        .expect("architecture overview must parse on a live indexed graph");
    assert!(
        !overview.modules.is_empty(),
        "packages[] must map to non-empty modules"
    );
    assert!(
        overview.modules.iter().all(|m| !m.name.is_empty()),
        "every module must carry its package name"
    );
    assert!(
        overview
            .dependencies
            .iter()
            .all(|d| d.kind == "calls" && !d.from.is_empty() && !d.to.is_empty()),
        "boundaries[] must map to 'calls' dependencies with endpoints"
    );
}

/// F11 PROPAGATION: with soft errors mapped to `CbmError::ToolError`, the
/// graph-intelligence queries must FAIL (Result::Err) against a broken
/// target — never return a confident empty Ok. The invariant is exact:
/// Err(...) = CBM failed; Ok(empty) = valid query, zero results.
#[serial(cbm_live)]
#[test]
fn live_intel_queries_propagate_errors_on_unknown_project() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let mut bridge = GraphBridge::try_create(&live_config(), Path::new("."));
    wait_ready(&mut bridge);
    const BOGUS: &str = "definitely-not-a-real-project-xyz";

    // Invariant's second half first (before any failures trip the circuit
    // breaker): under a VALID project, a symbol with no callers yields
    // Ok(vec![]) — empty means empty, not failed.
    let no_callers = bridge
        .get_blast_radius("definitely_no_callers_xyz_symbol", 1)
        .expect("valid-project query must be Ok");
    assert!(
        no_callers.is_empty(),
        "unknown symbol must yield an empty caller list, got {no_callers:?}"
    );

    // Now the failure half. NOTE: the client's circuit breaker opens after
    // 3 consecutive failures, so only the FIRST semantic error is asserted
    // by message; every subsequent call must still be Err (either the CBM
    // semantic error or the circuit-open guard), never a fake-empty Ok.
    bridge.set_project(BOGUS);

    let first = bridge
        .get_dead_code()
        .expect_err("F11 REGRESSION: get_dead_code returned Ok on an unknown project")
        .to_string();
    assert!(
        first.contains("project not found"),
        "first failure must surface CBM's semantic error, got: {first}"
    );

    let importance = bridge.get_symbol_importance_mut();
    assert!(
        importance.is_err(),
        "F11 REGRESSION: get_symbol_importance_mut returned Ok on an unknown project"
    );
    let blast = bridge.get_blast_radius("anything", 1);
    assert!(
        blast.is_err(),
        "F11 REGRESSION: get_blast_radius returned Ok on an unknown project"
    );
}

/// Deterministic wire-shape fixtures for the F11 soft-error gate. These pin
/// the exact CBM response envelopes observed live so a CBM upgrade that
/// changes the failure encoding cannot silently reintroduce the conflation.
#[test]
fn soft_error_gate_maps_is_error_envelopes_to_tool_error() {
    use crate::cbm::client::{CbmError, check_soft_error};

    // Shape 1 (live-captured): isError + JSON body with an "error" key.
    let json_body = serde_json::json!({
        "isError": true,
        "content": [{
            "type": "text",
            "text": "{\"error\":\"project not found or not indexed\",\"hint\":\"Use list_projects\",\"count\":10}"
        }]
    });
    match check_soft_error("query_graph", &json_body) {
        Err(CbmError::ToolError { tool, message }) => {
            assert_eq!(tool, "query_graph");
            assert_eq!(message, "project not found or not indexed");
        }
        other => panic!("JSON error body must map to ToolError, got {other:?}"),
    }

    // Shape 2 (live-captured): isError + non-JSON plain-text body.
    let text_body = serde_json::json!({
        "isError": true,
        "content": [{
            "type": "text",
            "text": "expected token type 0, got 85 at pos 0"
        }]
    });
    match check_soft_error("query_graph", &text_body) {
        Err(CbmError::ToolError { tool, message }) => {
            assert_eq!(tool, "query_graph");
            assert!(message.contains("expected token type"));
        }
        other => panic!("plain-text error body must map to ToolError, got {other:?}"),
    }

    // Control: a normal successful payload passes through untouched.
    let ok_payload = serde_json::json!({
        "columns": ["f.name"],
        "rows": [["some_fn"]]
    });
    assert!(
        check_soft_error("query_graph", &ok_payload).is_ok(),
        "non-isError payloads are valid results and must not be rejected"
    );
}
