// src/tests/cbm/caller_verify_proxy.rs
//
// Single-target caller-verification pins: `trace_path` / `query_graph` request
// construction and surface counts. Per-result `search_graph` orchestration is
// pinned in `src/tests/cbm/caller_verify_search.rs`.

use crate::cbm::bridge::test_helpers::new_mock_empty;
use crate::cbm::bridge::{CachedGraphData, QUERY_CACHE_KEY_NAMESPACE, convert_query_rows};
use crate::cbm::caller_verify::{
    CallerVerificationStatus, CallerVerificationSummary, CandidateSource, CsharpTargetSelector,
    annotate_caller_evidence, verify_csharp_callers,
};
use crate::cbm::caller_verify_arity::ParseMemo;
use crate::cbm::caller_verify_proxy::{
    VerificationAttempt, VerificationGap, VerificationRequest, annotate_surface_counts,
    candidate_path_query, candidate_paths, extract_qualified_target, verification_request,
    verify_request_sources,
};
use crate::cbm::caller_verify_search::{
    annotate_search_evidence, search_targets, verify_search_results,
};
use crate::mcp::McpState;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

fn summary() -> CallerVerificationSummary {
    CallerVerificationSummary {
        target_qualified_name: Some("A.OrderBy".into()),
        target_explicit_arity: Some(2),
        raw_candidates: 39,
        verified: 3,
        rejected_arity: 36,
        ambiguous: 0,
        unverifiable: 0,
        verified_caller_files: 1,
        compatible_caller_files: 1,
        resolution: CallerVerificationStatus::VerifiedCompatible,
        verified_candidates: Vec::new(),
        ambiguous_candidates: Vec::new(),
    }
}

#[test]
fn extracts_qualified_target_from_inbound_query() {
    let query = "MATCH (c)-[:CALLS]->(target) WHERE target.qualified_name = 'A.Foo' RETURN c";
    assert_eq!(extract_qualified_target(query).as_deref(), Some("A.Foo"));
    let outbound =
        "MATCH (caller)-[:CALLS]->(callee) WHERE caller.qualified_name = 'A.Foo' RETURN callee";
    assert_eq!(extract_qualified_target(outbound), None);
}

#[test]
fn candidate_path_query_is_label_neutral_and_escapes_identity() {
    let query = candidate_path_query("A.O'Reilly\\Foo");
    assert!(query.contains("MATCH (caller)-[:CALLS]->(target)"));
    assert!(!query.contains(":Function"));
    assert!(query.contains("A.O\\'Reilly\\\\Foo"));
}

#[test]
fn caller_surfaces_build_consistent_verification_requests() {
    let trace = verification_request(
        "trace_path",
        &json!({"function_name":"A.Foo","direction":"inbound"}),
        &json!({"callers":[{}, {}]}),
    )
    .expect("inbound trace should be verified");
    let query = verification_request(
        "query_graph",
        &json!({"query":"MATCH (c)-[:CALLS]->(t) WHERE t.qualified_name = 'A.Foo' RETURN c"}),
        &json!({"rows":[[], []]}),
    )
    .expect("inbound CALLS query should be verified");

    for request in [trace, query] {
        assert_eq!(request.target.as_deref(), Some("A.Foo"));
        assert_eq!(request.raw_candidates, 2);
    }
}

/// `search_graph` is a *plural* surface: the single-target request constructor
/// must not claim it, because verification is planned per result
/// (`caller_verify_search::search_targets`) instead of once per response.
#[test]
fn search_surface_is_never_a_single_target_request() {
    let payload = json!({"results":[{"qualified_name":"A.Foo","in_degree":2}]});
    assert!(verification_request("search_graph", &json!({}), &payload).is_none());
    assert!(
        verification_request("search_graph", &json!({}), &json!({"results":[{}]})).is_none(),
        "a search response with results but no in_degree must not build a request"
    );
}

/// The single-target counts stay pinned for `trace_path` / `query_graph`, and
/// `search_graph` evidence is deliberately *not* written here — it belongs to
/// each result.
#[test]
fn surface_counts_cover_only_the_single_target_surfaces() {
    let summary = summary();

    let mut trace = json!({"callers": []});
    annotate_surface_counts(&mut trace, "trace_path", &summary);
    assert_eq!(trace["caller_counts"]["raw_candidates"], 39);
    assert_eq!(trace["caller_counts"]["verified"], 3);
    assert_eq!(trace["caller_counts"]["rejected_arity"], 36);

    let mut query = json!({"rows": []});
    annotate_surface_counts(&mut query, "query_graph", &summary);
    assert_eq!(query["inbound_call_counts"]["verified"], 3);

    let mut search = json!({"results":[{"in_degree":39}]});
    annotate_surface_counts(&mut search, "search_graph", &summary);
    assert!(
        search["results"][0].get("raw_in_degree").is_none(),
        "search_graph evidence is attached per result, not by the single-target surface writer"
    );
    assert!(search.get("caller_counts").is_none());
}

/// The reason reported beside an unverified result is part of the response
/// contract, so the mapping is pinned.
#[test]
fn verification_gap_reasons_are_stable() {
    assert_eq!(
        VerificationGap::NoCallEvidence.reason(),
        "no_cbm_call_evidence"
    );
    assert_eq!(
        VerificationGap::TargetSourceUnreadable.reason(),
        "target_source_unreadable"
    );
    assert_eq!(
        VerificationGap::CandidateSourcesUnreadable.reason(),
        "candidate_sources_unreadable"
    );
    assert_eq!(
        VerificationGap::PartialCandidateSources.reason(),
        "partial_candidate_sources"
    );
}

// ── RED-CV5..CV12: cached discovery, retained identity, authority ────────────
/// Locate the cache key the bridge uses for project-explicit candidate
/// discovery — the same key shape production builds.
fn scoped_key(project: &str, target: &str) -> String {
    format!(
        "{QUERY_CACHE_KEY_NAMESPACE}:{project}:{}",
        candidate_path_query(target)
    )
}
/// Seed one candidate-discovery answer, converted through the PRODUCTION row
/// mapping so the transport contract is exercised rather than restated.
fn seed_discovery(
    bridge: &crate::cbm::bridge::GraphBridge,
    project: &str,
    target: &str,
    rows: &[(&str, &str)],
) {
    let columns = [
        "caller.file_path".to_string(),
        "target.file_path".to_string(),
    ];
    let rows: Vec<Vec<Value>> = rows
        .iter()
        .map(|(caller, file)| {
            vec![
                Value::String((*caller).into()),
                Value::String((*file).into()),
            ]
        })
        .collect();
    let result = convert_query_rows(&columns, &rows);
    bridge.cache.insert(
        scoped_key(project, target),
        CachedGraphData {
            data: serde_json::to_value(&result).expect("candidate view serializes"),
            expires_at: Instant::now() + Duration::from_secs(3600),
        },
    );
}
/// RED-CV7 — project-explicit candidate discovery reuses the graph-query cache.
///
/// The mock bridge owns no CBM client, so a provider call necessarily fails: a
/// cached answer can only be observed as a successful discovery with no error
/// recorded, and an unseeded symbol as exactly one failed provider contact.
#[test]
fn red_cv7_candidate_discovery_reuses_the_graph_cache() {
    let mut bridge = new_mock_empty();
    let project = bridge.project_str();
    seed_discovery(&bridge, &project, "A.Foo", &[("caller.cs", "target.cs")]);

    let first = candidate_paths(&mut bridge, "A.Foo", Some(&project));
    assert_eq!(
        first,
        Some(("target.cs".to_string(), vec!["caller.cs".to_string()])),
        "the cached candidate rows are served"
    );
    assert!(
        bridge.take_last_error().is_none(),
        "a cache hit must not contact the provider"
    );

    let second = candidate_paths(&mut bridge, "A.Foo", Some(&project));
    assert_eq!(second, first, "a repeated request is served identically");
    assert!(
        bridge.take_last_error().is_none(),
        "the repeat still costs no provider round-trip"
    );

    assert!(
        candidate_paths(&mut bridge, "A.Missing", Some(&project)).is_none(),
        "an unseeded symbol discovers nothing"
    );
    assert!(
        bridge.take_last_error().is_some(),
        "a miss is the only case that reaches the provider"
    );
}
/// RED-CV8 — the cache key carries the workspace, so two repositories that share
/// a symbol name can never read each other's candidates.
#[test]
fn red_cv8_scoped_keys_isolate_workspaces() {
    let mut bridge = new_mock_empty();
    let active = bridge.project_str();
    assert_ne!(
        scoped_key(&active, "OrderBy"),
        scoped_key("repo-b", "OrderBy"),
        "the same Cypher under two projects must not share one key"
    );

    seed_discovery(
        &bridge,
        &active,
        "OrderBy",
        &[("a/Caller.cs", "a/Target.cs")],
    );
    seed_discovery(
        &bridge,
        "repo-b",
        "OrderBy",
        &[("b/Caller.cs", "b/Target.cs")],
    );

    assert_eq!(
        candidate_paths(&mut bridge, "OrderBy", Some(&active)),
        Some(("a/Target.cs".to_string(), vec!["a/Caller.cs".to_string()]))
    );
    // The request names a project that is NOT the bridge's active project — the
    // shape `cbm_proxy` produces, and the case the active-project key cannot
    // serve. Discovery still stays cached and still answers for the right repo.
    assert_eq!(
        candidate_paths(&mut bridge, "OrderBy", Some("repo-b")),
        Some(("b/Target.cs".to_string(), vec!["b/Caller.cs".to_string()]))
    );
    assert!(
        bridge.take_last_error().is_none(),
        "neither workspace needed a provider call"
    );
}
/// RED-CV9 — the cache belongs to one bridge (one CBM provider), so a second
/// provider instance cannot observe another's entries.
#[test]
fn red_cv9_cache_is_owned_by_one_provider_instance() {
    let mut first = new_mock_empty();
    let project = first.project_str();
    seed_discovery(&first, &project, "A.Foo", &[("caller.cs", "target.cs")]);

    let mut second = new_mock_empty();
    assert!(
        first.cache.contains_key(&scoped_key(&project, "A.Foo")),
        "the first provider holds its seeded entry"
    );
    assert!(
        !second.cache.contains_key(&scoped_key(&project, "A.Foo")),
        "a distinct provider instance holds none of the first provider's entries"
    );
    assert!(candidate_paths(&mut second, "A.Foo", Some(&project)).is_none());
    assert!(
        second.take_last_error().is_some(),
        "the second provider is contacted on its own miss"
    );
    assert_eq!(
        candidate_paths(&mut first, "A.Foo", Some(&project)),
        Some(("target.cs".to_string(), vec!["caller.cs".to_string()])),
        "the first provider's entry is untouched"
    );
}

/// Drive the real verification pipeline for one target end to end: cached
/// candidate discovery, trusted-path reads, Clean-CTX verification.
fn verify_from_cache(
    bridge: &mut crate::cbm::bridge::GraphBridge,
    state: &McpState,
    target: &str,
    project: &str,
    workspace_root: &str,
) -> VerificationAttempt {
    let request = VerificationRequest {
        target: Some(target.to_string()),
        raw_candidates: 1,
        project: Some(project.to_string()),
    };
    verify_request_sources(
        bridge,
        state,
        &request,
        Some(workspace_root),
        &mut ParseMemo::new(),
    )
}

/// RED-CV5 — the response a consumer actually receives names the verified caller,
/// with no second CBM lookup.
///
/// Discovery is served from the cache for a project that is NOT the bridge's
/// active project (the `cbm_proxy` shape), verification runs against real source,
/// and the rendered surface carries the candidate identity and its evidence.
#[test]
fn red_cv5_verified_identity_reaches_the_response_surface() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let root = dir.path().to_string_lossy().into_owned();
    let target_file = dir.path().join("Example.cs");
    let caller_file = dir.path().join("Use.cs");
    std::fs::write(&target_file, "class Example { void Foo(int x) {} }\n").expect("target file");
    std::fs::write(&caller_file, "class Use { void M() { Foo(1); } }\n").expect("caller file");

    let state = McpState::new(crate::tests::test_config());
    let mut bridge = new_mock_empty();
    let target_path = target_file.to_string_lossy().into_owned();
    let caller_path = caller_file.to_string_lossy().into_owned();
    seed_discovery(
        &bridge,
        "repo-b",
        "Example.Foo",
        &[(&caller_path, &target_path)],
    );

    let attempt = verify_from_cache(&mut bridge, &state, "Example.Foo", "repo-b", &root);
    assert!(attempt.gap.is_none(), "source evidence was established");
    assert_eq!(attempt.summary.verified, 1);
    assert_eq!(attempt.summary.verified_caller_files, 1);
    assert!(
        bridge.take_last_error().is_none(),
        "the whole verification needed no provider round-trip"
    );

    // The externally visible block for the single-target surfaces.
    let mut payload = json!({ "rows": [] });
    annotate_caller_evidence(&mut payload, "query_graph", &attempt.summary);
    let block = &payload["clean_ctx_caller_verification"];
    assert_eq!(block["verified"], 1, "counts survive");
    assert_eq!(block["verified_caller_files"], 1);
    let candidates = block["verified_candidates"]
        .as_array()
        .expect("verified candidate identities are exposed");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0]["cbm_file"], json!(caller_path));
    assert_eq!(candidates[0]["file"], json!(caller_path));
    assert_eq!(candidates[0]["argument_counts"], json!([1]));
    assert_eq!(candidates[0]["status"], json!("verified_compatible"));
}

/// RED-CV12 — cached candidate discovery never becomes verification truth: a
/// candidate CBM proposed that source contradicts is still rejected.
#[test]
fn red_cv12_source_verification_refutes_a_cached_candidate() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let root = dir.path().to_string_lossy().into_owned();
    let target_file = dir.path().join("Example.cs");
    let caller_file = dir.path().join("Use.cs");
    std::fs::write(&target_file, "class Example { void Foo(int x) {} }\n").expect("target file");
    // The candidate calls the one-argument target with two arguments.
    std::fs::write(&caller_file, "class Use { void M() { Foo(1, 2); } }\n").expect("caller file");

    let state = McpState::new(crate::tests::test_config());
    let mut bridge = new_mock_empty();
    let caller_path = caller_file.to_string_lossy().into_owned();
    let target_path = target_file.to_string_lossy().into_owned();
    seed_discovery(
        &bridge,
        "repo-b",
        "Example.Foo",
        &[(&caller_path, &target_path)],
    );

    let attempt = verify_from_cache(&mut bridge, &state, "Example.Foo", "repo-b", &root);
    assert_eq!(attempt.summary.verified, 0);
    assert_eq!(attempt.summary.rejected_arity, 1);
    assert!(
        attempt.summary.verified_candidates.is_empty(),
        "a contradicted candidate is never reported as verified"
    );
    assert_eq!(
        attempt.summary.resolution,
        CallerVerificationStatus::RejectedArityMismatch
    );
    assert!(bridge.take_last_error().is_none());
}

/// RED-CV10 (batch level) — two `search_graph` results verified in ONE response
/// that read the same file parse it once for the whole batch.
#[test]
fn red_cv10_search_batch_parses_a_shared_file_once() {
    let target_source = "class Example { void Foo(int x) {} void Bar(int x) {} }";
    let caller_source = "class Use { void M() { Foo(1); Bar(1); } }";
    let mut payload = json!({
        "results": [
            {
                "name": "Foo", "qualified_name": "Example.Foo", "label": "Method",
                "file_path": "src/Example.cs", "in_degree": 1
            },
            {
                "name": "Bar", "qualified_name": "Example.Bar", "label": "Method",
                "file_path": "src/Example.cs", "in_degree": 1
            }
        ]
    });
    let targets = search_targets(&payload, None);
    assert_eq!(targets.len(), 2, "both results are eligible");

    let mut parses_after_each_result = Vec::new();
    let aggregate = verify_search_results(&mut payload, &targets, &mut |request, memo| {
        // Both results resolve their sources from the same canonical paths.
        let summary = verify_csharp_callers(
            target_source,
            "src/Example.cs",
            CsharpTargetSelector::new(request.target.as_deref().unwrap_or_default()),
            &[CandidateSource {
                file: "src/Use.cs",
                resolved_file: "src/Use.cs",
                text: caller_source,
            }],
            memo,
        );
        parses_after_each_result.push(memo.parses());
        VerificationAttempt { summary, gap: None }
    });
    annotate_search_evidence(&mut payload, &aggregate);

    assert_eq!(
        parses_after_each_result,
        vec![2, 2],
        "the second result reuses the first result's parses"
    );
    for index in [0usize, 1] {
        let candidates = payload["results"][index]["verified_candidates"]
            .as_array()
            .expect("each result exposes its own verified candidates");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0]["file"], json!("src/Use.cs"));
    }
    assert_eq!(
        payload["clean_ctx_caller_verification"]["verified"], 2,
        "the folded response block keeps the aggregate counts"
    );
}
