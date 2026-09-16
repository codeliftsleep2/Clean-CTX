use crate::cbm::caller_verify::{
    CallerVerificationStatus, CallerVerificationSummary, CandidateSource, CsharpTargetSelector,
    aggregate_resolution, annotate_caller_evidence, verify_csharp_callers as verify_with_memo,
};
use crate::cbm::caller_verify_arity::ParseMemo;
use serde_json::json;

/// A candidate source with its CBM identity and its resolved path.
fn source(file: &'static str, text: &'static str) -> CandidateSource<'static> {
    CandidateSource {
        file,
        resolved_file: file,
        text,
    }
}

/// One verification of one target with a fresh batch memo.
///
/// The production entry point takes the per-batch parse memo and the canonical
/// target path explicitly, so that a whole response parses each file once; a
/// single case here is a batch of one, so this wrapper supplies both. The
/// `RED-CV10` cases below call the real entry point directly to pin batch reuse
/// itself.
fn verify_csharp_callers(
    target_source: &str,
    selector: CsharpTargetSelector<'_>,
    candidates: &[CandidateSource<'_>],
) -> CallerVerificationSummary {
    verify_with_memo(
        target_source,
        "target.cs",
        selector,
        candidates,
        &mut ParseMemo::new(),
    )
}

#[test]
fn red_cbm_a1_simple_overload_by_arity() {
    let code = r#"
class Example {
    void Foo(int x) {}
    void Foo(int x, int y) {}
    void A() => Foo(1);
    void B() => Foo(1, 2);
}"#;
    let two = verify_csharp_callers(
        code,
        CsharpTargetSelector::new("Example.Foo").with_declared_arity(2),
        &[source("Example.cs", code)],
    );
    assert_eq!((two.verified, two.rejected_arity), (1, 1));
    let one = verify_csharp_callers(
        code,
        CsharpTargetSelector::new("Example.Foo").with_declared_arity(1),
        &[source("Example.cs", code)],
    );
    assert_eq!((one.verified, one.rejected_arity), (1, 1));
}

#[test]
fn red_cbm_a2_extension_receiver_is_implicit() {
    let target = r#"static class Extensions {
        public static void Foo(this Thing thing, int x, int y) {}
    }"#;
    let calls = "class Use { void M(Thing thing) { thing.Foo(1, 2); thing.Foo(1); } }";
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Extensions.Foo"),
        &[source("Use.cs", calls)],
    );
    assert_eq!(result.target_explicit_arity, Some(2));
    assert_eq!((result.verified, result.rejected_arity), (1, 1));
}

#[test]
fn red_cbm_a3_live_order_by_shape_filters_one_argument_calls() {
    let target = r#"static class CustomOrdering {
        public static IOrderedQueryable<T> OrderBy<T, TKey>(
            this IQueryable<T> source,
            Expression<Func<T, TKey>> keySelector,
            ListSortDirection sortOrder) => throw null;
    }"#;
    let calls = r#"class Use { void M(IQueryable<Person> query, ListSortDirection direction) {
        query.OrderBy(x => x.Name); query.OrderBy(x => x.Id);
        query.OrderBy(x => x.Name); query.OrderBy(x => x.Id);
        query.OrderBy(x => x.Name, direction);
        query.OrderBy(x => x.Id, direction);
        query.OrderBy(x => x.Age, direction);
    } }"#;
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("CustomOrdering.OrderBy"),
        &[source("Use.cs", calls)],
    );
    assert_eq!(result.raw_candidates, 7);
    assert_eq!((result.verified, result.rejected_arity), (3, 4));
}

#[test]
fn red_cbm_a4_same_name_same_arity_is_ambiguous() {
    let target = "class Example { void Foo(int x) {} void Foo(string x) {} }";
    let calls = "class Use { void M(dynamic value) { Foo(value); } }";
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Example.Foo").with_declared_arity(1),
        &[source("Use.cs", calls)],
    );
    assert_eq!((result.ambiguous, result.verified), (1, 0));
}

#[test]
fn red_cbm_a5_optional_parameters_are_not_rejected() {
    let target = "class Example { void Foo(int x, int y = 0) {} }";
    let calls = "class Use { void M() { Foo(1); Foo(1, 2); } }";
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Example.Foo"),
        &[source("Use.cs", calls)],
    );
    assert_eq!((result.rejected_arity, result.ambiguous), (0, 2));
}

#[test]
fn red_cbm_a6_params_is_not_fixed_arity() {
    let target = "class Example { void Foo(int x, params string[] values) {} }";
    let calls = r#"class Use { void M() { Foo(1); Foo(1, "a"); Foo(1, "a", "b"); } }"#;
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Example.Foo"),
        &[source("Use.cs", calls)],
    );
    assert_eq!((result.rejected_arity, result.ambiguous), (0, 3));
}

#[test]
fn red_cbm_a7_qualified_identity_is_preserved() {
    let target = "class A { void Foo(int x, int y) {} } class B { void Foo(int x) {} }";
    let calls = "class Use { void M(A a, B b) { a.Foo(1, 2); b.Foo(1); } }";
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("A.Foo"),
        &[source("Use.cs", calls)],
    );
    assert_eq!(result.target_qualified_name.as_deref(), Some("A.Foo"));
    assert_eq!((result.verified, result.rejected_arity), (1, 1));
}

#[test]
fn red_cbm_a8_unique_exact_arity_is_verified_without_warning() {
    let target = "class Example { void Foo(int x) {} }";
    let calls = "class Use { void M() { Foo(1); } }";
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Example.Foo"),
        &[source("Use.cs", calls)],
    );
    assert_eq!(result.verified, 1);
    assert_eq!(
        result.resolution,
        CallerVerificationStatus::VerifiedCompatible
    );
}

#[test]
fn red_s1_through_s4_all_surfaces_use_the_same_summary() {
    let target = "class Example { void Foo(int x, int y) {} }";
    let calls = "class Use { void M() { Foo(1); Foo(1, 2); } }";
    let summary = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Example.Foo"),
        &[source("Use.cs", calls)],
    );
    for tool in ["trace_path", "search_graph", "query_graph"] {
        let mut value = json!({"tool_payload": true});
        annotate_caller_evidence(&mut value, tool, &summary);
        assert_eq!(value["clean_ctx_caller_verification"]["verified"], 1);
        assert_eq!(value["clean_ctx_caller_verification"]["rejected_arity"], 1);
        assert_eq!(value["clean_ctx_caller_verification"]["evidence"], "arity");
        assert_eq!(value["clean_ctx_caller_verification"]["surface"], tool);
    }
}

fn summary(target: &str, verified: usize, rejected_arity: usize) -> CallerVerificationSummary {
    CallerVerificationSummary {
        target_qualified_name: Some(target.to_string()),
        target_explicit_arity: None,
        raw_candidates: verified + rejected_arity,
        verified,
        rejected_arity,
        ambiguous: 0,
        unverifiable: 0,
        verified_caller_files: 0,
        compatible_caller_files: 0,
        resolution: CallerVerificationStatus::VerifiedCompatible,
        verified_candidates: Vec::new(),
        ambiguous_candidates: Vec::new(),
    }
}

/// Aggregation pins (invariant `CBM-VERIFY-001`): counters fold, target
/// identity and arity survive only while every folded target agrees, and the
/// surface resolution is the weakest link across independently verified targets.
#[test]
fn aggregate_folds_targets_without_inventing_identity() {
    let mut first = summary("A.First", 3, 4);
    first.target_explicit_arity = Some(2);
    let mut second = summary("B.Second", 1, 2);
    second.target_explicit_arity = Some(3);

    let mut folded = CallerVerificationSummary::empty();
    folded.fold(&first);
    assert_eq!(
        folded.target_qualified_name.as_deref(),
        Some("A.First"),
        "a batch of one keeps its target identity"
    );
    assert_eq!(folded.target_explicit_arity, Some(2));
    assert_eq!(folded.raw_candidates, 7);

    folded.fold(&second);
    assert_eq!(folded.target_qualified_name, None);
    assert_eq!(folded.target_explicit_arity, None);
    assert_eq!(folded.verified, 4);
    assert_eq!(folded.rejected_arity, 6);

    assert_eq!(
        aggregate_resolution(&[
            CallerVerificationStatus::VerifiedCompatible,
            CallerVerificationStatus::RejectedArityMismatch
        ]),
        CallerVerificationStatus::VerifiedCompatible
    );
    assert_eq!(
        aggregate_resolution(&[
            CallerVerificationStatus::VerifiedCompatible,
            CallerVerificationStatus::Unverifiable
        ]),
        CallerVerificationStatus::Unverifiable
    );
    assert_eq!(
        aggregate_resolution(&[CallerVerificationStatus::Ambiguous]),
        CallerVerificationStatus::Ambiguous
    );
    assert_eq!(
        aggregate_resolution(&[]),
        CallerVerificationStatus::Unverifiable
    );
}

// ── RED-CV1..CV4, CV6, CV10, CV11: retained candidate identity ───────────────

/// RED-CV1 — the verified candidate's identity and file survive verification.
///
/// Three CBM candidates, one of which actually calls the target. The result must
/// name that candidate (and the file it was read from), not merely report
/// `verified = 1`.
#[test]
fn red_cv1_verified_identity_is_retained() {
    let target = "class Example { void Foo(int x) {} }";
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Example.Foo"),
        &[
            source("fileA.cs", "class A { void M() { Bar(1); } }"),
            source("fileB.cs", "class B { void M() { Foo(1); } }"),
            source("fileC.cs", "class C { void M() { Baz(1); } }"),
        ],
    );
    assert_eq!(result.verified, 1);
    assert_eq!(result.verified_candidates.len(), 1);
    let verified = &result.verified_candidates[0];
    assert_eq!(verified.cbm_file, "fileB.cs");
    assert_eq!(verified.file, "fileB.cs");
    assert_eq!(verified.argument_counts, vec![1]);
    assert_eq!(
        verified.status,
        CallerVerificationStatus::VerifiedCompatible
    );
}

/// RED-CV2 — a rejected candidate never appears as verified.
///
/// The candidate that calls the target with the wrong argument count is counted
/// as rejected and is absent from the verified identities.
#[test]
fn red_cv2_rejected_identity_is_not_verified() {
    let target = "class Example { void Foo(int x) {} }";
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Example.Foo"),
        &[
            source("fileA.cs", "class A { void M() { Foo(1, 2); } }"),
            source("fileB.cs", "class B { void M() { Foo(1); } }"),
        ],
    );
    assert_eq!((result.verified, result.rejected_arity), (1, 1));
    let verified_files: Vec<&str> = result
        .verified_candidates
        .iter()
        .map(|candidate| candidate.file.as_str())
        .collect();
    assert_eq!(verified_files, vec!["fileB.cs"]);
}

/// RED-CV3 — same-name/same-arity ambiguity stays ambiguous and is never
/// reported as a unique resolution.
#[test]
fn red_cv3_ambiguity_is_preserved_with_evidence() {
    let target = "class Example { void Foo(int x) {} void Foo(string x) {} }";
    let calls = "class Use { void M(dynamic value) { Foo(value); } }";
    let anchored = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Example.Foo").with_declared_arity(1),
        &[source("Use.cs", calls)],
    );
    assert_eq!((anchored.ambiguous, anchored.verified), (1, 0));
    assert!(anchored.verified_candidates.is_empty());
    assert_eq!(anchored.ambiguous_candidates.len(), 1);
    assert_eq!(
        anchored.ambiguous_candidates[0].status,
        CallerVerificationStatus::Ambiguous
    );
    assert_eq!(anchored.ambiguous_candidates[0].file, "Use.cs");
    assert_eq!(anchored.ambiguous_candidates[0].argument_counts, vec![1]);

    // The unanchored form (no declared arity, several declarations) is the
    // ambiguity path that never claims a unique candidate either.
    let unanchored = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Example.Foo"),
        &[source("Use.cs", calls)],
    );
    assert_eq!(
        unanchored.resolution,
        CallerVerificationStatus::Ambiguous,
        "several declarations with no arity basis stay ambiguous"
    );
    assert!(unanchored.verified_candidates.is_empty());
    assert_eq!(unanchored.ambiguous_candidates.len(), 1);
}
/// RED-CV4 — the established extension-method case keeps its evidence: the
/// receiver is excluded, the explicit argument count is retained, and the
/// verified candidate's identity is surfaced.
#[test]
fn red_cv4_extension_receiver_evidence_is_preserved() {
    let target = r#"static class CustomOrdering {
    public static IOrderedQueryable<T> OrderBy<T, TKey>(
        this IQueryable<T> source,
        Expression<Func<T, TKey>> keySelector,
        ListSortDirection sortOrder) => throw null;
}"#;
    let calls = r#"class Use { void M(IQueryable<Person> query, ListSortDirection direction) {
    query.OrderBy(x => x.Name);
    query.OrderBy(x => x.Id, direction);
} }"#;
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("CustomOrdering.OrderBy"),
        &[source("Use.cs", calls)],
    );
    assert_eq!(result.target_explicit_arity, Some(2), "receiver excluded");
    assert_eq!((result.verified, result.rejected_arity), (1, 1));
    assert_eq!(result.verified_candidates.len(), 1);
    assert_eq!(result.verified_candidates[0].file, "Use.cs");
    assert_eq!(
        result.verified_candidates[0].argument_counts,
        vec![2],
        "the retained evidence is the explicit argument count"
    );
}

/// RED-CV6 — every established counter keeps its meaning beside the new
/// evidence, including the per-candidate file counts.
#[test]
fn red_cv6_counts_are_preserved() {
    let target = "class Example { void Foo(int x) {} }";
    let result = verify_csharp_callers(
        target,
        CsharpTargetSelector::new("Example.Foo"),
        &[
            source("fileA.cs", "class A { void M() { Foo(1); Foo(2); } }"),
            source("fileB.cs", "class B { void M() { Foo(1, 2); } }"),
        ],
    );
    assert_eq!(result.raw_candidates, 3);
    assert_eq!(result.verified, 2);
    assert_eq!(result.rejected_arity, 1);
    assert_eq!(result.ambiguous, 0);
    assert_eq!(result.unverifiable, 0);
    assert_eq!(result.verified_caller_files, 1);
    assert_eq!(result.compatible_caller_files, 1);
    assert_eq!(
        result.resolution,
        CallerVerificationStatus::VerifiedCompatible
    );
    assert_eq!(result.verified_candidates.len(), 1);
    assert_eq!(
        result.verified_candidates[0].argument_counts,
        vec![1],
        "two accepted calls with the same argument count are one evidence entry"
    );

    // Distinct accepted counts are retained as a sorted set. A flexible shape is
    // compatible-but-ambiguous by design, so its evidence lands there; the three
    // call shapes below are the ones the existing `params` pin (`RED-CBM-A6`)
    // covers, so the counts are derived from established behaviour rather than
    // re-deriving the parameter arithmetic.
    let flexible = verify_csharp_callers(
        "class Example { void Foo(int x, params string[] values) {} }",
        CsharpTargetSelector::new("Example.Foo"),
        &[source(
            "Use.cs",
            r#"class Use { void M() { Foo(1); Foo(1, "a"); Foo(1, "a", "b"); } }"#,
        )],
    );
    assert_eq!((flexible.rejected_arity, flexible.ambiguous), (0, 3));
    assert!(
        flexible.verified_candidates.is_empty(),
        "a flexible shape never resolves a unique candidate"
    );
    assert_eq!(flexible.ambiguous_candidates.len(), 1);
    assert_eq!(
        flexible.ambiguous_candidates[0].argument_counts,
        vec![1, 2, 3],
        "distinct accepted counts are retained, sorted"
    );
}

/// RED-CV10 — one batch parses a referenced file once.
///
/// Two verification attempts inside the same batch (the shape a broad
/// `search_graph` produces) share one memo, so the second attempt re-parses
/// nothing while reaching the same verdict.
#[test]
fn red_cv10_one_batch_parses_a_referenced_file_once() {
    let target = "class Example { void Foo(int x) {} }";
    let calls = "class Use { void M() { Foo(1); } }";
    let mut memo = ParseMemo::new();

    let first = verify_with_memo(
        target,
        "target.cs",
        CsharpTargetSelector::new("Example.Foo"),
        &[source("Use.cs", calls)],
        &mut memo,
    );
    assert_eq!(memo.parses(), 2, "target + candidate, once each");
    let second = verify_with_memo(
        target,
        "target.cs",
        CsharpTargetSelector::new("Example.Foo"),
        &[source("Use.cs", calls)],
        &mut memo,
    );
    assert_eq!(
        memo.parses(),
        2,
        "the second attempt in the batch re-parses nothing"
    );
    assert_eq!(first.verified_candidates, second.verified_candidates);
    assert_eq!(second.verified_candidates.len(), 1);
    assert_eq!(second.verified_candidates[0].file, "Use.cs");
}

/// RED-CV11 — distinct files are still verified independently; memoization
/// never collapses two different paths into one representation.
#[test]
fn red_cv11_separate_files_are_verified_independently() {
    let target = "class Example { void Foo(int x) {} }";
    let mut memo = ParseMemo::new();
    let result = verify_with_memo(
        target,
        "target.cs",
        CsharpTargetSelector::new("Example.Foo"),
        &[
            source("fileA.cs", "class A { void M() { Foo(1); } }"),
            source("fileB.cs", "class B { void M() { Foo(2); } }"),
        ],
        &mut memo,
    );
    assert_eq!(result.verified, 2);
    assert_eq!(memo.parses(), 3, "target plus one parse per distinct file");
    let files: Vec<&str> = result
        .verified_candidates
        .iter()
        .map(|candidate| candidate.file.as_str())
        .collect();
    assert_eq!(files, vec!["fileA.cs", "fileB.cs"]);
}
