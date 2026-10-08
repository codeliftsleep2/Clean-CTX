use crate::native_text::{ansi::NormalizationFacts, redaction::RedactionFacts};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
    FailureNote,
    Ice,
    Unknown,
}

impl DiagnosticLevel {
    pub(crate) fn from_producer(value: &str) -> Self {
        match value {
            "error" => Self::Error,
            "warning" => Self::Warning,
            "note" => Self::Note,
            "help" => Self::Help,
            "failure-note" => Self::FailureNote,
            "ice" => Self::Ice,
            _ => Self::Unknown,
        }
    }

    pub(crate) fn is_error(self) -> bool {
        matches!(self, Self::Error | Self::FailureNote | Self::Ice)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Suggestion {
    pub replacement: String,
    pub applicability: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SanitizedSpan {
    pub file: String,
    pub line_start: u64,
    pub line_end: u64,
    pub column_start: u64,
    pub column_end: u64,
    pub label: Option<String>,
    pub suggestion: Option<Suggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChildDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub spans: Vec<SanitizedSpan>,
    pub spans_seen: usize,
    pub omitted_spans: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CargoDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub code: Option<String>,
    pub primary_spans: Vec<SanitizedSpan>,
    pub related_spans: Vec<SanitizedSpan>,
    pub children: Vec<ChildDiagnostic>,
    pub rendered_evidence: Option<String>,
    pub omitted_primary_spans: usize,
    pub omitted_related_spans: usize,
    pub omitted_children: usize,
    pub omitted_suggestions: usize,
    pub primary_spans_seen: usize,
    pub related_spans_seen: usize,
    pub children_seen: usize,
    pub children_head_retained: usize,
    pub children_tail_retained: usize,
    pub suggestions_seen: usize,
    pub suggestions_retained: usize,
    pub distinct_suggestions_retained: usize,
    pub suggestion_bytes_seen: usize,
    pub omitted_suggestion_bytes: usize,
    pub machine_applicable_suggestions_seen: usize,
    pub machine_applicable_suggestions_retained: usize,
    pub repeat_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCategory {
    Stderr,
    StdoutNonJson,
    MalformedOrTruncated,
    UnknownStructured,
    AuthorityMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvidenceItem {
    pub category: EvidenceCategory,
    pub text: String,
    pub producer_order: u64,
    pub selection: &'static str,
    pub observation: &'static str,
    pub anomaly: Option<&'static str>,
    pub source_bytes: usize,
    pub source_withheld: bool,
    pub sanitized_record_bytes: usize,
    pub sanitized_start: usize,
    pub sanitized_end: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct EvidenceCategoryFacts {
    pub original_records: usize,
    pub original_bytes: usize,
    pub retained_records: usize,
    pub retained_bytes: usize,
    pub omitted_records: usize,
    pub omitted_bytes: usize,
    pub limit_activated: bool,
    pub budget_bytes: usize,
    pub borrowed_bytes: usize,
    pub selection_policy: &'static str,
    pub sanitized_bytes: usize,
    pub omitted_sanitized_bytes: usize,
    pub withheld_source_bytes: usize,
    pub sanitized_empty_records: usize,
    pub partially_retained_records: usize,
    pub head_bytes: usize,
    pub tail_bytes: usize,
    pub exemplar_bytes: usize,
    pub anomaly_records: BTreeMap<&'static str, usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct EvidenceFacts {
    pub categories: BTreeMap<EvidenceCategory, EvidenceCategoryFacts>,
    pub total_budget_bytes: usize,
    pub retained_bytes: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ParserCoverage {
    pub stdout_frames: usize,
    pub stderr_frames: usize,
    pub compiler_messages: usize,
    pub compiler_artifacts: usize,
    pub build_scripts: usize,
    pub build_finished: usize,
    pub unknown_structured: usize,
    pub incompatible_structured: usize,
    pub malformed_json: usize,
    pub non_json_stdout: usize,
    pub truncated_frames: usize,
    pub over_limit_frames: usize,
    pub invalid_utf8_frames: usize,
    pub empty_stdout_frames: usize,
    pub empty_stderr_frames: usize,
    pub admission_cut_frames: usize,
    pub decoding_unavailable_frames: usize,
    pub unparsed_stdout_frames: usize,
    pub unparsed_stderr_frames: usize,
    pub unparsed_stdout_bytes: u64,
    pub unparsed_stderr_bytes: u64,
    pub json_candidates: usize,
    pub parsed_json_objects: usize,
    pub duplicate_json_fields: usize,
    pub stderr_terminal_samples: usize,
    pub invalid_utf8_terminal_samples: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CargoEvidence {
    pub build_finished_success: Option<bool>,
    pub artifact_records: usize,
    pub build_script_records: usize,
    pub conflicting_build_finished_records: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RetentionFacts {
    pub diagnostics_seen: usize,
    pub diagnostics_retained: usize,
    pub diagnostics_omitted: usize,
    pub errors_seen: usize,
    pub warnings_seen: usize,
    pub other_seen: usize,
    pub exact_repeats_collapsed: usize,
    pub errors_retained: usize,
    pub warnings_retained: usize,
    pub errors_omitted: usize,
    pub warnings_omitted: usize,
    pub other_omitted: usize,
    pub head_retained: usize,
    pub tail_retained: usize,
    pub peak_candidates: usize,
    pub selection_policy: &'static str,
    pub repeat_scope: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TransformationFacts {
    pub normalization: NormalizationFacts,
    pub redaction: RedactionFacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CargoCheckSemanticResult {
    pub operation: &'static str,
    pub diagnostics: Vec<CargoDiagnostic>,
    pub cargo_evidence: CargoEvidence,
    pub parser_coverage: ParserCoverage,
    pub retention: RetentionFacts,
    pub transformations: TransformationFacts,
    pub evidence: Vec<EvidenceItem>,
    pub evidence_facts: EvidenceFacts,
}
