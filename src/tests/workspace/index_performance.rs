// Deterministic work-scope regressions for WorkspaceIndex file removal.

use super::WorkspaceIndex;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

fn registration(entity_type: &'static str, name: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: EntityRef::new("test", entity_type, name),
        object: EntityRef::new("test", entity_type, name),
        layer: "test",
    }
}

fn dependency(subject: &str, object: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("test", "Node", subject),
        object: EntityRef::new("test", "Node", object),
        layer: "test",
    }
}

fn populate_unrelated(index: &mut WorkspaceIndex, count: usize) {
    for i in 0..count {
        let name = format!("Unrelated{i}");
        let file = format!("unrelated-{i}.rs");
        index.add_edges(&file, vec![registration("Unrelated", &name)]);
    }
}

fn sorted_names(entities: Vec<&EntityRef>) -> Vec<String> {
    let mut names: Vec<String> = entities
        .into_iter()
        .map(|entity| entity.name.clone())
        .collect();
    names.sort();
    names
}

#[test]
fn red_p1_cleanup_work_is_file_local() {
    let mut index = WorkspaceIndex::new();
    populate_unrelated(&mut index, 256);
    index.add_edges(
        "target.rs",
        vec![
            registration("Target", "TargetA"),
            registration("Target", "TargetB"),
        ],
    );

    index.remove_file("target.rs");

    assert_eq!(index.name_cleanup_buckets_examined().len(), 2);
}

#[test]
fn red_p2_unrelated_growth_does_not_increase_removal_work() {
    fn removal_work(unrelated_count: usize) -> usize {
        let mut index = WorkspaceIndex::new();
        populate_unrelated(&mut index, unrelated_count);
        index.add_edges(
            "target.rs",
            vec![
                registration("Target", "TargetA"),
                registration("Target", "TargetB"),
            ],
        );
        index.remove_file("target.rs");
        index.name_cleanup_buckets_examined().len()
    }

    assert_eq!(removal_work(4), removal_work(2_000));
}

#[test]
fn red_p3_last_occurrence_removes_name_bucket() {
    let mut index = WorkspaceIndex::new();
    index.add_edges("target.rs", vec![registration("Target", "OnlyHere")]);

    index.remove_file("target.rs");

    assert!(
        index
            .entities_by_identity("test", "Target", "OnlyHere")
            .is_empty()
    );
    assert!(!index.name_index.contains_key("OnlyHere"));
    assert!(index.find_entities_by_name("OnlyHere").is_empty());
}

#[test]
fn red_p4_shared_name_preserves_surviving_entity() {
    let mut index = WorkspaceIndex::new();
    index.add_edges("removed.rs", vec![registration("RemovedType", "Shared")]);
    index.add_edges(
        "surviving.rs",
        vec![registration("SurvivingType", "Shared")],
    );

    index.remove_file("removed.rs");

    assert!(
        index
            .entities_by_identity("test", "RemovedType", "Shared")
            .is_empty()
    );
    assert_eq!(
        index
            .entities_by_identity("test", "SurvivingType", "Shared")
            .len(),
        1
    );
    assert_eq!(index.name_index["Shared"].len(), 1);
    assert_eq!(index.find_entities_by_name("Shared").len(), 1);
}

#[test]
fn red_p5_unrelated_names_are_not_examined() {
    let mut index = WorkspaceIndex::new();
    populate_unrelated(&mut index, 64);
    index.add_edges("target.rs", vec![registration("Target", "TargetOnly")]);

    index.remove_file("target.rs");

    assert_eq!(
        index.name_cleanup_buckets_examined(),
        &["TargetOnly".to_string()]
    );
}

#[test]
fn red_p6_remove_readd_preserves_queries_and_edges() {
    let mut index = WorkspaceIndex::new();
    index.add_edges("target.rs", vec![dependency("Before", "OldDependency")]);
    index.remove_file("target.rs");
    index.add_edges("target.rs", vec![dependency("After", "NewDependency")]);

    assert!(index.find_entities_by_name("Before").is_empty());
    assert!(index.find_entities_by_name("OldDependency").is_empty());
    assert_eq!(
        sorted_names(index.entities_in_file("target.rs")),
        ["After", "NewDependency"]
    );
    assert_eq!(index.find_entities_by_name("After").len(), 1);
    assert_eq!(index.find_entities_by_name("NewDependency").len(), 1);
    assert_eq!(
        index
            .forward_edges_by_identity("test", "Node", "After")
            .len(),
        1
    );
    assert_eq!(
        index
            .reverse_edges_by_identity("test", "Node", "NewDependency")
            .len(),
        1
    );
    assert!(
        index
            .forward_edges_by_identity("test", "Node", "Before")
            .is_empty()
    );
    assert!(
        index
            .reverse_edges_by_identity("test", "Node", "OldDependency")
            .is_empty()
    );
}

#[test]
fn red_p7_repeated_replacement_does_not_accumulate_stale_keys() {
    let mut index = WorkspaceIndex::new();

    for generation in 0..5 {
        if generation > 0 {
            index.remove_file("target.rs");
        }
        index.add_edges(
            "target.rs",
            vec![
                registration("Target", "Stable"),
                registration("Target", &format!("Generation{generation}")),
            ],
        );

        assert_eq!(index.find_entities_by_name("Stable").len(), 1);
        assert_eq!(
            index
                .find_entities_by_name(&format!("Generation{generation}"))
                .len(),
            1
        );
        assert_eq!(index.name_index["Stable"].len(), 1);
        for stale_generation in 0..generation {
            let stale_name = format!("Generation{stale_generation}");
            assert!(index.find_entities_by_name(&stale_name).is_empty());
            assert!(!index.name_index.contains_key(&stale_name));
        }
    }
}
