//! Characterization, not approval to change declaration authority or reconciliation.
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::work_observations_af_perf_002 as work;
use crate::workspace::index::WorkspaceIndex;

fn declaration(name: &str, file: &str) -> SemanticEdge {
    let entity = EntityRef::new("af002", "Node", name).with_file(file.to_owned());
    SemanticEdge {
        subject: entity.clone(),
        object: entity,
        relation: SemanticRelation::Defines,
        layer: "af002-declaration-characterization",
        call_evidence: None,
    }
}

#[test]
fn af002_enclosing_declaration_growth() {
    for n in [1, 2, 4, 8, 16, 64, 256] {
        for same_identity in [false, true] {
            let mut index = WorkspaceIndex::new();
            work::reset();
            let started = std::time::Instant::now();
            for i in 0..n {
                let file = if same_identity {
                    format!("/af002/{i}.ts")
                } else {
                    "/af002/one.ts".into()
                };
                let name = if same_identity {
                    "Shared".into()
                } else {
                    format!("Node{i}")
                };
                index.add_edges(&file, vec![declaration(&name, &file)]);
            }
            let counts = work::snapshot();
            let dimension = usize::from(!same_identity);
            assert_eq!(counts.declaration_comparisons[dimension], n * (n - 1) / 2);
            assert_eq!(counts.declaration_comparisons[1 - dimension], 0);
            assert_eq!(counts.reconciliation, n);
            assert_eq!(counts.candidates, 0);
            assert_eq!(
                index.declarations_by_name("Shared").len(),
                if same_identity { n } else { 0 }
            );
            eprintln!(
                "AF002_ENCLOSING n={n} same_identity={same_identity} declaration_comparisons={:?} member_probes={:?} reconciliation={} us={}",
                counts.declaration_comparisons,
                counts.membership_probes,
                counts.reconciliation,
                started.elapsed().as_micros()
            );
        }
    }
}

#[test]
fn af002_enclosing_noop_and_duplicate_reconciliation() {
    let mut index = WorkspaceIndex::new();
    let file = "/af002/owner.ts";
    let edge = declaration("Owner", file);
    index.add_edges(file, vec![edge.clone()]);
    for n in [1, 2, 4, 8, 16, 64, 256] {
        for duplicate in [false, true] {
            work::reset();
            for _ in 0..n {
                index.add_edges(
                    file,
                    if duplicate {
                        vec![edge.clone()]
                    } else {
                        vec![]
                    },
                );
            }
            let counts = work::snapshot();
            assert_eq!(counts.reconciliation, n);
            assert_eq!(counts.candidates, 0);
            assert_eq!(index.declarations_by_name("Owner").len(), 1);
            eprintln!(
                "AF002_NOOP calls={n} duplicate={duplicate} reconciliation={} declaration_comparisons={:?}",
                counts.reconciliation, counts.declaration_comparisons
            );
        }
    }
}
