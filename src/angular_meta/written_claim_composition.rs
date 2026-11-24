//! Gated claim composition. No canonical/index/public/persistent consumer.
use crate::ir::source_observations::direct_identity::{
    self, Declaration, Envelopes, Evidence, Obligation,
};
#[path = "resolver_evidence/mod.rs"]
pub mod execution;
use crate::ir::source_observations::request_budget::{Request, Stop};
use crate::ir::source_observations::{Admitted, MappingObservation, Span, WrittenExpression};
use std::path::Path;
use std::path::PathBuf;

/// Borrowed claim records remain bound to their prepared source/proof snapshot.
/// They are not canonical registration or persistent/public transport records.
pub struct ClaimRecords<'a> {
    pub rule_version: u32,
    pub source: &'a Path,
    pub source_hash: &'a str,
    pub written: Option<&'a MappingObservation>,
    pub runtime_value_endpoint: Option<&'a Declaration>,
    pub type_endpoint: Option<&'a Declaration>,
    pub annotation: Option<Span>,
    pub classification: Classification,
    pub validation: (Validation, Validation, Validation),
    pub direct_obligations: &'a [Obligation],
    pub role_obligations: &'a [Obligation],
    pub direct_acquisition_error: Option<&'a Obligation>,
    pub role_acquisition_error: Option<&'a Obligation>,
}

pub struct AngularRole {
    evidence: Evidence,
}
impl AngularRole {
    pub fn acquire(
        request: &Request,
        source: &Admitted,
        index: usize,
        roots: &[PathBuf],
        envelopes: Envelopes,
    ) -> Result<Self, Obligation> {
        Ok(Self {
            evidence: direct_identity::capture_kind(
                request,
                source,
                index,
                roots,
                envelopes,
                true,
                Some(("@angular/router", "Routes")),
            )?,
        })
    }
    pub fn evidence(&self) -> &Evidence {
        &self.evidence
    }
    /// This only recognizes already captured public-entry/type evidence.
    pub fn direct_routes_role(&self) -> bool {
        let Some(id) = self.evidence.direct_target() else {
            return false;
        };
        self.evidence.type_space()
            && self.evidence.annotation().is_some()
            && id.name == "Routes"
            && matches!(
                id.kind.as_str(),
                "type_alias_declaration" | "interface_declaration"
            )
            && self
                .evidence
                .edges()
                .iter()
                .any(|e| e.specifier == "@angular/router")
            && self.evidence.manifests().iter().any(|m| {
                m.name == "@angular/router"
                    && m.version.as_ref().is_some_and(|v| !v.is_empty())
                    && m.path.parent().is_some_and(|p| id.source.starts_with(p))
                    && !self.evidence.manifests().iter().any(|other| {
                        other.path != m.path
                            && other.path.parent().is_some_and(|p| {
                                id.source.starts_with(p)
                                    && m.path.parent().is_some_and(|outer| p.starts_with(outer))
                            })
                    })
            })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Validation {
    Current,
    Stale,
    Incompatible,
    Missing,
    Unvalidated(Stop),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationClass {
    WrittenSourceObservation,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectClass {
    ModuleLocalValueDeclaration,
    QualifiedDeclarationEndpoint,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpretation {
    StronglySupported,
    Partial,
    Unknown,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    Open,
    ClosedSupportedWrittenSource,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchConclusion {
    Observed,
    NotObservedUnknownOpenCoverage,
    AbsentInClosedSupportedWrittenSource,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unestablished {
    NotEstablished,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Classification {
    pub composition_compatible: bool,
    pub observation: ObservationClass,
    pub direct: DirectClass,
    pub interpretation: Interpretation,
    pub angular_api_role: bool,
    pub known_contextual_contributor: bool,
    pub coverage: Coverage,
    pub search: SearchConclusion,
    pub javascript_implementation: Unestablished,
    pub actual_execution: Unestablished,
    pub canonical_registration: Unestablished,
}
/// Validated inputs and their explicit boundaries. Classification itself performs no I/O.
pub struct Prepared<'a> {
    direct_error: Option<Obligation>,
    role_error: Option<Obligation>,
    composition_compatible: bool,
    source: &'a Admitted,
    index: usize,
    direct: Option<Evidence>,
    role: Option<AngularRole>,
    source_validation: Validation,
    direct_validation: Validation,
    role_validation: Validation,
}
fn validated(
    result: Result<crate::ir::source_observations::request_budget::Reserved<bool>, Stop>,
) -> Validation {
    match result {
        Ok(value) if value.completed_after_stop.is_some() => {
            Validation::Unvalidated(value.completed_after_stop.expect("stop"))
        }
        Ok(value) if *value.value() => Validation::Current,
        Ok(_) => Validation::Stale,
        Err(stop) => Validation::Unvalidated(stop),
    }
}
impl<'a> Prepared<'a> {
    /// Retain precise optional acquisition failures rather than converting them
    /// into absence or a generic not-requested state.
    pub fn admit_results(
        request: &Request,
        source: &'a Admitted,
        index: usize,
        direct: Result<Evidence, Obligation>,
        role: Result<AngularRole, Obligation>,
    ) -> Self {
        let (direct, direct_error) = match direct {
            Ok(e) => (Some(e), None),
            Err(e) => (None, Some(e)),
        };
        let (role, role_error) = match role {
            Ok(e) => (Some(e), None),
            Err(e) => (None, Some(e)),
        };
        let mut result = Self::admit(request, source, index, direct, role);
        result.direct_error = direct_error;
        result.role_error = role_error;
        result
    }
    /// None denotes no acquired component; use admit_results for failed attempts.
    pub fn admit(
        request: &Request,
        source: &'a Admitted,
        index: usize,
        direct: Option<Evidence>,
        role: Option<AngularRole>,
    ) -> Self {
        let source_validation = validated(request.validate(source));
        Self::assemble(source, index, direct, role, source_validation, |e| {
            validated(e.validate(request))
        })
    }
    pub(crate) fn admit_at(
        boundary: &mut crate::ir::source_observations::bounded::Boundary<'_>,
        source: &'a Admitted,
        index: usize,
        direct: Option<Evidence>,
        role: Option<AngularRole>,
    ) -> Self {
        let source_validation = validated(
            boundary.pre_reserved(crate::ir::source_observations::bounded::witnesses(source)),
        );
        Self::assemble(source, index, direct, role, source_validation, |e| {
            validated(boundary.pre_reserved(e.witnesses()))
        })
    }
    fn assemble(
        source: &'a Admitted,
        index: usize,
        direct: Option<Evidence>,
        role: Option<AngularRole>,
        source_validation: Validation,
        mut validate: impl FnMut(&Evidence) -> Validation,
    ) -> Self {
        let record = source.observations().get(index);
        let matches = |e: &Evidence| {
            e.source() == source.source()
                && e.source_hash() == source.source_hash()
                && record.is_some_and(|r| r.mapping_pair == e.mapping_span())
        };
        let direct_validation = direct.as_ref().map_or(Validation::Missing, |e| {
            if !e.type_space() && matches(e) {
                validate(e)
            } else {
                Validation::Incompatible
            }
        });
        let role_validation = role.as_ref().map_or(Validation::Missing, |r| {
            if matches(r.evidence()) {
                validate(r.evidence())
            } else {
                Validation::Incompatible
            }
        });
        let composition_compatible = direct
            .as_ref()
            .zip(role.as_ref())
            .is_some_and(|(d, r)| d.same_scope(r.evidence()));
        Self {
            direct_error: None,
            role_error: None,
            composition_compatible,
            source,
            index,
            direct,
            role,
            source_validation,
            direct_validation,
            role_validation,
        }
    }
    pub fn observation(&self) -> Option<&MappingObservation> {
        self.source.observations().get(self.index)
    }
    pub fn source(&self) -> &Admitted {
        self.source
    }
    pub fn direct_evidence(&self) -> Option<&Evidence> {
        self.direct.as_ref()
    }
    pub fn role_evidence(&self) -> Option<&AngularRole> {
        self.role.as_ref()
    }
    pub fn validation(&self) -> (Validation, Validation, Validation) {
        (
            self.source_validation,
            self.direct_validation,
            self.role_validation,
        )
    }
    pub fn direct_target(&self) -> Option<&Declaration> {
        (self.source_validation == Validation::Current
            && self.direct_validation == Validation::Current)
            .then(|| self.direct.as_ref().and_then(|e| e.direct_target()))
            .flatten()
    }
    /// Closed scope is strictly this exact source's supported written-shape scan.
    /// Never a workspace, Angular, runtime, or unsupported-syntax absence proof.
    pub fn classify(&self, close_written_source: bool) -> Classification {
        let current = self.source_validation == Validation::Current;
        let present = self.observation().is_some() && current;
        let direct = if self.direct_target().is_none() {
            DirectClass::Unknown
        } else if self
            .direct
            .as_ref()
            .is_some_and(|d| d.module_local_binding_established())
        {
            DirectClass::ModuleLocalValueDeclaration
        } else {
            DirectClass::QualifiedDeclarationEndpoint
        };
        let role = self.role_validation == Validation::Current
            && self.role.as_ref().is_some_and(|r| r.direct_routes_role());
        let contributor = |e: &Evidence| {
            e.contextual_modules().iter().any(|name| {
                name == "@angular/router" || e.edges().iter().any(|edge| &edge.specifier == name)
            })
        };
        let known = (self.direct_validation == Validation::Current
            && self.direct.as_ref().is_some_and(contributor))
            || (self.role_validation == Validation::Current
                && self
                    .role
                    .as_ref()
                    .is_some_and(|r| contributor(r.evidence())));
        let only_qualifications = |e: &Evidence| {
            e.obligations()
                .iter()
                .all(|o| *o == Obligation::ContextualContributorsUnchecked)
        };
        let strong = self.composition_compatible
            && present
            && direct != DirectClass::Unknown
            && role
            && !known
            && self.direct.as_ref().is_some_and(only_qualifications)
            && self
                .role
                .as_ref()
                .is_some_and(|r| only_qualifications(r.evidence()));
        let unsupported = self
            .observation()
            .is_some_and(|o| matches!(o.expression, WrittenExpression::Unsupported { .. }));
        let interpretation = if !present {
            Interpretation::Unknown
        } else if unsupported {
            Interpretation::Unsupported
        } else if strong {
            Interpretation::StronglySupported
        } else {
            Interpretation::Partial
        };
        let coverage = if close_written_source && current && self.source.omissions().is_empty() {
            Coverage::ClosedSupportedWrittenSource
        } else {
            Coverage::Open
        };
        let search = if current && !self.source.observations().is_empty() {
            SearchConclusion::Observed
        } else if coverage == Coverage::ClosedSupportedWrittenSource {
            SearchConclusion::AbsentInClosedSupportedWrittenSource
        } else {
            SearchConclusion::NotObservedUnknownOpenCoverage
        };
        Classification {
            composition_compatible: self.composition_compatible,
            observation: if present {
                ObservationClass::WrittenSourceObservation
            } else {
                ObservationClass::Unknown
            },
            direct,
            interpretation,
            angular_api_role: role && !known,
            known_contextual_contributor: known,
            coverage,
            search,
            javascript_implementation: Unestablished::NotEstablished,
            actual_execution: Unestablished::NotEstablished,
            canonical_registration: Unestablished::NotEstablished,
        }
    }
    pub fn annotation_span(&self) -> Option<Span> {
        self.role.as_ref().and_then(|r| r.evidence().annotation())
    }
    /// Pure views: no reads, validation, acquisition, or fresh-confidence claim.
    pub fn records(&self, close_written_source: bool) -> ClaimRecords<'_> {
        ClaimRecords {
            rule_version: 1,
            source: self.source.source(),
            source_hash: self.source.source_hash(),
            written: self.observation(),
            runtime_value_endpoint: self.direct_target(),
            type_endpoint: if self.role_validation == Validation::Current {
                self.role
                    .as_ref()
                    .and_then(|r| r.evidence().direct_target())
            } else {
                None
            },
            annotation: self.annotation_span(),
            classification: self.classify(close_written_source),
            validation: self.validation(),
            direct_obligations: self.direct.as_ref().map_or(&[], |d| d.obligations()),
            role_obligations: self
                .role
                .as_ref()
                .map_or(&[], |r| r.evidence().obligations()),
            direct_acquisition_error: self.direct_error.as_ref(),
            role_acquisition_error: self.role_error.as_ref(),
        }
    }
}

#[cfg(test)]
#[path = "../tests/ir/written_claim_composition_e5.rs"]
mod tests;
