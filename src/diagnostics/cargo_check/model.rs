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
