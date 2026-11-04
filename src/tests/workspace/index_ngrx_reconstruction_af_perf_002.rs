//! Actual add_edges characterization; deliberately unsupported map/switch candidates refuse.
use super::{declaration, dispatches};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::work_observations_af_perf_002 as work;
use crate::workspace::index::WorkspaceIndex;

fn edge(
    subject: EntityRef,
    object: EntityRef,
    layer: &'static str,
    relation: SemanticRelation,
) -> SemanticEdge {
    SemanticEdge {
        subject,
        object,
        layer,
        relation,
        call_evidence: None,
    }
}

#[test]
fn af002_ngrx_reconstruction_unrelated_binding_and_candidate_axes() {
    // Independent E, B and M axes; no Cartesian test campaign or future repair budget.
    let mut dimensions = Vec::new();
    for n in [1, 2, 4, 8, 16, 64, 256] {
        dimensions.push((n, 0, 0));
        dimensions.push((0, n, 0));
        dimensions.push((0, 16, n));
        dimensions.push((0, n, 16));
    }
    for switch in [false, true] {
        for &(unrelated, bindings, candidates) in &dimensions {
            let root = tempfile::tempdir().unwrap();
            let target = root.path().join("bindings.ts");
            std::fs::write(&target, "export const Map = {}; ").unwrap();
            let target = crate::dictionary::path::canonical_identity_key(
                &target.canonicalize().unwrap().to_string_lossy(),
            );
            let caller = root.path().join("caller.ts").to_string_lossy().into_owned();
            let mut index = WorkspaceIndex::new();
            let mut facts = Vec::new();
            for n in 0..unrelated {
                facts.push(edge(
                    EntityRef::new("fixture", "Method", format!("Caller{n}")),
                    EntityRef::new("fixture", "Method", format!("Callee{n}")),
                    "fixture",
                    SemanticRelation::Calls,
                ));
            }
            for n in 0..bindings {
                facts.push(edge(
                    EntityRef::new("typescript", "ObjectMember", format!("Map.key{n}"))
                        .with_file(target.clone()),
                    EntityRef::new("typescript", "Value", format!("Unrelated{n}"))
                        .with_file(target.clone()),
                    crate::ir::typescript_object_members::LAYER,
                    SemanticRelation::ReferencesValue,
                ));
            }
            index.add_edges(&target, facts);
            for n in 0..candidates {
                let candidate = if switch {
                    edge(
                        EntityRef::new("ngrx", "Action", format!("Missing{n}"))
                            .with_file("./bindings".into()),
                        EntityRef::new("ngrx", "Reducer", "MissingReducer"),
                        crate::angular_meta::ngrx::object_actions::SWITCH_VALUE_LAYER,
                        SemanticRelation::TriggersReducer,
                    )
                } else {
                    edge(
                        EntityRef::new("ngrx", "StoreRegistration", format!("Store{n}")),
                        EntityRef::new("typescript", "ObjectMap", "Map")
                            .with_file("./bindings".into()),
                        crate::angular_meta::ngrx::object_registration::MAP_LAYER,
                        SemanticRelation::RegistersReducer,
                    )
                };
                index.add_edges(&caller, vec![candidate]);
            }
            work::reset();
            let start = std::time::Instant::now();
            index.add_edges(&caller, vec![]);
            let observed = work::snapshot();
            assert_eq!(observed.reconciliation, 1);
            assert_eq!(
                observed.ngrx_binding_collections,
                usize::from(candidates > 0)
            );
            assert_eq!(
                observed.ngrx_binding_records,
                if candidates > 0 { bindings } else { 0 }
            );
            assert_eq!(
                observed.ngrx_binding_edge_visits,
                if candidates > 0 {
                    unrelated + bindings
                } else {
                    0
                }
            );
            assert_eq!(observed.candidates, candidates);
            let visits = if switch {
                observed.ngrx_switch_binding_visits
            } else {
                observed.ngrx_map_binding_visits
            };
            assert_eq!(
                visits,
                candidates * bindings,
                "characterize existing candidate-by-binding work"
            );
            assert_eq!(
                dispatches(&index),
                0,
                "unsupported evidence cannot create dispatch authority"
            );
            assert!(
                index
                    .forward_edges_by_identity("ngrx", "StoreRegistration", "Store0")
                    .is_empty(),
                "no reducer declaration authority means no refined map registration"
            );
            eprintln!(
                "AF002_NGRX_RECON switch={switch} E={unrelated} B={bindings} M={candidates} edge_visits={} records={} clone_string_bytes={} joins={visits} export_reads={} export_parses={} reexport_reads={} reexport_parses={} us={}",
                observed.ngrx_binding_edge_visits,
                observed.ngrx_binding_records,
                observed.ngrx_binding_string_bytes,
                observed.export_reads,
                observed.export_parses,
                observed.ngrx_reexport_reads,
                observed.ngrx_reexport_parses,
                start.elapsed().as_micros()
            );
        }
    }
}

#[test]
fn af002_ngrx_reconstruction_reexport_io_and_retirement() {
    let root = tempfile::tempdir().unwrap();
    let action = root.path().join("action.ts");
    let barrel = root.path().join("barrel.ts");
    std::fs::write(&action, "export const Act=1;").unwrap();
    let bytes = "import { Act } from './action'; export { Act };";
    std::fs::write(&barrel, bytes).unwrap();
    let action = crate::dictionary::path::canonical_identity_key(
        &action.canonicalize().unwrap().to_string_lossy(),
    );
    let caller = root.path().join("caller.ts").to_string_lossy().into_owned();
    let mut index = WorkspaceIndex::new();
    index.add_edges(&action, vec![declaration(&action)]);
    index.add_edges(
        &caller,
        vec![edge(
            EntityRef::new("angular", "Component", "Caller"),
            EntityRef::new("ngrx", "Action", "Act").with_file("./barrel".into()),
            crate::angular_meta::ngrx::direct_imports::REFINEMENT_LAYER,
            SemanticRelation::Dispatches,
        )],
    );
    assert_eq!(
        dispatches(&index),
        1,
        "positive legitimate one-hop reexport prerequisite"
    );
    for (contents, expected) in [(bytes, 1), ("export {};", 0), (bytes, 1)] {
        std::fs::write(&barrel, contents).unwrap();
        work::reset();
        index.add_edges(&caller, vec![]);
        let observed = work::snapshot();
        assert_eq!(
            dispatches(&index),
            expected,
            "reexport edit retires; byte repair restores"
        );
        assert_eq!(observed.ngrx_reexport_reads, 1);
        assert_eq!(observed.ngrx_reexport_parses, 1);
        let edges = index.reverse_edges_by_identity("ngrx", "Action", "Act");
        for edge in edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::Dispatches)
        {
            assert_eq!(edge.layer, "ngrx");
            assert_eq!(edge.subject.name, "Caller");
            assert_eq!(edge.subject.file.as_deref(), Some(caller.as_str()));
            assert_eq!(edge.object.file.as_deref(), Some(action.as_str()));
        }
        eprintln!("AF002_NGRX_REEXPORT expected={expected} observations={observed:?}");
    }
}
