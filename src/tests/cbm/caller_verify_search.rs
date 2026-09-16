// src/tests/cbm/caller_verify_search.rs
//
// RED-SG1 .. RED-SG8: per-result caller verification for `search_graph`.
//
// Every case is deterministic — the shared verifier is injected as a canned
// closure — so these pins cover the orchestration (which results are verified,
// where the evidence lands, and how the surface aggregates) without CBM. The
// cases were derived from a real overload-conflation finding: an unanchored
// `name_pattern = "OrderBy"` search returned several matches and the custom
// overload's raw CBM `in_degree` was surfaced with no verification at all.

use crate::cbm::caller_verify::{CallerVerificationStatus, CallerVerificationSummary};
use crate::cbm::caller_verify_proxy::{VerificationAttempt, VerificationGap};
use crate::cbm::caller_verify_search::{
    SearchAggregate, annotate_search_evidence, search_targets, verify_search_results,
};
use serde_json::{Value, json};

/// The overloaded extension method the live finding was about.
const CUSTOM_OVERLOAD: &str = "P.CustomOrdering.OrderBy";
/// A second, unrelated callable symbol in the same search response.
const FRAMEWORK_OVERLOAD: &str = "P.Enumerable.OrderBy";
/// Two results sharing one identity (two overloads CBM cannot tell apart).
const AMBIGUOUS_OVERLOAD: &str = "P.OrderRepository.OrderBy";

fn summary(
    target: &str,
    verified: usize,
    rejected_arity: usize,
    ambiguous: usize,
    unverifiable: usize,
) -> CallerVerificationSummary {
    let resolution = if unverifiable > 0 {
        CallerVerificationStatus::Unverifiable
    } else if ambiguous > 0 {
        CallerVerificationStatus::Ambiguous
    } else if verified > 0 {
        CallerVerificationStatus::VerifiedCompatible
    } else if rejected_arity > 0 {
        CallerVerificationStatus::RejectedArityMismatch
    } else {
        CallerVerificationStatus::Unverifiable
    };
    CallerVerificationSummary {
        target_qualified_name: Some(target.to_string()),
        target_explicit_arity: None,
        raw_candidates: verified + rejected_arity + ambiguous,
        verified,
        rejected_arity,
        ambiguous,
        unverifiable,
        verified_caller_files: 0,
        compatible_caller_files: 0,
        resolution,
        verified_candidates: Vec::new(),
        ambiguous_candidates: Vec::new(),
    }
}

fn verified_attempt(target: &str, verified: usize, rejected_arity: usize) -> VerificationAttempt {
    VerificationAttempt {
        summary: summary(target, verified, rejected_arity, 0, 0),
        gap: None,
    }
}

fn ambiguous_attempt(target: &str, ambiguous: usize) -> VerificationAttempt {
    VerificationAttempt {
        summary: summary(target, 0, 0, ambiguous, 0),
        gap: None,
    }
}

fn unverifiable_attempt(target: &str, gap: VerificationGap) -> VerificationAttempt {
    VerificationAttempt {
        summary: summary(target, 0, 0, 0, 1),
        gap: Some(gap),
    }
}

/// Run the per-result orchestration with a canned verifier.
///
/// `table` supplies the attempt for each verifiable target; a target that is not
/// in the table yields an unverifiable attempt, so `asked` records exactly which
/// targets entered verification and in which order.
fn drive(
    payload: &mut Value,
    table: Vec<(&str, VerificationAttempt)>,
    asked: &mut Vec<String>,
) -> SearchAggregate {
    let targets = search_targets(payload, None);
    let mut pending = table;
    let aggregate = verify_search_results(payload, &targets, &mut |request, _memo| {
        let target = request.target.clone().unwrap_or_default();
        asked.push(target.clone());
        match pending.iter().position(|(name, _)| *name == target) {
            Some(index) => pending.remove(index).1,
            None => unverifiable_attempt(&target, VerificationGap::NoCallEvidence),
        }
    });
    annotate_search_evidence(payload, &aggregate);
    aggregate
}

/// The verification evidence attached to one result, for parity comparisons.
fn evidence_view(result: &Value) -> Value {
    json!({
        "raw_in_degree": result["raw_in_degree"],
        "verified_in_degree": result["verified_in_degree"],
        "in_degree_evidence": result["in_degree_evidence"],
        "in_degree_resolution": result["in_degree_resolution"],
    })
}

fn method(name: &str, qualified_name: &str, file_path: &str, in_degree: u64) -> Value {
    json!({
        "name": name,
        "qualified_name": qualified_name,
        "label": "Method",
        "file_path": file_path,
        "in_degree": in_degree
    })
}

fn kind(name: &str, qualified_name: &str, label: &str, in_degree: u64) -> Value {
    json!({
        "name": name,
        "qualified_name": qualified_name,
        "label": label,
        "file_path": "src/Program.cs",
        "in_degree": in_degree
    })
}

/// RED-SG1 — single-result control. An anchored search keeps exactly the
/// verification fields it had before the plural path existed.
#[test]
fn red_sg1_single_result_keeps_its_verification_fields() {
    let mut payload = json!({
        "total": 1,
        "results": [method("OrderBy", CUSTOM_OVERLOAD, "src/CustomOrdering.cs", 7)]
    });
    let mut asked = Vec::new();
    let aggregate = drive(
        &mut payload,
        vec![(CUSTOM_OVERLOAD, verified_attempt(CUSTOM_OVERLOAD, 3, 4))],
        &mut asked,
    );

    assert_eq!(
        asked,
        vec![CUSTOM_OVERLOAD],
        "a single result is simply a batch of one"
    );
    let result = &payload["results"][0];
    assert_eq!(result["in_degree"], 7, "the raw CBM value is preserved");
    assert_eq!(result["raw_in_degree"], 7);
    assert_eq!(result["verified_in_degree"], 3);
    assert_eq!(result["in_degree_evidence"], "cbm_raw");
    assert_eq!(result["in_degree_resolution"], "verified_compatible");
    assert!(result["in_degree_reason"].is_null());

    let top = &payload["clean_ctx_caller_verification"];
    assert_eq!(top["surface"], "search_graph");
    assert_eq!(top["evidence"], "arity");
    assert_eq!(top["cbm_relationships"], "raw_candidates_only");
    assert_eq!(top["target_qualified_name"], CUSTOM_OVERLOAD);
    assert_eq!(top["target_explicit_arity"], Value::Null);
    assert_eq!(top["raw_candidates"], 7);
    assert_eq!(top["verified"], 3);
    assert_eq!(top["rejected_arity"], 4);
    assert_eq!(top["resolution"], "verified_compatible");
    assert_eq!(top["results_total"], 1);
    assert_eq!(top["results_verified"], 1);
    assert_eq!(aggregate.results_total, 1);
    assert_eq!(aggregate.results_not_eligible, 0);
}

/// RED-SG2 — several matches, one callable: the eligible result is verified, the
/// non-callable kinds are annotated truthfully, and nothing is batch-skipped.
#[test]
fn red_sg2_only_the_callable_result_is_verified() {
    let mut payload = json!({
        "total": 3,
        "results": [
            kind("OrderBy", "P.OrderRepository.OrderBy", "Field", 2),
            method("OrderBy", CUSTOM_OVERLOAD, "src/CustomOrdering.cs", 7),
            kind("orderBy", "P.Program.orderBy", "Variable", 1)
        ]
    });
    let mut asked = Vec::new();
    let aggregate = drive(
        &mut payload,
        vec![(CUSTOM_OVERLOAD, verified_attempt(CUSTOM_OVERLOAD, 3, 4))],
        &mut asked,
    );

    assert_eq!(
        asked,
        vec![CUSTOM_OVERLOAD],
        "only the callable C# result is eligible for verification"
    );
    assert_eq!(payload["results"][1]["verified_in_degree"], 3);
    assert_eq!(payload["results"][1]["in_degree_evidence"], "cbm_raw");
    for index in [0usize, 2] {
        let result = &payload["results"][index];
        assert_eq!(
            result["in_degree_evidence"], "cbm_raw_unverified",
            "a non-callable kind must never look verified"
        );
        assert_eq!(result["verified_in_degree"], 0);
        assert_eq!(result["in_degree_resolution"], "unverifiable");
        assert_eq!(result["in_degree_reason"], "not_callable_symbol_kind");
    }
    assert_eq!(aggregate.results_total, 3);
    assert_eq!(aggregate.results_verified, 1);
    assert_eq!(aggregate.results_not_eligible, 2);
}

/// RED-SG3 — several callable results: each is verified independently and no
/// count bleeds between them.
#[test]
fn red_sg3_each_callable_result_is_verified_independently() {
    let mut payload = json!({
        "total": 2,
        "results": [
            method("OrderBy", CUSTOM_OVERLOAD, "src/CustomOrdering.cs", 7),
            method("OrderBy", FRAMEWORK_OVERLOAD, "src/Enumerable.cs", 3)
        ]
    });
    let mut asked = Vec::new();
    let aggregate = drive(
        &mut payload,
        vec![
            (CUSTOM_OVERLOAD, verified_attempt(CUSTOM_OVERLOAD, 3, 4)),
            (
                FRAMEWORK_OVERLOAD,
                verified_attempt(FRAMEWORK_OVERLOAD, 1, 2),
            ),
        ],
        &mut asked,
    );

    assert_eq!(asked, vec![CUSTOM_OVERLOAD, FRAMEWORK_OVERLOAD]);
    assert_eq!(payload["results"][0]["raw_in_degree"], 7);
    assert_eq!(payload["results"][0]["verified_in_degree"], 3);
    assert_eq!(payload["results"][1]["raw_in_degree"], 3);
    assert_eq!(payload["results"][1]["verified_in_degree"], 1);
    assert_eq!(payload["results"][1]["in_degree_evidence"], "cbm_raw");
    assert_eq!(aggregate.results_verified, 2);
    assert_eq!(aggregate.results_total, 2);
    assert_eq!(aggregate.summary.verified, 4);
    assert_eq!(aggregate.summary.rejected_arity, 6);
    assert_eq!(aggregate.summary.raw_candidates, 10);
    assert_eq!(
        aggregate.summary.target_qualified_name, None,
        "a plural aggregate must not report one target's identity"
    );
}

/// RED-SG4 — a verified result and an ambiguous one coexist: the ambiguity is
/// reported for its own result and never downgrades the verified neighbour.
#[test]
fn red_sg4_verified_and_ambiguous_results_coexist() {
    let mut payload = json!({
        "total": 2,
        "results": [
            method("OrderBy", CUSTOM_OVERLOAD, "src/CustomOrdering.cs", 7),
            {
                "name": "OrderBy",
                "qualified_name": AMBIGUOUS_OVERLOAD,
                "label": "Method",
                "file_path": "src/OrderRepository.cs",
                "in_degree": 5
            }
        ]
    });
    let mut asked = Vec::new();
    let aggregate = drive(
        &mut payload,
        vec![
            (CUSTOM_OVERLOAD, verified_attempt(CUSTOM_OVERLOAD, 3, 4)),
            (AMBIGUOUS_OVERLOAD, ambiguous_attempt(AMBIGUOUS_OVERLOAD, 2)),
        ],
        &mut asked,
    );

    assert_eq!(asked, vec![CUSTOM_OVERLOAD, AMBIGUOUS_OVERLOAD]);
    let verified = &payload["results"][0];
    assert_eq!(verified["verified_in_degree"], 3);
    assert_eq!(verified["in_degree_evidence"], "cbm_raw");
    assert_eq!(verified["in_degree_resolution"], "verified_compatible");
    assert!(verified["in_degree_reason"].is_null());

    let ambiguous = &payload["results"][1];
    assert_eq!(ambiguous["raw_in_degree"], 5);
    assert_eq!(ambiguous["verified_in_degree"], 0);
    assert_eq!(ambiguous["in_degree_evidence"], "cbm_raw_unverified");
    assert_eq!(ambiguous["in_degree_resolution"], "ambiguous");
    assert_eq!(ambiguous["in_degree_reason"], "ambiguous_arity_evidence");

    assert_eq!(aggregate.results_verified, 1);
    assert_eq!(aggregate.results_ambiguous, 1);
    assert_eq!(aggregate.summary.verified, 3);
    assert_eq!(aggregate.summary.ambiguous, 2);
    assert_eq!(
        aggregate.summary.resolution,
        CallerVerificationStatus::Ambiguous
    );
}

/// RED-SG5 — a verified result and an unverifiable one coexist, including a
/// result with no identity at all.
#[test]
fn red_sg5_verified_and_unverifiable_results_coexist() {
    let mut payload = json!({
        "total": 3,
        "results": [
            method("OrderBy", CUSTOM_OVERLOAD, "src/CustomOrdering.cs", 7),
            method("OrderBy", FRAMEWORK_OVERLOAD, "src/Enumerable.cs", 3),
            {"name": "orderBy", "label": "Method", "file_path": "src/Other.cs", "in_degree": 4}
        ]
    });
    let mut asked = Vec::new();
    let aggregate = drive(
        &mut payload,
        vec![
            (CUSTOM_OVERLOAD, verified_attempt(CUSTOM_OVERLOAD, 3, 4)),
            (
                FRAMEWORK_OVERLOAD,
                unverifiable_attempt(FRAMEWORK_OVERLOAD, VerificationGap::TargetSourceUnreadable),
            ),
        ],
        &mut asked,
    );

    assert_eq!(
        asked,
        vec![CUSTOM_OVERLOAD, FRAMEWORK_OVERLOAD],
        "a result without identity is never sent to the verifier"
    );
    let verified = &payload["results"][0];
    assert_eq!(verified["verified_in_degree"], 3);
    assert_eq!(verified["in_degree_evidence"], "cbm_raw");
    assert_eq!(
        verified["in_degree_resolution"], "verified_compatible",
        "a neighbour's failure must never downgrade a verified result"
    );

    let unverifiable = &payload["results"][1];
    assert_eq!(unverifiable["in_degree_evidence"], "cbm_raw_unverified");
    assert_eq!(unverifiable["verified_in_degree"], 0);
    assert_eq!(unverifiable["in_degree_resolution"], "unverifiable");
    assert_eq!(unverifiable["in_degree_reason"], "target_source_unreadable");

    let identity_less = &payload["results"][2];
    assert_eq!(identity_less["in_degree_evidence"], "cbm_raw_unverified");
    assert_eq!(identity_less["in_degree_resolution"], "unverifiable");
    assert_eq!(identity_less["in_degree_reason"], "missing_qualified_name");

    assert_eq!(aggregate.results_verified, 1);
    assert_eq!(aggregate.results_unverifiable, 1);
    assert_eq!(aggregate.results_not_eligible, 1);
    assert_eq!(aggregate.summary.unverifiable, 1);
    assert_eq!(
        aggregate.summary.resolution,
        CallerVerificationStatus::Unverifiable
    );
}

/// RED-SG6 — the live unanchored reproduction: `name_pattern = "OrderBy"`
/// returns several matches (fields, variables, other methods) and the custom
/// overload still receives its own verification evidence.
#[test]
fn red_sg6_unanchored_search_still_verifies_the_custom_overload() {
    let mut payload = json!({
        "total": 4,
        "results": [
            kind("OrderBy", "P.OrderRepository.OrderBy", "Field", 2),
            method("OrderBy", CUSTOM_OVERLOAD, "src/CustomOrdering.cs", 7),
            method("OrderBy", FRAMEWORK_OVERLOAD, "src/Enumerable.cs", 3),
            kind("orderBy", "P.Program.orderBy", "Variable", 1)
        ]
    });
    let mut asked = Vec::new();
    let aggregate = drive(
        &mut payload,
        vec![
            (CUSTOM_OVERLOAD, verified_attempt(CUSTOM_OVERLOAD, 3, 4)),
            (
                FRAMEWORK_OVERLOAD,
                verified_attempt(FRAMEWORK_OVERLOAD, 2, 3),
            ),
        ],
        &mut asked,
    );

    let overload = &payload["results"][1];
    assert_eq!(overload["in_degree"], 7);
    assert_eq!(overload["raw_in_degree"], 7);
    assert_eq!(overload["verified_in_degree"], 3);
    assert_eq!(overload["in_degree_evidence"], "cbm_raw");
    assert_eq!(overload["in_degree_resolution"], "verified_compatible");
    assert_eq!(asked, vec![CUSTOM_OVERLOAD, FRAMEWORK_OVERLOAD]);
    assert_eq!(aggregate.results_verified, 2);
    assert_eq!(aggregate.results_not_eligible, 2);
}

/// RED-SG7 — anchored/unanchored parity for the same symbol: search breadth must
/// never change the verification evidence of a returned symbol.
#[test]
fn red_sg7_anchored_and_unanchored_searches_agree_for_the_same_symbol() {
    let custom = method("OrderBy", CUSTOM_OVERLOAD, "src/CustomOrdering.cs", 7);
    let mut anchored = json!({"total": 1, "results": [custom.clone()]});
    let mut unanchored = json!({
        "total": 4,
        "results": [
            kind("OrderBy", "P.OrderRepository.OrderBy", "Field", 2),
            custom.clone(),
            method("OrderBy", FRAMEWORK_OVERLOAD, "src/Enumerable.cs", 3),
            kind("orderBy", "P.Program.orderBy", "Variable", 1)
        ]
    });

    let mut anchored_asked = Vec::new();
    drive(
        &mut anchored,
        vec![(CUSTOM_OVERLOAD, verified_attempt(CUSTOM_OVERLOAD, 3, 4))],
        &mut anchored_asked,
    );
    let mut unanchored_asked = Vec::new();
    drive(
        &mut unanchored,
        vec![
            (CUSTOM_OVERLOAD, verified_attempt(CUSTOM_OVERLOAD, 3, 4)),
            (
                FRAMEWORK_OVERLOAD,
                verified_attempt(FRAMEWORK_OVERLOAD, 2, 3),
            ),
        ],
        &mut unanchored_asked,
    );

    assert_eq!(anchored_asked, vec![CUSTOM_OVERLOAD]);
    assert_eq!(unanchored_asked, vec![CUSTOM_OVERLOAD, FRAMEWORK_OVERLOAD]);
    assert_eq!(
        evidence_view(&anchored["results"][0]),
        evidence_view(&unanchored["results"][1]),
        "the same symbol must receive identical evidence under both query shapes"
    );
    assert_eq!(
        anchored["clean_ctx_caller_verification"]["results_total"],
        1
    );
    assert_eq!(
        unanchored["clean_ctx_caller_verification"]["results_total"],
        4
    );
    assert_eq!(
        unanchored["clean_ctx_caller_verification"]["results_verified"],
        2
    );
}

/// RED-SG8 — no silent raw result: every `in_degree`-bearing entry of a mixed
/// response ends in an explicit state, and the disposition counters partition
/// the response.
#[test]
fn red_sg8_no_result_keeps_a_bare_raw_in_degree() {
    let duplicate = json!({
        "name": "OrderBy",
        "qualified_name": "P.Dup.OrderBy",
        "label": "Method",
        "file_path": "src/Dup.cs",
        "in_degree": 5
    });
    let mut payload = json!({
        "total": 9,
        "results": [
            kind("OrderBy", "P.OrderRepository.OrderBy", "Field", 2),
            method("OrderBy", CUSTOM_OVERLOAD, "src/CustomOrdering.cs", 7),
            method("OrderBy", FRAMEWORK_OVERLOAD, "src/Enumerable.cs", 3),
            duplicate.clone(),
            duplicate.clone(),
            {
                "name": "order_by",
                "qualified_name": "P.Program.order_by",
                "label": "Method",
                "file_path": "src/Program.rs",
                "in_degree": 4
            },
            {"name": "orderBy", "label": "Method", "file_path": "src/Other.cs", "in_degree": 1},
            kind("OrderBy", "P.Pricing.OrderBy", "Class", 9),
            {
                "name": "NoDegree",
                "qualified_name": "P.X.NoDegree",
                "label": "Method",
                "file_path": "src/X.cs"
            }
        ]
    });
    let mut asked = Vec::new();
    let aggregate = drive(
        &mut payload,
        vec![
            (CUSTOM_OVERLOAD, verified_attempt(CUSTOM_OVERLOAD, 3, 4)),
            (FRAMEWORK_OVERLOAD, ambiguous_attempt(FRAMEWORK_OVERLOAD, 2)),
        ],
        &mut asked,
    );

    assert_eq!(
        asked,
        vec![CUSTOM_OVERLOAD, FRAMEWORK_OVERLOAD],
        "duplicate identities and ineligible kinds never reach the verifier"
    );

    for (index, result) in payload["results"]
        .as_array()
        .expect("results array")
        .iter()
        .enumerate()
    {
        if result.get("in_degree").is_none() {
            assert!(
                result.get("in_degree_evidence").is_none(),
                "result {index}: a result without in_degree stays untouched"
            );
            continue;
        }
        let evidence = result["in_degree_evidence"].as_str().unwrap_or_default();
        assert!(
            matches!(evidence, "cbm_raw" | "cbm_raw_unverified"),
            "result {index}: in_degree_evidence must be explicit, got '{evidence}'"
        );
        assert!(
            !result["in_degree_resolution"].is_null(),
            "result {index}: every in_degree result carries a resolution"
        );
        assert!(
            result["verified_in_degree"].is_u64(),
            "result {index}: every in_degree result carries a verified count"
        );
    }

    for index in [3usize, 4] {
        assert_eq!(
            payload["results"][index]["in_degree_resolution"],
            "ambiguous"
        );
        assert_eq!(
            payload["results"][index]["in_degree_reason"],
            "duplicate_qualified_name"
        );
    }
    assert_eq!(
        payload["results"][5]["in_degree_reason"],
        "non_csharp_target"
    );
    assert_eq!(
        payload["results"][6]["in_degree_reason"],
        "missing_qualified_name"
    );
    assert_eq!(
        payload["results"][7]["in_degree_reason"],
        "not_callable_symbol_kind"
    );

    assert_eq!(aggregate.results_total, 9);
    assert_eq!(aggregate.results_verified, 1);
    assert_eq!(aggregate.results_ambiguous, 1);
    assert_eq!(aggregate.results_not_eligible, 7);
    assert_eq!(
        aggregate.results_total,
        aggregate.results_verified
            + aggregate.results_ambiguous
            + aggregate.results_rejected_or_adjusted
            + aggregate.results_unverifiable
            + aggregate.results_not_eligible,
        "the disposition counters must partition the response"
    );
}

/// A response with no `results` array, or no `in_degree` at all, has nothing to
/// verify: `search_graph` is then returned untouched (as before this layer).
#[test]
fn search_targets_are_empty_without_a_results_array_or_degree() {
    assert!(search_targets(&json!({"error": "no results"}), None).is_empty());
    assert!(
        search_targets(&json!({"results": [{"name": "X"}]}), None).is_empty(),
        "a result without in_degree holds nothing to verify"
    );
    assert_eq!(
        search_targets(&json!({"results": [{"name": "X", "in_degree": 0}]}), None).len(),
        1,
        "an explicit zero degree is still a raw CBM value"
    );
}
