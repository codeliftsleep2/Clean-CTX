//! CargoCheck-specific semantic compilation.
//!
//! This module does not execute Cargo. It converts already framed producer
//! evidence into a bounded, sanitized semantic result.

mod authority;
mod evidence;
mod model;
mod parser;
mod policy;
mod sanitize;

pub use authority::{
    ApprovedCargoExecutable, ApprovedWorkspaceRoot, AuthorityError, AuthoritySource, DisplayPath,
    FileIdentity, PathClassification,
};
pub use model::{
    CargoCheckSemanticResult, CargoDiagnostic, CargoEvidence, ChildDiagnostic, DiagnosticLevel,
    EvidenceCategory, EvidenceFacts, EvidenceItem, ParserCoverage, RetentionFacts, SanitizedSpan,
    Suggestion, TransformationFacts,
};
pub use parser::CargoCheckCompiler;
pub use policy::CargoCheckPolicy;

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_authority.rs"]
mod authority_tests;
