// src/tests/cbm/caller_verify_proxy.rs
//
// Single-target caller-verification pins: `trace_path` / `query_graph` request
// construction and surface counts. Per-result `search_graph` orchestration is
// pinned in `src/tests/cbm/caller_verify_search.rs`.

use crate::cbm::caller_verify::{CallerVerificationStatus, CallerVerificationSummary};
use crate::cbm::caller_verify_proxy::{
    VerificationGap, annotate_surface_counts, candidate_path_query, extract_qualified_target,
    verification_request,
};
use serde_json::json;

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
