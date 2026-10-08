use super::{EvidenceCategory, EvidenceCollector, TransformationFacts};

#[test]
fn evidence_regression_late_stdout_cause_survives_category_exhaustion() {
    let mut collector = EvidenceCollector::new();
    let mut transformations = TransformationFacts::default();
    collector.observe(
        EvidenceCategory::StdoutNonJson,
        &"a".repeat(16 * 1024),
        &mut transformations,
    );
    collector.observe(
        EvidenceCategory::StdoutNonJson,
        "terminal stdout cause",
        &mut transformations,
    );
    let (items, _) = collector.finish(96 * 1024);
    assert!(
        items
            .iter()
            .any(|item| item.text.contains("terminal stdout cause"))
    );
}

#[test]
fn evidence_regression_stderr_preserves_approved_sixteen_thirtytwo_split() {
    let mut collector = EvidenceCollector::new();
    let mut transformations = TransformationFacts::default();
    collector.observe(
        EvidenceCategory::Stderr,
        &"h".repeat(48 * 1024),
        &mut transformations,
    );
    collector.observe_terminal_stderr(&"t".repeat(40 * 1024), &mut transformations);
    let (items, _) = collector.finish(96 * 1024);
    let head: usize = items
        .iter()
        .map(|item| item.text.bytes().filter(|byte| *byte == b'h').count())
        .sum();
    let tail: usize = items
        .iter()
        .map(|item| item.text.bytes().filter(|byte| *byte == b't').count())
        .sum();
    assert_eq!(head, 16 * 1024);
    assert_eq!(tail, 32 * 1024);
}

#[test]
fn evidence_regression_trimmed_unknown_record_keeps_true_source_length() {
    use crate::diagnostics::cargo_check::CargoCheckCompiler;
    let raw = b"   ordinary text   ";
    let mut compiler = CargoCheckCompiler::default();
    compiler.observe_stdout_frame(raw, true, false);
    let result = compiler.finish();
    assert_eq!(
        result.evidence_facts.categories[&EvidenceCategory::StdoutNonJson].original_bytes,
        raw.len()
    );
}

#[test]
fn evidence_regression_empty_sanitized_records_cannot_grow_item_storage() {
    let mut collector = EvidenceCollector::new();
    let mut transformations = TransformationFacts::default();
    for _ in 0..1000 {
        collector.observe(EvidenceCategory::Stderr, "\x1b[0m", &mut transformations);
    }
    let (items, _) = collector.finish(96 * 1024);
    assert!(items.is_empty());
}

#[test]
fn evidence_regression_one_frame_truncation_never_joins_nonadjacent_text() {
    let mut collector = EvidenceCollector::new();
    let mut transformations = TransformationFacts::default();
    let raw = format!("start{}finish", ".".repeat(32 * 1024));
    collector.observe(EvidenceCategory::StdoutNonJson, &raw, &mut transformations);
    let (items, _) = collector.finish(96 * 1024);
    assert!(
        items
            .iter()
            .all(|item| !(item.text.starts_with("start") && item.text.ends_with("finish")))
    );
}
