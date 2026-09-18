use super::IdentityError;

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

pub(super) fn require_non_empty_payload(
    operation: &'static str,
    values: &[String],
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
