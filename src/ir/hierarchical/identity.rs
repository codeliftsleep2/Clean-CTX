// Stable semantic identity validation for hierarchical projection.
//
// `CoreOp` remains the serialized DTO during this migration, so its operands
// stay strings on the wire. This module is the checked internal boundary that
// converts the approved first-slice identities into distinct Rust types before
// projection may use them.

use super::{CompiledIR, CoreOp, HierarchicalProjectionError, ProjectionIdentityKind};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ClassId(String);

impl ClassId {
    pub(super) fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct MethodId(String);

impl MethodId {
    pub(super) fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParameterId(String);

impl ParameterId {
    fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone)]
pub(super) struct FirstSliceIdentities {
    pub(super) classes: HashMap<ClassId, usize>,
    pub(super) methods: HashMap<MethodId, usize>,
}

/// Validate the complete first-slice identity graph without using instruction
/// position as semantic authority.
pub(super) fn validate_first_slice(
    ir: &CompiledIR,
) -> Result<FirstSliceIdentities, HierarchicalProjectionError> {
    let mut classes = HashMap::new();
    let mut methods = HashMap::new();
    let mut parameters = HashMap::new();
    let mut returns = HashMap::new();

    for (instruction, op) in ir.instructions.iter().enumerate() {
        match op {
            CoreOp::DefClass(raw_id, _) => {
                require_non_empty(
                    "DEF_C",
                    ProjectionIdentityKind::Class,
                    raw_id,
                    None,
                    instruction,
                )?;
                let id = ClassId::from_serialized(raw_id);
                if let Some(first_instruction) = classes.insert(id.clone(), instruction) {
                    return Err(HierarchicalProjectionError::DuplicateIdentity {
                        operation: "DEF_C",
                        kind: ProjectionIdentityKind::Class,
                        id: raw_id.clone(),
                        owner: None,
                        first_instruction,
                        duplicate_instruction: instruction,
                    });
                }
            }
            CoreOp::DefMethod(raw_owner, raw_id, _) => {
                require_non_empty(
                    "DEF_M owner",
                    ProjectionIdentityKind::Class,
                    raw_owner,
                    None,
                    instruction,
                )?;
                require_non_empty(
                    "DEF_M",
                    ProjectionIdentityKind::Method,
                    raw_id,
                    Some(raw_owner.as_str()),
                    instruction,
                )?;
                let id = MethodId::from_serialized(raw_id);
                if let Some(first_instruction) = methods.insert(id, instruction) {
                    return Err(HierarchicalProjectionError::DuplicateIdentity {
                        operation: "DEF_M",
                        kind: ProjectionIdentityKind::Method,
                        id: raw_id.clone(),
                        owner: Some(raw_owner.clone()),
                        first_instruction,
                        duplicate_instruction: instruction,
                    });
                }
            }
            CoreOp::Param(raw_method, raw_parameter, _, _) => {
                require_non_empty(
                    "SIG target",
                    ProjectionIdentityKind::Method,
                    raw_method,
                    None,
                    instruction,
                )?;
                require_non_empty(
                    "SIG",
                    ProjectionIdentityKind::Parameter,
                    raw_parameter,
                    Some(raw_method.as_str()),
                    instruction,
                )?;
                let key = (
                    MethodId::from_serialized(raw_method),
                    ParameterId::from_serialized(raw_parameter),
                );
                if let Some(first_instruction) = parameters.insert(key, instruction) {
                    return Err(HierarchicalProjectionError::DuplicateIdentity {
                        operation: "SIG",
                        kind: ProjectionIdentityKind::Parameter,
                        id: raw_parameter.clone(),
                        owner: Some(raw_method.clone()),
                        first_instruction,
                        duplicate_instruction: instruction,
                    });
                }
            }
            CoreOp::Return(raw_method, _) => {
                require_non_empty(
                    "RET",
                    ProjectionIdentityKind::Method,
                    raw_method,
                    None,
                    instruction,
                )?;
                let method = MethodId::from_serialized(raw_method);
                if let Some(first_instruction) = returns.insert(method, instruction) {
                    return Err(HierarchicalProjectionError::DuplicateReturn {
                        method_id: raw_method.clone(),
                        first_instruction,
                        duplicate_instruction: instruction,
                    });
                }
            }
            CoreOp::DefField(..)
            | CoreOp::DefInterface(..)
            | CoreOp::FieldType(..)
            | CoreOp::Flags(..)
            | CoreOp::ClassFlags(..)
            | CoreOp::Extends(..)
            | CoreOp::Implements(..)
            | CoreOp::Injects(..)
            | CoreOp::Import(..)
            | CoreOp::TypeAlias(..)
            | CoreOp::Pattern(..)
            | CoreOp::Body(..)
            | CoreOp::DataFlow(..)
            | CoreOp::ControlFlow(..)
            | CoreOp::SideEffect(..)
            | CoreOp::ExecutionContext(..)
            | CoreOp::Call(..) => {}
        }
    }

    for (instruction, op) in ir.instructions.iter().enumerate() {
        let CoreOp::DefMethod(raw_owner, raw_method, _) = op else {
            continue;
        };
        let owner = ClassId::from_serialized(raw_owner);
        let method_id = MethodId::from_serialized(raw_method);
        if !classes.contains_key(&owner) {
            return Err(unresolved_or_wrong_kind(
                "DEF_M owner",
                ProjectionIdentityKind::Class,
                raw_owner,
                instruction,
                &classes,
                &methods,
            ));
        }
        if classes.contains_key(&ClassId::from_serialized(method_id.as_str())) {
            return Err(HierarchicalProjectionError::KindMismatch {
                operation: "DEF_M identity",
                id: method_id.as_str().to_owned(),
                expected: ProjectionIdentityKind::Method,
                actual: ProjectionIdentityKind::Class,
                instruction,
            });
        }
    }

    for (instruction, op) in ir.instructions.iter().enumerate() {
        let (operation, raw_method) = match op {
            CoreOp::Param(method, ..) => ("SIG", method),
            CoreOp::Return(method, _) => ("RET", method),
            _ => continue,
        };
        let method = MethodId::from_serialized(raw_method);
        if !methods.contains_key(&method) {
            return Err(unresolved_or_wrong_kind(
                operation,
                ProjectionIdentityKind::Method,
                raw_method,
                instruction,
                &classes,
                &methods,
            ));
        }
    }

    Ok(FirstSliceIdentities { classes, methods })
}

fn require_non_empty(
    operation: &'static str,
    kind: ProjectionIdentityKind,
    raw_id: &str,
    owner: Option<&str>,
    instruction: usize,
) -> Result<(), HierarchicalProjectionError> {
    if raw_id.is_empty() {
        return Err(HierarchicalProjectionError::InvalidIdentity {
            operation,
            kind,
            id: raw_id.to_owned(),
            owner: owner.map(str::to_owned),
            instruction,
            reason: "identity must not be empty",
        });
    }
    Ok(())
}

fn unresolved_or_wrong_kind(
    operation: &'static str,
    expected: ProjectionIdentityKind,
    raw_id: &str,
    instruction: usize,
    classes: &HashMap<ClassId, usize>,
    methods: &HashMap<MethodId, usize>,
) -> HierarchicalProjectionError {
    let actual = match expected {
        ProjectionIdentityKind::Class
            if methods.contains_key(&MethodId::from_serialized(raw_id)) =>
        {
            Some(ProjectionIdentityKind::Method)
        }
        ProjectionIdentityKind::Method
            if classes.contains_key(&ClassId::from_serialized(raw_id)) =>
        {
            Some(ProjectionIdentityKind::Class)
        }
        _ => None,
    };

    if let Some(actual) = actual {
        HierarchicalProjectionError::KindMismatch {
            operation,
            id: raw_id.to_owned(),
            expected,
            actual,
            instruction,
        }
    } else {
        HierarchicalProjectionError::UnresolvedIdentity {
            operation,
            expected,
            id: raw_id.to_owned(),
            instruction,
        }
    }
}
