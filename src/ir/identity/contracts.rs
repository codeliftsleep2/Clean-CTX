use super::cardinality::{insert_definition, insert_singular};
use super::{
    ClassId, FieldId, IdentityError, IdentityIndex, IdentityKind, InterfaceId, MethodId, ParameterId,
};
use crate::ir::CompiledIR;
use crate::ir::opcodes::CoreOp;
use std::collections::HashMap;

mod references;

pub(super) fn validate(ir: &CompiledIR) -> Result<IdentityIndex, IdentityError> {
    let index = collect_definitions_and_singular_facts(ir)?;
    references::validate(ir, &index)?;
    Ok(index)
}

#[derive(Default)]
struct SingularFacts {
    parameters: HashMap<(MethodId, ParameterId), usize>,
    returns: HashMap<MethodId, usize>,
    field_types: HashMap<FieldId, usize>,
    extends: HashMap<ClassId, usize>,
    bodies: HashMap<MethodId, usize>,
    import_aliases: HashMap<String, usize>,
}
fn collect_definitions_and_singular_facts(ir: &CompiledIR) -> Result<IdentityIndex, IdentityError> {
    let mut index = IdentityIndex {
        classes: HashMap::new(),
        methods: HashMap::new(),
        fields: HashMap::new(),
        method_owners: HashMap::new(),
        interface_method_owners: HashMap::new(),
        interface_field_owners: HashMap::new(),
        interfaces: HashMap::new(),
    };
    let mut facts = SingularFacts::default();
    let mut definition_kinds: HashMap<String, IdentityKind> = HashMap::new();

    for (instruction, op) in ir.instructions.iter().enumerate() {
        match op {
            CoreOp::DefClass(raw_id, _) => {
                require_non_empty("DEF_C", IdentityKind::Class, raw_id, None, instruction)?;
                register_definition_kind(
                    &mut definition_kinds,
                    "DEF_C",
                    raw_id,
                    IdentityKind::Class,
                    instruction,
                )?;
                insert_definition(
                    &mut index.classes,
                    ClassId::from_serialized(raw_id),
                    "DEF_C",
                    IdentityKind::Class,
                    raw_id,
                    None,
                    instruction,
                )?;
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
                register_definition_kind(
                    &mut definition_kinds,
                    "DEF_M identity",
                    raw_id,
                    IdentityKind::Method,
                    instruction,
                )?;
                let method = MethodId::from_serialized(raw_id);
                insert_definition(
                    &mut index.methods,
                    method.clone(),
                    "DEF_M",
                    IdentityKind::Method,
                    raw_id,
                    Some(raw_owner),
                    instruction,
                )?;
                index
                    .method_owners
                    .insert(method, ClassId::from_serialized(raw_owner));
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
                register_definition_kind(
                    &mut definition_kinds,
                    "DEF_F identity",
                    raw_id,
                    IdentityKind::Field,
                    instruction,
                )?;
                insert_definition(
                    &mut index.fields,
                    FieldId::from_serialized(raw_id),
                    "DEF_F",
                    IdentityKind::Field,
                    raw_id,
                    Some(raw_owner),
                    instruction,
                )?;
            }
            CoreOp::DefInterface(raw_id, _) => {
                require_non_empty("DEF_I", IdentityKind::Interface, raw_id, None, instruction)?;
                register_definition_kind(
                    &mut definition_kinds,
                    "DEF_I",
                    raw_id,
                    IdentityKind::Interface,
                    instruction,
                )?;
                insert_definition(
                    &mut index.interfaces,
                    InterfaceId::from_serialized(raw_id),
                    "DEF_I",
                    IdentityKind::Interface,
                    raw_id,
                    None,
                    instruction,
                )?;
            }
            CoreOp::DefInterfaceMethod(raw_owner, raw_id, _) => {
                require_non_empty(
                    "DEF_IM owner",
                    IdentityKind::Interface,
                    raw_owner,
                    None,
                    instruction,
                )?;
                require_non_empty(
                    "DEF_IM",
                    IdentityKind::Method,
                    raw_id,
                    Some(raw_owner),
                    instruction,
                )?;
                register_definition_kind(
                    &mut definition_kinds,
                    "DEF_IM identity",
                    raw_id,
                    IdentityKind::Method,
                    instruction,
                )?;
                let method = MethodId::from_serialized(raw_id);
                insert_definition(
                    &mut index.methods,
                    method.clone(),
                    "DEF_IM",
                    IdentityKind::Method,
                    raw_id,
                    Some(raw_owner),
                    instruction,
                )?;
                index
                    .interface_method_owners
                    .insert(method, InterfaceId::from_serialized(raw_owner));
            }
            CoreOp::DefInterfaceField(raw_owner, raw_id, _) => {
                require_non_empty(
                    "DEF_IF owner",
                    IdentityKind::Interface,
                    raw_owner,
                    None,
                    instruction,
                )?;
                require_non_empty(
                    "DEF_IF",
                    IdentityKind::Field,
                    raw_id,
                    Some(raw_owner),
                    instruction,
                )?;
                register_definition_kind(
                    &mut definition_kinds,
                    "DEF_IF identity",
                    raw_id,
                    IdentityKind::Field,
                    instruction,
                )?;
                let field = FieldId::from_serialized(raw_id);
                insert_definition(
                    &mut index.fields,
                    field.clone(),
                    "DEF_IF",
                    IdentityKind::Field,
                    raw_id,
                    Some(raw_owner),
                    instruction,
                )?;
                index
                    .interface_field_owners
                    .insert(field, InterfaceId::from_serialized(raw_owner));
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
                insert_definition(
                    &mut facts.parameters,
                    (
                        MethodId::from_serialized(raw_method),
                        ParameterId::from_serialized(raw_parameter),
                    ),
                    "SIG",
                    IdentityKind::Parameter,
                    raw_parameter,
                    Some(raw_method),
                    instruction,
                )?;
            }
            CoreOp::Return(raw_method, _) => {
                require_non_empty("RET", IdentityKind::Method, raw_method, None, instruction)?;
                let method = MethodId::from_serialized(raw_method);
                if let Some(first_instruction) = facts.returns.insert(method, instruction) {
                    return Err(IdentityError::DuplicateReturn {
                        method_id: raw_method.clone(),
                        first_instruction,
                        duplicate_instruction: instruction,
                    });
                }
            }
            CoreOp::FieldType(raw_field, _) => {
                require_non_empty("FIELD_T", IdentityKind::Field, raw_field, None, instruction)?;
                let field = FieldId::from_serialized(raw_field);
                if let Some(first_instruction) = facts.field_types.insert(field, instruction) {
                    return Err(IdentityError::DuplicateFieldType {
                        field_id: raw_field.clone(),
                        first_instruction,
                        duplicate_instruction: instruction,
                    });
                }
            }
            CoreOp::Extends(raw_class, _) => insert_singular(
                &mut facts.extends,
                ClassId::from_serialized(raw_class),
                "EXT",
                IdentityKind::Class,
                raw_class,
                instruction,
            )?,
            CoreOp::Import(alias, ..) => {
                require_non_empty("IMP", IdentityKind::ImportAlias, alias, None, instruction)?;
                insert_definition(
                    &mut facts.import_aliases,
                    alias.clone(),
                    "IMP",
                    IdentityKind::ImportAlias,
                    alias,
                    None,
                    instruction,
                )?;
            }
            CoreOp::TypeAlias(alias, _) => {
                require_non_empty("TYPE", IdentityKind::TypeAlias, alias, None, instruction)?;
            }
            CoreOp::Body(raw_method, ..) => insert_singular(
                &mut facts.bodies,
                MethodId::from_serialized(raw_method),
                "BODY",
                IdentityKind::Method,
                raw_method,
                instruction,
            )?,
            CoreOp::Flags(..)
            | CoreOp::ClassFlags(..)
            | CoreOp::MethodModifiers(..)
            | CoreOp::ClassModifiers(..)
            | CoreOp::InterfaceModifiers(..)
            | CoreOp::ControlSummary(..)
            | CoreOp::PatternFacts(..)
            | CoreOp::Implements(..)
            | CoreOp::InterfaceExtends(..)
            | CoreOp::Injects(..)
            | CoreOp::Pattern(..)
            | CoreOp::DataFlow(..)
            | CoreOp::ControlFlow(..)
            | CoreOp::SideEffect(..)
            | CoreOp::ExecutionContext(..)
            | CoreOp::Call(..) => {}
        }
    }

    Ok(index)
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
fn register_definition_kind(
    definitions: &mut HashMap<String, IdentityKind>,
    operation: &'static str,
    raw_id: &str,
    kind: IdentityKind,
    instruction: usize,
) -> Result<(), IdentityError> {
    if let Some(actual) = definitions.get(raw_id).copied() {
        if actual == kind {
            return Ok(());
        }
        return Err(IdentityError::KindMismatch {
            operation,
            id: raw_id.to_owned(),
            expected: kind,
            actual,
            instruction,
        });
    }
    definitions.insert(raw_id.to_owned(), kind);
    Ok(())
}
pub(super) fn require_target(
    operation: &'static str,
    raw_id: &str,
    expected: IdentityKind,
    instruction: usize,
    index: &IdentityIndex,
) -> Result<(), IdentityError> {
    require_non_empty(operation, expected, raw_id, None, instruction)?;
    let resolved = match expected {
        IdentityKind::Class => index
            .classes
            .contains_key(&ClassId::from_serialized(raw_id)),
        IdentityKind::Method => index
            .methods
            .contains_key(&MethodId::from_serialized(raw_id)),
        IdentityKind::Field => index.fields.contains_key(&FieldId::from_serialized(raw_id)),
        IdentityKind::Interface => index
            .interfaces
            .contains_key(&InterfaceId::from_serialized(raw_id)),
        IdentityKind::Parameter | IdentityKind::ImportAlias | IdentityKind::TypeAlias => false,
    };
    if resolved {
        return Ok(());
    }
    if let Some(actual) = conflicting_kind(raw_id, expected, index) {
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
    index: &IdentityIndex,
) -> Option<IdentityKind> {
    if expected != IdentityKind::Class
        && index
            .classes
            .contains_key(&ClassId::from_serialized(raw_id))
    {
        return Some(IdentityKind::Class);
    }
    if expected != IdentityKind::Method
        && index
            .methods
            .contains_key(&MethodId::from_serialized(raw_id))
    {
        return Some(IdentityKind::Method);
    }
    if expected != IdentityKind::Field
        && index.fields.contains_key(&FieldId::from_serialized(raw_id))
    {
        return Some(IdentityKind::Field);
    }
    if expected != IdentityKind::Interface
        && index
            .interfaces
            .contains_key(&InterfaceId::from_serialized(raw_id))
    {
        return Some(IdentityKind::Interface);
    }
    None
}
