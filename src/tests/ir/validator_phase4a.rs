// Phase 4A contracts for the shared validation authority.

use crate::compression::Fidelity;
use crate::ir::compiler::CompiledIR;
use crate::ir::opcodes::{ControlSummary, CoreOp, DeclarationModifier};
use crate::ir::pipeline::{IRPass, PassContext, ValidationPass};
use crate::ir::validator::{DefaultValidator, IRValidator};

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "validator_phase4a.rs".into(),
        instructions,
        version: 1,
    }
}

fn definitions() -> Vec<CoreOp> {
    vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefClass("C2".into(), "Other".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "method".into()),
        CoreOp::DefInterface("I1".into(), "Contract".into()),
    ]
}

fn one_error(instructions: Vec<CoreOp>) -> crate::ir::validator::ValidationError {
    let mut errors = DefaultValidator::new().validate(&compiled(instructions));
    assert_eq!(errors.len(), 1, "expected one error, got {errors:?}");
    errors.remove(0)
}

#[test]
fn accepts_all_operation_contracts_in_legal_predefinition_order() {
    let mut instructions = vec![
        CoreOp::MethodModifiers(
            "M1".into(),
            vec![DeclarationModifier::Async, DeclarationModifier::Async],
        ),
        CoreOp::ControlSummary("M1".into(), vec![ControlSummary::Return]),
        CoreOp::ClassModifiers("C1".into(), vec![DeclarationModifier::Export]),
        CoreOp::Extends("C1".into(), "ExternalBase".into()),
        CoreOp::Implements("C1".into(), "ExternalInterface".into()),
        CoreOp::Implements("C1".into(), "ExternalInterface".into()),
        CoreOp::Injects("C1".into(), vec!["Dependency".into(), "Dependency".into()]),
        CoreOp::Import("Alias".into(), "module".into(), "Named".into()),
        CoreOp::TypeAlias("Local".into(), "ExternalType".into()),
        CoreOp::Pattern("OVERRIDE".into(), vec!["C1".into(), "M1".into()]),
        CoreOp::Body("M1".into(), "{}".into(), Some(10), Some(12)),
        CoreOp::DataFlow("M1".into(), "reads".into(), "value".into()),
        CoreOp::ControlFlow("M1".into(), "if".into(), "condition".into()),
        CoreOp::SideEffect("M1".into(), "io".into()),
        CoreOp::ExecutionContext("M1".into(), "async".into()),
        CoreOp::Call("M1".into(), "external_call".into(), 1, false),
    ];
    instructions.extend(definitions());

    let errors = DefaultValidator::new().validate(&compiled(instructions));

    assert!(errors.is_empty(), "valid contracts must pass: {errors:?}");
}

#[test]
fn rejects_duplicate_interface_import_extends_and_body() {
    let cases = [
        vec![
            CoreOp::DefInterface("I1".into(), "First".into()),
            CoreOp::DefInterface("I1".into(), "Second".into()),
        ],
        vec![
            CoreOp::Import("A".into(), "one".into(), "X".into()),
            CoreOp::Import("A".into(), "two".into(), "Y".into()),
        ],
        vec![
            CoreOp::Extends("C1".into(), "Base1".into()),
            CoreOp::Extends("C1".into(), "Base2".into()),
        ],
        vec![
            CoreOp::Body("M1".into(), "one".into(), None, None),
            CoreOp::Body("M1".into(), "two".into(), None, None),
        ],
    ];

    for instructions in cases {
        let error = one_error(instructions);
        assert!(
            error.message.contains("duplicate"),
            "duplicate contract must be diagnosed: {error:?}"
        );
    }
}

#[test]
fn repeated_type_alias_occurrences_are_valid_and_preserved_by_contract() {
    let errors = DefaultValidator::new().validate(&compiled(vec![
        CoreOp::TypeAlias("@action".into(), "GET first".into()),
        CoreOp::TypeAlias("@action".into(), "POST second".into()),
        CoreOp::TypeAlias("@action".into(), "POST second".into()),
    ]));

    assert!(
        errors.is_empty(),
        "TYPE is an ordered occurrence fact, not a unique definition: {errors:?}"
    );
}

#[test]
fn rejects_empty_flag_payloads_and_invalid_body_spans() {
    let mut empty_flags = definitions();
    empty_flags.push(CoreOp::Flags("M1".into(), Vec::new()));
    let flags_error = one_error(empty_flags);
    assert_eq!(flags_error.code, "E003");
    assert!(flags_error.message.contains("payload must not be empty"));

    let mut partial_span = definitions();
    partial_span.push(CoreOp::Body("M1".into(), "{}".into(), Some(10), None));
    let span_error = one_error(partial_span);
    assert_eq!(span_error.code, "ir_projection_invalid_operation");
    assert!(span_error.message.contains("both be present"));

    let mut reversed_span = definitions();
    reversed_span.push(CoreOp::Body("M1".into(), "{}".into(), Some(12), Some(10)));
    let reversed_error = one_error(reversed_span);
    assert!(reversed_error.message.contains("start exceeds end"));
}

#[test]
fn rejects_unresolved_targets_for_newly_covered_families() {
    let cases = [
        (
            CoreOp::ClassModifiers("C404".into(), vec![DeclarationModifier::Export]),
            "MOD_C",
        ),
        (
            CoreOp::Pattern("CUSTOM".into(), vec!["C404".into()]),
            "PAT class",
        ),
        (CoreOp::Body("M404".into(), "{}".into(), None, None), "BODY"),
    ];

    for (operation, expected_name) in cases {
        let error = one_error(vec![operation]);
        assert_eq!(error.code, "ir_projection_unresolved_identity");
        assert!(
            error.message.contains(expected_name),
            "diagnostic must name {expected_name}: {error:?}"
        );
    }
}

#[test]
fn rejects_cross_kind_interface_identity_collisions() {
    let error = one_error(vec![
        CoreOp::DefClass("Shared".into(), "Class".into()),
        CoreOp::DefInterface("Shared".into(), "Interface".into()),
    ]);

    assert_eq!(error.code, "ir_projection_kind_mismatch");
    assert!(error.message.contains("interface identity"));
    assert!(error.message.contains("class identity"));
}

#[test]
fn rejects_values_outside_execution_vocabularies() {
    let invalid_ops = [
        CoreOp::DataFlow("M1".into(), "copies".into(), "value".into()),
        CoreOp::ControlFlow("M1".into(), "goto".into(), "label".into()),
        CoreOp::SideEffect("M1".into(), "network".into()),
        CoreOp::ExecutionContext("M1".into(), "worker".into()),
    ];

    for invalid in invalid_ops {
        let mut instructions = definitions();
        instructions.push(invalid);
        let error = one_error(instructions);
        assert!(
            error.message.contains("unknown"),
            "unexpected error: {error:?}"
        );
    }
}

#[test]
fn validates_pattern_schema_targets_and_ownership_without_prefix_inference() {
    let mut wrong_owner = definitions();
    wrong_owner.push(CoreOp::Pattern(
        "OVERRIDE".into(),
        vec!["C2".into(), "M1".into()],
    ));
    let owner_error = one_error(wrong_owner);
    assert_eq!(owner_error.code, "ir_projection_owner_mismatch");

    let mut class_pattern = definitions();
    class_pattern.push(CoreOp::Pattern(
        "CUSTOM".into(),
        vec!["C1".into(), "M-looking-metadata".into()],
    ));
    assert!(
        DefaultValidator::new()
            .validate(&compiled(class_pattern))
            .is_empty(),
        "generic class patterns must not infer a method target from metadata"
    );

    let mut empty_name = definitions();
    empty_name.push(CoreOp::Pattern(String::new(), vec!["C1".into()]));
    let name_error = one_error(empty_name);
    assert!(
        name_error
            .message
            .contains("pattern name must not be empty")
    );

    let mut wrong_kind = definitions();
    wrong_kind.push(CoreOp::Pattern(
        "OVERRIDE".into(),
        vec!["C1".into(), "C2".into()],
    ));
    let kind_error = one_error(wrong_kind);
    assert_eq!(kind_error.code, "ir_projection_kind_mismatch");
}

#[test]
fn production_validation_pass_enforces_phase4a_contracts() {
    let mut context = PassContext::new(String::new(), "validator_phase4a.rs".into(), Fidelity::Low);
    context.instructions = definitions();
    context
        .instructions
        .push(CoreOp::ClassFlags("C1".into(), Vec::new()));

    let error = ValidationPass::new()
        .run(&mut context)
        .expect_err("production validation must reject an empty class-flags payload");

    assert_eq!(error.pass_name, "validation");
    assert!(error.message.contains("ir_projection_invalid_operation"));
    assert!(error.message.contains("FLAGS_C"));
}
