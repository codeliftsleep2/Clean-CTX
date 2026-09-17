// Response-level regressions for the sparse discovery diagnostics (RED-DIAG9..15).
//
// Sibling module of `workspace_query_diagnostics.rs`, declared from it with the
// nested `#[path]` idiom, so this module is a DESCENDANT of that test module and
// `use super::*` inherits its entire scope (imports, fixtures, helpers). The
// split exists because the two halves answer different questions: the parent
// pins the projection, this file drives the REAL MCP dispatch path and pins the
// produced response.

use super::*;

// Used only from this module. Imported here rather than in the parent so the
// parent's import list carries no binding it never uses itself (which the
// zero-warning gate rejects as an unused import).
use crate::mcp::tool_handlers::hydration::clear_test_project_search_results;

/// A representative degraded pass: the filesystem fallback supplied coverage,
/// discovery did not complete across every configured root, candidates were
/// found and compiled, and one configured project's CBM search failed.
fn degraded_report() -> HydrationReport {
    HydrationReport {
        discovery_provider: "filesystem",
        discovery_status: "partial",
        fallback_reason: Some("cbm_partial_failure"),
        candidates_discovered: 5,
        candidates_compiled: 3,
        project_coverage: vec![
            coverage("Healthy", "searched", Some("ready"), None),
            coverage(
                "Failing",
                "search_failed",
                Some("ready"),
                Some("search_failed"),
            ),
        ],
        ..steady_state_report()
    }
}

/// Sorted key set of a JSON object, so field-order assertions cannot be affected
/// by the map implementation.
fn sorted_keys(value: &Value) -> Vec<String> {
    let mut keys: Vec<String> = value
        .as_object()
        .expect("discovery object")
        .keys()
        .cloned()
        .collect();
    keys.sort_unstable();
    keys
}

/// A field that holds the value absence already means: nothing default-valued
/// may ever appear in a diagnostic.
fn is_default_valued(key: &str, value: &Value) -> bool {
    match key {
        "provider" => value == &json!("cbm"),
        "status" => value == &json!("completed"),
        "attempted" => value == &json!(true),
        "fallback" => value == &json!(false),
        "discovered" | "compiled" | "projects_truncated" => value == &json!(0),
        "projects" => value.as_array().is_some_and(|entries| entries.is_empty()),
        _ => false,
    }
}

/// RED-DIAG9 — a normal `reverse_edges` answer is byte-for-byte the index answer
/// (count, edges, entities, call evidence, provenance); only diagnostics shrink.
#[test]
fn red_diag9_semantic_payload_is_unchanged() {
    let _serial = serialize_tests();
    let (state, root) = seeded_steady_state();
    let root_str = root.path().to_string_lossy().into_owned();

    let sc = structured(
        &state,
        json!({
            "type": "reverse_edges",
            "domain": "spring",
            "entity_type": "Service",
            "name": "UserService",
            "workspaceRoot": root_str,
        }),
    );

    // The authoritative answer, produced independently through the same scoped
    // method the handler uses.
    let expected = {
        let index = state.workspace_index_read();
        let scope = crate::workspace::scope::WorkspaceScope::new(
            Some(&root_str),
            &state.config.additional_roots,
        )
        .expect("authorized scope");
        serde_json::to_value(index.reverse_edges_by_identity_in_scope(
            "spring",
            "Service",
            "UserService",
            &scope,
        ))
        .unwrap_or_default()
    };
    assert!(
        !expected.as_array().expect("edges array").is_empty(),
        "the fixture must actually answer, or the comparison below is vacuous"
    );
    assert_eq!(
        sc["edges"], expected,
        "the returned edges must be exactly the index answer"
    );
    assert_eq!(
        sc["count"].as_u64(),
        Some(expected.as_array().expect("edges array").len() as u64),
        "count must describe those edges"
    );
    assert!(
        sc["edges"][0]["subject"]["file"].as_str().is_some(),
        "occurrence provenance must survive the projection"
    );
    assert!(
        discovery_of(&sc).is_none(),
        "a healthy answer carries no discovery diagnostics: {sc:?}"
    );
    assert_no_legacy_flat_diagnostics(&sc);

    // Call evidence survives on the real call fact.
    let calls = structured(
        &state,
        json!({
            "type": "reverse_edges",
            "domain": "builtin",
            "entity_type": "Method",
            "name": "OrderBy",
            "workspaceRoot": root_str,
        }),
    );
    assert_eq!(calls["count"], json!(1), "one real caller fact: {calls:?}");
    assert_eq!(calls["edges"][0]["relation"], json!("Calls"));
    assert_eq!(
        calls["edges"][0]["call_evidence"]["explicit_arg_count"],
        json!(2),
        "argc evidence must survive the projection"
    );
    assert!(discovery_of(&calls).is_none());
    assert_no_legacy_flat_diagnostics(&calls);

    clear_test_project_search_results();
}

// ── RED-DIAG10..13: steady-state surfaces ─────────────────────────────

/// A steady-state response carries its semantic answer and no boring
/// diagnostics at all: no `discovery` object, and none of the ten flat fields.
fn assert_steady_state_response(sc: &Map<String, Value>, semantic_key: &str) {
    assert!(
        sc[semantic_key]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "the semantic answer must be present under '{semantic_key}': {sc:?}"
    );
    assert!(
        sc["count"].as_u64().unwrap_or(0) >= 1,
        "count must describe the answer: {sc:?}"
    );
    assert!(
        discovery_of(sc).is_none(),
        "expected discovery must not be serialized: {sc:?}"
    );
    assert_no_legacy_flat_diagnostics(sc);
}

#[test]
fn red_diag10_find_entities_steady_state_carries_no_diagnostics() {
    let _serial = serialize_tests();
    let (state, root) = seeded_steady_state();
    let sc = structured(
        &state,
        json!({
            "type": "find_entities",
            "name": "UserService",
            "workspaceRoot": root.path().to_string_lossy(),
        }),
    );
    assert_steady_state_response(&sc, "entities");
    clear_test_project_search_results();
}

#[test]
fn red_diag11_forward_edges_steady_state_carries_no_diagnostics() {
    let _serial = serialize_tests();
    let (state, root) = seeded_steady_state();
    let sc = structured(
        &state,
        json!({
            "type": "forward_edges",
            "domain": "spring",
            "entity_type": "Controller",
            "name": "UserController",
            "workspaceRoot": root.path().to_string_lossy(),
        }),
    );
    assert_steady_state_response(&sc, "edges");
    clear_test_project_search_results();
}

#[test]
fn red_diag12_reverse_edges_steady_state_carries_no_diagnostics() {
    let _serial = serialize_tests();
    let (state, root) = seeded_steady_state();
    let sc = structured(
        &state,
        json!({
            "type": "reverse_edges",
            "domain": "spring",
            "entity_type": "Service",
            "name": "UserService",
            "workspaceRoot": root.path().to_string_lossy(),
        }),
    );
    assert_steady_state_response(&sc, "edges");
    clear_test_project_search_results();
}

#[test]
fn red_diag13_transitive_dependencies_steady_state_carries_no_diagnostics() {
    let _serial = serialize_tests();
    let (state, root) = seeded_steady_state();
    let sc = structured(
        &state,
        json!({
            "type": "transitive_dependencies",
            "domain": "spring",
            "entity_type": "Controller",
            "name": "UserController",
            "depth": 1,
            "workspaceRoot": root.path().to_string_lossy(),
        }),
    );
    assert_steady_state_response(&sc, "dependencies");
    assert_eq!(sc["depth_used"], json!(1));
    clear_test_project_search_results();
}

// ── RED-DIAG14: a degraded result stays informative ───────────────────

#[test]
fn red_diag14_degraded_result_still_informative() {
    let report = degraded_report();
    // `super` is the parent test module; the projection lives one level above it
    // (in the `diagnostics` module this test module is declared from).
    let discovery =
        super::super::discovery_field(&report).expect("a degraded pass must be reported");
    assert_eq!(
        discovery,
        json!({
            "provider": "filesystem",
            "status": "partial",
            "fallback_reason": "cbm_partial_failure",
            "discovered": 5,
            "compiled": 3,
            "projects": [{ "project": "Failing", "status": "search_failed" }],
        }),
        "every decision-relevant deviation survives, and nothing else does"
    );

    // Measured reduction for this representative degraded case.
    let new = Some(discovery.clone());
    let legacy = Some(legacy_flat_diagnostics(&report));
    let legacy_chars = diagnostic_chars(&legacy);
    let new_chars = diagnostic_chars(&new);
    let legacy_tokens = diagnostic_tokens(&legacy);
    let new_tokens = diagnostic_tokens(&new);
    assert!(
        new_chars < legacy_chars,
        "degraded diagnostics must shrink: {legacy_chars} chars/{legacy_tokens} tokens -> \
         {new_chars} chars/{new_tokens} tokens"
    );
    assert!(
        new_tokens < legacy_tokens,
        "degraded diagnostics must shrink in tokens too: {legacy_tokens} -> {new_tokens}"
    );
}

// ── RED-DIAG15: an exceptional response is minimal ────────────────────

#[test]
fn red_diag15_exceptional_response_is_minimal() {
    let discovery = super::super::discovery_field(&degraded_report()).expect("reported");
    assert_eq!(
        sorted_keys(&discovery),
        vec![
            "compiled",
            "discovered",
            "fallback_reason",
            "projects",
            "provider",
            "status"
        ],
        "the exact allowed field set of this fixture"
    );
    for absent in [
        "attempted",
        "fallback",
        "completed",
        "discovery_completed",
        "projects_truncated",
    ] {
        assert!(
            discovery.get(absent).is_none(),
            "redundant/default field '{absent}' must be absent: {discovery:?}"
        );
    }
    let entry = &discovery["projects"][0];
    assert!(
        entry.get("reason").is_none(),
        "a failed search must not repeat its status as its reason: {entry:?}"
    );
    assert!(
        entry.get("readiness").is_none(),
        "readiness changes no conclusion for a failed search: {entry:?}"
    );

    // The same minimality holds on the real dispatch path: a degraded
    // filesystem-only discovery (one configured root cannot be walked) reports
    // exactly the deviations it observed.
    let _serial = serialize_tests();
    let root = tempfile::TempDir::new().expect("temp workspace root");
    std::fs::write(
        root.path().join("DegradedTarget.ts"),
        "export class DegradedTarget {}\n",
    )
    .expect("fixture source");
    let missing_root = root.path().join("does-not-exist");
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = vec![missing_root.to_string_lossy().into_owned()];
    let state = McpState::new(config);
    *state.graph_bridge_lock() = None;

    let sc = structured(
        &state,
        json!({
            "type": "find_entities",
            "name": "DegradedTarget",
            "workspaceRoot": root.path().to_string_lossy(),
        }),
    );

    assert!(
        sc["count"].as_u64().unwrap_or(0) >= 1,
        "the semantic answer is untouched by the projection: {sc:?}"
    );
    let discovery = discovery_of(&sc).expect("a degraded discovery must be reported");
    assert_eq!(
        sorted_keys(discovery),
        vec![
            "compiled",
            "discovered",
            "fallback_reason",
            "provider",
            "status"
        ],
        "the exact allowed field set of this degraded response: {discovery:?}"
    );
    assert_eq!(discovery["provider"], json!("filesystem"));
    assert_eq!(discovery["status"], json!("partial"));
    assert_eq!(discovery["fallback_reason"], json!("cbm_unavailable"));
    assert!(discovery["discovered"].as_u64().unwrap_or(0) >= 1);
    assert!(discovery["compiled"].as_u64().unwrap_or(0) >= 1);
    for (key, value) in discovery.as_object().expect("discovery object") {
        assert!(
            !is_default_valued(key, value),
            "a degraded response must contain zero default-valued diagnostics, found '{key}'"
        );
    }
    assert_no_legacy_flat_diagnostics(&sc);
}
