use super::IdentityError;
use crate::ir::opcodes::{ControlSummary, DeclarationModifier, PatternFact};

pub(super) fn reject_typed_flag_payload(
    operation: &'static str,
    values: &[String],
    instruction: usize,
) -> Result<(), IdentityError> {
    if let Some(value) = values.iter().find(|value| {
        DeclarationModifier::from_serialized(value).is_some()
            || ControlSummary::from_serialized(value).is_some()
            || PatternFact::is_serialized_kind(value)
    }) {
        return Err(IdentityError::InvalidOperation {
            operation,
            instruction,
            detail: format!("typed value '{value}' must use its semantic-family operation"),
        });
    }
    Ok(())
}

pub(super) fn validate_body_span(
    start: Option<u64>,
    end: Option<u64>,
    instruction: usize,
) -> Result<(), IdentityError> {
    match (start, end) {
        (None, None) => Ok(()),
        (Some(start), Some(end)) if start <= end => Ok(()),
        (Some(_), Some(_)) => Err(IdentityError::InvalidOperation {
            operation: "BODY",
            instruction,
            detail: "body span start exceeds end".into(),
        }),
        _ => Err(IdentityError::InvalidOperation {
            operation: "BODY",
            instruction,
            detail: "body span start and end must both be present or both be absent".into(),
        }),
    }
}

pub(super) fn require_non_empty_payload<T>(
    operation: &'static str,
    values: &[T],
    instruction: usize,
) -> Result<(), IdentityError> {
    if values.is_empty() {
        return Err(IdentityError::InvalidOperation {
            operation,
            instruction,
            detail: "payload must not be empty".into(),
        });
    }
    Ok(())
}

pub(super) fn require_vocabulary(
    operation: &'static str,
    field: &'static str,
    value: &str,
    accepted: &[&str],
    instruction: usize,
) -> Result<(), IdentityError> {
    if !accepted.contains(&value) {
        return Err(IdentityError::InvalidOperation {
            operation,
            instruction,
            detail: format!("unknown {field} '{value}'"),
        });
    }
    Ok(())
}
