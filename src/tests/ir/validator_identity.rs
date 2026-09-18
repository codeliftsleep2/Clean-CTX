// Shared identity authority across canonical validation and projection.

use crate::compression::Fidelity;
use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{
    HierarchicalProjectionError, ProjectionIdentityKind, try_ir_to_hierarchical,
};
use crate::ir::opcodes::CoreOp;
use crate::ir::pipeline::{IRPass, PassContext, ValidationPass};
use crate::ir::validator::{DefaultValidator, IRValidator};

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "validator_identity.rs".into(),
        instructions,
        version: 1,
    }
}

#[test]
fn validation_accepts_legal_predefinition_order_for_all_approved_identities() {
    let ir = compiled(vec![
        CoreOp::FieldType("F1".into(), "$s".into()),
        CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "value".into()),
        CoreOp::Return("M1".into(), "$v".into()),
        CoreOp::DefField("C1".into(), "F1".into(), "field".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "method".into()),
        CoreOp::DefClass("C1".into(), "Owner".into()),
    ]);

    assert!(DefaultValidator::new().validate(&ir).is_empty());
    assert!(try_ir_to_hierarchical(&ir).is_ok());
}

#[test]
fn validator_and_projection_share_duplicate_identity_diagnostic() {
    let ir = compiled(vec![
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefClass("C1".into(), "Second".into()),
    ]);

    let validation = DefaultValidator::new().validate(&ir);
    let projection = try_ir_to_hierarchical(&ir).expect_err("duplicate class must fail");

    assert_eq!(validation.len(), 1);
    assert_eq!(validation[0].code, projection.code());
    assert_eq!(validation[0].instruction_index, Some(1));
    assert_eq!(validation[0].message, projection.to_string());
}

#[test]
fn validator_reports_wrong_kind_field_target_from_shared_authority() {
    let ir = compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::FieldType("C1".into(), "$s".into()),
    ]);

    let errors = DefaultValidator::new().validate(&ir);

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "ir_projection_kind_mismatch");
    assert_eq!(errors[0].instruction_index, Some(1));
    assert!(errors[0].message.contains("requires a field identity"));
}

#[test]
fn validator_preserves_legacy_codes_for_param_and_return_targets() {
    let param_errors = DefaultValidator::new().validate(&compiled(vec![CoreOp::Param(
        "M404".into(),
        "P1".into(),
        "$s".into(),
        "value".into(),
    )]));
    let return_errors = DefaultValidator::new()
        .validate(&compiled(vec![CoreOp::Return("M404".into(), "$v".into())]));

    assert_eq!(param_errors[0].code, "E002");
    assert_eq!(return_errors[0].code, "E001");
}

#[test]
fn production_validation_pass_rejects_invalid_field_owner() {
    let mut context =
        PassContext::new(String::new(), "validator_identity.rs".into(), Fidelity::Low);
    context.instructions = vec![CoreOp::DefField(
        "C404".into(),
        "F1".into(),
        "orphan".into(),
    )];

    let error = ValidationPass::new()
        .run(&mut context)
        .expect_err("production validation must reject orphan fields");

    assert_eq!(error.pass_name, "validation");
    assert!(error.message.contains("ir_projection_unresolved_identity"));
    assert!(error.message.contains("DEF_F owner"));
}

#[test]
fn projection_alias_retains_public_structured_error_shape() {
    let error = try_ir_to_hierarchical(&compiled(vec![CoreOp::FieldType(
        "F404".into(),
        "$s".into(),
    )]))
    .expect_err("unknown field must fail");

    assert_eq!(
        error,
        HierarchicalProjectionError::UnresolvedIdentity {
            operation: "FIELD_T",
            expected: ProjectionIdentityKind::Field,
            id: "F404".into(),
            instruction: 0,
        }
    );
}
