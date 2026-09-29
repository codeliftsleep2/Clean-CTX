use super::{ClassId, IdentityError, MethodId};

/// Typed internal target derived from the pattern schema.
///
/// `CoreOp::Pattern` retains its existing serialized string operands. This
/// type prevents consumers from reinterpreting those operands by ID shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PatternTarget {
    Class(ClassId),
    Method { class: ClassId, method: MethodId },
}

/// Parse the established pattern schemas without inspecting identifier text.
pub(crate) fn pattern_target(
    name: &str,
    args: &[String],
    instruction: usize,
) -> Result<PatternTarget, IdentityError> {
    if name.is_empty() {
        return Err(IdentityError::InvalidOperation {
            operation: "PAT",
            instruction,
            detail: "pattern name must not be empty".into(),
        });
    }

    let method_pattern = matches!(
        name,
        "CTOR" | "EMPTY_CTOR" | "OBSERVABLE" | "PROMISE" | "GETTER" | "SETTER" | "OVERRIDE"
    );
    let valid_arity = match name {
        "CTOR" => args.len() >= 2,
        "EMPTY_CTOR" | "OVERRIDE" => args.len() == 2,
        "OBSERVABLE" | "PROMISE" | "GETTER" | "SETTER" => args.len() == 3,
        _ => !args.is_empty(),
    };
    if !valid_arity {
        return Err(IdentityError::InvalidOperation {
            operation: "PAT",
            instruction,
            detail: format!("pattern '{name}' has invalid operand count {}", args.len()),
        });
    }

    let class = ClassId::from_serialized(&args[0]);
    if method_pattern {
        Ok(PatternTarget::Method {
            class,
            method: MethodId::from_serialized(&args[1]),
        })
    } else {
        Ok(PatternTarget::Class(class))
    }
}
