use super::IdentityKind;

/// A structured identity failure shared by validation and projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityError {
    DuplicateIdentity {
        operation: &'static str,
        kind: IdentityKind,
        id: String,
        owner: Option<String>,
        first_instruction: usize,
        duplicate_instruction: usize,
    },
    InvalidIdentity {
        operation: &'static str,
        kind: IdentityKind,
        id: String,
        owner: Option<String>,
        instruction: usize,
        reason: &'static str,
    },
    UnresolvedIdentity {
        operation: &'static str,
        expected: IdentityKind,
        id: String,
        instruction: usize,
    },
    KindMismatch {
        operation: &'static str,
        id: String,
        expected: IdentityKind,
        actual: IdentityKind,
        instruction: usize,
    },
    DuplicateReturn {
        method_id: String,
        first_instruction: usize,
        duplicate_instruction: usize,
    },
    DuplicateFieldType {
        field_id: String,
        first_instruction: usize,
        duplicate_instruction: usize,
    },
    DuplicateFact {
        operation: &'static str,
        target_kind: IdentityKind,
        target_id: String,
        first_instruction: usize,
        duplicate_instruction: usize,
    },
    InvalidOperation {
        operation: &'static str,
        instruction: usize,
        detail: String,
    },
    OwnerMismatch {
        operation: &'static str,
        id: String,
        expected_owner: String,
        instruction: usize,
    },
}

impl IdentityError {
    /// Stable machine-readable classification retained for MCP compatibility.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DuplicateIdentity { .. } => "ir_projection_duplicate_identity",
            Self::InvalidIdentity { .. } => "ir_projection_invalid_identity",
            Self::UnresolvedIdentity { .. } => "ir_projection_unresolved_identity",
            Self::KindMismatch { .. } => "ir_projection_kind_mismatch",
            Self::DuplicateReturn { .. } => "ir_projection_duplicate_return",
            Self::DuplicateFieldType { .. } => "ir_projection_duplicate_field_type",
            Self::DuplicateFact { .. } => "ir_projection_duplicate_fact",
            Self::InvalidOperation { .. } => "ir_projection_invalid_operation",
            Self::OwnerMismatch { .. } => "ir_projection_owner_mismatch",
        }
    }

    pub(crate) fn instruction(&self) -> usize {
        match self {
            Self::DuplicateIdentity {
                duplicate_instruction,
                ..
            }
            | Self::DuplicateReturn {
                duplicate_instruction,
                ..
            }
            | Self::DuplicateFieldType {
                duplicate_instruction,
                ..
            }
            | Self::DuplicateFact {
                duplicate_instruction,
                ..
            } => *duplicate_instruction,
            Self::InvalidIdentity { instruction, .. }
            | Self::UnresolvedIdentity { instruction, .. }
            | Self::KindMismatch { instruction, .. }
            | Self::InvalidOperation { instruction, .. }
            | Self::OwnerMismatch { instruction, .. } => *instruction,
        }
    }

    pub(crate) fn validation_code(&self) -> &'static str {
        let operation = match self {
            Self::InvalidIdentity { operation, .. }
            | Self::UnresolvedIdentity { operation, .. }
            | Self::KindMismatch { operation, .. }
            | Self::DuplicateFact { operation, .. }
            | Self::InvalidOperation { operation, .. }
            | Self::OwnerMismatch { operation, .. } => Some(*operation),
            Self::DuplicateIdentity { .. }
            | Self::DuplicateReturn { .. }
            | Self::DuplicateFieldType { .. } => None,
        };
        match operation {
            Some("RET") => "E001",
            Some("SIG") => "E002",
            Some("FLAGS") => "E003",
            Some("EXT") => "E004",
            Some("IMPL") => "E005",
            Some("INJECTS") => "E006",
            Some("DATAFLOW") => "E007",
            Some("CTRL") => "E008",
            Some("EFFECT") => "E009",
            Some("CTX") => "E010",
            Some("CALL") => "E011",
            _ => self.code(),
        }
    }
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateIdentity {
                operation,
                kind,
                id,
                owner,
                first_instruction,
                duplicate_instruction,
            } => {
                write!(f, "{operation} defines duplicate {kind} identity '{id}'")?;
                if let Some(owner) = owner {
                    write!(f, " under owner identity '{owner}'")?;
                }
                write!(
                    f,
                    " at instruction {duplicate_instruction} (first defined at {first_instruction})"
                )
            }
            Self::InvalidIdentity {
                operation,
                kind,
                id,
                owner,
                instruction,
                reason,
            } => {
                write!(
                    f,
                    "{operation} at instruction {instruction} has invalid {kind} identity '{id}'"
                )?;
                if let Some(owner) = owner {
                    write!(f, " under owner identity '{owner}'")?;
                }
                write!(f, ": {reason}")
            }
            Self::UnresolvedIdentity {
                operation,
                expected,
                id,
                instruction,
            } => write!(
                f,
                "{operation} at instruction {instruction} references unknown {expected} identity '{id}'"
            ),
            Self::KindMismatch {
                operation,
                id,
                expected,
                actual,
                instruction,
            } => write!(
                f,
                "{operation} at instruction {instruction} requires a {expected} identity, but '{id}' is a {actual} identity"
            ),
            Self::DuplicateReturn {
                method_id,
                first_instruction,
                duplicate_instruction,
            } => write!(
                f,
                "RET at instruction {duplicate_instruction} duplicates the return fact for method '{method_id}' (first defined at {first_instruction})"
            ),
            Self::DuplicateFieldType {
                field_id,
                first_instruction,
                duplicate_instruction,
            } => write!(
                f,
                "FIELD_T at instruction {duplicate_instruction} duplicates the type fact for field '{field_id}' (first defined at {first_instruction})"
            ),
            Self::DuplicateFact {
                operation,
                target_kind,
                target_id,
                first_instruction,
                duplicate_instruction,
            } => write!(
                f,
                "{operation} at instruction {duplicate_instruction} duplicates the singular fact for {target_kind} '{target_id}' (first defined at {first_instruction})"
            ),
            Self::InvalidOperation {
                operation,
                instruction,
                detail,
            } => write!(
                f,
                "{operation} at instruction {instruction} is invalid: {detail}"
            ),
            Self::OwnerMismatch {
                operation,
                id,
                expected_owner,
                instruction,
            } => write!(
                f,
                "{operation} at instruction {instruction} targets identity '{id}' outside declared owner '{expected_owner}'"
            ),
        }
    }
}

impl std::error::Error for IdentityError {}
