//! Real SQLite replay work; fixtures are outside the observation interval.
use crate::compression::Fidelity;
use crate::ir::{CompiledIR, CoreOp};
use crate::mcp::context_store::ContextStore;
use crate::mcp::sqlite_store::SqliteStore;
use crate::work_observations_af_perf_002 as work;
use std::path::Path;
fn baseline(methods: usize) -> CompiledIR {
    let mut instructions = vec![CoreOp::DefClass("C".into(), "Owner".into())];
    instructions.extend(
        (0..methods).map(|m| CoreOp::DefMethod("C".into(), format!("M{m}"), format!("run{m}"))),
    );
    CompiledIR {
        file_id: "/sqlite-work.ts".into(),
        version: 1,
        instructions,
    }
}
#[test]
fn af002_sqlite_instruction_delta_dimensions() {
    let mut axes: Vec<_> = [1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|n| (n, 1))
        .collect();
    axes.extend([2, 4, 8, 16, 64, 256].into_iter().map(|n| (1, n)));
    axes.push((16, 16));
    for (methods, deltas) in axes {
        let mut store = SqliteStore::open(Path::new(":memory:")).unwrap();
        let initial = baseline(methods);
        let binary = crate::ir::binary_wire::encode(&initial);
        let id = store
            .save_context(
                &initial.file_id,
                Fidelity::High,
                "baseline",
                Some(&binary),
                "fixture-hash",
                0,
                0,
            )
            .unwrap();
        let mut current = initial.clone();
        for step in 1..=deltas {
            let mut next = current.clone();
            next.version += 1;
            next.instructions[1] =
                CoreOp::DefMethod("C".into(), "M0".into(), format!("revision{step}"));
            let delta = crate::ir::delta::SequenceDeltaComputer::new()
                .compute(&current, &next)
                .unwrap();
            store
                .append_delta(
                    &id,
                    &serde_json::to_vec(&delta).unwrap(),
                    Some("sequence_v2"),
                )
                .unwrap();
            current = next;
        }
        for target in [Some(0), None] {
            work::reset();
            let started = std::time::Instant::now();
            let (actual, version) = store
                .load_context_with_deltas(&initial.file_id, target)
                .unwrap()
                .unwrap();
            let elapsed = started.elapsed().as_micros();
            let expected = if target == Some(0) {
                &initial
            } else {
                &current
            };
            assert_eq!(actual.file_id, expected.file_id);
            assert_eq!(actual.version, expected.version);
            assert_eq!(version as u64, expected.version);
            assert_eq!(actual.instructions, expected.instructions);
            let c = work::snapshot();
            eprintln!(
                "AF002_SQLITE methods={methods} deltas={deltas} target={target:?} output_instructions={} baseline_queries={} owner_queries={} decoded_bytes={} delta_rows={} delta_payload_bytes={} output_tuple_clones={} us={elapsed} boundary=canonical-replay",
                actual.instructions.len(),
                c.sqlite_baseline_queries,
                c.sqlite_owner_queries,
                c.sqlite_decode_bytes,
                c.sqlite_delta_rows,
                c.sqlite_delta_payload_bytes,
                c.sqlite_output_tuple_clones
            );
            eprintln!(
                "AF002_SQLITE_INTERMEDIATE methods={methods} deltas={deltas} target={target:?} candidate_clones={} candidate_tuples={} candidate_string_bytes={} full_validation_passes={} full_validation_visits={} all_tuple_validations={} index_rebuilds={} index_visits={} occurrence_prefix_visits={} sequence_commits={} final_output_tuple_clones={}",
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
#[test]
fn af002_sqlite_enclosing_durable_baseline() {
    for methods in [1, 2, 4, 8, 16, 64, 256] {
        let mut store = SqliteStore::open(Path::new(":memory:")).unwrap();
        let ir = baseline(methods);
        store
            .save_context_with_semantics(
                &ir.file_id,
                Fidelity::High,
                "baseline",
                &crate::ir::binary_wire::encode(&ir),
                "fixture-hash",
                1,
                &[],
                0,
                0,
            )
            .unwrap();
        work::reset();
        let started = std::time::Instant::now();
        let actual = store
            .load_durable_context(&ir.file_id, None)
            .unwrap()
            .unwrap();
        let elapsed = started.elapsed().as_micros();
        assert_eq!(actual.ir.instructions, ir.instructions);
        assert_eq!(actual.ir.file_id, ir.file_id);
        assert_eq!(actual.ir.version, 1);
        assert!(actual.semantic_edges.is_empty());
        assert_eq!(actual.source_hash, "fixture-hash");
        assert_eq!(actual.fidelity, Fidelity::High);
        let c = work::snapshot();
        eprintln!(
            "AF002_SQLITE_DURABLE methods={methods} baseline_queries={} owner_queries={} decoded_bytes={} delta_rows={} output_tuple_clones={} us={elapsed}",
            c.sqlite_baseline_queries,
            c.sqlite_owner_queries,
            c.sqlite_decode_bytes,
            c.sqlite_delta_rows,
            c.sqlite_output_tuple_clones
        );
        work::reset();
        assert!(
            store
                .load_durable_context("/missing.ts", None)
                .unwrap()
                .is_none()
        );
        let absent = work::snapshot();
        eprintln!(
            "AF002_SQLITE_ABSENT baseline_queries={} owner_queries={}",
            absent.sqlite_baseline_queries, absent.sqlite_owner_queries
        );
    }
}

#[path = "sqlite_text_design_af_perf_002.rs"]
mod text_design;

#[path = "sqlite_text_repair_af_perf_002.rs"]
mod text_repair;

#[path = "sqlite_text_integrated_af_perf_002.rs"]
mod text_integrated;

#[path = "startup_listing_discovery_af_perf_002.rs"]
mod startup_listing_discovery;

#[path = "startup_projection_design_af_perf_002.rs"]
mod startup_projection_design;
