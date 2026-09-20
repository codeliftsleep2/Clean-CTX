use crate::ir::binary_wire::{decode, encode};
use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{hierarchical_to_ir, try_ir_to_hierarchical};
use crate::ir::opcodes::{ControlSummary, CoreOp, DeclarationModifier};
use crate::ir::wire::{op_to_tuple, tuple_to_op};

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "modifier-contract".into(),
        version: 1,
        instructions,
    }
}

fn contract_stream() -> CompiledIR {
    compiled(vec![
        CoreOp::MethodModifiers(
            "M1".into(),
            vec![DeclarationModifier::Async, DeclarationModifier::Async],
        ),
        CoreOp::ControlSummary(
            "M1".into(),
            vec![ControlSummary::Branch, ControlSummary::Return],
        ),
        CoreOp::ClassModifiers(
            "C1".into(),
            vec![DeclarationModifier::Export, DeclarationModifier::Abstract],
        ),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
        CoreOp::DefClass("C1".into(), "Owner".into()),
    ])
}

#[test]
fn named_wire_round_trips_typed_modifiers_and_rejects_unknown_values() {
    let method = CoreOp::MethodModifiers(
        "M1".into(),
        vec![DeclarationModifier::Async, DeclarationModifier::Static],
    );
    let class = CoreOp::ClassModifiers(
        "C1".into(),
        vec![DeclarationModifier::Export, DeclarationModifier::Abstract],
    );

    assert_eq!(op_to_tuple(&method), vec!["MOD_M", "M1", "ASYNC", "STATIC"]);
    assert_eq!(
        op_to_tuple(&class),
        vec!["MOD_C", "C1", "EXPORT", "ABSTRACT"]
    );
    assert_eq!(tuple_to_op(&op_to_tuple(&method)), Some(method));
    assert_eq!(tuple_to_op(&op_to_tuple(&class)), Some(class));
    assert_eq!(
        tuple_to_op(&["MOD_M".into(), "M1".into(), "UNKNOWN".into()]),
        None
    );
}

#[test]
fn checked_projection_preserves_typed_family_boundaries_and_occurrences() {
    let ir = contract_stream();
    let hierarchy = try_ir_to_hierarchical(&ir).expect("identity graph is valid");
    let class = &hierarchy.classes[0];
    let method = &class.methods[0];

    assert_eq!(
        class.modifiers,
        vec![vec![
            DeclarationModifier::Export,
            DeclarationModifier::Abstract
        ]]
    );
    assert_eq!(
        method.modifiers,
        vec![vec![DeclarationModifier::Async, DeclarationModifier::Async]]
    );
    assert_eq!(
        method.control_summaries,
        vec![vec![ControlSummary::Branch, ControlSummary::Return]]
    );
    assert_eq!(
        hierarchical_to_ir(&hierarchy),
        vec![
            CoreOp::DefClass("C1".into(), "Owner".into()),
            CoreOp::ClassModifiers(
                "C1".into(),
                vec![DeclarationModifier::Export, DeclarationModifier::Abstract],
            ),
            CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
            CoreOp::MethodModifiers(
                "M1".into(),
                vec![DeclarationModifier::Async, DeclarationModifier::Async],
            ),
            CoreOp::ControlSummary(
                "M1".into(),
                vec![ControlSummary::Branch, ControlSummary::Return],
            ),
        ]
    );
}

#[test]
fn checked_projection_rejects_declaration_values_in_residual_flags() {
    for instruction in [
        CoreOp::Flags("M1".into(), vec!["ASYNC".into()]),
        CoreOp::ClassFlags("C1".into(), vec!["EXPORT".into()]),
    ] {
        let error = try_ir_to_hierarchical(&compiled(vec![
            CoreOp::DefClass("C1".into(), "Owner".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
            instruction,
        ]))
        .expect_err("declaration values require typed modifier operations");
        assert!(
            error.to_string().contains("semantic-family operation"),
            "{error}"
        );
    }
}

#[test]
fn additive_binary_opcodes_round_trip_under_physical_version_three() {
    let ir = compiled(vec![
        CoreOp::MethodModifiers(
            "M1".into(),
            vec![DeclarationModifier::Async, DeclarationModifier::Static],
        ),
        CoreOp::ClassModifiers(
            "C1".into(),
            vec![DeclarationModifier::Export, DeclarationModifier::Abstract],
        ),
    ]);
    let bytes = encode(&ir);
    assert_eq!(bytes[2], 0x04, "Phase 8 corrected physical version");
    let decoded = decode(&bytes).expect("binary modifier round trip");
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);
}
