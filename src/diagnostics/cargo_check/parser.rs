mod framing;
mod json;
mod nested;
mod retention;
mod shape;
pub(crate) use framing::{CapturedFrame, ProducerStream};

use super::evidence::EvidenceCollector;
use super::model::{
    CargoCheckSemanticResult, CargoEvidence, DiagnosticLevel, EvidenceCategory, ParserCoverage,
    RetentionFacts, TransformationFacts,
};
use super::policy::CargoCheckPolicy;
use retention::Candidate;
use serde_json::Value;

pub struct CargoCheckCompiler {
    policy: CargoCheckPolicy,
    diagnostics: Vec<Candidate>,
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
            cargo_evidence: CargoEvidence::default(),
            coverage: ParserCoverage::default(),
            retention: RetentionFacts::default(),
            transformations: TransformationFacts::default(),
            evidence: EvidenceCollector::new(),
        }
    }

    pub fn finish(mut self) -> CargoCheckSemanticResult {
        let diagnostics = retention::finish(
            std::mem::take(&mut self.diagnostics),
            self.policy.maximum_diagnostics,
            &mut self.retention,
        );
        let (evidence, evidence_facts) = self.evidence.finish(self.policy.evidence_bytes);

        let mut result = CargoCheckSemanticResult {
            operation: "cargo_check",
            diagnostics,
            cargo_evidence: self.cargo_evidence,
            parser_coverage: self.coverage,
            retention: self.retention,
            transformations: self.transformations,
            evidence,
            evidence_facts,
            result_budget: Default::default(),
        };
        super::budget::initialize(&mut result, self.policy);
        let _ = super::budget::enforce(
            &mut result,
            self.policy.structured_content_bytes,
            super::budget::serialized_size,
        );
        result
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

        let identity = retention::identity(message);
        if retention::observe_repeat(&mut self.diagnostics, &identity) {
            return;
        }
        if !level.is_error() && level != DiagnosticLevel::Warning {
            // Approved policy keeps notes/help as children, not top-level competitors.
            return;
        }
        let diagnostic = nested::compile_diagnostic(
            message,
            level,
            text,
            self.policy,
            &mut self.transformations,
        );
        retention::admit(
            &mut self.diagnostics,
            Candidate {
                diagnostic,
                identity,
            },
            self.policy.maximum_diagnostics,
            &mut self.retention,
        );
    }
}

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_retention.rs"]
mod retention_tests;
