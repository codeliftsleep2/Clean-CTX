// Structural validation for canonical IR.
//
// Every `CoreOp` contract is enforced by the shared typed identity boundary.
// Projection consumes the same authority, so validation and projection cannot
// disagree about identity, ownership, cardinality, or payload validity.

use super::compiler::CompiledIR;
use super::identity::{IdentityError, validate_identity_graph};

/// Validates a `CompiledIR` against structural invariants.
pub trait IRValidator {
    fn validate(&self, ir: &CompiledIR) -> Vec<ValidationError>;
}

/// A stable validation diagnostic returned by the production validation pass.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub instruction_index: Option<usize>,
}

impl From<IdentityError> for ValidationError {
    fn from(error: IdentityError) -> Self {
        Self {
            code: error.validation_code().to_owned(),
            message: error.to_string(),
            instruction_index: Some(error.instruction()),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

/// The default structural validator used by the production pipeline.
pub struct DefaultValidator;

impl DefaultValidator {
    pub fn new() -> Self {
        Self
    }
}

impl IRValidator for DefaultValidator {
    fn validate(&self, ir: &CompiledIR) -> Vec<ValidationError> {
        validate_identity_graph(ir)
            .err()
            .map(ValidationError::from)
            .into_iter()
            .collect()
    }
}

impl Default for DefaultValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/ir/validator.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/ir/validator_identity.rs"]
mod identity_tests;

#[cfg(test)]
#[path = "../tests/ir/validator_phase4a.rs"]
mod phase4a_tests;
