//! AF-PERF-002 characterization, not approved future complexity budgets.
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::work_observations_af_perf_002 as work;
use crate::workspace::index::WorkspaceIndex;
fn edge(name: &str, file: &str) -> SemanticEdge {
    SemanticEdge {
        subject: EntityRef::new("test", "Node", name).with_file(file.to_owned()),
        object: EntityRef::new("test", "Node", name).with_file(file.to_owned()),
        relation: SemanticRelation::Defines,
        layer: "test",
        call_evidence: None,
    }
}
#[test]
fn af_perf_002_index_uniqueness_density_and_root_removal_baseline() {
    for n in [1, 2, 4, 8, 16] {
        for density in [1, 4] {
            let mut index = WorkspaceIndex::new();
            work::reset();
            let started = std::time::Instant::now();
            for i in 0..n {
                let file = format!("/af-perf-002/file-{i}.ts");
                index.add_edges(&file, (0..density).map(|_| edge("Shared", &file)).collect());
            }
            let counts = work::snapshot();
            assert_eq!(
                index.entities_by_identity("test", "Node", "Shared").len(),
                n
            );
            assert_eq!(counts.registrations, n * density);
            eprintln!(
                "AF002_INDEX files={n} duplicate_density={density} us={} counts={counts:?}",
                started.elapsed().as_micros()
            );
            work::reset();
            assert_eq!(index.remove_files_in_root("/af-perf-002"), n);
            let counts = work::snapshot();
            assert!(
                index
                    .entities_by_identity("test", "Node", "Shared")
                    .is_empty()
            );
            eprintln!("AF002_REMOVE files={n} counts={counts:?}");
        }
        let mut index = WorkspaceIndex::new();
        work::reset();
        index.add_edges(
            "/af-perf-002/unique.ts",
            (0..n)
                .map(|i| edge(&format!("Unique{i}"), "/af-perf-002/unique.ts"))
                .collect(),
        );
        eprintln!("AF002_UNIQUE entities={n} counts={:?}", work::snapshot());
    }
}
#[cfg(feature = "typescript")]
#[test]
fn af_perf_002_exact_witness_expectations_and_io_baseline() {
    use crate::ir::typescript_resolution::Witness;
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("source.ts");
    std::fs::write(&file, "export const A=1;").unwrap();
    let hash = crate::cache::content_hash(b"export const A=1;");
    let valid = Witness::Content {
        path: file.clone(),
        hash,
    };
    let wrong = Witness::Content {
        path: file.clone(),
        hash: "wrong".into(),
    };
    for n in [1, 2, 4, 8, 16] {
        work::reset();
        for _ in 0..n {
            assert!(valid.current());
            assert!(!wrong.current());
        }
        let counts = work::snapshot();
        assert_eq!(counts.witnesses, 2 * n);
        assert_eq!(counts.identities.len(), 2);
        assert_eq!(counts.reads, 2 * n);
        assert_eq!(counts.hashes, 2 * n);
        eprintln!("AF002_WITNESS repeats={n} counts={counts:?}");
    }
    work::reset();
    assert!(
        Witness::Missing {
            path: root.path().join("missing")
        }
        .current()
    );
    assert!(
        Witness::Kind {
            path: file.clone(),
            file: true,
            directory: false
        }
        .current()
    );
    assert!(
        Witness::Physical {
            path: file.clone(),
            target: file.canonicalize().unwrap()
        }
        .current()
    );
    assert!(!Witness::Unreadable { path: file.clone() }.current());
    assert!(!Witness::OutsideScope { path: file }.current());
    let mut entries = std::fs::read_dir(root.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    assert!(
        Witness::Directory {
            path: root.path().to_path_buf(),
            entries
        }
        .current()
    );
    let counts = work::snapshot();
    assert_eq!(counts.enumerations, 1);
    assert_eq!(counts.metadata, 2);
    assert_eq!(counts.canonicalizations, 1);
    eprintln!("AF002_TYPED_VARIANTS counts={counts:?}");
}

#[test]
fn af_perf_002_populated_reconciliation_and_survivors() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("action.ts");
    std::fs::write(&target, "export const Act=1;").unwrap();
    let target = crate::dictionary::path::canonical_identity_key(
        &target.canonicalize().unwrap().to_string_lossy(),
    );
    for n in [1, 2, 4, 8, 16] {
        let mut index = WorkspaceIndex::new();
        let action = EntityRef::new("ngrx", "Action", "Act").with_file(target.clone());
        index.add_edges(
            &target,
            vec![SemanticEdge {
                subject: action.clone(),
                object: action,
                relation: SemanticRelation::Defines,
                layer: "ngrx",
                call_evidence: None,
            }],
        );
        assert_eq!(
            index
                .declarations_by_identity("ngrx", "Action", "Act")
                .len(),
            1,
            "fixture requires independently registered exact target declaration"
        );
        let mut callers = Vec::new();
        work::reset();
        for i in 0..n {
            let file = root
                .path()
                .join(format!("caller-{i}.ts"))
                .to_string_lossy()
                .into_owned();
            callers.push(file.clone());
            index.add_edges(
                &file,
                vec![SemanticEdge {
                    subject: EntityRef::new("angular", "Component", format!("Caller{i}")),
                    object: EntityRef::new("ngrx", "Action", "Act")
                        .with_file("./action".to_owned()),
                    relation: SemanticRelation::Dispatches,
                    layer: crate::angular_meta::ngrx::direct_imports::REFINEMENT_LAYER,
                    call_evidence: None,
                }],
            );
        }
        assert_eq!(
            index
                .reverse_edges_by_identity("ngrx", "Action", "Act")
                .iter()
                .filter(|e| e.relation == SemanticRelation::Dispatches)
                .count(),
            n,
            "populated refinement fixture"
        );
        eprintln!("AF002_POPULATED files={n} counts={:?}", work::snapshot());
        work::reset();
        index.remove_file(&callers[0]);
        assert_eq!(
            index
                .reverse_edges_by_identity("ngrx", "Action", "Act")
                .iter()
                .filter(|e| e.relation == SemanticRelation::Dispatches)
                .count(),
            n - 1,
            "independent callers survive"
        );
        eprintln!(
            "AF002_POPULATED_REMOVE files={n} counts={:?}",
            work::snapshot()
        );
        work::reset();
        index.remove_file(&target);
        assert_eq!(
            index
                .reverse_edges_by_identity("ngrx", "Action", "Act")
                .iter()
                .filter(|e| e.relation == SemanticRelation::Dispatches)
                .count(),
            0,
            "stale target retirement"
        );
        eprintln!(
            "AF002_POPULATED_TARGET_RETIRE files={n} counts={:?}",
            work::snapshot()
        );
    }
}
