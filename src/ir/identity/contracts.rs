use super::cardinality::{insert_definition, insert_singular};
use super::payload::{
    reject_typed_flag_payload, require_non_empty_payload, require_vocabulary, validate_body_span,
};
use super::{
    ClassId, FieldId, IdentityError, IdentityIndex, IdentityKind, MethodId, ParameterId,
    PatternTarget, pattern_target,
};
use crate::ir::CompiledIR;
use crate::ir::opcodes::{
    CTRL_AWAIT, CTRL_IF, CTRL_LOOP, CTRL_MATCH, CTRL_RETURN, CTRL_TRY, CoreOp, DATAFLOW_READ,
    DATAFLOW_WRITE,
};
use std::collections::HashMap;

pub(super) fn validate(ir: &CompiledIR) -> Result<IdentityIndex, IdentityError> {
    let index = collect_definitions_and_singular_facts(ir)?;
    validate_references_and_payloads(ir, &index)?;
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
                    raw_id.clone(),
                    "DEF_I",
                    IdentityKind::Interface,
                    raw_id,
                    None,
                    instruction,
                )?;
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
            | CoreOp::ControlSummary(..)
            | CoreOp::PatternFacts(..)
            | CoreOp::Implements(..)
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
fn validate_references_and_payloads(
    ir: &CompiledIR,
    index: &IdentityIndex,
) -> Result<(), IdentityError> {
    for (instruction, op) in ir.instructions.iter().enumerate() {
        match op {
            CoreOp::DefClass(..) => {}
            CoreOp::DefMethod(raw_owner, ..) => {
                require_target(
                    "DEF_M owner",
                    raw_owner,
                    IdentityKind::Class,
                    instruction,
                    index,
                )?;
            }
            CoreOp::DefField(raw_owner, ..) => {
                require_target(
                    "DEF_F owner",
                    raw_owner,
                    IdentityKind::Class,
                    instruction,
                    index,
                )?;
            }
            CoreOp::DefInterface(..) => {}
            CoreOp::Param(raw_method, ..) => {
                require_target("SIG", raw_method, IdentityKind::Method, instruction, index)?
            }
            CoreOp::Return(raw_method, _) => {
                require_target("RET", raw_method, IdentityKind::Method, instruction, index)?
            }
            CoreOp::FieldType(raw_field, _) => require_target(
                "FIELD_T",
                raw_field,
                IdentityKind::Field,
                instruction,
                index,
            )?,
            CoreOp::MethodModifiers(raw_method, modifiers) => {
                require_target(
                    "MOD_M",
                    raw_method,
                    IdentityKind::Method,
                    instruction,
                    index,
                )?;
                require_non_empty_payload("MOD_M", modifiers, instruction)?;
            }
            CoreOp::ClassModifiers(raw_class, modifiers) => {
                require_target("MOD_C", raw_class, IdentityKind::Class, instruction, index)?;
                require_non_empty_payload("MOD_C", modifiers, instruction)?;
            }
            CoreOp::ControlSummary(raw_method, summaries) => {
                require_target(
                    "CTRL_SUM",
                    raw_method,
                    IdentityKind::Method,
                    instruction,
                    index,
                )?;
                require_non_empty_payload("CTRL_SUM", summaries, instruction)?;
            }
            CoreOp::PatternFacts(raw_method, facts) => {
                require_target(
                    "PAT_FACT",
                    raw_method,
                    IdentityKind::Method,
                    instruction,
                    index,
                )?;
                require_non_empty_payload("PAT_FACT", facts, instruction)?;
            }
            CoreOp::Flags(raw_method, flags) => {
                require_target(
                    "FLAGS",
                    raw_method,
                    IdentityKind::Method,
                    instruction,
                    index,
                )?;
                require_non_empty_payload("FLAGS", flags, instruction)?;
                reject_typed_flag_payload("FLAGS", flags, instruction)?;
            }
            CoreOp::ClassFlags(raw_class, flags) => {
                require_target(
                    "FLAGS_C",
                    raw_class,
                    IdentityKind::Class,
                    instruction,
                    index,
                )?;
                require_non_empty_payload("FLAGS_C", flags, instruction)?;
                reject_typed_flag_payload("FLAGS_C", flags, instruction)?;
            }
            CoreOp::Extends(raw_class, _) => {
                require_target("EXT", raw_class, IdentityKind::Class, instruction, index)?
            }
            CoreOp::Implements(raw_class, _) => {
                require_target("IMPL", raw_class, IdentityKind::Class, instruction, index)?
            }
            CoreOp::Injects(raw_class, _) => require_target(
                "INJECTS",
                raw_class,
                IdentityKind::Class,
                instruction,
                index,
            )?,
            CoreOp::Import(..) | CoreOp::TypeAlias(..) => {}
            CoreOp::Pattern(name, args) => validate_pattern(name, args, instruction, index)?,
            CoreOp::Body(raw_method, _, start, end) => {
                require_target("BODY", raw_method, IdentityKind::Method, instruction, index)?;
                validate_body_span(*start, *end, instruction)?;
            }
            CoreOp::DataFlow(raw_method, direction, _) => {
                require_target(
                    "DATAFLOW",
                    raw_method,
                    IdentityKind::Method,
                    instruction,
                    index,
                )?;
                require_vocabulary(
                    "DATAFLOW",
                    "direction",
                    direction,
                    &[DATAFLOW_READ, DATAFLOW_WRITE],
                    instruction,
                )?;
            }
            CoreOp::ControlFlow(raw_method, kind, _) => {
                require_target("CTRL", raw_method, IdentityKind::Method, instruction, index)?;
                require_vocabulary(
                    "CTRL",
                    "kind",
                    kind,
                    &[
                        CTRL_IF,
                        CTRL_LOOP,
                        CTRL_MATCH,
                        CTRL_TRY,
                        CTRL_AWAIT,
                        CTRL_RETURN,
                    ],
                    instruction,
                )?;
            }
            CoreOp::SideEffect(raw_method, _) => {
                require_target(
                    "EFFECT",
                    raw_method,
                    IdentityKind::Method,
                    instruction,
                    index,
                )?;
            }
            CoreOp::ExecutionContext(raw_method, _) => {
                require_target("CTX", raw_method, IdentityKind::Method, instruction, index)?;
            }
            CoreOp::Call(raw_caller, ..) => {
                require_target("CALL", raw_caller, IdentityKind::Method, instruction, index)?
            }
        }
    }
    Ok(())
}
fn validate_pattern(
    name: &str,
    args: &[String],
    instruction: usize,
    index: &IdentityIndex,
) -> Result<(), IdentityError> {
    let target = pattern_target(name, args, instruction)?;
    let (class, method) = match &target {
        PatternTarget::Class(class) => (class, None),
        PatternTarget::Method { class, method } => (class, Some(method)),
    };
    require_target(
        "PAT class",
        class.as_str(),
        IdentityKind::Class,
        instruction,
        index,
    )?;
    let Some(method) = method else {
        return Ok(());
    };
    require_target(
        "PAT method",
        method.as_str(),
        IdentityKind::Method,
        instruction,
        index,
    )?;
    if index.method_owners.get(method) != Some(class) {
        return Err(IdentityError::OwnerMismatch {
            operation: "PAT",
            id: method.as_str().to_owned(),
            expected_owner: class.as_str().to_owned(),
            instruction,
        });
    }
    Ok(())
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
fn require_target(
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
        IdentityKind::Interface => index.interfaces.contains_key(raw_id),
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
    if expected != IdentityKind::Interface && index.interfaces.contains_key(raw_id) {
        return Some(IdentityKind::Interface);
    }
    None
}
