use crate::compression::Fidelity;
use crate::ir::binary_wire::{BinaryDecodeError, decode, encode};
use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{
    HierarchicalProjectionError, ProjectionIdentityKind, ir_to_hierarchical_wire,
    try_ir_to_hierarchical, wire_to_ir as hierarchical_wire_to_ir,
};
use crate::ir::opcodes::{CoreOp, SideEffectKind};
use crate::ir::render_llm::render_hierarchical_for_llm;
use crate::ir::wire::{op_to_tuple, tuple_to_op};

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "side-effect-contract".into(),
        version: 1,
        instructions,
    }
}

#[test]
fn side_effect_vocabulary_and_named_wire_are_closed() {
    let values = [
        SideEffectKind::Pure,
        SideEffectKind::Io,
        SideEffectKind::Mutation,
        SideEffectKind::Async,
        SideEffectKind::Transaction,
    ];
    for effect in values {
        let operation = CoreOp::SideEffect("M1".into(), effect);
        let tuple = op_to_tuple(&operation);
        assert_eq!(
            tuple,
            vec!["EFFECT".to_string(), "M1".to_string(), effect.to_string()]
        );
        assert_eq!(tuple_to_op(&tuple), Some(operation));
    }
    assert_eq!(
        tuple_to_op(&["EFFECT".into(), "M1".into(), "network".into()]),
        None
    );
}

#[test]
fn binary_wire_preserves_typed_side_effect_and_rejects_unknown_value() {
    let ir = compiled(vec![CoreOp::SideEffect("M1".into(), SideEffectKind::Io)]);
    let bytes = encode(&ir);
    assert_eq!(bytes[2], 0x04, "Phase 8 corrected physical version");
    assert_eq!(
        decode(&bytes).expect("binary round trip").instructions,
        ir.instructions
    );

    let mut malformed = bytes;
    let value = malformed
        .windows(2)
        .position(|window| window == b"io")
        .expect("encoded io vocabulary value");
    malformed[value..value + 2].copy_from_slice(b"zz");
    assert!(matches!(
        decode(&malformed),
        Err(BinaryDecodeError::InvalidSideEffect(value)) if value == "zz"
    ));
}

#[test]
fn hierarchical_wire_rejects_unknown_side_effect_value() {
    let wire = serde_json::json!({
        "file": "invalid", "v": 1, "encoding": "hierarchical", "hs": 6,
        "ir": { "c": [{
            "n": "C1", "nm": "Owner", "m": [{
                "n": "M1", "nm": "work", "se": ["network"]
            }]
        }]}
    });
    let error = hierarchical_wire_to_ir(&wire).expect_err("unknown side effect must fail");
    assert!(error.to_string().contains("network"), "{error}");
}

#[test]
fn hierarchy_preserves_side_effect_order_duplicates_and_llm_shape() {
    let ir = compiled(vec![
        CoreOp::SideEffect("M1".into(), SideEffectKind::Io),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
        CoreOp::SideEffect("M1".into(), SideEffectKind::Io),
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::SideEffect("M1".into(), SideEffectKind::Mutation),
    ]);
    let hierarchy = try_ir_to_hierarchical(&ir).expect("valid identity graph");
    assert_eq!(
        hierarchy.classes[0].methods[0].side_effect,
        vec![
            SideEffectKind::Io,
            SideEffectKind::Io,
            SideEffectKind::Mutation
        ]
    );
    assert_eq!(ir_to_hierarchical_wire(&ir)["hs"], 8);
    let rendered = render_hierarchical_for_llm(&hierarchy, Fidelity::High);
    assert!(rendered.contains(" se:io,io,mutation"), "{rendered}");
}

#[test]
fn checked_projection_rejects_unresolved_and_wrong_kind_side_effect_targets() {
    let unresolved = try_ir_to_hierarchical(&compiled(vec![CoreOp::SideEffect(
        "M1".into(),
        SideEffectKind::Io,
    )]))
    .expect_err("unknown method must fail");
    assert!(matches!(
        unresolved,
        HierarchicalProjectionError::UnresolvedIdentity {
            operation: "EFFECT",
            expected: ProjectionIdentityKind::Method,
            ref id,
            instruction: 0,
        } if id == "M1"
    ));

    let wrong_kind = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::SideEffect("C1".into(), SideEffectKind::Io),
    ]))
    .expect_err("class identity cannot target a method fact");
    assert!(matches!(
        wrong_kind,
        HierarchicalProjectionError::KindMismatch {
            operation: "EFFECT",
            ref id,
            expected: ProjectionIdentityKind::Method,
            actual: ProjectionIdentityKind::Class,
            instruction: 1,
        } if id == "C1"
    ));
}
