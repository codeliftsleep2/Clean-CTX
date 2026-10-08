//! CargoCheck-specific semantic compilation.
//!
//! Prepared invocations execute inside an OS-owned boundary; raw producer
//! bytes remain internal to bounded capture and semantic compilation.

mod authority;
mod budget;
mod cli;
mod environment;
mod evidence;
mod execution;
mod invocation;
mod model;
mod parser;
mod policy;
mod presentation;
mod sanitize;
mod startup;

pub use authority::{
    ApprovedCargoExecutable, ApprovedWorkspaceRoot, AuthorityError, AuthoritySource, DisplayPath,
    FileIdentity, PathClassification,
};
pub use budget::BoundedResultError;
pub use cli::{
    CargoCheckCliOptions, CargoCheckCliReport, CliDisposition, cli_disposition, run_cargo_check_cli,
};
pub use environment::{CargoCheckEnvironment, EnvironmentFacts};
pub use execution::{
    CancellationSource, CaptureFacts, CargoCheckCancellation, CargoCheckExecution, CleanupFacts,
    ExecutionError, OwnershipFacts, ProcessOutcome, StreamCaptureFacts, execute_cargo_check,
};
pub use invocation::{CargoCheckInvocation, CargoCheckRequest, InvocationFacts};
pub use model::{
    CargoCheckSemanticResult, CargoDiagnostic, CargoEvidence, CausalPreview, ChildDiagnostic,
    DetailCounts, DiagnosticLevel, EvidenceCategory, EvidenceFacts, EvidenceItem, ParserCoverage,
    RetentionFacts, SanitizedSpan, StructuredBudgetFacts, Suggestion, TransformationFacts,
};
pub use parser::CargoCheckCompiler;
pub use policy::CargoCheckPolicy;
pub use presentation::{CargoCheckProjection, TextBudgetFacts, project_cargo_check};
pub use startup::{CargoCheckStartupOptions, prepare_cargo_check_startup};

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_authority.rs"]
mod authority_tests;

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_invocation.rs"]
mod invocation_tests;

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_result_budget.rs"]
mod result_budget_tests;
