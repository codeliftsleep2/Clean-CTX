//! Companion consistency and retained ownership/order, not new retirement policy.
use super::*;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use std::collections::{HashMap, HashSet};
fn check(index: &WorkspaceIndex) {
    let names: HashMap<_, _> = index
        .name_index
        .iter()
        .map(|(name, keys)| {
            let set: HashSet<_> = keys.iter().cloned().collect();
            assert_eq!(set.len(), keys.len());
            (name.clone(), set)
        })
        .collect();
    let occurrences: HashMap<_, _> = index
        .entities
        .iter()
        .map(|(key, items)| {
            let set: HashSet<_> = items.iter().map(|e| e.file.clone()).collect();
            assert_eq!(set.len(), items.len());
            (key.clone(), set)
        })
        .collect();
    let publishers: HashMap<_, _> = index
        .file_map
        .iter()
        .map(|(file, keys)| {
            let set: HashSet<_> = keys.iter().cloned().collect();
            assert_eq!(set.len(), keys.len());
            (file.clone(), set)
        })
        .collect();
    assert_eq!(index.name_members, names);
    assert_eq!(index.occurrence_members, occurrences);
    assert_eq!(index.publisher_members, publishers);
}
fn registration(name: &str, physical: &str) -> SemanticEdge {
    let entity = EntityRef::new("af174", "Node", name).with_file(physical.to_owned());
    SemanticEdge {
        subject: entity.clone(),
        object: entity,
        relation: SemanticRelation::Defines,
        layer: "af174",
        call_evidence: None,
    }
}
fn keys(index: &WorkspaceIndex, file: &str) -> Vec<String> {
    index
        .file_map
        .get(file)
        .unwrap()
        .iter()
        .map(|k| k.2.clone())
        .collect()
}
#[test]
fn af174_invariant_publishers_order_clone_and_retirement() {
    let mut index = WorkspaceIndex::new();
    check(&index);
    for publisher in ["/af174/publisher-a.ts", "/af174/publisher-b.ts"] {
        index.add_edges(
            publisher,
            vec![
                registration("B", "/af174/physical.ts"),
                registration("A", "/af174/physical.ts"),
                registration("B", "/af174/physical.ts"),
            ],
        );
        check(&index);
        assert_eq!(keys(&index, publisher), vec!["B", "A"]);
    }
    assert_eq!(index.entities_by_identity("af174", "Node", "B").len(), 1);
    index.remove_file("/af174/publisher-a.ts");
    check(&index);
    assert_eq!(keys(&index, "/af174/publisher-b.ts"), vec!["B", "A"]);
    assert_eq!(
        index.entities_by_identity("af174", "Node", "B").len(),
        1,
        "removing an asserting publisher must preserve the independently located physical occurrence under existing semantics"
    );
    let mut clone = index.clone();
    check(&clone);
    clone.add_edges(
        "/af174/publisher-b.ts",
        vec![registration("C", "/af174/physical.ts")],
    );
    check(&clone);
    check(&index);
    assert_eq!(keys(&clone, "/af174/publisher-b.ts"), vec!["B", "A", "C"]);
    assert_eq!(keys(&index, "/af174/publisher-b.ts"), vec!["B", "A"]);
    clone.remove_file("/af174/publisher-b.ts");
    check(&clone);
    clone.add_edges(
        "/af174/publisher-b.ts",
        vec![
            registration("A", "/af174/physical.ts"),
            registration("B", "/af174/physical.ts"),
        ],
    );
    check(&clone);
    assert_eq!(keys(&clone, "/af174/publisher-b.ts"), vec!["A", "B"]);
    clone.remove_files_in_root("/af174");
    check(&clone);
    // Existing removal retires physical occurrences only when their file matches
    // the removed path; this optimization does not invent last-publisher policy.
    assert_eq!(index.entities_by_identity("af174", "Node", "B").len(), 1);
}
#[test]
fn af174_invariant_physical_owner_and_none_identity() {
    let mut index = WorkspaceIndex::new();
    index.add_edges("/af174/a.ts", vec![registration("Shared", "/af174/a.ts")]);
    index.add_edges("/af174/b.ts", vec![registration("Shared", "/af174/b.ts")]);
    check(&index);
    let files = || {
        vec![
            Some("/af174/a.ts".to_owned()),
            Some("/af174/b.ts".to_owned()),
        ]
    };
    assert_eq!(
        index
            .entities_by_identity("af174", "Node", "Shared")
            .iter()
            .map(|e| e.file.clone())
            .collect::<Vec<_>>(),
        files()
    );
    index.remove_file("/af174/a.ts");
    check(&index);
    assert_eq!(
        index.entities_by_identity("af174", "Node", "Shared")[0]
            .file
            .as_deref(),
        Some("/af174/b.ts")
    );
    let key = ("af174".into(), "Node".into(), "NoFile".into());
    let mut participation = Vec::new();
    index.register_entity(
        &key,
        EntityRef::new("af174", "Node", "NoFile"),
        &mut participation,
    );
    index.register_entity(
        &key,
        EntityRef::new("af174", "Node", "NoFile"),
        &mut participation,
    );
    check(&index);
    assert_eq!(
        index.entities_by_identity("af174", "Node", "NoFile").len(),
        1
    );
}
#[test]
fn af174_memory_companion_capacity_payload_and_clone() {
    for n in [1, 2, 4, 8, 16, 64, 256] {
        for duplicates in [1, 4] {
            let mut index = WorkspaceIndex::new();
            for publisher in ["/af174/publisher-a.ts", "/af174/publisher-b.ts"] {
                index.add_edges(
                    publisher,
                    (0..n)
                        .flat_map(|i| {
                            let name = format!("LongEntity_{i}_{}", "x".repeat(80));
                            let physical = format!("/af174/{}/physical-{i}.ts", "deep/".repeat(16));
                            (0..duplicates).map(move |_| registration(&name, &physical))
                        })
                        .collect(),
                );
            }
            check(&index);
            let mut string_capacity = 0;
            let mut key_slots = 0;
            let mut set_capacity = 0;
            for (name, set) in &index.name_members {
                string_capacity += name.capacity();
                set_capacity += set.capacity();
                for key in set {
                    key_slots += 1;
                    string_capacity += key.0.capacity() + key.1.capacity() + key.2.capacity();
                }
            }
            for (key, set) in &index.occurrence_members {
                key_slots += 1;
                string_capacity += key.0.capacity() + key.1.capacity() + key.2.capacity();
                set_capacity += set.capacity();
                for file in set {
                    string_capacity += file.as_ref().map_or(0, String::capacity);
                }
            }
            for (file, set) in &index.publisher_members {
                string_capacity += file.capacity();
                set_capacity += set.capacity();
                for key in set {
                    key_slots += 1;
                    string_capacity += key.0.capacity() + key.1.capacity() + key.2.capacity();
                }
            }
            let clone = index.clone();
            check(&clone);
            eprintln!(
                "AF174_MEMORY unique={n} duplicate_density={duplicates} companion_string_capacity={string_capacity} key_values={key_slots} summed_set_capacity={set_capacity} map_capacities={:?} EntityKey_header={} OptionString_header={} clone_equal=true accounting=payload-and-capacity-not-exact-allocator-bytes",
                [
                    index.name_members.capacity(),
                    index.occurrence_members.capacity(),
                    index.publisher_members.capacity()
                ],
                std::mem::size_of::<EntityKey>(),
                std::mem::size_of::<Option<String>>()
            );
            index.remove_files_in_root("/af174");
            check(&index);
            eprintln!(
                "AF174_MEMORY_AFTER_RETIRE unique={n} name_buckets={} occurrence_buckets={} publisher_buckets={} map_capacities={:?}",
                index.name_members.len(),
                index.occurrence_members.len(),
                index.publisher_members.len(),
                [
                    index.name_members.capacity(),
                    index.occurrence_members.capacity(),
                    index.publisher_members.capacity()
                ]
            );
        }
    }
}

#[test]
fn af174_invariant_ngrx_synthetic_add_remove() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("actions.ts");
    std::fs::write(&target, "export const Act=1;").unwrap();
    let target = crate::dictionary::path::canonical_identity_key(
        &target.canonicalize().unwrap().to_string_lossy(),
    );
    let mut index = WorkspaceIndex::new();
    let entity = EntityRef::new("ngrx", "Action", "Act").with_file(target.clone());
    index.add_edges(
        &target,
        vec![SemanticEdge {
            subject: entity.clone(),
            object: entity,
            relation: SemanticRelation::Defines,
            layer: "ngrx",
            call_evidence: None,
        }],
    );
    check(&index);
    let caller = root.path().join("caller.ts").to_string_lossy().into_owned();
    let candidate = SemanticEdge {
        subject: EntityRef::new("angular", "Component", "SyntheticCaller"),
        object: EntityRef::new("ngrx", "Action", "Act").with_file("./actions".to_owned()),
        relation: SemanticRelation::Dispatches,
        layer: crate::angular_meta::ngrx::direct_imports::REFINEMENT_LAYER,
        call_evidence: None,
    };
    index.add_edges(&caller, vec![candidate.clone()]);
    check(&index);
    assert_eq!(
        index
            .forward_edges_by_identity("angular", "Component", "SyntheticCaller")
            .len(),
        1
    );
    index.remove_file(&target);
    check(&index);
    assert!(
        index
            .forward_edges_by_identity("angular", "Component", "SyntheticCaller")
            .is_empty()
    );
    let entity = EntityRef::new("ngrx", "Action", "Act").with_file(target.clone());
    index.add_edges(
        &target,
        vec![SemanticEdge {
            subject: entity.clone(),
            object: entity,
            relation: SemanticRelation::Defines,
            layer: "ngrx",
            call_evidence: None,
        }],
    );
    check(&index);
    assert_eq!(
        index
            .forward_edges_by_identity("angular", "Component", "SyntheticCaller")
            .len(),
        1
    );
    index.remove_file(&caller);
    check(&index);
    assert!(
        index
            .forward_edges_by_identity("angular", "Component", "SyntheticCaller")
            .is_empty()
    );
}
