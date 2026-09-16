// Filesystem-fallback hydration discovery-cache regressions (RED-FS1..FS5).
//
// These are the primary motivation for the discovery cache: for roots CBM
// cannot cover (CBM unavailable, degraded, or project-explicit fallback), a
// repeated identical query used to re-walk and re-read the whole root — the most
// expensive discovery path in Clean-CTX.
//
// Instrumentation: `scan` records an invocation count (thread-local, structural
// — never wall-clock), and `TraversalStats` reports how many files a scan
// actually read, which is how per-root granularity is proven.

use crate::cbm::GraphBridge;
use crate::cbm::bridge::cbm_project_slug;
use crate::mcp::tool_handlers::hydration::{
    TestProjectReadiness, clear_test_project_search_results, last_test_traversal_stats,
    reset_test_scan_calls, set_test_project_readiness, set_test_project_search_results,
    test_scan_calls,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

static TEST_SERIALIZE: &std::sync::Mutex<()> =
    &crate::mcp::tool_handlers::hydration::TEST_PROJECT_HYDRATION_SERIALIZE;

fn serialize() -> std::sync::MutexGuard<'static, ()> {
    TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// State with no CBM bridge at all: every root goes through filesystem discovery.
fn filesystem_only_state(additional: &[PathBuf]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}

/// State with a (disabled) bridge, used to exercise the per-project fallback.
fn bridge_state(primary: &Path) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create_with_roots(
        &bridge_config,
        primary,
        &[],
    ));
    state
}

fn project_slug(root: &Path) -> String {
    cbm_project_slug(&root.canonicalize().expect("test root canonicalizes"))
}

fn call_query(
    state: &crate::mcp::McpState,
    query_type: &str,
    name: &str,
    root: &Path,
) -> serde_json::Map<String, serde_json::Value> {
    let (result, count, attempted, hydration) = super::run_query_with_hydration(
        state,
        query_type,
        name,
        Some(&root.to_string_lossy()),
        |index| {
            let values = match query_type {
                "find_entities" => serde_json::to_value(index.find_entities_by_name(name)),
                "reverse_edges" => serde_json::to_value(
                    index.reverse_edges_by_identity("angular", "Service", name),
                ),
                other => panic!("unsupported test query: {other}"),
            }
            .unwrap_or_default();
            let count = values.as_array().map_or(0, Vec::len);
            (values, count)
        },
    );
    serde_json::json!({
        "result": result,
        "count": count,
        "hydration_attempted": attempted,
        "candidates_discovered": hydration.candidates_discovered,
        "candidates_compiled": hydration.candidates_compiled,
        "discovery_completed": hydration.discovery_completed,
        "discovery_provider": hydration.discovery_provider,
        "fallback_occurred": hydration.fallback_occurred,
        "fallback_reason": hydration.fallback_reason,
    })
    .as_object()
    .expect("structured result")
    .clone()
}

fn write_source(path: &Path, contents: &str) {
    std::fs::write(path, contents).unwrap();
}

// ── RED-FS1: a repeated query does not re-walk the filesystem ───────────

#[test]
fn red_fs1_repeated_query_does_not_repeat_the_filesystem_scan() {
    let _serial = serialize();
    reset_test_scan_calls();
    let root = tempfile::TempDir::new().unwrap();
    write_source(&root.path().join("Target.ts"), "export class Target {}\n");
    let state = filesystem_only_state(&[]);

    let first = call_query(&state, "find_entities", "Target", root.path());
    assert_eq!(first["discovery_provider"], "filesystem");
    assert_eq!(first["candidates_discovered"], 1);
    assert_eq!(first["candidates_compiled"], 1);
    assert_eq!(test_scan_calls(), 1, "the first call walks the root");

    let second = call_query(&state, "find_entities", "Target", root.path());
    assert_eq!(
        test_scan_calls(),
        1,
        "an unchanged repeated query must not walk or read the root again"
    );
    assert_eq!(second["candidates_discovered"], 0);
    assert_eq!(second["candidates_compiled"], 0);
    assert_eq!(second["count"], 1, "the WorkspaceIndex still answers");
    assert_eq!(second["discovery_completed"], true);
    assert_eq!(second["hydration_attempted"], true);
}

// ── RED-FS2: degraded CBM falls back once, not on every query ───────────

#[test]
fn red_fs2_degraded_cbm_fallback_scan_runs_once() {
    let _serial = serialize();
    reset_test_scan_calls();
    let root = tempfile::TempDir::new().unwrap();
    write_source(
        &root.path().join("Fallback.ts"),
        "export class Fallback {}\n",
    );
    let state = bridge_state(root.path());
    let slug = project_slug(root.path());
    set_test_project_search_results(HashMap::from([(slug.clone(), Ok(Vec::new()))]));
    set_test_project_readiness(HashMap::from([(slug, TestProjectReadiness::Unavailable)]));

    let first = call_query(&state, "find_entities", "Fallback", root.path());
    assert_eq!(first["discovery_provider"], "filesystem");
    assert_eq!(first["fallback_reason"], "cbm_unavailable");
    assert_eq!(first["fallback_occurred"], true);
    assert_eq!(first["count"], 1);
    assert_eq!(test_scan_calls(), 1);

    let second = call_query(&state, "find_entities", "Fallback", root.path());
    assert_eq!(
        test_scan_calls(),
        1,
        "the fallback scan for an already-discovered root must not repeat"
    );
    assert_eq!(second["discovery_provider"], "filesystem");
    assert_eq!(second["discovery_completed"], true);
    assert_eq!(second["count"], 1);

    clear_test_project_search_results();
}

// ── RED-FS3: a detected external modification invalidates discovery ─────

#[test]
fn red_fs3_detected_external_modification_invalidates_discovery() {
    let _serial = serialize();
    reset_test_scan_calls();
    let root = tempfile::TempDir::new().unwrap();
    let file = root.path().join("Target.ts");
    write_source(&file, "export class Target {}\n");
    let state = filesystem_only_state(&[]);
    let file_path = file.to_string_lossy().into_owned();

    // Session read first, so the source cache holds this file's metadata.
    let _ = state.read_source(&file_path).unwrap();

    let _ = call_query(&state, "find_entities", "Target", root.path());
    let _ = call_query(&state, "find_entities", "Target", root.path());
    assert_eq!(test_scan_calls(), 1, "repeated query is cached");

    // External (host-editor style) modification: not routed through apply_edit.
    write_source(&file, "export class Target {}\nexport class Extra {}\n");
    // The next session read observes it through the existing mtime/size path.
    let _ = state.read_source(&file_path).unwrap();

    let third = call_query(&state, "find_entities", "Target", root.path());
    assert_eq!(
        test_scan_calls(),
        2,
        "a detected external change must force rediscovery"
    );
    assert_eq!(third["discovery_completed"], true);
}

// ── RED-FS4: partial / failed filesystem discovery is not cached ────────

#[test]
fn red_fs4_partial_filesystem_discovery_is_not_cached() {
    let _serial = serialize();
    reset_test_scan_calls();
    let root = tempfile::TempDir::new().unwrap();
    let missing = root.path().join("does-not-exist");
    let state = filesystem_only_state(&[]);

    let first = call_query(&state, "find_entities", "Target", &missing);
    assert_eq!(first["discovery_completed"], false);
    assert_eq!(test_scan_calls(), 1);

    let _ = call_query(&state, "find_entities", "Target", &missing);
    assert_eq!(
        test_scan_calls(),
        2,
        "a failed or partial discovery must never be cached"
    );
}

// ── RED-FS5: the fallback scan skips only already-discovered roots ──────

#[test]
fn red_fs5_fallback_scan_skips_only_already_discovered_roots() {
    let _serial = serialize();
    reset_test_scan_calls();
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    write_source(
        &primary.path().join("PrimaryTarget.ts"),
        "export class PrimaryTarget {}\n",
    );
    write_source(
        &additional.path().join("AdditionalTarget.ts"),
        "export class AdditionalTarget {}\n",
    );
    let state = filesystem_only_state(&[additional.path().to_path_buf()]);

    let first = call_query(&state, "find_entities", "Target", primary.path());
    assert_eq!(test_scan_calls(), 1);
    assert_eq!(first["candidates_discovered"], 2, "both roots are scanned");
    assert_eq!(last_test_traversal_stats().files_read, 2);

    // Invalidate one root only, as an edit inside that root would.
    crate::mcp::tool_handlers::hydration::invalidate_discovery_for_root(
        &state,
        &additional.path().to_string_lossy(),
    );

    let second = call_query(&state, "find_entities", "Target", primary.path());
    assert_eq!(test_scan_calls(), 2, "exactly one further scan");
    assert_eq!(
        last_test_traversal_stats().files_read,
        1,
        "only the invalidated root is walked; the cached root is skipped"
    );
    assert_eq!(second["hydration_attempted"], true);
}
