use crate::native_text::{
    ansi::NormalizationFacts, filter_facts::FilterFacts, redaction::RedactionFacts,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingOutcome {
    Replaced,
    PassedThrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PassThroughReason {
    UnsupportedEvent,
    UnsupportedTool,
    UnsupportedInput,
    UnsupportedSchema,
    Interrupted,
    ImageResult,
    InvalidEnvelope,
    TransformError,
    Unchanged,
    ReconstructionFailed,
    ValidationFailed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct FieldFacts {
    pub field: &'static str,
    pub normalization: NormalizationFacts,
    pub redaction: RedactionFacts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<FilterFacts>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaudeNativeFacts {
    pub outcome: ProcessingOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<&'static str>,
    pub adapter_id: &'static str,
    pub adapter_version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_id: Option<&'static str>,
    pub validation_succeeded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass_through_reason: Option<PassThroughReason>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldFacts>,
    pub duration_micros: u128,
}

impl ClaudeNativeFacts {
    pub(super) fn passed(reason: PassThroughReason, duration_micros: u128) -> Self {
        Self {
            outcome: ProcessingOutcome::PassedThrough,
            tool: None,
            adapter_id: "claude-post-tool-use",
            adapter_version: 1,
            schema_id: None,
            validation_succeeded: false,
            pass_through_reason: Some(reason),
            fields: Vec::new(),
            duration_micros,
        }
    }
}
