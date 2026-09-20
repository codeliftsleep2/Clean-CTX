use crate::ir::compiler::CompiledIR;
use crate::ir::delta::{
    DeltaOps, IRDelta, ModOp, SEQUENCE_DELTA_VERSION, SequenceDelta, SequenceDeltaComputer,
    SequenceEdit, compact_sequence_decode, compact_sequence_encode,
};
use crate::ir::opcodes::{CoreOp, ExecutionContextKind, SideEffectKind};
use crate::ir::replay::{ContextState, DeltaError};
use crate::ir::wire::op_to_tuple;
use proptest::prelude::*;

fn compiled(version: u64, facts: Vec<CoreOp>) -> CompiledIR {
    let mut instructions = vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
    ];
    instructions.extend(facts);
    CompiledIR {
        file_id: "sequence.rs".into(),
        version,
        instructions,
    }
}

fn replay(base: &CompiledIR, target: &CompiledIR) -> (SequenceDelta, Vec<Vec<String>>) {
    let delta = SequenceDeltaComputer::new()
        .compute(base, target)
        .expect("different streams produce a delta");
    let mut state = ContextState::new();
    state.load_ir(base.clone(), None);
    state
        .apply_sequence(delta.clone())
        .expect("generated delta must replay");
    (
        delta,
        state.get_ir(&base.file_id).expect("tracked file").clone(),
    )
}

fn target_tuples(target: &CompiledIR) -> Vec<Vec<String>> {
    target.instructions.iter().map(op_to_tuple).collect()
}

#[test]
fn corrected_delta_preserves_duplicates_payloads_order_and_positions() {
    let io = CoreOp::SideEffect("M1".into(), SideEffectKind::Io);
    let mutation = CoreOp::SideEffect("M1".into(), SideEffectKind::Mutation);
    let async_context = CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::Async);
    let base = compiled(1, vec![io.clone(), io.clone(), mutation.clone()]);
    let target = compiled(2, vec![io.clone(), async_context, io.clone(), mutation]);

    let (delta, replayed) = replay(&base, &target);
    assert_eq!(delta.delta_version, SEQUENCE_DELTA_VERSION);
    assert_eq!(replayed, target_tuples(&target));
    assert!(
        delta
            .edits
            .iter()
            .any(|edit| matches!(edit, SequenceEdit::Insert { at: 3, .. }))
    );
}

#[test]
fn corrected_delta_supports_arbitrary_removal_and_duplicate_replacement() {
    let io = CoreOp::SideEffect("M1".into(), SideEffectKind::Io);
    let base = compiled(
        4,
        vec![
            io.clone(),
            io.clone(),
            CoreOp::SideEffect("M1".into(), SideEffectKind::Mutation),
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::Realtime),
        ],
    );
    let target = compiled(
        5,
        vec![
            io,
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::Async),
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::Realtime),
        ],
    );

    let (delta, replayed) = replay(&base, &target);
    assert_eq!(replayed, target_tuples(&target));
    assert!(
        delta
            .edits
            .iter()
            .any(|edit| matches!(edit, SequenceEdit::Remove { at: 4, .. }))
    );
    assert!(delta.edits.iter().any(
        |edit| matches!(edit, SequenceEdit::Replace { at: 3, target, .. } if target.occurrence == 1)
    ));
}

#[test]
fn empty_and_identical_delta_algebra_holds() {
    let base = compiled(
        8,
        vec![CoreOp::ExecutionContext(
            "M1".into(),
            ExecutionContextKind::Sync,
        )],
    );
    assert!(SequenceDeltaComputer::new().compute(&base, &base).is_none());

    let mut state = ContextState::new();
    state.load_ir(base.clone(), None);
    state
        .apply_sequence(SequenceDelta::empty(&base.file_id, 8, 9))
        .expect("empty delta advances only the version");
    assert_eq!(state.get_ir(&base.file_id), Some(&target_tuples(&base)));
}

#[test]
fn corrected_delta_is_deterministic_and_compact_round_trip_is_lossless() {
    let base = compiled(1, vec![]);
    let target = compiled(
        2,
        vec![
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::ThreadBound),
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::ThreadBound),
        ],
    );
    let first = SequenceDeltaComputer::new()
        .compute(&base, &target)
        .unwrap();
    let second = SequenceDeltaComputer::new()
        .compute(&base, &target)
        .unwrap();
    assert_eq!(first, second);
    assert_eq!(
        compact_sequence_decode(&compact_sequence_encode(&first)),
        first
    );
}

#[test]
fn destructive_edit_rejects_an_incorrect_base_transactionally() {
    let base = compiled(1, vec![CoreOp::SideEffect("M1".into(), SideEffectKind::Io)]);
    let target = compiled(
        2,
        vec![CoreOp::SideEffect("M1".into(), SideEffectKind::Mutation)],
    );
    let delta = SequenceDeltaComputer::new()
        .compute(&base, &target)
        .unwrap();
    let conflicting = compiled(
        1,
        vec![CoreOp::SideEffect("M1".into(), SideEffectKind::Async)],
    );
    let original = target_tuples(&conflicting);
    let mut state = ContextState::new();
    state.load_ir(conflicting, None);
    assert!(matches!(
        state.apply_sequence(delta),
        Err(DeltaError::SequenceConflict { .. })
    ));
    assert_eq!(state.get_ir("sequence.rs"), Some(&original));
}

#[test]
fn ambiguous_legacy_modification_fails_structurally() {
    let duplicate = CoreOp::SideEffect("M1".into(), SideEffectKind::Io);
    let base = compiled(1, vec![duplicate.clone(), duplicate]);
    let mut state = ContextState::new();
    state.load_ir(base, None);
    let legacy = IRDelta {
        file: "sequence.rs".into(),
        from: 1,
        to: 2,
        ops: DeltaOps {
            adds: Vec::new(),
            mods: vec![ModOp::new_replace(
                vec!["EFFECT".into(), "M1".into()],
                vec!["EFFECT".into(), "M1".into(), "mutation".into()],
            )],
            dels: Vec::new(),
        },
        intent: None,
    };
    assert!(matches!(
        state.apply(legacy),
        Err(DeltaError::AmbiguousLegacyTarget(key)) if key == "EFFECT:M1"
    ));
}

fn fact_strategy() -> impl Strategy<Value = CoreOp> {
    prop_oneof![
        Just(CoreOp::SideEffect("M1".into(), SideEffectKind::Io)),
        Just(CoreOp::SideEffect("M1".into(), SideEffectKind::Mutation)),
        Just(CoreOp::ExecutionContext(
            "M1".into(),
            ExecutionContextKind::Sync
        )),
        Just(CoreOp::ExecutionContext(
            "M1".into(),
            ExecutionContextKind::Async
        )),
        Just(CoreOp::DataFlow(
            "M1".into(),
            "reads".into(),
            "value".into()
        )),
        Just(CoreOp::ControlFlow(
            "M1".into(),
            "return".into(),
            "value".into()
        )),
    ]
}

proptest! {
    #[test]
    fn valid_canonical_streams_obey_replay_algebra(
        base_facts in prop::collection::vec(fact_strategy(), 0..16),
        target_facts in prop::collection::vec(fact_strategy(), 0..16),
    ) {
        let base = compiled(1, base_facts);
        let target = compiled(2, target_facts);
        match SequenceDeltaComputer::new().compute(&base, &target) {
            None => prop_assert_eq!(target_tuples(&base), target_tuples(&target)),
            Some(delta) => {
                let mut state = ContextState::new();
                state.load_ir(base, None);
                state.apply_sequence(delta).expect("generated delta must apply");
                let expected = target_tuples(&target);
                prop_assert_eq!(state.get_ir("sequence.rs"), Some(&expected));
            }
        }
    }
}
