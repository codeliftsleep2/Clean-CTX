//! #174 intended linear-work RED/GREEN budgets at real public insertion boundaries.
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::work_observations_af_perf_002 as work;
use crate::workspace::index::WorkspaceIndex;
use std::time::Instant;
fn record(kind: &'static str, name: &str, file: &str) -> SemanticEdge {
    let entity = EntityRef::new("af174", kind, name).with_file(file.to_owned());
    SemanticEdge {
        subject: entity.clone(),
        object: entity,
        relation: SemanticRelation::Defines,
        layer: "af174",
        call_evidence: None,
    }
}
fn kind(i: usize) -> &'static str {
    static KINDS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    KINDS.get_or_init(|| (0..256).map(|i| format!("Kind{i}")).collect())[i].as_str()
}
fn measured(dimension: &str, n: usize, action: impl FnOnce(&mut WorkspaceIndex)) {
    let mut index = WorkspaceIndex::new();
    work::reset();
    let start = Instant::now();
    action(&mut index);
    let counts = work::snapshot();
    eprintln!(
        "AF174 dimension={dimension} n={n} registrations={} comparisons={:?} probes={:?} us={}",
        counts.registrations,
        counts.comparisons,
        counts.membership_probes,
        start.elapsed().as_micros()
    );
    assert_eq!(counts.registrations, n, "fixture participation count");
    for bucket in 0..3 {
        let units = counts.comparisons[bucket] + counts.membership_probes[bucket];
        // One name/occurrence probe per registration and one publisher membership
        // decision per participating key. Permit two visits for delegation plumbing;
        // never a bound proportional to the accumulated bucket length.
        assert!(
            units <= 2 * n,
            "AF174 quadratic membership dimension={dimension} bucket={bucket} n={n}: units={units}, linear_budget={}",
            2 * n
        );
    }
}
#[test]
fn af174_budget_occurrence_growth() {
    for n in [1, 2, 4, 8, 16, 64, 256] {
        measured("physical-occurrences", n, |index| {
            for i in 0..n {
                let file = format!("/af174/owner-{i}.ts");
                index.add_edges(&file, vec![record("Node", "Shared", &file)]);
            }
            assert_eq!(
                index.entities_by_identity("af174", "Node", "Shared").len(),
                n
            );
        });
    }
}
#[test]
fn af174_budget_name_identity_growth() {
    for n in [1, 2, 4, 8, 16, 64, 256] {
        measured("same-name-identities", n, |index| {
            for i in 0..n {
                let file = format!("/af174/owner-{i}.ts");
                index.add_edges(&file, vec![record(kind(i), "Shared", &file)]);
            }
            for i in 0..n {
                assert_eq!(
                    index.entities_by_identity("af174", kind(i), "Shared").len(),
                    1
                );
            }
        });
    }
}
#[test]
fn af174_budget_publisher_unique_key_growth() {
    for n in [1, 2, 4, 8, 16, 64, 256] {
        measured("publisher-unique-keys", n, |index| {
            index.add_edges(
                "/af174/publisher.ts",
                (0..n)
                    .map(|i| record("Node", &format!("Unique{i}"), "/af174/publisher.ts"))
                    .collect(),
            );
            let keys = index.file_map().get("/af174/publisher.ts").unwrap();
            assert_eq!(
                keys.iter().map(|k| k.2.clone()).collect::<Vec<_>>(),
                (0..n).map(|i| format!("Unique{i}")).collect::<Vec<_>>()
            );
        });
    }
}
#[test]
fn af174_existing_duplicate_order_and_clone_behavior() {
    let mut index = WorkspaceIndex::new();
    let file = "/af174/ordered.ts";
    index.add_edges(
        file,
        vec![
            record("Node", "B", file),
            record("Node", "A", file),
            record("Node", "B", file),
        ],
    );
    let names = |index: &WorkspaceIndex| {
        index
            .file_map()
            .get(file)
            .unwrap()
            .iter()
            .map(|k| k.2.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(names(&index), vec!["B", "A"]);
    let mut clone = index.clone();
    clone.add_edges(file, vec![record("Node", "C", file)]);
    assert_eq!(names(&clone), vec!["B", "A", "C"]);
    assert_eq!(names(&index), vec!["B", "A"]);
    clone.remove_file(file);
    assert!(!clone.file_map().contains_key(file));
    clone.add_edges(
        file,
        vec![record("Node", "A", file), record("Node", "B", file)],
    );
    assert_eq!(names(&clone), vec!["A", "B"]);
}
