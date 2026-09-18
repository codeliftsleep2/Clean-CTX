// Stable semantic identity validation for hierarchical projection.
//
// `CoreOp` remains the serialized DTO during this migration, so its operands
// stay strings on the wire. This module is the checked internal boundary that
// converts approved identities into distinct Rust types before projection may
// use them.

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
pub(super) struct FieldId(String);

impl FieldId {
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
pub(super) struct ProjectionIdentities {
    pub(super) classes: HashMap<ClassId, usize>,
    pub(super) methods: HashMap<MethodId, usize>,
    pub(super) fields: HashMap<FieldId, usize>,
}

/// Validate the migrated identity graph without using instruction position as
/// semantic authority.
pub(super) fn validate_projection_identities(
    ir: &CompiledIR,
) -> Result<ProjectionIdentities, HierarchicalProjectionError> {
    let mut classes = HashMap::new();
    let mut methods = HashMap::new();
    let mut fields = HashMap::new();
    let mut parameters = HashMap::new();
    let mut returns = HashMap::new();
    let mut field_types = HashMap::new();

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
            CoreOp::DefField(raw_owner, raw_id, _) => {
                require_non_empty(
                    "DEF_F owner",
                    ProjectionIdentityKind::Class,
                    raw_owner,
                    None,
                    instruction,
                )?;
                require_non_empty(
                    "DEF_F",
                    ProjectionIdentityKind::Field,
                    raw_id,
                    Some(raw_owner.as_str()),
                    instruction,
                )?;
                let id = FieldId::from_serialized(raw_id);
                if let Some(first_instruction) = fields.insert(id, instruction) {
                    return Err(HierarchicalProjectionError::DuplicateIdentity {
                        operation: "DEF_F",
                        kind: ProjectionIdentityKind::Field,
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
            CoreOp::FieldType(raw_field, _) => {
                require_non_empty(
                    "FIELD_T",
                    ProjectionIdentityKind::Field,
                    raw_field,
                    None,
                    instruction,
                )?;
                let field = FieldId::from_serialized(raw_field);
                if let Some(first_instruction) = field_types.insert(field, instruction) {
                    return Err(HierarchicalProjectionError::DuplicateFieldType {
                        field_id: raw_field.clone(),
                        first_instruction,
                        duplicate_instruction: instruction,
                    });
                }
            }
            CoreOp::DefInterface(..)
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
                &fields,
            ));
        }
        if let Some(actual) = conflicting_definition_kind(
            method_id.as_str(),
            ProjectionIdentityKind::Method,
            &classes,
            &methods,
            &fields,
        ) {
            return Err(HierarchicalProjectionError::KindMismatch {
                operation: "DEF_M identity",
                id: method_id.as_str().to_owned(),
                expected: ProjectionIdentityKind::Method,
                actual,
                instruction,
            });
        }
    }

    for (instruction, op) in ir.instructions.iter().enumerate() {
        let CoreOp::DefField(raw_owner, raw_field, _) = op else {
            continue;
        };
        let owner = ClassId::from_serialized(raw_owner);
        let field_id = FieldId::from_serialized(raw_field);
        if !classes.contains_key(&owner) {
            return Err(unresolved_or_wrong_kind(
                "DEF_F owner",
                ProjectionIdentityKind::Class,
                raw_owner,
                instruction,
                &classes,
                &methods,
                &fields,
            ));
        }
        if let Some(actual) = conflicting_definition_kind(
            field_id.as_str(),
            ProjectionIdentityKind::Field,
            &classes,
            &methods,
            &fields,
        ) {
            return Err(HierarchicalProjectionError::KindMismatch {
                operation: "DEF_F identity",
                id: field_id.as_str().to_owned(),
                expected: ProjectionIdentityKind::Field,
                actual,
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
                &fields,
            ));
        }
    }

    for (instruction, op) in ir.instructions.iter().enumerate() {
        let CoreOp::FieldType(raw_field, _) = op else {
            continue;
        };
        let field = FieldId::from_serialized(raw_field);
        if !fields.contains_key(&field) {
            return Err(unresolved_or_wrong_kind(
                "FIELD_T",
                ProjectionIdentityKind::Field,
                raw_field,
                instruction,
                &classes,
                &methods,
                &fields,
            ));
        }
    }

    Ok(ProjectionIdentities {
        classes,
        methods,
        fields,
    })
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
    fields: &HashMap<FieldId, usize>,
) -> HierarchicalProjectionError {
    let actual = match expected {
        ProjectionIdentityKind::Class
            if methods.contains_key(&MethodId::from_serialized(raw_id)) =>
        {
            Some(ProjectionIdentityKind::Method)
        }
        ProjectionIdentityKind::Class if fields.contains_key(&FieldId::from_serialized(raw_id)) => {
            Some(ProjectionIdentityKind::Field)
        }
        ProjectionIdentityKind::Method
            if classes.contains_key(&ClassId::from_serialized(raw_id)) =>
        {
            Some(ProjectionIdentityKind::Class)
        }
        ProjectionIdentityKind::Method
            if fields.contains_key(&FieldId::from_serialized(raw_id)) =>
        {
            Some(ProjectionIdentityKind::Field)
        }
        ProjectionIdentityKind::Field
            if classes.contains_key(&ClassId::from_serialized(raw_id)) =>
        {
            Some(ProjectionIdentityKind::Class)
        }
        ProjectionIdentityKind::Field
            if methods.contains_key(&MethodId::from_serialized(raw_id)) =>
        {
            Some(ProjectionIdentityKind::Method)
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

fn conflicting_definition_kind(
    raw_id: &str,
    expected: ProjectionIdentityKind,
    classes: &HashMap<ClassId, usize>,
    methods: &HashMap<MethodId, usize>,
    fields: &HashMap<FieldId, usize>,
) -> Option<ProjectionIdentityKind> {
    if expected != ProjectionIdentityKind::Class
        && classes.contains_key(&ClassId::from_serialized(raw_id))
    {
        return Some(ProjectionIdentityKind::Class);
    }
    if expected != ProjectionIdentityKind::Method
        && methods.contains_key(&MethodId::from_serialized(raw_id))
    {
        return Some(ProjectionIdentityKind::Method);
    }
    if expected != ProjectionIdentityKind::Field
        && fields.contains_key(&FieldId::from_serialized(raw_id))
    {
        return Some(ProjectionIdentityKind::Field);
    }
    None
}
