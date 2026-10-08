use super::{Anomaly, EvidenceCategory, EvidenceCollector, TransformationFacts};

#[test]
fn category_reservations_are_independent_and_late_causes_remain_visible() {
    let categories = [
        EvidenceCategory::Stderr,
        EvidenceCategory::StdoutNonJson,
        EvidenceCategory::MalformedOrTruncated,
        EvidenceCategory::UnknownStructured,
        EvidenceCategory::AuthorityMismatch,
    ];
    let mut collector = EvidenceCollector::new();
    let mut transformations = TransformationFacts::default();
    for category in categories {
        for _ in 0..4 {
            collector.observe(category, &"noise".repeat(20 * 1024), &mut transformations);
        }
        collector.observe(category, "late cause", &mut transformations);
    }
    let (items, facts) = collector.finish(96 * 1024);
    assert!(facts.retained_bytes <= 96 * 1024);
    for category in categories {
        let category_facts = &facts.categories[&category];
        assert!(category_facts.retained_bytes <= category_facts.budget_bytes);
        assert_eq!(category_facts.original_records, 5);
        assert_eq!(
            category_facts.original_records,
            category_facts.retained_records
                + category_facts.omitted_records
                + category_facts.sanitized_empty_records
        );
        assert_eq!(
            category_facts.sanitized_bytes,
            category_facts.retained_bytes + category_facts.omitted_sanitized_bytes
        );
        assert_eq!(
            category_facts.retained_bytes,
            category_facts.head_bytes + category_facts.tail_bytes + category_facts.exemplar_bytes
        );
        assert_eq!(category_facts.borrowed_bytes, 0);
        assert!(
            items
                .iter()
                .any(|item| item.category == category && item.text.contains("late cause"))
        );
    }
}

#[test]
fn utf8_fragments_are_nonoverlapping_original_sanitized_ranges() {
    let raw = "初🙂末".repeat(10 * 1024);
    let mut collector = EvidenceCollector::new();
    collector.observe(
        EvidenceCategory::StdoutNonJson,
        &raw,
        &mut TransformationFacts::default(),
    );
    let (items, facts) = collector.finish(96 * 1024);
    assert_eq!(items.len(), 2);
    assert!(items[0].sanitized_end < items[1].sanitized_start);
    for item in &items {
        assert_eq!(item.text, raw[item.sanitized_start..item.sanitized_end]);
        assert_eq!(item.sanitized_record_bytes, raw.len());
        assert!(item.text.len() <= 8 * 1024);
    }
    let category = &facts.categories[&EvidenceCategory::StdoutNonJson];
    assert_eq!(category.retained_records, 1);
    assert_eq!(category.partially_retained_records, 1);
    assert_eq!(
        category.sanitized_bytes,
        category.retained_bytes + category.omitted_sanitized_bytes
    );
}

#[test]
fn shrinking_tail_preserves_latest_unicode_and_exact_fragment_ranges() {
    let mut collector = EvidenceCollector::new();
    let mut transformations = TransformationFacts::default();
    collector.observe(
        EvidenceCategory::StdoutNonJson,
        &"head".repeat(2 * 1024),
        &mut transformations,
    );
    let first = "🙂".repeat(2 * 1024);
    let second = format!("{}terminal", "界".repeat(2 * 1024));
    collector.observe(
        EvidenceCategory::StdoutNonJson,
        &first,
        &mut transformations,
    );
    collector.observe(
        EvidenceCategory::StdoutNonJson,
        &second,
        &mut transformations,
    );
    let (items, facts) = collector.finish(96 * 1024);
    for item in items.iter().filter(|item| item.selection == "tail") {
        let source = if item.producer_order == 2 {
            &first
        } else {
            &second
        };
        assert_eq!(item.text, source[item.sanitized_start..item.sanitized_end]);
    }
    assert!(items.last().unwrap().text.ends_with("terminal"));
    assert!(facts.categories[&EvidenceCategory::StdoutNonJson].tail_bytes <= 8 * 1024);
}

#[test]
fn rare_fault_exemplars_survive_repeated_fault_floods() {
    let mut collector = EvidenceCollector::new();
    let mut transformations = TransformationFacts::default();
    let kinds = [
        Anomaly::OverLimit,
        Anomaly::AdmissionCut,
        Anomaly::Unterminated,
        Anomaly::DecodingUnavailable,
        Anomaly::InvalidUtf8,
        Anomaly::MalformedJson,
    ];
    for kind in kinds {
        collector.observe_anomaly(kind, kind.label(), 100, &mut transformations);
    }
    for _ in 0..1000 {
        collector.observe_anomaly(
            Anomaly::MalformedJson,
            &"generic fault".repeat(100),
            100,
            &mut transformations,
        );
    }
    let (items, facts) = collector.finish(96 * 1024);
    for kind in kinds {
        assert!(
            items
                .iter()
                .any(|item| item.selection == "exemplar" && item.anomaly == Some(kind.label()))
        );
    }
    let category = &facts.categories[&EvidenceCategory::MalformedOrTruncated];
    assert_eq!(category.anomaly_records["malformed_json"], 1001);
    assert!(category.exemplar_bytes <= 8 * 1024);
    assert!(category.retained_bytes <= 16 * 1024);
    assert_eq!(category.withheld_source_bytes, 100600);
}

#[test]
fn redaction_occurs_before_any_head_tail_or_exemplar_cut() {
    let secret = "PRIVATE_CANARY_123456789";
    let raw = format!(
        "{} token={secret} {}",
        "a".repeat(8 * 1024 - 8),
        "z".repeat(20 * 1024)
    );
    let mut collector = EvidenceCollector::new();
    let mut transformations = TransformationFacts::default();
    collector.observe(EvidenceCategory::StdoutNonJson, &raw, &mut transformations);
    collector.observe_terminal_stderr(&format!("password={secret}"), &mut transformations);
    let (items, facts) = collector.finish(96 * 1024);
    let serialized = serde_json::to_string(&items).unwrap();
    assert!(!serialized.contains(secret));
    assert!(serialized.contains("REDACTED"));
    assert_eq!(
        facts.categories[&EvidenceCategory::StdoutNonJson].original_bytes,
        raw.len()
    );
    assert!(facts.categories[&EvidenceCategory::StdoutNonJson].sanitized_bytes < raw.len());
}

#[test]
fn normalization_to_empty_counts_records_without_retaining_storage() {
    let mut collector = EvidenceCollector::new();
    let mut transformations = TransformationFacts::default();
    for _ in 0..1000 {
        collector.observe(EvidenceCategory::Stderr, "\x1b[0m", &mut transformations);
    }
    let (items, facts) = collector.finish(96 * 1024);
    let category = &facts.categories[&EvidenceCategory::Stderr];
    assert!(items.is_empty());
    assert_eq!(category.original_bytes, 4000);
    assert_eq!(category.original_records, 1000);
    assert_eq!(category.sanitized_empty_records, 1000);
    assert_eq!(category.retained_records, 0);
    assert_eq!(category.omitted_records, 0);
    assert_eq!(category.sanitized_bytes, 0);
    assert_eq!(transformations.normalization.sequences_removed, 1000);
}

#[test]
fn invalid_terminal_sample_discloses_source_size_without_exporting_sample() {
    use crate::diagnostics::cargo_check::CargoCheckCompiler;
    let mut compiler = CargoCheckCompiler::default();
    let raw = b"PRIVATE_CANARY\xff";
    compiler.observe_stderr_terminal_sample(raw);
    let result = compiler.finish();
    let category = &result.evidence_facts.categories[&EvidenceCategory::Stderr];
    assert_eq!(category.original_bytes, raw.len());
    assert_eq!(category.withheld_source_bytes, raw.len());
    assert_eq!(result.parser_coverage.invalid_utf8_terminal_samples, 1);
    assert!(
        result
            .evidence
            .iter()
            .all(|item| item.source_withheld && item.observation == "terminal_sample")
    );
    assert!(
        !serde_json::to_string(&result)
            .unwrap()
            .contains("PRIVATE_CANARY")
    );
}

#[test]
fn smaller_explicit_shared_budget_is_enforced_and_omissions_recomputed() {
    let mut collector = EvidenceCollector::new();
    collector.observe(
        EvidenceCategory::Stderr,
        "abcdefgh",
        &mut TransformationFacts::default(),
    );
    let (items, facts) = collector.finish(3);
    assert_eq!(items[0].text, "abc");
    assert_eq!(facts.retained_bytes, 3);
    assert_eq!(
        facts.categories[&EvidenceCategory::Stderr].omitted_sanitized_bytes,
        5
    );
    assert_eq!(
        facts.categories[&EvidenceCategory::Stderr].partially_retained_records,
        1
    );
    assert!(facts.categories[&EvidenceCategory::Stderr].limit_activated);
}
