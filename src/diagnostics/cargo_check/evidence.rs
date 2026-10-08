use super::model::{
    EvidenceCategory, EvidenceCategoryFacts, EvidenceFacts, EvidenceItem, TransformationFacts,
};
use super::sanitize::sanitize;
use std::collections::BTreeMap;

const STDERR_BUDGET: usize = 48 * 1024;
const STDOUT_NON_JSON_BUDGET: usize = 16 * 1024;
const ANOMALY_BUDGET: usize = 16 * 1024;
const UNKNOWN_BUDGET: usize = 8 * 1024;
const MISMATCH_BUDGET: usize = 8 * 1024;

pub(crate) struct EvidenceCollector {
    items: Vec<EvidenceItem>,
    facts: BTreeMap<EvidenceCategory, EvidenceCategoryFacts>,
    used: BTreeMap<EvidenceCategory, usize>,
    order: u64,
}

impl EvidenceCollector {
    pub(crate) fn new() -> Self {
        Self {
            items: Vec::new(),
            facts: BTreeMap::new(),
            used: BTreeMap::new(),
            order: 0,
        }
    }

    pub(crate) fn observe(
        &mut self,
        category: EvidenceCategory,
        raw: &str,
        transformations: &mut TransformationFacts,
    ) {
        self.order = self.order.saturating_add(1);
        let original_bytes = raw.len();
        let budget = category_budget(category);
        let used = self.used.entry(category).or_default();
        let facts = self.facts.entry(category).or_default();
        facts.original_records += 1;
        facts.original_bytes += original_bytes;

        if *used >= budget {
            facts.omitted_records += 1;
            facts.omitted_bytes += original_bytes;
            facts.limit_activated = true;
            return;
        }

        let sanitized = sanitize(raw, transformations);
        let remaining = budget - *used;
        let retained = truncate_utf8_head_tail(&sanitized, remaining);
        let retained_bytes = retained.len();
        *used += retained_bytes;
        facts.retained_records += 1;
        facts.retained_bytes += retained_bytes;
        if retained_bytes < sanitized.len() {
            facts.omitted_bytes += sanitized.len() - retained_bytes;
            facts.limit_activated = true;
        }
        self.items.push(EvidenceItem {
            category,
            text: retained,
            producer_order: self.order,
        });
    }

    pub(crate) fn finish(self, total_budget: usize) -> (Vec<EvidenceItem>, EvidenceFacts) {
        let retained_bytes = self.items.iter().map(|item| item.text.len()).sum();
        (
            self.items,
            EvidenceFacts {
                categories: self.facts,
                total_budget_bytes: total_budget,
                retained_bytes,
            },
        )
    }

    /// Preserve one complete late stderr sample without enlarging the approved
    /// category budget. Earlier stderr retains the head half; the terminal
    /// sample gets the tail half. Call only once after that pipe is drained.
    pub(crate) fn observe_terminal_stderr(
        &mut self,
        raw: &str,
        transformations: &mut TransformationFacts,
    ) {
        if raw.is_empty() {
            return;
        }
        let category = EvidenceCategory::Stderr;
        let sanitized = sanitize(raw, transformations);
        let tail = truncate_utf8_head_tail(&sanitized, STDERR_BUDGET / 2);
        let mut remaining_head = STDERR_BUDGET / 2;
        let facts = self.facts.entry(category).or_default();
        self.items.retain_mut(|item| {
            if item.category != category {
                return true;
            }
            let old = item.text.len();
            if remaining_head == 0 {
                facts.retained_records = facts.retained_records.saturating_sub(1);
                facts.omitted_records += 1;
                facts.omitted_bytes += old;
                return false;
            }
            let mut end = old.min(remaining_head);
            while !item.text.is_char_boundary(end) {
                end -= 1;
            }
            item.text.truncate(end);
            remaining_head -= end;
            facts.omitted_bytes += old - end;
            true
        });
        self.order = self.order.saturating_add(1);
        facts.original_records += 1;
        facts.original_bytes += raw.len();
        facts.retained_records += 1;
        facts.omitted_bytes += sanitized.len().saturating_sub(tail.len());
        facts.retained_bytes = self
            .items
            .iter()
            .filter(|item| item.category == category)
            .map(|item| item.text.len())
            .sum::<usize>()
            + tail.len();
        facts.limit_activated = true;
        self.used.insert(category, facts.retained_bytes);
        self.items.push(EvidenceItem {
            category,
            text: tail,
            producer_order: self.order,
        });
    }
}

fn category_budget(category: EvidenceCategory) -> usize {
    match category {
        EvidenceCategory::Stderr => STDERR_BUDGET,
        EvidenceCategory::StdoutNonJson => STDOUT_NON_JSON_BUDGET,
        EvidenceCategory::MalformedOrTruncated => ANOMALY_BUDGET,
        EvidenceCategory::UnknownStructured => UNKNOWN_BUDGET,
        EvidenceCategory::AuthorityMismatch => MISMATCH_BUDGET,
    }
}

fn truncate_utf8_head_tail(value: &str, maximum: usize) -> String {
    if value.len() <= maximum {
        return value.to_owned();
    }
    if maximum == 0 {
        return String::new();
    }

    let head_target = maximum / 2;
    let tail_target = maximum - head_target;
    let mut head_end = head_target;
    while head_end > 0 && !value.is_char_boundary(head_end) {
        head_end -= 1;
    }
    let mut tail_start = value.len().saturating_sub(tail_target);
    while tail_start < value.len() && !value.is_char_boundary(tail_start) {
        tail_start += 1;
    }
    format!("{}{}", &value[..head_end], &value[tail_start..])
}
