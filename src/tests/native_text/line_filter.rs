use super::*;

const POLICY: LineFilterPolicy = LineFilterPolicy {
    filter_id: "test-filter-v1",
    disclosure_label: "test-filter",
    max_output_lines: 4,
};

#[test]
fn owns_accounting_disclosure_and_repeat_protection() {
    let input = "keep\nnoise\nkeep too";
    let result = filter_lines(input, POLICY, |line| line == "noise");
    let facts = result.facts.as_ref().unwrap();
    assert_eq!(facts.filter_id, "test-filter-v1");
    assert_eq!(facts.original_bytes, input.len());
    assert_eq!(facts.reduced_bytes, result.text.len());
    assert_eq!(facts.original_lines, 3);
    assert_eq!(facts.reduced_lines, 3);
    assert_eq!(facts.reduction_kind, ReductionKind::NoiseRemoval);
    assert_eq!(result.text.matches("§FILTERED test-filter:").count(), 1);
    assert_eq!(
        filter_lines(&result.text, POLICY, |_| true).text,
        result.text
    );
}

#[test]
fn applies_total_line_bound_without_collapse() {
    let result = filter_lines("one\ntwo\nthree\nfour\nfive", POLICY, |_| false);
    let facts = result.facts.unwrap();
    assert_eq!(result.text.lines().count(), 4);
    assert!(facts.truncated);
    assert!(!facts.collapsed);
    assert_eq!(facts.reduction_kind, ReductionKind::Truncation);
}

#[test]
fn preserves_unchanged_text_exactly() {
    let input = "one\ntwo\nthree";
    let result = filter_lines(input, POLICY, |_| false);
    assert_eq!(result.text, input);
    assert!(result.facts.is_none());
}

#[test]
fn anchored_truncation_reserves_tail_and_preserves_original_order() {
    let result = filter_lines_with_tail(
        "head one\nhead two\nhead three\nsummary one\nsummary two",
        POLICY,
        |_| false,
        |line| line.starts_with("summary"),
    );
    assert_eq!(
        result.text,
        "head one\nsummary one\nsummary two\n§FILTERED test-filter: 5 → 3 lines"
    );
    assert!(result.facts.unwrap().truncated);
}
