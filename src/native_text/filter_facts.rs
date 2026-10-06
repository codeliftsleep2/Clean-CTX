use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReductionKind {
    NoiseRemoval,
    Truncation,
    NoiseRemovalAndTruncation,
    AlreadyFiltered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FilterFacts {
    pub filter_id: &'static str,
    pub original_bytes: usize,
    pub reduced_bytes: usize,
    pub original_lines: usize,
    pub reduced_lines: usize,
    pub reduction_kind: ReductionKind,
    pub truncated: bool,
    pub collapsed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredText {
    pub text: String,
    pub facts: Option<FilterFacts>,
}
