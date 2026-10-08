mod framing;
mod json;
mod shape;
pub(crate) use framing::{CapturedFrame, ProducerStream};

use super::evidence::EvidenceCollector;
use super::model::{
    CargoCheckSemanticResult, CargoDiagnostic, CargoEvidence, ChildDiagnostic, DiagnosticLevel,
    EvidenceCategory, ParserCoverage, RetentionFacts, SanitizedSpan, Suggestion,
    TransformationFacts,
};
use super::policy::CargoCheckPolicy;
use super::sanitize::sanitize;
use serde_json::Value;
use std::collections::BTreeMap;

pub struct CargoCheckCompiler {
    policy: CargoCheckPolicy,
    diagnostics: Vec<CargoDiagnostic>,
    diagnostic_repeats: BTreeMap<String, usize>,
    cargo_evidence: CargoEvidence,
    coverage: ParserCoverage,
    retention: RetentionFacts,
    transformations: TransformationFacts,
    evidence: EvidenceCollector,
}

impl Default for CargoCheckCompiler {
    fn default() -> Self {
        Self::new(CargoCheckPolicy::APPROVED)
    }
}

impl CargoCheckCompiler {
    pub fn new(policy: CargoCheckPolicy) -> Self {
        Self {
            policy,
            diagnostics: Vec::new(),
            diagnostic_repeats: BTreeMap::new(),
            cargo_evidence: CargoEvidence::default(),
            coverage: ParserCoverage::default(),
            retention: RetentionFacts::default(),
            transformations: TransformationFacts::default(),
            evidence: EvidenceCollector::new(),
        }
    }

    pub fn finish(mut self) -> CargoCheckSemanticResult {
        let diagnostics = select_diagnostics(
            std::mem::take(&mut self.diagnostics),
            self.policy.maximum_diagnostics,
        );
        self.retention.diagnostics_retained = diagnostics.len();
        self.retention.diagnostics_omitted = self
            .retention
            .diagnostics_seen
            .saturating_sub(self.retention.diagnostics_retained)
            .saturating_sub(self.retention.exact_repeats_collapsed);
        let (evidence, evidence_facts) = self.evidence.finish(self.policy.evidence_bytes);

        CargoCheckSemanticResult {
            operation: "cargo_check",
            diagnostics,
            cargo_evidence: self.cargo_evidence,
            parser_coverage: self.coverage,
            retention: self.retention,
            transformations: self.transformations,
            evidence,
            evidence_facts,
        }
    }

    fn observe_incompatible(&mut self, source_bytes: usize) {
        self.coverage.incompatible_structured += 1;
        self.evidence.observe_summary(
            EvidenceCategory::UnknownStructured,
            "[Cargo JSON record withheld: ambiguous fields or incompatible shape]",
            source_bytes,
            &mut self.transformations,
        );
    }

    fn observe_structured(&mut self, value: Value, source_bytes: usize) {
        let Some(object) = value.as_object() else {
            self.observe_incompatible(source_bytes);
            return;
        };
        let Some(reason) = object.get("reason").and_then(Value::as_str) else {
            self.observe_incompatible(source_bytes);
            return;
        };

        match reason {
            "compiler-message" => self.observe_compiler_message(object, source_bytes),
            "compiler-artifact" => {
                if !shape::artifact(object) {
                    self.observe_incompatible(source_bytes);
                    return;
                }
                self.coverage.compiler_artifacts += 1;
                self.cargo_evidence.artifact_records += 1;
            }
            "build-script-executed" => {
                if !shape::build_script(object) {
                    self.observe_incompatible(source_bytes);
                    return;
                }
                self.coverage.build_scripts += 1;
                self.cargo_evidence.build_script_records += 1;
            }
            "build-finished" => self.observe_build_finished(object, source_bytes),
            other => {
                self.coverage.unknown_structured += 1;
                let summary = format!("unknown Cargo record reason: {other}");
                self.evidence.observe_summary(
                    EvidenceCategory::UnknownStructured,
                    &summary,
                    source_bytes,
                    &mut self.transformations,
                );
            }
        }
    }

    fn observe_build_finished(
        &mut self,
        object: &serde_json::Map<String, Value>,
        source_bytes: usize,
    ) {
        let Some(success) = object.get("success").and_then(Value::as_bool) else {
            self.observe_incompatible(source_bytes);
            return;
        };
        self.coverage.build_finished += 1;
        if self
            .cargo_evidence
            .build_finished_success
            .is_some_and(|previous| previous != success)
        {
            self.cargo_evidence.conflicting_build_finished_records += 1;
            self.evidence.observe(
                EvidenceCategory::AuthorityMismatch,
                "conflicting Cargo build-finished records",
                &mut self.transformations,
            );
        }
        self.cargo_evidence.build_finished_success = Some(success);
    }

    fn observe_compiler_message(
        &mut self,
        object: &serde_json::Map<String, Value>,
        source_bytes: usize,
    ) {
        if !object.get("message").is_some_and(shape::diagnostic) {
            self.observe_incompatible(source_bytes);
            return;
        }
        let Some(message) = object.get("message").and_then(Value::as_object) else {
            self.observe_incompatible(source_bytes);
            return;
        };
        let Some(text) = message.get("message").and_then(Value::as_str) else {
            self.observe_incompatible(source_bytes);
            return;
        };
        let Some(level_text) = message.get("level").and_then(Value::as_str) else {
            self.observe_incompatible(source_bytes);
            return;
        };

        self.coverage.compiler_messages += 1;
        self.retention.diagnostics_seen += 1;
        let level = DiagnosticLevel::from_producer(level_text);
        if level.is_error() {
            self.retention.errors_seen += 1;
        } else if level == DiagnosticLevel::Warning {
            self.retention.warnings_seen += 1;
        } else {
            self.retention.other_seen += 1;
        }

        let diagnostic = self.compile_diagnostic(message, level, text);
        let identity = diagnostic_identity(&diagnostic);
        if let Some(index) = self.diagnostic_repeats.get(&identity).copied() {
            self.diagnostics[index].repeat_count += 1;
            self.retention.exact_repeats_collapsed += 1;
            return;
        }
        let index = self.diagnostics.len();
        self.diagnostic_repeats.insert(identity, index);
        self.diagnostics.push(diagnostic);
    }

    fn compile_diagnostic(
        &mut self,
        message: &serde_json::Map<String, Value>,
        level: DiagnosticLevel,
        text: &str,
    ) -> CargoDiagnostic {
        let mut primary = Vec::new();
        let mut related = Vec::new();
        let mut omitted_primary = 0;
        let mut omitted_related = 0;
        let mut suggestions = 0;
        let mut omitted_suggestions = 0;

        if let Some(spans) = message.get("spans").and_then(Value::as_array) {
            for span in spans {
                let is_primary = span
                    .get("is_primary")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let maximum = if is_primary {
                    self.policy.maximum_primary_spans
                } else {
                    self.policy.maximum_related_spans
                };
                let target = if is_primary {
                    &mut primary
                } else {
                    &mut related
                };
                if target.len() >= maximum {
                    if is_primary {
                        omitted_primary += 1;
                    } else {
                        omitted_related += 1;
                    }
                    continue;
                }
                let allow_suggestion = suggestions < self.policy.maximum_suggestions;
                let compiled = compile_span(span, allow_suggestion, &mut self.transformations);
                if compiled.suggestion.is_some() {
                    suggestions += 1;
                } else if span
                    .get("suggested_replacement")
                    .and_then(Value::as_str)
                    .is_some()
                {
                    omitted_suggestions += 1;
                }
                target.push(compiled);
            }
        }

        let children_values = message
            .get("children")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let selected_children = select_head_tail(children_values, self.policy.maximum_children, 6);
        let children = selected_children
            .into_iter()
            .filter_map(|child| compile_child(&child, &mut self.transformations))
            .collect();

        CargoDiagnostic {
            level,
            message: sanitize(text, &mut self.transformations),
            code: message
                .get("code")
                .and_then(|code| code.get("code"))
                .and_then(Value::as_str)
                .map(|code| sanitize(code, &mut self.transformations)),
            primary_spans: primary,
            related_spans: related,
            children,
            rendered_evidence: message
                .get("rendered")
                .and_then(Value::as_str)
                .map(|rendered| sanitize(rendered, &mut self.transformations)),
            omitted_primary_spans: omitted_primary,
            omitted_related_spans: omitted_related,
            omitted_children: children_values
                .len()
                .saturating_sub(self.policy.maximum_children),
            omitted_suggestions,
            repeat_count: 1,
        }
    }
}

fn compile_span(
    value: &Value,
    allow_suggestion: bool,
    transformations: &mut TransformationFacts,
) -> SanitizedSpan {
    SanitizedSpan {
        file: value
            .get("file_name")
            .and_then(Value::as_str)
            .map(|value| sanitize(value, transformations))
            .unwrap_or_else(|| "<unknown>".to_owned()),
        line_start: number(value, "line_start"),
        line_end: number(value, "line_end"),
        column_start: number(value, "column_start"),
        column_end: number(value, "column_end"),
        label: value
            .get("label")
            .and_then(Value::as_str)
            .map(|value| sanitize(value, transformations)),
        suggestion: allow_suggestion
            .then(|| value.get("suggested_replacement").and_then(Value::as_str))
            .flatten()
            .map(|replacement| Suggestion {
                replacement: sanitize(replacement, transformations),
                applicability: value
                    .get("suggestion_applicability")
                    .and_then(Value::as_str)
                    .map(|value| sanitize(value, transformations)),
            }),
    }
}

fn compile_child(
    value: &Value,
    transformations: &mut TransformationFacts,
) -> Option<ChildDiagnostic> {
    Some(ChildDiagnostic {
        level: DiagnosticLevel::from_producer(value.get("level")?.as_str()?),
        message: sanitize(value.get("message")?.as_str()?, transformations),
        spans: value
            .get("spans")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .take(8)
            .map(|span| compile_span(span, false, transformations))
            .collect(),
    })
}

fn number(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn diagnostic_identity(value: &CargoDiagnostic) -> String {
    let location = value.primary_spans.first();
    format!(
        "{:?}|{}|{}|{}|{}",
        value.level,
        value.code.as_deref().unwrap_or_default(),
        value.message,
        location.map(|span| span.file.as_str()).unwrap_or_default(),
        location.map(|span| span.line_start).unwrap_or_default(),
    )
}

fn select_diagnostics(values: Vec<CargoDiagnostic>, maximum: usize) -> Vec<CargoDiagnostic> {
    if values.len() <= maximum {
        return values;
    }
    let errors: Vec<_> = values
        .iter()
        .filter(|item| item.level.is_error())
        .cloned()
        .collect();
    let warnings: Vec<_> = values
        .iter()
        .filter(|item| item.level == DiagnosticLevel::Warning)
        .cloned()
        .collect();
    let other: Vec<_> = values
        .iter()
        .filter(|item| !item.level.is_error() && item.level != DiagnosticLevel::Warning)
        .cloned()
        .collect();

    let mut selected = select_head_tail(&errors, maximum, maximum.saturating_mul(3) / 4);
    let remaining = maximum.saturating_sub(selected.len());
    selected.extend(select_head_tail(
        &warnings,
        remaining,
        remaining.saturating_mul(3) / 4,
    ));
    let remaining = maximum.saturating_sub(selected.len());
    selected.extend(select_head_tail(
        &other,
        remaining,
        remaining.saturating_mul(3) / 4,
    ));
    selected
}

fn select_head_tail<T: Clone>(values: &[T], maximum: usize, preferred_head: usize) -> Vec<T> {
    if values.len() <= maximum {
        return values.to_vec();
    }
    let head = preferred_head.min(maximum);
    let tail = maximum - head;
    values[..head]
        .iter()
        .chain(values[values.len() - tail..].iter())
        .cloned()
        .collect()
}
