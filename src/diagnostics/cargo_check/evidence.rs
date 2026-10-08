mod buffer;
use super::model::{
    EvidenceCategory, EvidenceCategoryFacts, EvidenceFacts, EvidenceItem, TransformationFacts,
};
use super::sanitize::sanitize;
pub(crate) use buffer::Anomaly;
use buffer::{Buffer, prefix_end};
use std::collections::BTreeMap;

const CATEGORIES: [EvidenceCategory; 5] = [
    EvidenceCategory::Stderr,
    EvidenceCategory::StdoutNonJson,
    EvidenceCategory::MalformedOrTruncated,
    EvidenceCategory::UnknownStructured,
    EvidenceCategory::AuthorityMismatch,
];

struct RecordMetadata {
    source_bytes: usize,
    source_withheld: bool,
    observation: &'static str,
    anomaly: Option<Anomaly>,
}

pub(crate) struct EvidenceCollector {
    buffers: BTreeMap<EvidenceCategory, Buffer>,
    facts: BTreeMap<EvidenceCategory, EvidenceCategoryFacts>,
    order: u64,
}

impl EvidenceCollector {
    pub(crate) fn new() -> Self {
        let mut buffers = BTreeMap::new();
        let mut facts = BTreeMap::new();
        for category in CATEGORIES {
            let (head, tail, exemplars, strategy) = allocation(category);
            buffers.insert(category, Buffer::new(head, tail, exemplars));
            facts.insert(
                category,
                EvidenceCategoryFacts {
                    budget_bytes: head + tail + exemplars,
                    selection_policy: strategy,
                    ..EvidenceCategoryFacts::default()
                },
            );
        }
        Self {
            buffers,
            facts,
            order: 0,
        }
    }

    pub(crate) fn observe(
        &mut self,
        category: EvidenceCategory,
        raw: &str,
        transformations: &mut TransformationFacts,
    ) {
        self.observe_record(
            category,
            raw,
            RecordMetadata {
                source_bytes: raw.len(),
                source_withheld: false,
                observation: "record",
                anomaly: None,
            },
            transformations,
        );
    }

    pub(crate) fn observe_summary(
        &mut self,
        category: EvidenceCategory,
        summary: &str,
        source_bytes: usize,
        transformations: &mut TransformationFacts,
    ) {
        self.observe_record(
            category,
            summary,
            RecordMetadata {
                source_bytes,
                source_withheld: true,
                observation: "summary",
                anomaly: None,
            },
            transformations,
        );
    }

    pub(crate) fn observe_anomaly(
        &mut self,
        anomaly: Anomaly,
        summary: &str,
        source_bytes: usize,
        transformations: &mut TransformationFacts,
    ) {
        self.observe_record(
            EvidenceCategory::MalformedOrTruncated,
            summary,
            RecordMetadata {
                source_bytes,
                source_withheld: true,
                observation: "summary",
                anomaly: Some(anomaly),
            },
            transformations,
        );
    }

    pub(crate) fn observe_terminal_stderr(
        &mut self,
        raw: &str,
        transformations: &mut TransformationFacts,
    ) {
        self.observe_record(
            EvidenceCategory::Stderr,
            raw,
            RecordMetadata {
                source_bytes: raw.len(),
                source_withheld: false,
                observation: "terminal_sample",
                anomaly: None,
            },
            transformations,
        );
    }

    pub(crate) fn observe_terminal_summary(
        &mut self,
        summary: &str,
        source_bytes: usize,
        transformations: &mut TransformationFacts,
    ) {
        self.observe_record(
            EvidenceCategory::Stderr,
            summary,
            RecordMetadata {
                source_bytes,
                source_withheld: true,
                observation: "terminal_sample",
                anomaly: None,
            },
            transformations,
        );
    }

    fn observe_record(
        &mut self,
        category: EvidenceCategory,
        raw: &str,
        metadata: RecordMetadata,
        transformations: &mut TransformationFacts,
    ) {
        let RecordMetadata {
            source_bytes,
            source_withheld,
            observation,
            anomaly,
        } = metadata;
        self.order = self.order.saturating_add(1);
        let facts = self
            .facts
            .get_mut(&category)
            .expect("fixed evidence category");
        facts.original_records = facts.original_records.saturating_add(1);
        facts.original_bytes = facts.original_bytes.saturating_add(source_bytes);
        if source_withheld {
            facts.withheld_source_bytes = facts.withheld_source_bytes.saturating_add(source_bytes);
        }
        if let Some(anomaly) = anomaly {
            let count = facts.anomaly_records.entry(anomaly.label()).or_default();
            *count = count.saturating_add(1);
        }
        // Always sanitize before selection, including when a reservation is full:
        // a new tail or exemplar remains eligible and no partial raw prefix escapes.
        let sanitized = sanitize(raw, transformations);
        let length = sanitized.len();
        facts.sanitized_bytes = facts.sanitized_bytes.saturating_add(length);
        if length == 0 {
            facts.sanitized_empty_records = facts.sanitized_empty_records.saturating_add(1);
            return;
        }
        self.buffers
            .get_mut(&category)
            .expect("fixed evidence category")
            .observe(
                EvidenceItem {
                    category,
                    text: sanitized,
                    producer_order: self.order,
                    selection: "",
                    observation,
                    anomaly: anomaly.map(Anomaly::label),
                    source_bytes,
                    source_withheld,
                    sanitized_record_bytes: length,
                    sanitized_start: 0,
                    sanitized_end: length,
                },
                anomaly,
            );
    }

    pub(crate) fn finish(mut self, total_budget: usize) -> (Vec<EvidenceItem>, EvidenceFacts) {
        let mut items: Vec<_> = self
            .buffers
            .into_values()
            .flat_map(Buffer::finish)
            .collect();
        items.sort_by_key(|item| (item.producer_order, item.sanitized_start));
        // Approved reservations sum to 96 KiB. Honor a smaller explicit local
        // policy too; final structured-result enforcement is a later boundary.
        let mut remaining = total_budget;
        items.retain_mut(|item| {
            let end = prefix_end(&item.text, remaining);
            if end == 0 {
                return false;
            }
            item.text.truncate(end);
            item.sanitized_end = item.sanitized_start + end;
            remaining -= end;
            true
        });
        for category in CATEGORIES {
            let facts = self
                .facts
                .get_mut(&category)
                .expect("fixed evidence category");
            let selected: Vec<_> = items
                .iter()
                .filter(|item| item.category == category)
                .collect();
            let mut records: BTreeMap<u64, (usize, usize)> = BTreeMap::new();
            for item in &selected {
                let record = records
                    .entry(item.producer_order)
                    .or_insert((0, item.sanitized_record_bytes));
                record.0 += item.text.len();
            }
            facts.retained_records = records.len();
            facts.retained_bytes = selected.iter().map(|item| item.text.len()).sum();
            facts.omitted_records = facts
                .original_records
                .saturating_sub(facts.retained_records)
                .saturating_sub(facts.sanitized_empty_records);
            facts.omitted_sanitized_bytes =
                facts.sanitized_bytes.saturating_sub(facts.retained_bytes);
            // Compatibility aggregate; explicit byte-domain counters above are
            // the denominators. This is not original_bytes minus retained_bytes.
            facts.omitted_bytes = facts
                .withheld_source_bytes
                .saturating_add(facts.omitted_sanitized_bytes);
            facts.partially_retained_records = records
                .values()
                .filter(|(retained, original)| retained < original)
                .count();
            facts.head_bytes = selected
                .iter()
                .filter(|item| item.selection == "head")
                .map(|item| item.text.len())
                .sum();
            facts.tail_bytes = selected
                .iter()
                .filter(|item| item.selection == "tail")
                .map(|item| item.text.len())
                .sum();
            facts.exemplar_bytes = selected
                .iter()
                .filter(|item| item.selection == "exemplar")
                .map(|item| item.text.len())
                .sum();
            facts.limit_activated = facts.omitted_records != 0
                || facts.omitted_sanitized_bytes != 0
                || facts.withheld_source_bytes != 0;
        }
        let retained_bytes = items.iter().map(|item| item.text.len()).sum();
        (
            items,
            EvidenceFacts {
                categories: self.facts,
                total_budget_bytes: total_budget,
                retained_bytes,
            },
        )
    }
}

fn allocation(category: EvidenceCategory) -> (usize, usize, usize, &'static str) {
    match category {
        EvidenceCategory::Stderr => (16 * 1024, 32 * 1024, 0, "16_kib_head_32_kib_tail"),
        EvidenceCategory::StdoutNonJson => (8 * 1024, 8 * 1024, 0, "equal_head_tail"),
        EvidenceCategory::MalformedOrTruncated => (
            4 * 1024,
            4 * 1024,
            8 * 1024,
            "first_fault_exemplars_then_equal_head_tail",
        ),
        EvidenceCategory::UnknownStructured => (4 * 1024, 4 * 1024, 0, "summary_head_tail"),
        EvidenceCategory::AuthorityMismatch => {
            (4 * 1024, 4 * 1024, 0, "reserved_mismatch_head_tail")
        }
    }
}

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_evidence_accounting.rs"]
mod accounting_tests;

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_evidence.rs"]
mod tests;
