use crate::cbm::caller_verify::{
    CallerVerificationStatus, CallerVerificationSummary, CandidateSource, CsharpTargetSelector,
    aggregate_resolution, annotate_caller_evidence, verify_csharp_callers,
};
use serde_json::json;

fn source(file: &'static str, text: &'static str) -> CandidateSource<'static> {
    CandidateSource { file, text }
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
