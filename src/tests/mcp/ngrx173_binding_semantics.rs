//! Public producer fixture, then actual index reconstruction with duplicate occurrence control.
use super::{fs, new_state, publish_all, write_bare_reducer, write_dispatch};
use crate::layers::meta::semantic::SemanticRelation;
use crate::work_observations_af_perf_002 as work;

#[test]
fn af173_binding_semantics_positive_mixed_duplicates_and_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("actions.ts"),
        "export const ACTION_1='v1'; export const ActionObjectA={ ACTION_1 }; ",
    )
    .unwrap();
    write_bare_reducer(root.path());
    write_dispatch(root.path());
    fs::write(
        root.path().join("reducers.ts"),
        "import { ReducerA } from './reducer'; export const reducers={feature: ReducerA};",
    )
    .unwrap();
    fs::write(root.path().join("module.ts"), "import { StoreModule } from '@ngrx/store'; import { reducers } from './reducers'; StoreModule.forRoot(reducers);").unwrap();
    let state = new_state(root.path());
    publish_all(
        &state,
        root.path(),
        &[
            "module.ts",
            "reducers.ts",
            "reducer.ts",
            "dispatch.ts",
            "actions.ts",
        ],
    );
    let owner = |name: &str| state.semantic_owner_path(&root.path().join(name).to_string_lossy());
    let action_file = owner("actions.ts");
    let reducer_file = owner("reducer.ts");
    let module_file = owner("module.ts");
    let mut index = state.workspace_index_lock();
    work::reset();
    index.add_edges(&module_file, vec![]);
    let counts = work::snapshot();
    assert_eq!(
        counts.ngrx_binding_collections, 1,
        "map and switch share one original-position collection"
    );
    assert!(counts.ngrx_map_binding_visits > 0 && counts.ngrx_switch_binding_visits > 0);
    let handlers: Vec<_> = index
        .forward_edges_by_identity("ngrx", "Action", "ActionObjectA.ACTION_1")
        .into_iter()
        .filter(|e| e.relation == SemanticRelation::TriggersReducer)
        .cloned()
        .collect();
    assert_eq!(handlers.len(), 1);
    let handler = &handlers[0];
    assert_eq!(handler.layer, "ngrx");
    assert_eq!(handler.subject.file.as_deref(), Some(action_file.as_str()));
    assert_eq!(handler.object.domain, "ngrx");
    assert_eq!(handler.object.entity_type, "Reducer");
    assert_eq!(handler.object.name, "ReducerA");
    assert_eq!(handler.object.file.as_deref(), Some(reducer_file.as_str()));
    let registrations: Vec<_> = index
        .forward_edges_by_identity("ngrx", "StoreRegistration", "StoreModule.forRoot")
        .into_iter()
        .filter(|e| e.relation == SemanticRelation::RegistersReducer)
        .collect();
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].layer, "ngrx");
    assert_eq!(
        registrations[0].subject.file.as_deref(),
        Some(module_file.as_str())
    );
    assert_eq!(registrations[0].object.name, "ReducerA");
    assert_eq!(
        registrations[0].object.file.as_deref(),
        Some(reducer_file.as_str())
    );
    let binding = index
        .forward_edges_by_identity("typescript", "ObjectMember", "ActionObjectA.ACTION_1")
        .into_iter()
        .find(|e| e.relation == SemanticRelation::ReferencesValue)
        .expect("captured value binding")
        .clone();
    let duplicate_publisher = root
        .path()
        .join("duplicate.ts")
        .to_string_lossy()
        .into_owned();
    index.add_edges(&duplicate_publisher, vec![binding]);
    assert!(
        index
            .forward_edges_by_identity("ngrx", "Action", "ActionObjectA.ACTION_1")
            .iter()
            .all(|e| e.relation != SemanticRelation::TriggersReducer),
        "competing binding occurrences refuse, never deduplicate proof"
    );
    index.remove_file(&duplicate_publisher);
    let repaired: Vec<_> = index
        .forward_edges_by_identity("ngrx", "Action", "ActionObjectA.ACTION_1")
        .into_iter()
        .filter(|e| e.relation == SemanticRelation::TriggersReducer)
        .cloned()
        .collect();
    assert_eq!(
        serde_json::to_value(repaired).unwrap(),
        serde_json::to_value(handlers).unwrap(),
        "exact singleton identity/order restored"
    );
    eprintln!(
        "AF173_POSITIVE_MIXED collections={} prepass_checks={} visits={} clone_bytes={} map_joins={} switch_joins={} exports={}/{}",
        counts.ngrx_binding_collections,
        counts.ngrx_binding_demand_checks,
        counts.ngrx_binding_edge_visits,
        counts.ngrx_binding_string_bytes,
        counts.ngrx_map_binding_visits,
        counts.ngrx_switch_binding_visits,
        counts.export_reads,
        counts.export_parses
    );
}
