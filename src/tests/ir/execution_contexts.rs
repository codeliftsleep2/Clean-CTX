use crate::compression::Fidelity;
use crate::ir::binary_wire::{BinaryDecodeError, decode, encode};
use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{
    HierarchicalProjectionError, ProjectionIdentityKind, ir_to_hierarchical_wire,
    try_ir_to_hierarchical, wire_to_ir as hierarchical_wire_to_ir,
};
use crate::ir::opcodes::{CoreOp, ExecutionContextKind};
use crate::ir::render_llm::render_hierarchical_for_llm;
use crate::ir::wire::{op_to_tuple, tuple_to_op};

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "execution-context-contract".into(),
        version: 1,
        instructions,
    }
}

#[test]
fn execution_context_vocabulary_and_named_wire_are_closed() {
    let values = [
        ExecutionContextKind::Sync,
        ExecutionContextKind::Async,
        ExecutionContextKind::ThreadBound,
        ExecutionContextKind::TransactionScope,
        ExecutionContextKind::Realtime,
    ];
    for context in values {
        let operation = CoreOp::ExecutionContext("M1".into(), context);
        let tuple = op_to_tuple(&operation);
        assert_eq!(
            tuple,
            vec!["CTX".to_string(), "M1".to_string(), context.to_string()]
        );
        assert_eq!(tuple_to_op(&tuple), Some(operation));
    }
    assert_eq!(
        tuple_to_op(&["CTX".into(), "M1".into(), "di_scope".into()]),
        None
    );
}

#[test]
fn binary_wire_preserves_typed_execution_context_and_rejects_unknown_value() {
    let ir = compiled(vec![CoreOp::ExecutionContext(
        "M1".into(),
        ExecutionContextKind::Async,
    )]);
    let bytes = encode(&ir);
    assert_eq!(bytes[2], 0x04, "Phase 8 corrected physical version");
    assert_eq!(
        decode(&bytes).expect("binary round trip").instructions,
        ir.instructions
    );

    let mut malformed = bytes;
    let value = malformed
        .windows(5)
        .position(|window| window == b"async")
        .expect("encoded async vocabulary value");
    malformed[value..value + 5].copy_from_slice(b"bogus");
    assert!(matches!(
        decode(&malformed),
        Err(BinaryDecodeError::InvalidExecutionContext(value)) if value == "bogus"
    ));
}

#[test]
fn hierarchical_wire_rejects_unknown_execution_context_value() {
    let wire = serde_json::json!({
        "file": "invalid", "v": 1, "encoding": "hierarchical", "hs": 7,
        "ir": { "c": [{
            "n": "C1", "nm": "Owner", "m": [{
                "n": "M1", "nm": "work", "ec": ["di_scope"]
            }]
        }]}
    });
    let error = hierarchical_wire_to_ir(&wire).expect_err("unknown context must fail");
    assert!(error.to_string().contains("di_scope"), "{error}");
}

#[test]
fn hierarchy_preserves_context_order_duplicates_and_compact_llm_shape() {
    let ir = compiled(vec![
        CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::Sync),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
        CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::Sync),
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::Realtime),
    ]);
    let hierarchy = try_ir_to_hierarchical(&ir).expect("valid identity graph");
    assert_eq!(
        hierarchy.classes[0].methods[0].execution_context,
        vec![
            ExecutionContextKind::Sync,
            ExecutionContextKind::Sync,
            ExecutionContextKind::Realtime
        ]
    );
    assert_eq!(ir_to_hierarchical_wire(&ir)["hs"], 7);
    let rendered = render_hierarchical_for_llm(&hierarchy, Fidelity::High);
    assert!(rendered.contains(" ec:sync,sync,realtime"), "{rendered}");
}

#[test]
fn checked_projection_rejects_unresolved_and_wrong_kind_context_targets() {
    let unresolved = try_ir_to_hierarchical(&compiled(vec![CoreOp::ExecutionContext(
        "M1".into(),
        ExecutionContextKind::Async,
    )]))
    .expect_err("unknown method must fail");
    assert!(matches!(
        unresolved,
        HierarchicalProjectionError::UnresolvedIdentity {
            operation: "CTX",
            expected: ProjectionIdentityKind::Method,
            ref id,
            instruction: 0,
        } if id == "M1"
    ));

    let wrong_kind = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::ExecutionContext("C1".into(), ExecutionContextKind::Async),
    ]))
    .expect_err("class identity cannot target a method fact");
    assert!(matches!(
        wrong_kind,
        HierarchicalProjectionError::KindMismatch {
            operation: "CTX",
            ref id,
            expected: ProjectionIdentityKind::Method,
            actual: ProjectionIdentityKind::Class,
            instruction: 1,
        } if id == "C1"
    ));
}
