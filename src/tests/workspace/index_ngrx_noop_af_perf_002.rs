//! Populated no-op characterization and the export-currentness boundary.
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::work_observations_af_perf_002 as work;
use crate::workspace::index::WorkspaceIndex;

fn declaration(target: &str) -> SemanticEdge {
    let action = EntityRef::new("ngrx", "Action", "Act").with_file(target.to_owned());
    SemanticEdge {
        subject: action.clone(),
        object: action,
        relation: SemanticRelation::Defines,
        layer: "ngrx",
        call_evidence: None,
    }
}

fn fixture(root: &std::path::Path, n: usize) -> (WorkspaceIndex, String) {
    let target = root.join("action.ts");
    std::fs::write(&target, "export const Act=1;").unwrap();
    let target = crate::dictionary::path::canonical_identity_key(
        &target.canonicalize().unwrap().to_string_lossy(),
    );
    let mut index = WorkspaceIndex::new();
    index.add_edges(&target, vec![declaration(&target)]);
    for i in 0..n {
        let caller = root
            .join(format!("caller-{i}.ts"))
            .to_string_lossy()
            .into_owned();
        index.add_edges(
            &caller,
            vec![SemanticEdge {
                subject: EntityRef::new("angular", "Component", format!("Caller{i}")),
                object: EntityRef::new("ngrx", "Action", "Act").with_file("./action".to_owned()),
                relation: SemanticRelation::Dispatches,
                layer: crate::angular_meta::ngrx::direct_imports::REFINEMENT_LAYER,
                call_evidence: None,
            }],
        );
    }
    assert_eq!(dispatches(&index), n);
    (index, target)
}

fn dispatches(index: &WorkspaceIndex) -> usize {
    index
        .reverse_edges_by_identity("ngrx", "Action", "Act")
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::Dispatches)
        .count()
}

#[test]
fn af002_ngrx_populated_empty_duplicate_work() {
    for n in [1, 2, 4, 8, 16] {
        let root = tempfile::tempdir().unwrap();
        let (mut index, target) = fixture(root.path(), n);
        for duplicate in [false, true] {
            for calls in [1, 2, 4, 8, 16] {
                work::reset();
                let started = std::time::Instant::now();
                for _ in 0..calls {
                    index.add_edges(
                        &target,
                        if duplicate {
                            vec![declaration(&target)]
                        } else {
                            vec![]
                        },
                    );
                }
                let counts = work::snapshot();
                assert_eq!(counts.reconciliation, calls);
                assert_eq!(counts.candidates, n * calls);
                assert_eq!(dispatches(&index), n);
                eprintln!(
                    "AF002_NGRX_NOOP candidates={n} calls={calls} duplicate={duplicate} rebuilds={} candidates_visited={} retired={} export_reads={} export_parses={} us={}",
                    counts.reconciliation,
                    counts.candidates,
                    counts.retired,
                    counts.export_reads,
                    counts.export_parses,
                    started.elapsed().as_micros()
                );
            }
        }
    }
}

#[test]
fn af002_ngrx_empty_call_observes_changed_export() {
    let root = tempfile::tempdir().unwrap();
    let (mut index, target) = fixture(root.path(), 4);
    // The index mutation is empty, but its consumed filesystem evidence changed.
    std::fs::write(&target, "const Act=1;").unwrap();
    work::reset();
    index.add_edges(&target, vec![]);
    assert_eq!(
        dispatches(&index),
        0,
        "empty insertion must not retain now-unproved export support"
    );
    let refused = work::snapshot();
    std::fs::write(&target, "export const Act=1;").unwrap();
    work::reset();
    index.add_edges(&target, vec![]);
    assert_eq!(
        dispatches(&index),
        4,
        "byte repair restores independently supported callers"
    );
    eprintln!(
        "AF002_NGRX_EMPTY_MUTATION refused={refused:?} repaired={:?}",
        work::snapshot()
    );
}
