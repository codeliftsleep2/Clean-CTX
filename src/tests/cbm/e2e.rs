// src/tests/cbm/e2e.rs
//
// End-to-end tests for the full CBM integration pipeline.
// These tests exercise the complete flow:
//   CbmClient → GraphBridge → intelligence layer → enrichment injection.
//
// All tests that require a live CBM binary check availability first
// and skip gracefully if CBM is not installed.
//
// Live-CBM tests share at most ONE process-scoped instance at a time (McpState →
// GraphBridge → CBM subprocess → configured project indexes → many requests).
// A degraded instance is dropped before its serial successor launches a replacement.
// Non-CBM / mock tests remain CBM-disabled and isolated.

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
// this module all acquire the same healthy instance via `shared_live_state()`:
//
//   one McpState → one GraphBridge → one CBM subprocess → configured project
//   indexes (started at construction) → wait for Complete → many MCP requests
//
// Live queries target the bridge's CANONICAL project: the CBM slug derived
// from the canonical `project_root` path (`cbm_project_slug`), which is what
// construction-time `start_indexing_roots()` indexed. An explicit `project`
// override resolves through the same canonical identity map, so overrides can
// no longer diverge from the indexed project. The live-CBM dispatch tests omit
// `project` (defaulting to the primary root) for that reason.
//
// Every live test is tagged `#[serial(cbm_live)]` so the shared instance is
// never touched by two threads at once. If a test degrades the transport, its
// successor drops that instance before launching a replacement. Non-CBM / mock
// tests in this file do NOT call these helpers and remain CBM-disabled.
//
// ## Cleanup — normal resource lifetime, no teardown hook
//
// A healthy shared `McpState` is retained by the `static` below. Cleanup happens
// through ordinary resource lifetime, with no teardown test or exit hook:
//
//   1. `McpState` (and its `GraphBridge`/`CbmClient`) are valid for the whole
//      process, so every live test reuses the same subprocess and indexed graph.
//   2. When the test process exits, the OS closes the parent→child stdin pipe,
//      CBM (an MCP server reading stdin) observes EOF and terminates.
//   3. When a degraded instance is replaced, dropping its final `Arc` runs
//      `CbmClient::drop`, which kills and reaps the old child before replacement.
//
// Both are "after normal lifetime" mechanisms, not explicit test or exit hooks.

/// How long the shared async `index_repository` may take before the suite gives up.
const LIVE_INDEXING_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(180);

/// The single process-scoped live-CBM `McpState`, shared by every
/// `#[serial(cbm_live)]` test. `None` until the first live test initializes it;
/// the `Mutex` makes first-arrival initialization atomic (belt-and-suspenders
/// on top of `#[serial]`).
static SHARED_LIVE_STATE: Mutex<Option<Arc<crate::mcp::McpState>>> = Mutex::new(None);

/// Acquire the healthy shared live-CBM state, initializing it when absent or
/// replacing it after circuit/transport degradation. Construction waits for
/// the async indexes to reach `Complete` before returning.
///
/// Callers MUST guard with `cbm_binary_exists()` first — this panics (with a
/// diagnostic) when CBM is not running.
fn shared_live_state() -> Arc<crate::mcp::McpState> {
    let mut guard = SHARED_LIVE_STATE.lock().unwrap_or_else(|p| p.into_inner());
    let replace_degraded = guard.as_ref().is_some_and(|state| {
        let mut bridge_guard = state.graph_bridge_lock();
        let bridge = bridge_guard
            .as_mut()
            .expect("shared live state must contain a GraphBridge");
        bridge.update_status();
        !bridge.is_available()
    });
    if replace_degraded {
        eprintln!("[cbm-e2e] Replacing degraded shared CBM subprocess...");
        *guard = None;
    }
    if guard.is_none() {
        eprintln!(
            "[cbm-e2e] Initializing shared live-CBM McpState (one subprocess for the suite)..."
        );
        let mut config = crate::config::CleanCtxConfig::default();
        config.cbm.enabled = true;
        let fixture_root = multiroot::prepare_shared_fixture();
        config
            .additional_roots
            .push(fixture_root.to_string_lossy().into_owned());
        let state = crate::mcp::McpState::new(config);
        wait_for_indexing_complete(&state);
        let fixture_project = {
            let mut bridge = state.graph_bridge_lock();
            bridge
                .as_mut()
                .expect("live bridge")
                .resolve_project_id(&fixture_root.to_string_lossy())
        };
        wait_for_project_indexing_complete(&state, &fixture_project);
        *guard = Some(Arc::new(state));
        eprintln!("[cbm-e2e] Shared live-CBM indexes Complete — graphs ready.");
    }
    Arc::clone(
        guard
            .as_ref()
            .expect("shared live state was just initialized"),
    )
}

fn wait_for_project_indexing_complete(state: &crate::mcp::McpState, project: &str) {
    let deadline = std::time::Instant::now() + LIVE_INDEXING_TIMEOUT;
    loop {
        let status = {
            let mut guard = state.graph_bridge_lock();
            guard
                .as_mut()
                .expect("live bridge")
                .ensure_indexed_for(project)
        };
        match status {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => return,
            Ok(crate::cbm::bridge::IndexingStatus::StillIndexing { .. }) => {
                if std::time::Instant::now() >= deadline {
                    panic!(
                        "[cbm-e2e] Timed out after {}s waiting for project {project}",
                        LIVE_INDEXING_TIMEOUT.as_secs()
                    );
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(error) => panic!("[cbm-e2e] Indexing failed for project {project}: {error}"),
        }
    }
}

struct ActiveWorkspaceGuard {
    state: Arc<crate::mcp::McpState>,
    previous_root: std::path::PathBuf,
}

impl Drop for ActiveWorkspaceGuard {
    fn drop(&mut self) {
        let mut guard = self.state.graph_bridge_lock();
        if let Some(bridge) = guard.as_mut() {
            bridge.set_workspace_root(&self.previous_root);
        }
    }
}

fn activate_shared_workspace(
    state: &Arc<crate::mcp::McpState>,
    root: &std::path::Path,
) -> ActiveWorkspaceGuard {
    let previous_root = {
        let mut guard = state.graph_bridge_lock();
        let bridge = guard.as_mut().expect("live bridge");
        let previous_root = bridge.project_root.clone();
        bridge.set_workspace_root(root);
        previous_root
    };
    ActiveWorkspaceGuard {
        state: Arc::clone(state),
        previous_root,
    }
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

/// Prove consecutive acquisitions reuse one healthy `McpState` (→ one
/// GraphBridge, one CBM subprocess, one set of indexed graphs).
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

#[path = "e2e_basic.rs"]
mod basic;
#[path = "e2e_live_queries.rs"]
mod live_queries;
#[path = "e2e_multiroot.rs"]
mod multiroot;
#[path = "e2e_reindex.rs"]
mod reindex;
