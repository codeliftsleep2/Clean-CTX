use super::super::payload::{
    reject_typed_flag_payload, require_non_empty_payload, require_vocabulary, validate_body_span,
};
use super::super::{IdentityError, IdentityIndex, IdentityKind, PatternTarget, pattern_target};
use super::require_target;
use crate::ir::CompiledIR;
use crate::ir::opcodes::{
    CTRL_AWAIT, CTRL_IF, CTRL_LOOP, CTRL_MATCH, CTRL_RETURN, CTRL_TRY, CoreOp, DATAFLOW_READ,
    DATAFLOW_WRITE,
};

pub(super) fn validate(ir: &CompiledIR, index: &IdentityIndex) -> Result<(), IdentityError> {
    for (instruction, op) in ir.instructions.iter().enumerate() {
        match op {
            CoreOp::DefClass(..) | CoreOp::DefInterface(..) => {}
            CoreOp::DefMethod(raw_owner, ..) => require_target(
                "DEF_M owner", raw_owner, IdentityKind::Class, instruction, index,
            )?,
            CoreOp::DefField(raw_owner, ..) => require_target(
                "DEF_F owner", raw_owner, IdentityKind::Class, instruction, index,
            )?,
            CoreOp::DefInterfaceMethod(raw_owner, ..) => require_target(
                "DEF_IM owner", raw_owner, IdentityKind::Interface, instruction, index,
            )?,
            CoreOp::DefInterfaceField(raw_owner, ..) => require_target(
                "DEF_IF owner", raw_owner, IdentityKind::Interface, instruction, index,
            )?,
            CoreOp::Param(raw_method, ..) => {
                require_target("SIG", raw_method, IdentityKind::Method, instruction, index)?
            }
            CoreOp::Return(raw_method, _) => {
                require_target("RET", raw_method, IdentityKind::Method, instruction, index)?
            }
            CoreOp::FieldType(raw_field, _) => require_target(
                "FIELD_T", raw_field, IdentityKind::Field, instruction, index,
            )?,
            CoreOp::MethodModifiers(raw_method, modifiers) => {
                require_target("MOD_M", raw_method, IdentityKind::Method, instruction, index)?;
                require_non_empty_payload("MOD_M", modifiers, instruction)?;
            }
            CoreOp::ClassModifiers(raw_class, modifiers) => {
                require_target("MOD_C", raw_class, IdentityKind::Class, instruction, index)?;
                require_non_empty_payload("MOD_C", modifiers, instruction)?;
            }
            CoreOp::InterfaceModifiers(raw_interface, modifiers) => {
                require_target(
                    "MOD_I", raw_interface, IdentityKind::Interface, instruction, index,
                )?;
                require_non_empty_payload("MOD_I", modifiers, instruction)?;
            }
            CoreOp::ControlSummary(raw_method, summaries) => {
                require_target(
                    "CTRL_SUM", raw_method, IdentityKind::Method, instruction, index,
                )?;
                require_non_empty_payload("CTRL_SUM", summaries, instruction)?;
            }
            CoreOp::PatternFacts(raw_method, facts) => {
                require_target(
                    "PAT_FACT", raw_method, IdentityKind::Method, instruction, index,
                )?;
                require_non_empty_payload("PAT_FACT", facts, instruction)?;
            }
            CoreOp::Flags(raw_method, flags) => {
                require_target("FLAGS", raw_method, IdentityKind::Method, instruction, index)?;
                require_non_empty_payload("FLAGS", flags, instruction)?;
                reject_typed_flag_payload("FLAGS", flags, instruction)?;
            }
            CoreOp::ClassFlags(raw_class, flags) => {
                require_target("FLAGS_C", raw_class, IdentityKind::Class, instruction, index)?;
                require_non_empty_payload("FLAGS_C", flags, instruction)?;
                reject_typed_flag_payload("FLAGS_C", flags, instruction)?;
            }
            CoreOp::Extends(raw_class, _) => {
                require_target("EXT", raw_class, IdentityKind::Class, instruction, index)?
            }
            CoreOp::InterfaceExtends(raw_interface, _) => require_target(
                "EXT_I", raw_interface, IdentityKind::Interface, instruction, index,
            )?,
            CoreOp::Implements(raw_class, _) => {
                require_target("IMPL", raw_class, IdentityKind::Class, instruction, index)?
            }
            CoreOp::Injects(raw_class, _) => require_target(
                "INJECTS", raw_class, IdentityKind::Class, instruction, index,
            )?,
            CoreOp::Import(..) | CoreOp::TypeAlias(..) => {}
            CoreOp::Pattern(name, args) => validate_pattern(name, args, instruction, index)?,
            CoreOp::Body(raw_method, _, start, end) => {
                require_target("BODY", raw_method, IdentityKind::Method, instruction, index)?;
                validate_body_span(*start, *end, instruction)?;
            }
            CoreOp::DataFlow(raw_method, direction, _) => {
                require_target(
                    "DATAFLOW", raw_method, IdentityKind::Method, instruction, index,
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
                    &[CTRL_IF, CTRL_LOOP, CTRL_MATCH, CTRL_TRY, CTRL_AWAIT, CTRL_RETURN],
                    instruction,
                )?;
            }
            CoreOp::SideEffect(raw_method, _) => require_target(
                "EFFECT", raw_method, IdentityKind::Method, instruction, index,
            )?,
            CoreOp::ExecutionContext(raw_method, _) => {
                require_target("CTX", raw_method, IdentityKind::Method, instruction, index)?
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
        "PAT class", class.as_str(), IdentityKind::Class, instruction, index,
    )?;
    let Some(method) = method else {
        return Ok(());
    };
    require_target(
        "PAT method", method.as_str(), IdentityKind::Method, instruction, index,
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
