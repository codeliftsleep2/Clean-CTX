//! CargoCheck-specific semantic compilation.
//!
//! Prepared invocations execute inside an OS-owned boundary; raw producer
//! bytes remain internal to bounded capture and semantic compilation.

mod authority;
mod environment;
mod evidence;
mod execution;
mod invocation;
mod model;
mod parser;
mod policy;
mod sanitize;

pub use authority::{
    ApprovedCargoExecutable, ApprovedWorkspaceRoot, AuthorityError, AuthoritySource, DisplayPath,
    FileIdentity, PathClassification,
};
pub use environment::{CargoCheckEnvironment, EnvironmentFacts};
pub use execution::{
    CancellationSource, CaptureFacts, CargoCheckCancellation, CargoCheckExecution, CleanupFacts,
    ExecutionError, OwnershipFacts, ProcessOutcome, StreamCaptureFacts, execute_cargo_check,
};
pub use invocation::{CargoCheckInvocation, CargoCheckRequest};
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

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_invocation.rs"]
mod invocation_tests;
