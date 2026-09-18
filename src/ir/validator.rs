// Structural validation for canonical IR.
//
// Approved class/method/field/signature identities are validated by the
// shared typed identity boundary. Existing rules for later operation families
// remain behaviorally preserved until their matrix rows become normative.

use super::compiler::CompiledIR;
use super::identity::{IdentityError, IdentityIndex, validate_identity_graph};
use super::opcodes::CoreOp;

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
        let identities = match validate_identity_graph(ir) {
            Ok(identities) => identities,
            Err(error) => return vec![error.into()],
        };

        validate_deferred_operation_contracts(ir, &identities)
    }
}

impl Default for DefaultValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Preserve the established E003-E011 checks for operation families whose
/// richer identity/cardinality contracts remain deferred. Every `CoreOp` is
/// named explicitly so a new variant cannot silently bypass validation review.
fn validate_deferred_operation_contracts(
    ir: &CompiledIR,
    identities: &IdentityIndex,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for (instruction, op) in ir.instructions.iter().enumerate() {
        match op {
            CoreOp::DefClass(..) => {}
            CoreOp::DefMethod(..) => {}
            CoreOp::DefField(..) => {}
            CoreOp::DefInterface(..) => {}
            CoreOp::Param(..) => {}
            CoreOp::Return(..) => {}
            CoreOp::FieldType(..) => {}
            CoreOp::Flags(method, _) => push_unknown_method(
                &mut errors,
                identities,
                method,
                instruction,
                "E003",
                "FLAGS",
                "method",
            ),
            CoreOp::ClassFlags(..) => {}
            CoreOp::Extends(class, _) => push_unknown_class(
                &mut errors,
                identities,
                class,
                instruction,
                "E004",
                "EXT",
                "child class",
            ),
            CoreOp::Implements(class, _) => push_unknown_class(
                &mut errors,
                identities,
                class,
                instruction,
                "E005",
                "IMPL",
                "class",
            ),
            CoreOp::Injects(class, _) => push_unknown_class(
                &mut errors,
                identities,
                class,
                instruction,
                "E006",
                "INJECTS",
                "class",
            ),
            CoreOp::Import(..) => {}
            CoreOp::TypeAlias(..) => {}
            CoreOp::Pattern(..) => {}
            CoreOp::Body(..) => {}
            CoreOp::DataFlow(method, ..) => push_unknown_method(
                &mut errors,
                identities,
                method,
                instruction,
                "E007",
                "DATAFLOW",
                "method",
            ),
            CoreOp::ControlFlow(method, ..) => push_unknown_method(
                &mut errors,
                identities,
                method,
                instruction,
                "E008",
                "CTRL",
                "method",
            ),
            CoreOp::SideEffect(method, _) => push_unknown_method(
                &mut errors,
                identities,
                method,
                instruction,
                "E009",
                "EFFECT",
                "method",
            ),
            CoreOp::ExecutionContext(method, _) => push_unknown_method(
                &mut errors,
                identities,
                method,
                instruction,
                "E010",
                "CTX",
                "method",
            ),
            CoreOp::Call(caller, ..) => push_unknown_method(
                &mut errors,
                identities,
                caller,
                instruction,
                "E011",
                "CALL",
                "caller method",
            ),
        }
    }

    errors
}

fn push_unknown_method(
    errors: &mut Vec<ValidationError>,
    identities: &IdentityIndex,
    method: &str,
    instruction: usize,
    code: &'static str,
    operation: &'static str,
    target_kind: &'static str,
) {
    if !identities.contains_method(method) {
        errors.push(ValidationError {
            code: code.to_owned(),
            message: format!("{operation} references unknown {target_kind} '{method}'"),
            instruction_index: Some(instruction),
        });
    }
}

fn push_unknown_class(
    errors: &mut Vec<ValidationError>,
    identities: &IdentityIndex,
    class: &str,
    instruction: usize,
    code: &'static str,
    operation: &'static str,
    target_kind: &'static str,
) {
    if !identities.contains_class(class) {
        errors.push(ValidationError {
            code: code.to_owned(),
            message: format!("{operation} references unknown {target_kind} '{class}'"),
            instruction_index: Some(instruction),
        });
    }
}

#[cfg(test)]
#[path = "../tests/ir/validator.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/ir/validator_identity.rs"]
mod identity_tests;
