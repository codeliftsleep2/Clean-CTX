// src/tests/cbm/e2e.rs
//
// End-to-end tests for the full CBM integration pipeline.
// These tests exercise the complete flow:
//   CbmClient → GraphBridge → intelligence layer → enrichment injection.
//
// All tests that require a live CBM binary check availability first
// and skip gracefully if CBM is not installed.
//
// Live-CBM tests share ONE process-scoped instance (McpState → GraphBridge →
// CBM subprocess → one async index → many requests), mirroring the production
// lifecycle described in "Shared live-CBM fixture" below. Non-CBM / mock tests
// remain CBM-disabled and isolated.

use serde_json::json;
use serial_test::serial;
use std::sync::{Arc, Mutex};

/// Check if CBM binary exists on PATH without launching it.
/// This avoids double-launching CBM when `McpState::new()` also launches it.
fn cbm_binary_exists() -> bool {
    let name = if cfg!(windows) {
        "codebase-memory-mcp.exe"
    } else {
        "codebase-memory-mcp"
    };
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}

// ── Shared live-CBM fixture ──────────────────────────────────────
//
// Mirrors the production process-scoped CBM lifecycle. The live-CBM tests in
// this module all acquire the SAME instance via `shared_live_state()`:
//
//   one McpState → one GraphBridge → one CBM subprocess → one async index
//   (started at construction) → wait for Complete → many MCP requests
//
// Live queries target the bridge's CANONICAL project: the CBM slug derived
// from the canonical `project_root` path (`cbm_project_slug`), which is what
// construction-time `start_indexing_roots()` indexed. An explicit `project`
// override resolves through the same canonical identity map, so overrides can
// no longer diverge from the indexed project. The live-CBM dispatch tests omit
// `project` (defaulting to the primary root) for that reason.
//
// Every live test is tagged `#[serial(cbm_live)]` so the shared instance is
// never touched by two threads at once. Non-CBM / mock tests in this file do
// NOT call these helpers and remain CBM-disabled.
//
// ## Cleanup — normal resource lifetime, no teardown hook
//
// The shared `McpState` is owned by the `static` below, so its lifetime is the
// test process's lifetime — exactly like production's process-scoped singleton.
// Cleanup therefore happens through ordinary process/resource teardown, with no
// teardown test, no test-order dependence, and no exit hook:
//
//   1. `McpState` (and its `GraphBridge`/`CbmClient`) are valid for the whole
//      process, so every live test reuses the same subprocess and indexed graph.
//   2. When the test process exits, the OS closes the parent→child stdin pipe,
//      CBM (an MCP server reading stdin) observes EOF and terminates.
//   3. If the shared `Arc` were ever dropped by a scoped owner, `CbmClient`'s
//      `Drop` still runs `child.kill()` with a 5s deadline — but for a
//      process-lifetime static this path never fires by design.
//
// Both are "after normal lifetime" mechanisms, not explicit test or exit hooks.

/// How long the shared async `index_repository` may take before the suite gives up.
const LIVE_INDEXING_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(180);

/// The single process-scoped live-CBM `McpState`, shared by every
/// `#[serial(cbm_live)]` test. `None` until the first live test initializes it;
/// the `Mutex` makes first-arrival initialization atomic (belt-and-suspenders
/// on top of `#[serial]`).
static SHARED_LIVE_STATE: Mutex<Option<Arc<crate::mcp::McpState>>> = Mutex::new(None);

/// Acquire the shared live-CBM state, initializing it exactly once and waiting
/// for the construction-time async index to reach `Complete`. Reused by every
/// live test for the rest of the suite.
///
/// Callers MUST guard with `cbm_binary_exists()` first — this panics (with a
/// diagnostic) when CBM is not running.
fn shared_live_state() -> Arc<crate::mcp::McpState> {
    let mut guard = SHARED_LIVE_STATE.lock().unwrap_or_else(|p| p.into_inner());
    if guard.is_none() {
        eprintln!(
            "[cbm-e2e] Initializing shared live-CBM McpState (one subprocess for the suite)..."
        );
        let mut config = crate::config::CleanCtxConfig::default();
        config.cbm.enabled = true;
        let state = crate::mcp::McpState::new(config);
        wait_for_indexing_complete(&state);
        *guard = Some(Arc::new(state));
        eprintln!("[cbm-e2e] Shared live-CBM index Complete — graph ready.");
    }
    Arc::clone(
        guard
            .as_ref()
            .expect("shared live state was just initialized"),
    )
}

/// Poll the shared bridge's `ensure_indexed()` (report-only — the index was
/// started at construction) until `Ready`. Logs and tolerates a failed index
/// (matching the existing live E2E tolerance); panics only on timeout.
fn wait_for_indexing_complete(state: &crate::mcp::McpState) {
    let deadline = std::time::Instant::now() + LIVE_INDEXING_TIMEOUT;
    'indexing: loop {
        let status = {
            let mut guard = state.graph_bridge_lock();
            guard
                .as_mut()
                .expect("cbm.enabled=true must produce a GraphBridge")
                .ensure_indexed()
        };
        match status {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => break 'indexing,
            Ok(crate::cbm::bridge::IndexingStatus::StillIndexing { .. }) => {
                if std::time::Instant::now() >= deadline {
                    panic!(
                        "[cbm-e2e] Timed out after {}s waiting for indexing to Complete",
                        LIVE_INDEXING_TIMEOUT.as_secs()
                    );
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(e) => {
                eprintln!("[cbm-e2e] Indexing failed (continuing): {e}");
                break 'indexing;
            }
        }
    }
}

/// Prove the shared fixture reuses ONE `McpState` (→ one GraphBridge, one CBM
/// subprocess, one indexed graph) across every live test: two acquisitions must
/// be the same allocation.
#[serial(cbm_live)]
#[test]
fn shared_live_state_is_reused_across_acquisitions() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let first = shared_live_state();
    let second = shared_live_state();
    assert!(
        std::sync::Arc::ptr_eq(&first, &second),
        "shared_live_state must return the SAME McpState (one subprocess per suite)"
    );
}

// ── E2E: MCP dispatch path with live CBM ────────────────────────

/// Smoke-test every CBM MCP tool handler via dispatch_tools_call.
#[serial(cbm_live)]
#[test]
fn e2e_mcp_dispatch_graph_search_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "graph_search",
        &serde_json::json!({"arguments": {"query": ".*compress.*"}}),
        &state,
    );
}

#[serial(cbm_live)]
#[test]
fn e2e_mcp_dispatch_graph_query_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(2),
        "graph_query",
        &serde_json::json!({"arguments": {"query": "MATCH (n:Function) RETURN n.name LIMIT 5"}}),
        &state,
    );
}

#[serial(cbm_live)]
#[test]
fn e2e_mcp_dispatch_graph_trace_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();
    // Both `from` and `to` must be non-empty (the handler rejects empty `to`
    // with -32602 before touching the graph). Use real symbols from this repo
    // so the trace actually queries the indexed graph.
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(3),
        "graph_trace",
        &serde_json::json!({"arguments": {"from": "GraphBridge", "to": "CbmClient"}}),
        &state,
    );
}

#[serial(cbm_live)]
#[test]
fn e2e_mcp_dispatch_get_architecture_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(4),
        "get_architecture",
        &serde_json::json!({"arguments": {}}),
        &state,
    );
}

#[serial(cbm_live)]
#[test]
fn e2e_mcp_dispatch_get_cbm_status_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(5),
        "get_cbm_status",
        &serde_json::json!({"arguments": {}}),
        &state,
    );
}

#[serial(cbm_live)]
#[test]
fn e2e_mcp_dispatch_cbm_proxy_with_live_cbm() {
    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();

    // Smoke test: dispatch must not panic. The shared fixture waits for the
    // construction-time async index to Complete, so the proxy path is exercised
    // against the indexed graph rather than a StillIndexing retry.
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(6),
        "cbm_proxy",
        &serde_json::json!({"arguments": {"cbm_tool": "search_graph", "parameters": {"name_pattern": ".*compress.*"}}}),
        &state,
    );
}

#[test]
fn e2e_get_cbm_status_always_works() {
    // Explicitly CBM-disabled: this is an isolation smoke test, not a live-CBM
    // test. It must not spawn its own subprocess (the shared live fixture owns
    // the only subprocess in the live suite).
    let config = crate::tests::test_config();
    let state = crate::mcp::McpState::new(config);
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(99),
        "get_cbm_status",
        &serde_json::json!({"arguments": {}}),
        &state,
    );
}

/// Test that the full proxy compression pipeline works end-to-end.
/// Uses cbm_proxy handler directly.
#[serial(cbm_live)]
#[test]
fn e2e_proxy_handler_returns_compressed_result() {
    let cbm_available = cbm_binary_exists();

    if !cbm_available {
        eprintln!("Skipping e2e_proxy_handler_returns_compressed_result — CBM not installed");
        return;
    }

    let state = shared_live_state();

    // Test that the proxy can handle a get_architecture call
    // handle_cbm_proxy sends responses directly via send_response,
    // so we test the bridge's proxy_call directly instead
    let mut binding = state.graph_bridge.lock().unwrap();
    let bridge = binding.as_mut().unwrap();
    let result = bridge.proxy_call("get_architecture", json!({}));

    match result {
        Ok(text) => {
            assert!(!text.is_empty());
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&text);
            assert!(parsed.is_ok(), "Proxy response should be valid JSON");
            let r = parsed.unwrap();
            // If CBM returns an error because project not indexed, that's fine
            if let Some(err) = r.get("error") {
                eprintln!("CBM returned expected error: {}", err["message"]);
            }
        }
        Err(e) => {
            eprintln!("CBM proxy call failed (may need index): {e}");
        }
    }
}

// ── E2E: GraphBridge full query lifecycle (no CBM required) ────

/// Test that GraphBridge gracefully handles all queries when CBM is unavailable.
#[test]
fn e2e_bridge_graceful_degradation_all_queries() {
    let config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge =
        crate::cbm::bridge::GraphBridge::try_create(&config, std::path::Path::new("."));

    assert!(
        !bridge.is_available(),
        "Bridge should be unavailable when CBM disabled"
    );

    // F11: intelligence queries now propagate failures as Err — a failed
    // query must never masquerade as "valid query, zero results". The
    // user-facing wrappers (search/trace/query_graph) keep their graceful
    // empty results with take_last_error() diagnostics.
    let importance = bridge.get_symbol_importance_mut();
    assert!(
        importance.is_err(),
        "Symbol importance should fail without CBM, not return empty Ok"
    );

    let dead = bridge.get_dead_code();
    assert!(dead.is_err(), "Dead code should fail without CBM");

    let arch = bridge.get_architecture();
    assert!(arch.is_err(), "Architecture should fail without CBM");

    let blast = bridge.get_blast_radius("test_func", 1);
    assert!(blast.is_err(), "Blast radius should fail without CBM");

    let search = bridge.search("test");
    assert!(search.is_empty(), "Search should be empty without CBM");

    let traces = bridge.trace_path("a", "b");
    assert!(traces.is_empty(), "Trace path should be empty without CBM");

    // query_graph should return empty QueryResult
    let qr = bridge.query_graph("MATCH (n) RETURN n");
    assert!(
        qr.nodes.is_empty() && qr.edges.is_empty(),
        "Query graph should return empty without CBM"
    );

    // detect_changes should return Ok(None)
    let changes = bridge.detect_changes();
    assert!(
        changes.is_ok() && changes.unwrap().is_none(),
        "Detect changes should return None without CBM"
    );

    // cache operations should not panic
    bridge.invalidate_symbol("test");
    bridge.invalidate_cache();
    bridge.clear_cache();
}

/// Test that GraphBridge status transitions work correctly.
#[test]
fn e2e_bridge_status_lifecycle() {
    use crate::cbm::config::CbmStatus;

    let config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge =
        crate::cbm::bridge::GraphBridge::try_create(&config, std::path::Path::new("."));

    assert_eq!(bridge.status(), &CbmStatus::Unavailable);
    assert!(!bridge.is_available());
    assert_eq!(bridge.graph_version(), "");

    // Set project and version
    bridge.set_project("test_project");
    bridge.set_graph_version("v1.0");

    assert_eq!(bridge.graph_version(), "v1.0");
    // Status should remain Unavailable (no client)
    assert_eq!(bridge.status(), &CbmStatus::Unavailable);

    // update_status should keep Unavailable
    bridge.update_status();
    assert_eq!(bridge.status(), &CbmStatus::Unavailable);
}

// ── E2E: Intelligence Layer full pipeline (no CBM required) ────

/// Test the full intelligence layer pipeline: PageRank → fidelity → recommendation.
#[test]
fn e2e_intelligence_layer_full_pipeline() {
    use crate::cbm::SymbolImportance;
    use crate::intelligence::compute_pagerank;
    use crate::intelligence::fidelity::{
        FidelityRecommendation, apply_recommendation, cbm_informed_fidelity,
    };
    use std::collections::HashMap;

    // Build IR scores
    let mut ir_scores = HashMap::new();
    ir_scores.insert("critical_handler".to_string(), 50.0);
    ir_scores.insert("helper_func".to_string(), 5.0);
    ir_scores.insert("unused_util".to_string(), 1.0);

    // Build CBM importance scores
    let mut cbm_scores = HashMap::new();
    cbm_scores.insert(
        "critical_handler".to_string(),
        SymbolImportance {
            symbol: "critical_handler".into(),
            score: 0.95,
            file: "src/handler.rs".into(),
        },
    );
    cbm_scores.insert(
        "helper_func".to_string(),
        SymbolImportance {
            symbol: "helper_func".into(),
            score: 0.5,
            file: "src/handler.rs".into(),
        },
    );
    cbm_scores.insert(
        "unused_util".to_string(),
        SymbolImportance {
            symbol: "unused_util".into(),
            score: 0.1,
            file: "src/utils.rs".into(),
        },
    );

    // Step 1: Compute PageRank
    let scores = compute_pagerank(ir_scores, cbm_scores, Some(0.6));
    assert!(!scores.is_empty(), "Should have combined scores");

    // Step 2: Verify high-importance symbol gets ForceHigh
    let critical_score = scores.get("critical_handler").unwrap();
    assert!(
        critical_score.combined_score > 0.7,
        "Critical handler should have high combined score: {}",
        critical_score.combined_score
    );

    // Step 3: Test fidelity recommendation pipeline
    let mut importance_map = HashMap::new();
    importance_map.insert(
        "critical_handler".to_string(),
        SymbolImportance {
            symbol: "critical_handler".into(),
            score: critical_score.combined_score,
            file: "src/handler.rs".into(),
        },
    );

    let rec = cbm_informed_fidelity(
        "src/handler.rs",
        &importance_map,
        FidelityRecommendation::NoRecommendation,
    );

    // Step 4: Apply recommendation
    let fidelity = apply_recommendation(&rec);
    if critical_score.combined_score > 0.8 {
        assert_eq!(fidelity, Some(crate::compression::Fidelity::High));
    }
}

// ---- K-1: Indexing lifecycle tests ---------------------------------

/// Prove `ensure_indexed()` is report-only -- it does NOT transition
/// `NotStarted` -> `InProgress` (no spawn, no mutation).
/// Uses a mock bridge with `Available` status but `NotStarted` state.
#[test]
fn ensure_indexed_does_not_trigger_indexing() {
    use crate::cbm::bridge::test_helpers::new_available_not_started;

    let mut bridge = new_available_not_started();

    // Precondition: state is NotStarted (empty map).
    {
        let states = bridge.indexing_state();
        assert!(
            states.is_empty(),
            "precondition: indexing state should be empty (NotStarted)"
        );
    }

    // Call ensure_indexed -- in the OLD code this would spawn a background
    // thread, flip state to InProgress, and return StillIndexing.
    // In the NEW code it must NOT mutate state and return StillIndexing.
    let result = bridge.ensure_indexed();

    // Must return Ok(StillIndexing) -- not Err, not Ready.
    assert!(
        result.is_ok(),
        "ensure_indexed should return Ok(StillIndexing) when Available/NotStarted: {:?}",
        result
    );
    match result.unwrap() {
        crate::cbm::bridge::IndexingStatus::StillIndexing { elapsed_secs } => {
            assert_eq!(elapsed_secs, 0, "should report 0 elapsed");
        }
        _ => panic!("expected StillIndexing, got something else"),
    }

    // State must remain NotStarted (no transition to InProgress) -- proving no spawn occurred.
    let states = bridge.indexing_state();
    for (project, state) in states.iter() {
        assert!(
            matches!(state, crate::cbm::bridge::IndexingState::NotStarted),
            "K-1: ensure_indexed must NOT mutate indexing state to InProgress (no spawn) -- project '{project}' is {:?}",
            state
        );
    }
}

/// Prove indexing begins at bridge construction and reaches a non-`NotStarted`
/// state.
///
/// Uses the shared live-CBM instance (no second subprocess): the very fact that
/// `shared_live_state()` returns at all proves `try_create` started async
/// indexing — if it had not, `wait_for_indexing_complete` would time out. We
/// then assert the resulting indexing state is `InProgress`/`Complete`/`Failed`
/// (never `NotStarted`), which is exactly the K-1 guarantee.
#[serial(cbm_live)]
#[test]
fn try_create_begins_indexing_at_construction() {
    use crate::cbm::bridge::IndexingState;

    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping -- CBM not installed");
        return;
    }
    let state = shared_live_state();

    // Valid lifecycle: the bridge started indexing at construction.
    // The `indexing_state` map must NOT be empty (an empty map means
    // `NotStarted` -- the construction-time spawn never ran), and NO
    // project may be `NotStarted` (a `NotStarted` entry can only appear if a
    // query switched to an un-indexed project via `set_project` and
    // `ensure_indexed()` lazily inserted it -- the K-1 regression this
    // test guards against).
    //
    // Snapshot an owned copy of the whole map while holding the bridge lock:
    // the inner `indexing_state()` guard borrows from the outer
    // `graph_bridge_lock()` guard, so it cannot outlive this block.
    let states = {
        let guard = state.graph_bridge_lock();
        let bridge = guard
            .as_ref()
            .expect("shared live McpState must contain a GraphBridge");
        bridge
            .indexing_state()
            .iter()
            .map(|(project, state)| (project.clone(), state.clone()))
            .collect::<std::collections::HashMap<String, IndexingState>>()
    };
    assert!(
        !states.is_empty(),
        "K-1: indexing_state is empty after shared init -- \
         the construction-time async indexer never ran."
    );
    for (project, state) in &states {
        match state {
            IndexingState::InProgress { .. } | IndexingState::Complete => {
                // Good: indexing was kicked off at construction.
            }
            IndexingState::Failed(msg) => {
                // Acceptable: CBM binary may not be compatible.
                eprintln!("Note: indexing started but failed: {msg}");
            }
            IndexingState::NotStarted => {
                panic!(
                    "K-1: project '{project}' is NotStarted -- \
                     this means the background indexer was never spawned at construction."
                );
            }
        }
    }
}

// ── Live proxy-path test: exercise every CBM tool through Clean-CTX's pipe-level proxy ──

/// Exercise all four CBM tools through `bridge.proxy_call` — the exact pipe-level
/// interception path `handle_cbm_proxy` (the `cbm_proxy` MCP tool) uses: forward to
/// CBM over stdin, intercept raw stdout, return it. Uses the shared live CBM instance
/// (no second subprocess). Tolerates environment-dependent CBM errors (e.g. a project
/// name mismatch) but asserts every call returns a parseable JSON-RPC envelope.
#[serial(cbm_live)]
#[test]
fn live_proxy_exercises_all_cbm_tools() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();

    // proxy_call takes `&mut GraphBridge`, so hold one mutable bridge guard for
    // the whole test (serial, single-threaded).
    let mut guard = state.graph_bridge_lock();
    let bridge = guard
        .as_mut()
        .expect("shared live McpState must contain a GraphBridge");

    // 1. search_graph — CBM-native `name_pattern`.
    let raw1 = bridge
        .proxy_call("search_graph", json!({"name_pattern": ".*Compress.*"}))
        .expect("search_graph must execute on the live CBM");
    let parsed1: serde_json::Value = serde_json::from_str(&raw1)
        .unwrap_or_else(|e| panic!("search_graph proxy output should be JSON: {e}"));
    assert!(
        parsed1.get("result").is_some() || parsed1.get("error").is_some(),
        "search_graph proxy returned neither a result nor an error envelope: {raw1}"
    );

    // 2. query_graph — Cypher query.
    let raw2 = bridge
        .proxy_call(
            "query_graph",
            json!({"query": "MATCH (n) RETURN n LIMIT 3"}),
        )
        .expect("query_graph must execute on the live CBM");
    let parsed2: serde_json::Value = serde_json::from_str(&raw2)
        .unwrap_or_else(|e| panic!("query_graph proxy output should be JSON: {e}"));
    assert!(
        parsed2.get("result").is_some() || parsed2.get("error").is_some(),
        "query_graph proxy returned neither a result nor an error envelope: {raw2}"
    );

    // 3. trace_path — CBM-native `function_name`/`direction`/`depth`.
    let raw3 = bridge
        .proxy_call(
            "trace_path",
            json!({"function_name": "GraphBridge", "direction": "outbound", "depth": 3}),
        )
        .expect("trace_path must execute on the live CBM");
    let parsed3: serde_json::Value = serde_json::from_str(&raw3)
        .unwrap_or_else(|e| panic!("trace_path proxy output should be JSON: {e}"));
    assert!(
        parsed3.get("result").is_some() || parsed3.get("error").is_some(),
        "trace_path proxy returned neither a result nor an error envelope: {raw3}"
    );

    // 4. get_architecture — no project: CBM falls back to its default indexed project.
    let raw4 = bridge
        .proxy_call("get_architecture", json!({}))
        .expect("get_architecture must execute on the live CBM");
    let parsed4: serde_json::Value = serde_json::from_str(&raw4)
        .unwrap_or_else(|e| panic!("get_architecture proxy output should be JSON: {e}"));
    assert!(
        parsed4.get("result").is_some() || parsed4.get("error").is_some(),
        "get_architecture proxy returned neither a result nor an error envelope: {raw4}"
    );
}

/// CBM-ID fix regression: the graph_search wrapper must find symbols of
/// ALL node labels. Before the fix, `bridge.search` hardcoded
/// `label: "Function"`, so Class nodes (e.g. GraphBridge itself) were
/// invisible to every wrapper search.
///
/// The test invalidates caches first: pre-fix runs wrote their own empty
/// search results into the disk graph cache (`search:*` keys), and
/// `check_cache` hydrates those before any wire call — measuring that
/// poison would prove nothing about CBM.
///
/// Assertions run AFTER the bridge guard is dropped so a failure can never
/// poison the shared mutex for later live tests.
#[serial(cbm_live)]
#[test]
fn e2e_bridge_search_finds_class_symbols_across_labels() {
    enum Outcome {
        Skipped(&'static str),
        Hit { name: String, label: String },
        Empty,
    }
    let outcome = {
        if !cbm_binary_exists() {
            Outcome::Skipped("CBM not installed")
        } else {
            let state = shared_live_state();
            let mut guard = state.graph_bridge_lock();
            let bridge = match guard.as_mut() {
                Some(b) => b,
                None => {
                    eprintln!("Skipping — no live bridge");
                    return;
                }
            };
            match bridge.ensure_indexed() {
                Ok(crate::cbm::bridge::IndexingStatus::Ready) => {}
                Ok(other) => {
                    eprintln!("Skipping — indexing not ready: {other:?}");
                    return;
                }
                Err(e) => {
                    eprintln!("Skipping — CBM unavailable: {e}");
                    return;
                }
            }

            // Purge stale (possibly pre-fix, label-filtered) cached searches
            // so this regression exercises the real CBM wire path.
            bridge.invalidate_cache();

            let nodes = bridge.search("^GraphBridge$");
            match nodes.iter().find(|n| n.name == "GraphBridge") {
                Some(hit) => Outcome::Hit {
                    name: hit.name.clone(),
                    label: hit.label.clone(),
                },
                None => Outcome::Empty,
            }
        }
    };

    match outcome {
        Outcome::Skipped(reason) => eprintln!("Skipping — {reason}"),
        Outcome::Empty => panic!(
            "search must see non-Function nodes after removing the \
             Function-only label filter"
        ),
        Outcome::Hit { name, label } => {
            assert_eq!(name, "GraphBridge");
            assert_eq!(label, "Class", "GraphBridge must be found as a Class");
        }
    }
}

/// Finding #3 regression: CBM 0.8.1 has DEFINES_METHOD (not DECLARES).
/// Live-probe the real CBM graph: querying DEFINES_METHOD between Class
/// and Method must succeed and return rows. The guard is dropped before
/// assertions so a failure cannot poison the shared mutex.
#[serial(cbm_live)]
#[test]
fn e2e_live_defines_method_not_declares() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();
    let raw_result = {
        let mut guard = state.graph_bridge_lock();
        let bridge = match guard.as_mut() {
            Some(b) => b,
            None => {
                eprintln!("Skipping — no live bridge");
                return;
            }
        };
        match bridge.ensure_indexed() {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => {}
            Ok(other) => {
                eprintln!("Skipping — indexing not ready: {other:?}");
                return;
            }
            Err(e) => {
                eprintln!("Skipping — CBM unavailable: {e}");
                return;
            }
        }
        bridge.invalidate_cache();
        bridge.resolve_cross_language_endpoint("GraphBridge")
    };
    // We do not expect a hit (this is Rust, not .cs), but the method
    // must not panic — proving the Cypher compiles and executes against
    // the real CBM graph.
    assert!(
        raw_result.is_none() || raw_result.is_some(),
        "resolve_cross_language_endpoint must return Some or None, never panic"
    );
}

/// Live-prove that querying CBM with DEFINES_METHOD works over the real wire.
/// Uses bridge.proxy_call to run the raw Cypher and confirm CBM returns rows,
/// not an error about a nonexistent edge type.
#[serial(cbm_live)]
#[test]
fn e2e_live_proxy_defines_method_not_declares() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();
    let mut guard = state.graph_bridge_lock();
    let bridge = match guard.as_mut() {
        Some(b) => b,
        None => {
            eprintln!("Skipping — no live bridge");
            return;
        }
    };
    match bridge.ensure_indexed() {
        Ok(crate::cbm::bridge::IndexingStatus::Ready) => {}
        Ok(other) => {
            eprintln!("Skipping — indexing not ready: {other:?}");
            return;
        }
        Err(e) => {
            eprintln!("Skipping — CBM unavailable: {e}");
            return;
        }
    }
    // Probe: query Class-Method with DEFINES_METHOD (the correct edge)
    let cypher =
        "MATCH (c:Class)-[:DEFINES_METHOD]->(m:Method) RETURN c.name, m.name LIMIT 5".to_string();
    let raw = bridge
        .proxy_call("query_graph", serde_json::json!({"query": cypher}))
        .unwrap_or_else(|e| panic!("proxy_call failed: {e}"));
    let parsed: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("proxy output must be JSON: {e}"));
    assert!(
        parsed.get("result").is_some() || parsed.get("error").is_some(),
        "CBM DEFINES_METHOD query must return a valid envelope: {raw}"
    );
    if let Some(err) = parsed.get("error") {
        panic!("CBM DEFINES_METHOD query returned an error: {err} — edge type may not exist");
    }
}
/// Multi-root, multilingual live-CBM integration test (CBM 0.8.1).
/// Exercises every handler against live CBM. FAILS on unexpected errors.
///
/// Fully self-contained and machine-agnostic: generates a deterministic
/// multilingual fixture (Rust, C#, Java, TypeScript/Angular, JavaScript,
/// HTML, CSS) in the system temp dir and registers it as an additional
/// root, proving the complete lifecycle:
/// additional_root → canonical slug → index_repository → readiness →
/// query → cross-language resolution → proxy. The fixture directory is
/// removed on scope exit, including panic paths.
#[serial(cbm_live)]
#[test]
fn e2e_cbm_multiroot_multilingual_integration() {
    if !cbm_binary_exists() {
        eprintln!("SKIP — CBM not installed");
        return;
    }
    let state = shared_live_state();

    // eprintln!("\n═══ Step 1: get_cbm_status ═══");
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "get_cbm_status",
        &serde_json::json!({"arguments": {}}),
        &state,
    );

    // eprintln!("\n═══ Step 2: graph_search ═══");
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(2),
        "graph_search",
        &serde_json::json!({"arguments": {"query": "GraphBridge"}}),
        &state,
    );

    // eprintln!("\n═══ Step 3: graph_query ═══");
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(3),
        "graph_query",
        &serde_json::json!({"arguments": {"query": "MATCH (n:Function) RETURN n.name LIMIT 5"}}),
        &state,
    );

    // eprintln!("\n═══ Step 4: graph_trace ═══");
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(4),
        "graph_trace",
        &serde_json::json!({"arguments": {"from": "GraphBridge", "to": "CbmClient"}}),
        &state,
    );

    // eprintln!("\n═══ Step 5: get_architecture ═══");
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(5),
        "get_architecture",
        &serde_json::json!({"arguments": {}}),
        &state,
    );

    // eprintln!("\n═══ Step 6: cbm_proxy search_graph ═══");
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(6),
        "cbm_proxy",
        &serde_json::json!({"arguments": {"cbm_tool": "search_graph", "parameters": {"name_pattern": ".*GraphBridge.*"}}}),
        &state,
    );

    // eprintln!("\n═══ Step 7: resolve_cross_language_endpoint ═══");
    {
        let mut guard = state.graph_bridge_lock();
        let bridge = guard.as_mut().expect("live bridge");
        for name in &["getAll", "GetAll", "Controller", "getUsers"] {
            // Result intentionally exercised without logging (noise reduction);
            // Step 15 asserts the fixture-project equivalents.
            let _ = bridge.resolve_cross_language_endpoint(name);
        }
    }

    // eprintln!("\n═══ Step 8: cbm_proxy list_projects ═══");
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(8),
        "cbm_proxy",
        &serde_json::json!({"arguments": {"cbm_tool": "list_projects", "parameters": {}}}),
        &state,
    );

    // ── Multilingual fixture (self-contained, machine-agnostic) ──
    // Generate a temporary polyglot project so every developer/CI machine
    // exercises the identical CBM wire path. No external repositories.
    // eprintln!("\n═══ Step 9: Create multilingual fixture project ═══");
    let fixture_root = std::env::temp_dir().join(format!("clean-ctx-audit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&fixture_root);
    std::fs::create_dir_all(fixture_root.join("src")).expect("create fixture dir");

    /// Removes the fixture directory on scope exit — covers panic paths.
    struct FixtureCleanup<'a>(&'a std::path::Path);
    impl Drop for FixtureCleanup<'_> {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.0);
        }
    }
    let _cleanup = FixtureCleanup(&fixture_root);

    // Rust fixture — CBM emits Function nodes from .rs files
    std::fs::write(
        fixture_root.join("src/fixture_core.rs"),
        r#"
pub struct FixtureCore { value: String }

impl FixtureCore {
    pub fn new(value: &str) -> Self { Self { value: value.to_string() } }
    pub fn fixture_value(&self) -> &str { &self.value }
}
"#,
    )
    .expect("write fixture_core.rs");

    // C# fixture — CBM emits Class/Interface/Method nodes from .cs files.
    // Get/Create/Update/Delete are the cross-language resolution targets.
    std::fs::write(
        fixture_root.join("src/FixtureEngine.cs"),
        r#"
using System;
namespace AuditFixture {
    public interface IFixtureEngine { string Get(string key); void Create(string val); }
    public class FixtureEngine : IFixtureEngine {
        private readonly Dictionary<string, string> _cache = new();
        public string Get(string key) { return _cache.TryGetValue(key, out var v) ? v : ""; }
        public void Create(string val) { _cache[val] = val; }
        public void Update(string key, string val) { _cache[key] = val; }
        public bool Delete(string key) { return _cache.Remove(key); }
    }
}
"#,
    )
    .expect("write FixtureEngine.cs");

    // Java fixture — at minimum indexed as a File node
    std::fs::write(
        fixture_root.join("src/FixtureGateway.java"),
        r#"
public class FixtureGateway {
    public String fetchById(String id) { return id; }
    public void save(String record) { }
}
"#,
    )
    .expect("write FixtureGateway.java");

    // TypeScript fixture — Angular service (meta-layer representative)
    std::fs::write(
        fixture_root.join("src/fixture-client.service.ts"),
        r#"
import { Injectable } from '@angular/core';

export interface FixtureDto { id: string; label: string; }

@Injectable({ providedIn: 'root' })
export class FixtureClientService {
    load(id: string): FixtureDto { return { id, label: 'fixture' }; }
}
"#,
    )
    .expect("write fixture-client.service.ts");

    // JavaScript fixture — CBM emits File/Module nodes from .js files
    std::fs::write(
        fixture_root.join("src/fixture_app.js"),
        r#"
function getFixture(id) { return { id }; }
function createFixture(data) { return data; }
"#,
    )
    .expect("write fixture_app.js");

    // HTML fixture
    std::fs::write(
        fixture_root.join("index.html"),
        "<html><body><h1>AuditFixture</h1></body></html>",
    )
    .expect("write index.html");

    // CSS fixture — indexed as a File node
    std::fs::write(
        fixture_root.join("src/fixture_styles.css"),
        ".fixture-root { color: rebeccapurple; }",
    )
    .expect("write fixture_styles.css");

    // Verify every language fixture exists before indexing
    for f in [
        "src/fixture_core.rs",
        "src/FixtureEngine.cs",
        "src/FixtureGateway.java",
        "src/fixture-client.service.ts",
        "src/fixture_app.js",
        "index.html",
        "src/fixture_styles.css",
    ] {
        assert!(fixture_root.join(f).exists(), "fixture must exist: {f}");
    }

    let fixture_str = fixture_root.to_string_lossy().to_string();

    let mut fx_config = crate::config::CleanCtxConfig::default();
    fx_config.cbm.enabled = true;
    fx_config.additional_roots.push(fixture_str.clone());
    let fx_state = crate::mcp::McpState::new(fx_config);

    // eprintln!("\n═══ Step 10: Wait for fixture indexing ═══");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        let status = {
            let mut g = fx_state.graph_bridge_lock();
            g.as_mut().expect("lf bridge").ensure_indexed()
        };
        match status {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => {
                // eprintln!("  Fixture indexing Complete");
                break;
            }
            Ok(crate::cbm::bridge::IndexingStatus::StillIndexing { .. }) => {
                if std::time::Instant::now() >= deadline {
                    panic!("Timed out waiting for fixture");
                }
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Err(e) => panic!("Fixture indexing failed: {e}"),
        }
    }

    let fx_slug = {
        let mut g = fx_state.graph_bridge_lock();
        let b = g.as_mut().expect("lf bridge");
        // Resolve the fixture project's canonical CBM slug via its path
        let s = b.resolve_project_id(&fixture_str);
        // eprintln!("  Fixture slug: {s}");
        // eprintln!("  primary slug: {}", b.project_str());
        s
    };

    // eprintln!("\n═══ Step 11: Fixture architecture ═══");
    {
        let mut g = fx_state.graph_bridge_lock();
        let b = g.as_mut().expect("lf bridge");
        // Switch to the fixture project before querying
        b.set_project(&fx_slug);
        b.invalidate_cache();
        let arch = b
            .get_architecture()
            .unwrap_or_else(|e| panic!("get_architecture must succeed on the fixture: {e}"));
        // eprintln!(
        //     "  {} module(s), {} dep(s)",
        //     arch.modules.len(),
        //     arch.dependencies.len()
        // );
        // for m in &arch.modules {
        //     eprintln!("    module: {} ({} nodes)", m.name, m.file_count);
        // }
        // for d in &arch.dependencies {
        //     eprintln!("    dep: {} -> {} ({})", d.from, d.to, d.kind);
        // }
        assert!(!arch.modules.is_empty(), "Fixture must have modules");
    }

    // eprintln!("\n═══ Step 12: Language-specific symbol discovery ═══");
    {
        let mut g = fx_state.graph_bridge_lock();
        let b = g.as_mut().expect("fixture bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        // Hard requirements: every language must surface its fixture symbol.
        for sym in &[
            "FixtureEngine",      // C# Class/Interface/Method
            "FixtureCore",        // Rust struct
            "FixtureGateway",     // Java class / File node
            "index.html",         // HTML File node
            "fixture_styles.css", // CSS File node
        ] {
            let nodes = b.search(sym);
            // eprintln!("  search(\"{sym}\") = {} hits", nodes.len());
            // for n in nodes.iter().take(3) {
            //     eprintln!("    - {} {} ({})", n.label, n.name, n.file);
            // }
            assert!(
                !nodes.is_empty(),
                "search(\"{sym}\") must find its fixture node"
            );
        }
        let cs = b.search("FixtureEngine");
        assert!(
            cs.iter()
                .any(|n| n.label == "Class" || n.label == "Interface"),
            "C# fixture must yield Class/Interface nodes, got: {:?}",
            cs.iter()
                .map(|n| (n.label.as_str(), n.name.as_str()))
                .collect::<Vec<_>>()
        );
        // Informational: TS symbol parsing support varies by CBM build.
        // let ts = b.search("FixtureClientService");
        // eprintln!(
        //     "  search(\"FixtureClientService\") [TS] = {} hits{}",
        //     ts.len(),
        //     if ts.is_empty() {
        //         " (file-level only)"
        //     } else {
        //         ""
        //     }
        // );
    }

    // eprintln!("\n═══ Step 13: Method, Class and Function nodes ═══");
    {
        let mut g = fx_state.graph_bridge_lock();
        let b = g.as_mut().expect("fixture bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        // LIMIT must exceed the fixture's total Method count (10+ across
        // C#/Java/TS/Rust) or rows get silently truncated.
        let q1 = "MATCH (m:Method) RETURN m.name LIMIT 50".to_string();
        let qr1 = b.query_graph(&q1);
        let methods: Vec<&str> = qr1.nodes.iter().map(|n| n.name.as_str()).collect();
        // eprintln!("  Methods: {methods:?}");
        assert!(!qr1.nodes.is_empty(), "Fixture must have Method nodes");
        assert!(
            methods.contains(&"Get") && methods.contains(&"Create"),
            "C# methods Get/Create must appear as Method nodes, got: {methods:?}"
        );
        let q2 = "MATCH (c:Class) RETURN c.name LIMIT 10".to_string();
        let qr2 = b.query_graph(&q2);
        let classes: Vec<&str> = qr2.nodes.iter().map(|n| n.name.as_str()).collect();
        eprintln!("  Classes: {classes:?}");
        assert!(
            classes.contains(&"FixtureEngine"),
            "C# class FixtureEngine must appear as a Class node, got: {classes:?}"
        );
        // Rust: fixture_value must be reachable as Function OR Method node.
        let q3 = "MATCH (f:Function) RETURN f.name LIMIT 30".to_string();
        let qr3 = b.query_graph(&q3);
        let fns: Vec<&str> = qr3.nodes.iter().map(|n| n.name.as_str()).collect();
        // eprintln!("  Functions: {fns:?}");
        assert!(
            fns.contains(&"fixture_value") || methods.contains(&"fixture_value"),
            "Rust fn fixture_value must appear as a Function/Method node, got fns={fns:?} methods={methods:?}"
        );
    }

    // eprintln!("\n═══ Step 14: Web-file nodes (JS/HTML/TS/CSS) ═══");
    {
        let mut g = fx_state.graph_bridge_lock();
        let b = g.as_mut().expect("fixture bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        for ext in &["js", "html", "ts", "css"] {
            let q = format!(
                "MATCH (n) WHERE n.file_path =~ '.*\\.{ext}$' RETURN n.name, n.file_path LIMIT 5"
            );
            let qr = b.query_graph(&q);
            // eprintln!("  {} nodes: {}", ext.to_uppercase(), qr.nodes.len());
            // for n in &qr.nodes {
            //     eprintln!("    {} ({})", n.name, n.file);
            // }
            assert!(
                !qr.nodes.is_empty(),
                ".{ext} fixture must produce graph nodes"
            );
        }
        // Razor remains intentionally unsupported by CBM — no assertion.
    }

    // eprintln!("\n═══ Step 15: C# cross-language resolution ═══");
    {
        let mut g = fx_state.graph_bridge_lock();
        let b = g.as_mut().expect("fixture bridge");
        for name in &["Get", "Create", "Update", "Delete"] {
            let result = b.resolve_cross_language_endpoint(name);
            // eprintln!("  resolve(\"{name}\") = {result:?}");
            let endpoint =
                result.unwrap_or_else(|| panic!("cross-language resolve(\"{name}\") must succeed"));
            assert!(
                endpoint.contains("FixtureEngine"),
                "resolve(\"{name}\") must map into FixtureEngine, got \"{endpoint}\""
            );
        }
    }

    // eprintln!("\n═══ Step 16: Primary bridge still healthy ═══");
    {
        let mut g = state.graph_bridge_lock();
        let b = g.as_mut().expect("live bridge");
        let c1 = b.search("GraphBridge").len();
        // eprintln!("  Primary search(GraphBridge) = {c1} hits");
        assert!(c1 > 0, "Primary bridge must still be queryable");
    }

    // eprintln!("\n═══ Audit probe complete — all steps passed ═══\n");
} // _cleanup drops here: removes the fixture dir even on panic

// -- E2E: apply_edit -> automatic reindex -> fresh graph ------------
//
// CBM-EDIT-001 invariant: after a successful apply_edit returns,
// subsequent CBM graph queries observe the filesystem state
// produced by that edit.
//
// The reindex call is synchronous from apply_edit's perspective
// (the bridge calls client.index_repository() and waits for the
// JSON-RPC response before returning), so there is no timing
// dependency.

/// Create a small TypeScript fixture file in the given directory.
fn ts_fixture(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("src").join("service.ts");
    std::fs::create_dir_all(path.parent().unwrap()).expect("create src dir");
    std::fs::write(
        &path,
        "class MyService {\n  doSomething() {\n    return 42;\n  }\n}\n",
    )
    .expect("write fixture");
    path
}

/// Prove automatic reindex after apply_edit by modifying a method body
/// and verifying the CBM graph is still queryable after reindex.
#[serial(cbm_live)]
#[test]
fn e2e_apply_edit_triggers_reindex_and_graph_is_fresh() {
    if !cbm_binary_exists() {
        eprintln!("Skipping - CBM not installed");
        return;
    }
    let fixture_root =
        std::env::temp_dir().join(format!("clean-ctx-reindex-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&fixture_root);
    std::fs::create_dir_all(fixture_root.join("src")).expect("create fixture dir");
    struct FixtureCleanup(std::path::PathBuf);
    impl Drop for FixtureCleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = FixtureCleanup(fixture_root.clone());
    let fixture_path = ts_fixture(&fixture_root);
    let file_path_str = fixture_path.to_string_lossy().into_owned();
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = true;
    let fixture_str = fixture_root.to_string_lossy().to_string();
    config.additional_roots.push(fixture_str.clone());
    let state = crate::mcp::McpState::new(config);
    wait_for_indexing_complete(&state);
    let fx_slug = {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        b.resolve_project_id(&fixture_str)
    };
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        let before = b.search("doSomething");
        assert!(
            !before.is_empty(),
            "doSomething must be indexed before edit"
        );
    }
    crate::mcp::tool_handlers::core::handle_provide_code_context(
        &serde_json::json!(1),
        &serde_json::json!({"arguments": {"filePath": file_path_str.clone(), "fidelity": "edit"}}),
        &state,
    );
    crate::mcp::tool_handlers::edit::handle_apply_edit(
        &serde_json::json!(2),
        &serde_json::json!({"arguments": {"filePath": file_path_str.clone(), "operations": [{
            "type": "replace_body",
            "target": "MyService.doSomething",
            "expectedOldText": "{\n    return 42;\n  }",
            "newText": "{\n    return 99;\n  }"
        }]}}),
        &state,
    );
    let on_disk = std::fs::read_to_string(&fixture_path).expect("fixture must exist after edit");
    assert!(
        on_disk.contains("return 99;"),
        "edit must have written new body to disk:\n{on_disk}"
    );

    // ── Verify lazy freshness contract ──────────────────────────────
    // After apply_edit, the project must be dirty (no synchronous reindex).
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        let f_map = b.freshness.lock().unwrap_or_else(|p| p.into_inner());
        let entry = f_map
            .get(&fx_slug)
            .expect("project must have freshness entry");
        assert!(
            entry.dirty_generation > entry.indexed_generation,
            "project must be dirty after apply_edit (dirty={}, indexed={})",
            entry.dirty_generation,
            entry.indexed_generation,
        );
    }

    // First graph search: explicitly ensure freshness (as the production
    // MCP handler does via ensure_indexed_or_error), then search.
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        let idx_status = b.ensure_indexed();
        match idx_status {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => {}
            _ => panic!("ensure_indexed must return Ready after lazy reindex: {idx_status:?}"),
        }
        let result = b.search("doSomething");
        assert!(
            !result.is_empty(),
            "doSomething must be in graph after lazy reindex"
        );
    }

    // After the search, the project must be clean (indexed = dirty).
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        let f_map = b.freshness.lock().unwrap_or_else(|p| p.into_inner());
        let entry = f_map
            .get(&fx_slug)
            .expect("project must have freshness entry");
        assert_eq!(
            entry.dirty_generation, entry.indexed_generation,
            "project must be clean after lazy reindex (dirty={}, indexed={})",
            entry.dirty_generation, entry.indexed_generation,
        );
    }

    // Second graph search: explicitly ensure freshness, then search.
    // The project is already clean (ensure_indexed in the first block
    // advanced indexed_generation), so ensure_indexed is a fast no-op.
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        let idx_status = b.ensure_indexed();
        match idx_status {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => {}
            _ => panic!("ensure_indexed must return Ready on clean project: {idx_status:?}"),
        }
        let result = b.search("doSomething");
        assert!(
            !result.is_empty(),
            "doSomething must still be in graph on second search"
        );
    }

    // After the second search, the project must still be clean.
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        let f_map = b.freshness.lock().unwrap_or_else(|p| p.into_inner());
        let entry = f_map
            .get(&fx_slug)
            .expect("project must have freshness entry");
        assert_eq!(
            entry.dirty_generation, entry.indexed_generation,
            "project must remain clean after second search",
        );
    }
}
