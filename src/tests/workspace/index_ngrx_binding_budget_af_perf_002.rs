//! Approved original-position no-family invariant; real add_edges operation.
use super::{dispatches, fixture};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::work_observations_af_perf_002 as work;
use crate::workspace::index::WorkspaceIndex;

fn binding(n: usize, file: &str) -> SemanticEdge {
    SemanticEdge {
        subject: EntityRef::new("typescript", "ObjectMember", format!("Map.key{n}"))
            .with_file(file.into()),
        object: EntityRef::new("typescript", "Value", format!("Value{n}")).with_file(file.into()),
        relation: SemanticRelation::ReferencesValue,
        layer: crate::ir::typescript_object_members::LAYER,
        call_evidence: None,
    }
}

#[test]
fn af173_binding_budget_no_family_zero_work() {
    let mut failures = Vec::new();
    for n in [1, 2, 4, 8, 16, 64, 256] {
        for direct in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let (mut index, file) = if direct {
                fixture(root.path(), 1)
            } else {
                (
                    WorkspaceIndex::new(),
                    root.path()
                        .join("bindings.ts")
                        .to_string_lossy()
                        .into_owned(),
                )
            };
            index.add_edges(&file, (0..n).map(|i| binding(i, &file)).collect());
            work::reset();
            let start = std::time::Instant::now();
            index.add_edges(&file, vec![]);
            let counts = work::snapshot();
            assert_eq!(
                counts.reconciliation, 1,
                "reconstruction must still run immediately"
            );
            assert_eq!(
                dispatches(&index),
                usize::from(direct),
                "unrelated supported dispatch survives"
            );
            eprintln!(
                "AF173_BINDING_NO_FAMILY n={n} direct={direct} collections={} visits={} records={} clone_bytes={} prepass_checks={} export_reads={} export_parses={} us={}",
                counts.ngrx_binding_collections,
                counts.ngrx_binding_edge_visits,
                counts.ngrx_binding_records,
                counts.ngrx_binding_string_bytes,
                counts.ngrx_binding_demand_checks,
                counts.export_reads,
                counts.export_parses,
                start.elapsed().as_micros()
            );
            if counts.ngrx_binding_collections != 0
                || counts.ngrx_binding_edge_visits != 0
                || counts.ngrx_binding_records != 0
                || counts.ngrx_binding_string_bytes != 0
            {
                failures.push((
                    n,
                    direct,
                    counts.ngrx_binding_collections,
                    counts.ngrx_binding_edge_visits,
                    counts.ngrx_binding_string_bytes,
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "AF173 unused binding collection: {failures:?}"
    );
}

#[test]
fn af173_binding_budget_demand_once_fresh_and_mixed() {
    for mode in [0, 1, 2] {
        let root = tempfile::tempdir().unwrap();
        let file = root
            .path()
            .join("bindings.ts")
            .to_string_lossy()
            .into_owned();
        let caller = root.path().join("caller.ts").to_string_lossy().into_owned();
        let mut index = WorkspaceIndex::new();
        let mut candidates = Vec::new();
        if mode != 1 {
            candidates.push(SemanticEdge {
                subject: EntityRef::new("ngrx", "StoreRegistration", "StoreModule.forRoot"),
                object: EntityRef::new("typescript", "ObjectMap", "Map"),
                relation: SemanticRelation::RegistersReducer,
                layer: crate::angular_meta::ngrx::object_registration::MAP_LAYER,
                call_evidence: None,
            });
        }
        if mode != 0 {
            candidates.push(SemanticEdge {
                subject: EntityRef::new("ngrx", "Action", "Missing"),
                object: EntityRef::new("ngrx", "Reducer", "MissingReducer"),
                relation: SemanticRelation::TriggersReducer,
                layer: crate::angular_meta::ngrx::object_actions::SWITCH_VALUE_LAYER,
                call_evidence: None,
            });
        }
        index.add_edges(&caller, candidates);
        for records in [1, 2] {
            work::reset();
            index.add_edges(&file, vec![binding(records, &file)]);
            let counts = work::snapshot();
            assert_eq!(counts.reconciliation, 1);
            assert_eq!(
                counts.ngrx_binding_collections, 1,
                "one original-position collection per demanding operation"
            );
            assert_eq!(
                counts.ngrx_binding_records, records,
                "fresh authority facts on each call"
            );
            assert_eq!(
                dispatches(&index),
                0,
                "family presence is not proof; invalid targets still refuse"
            );
            eprintln!(
                "AF173_BINDING_DEMAND mode={mode} records={records} collections={} visits={} clone_bytes={} prepass_checks={}",
                counts.ngrx_binding_collections,
                counts.ngrx_binding_edge_visits,
                counts.ngrx_binding_string_bytes,
                counts.ngrx_binding_demand_checks
            );
        }
    }
}
