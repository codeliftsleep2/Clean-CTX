//! Intermediate replay work and late-refusal ownership; no optimization budget.
use crate::ir::delta::{SequenceDeltaComputer, SequenceEdit};
use crate::ir::replay::ContextState;
use crate::ir::wire::op_to_tuple;
use crate::ir::{CompiledIR, CoreOp};
use crate::work_observations_af_perf_002 as work;
fn baseline(methods: usize) -> CompiledIR {
    let mut instructions = vec![CoreOp::DefClass("C".into(), "Owner".into())];
    instructions.extend(
        (0..methods).map(|m| CoreOp::DefMethod("C".into(), format!("M{m}"), format!("run{m}"))),
    );
    CompiledIR {
        file_id: "replay-work".into(),
        version: 1,
        instructions,
    }
}
#[test]
fn af002_replay_intermediate_growth() {
    let mut axes: Vec<_> = [1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|n| (n, 1))
        .collect();
    axes.extend([2, 4, 8, 16, 64, 256].into_iter().map(|n| (1, n)));
    axes.push((16, 16));
    for (methods, deltas) in axes {
        for position in [1, methods] {
            let initial = baseline(methods);
            let mut current = initial.clone();
            let mut history = Vec::new();
            for step in 0..deltas {
                let mut next = current.clone();
                next.version += 1;
                next.instructions[position] = CoreOp::DefMethod(
                    "C".into(),
                    format!("M{}", position - 1),
                    format!("revision{step}"),
                );
                history.push(
                    SequenceDeltaComputer::new()
                        .compute(&current, &next)
                        .unwrap(),
                );
                current = next;
            }
            let mut state = ContextState::new();
            state.load_ir(initial.clone(), Some("baseline-hash".into()));
            work::reset();
            let started = std::time::Instant::now();
            for delta in history {
                state.apply_sequence(delta).unwrap();
            }
            let elapsed = started.elapsed().as_micros();
            let expected: Vec<_> = current.instructions.iter().map(op_to_tuple).collect();
            assert_eq!(state.get_ir(&initial.file_id), Some(&expected));
            assert_eq!(state.file_version(&initial.file_id), Some(current.version));
            assert_eq!(state.version(), current.version);
            let c = work::snapshot();
            eprintln!(
                "AF002_REPLAY methods={methods} instructions={} deltas={deltas} changed_position={position} calls={} candidate_clones={} candidate_tuples={} candidate_string_bytes={} full_validation_passes={} full_validation_visits={} all_tuple_validations={} index_rebuilds={} index_visits={} occurrence_prefix_visits={} commits={} final_output_tuple_clones={} us={elapsed} boundary=context-apply-sequence",
                initial.instructions.len(),
                c.replay_sequence_calls,
                c.replay_candidate_clones,
                c.replay_candidate_tuples,
                c.replay_candidate_string_bytes,
                c.replay_full_validation_passes,
                c.replay_full_validation_tuple_visits,
                c.replay_tuple_validation_calls,
                c.replay_index_rebuilds,
                c.replay_index_tuple_visits,
                c.replay_occurrence_prefix_visits,
                c.replay_sequence_commits,
                c.sqlite_output_tuple_clones
            );
        }
    }
}
fn assert_unchanged(actual: &ContextState, expected: &ContextState) {
    assert_eq!(actual.version, expected.version);
    assert_eq!(actual.source_hashes, expected.source_hashes);
    assert_eq!(actual.files.len(), expected.files.len());
    for (key, file) in &expected.files {
        let actual = &actual.files[key];
        assert_eq!(actual.instructions, file.instructions);
        assert_eq!(actual.version, file.version);
        assert_eq!(actual.index, file.index);
        assert_eq!(actual.occurrence_index, file.occurrence_index);
    }
}
#[test]
fn af002_replay_late_failure_atomicity() {
    let initial = baseline(4);
    let mut target = initial.clone();
    target.version = 2;
    target.instructions[1] = CoreOp::DefMethod("C".into(), "M0".into(), "first_changed".into());
    target.instructions[2] = CoreOp::DefMethod("C".into(), "M1".into(), "second_changed".into());
    let valid = SequenceDeltaComputer::new()
        .compute(&initial, &target)
        .unwrap();
    assert_eq!(
        valid.edits.len(),
        2,
        "late-failure fixture requires a prior successful edit"
    );
    for mode in [
        "expected",
        "occurrence",
        "malformed",
        "version",
        "unsupported",
        "missing",
        "nonmonotonic",
    ] {
        let mut state = ContextState::new();
        state.load_ir(initial.clone(), Some("owner-hash".into()));
        let mut peer = baseline(1);
        peer.file_id = "peer".into();
        peer.version = 7;
        state.load_ir(peer, Some("peer-hash".into()));
        let snapshot = state.clone();
        let mut delta = valid.clone();
        match mode {
            "version" => delta.from = 99,
            "unsupported" => delta.delta_version = 255,
            "missing" => delta.file = "missing".into(),
            "nonmonotonic" => delta.to = delta.from,
            _ => {
                let SequenceEdit::Replace {
                    expected,
                    value,
                    replacement,
                    ..
                } = delta.edits.last_mut().unwrap()
                else {
                    panic!("expected replacement");
                };
                match mode {
                    "expected" => expected.push("conflict".into()),
                    "occurrence" => value.occurrence += 1,
                    _ => {
                        replacement.pop();
                    }
                }
            }
        }
        work::reset();
        let error = state.apply_sequence(delta).unwrap_err();
        let c = work::snapshot();
        assert_unchanged(&state, &snapshot);
        assert_eq!(c.replay_sequence_commits, 0);
        assert_eq!(c.replay_index_rebuilds, 0);
        assert_eq!(
            c.replay_candidate_clones,
            usize::from(matches!(mode, "expected" | "occurrence" | "malformed"))
        );
        eprintln!(
            "AF002_REPLAY_REFUSAL mode={mode} candidate_clones={} candidate_tuples={} validations={} index_rebuilds={} commits={} error={error:?}",
            c.replay_candidate_clones,
            c.replay_candidate_tuples,
            c.replay_tuple_validation_calls,
            c.replay_index_rebuilds,
            c.replay_sequence_commits
        );
        state.apply_sequence(valid.clone()).unwrap();
        assert_eq!(state.file_version(&initial.file_id), Some(2));
        assert_eq!(
            state.version(),
            7,
            "peer owner keeps global version monotonic"
        );
        let expected: Vec<_> = target.instructions.iter().map(op_to_tuple).collect();
        assert_eq!(state.get_ir(&initial.file_id), Some(&expected));
        assert_eq!(
            state.files["peer"].instructions,
            snapshot.files["peer"].instructions
        );
    }
}
