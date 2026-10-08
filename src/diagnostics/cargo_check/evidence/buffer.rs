//! Byte-bounded fragments. Head and tail are separate ranges in a sanitized
//! record, never concatenated across a discarded middle or across records.
use super::super::model::EvidenceItem;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Anomaly {
    OverLimit,
    AdmissionCut,
    Unterminated,
    DecodingUnavailable,
    InvalidUtf8,
    MalformedJson,
}

impl Anomaly {
    pub const COUNT: usize = 6;
    pub fn label(self) -> &'static str {
        match self {
            Self::OverLimit => "over_limit",
            Self::AdmissionCut => "admission_cut",
            Self::Unterminated => "unterminated",
            Self::DecodingUnavailable => "decoding_unavailable",
            Self::InvalidUtf8 => "invalid_utf8",
            Self::MalformedJson => "malformed_json",
        }
    }
}

pub(super) struct Buffer {
    head: Vec<EvidenceItem>,
    tail: VecDeque<EvidenceItem>,
    exemplars: BTreeMap<Anomaly, EvidenceItem>,
    head_budget: usize,
    tail_budget: usize,
    exemplar_budget: usize,
    head_bytes: usize,
    tail_bytes: usize,
    head_closed: bool,
}

impl Buffer {
    pub fn new(head_budget: usize, tail_budget: usize, exemplar_budget: usize) -> Self {
        Self {
            head: Vec::new(),
            tail: VecDeque::new(),
            exemplars: BTreeMap::new(),
            head_budget,
            tail_budget,
            exemplar_budget,
            head_bytes: 0,
            tail_bytes: 0,
            head_closed: false,
        }
    }

    pub fn observe(&mut self, mut record: EvidenceItem, anomaly: Option<Anomaly>) {
        let mut start = 0;
        if let Some(anomaly) = anomaly {
            if self.exemplar_budget != 0 && !self.exemplars.contains_key(&anomaly) {
                let end = prefix_end(&record.text, self.exemplar_budget / Anomaly::COUNT);
                if end != 0 {
                    self.exemplars
                        .insert(anomaly, fragment(&record, 0, end, "exemplar"));
                    start = end;
                }
            }
        }
        if !self.head_closed {
            let remaining = self.head_budget.saturating_sub(self.head_bytes);
            let end = start + prefix_end(&record.text[start..], remaining);
            if end > start {
                self.head_bytes += end - start;
                self.head.push(fragment(&record, start, end, "head"));
            }
            self.head_closed = end < record.text.len() || self.head_bytes == self.head_budget;
            start = end;
        }
        if start < record.text.len() && self.tail_budget != 0 {
            let tail_start = start.max(suffix_start(&record.text, self.tail_budget));
            // Move a bounded suffix out of the current record rather than retain
            // an oversized String capacity behind a short text fragment.
            let text = record.text[tail_start..].to_owned();
            if text.is_empty() {
                return;
            }
            record.text = text;
            record.sanitized_start = tail_start;
            record.sanitized_end = record.sanitized_record_bytes;
            record.selection = "tail";
            self.tail_bytes += record.text.len();
            self.tail.push_back(record);
            self.trim_tail();
        }
    }

    fn trim_tail(&mut self) {
        while self.tail_bytes > self.tail_budget {
            let excess = self.tail_bytes - self.tail_budget;
            let Some(front) = self.tail.front_mut() else {
                break;
            };
            if front.text.len() <= excess {
                self.tail_bytes -= front.text.len();
                self.tail.pop_front();
            } else {
                let mut cut = excess;
                while !front.text.is_char_boundary(cut) {
                    cut += 1;
                }
                front.text = front.text[cut..].to_owned();
                front.sanitized_start += cut;
                self.tail_bytes -= cut;
                if front.text.is_empty() {
                    self.tail.pop_front();
                }
            }
        }
    }

    pub fn finish(self) -> Vec<EvidenceItem> {
        self.head
            .into_iter()
            .chain(self.tail)
            .chain(self.exemplars.into_values())
            .collect()
    }
}

fn fragment(
    record: &EvidenceItem,
    start: usize,
    end: usize,
    selection: &'static str,
) -> EvidenceItem {
    EvidenceItem {
        category: record.category,
        text: record.text[start..end].to_owned(),
        producer_order: record.producer_order,
        selection,
        observation: record.observation,
        anomaly: record.anomaly,
        source_bytes: record.source_bytes,
        source_withheld: record.source_withheld,
        sanitized_record_bytes: record.sanitized_record_bytes,
        sanitized_start: start,
        sanitized_end: end,
    }
}

pub(super) fn prefix_end(text: &str, maximum: usize) -> usize {
    let mut end = text.len().min(maximum);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    end
}

fn suffix_start(text: &str, maximum: usize) -> usize {
    let mut start = text.len().saturating_sub(maximum);
    while !text.is_char_boundary(start) {
        start += 1;
    }
    start
}
