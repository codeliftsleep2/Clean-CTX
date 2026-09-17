// Session-level hydration discovery-cache regressions (RED-H1..H10).
//
// These pin the contract introduced by the discovery cache:
//   1. discovery for one semantic target runs at most once per project/root and
//      workspace generation;
//   2. the WorkspaceIndex query itself still runs on every call;
//   3. successful zero-result discovery is cached, failures and not-yet-ready
//      projects are retried;
//   4. `apply_edit` and an explicit repository refresh invalidate the cache for
//      the affected root only;
//   5. caching is per project/root and per discovery mode.
//
// Instrumentation: the existing cfg(test) project-search injection records one
// entry per discovery call, so "discovery did not run again" is asserted
// structurally (call counts), never by wall-clock timing.

use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::{GraphBridge, GraphNode};
use crate::mcp::tool_handlers::hydration::{
    TestDiscoveryKind, TestProjectReadiness, TestProjectSearchResult,
    clear_test_project_search_results, discovery_calls, reset_test_scan_calls,
    set_test_inbound_project_search_results, set_test_project_readiness,
    set_test_project_search_results, test_scan_calls,
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

fn node(file: impl Into<String>) -> GraphNode {
    GraphNode {
        id: "candidate".into(),
        label: "Class".into(),
        name: "candidate".into(),
        file: file.into(),
        properties: HashMap::new(),
    }
}

fn state_with_roots(primary: &Path, additional: &[PathBuf]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    let state = crate::mcp::McpState::new(config);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create_with_roots(
        &bridge_config,
        primary,
        additional,
    ));
    state
}

fn project_slug(root: &Path) -> String {
    cbm_project_slug(&root.canonicalize().expect("test root canonicalizes"))
}

fn configure(
    declaration: Vec<(String, TestProjectSearchResult)>,
    inbound: Vec<(String, TestProjectSearchResult)>,
) {
    set_test_project_search_results(declaration.into_iter().collect());
    set_test_inbound_project_search_results(inbound.into_iter().collect());
}

/// Number of recorded discovery calls for one project.
fn calls_for(project: &str) -> usize {
    discovery_calls()
        .iter()
        .filter(|(searched, _)| searched == project)
        .count()
}

fn call_query(
    state: &crate::mcp::McpState,
    query_type: &str,
    name: &str,
    root: &Path,
) -> serde_json::Map<String, serde_json::Value> {
    let (result, count, hydration) = super::run_query_with_hydration(
        state,
        query_type,
        name,
        Some(&root.to_string_lossy()),
        |index| {
            let values = match query_type {
                "find_entities" => serde_json::to_value(index.find_entities_by_name(name)),
                "forward_edges" => serde_json::to_value(
                    index.forward_edges_by_identity("angular", "Service", name),
                ),
                "reverse_edges" => serde_json::to_value(
                    index.reverse_edges_by_identity("angular", "Service", name),
                ),
                "transitive_dependencies" => serde_json::to_value(
                    index.transitive_dependencies("angular", "Service", name, 1),
                ),
                other => panic!("unsupported test query: {other}"),
            }
            .unwrap_or_default();
            let count = values.as_array().map_or(0, Vec::len);
            (values, count)
        },
    );
    // The response projection itself, so these regressions assert the contract
    // the handler actually produces and cannot drift from it.
    let mut structured = serde_json::json!({ "result": result, "count": count })
        .as_object()
        .cloned()
        .expect("structured result");
    if let Some(discovery) = super::discovery_field(&hydration) {
        structured.insert("discovery".to_string(), discovery);
    }
    structured
}

fn write_service(root: &Path) {
    std::fs::write(
        root.join("ServiceA.ts"),
        "import { Injectable } from '@angular/core';\n@Injectable()\nexport class ServiceA {}\n",
    )
    .unwrap();
}

fn write_consumer(root: &Path, file: &str, class_name: &str) {
    std::fs::write(
        root.join(file),
        format!(
            "import {{ Component }} from '@angular/core';\nimport {{ ServiceA }} from './ServiceA';\n@Component({{ selector: 'x-{class_name}' }})\nexport class {class_name} {{\n  constructor(private service: ServiceA) {{}}\n}}\n"
        ),
    )
    .unwrap();
}

// ── RED-H1: repeated identical query skips the second discovery ──────────

#[test]
fn red_h1_repeated_identical_query_does_not_repeat_discovery() {
    let _serial = serialize();
    let root = tempfile::TempDir::new().unwrap();
    write_service(root.path());
    write_consumer(root.path(), "ConsumerA.ts", "ConsumerA");
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure(
        vec![(slug.clone(), Ok(vec![node("ServiceA.ts")]))],
        vec![(slug.clone(), Ok(vec![node("ConsumerA.ts")]))],
    );

    let first = call_query(&state, "reverse_edges", "ServiceA", root.path());
    assert_eq!(calls_for(&slug), 1, "first call must discover");
    assert_eq!(first["discovery"]["discovered"], 1);
    assert_eq!(first["discovery"]["compiled"], 1);
    assert_eq!(
        first["count"], 1,
        "the live WorkspaceIndex answers the query"
    );
    assert_eq!(
        state.hydration_discovery_lock().completed_entries(),
        1,
        "exactly one discovery target is recorded"
    );

    let second = call_query(&state, "reverse_edges", "ServiceA", root.path());
    assert_eq!(
        calls_for(&slug),
        1,
        "an unchanged session must not re-run discovery for the same target"
    );
    assert!(
        second.get("discovery").is_none(),
        "a cached repeat discovers nothing and completes as expected, so it reports nothing: {second:?}"
    );
    assert_eq!(
        second["count"], 1,
        "the WorkspaceIndex query itself still executes"
    );

    clear_test_project_search_results();
}

// ── RED-H2: the second call skips discovery, not just compilation ────────

#[test]
fn red_h2_second_call_skips_discovery_not_only_compilation() {
    let _serial = serialize();
    let root = tempfile::TempDir::new().unwrap();
    write_service(root.path());
    write_consumer(root.path(), "ConsumerA.ts", "ConsumerA");
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure(
        vec![(slug.clone(), Ok(vec![node("ServiceA.ts")]))],
        vec![(slug.clone(), Ok(vec![node("ConsumerA.ts")]))],
    );

    let _ = call_query(&state, "reverse_edges", "ServiceA", root.path());
    let _ = call_query(&state, "reverse_edges", "ServiceA", root.path());

    // The discovery entry point itself was invoked exactly once. A "discovery
    // re-ran but selected zero compile candidates" implementation would record
    // two calls here, so this assertion cannot be satisfied by the pre-existing
    // compiled-file dedup in `select_candidates`.
    assert_eq!(
        discovery_calls(),
        vec![(slug.clone(), TestDiscoveryKind::InboundReference)]
    );

    // The cache is per target: an unseen name in the same project still runs
    // discovery.
    let other = call_query(&state, "reverse_edges", "ServiceB", root.path());
    assert_eq!(calls_for(&slug), 2, "a new target must still discover");
    assert_eq!(
        other["discovery"]["discovered"], 1,
        "a new target's discovery must still be reported: {other:?}"
    );

    clear_test_project_search_results();
}

// ── RED-H3: successful zero-result discovery is cached ──────────────────

#[test]
fn red_h3_successful_zero_result_discovery_is_cached() {
    let _serial = serialize();
    let root = tempfile::TempDir::new().unwrap();
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure(
        vec![(slug.clone(), Ok(Vec::new()))],
        vec![(slug.clone(), Ok(Vec::new()))],
    );

    let first = call_query(&state, "reverse_edges", "AbsentService", root.path());
    assert_eq!(calls_for(&slug), 1);
    assert!(
        first.get("discovery").is_none(),
        "a completed CBM pass with zero candidates and a healthy project reports nothing: {first:?}"
    );

    let _ = call_query(&state, "reverse_edges", "AbsentService", root.path());
    assert_eq!(
        calls_for(&slug),
        1,
        "a successful discovery that found nothing must be cached, not repeated"
    );

    clear_test_project_search_results();
}

// ── RED-H4: failed discovery is NOT cached ──────────────────────────────

#[test]
fn red_h4_failed_discovery_is_retried() {
    let _serial = serialize();
    reset_test_scan_calls();
    let root = tempfile::TempDir::new().unwrap();
    write_service(root.path());
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure(
        vec![(slug.clone(), Err("search failed".into()))],
        vec![(slug.clone(), Err("search failed".into()))],
    );

    let first = call_query(&state, "reverse_edges", "ServiceA", root.path());
    assert_eq!(calls_for(&slug), 1);
    // Existing hydration metadata contract: a failed project search is reported
    // per project, and the root still falls back to filesystem discovery (which
    // is why the overall discovery reports `completed` — it did complete, via
    // the fallback provider).
    assert_eq!(
        first["discovery"]["projects"][0]["status"], "search_failed",
        "a failed CBM discovery must be reported, never presented as absence"
    );
    assert_eq!(first["discovery"]["provider"], "filesystem");
    assert_eq!(test_scan_calls(), 1, "the fallback scan ran once");

    let second = call_query(&state, "reverse_edges", "ServiceA", root.path());
    assert_eq!(
        calls_for(&slug),
        2,
        "a transient discovery failure must never be cached as absence"
    );
    assert_eq!(
        second["discovery"]["projects"][0]["status"], "search_failed",
        "the retry must re-attempt the failed project search and re-report it"
    );
    assert_eq!(
        test_scan_calls(),
        1,
        "the successful filesystem fallback for this root is cached, so only the \
         failed CBM project search is retried"
    );

    clear_test_project_search_results();
}

// ── RED-H5: apply_edit invalidates discovery for the edited root ────────

#[test]
fn red_h5_apply_edit_invalidates_discovery_for_the_edited_root() {
    let _serial = serialize();
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    let editable = additional.path().join("Editable.ts");
    std::fs::write(
        &editable,
        "export class Editable {\n  ping() {\n    return 1;\n  }\n}\n",
    )
    .unwrap();
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure(
        vec![
            (primary_slug.clone(), Ok(vec![node("Editable.ts")])),
            (additional_slug.clone(), Ok(vec![node("Editable.ts")])),
        ],
        vec![
            (primary_slug.clone(), Ok(vec![node("Editable.ts")])),
            (additional_slug.clone(), Ok(vec![node("Editable.ts")])),
        ],
    );

    let _ = call_query(&state, "reverse_edges", "Editable", primary.path());
    let _ = call_query(&state, "reverse_edges", "Editable", primary.path());
    assert_eq!(
        calls_for(&additional_slug),
        1,
        "unchanged session must not rediscover"
    );

    // Supported edit path: establish tracked state, then apply_edit.
    let editable_path = editable.to_string_lossy().into_owned();
    crate::protocol::captured_responses().clear();
    crate::mcp::tool_handlers::core::handle_provide_code_context(
        &serde_json::json!(1),
        &serde_json::json!({
            "arguments": { "filePath": editable_path.clone(), "fidelity": "edit" }
        }),
        &state,
    );
    crate::protocol::captured_responses().clear();
    crate::mcp::tool_handlers::edit::handle_apply_edit(
        &serde_json::json!(2),
        &serde_json::json!({
            "arguments": {
                "filePath": editable_path,
                "operations": [{
                    "type": "replace_body",
                    "target": "Editable.ping",
                    "expectedOldText": "{\n    return 1;\n  }",
                    "newText": "{\n    return 2;\n  }"
                }]
            }
        }),
        &state,
    );
    let on_disk = std::fs::read_to_string(&editable).unwrap();
    assert!(
        on_disk.contains("return 2;"),
        "apply_edit must have written the new body: {on_disk}"
    );

    let _ = call_query(&state, "reverse_edges", "Editable", primary.path());
    assert_eq!(
        calls_for(&additional_slug),
        2,
        "an edit inside the additional root must invalidate that root's discovery"
    );
    assert_eq!(
        calls_for(&primary_slug),
        1,
        "invalidation is scoped to the edited root"
    );

    clear_test_project_search_results();
}

// ── RED-H6: a cache hit in one project never suppresses another ─────────

#[test]
fn red_h6_cache_hit_in_one_project_does_not_suppress_another() {
    let _serial = serialize();
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    std::fs::write(
        primary.path().join("ServiceA.ts"),
        "export class ServiceA {}\n",
    )
    .unwrap();
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure(
        vec![
            (primary_slug.clone(), Ok(Vec::new())),
            (additional_slug.clone(), Ok(Vec::new())),
        ],
        vec![
            (primary_slug.clone(), Ok(Vec::new())),
            (additional_slug.clone(), Ok(Vec::new())),
        ],
    );
    set_test_project_readiness(HashMap::from([(
        additional_slug.clone(),
        TestProjectReadiness::StillIndexing,
    )]));

    let _ = call_query(&state, "find_entities", "ServiceA", primary.path());
    assert_eq!(calls_for(&primary_slug), 1);
    assert_eq!(calls_for(&additional_slug), 1);

    let _ = call_query(&state, "find_entities", "ServiceA", primary.path());
    assert_eq!(calls_for(&primary_slug), 1, "ready project is cached");
    assert_eq!(
        calls_for(&additional_slug),
        2,
        "a project that is still indexing must keep discovering"
    );

    clear_test_project_search_results();
}

// ── RED-H7: declaration and inbound-reference discovery are separate ────

#[test]
fn red_h7_discovery_modes_have_separate_cache_entries() {
    let _serial = serialize();
    let root = tempfile::TempDir::new().unwrap();
    write_service(root.path());
    write_consumer(root.path(), "ConsumerA.ts", "ConsumerA");
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure(
        vec![(slug.clone(), Ok(vec![node("ServiceA.ts")]))],
        vec![(slug.clone(), Ok(vec![node("ConsumerA.ts")]))],
    );

    let _ = call_query(&state, "find_entities", "ServiceA", root.path());
    let _ = call_query(&state, "reverse_edges", "ServiceA", root.path());
    assert_eq!(
        discovery_calls(),
        vec![
            (slug.clone(), TestDiscoveryKind::Declaration),
            (slug.clone(), TestDiscoveryKind::InboundReference),
        ],
        "each discovery mode must run its own discovery"
    );

    let _ = call_query(&state, "find_entities", "ServiceA", root.path());
    let _ = call_query(&state, "reverse_edges", "ServiceA", root.path());
    assert_eq!(
        calls_for(&slug),
        2,
        "repeating both modes must hit each mode's own cache entry"
    );

    clear_test_project_search_results();
}

// ── RED-H8: identical discovery operations share one cache entry ────────

#[test]
fn red_h8_declaration_oriented_queries_share_one_discovery_entry() {
    let _serial = serialize();
    let root = tempfile::TempDir::new().unwrap();
    write_service(root.path());
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure(
        vec![(slug.clone(), Ok(vec![node("ServiceA.ts")]))],
        vec![(slug.clone(), Ok(vec![node("ConsumerA.ts")]))],
    );

    let first = call_query(&state, "find_entities", "ServiceA", root.path());
    let second = call_query(&state, "forward_edges", "ServiceA", root.path());
    let third = call_query(&state, "transitive_dependencies", "ServiceA", root.path());

    assert_eq!(
        first["discovery"]["discovered"], 1,
        "the shared declaration discovery is performed and reported once: {first:?}"
    );
    assert!(
        second.get("discovery").is_none() && third.get("discovery").is_none(),
        "the second and third call reuse the same discovery and report nothing: {second:?} / {third:?}"
    );
    assert_eq!(
        discovery_calls(),
        vec![(slug.clone(), TestDiscoveryKind::Declaration)],
        "find_entities / forward_edges / transitive_dependencies perform the identical \
         declaration discovery and must share one cache entry"
    );
    assert_eq!(
        state.hydration_discovery_lock().completed_entries(),
        1,
        "one target + one discovery mode = one cache entry"
    );

    clear_test_project_search_results();
}

// ── RED-H9: a generation change invalidates cached discovery ────────────

#[test]
fn red_h9_explicit_repository_refresh_invalidates_discovery() {
    let _serial = serialize();
    let root = tempfile::TempDir::new().unwrap();
    write_service(root.path());
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure(
        vec![(slug.clone(), Ok(vec![node("ServiceA.ts")]))],
        vec![(slug.clone(), Ok(vec![node("ConsumerA.ts")]))],
    );

    let _ = call_query(&state, "reverse_edges", "ServiceA", root.path());
    let _ = call_query(&state, "reverse_edges", "ServiceA", root.path());
    assert_eq!(calls_for(&slug), 1, "unchanged generation is cached");

    // Explicit repository refresh: the documented boundary for external edits.
    crate::mcp::tool_handlers::hydration::invalidate_discovery_for_root(
        &state,
        &root.path().to_string_lossy(),
    );

    let _ = call_query(&state, "reverse_edges", "ServiceA", root.path());
    assert_eq!(
        calls_for(&slug),
        2,
        "a generation change must force rediscovery"
    );

    clear_test_project_search_results();
}

// ── RED-H10: every configured root is discovered independently ──────────

#[test]
fn red_h10_additional_roots_are_discovered_independently() {
    let _serial = serialize();
    let primary = tempfile::TempDir::new().unwrap();
    let first_extra = tempfile::TempDir::new().unwrap();
    let second_extra = tempfile::TempDir::new().unwrap();
    std::fs::write(primary.path().join("Shared.ts"), "export class Shared {}\n").unwrap();
    let state = state_with_roots(
        primary.path(),
        &[
            first_extra.path().to_path_buf(),
            second_extra.path().to_path_buf(),
        ],
    );
    let slugs = [
        project_slug(primary.path()),
        project_slug(first_extra.path()),
        project_slug(second_extra.path()),
    ];
    configure(
        slugs
            .iter()
            .map(|slug| (slug.clone(), Ok(vec![node("Shared.ts")])))
            .collect(),
        slugs
            .iter()
            .map(|slug| (slug.clone(), Ok(vec![node("Shared.ts")])))
            .collect(),
    );

    for _ in 0..3 {
        let _ = call_query(&state, "find_entities", "Shared", primary.path());
    }

    for slug in &slugs {
        assert_eq!(
            calls_for(slug),
            1,
            "each configured root must discover exactly once: {slug}"
        );
    }
    assert_eq!(
        discovery_calls().len(),
        3,
        "no root may be skipped and none may be rediscovered"
    );

    clear_test_project_search_results();
}
