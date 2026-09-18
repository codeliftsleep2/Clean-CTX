// Shared typed semantic-identity validation for canonical IR consumers.
//
// `CoreOp` remains the serialized DTO during this migration, so identity
// operands stay strings on the wire. This module is the single checked
// internal boundary used by production validation and hierarchical projection.

use super::{CompiledIR, CoreOp};
use std::collections::HashMap;

mod error;

pub use error::IdentityError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityKind {
    Class,
    Method,
    Field,
    Parameter,
}

impl std::fmt::Display for IdentityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class => f.write_str("class"),
            Self::Method => f.write_str("method"),
            Self::Field => f.write_str("field"),
            Self::Parameter => f.write_str("parameter"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ClassId(String);

impl ClassId {
    pub(crate) fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MethodId(String);

impl MethodId {
    pub(crate) fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FieldId(String);

impl FieldId {
    pub(crate) fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }

    pub(crate) fn as_str(&self) -> &str {
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
pub(crate) struct IdentityIndex {
    pub(crate) classes: HashMap<ClassId, usize>,
    pub(crate) methods: HashMap<MethodId, usize>,
    pub(crate) fields: HashMap<FieldId, usize>,
}

impl IdentityIndex {
    pub(crate) fn contains_class(&self, raw_id: &str) -> bool {
        self.classes.contains_key(&ClassId::from_serialized(raw_id))
    }

    pub(crate) fn contains_method(&self, raw_id: &str) -> bool {
        self.methods
            .contains_key(&MethodId::from_serialized(raw_id))
    }
}

/// Validate the six approved identity-bearing operations without using stream
/// position as semantic authority. Every other `CoreOp` has an explicit
/// deferred arm so adding a new variant requires a compiler-visible decision.
pub(crate) fn validate_identity_graph(ir: &CompiledIR) -> Result<IdentityIndex, IdentityError> {
    let mut classes = HashMap::new();
    let mut methods = HashMap::new();
    let mut fields = HashMap::new();
    let mut parameters = HashMap::new();
    let mut returns = HashMap::new();
    let mut field_types = HashMap::new();

    for (instruction, op) in ir.instructions.iter().enumerate() {
        match op {
            CoreOp::DefClass(raw_id, _) => {
                require_non_empty("DEF_C", IdentityKind::Class, raw_id, None, instruction)?;
                let id = ClassId::from_serialized(raw_id);
                if let Some(first_instruction) = classes.insert(id, instruction) {
                    return Err(IdentityError::DuplicateIdentity {
                        operation: "DEF_C",
                        kind: IdentityKind::Class,
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
                    IdentityKind::Class,
                    raw_owner,
                    None,
                    instruction,
                )?;
                require_non_empty(
                    "DEF_M",
                    IdentityKind::Method,
                    raw_id,
                    Some(raw_owner),
                    instruction,
                )?;
                let id = MethodId::from_serialized(raw_id);
                if let Some(first_instruction) = methods.insert(id, instruction) {
                    return Err(IdentityError::DuplicateIdentity {
                        operation: "DEF_M",
                        kind: IdentityKind::Method,
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
                    IdentityKind::Class,
                    raw_owner,
                    None,
                    instruction,
                )?;
                require_non_empty(
                    "DEF_F",
                    IdentityKind::Field,
                    raw_id,
                    Some(raw_owner),
                    instruction,
                )?;
                let id = FieldId::from_serialized(raw_id);
                if let Some(first_instruction) = fields.insert(id, instruction) {
                    return Err(IdentityError::DuplicateIdentity {
                        operation: "DEF_F",
                        kind: IdentityKind::Field,
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
                    IdentityKind::Method,
                    raw_method,
                    None,
                    instruction,
                )?;
                require_non_empty(
                    "SIG",
                    IdentityKind::Parameter,
                    raw_parameter,
                    Some(raw_method),
                    instruction,
                )?;
                let key = (
                    MethodId::from_serialized(raw_method),
                    ParameterId::from_serialized(raw_parameter),
                );
                if let Some(first_instruction) = parameters.insert(key, instruction) {
                    return Err(IdentityError::DuplicateIdentity {
                        operation: "SIG",
                        kind: IdentityKind::Parameter,
                        id: raw_parameter.clone(),
                        owner: Some(raw_method.clone()),
                        first_instruction,
                        duplicate_instruction: instruction,
                    });
                }
            }
            CoreOp::Return(raw_method, _) => {
                require_non_empty("RET", IdentityKind::Method, raw_method, None, instruction)?;
                if let Some(first_instruction) =
                    returns.insert(MethodId::from_serialized(raw_method), instruction)
                {
                    return Err(IdentityError::DuplicateReturn {
                        method_id: raw_method.clone(),
                        first_instruction,
                        duplicate_instruction: instruction,
                    });
                }
            }
            CoreOp::FieldType(raw_field, _) => {
                require_non_empty("FIELD_T", IdentityKind::Field, raw_field, None, instruction)?;
                if let Some(first_instruction) =
                    field_types.insert(FieldId::from_serialized(raw_field), instruction)
                {
                    return Err(IdentityError::DuplicateFieldType {
                        field_id: raw_field.clone(),
                        first_instruction,
                        duplicate_instruction: instruction,
                    });
                }
            }
            CoreOp::DefInterface(..) => {}
            CoreOp::Flags(..) => {}
            CoreOp::ClassFlags(..) => {}
            CoreOp::Extends(..) => {}
            CoreOp::Implements(..) => {}
            CoreOp::Injects(..) => {}
            CoreOp::Import(..) => {}
            CoreOp::TypeAlias(..) => {}
            CoreOp::Pattern(..) => {}
            CoreOp::Body(..) => {}
            CoreOp::DataFlow(..) => {}
            CoreOp::ControlFlow(..) => {}
            CoreOp::SideEffect(..) => {}
            CoreOp::ExecutionContext(..) => {}
            CoreOp::Call(..) => {}
        }
    }

    for (instruction, op) in ir.instructions.iter().enumerate() {
        match op {
            CoreOp::DefMethod(raw_owner, raw_method, _) => {
                require_owner(
                    "DEF_M owner",
                    raw_owner,
                    instruction,
                    &classes,
                    &methods,
                    &fields,
                )?;
                require_definition_kind(
                    "DEF_M identity",
                    raw_method,
                    IdentityKind::Method,
                    instruction,
                    &classes,
                    &methods,
                    &fields,
                )?;
            }
            CoreOp::DefField(raw_owner, raw_field, _) => {
                require_owner(
                    "DEF_F owner",
                    raw_owner,
                    instruction,
                    &classes,
                    &methods,
                    &fields,
                )?;
                require_definition_kind(
                    "DEF_F identity",
                    raw_field,
                    IdentityKind::Field,
                    instruction,
                    &classes,
                    &methods,
                    &fields,
                )?;
            }
            CoreOp::Param(raw_method, ..) => require_target(
                "SIG",
                raw_method,
                IdentityKind::Method,
                instruction,
                &classes,
                &methods,
                &fields,
            )?,
            CoreOp::Return(raw_method, _) => require_target(
                "RET",
                raw_method,
                IdentityKind::Method,
                instruction,
                &classes,
                &methods,
                &fields,
            )?,
            CoreOp::FieldType(raw_field, _) => require_target(
                "FIELD_T",
                raw_field,
                IdentityKind::Field,
                instruction,
                &classes,
                &methods,
                &fields,
            )?,
            CoreOp::DefClass(..) => {}
            CoreOp::DefInterface(..) => {}
            CoreOp::Flags(..) => {}
            CoreOp::ClassFlags(..) => {}
            CoreOp::Extends(..) => {}
            CoreOp::Implements(..) => {}
            CoreOp::Injects(..) => {}
            CoreOp::Import(..) => {}
            CoreOp::TypeAlias(..) => {}
            CoreOp::Pattern(..) => {}
            CoreOp::Body(..) => {}
            CoreOp::DataFlow(..) => {}
            CoreOp::ControlFlow(..) => {}
            CoreOp::SideEffect(..) => {}
            CoreOp::ExecutionContext(..) => {}
            CoreOp::Call(..) => {}
        }
    }

    Ok(IdentityIndex {
        classes,
        methods,
        fields,
    })
}

fn require_non_empty(
    operation: &'static str,
    kind: IdentityKind,
    raw_id: &str,
    owner: Option<&str>,
    instruction: usize,
) -> Result<(), IdentityError> {
    if raw_id.is_empty() {
        return Err(IdentityError::InvalidIdentity {
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

fn require_owner(
    operation: &'static str,
    raw_owner: &str,
    instruction: usize,
    classes: &HashMap<ClassId, usize>,
    methods: &HashMap<MethodId, usize>,
    fields: &HashMap<FieldId, usize>,
) -> Result<(), IdentityError> {
    require_target(
        operation,
        raw_owner,
        IdentityKind::Class,
        instruction,
        classes,
        methods,
        fields,
    )
}

fn require_definition_kind(
    operation: &'static str,
    raw_id: &str,
    expected: IdentityKind,
    instruction: usize,
    classes: &HashMap<ClassId, usize>,
    methods: &HashMap<MethodId, usize>,
    fields: &HashMap<FieldId, usize>,
) -> Result<(), IdentityError> {
    if let Some(actual) = conflicting_kind(raw_id, expected, classes, methods, fields) {
        return Err(IdentityError::KindMismatch {
            operation,
            id: raw_id.to_owned(),
            expected,
            actual,
            instruction,
        });
    }
    Ok(())
}

fn require_target(
    operation: &'static str,
    raw_id: &str,
    expected: IdentityKind,
    instruction: usize,
    classes: &HashMap<ClassId, usize>,
    methods: &HashMap<MethodId, usize>,
    fields: &HashMap<FieldId, usize>,
) -> Result<(), IdentityError> {
    let resolved = match expected {
        IdentityKind::Class => classes.contains_key(&ClassId::from_serialized(raw_id)),
        IdentityKind::Method => methods.contains_key(&MethodId::from_serialized(raw_id)),
        IdentityKind::Field => fields.contains_key(&FieldId::from_serialized(raw_id)),
        IdentityKind::Parameter => false,
    };
    if resolved {
        return Ok(());
    }
    if let Some(actual) = conflicting_kind(raw_id, expected, classes, methods, fields) {
        return Err(IdentityError::KindMismatch {
            operation,
            id: raw_id.to_owned(),
            expected,
            actual,
            instruction,
        });
    }
    Err(IdentityError::UnresolvedIdentity {
        operation,
        expected,
        id: raw_id.to_owned(),
        instruction,
    })
}

fn conflicting_kind(
    raw_id: &str,
    expected: IdentityKind,
    classes: &HashMap<ClassId, usize>,
    methods: &HashMap<MethodId, usize>,
    fields: &HashMap<FieldId, usize>,
) -> Option<IdentityKind> {
    if expected != IdentityKind::Class && classes.contains_key(&ClassId::from_serialized(raw_id)) {
        return Some(IdentityKind::Class);
    }
    if expected != IdentityKind::Method && methods.contains_key(&MethodId::from_serialized(raw_id))
    {
        return Some(IdentityKind::Method);
    }
    if expected != IdentityKind::Field && fields.contains_key(&FieldId::from_serialized(raw_id)) {
        return Some(IdentityKind::Field);
    }
    None
}
