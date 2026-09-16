// src/tests/workspace/index_calls.rs
//
// WorkspaceIndex behaviour for native call facts (`SemanticRelation::Calls`).
//
// Covers RED-CALL6  (duplicate identical facts collapse to one occurrence),
// RED-CALL15 (compile-order independence),
// RED-CALL16 (removing the caller file removes only its evidence),
// RED-CALL17 (recompiling a caller file updates only that file),
// RED-CALL18 (arity participates in occurrence identity),
// RED-CALL19 (two files asserting the same call both survive),
// RED-CALL27 (three callers across three files yield three reverse occurrences),
// and the approved traversal policy (no Calls in transitive dependencies, no
// Calls in cycle detection).

use crate::layers::meta::semantic::{CallEvidence, EntityRef, SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;

/// One native call fact: `caller --Calls(argc)--> callee`.
fn call_edge(caller: &str, callee: &str, argc: usize, file: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Calls,
        subject: EntityRef::new("builtin", "Method", caller).with_file(file.to_string()),
        object: EntityRef::new("builtin", "Method", callee).with_file(file.to_string()),
        layer: "builtin",
        call_evidence: Some(CallEvidence::new(argc)),
    }
}

/// One callable declaration occurrence (registration carrier).
fn method_declaration(name: &str, file: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: EntityRef::new("builtin", "Method", name).with_file(file.to_string()),
        object: EntityRef::new("builtin", "Method", name).with_file(file.to_string()),
        layer: "builtin",
        call_evidence: None,
    }
}

fn callers_of(index: &WorkspaceIndex, callee: &str) -> Vec<String> {
    let mut callers: Vec<String> = index
        .reverse_edges_by_identity("builtin", "Method", callee)
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::Calls)
        .map(|edge| edge.subject.name.clone())
        .collect();
    callers.sort();
    callers
}

fn arities_of(index: &WorkspaceIndex, callee: &str) -> Vec<usize> {
    let mut arities: Vec<usize> = index
        .reverse_edges_by_identity("builtin", "Method", callee)
        .iter()
        .filter_map(|edge| {
            edge.call_evidence
                .map(|evidence| evidence.explicit_arg_count)
        })
        .collect();
    arities.sort_unstable();
    arities
}

// ── RED-CALL6 / RED-CALL18: occurrence identity ──────────────────────

#[test]
fn calls_identical_facts_collapse_to_one_occurrence() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "A.cs",
        vec![
            call_edge("A", "Foo", 1, "A.cs"),
            call_edge("A", "Foo", 1, "A.cs"),
        ],
    );

    assert_eq!(index.edge_count(), 1, "identical facts are one occurrence");
    assert_eq!(callers_of(&index, "Foo"), vec!["A".to_string()]);
    assert_eq!(arities_of(&index, "Foo"), vec![1]);
}

#[test]
fn calls_arity_participates_in_occurrence_identity() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "A.cs",
        vec![
            call_edge("A", "Foo", 1, "A.cs"),
            call_edge("A", "Foo", 2, "A.cs"),
        ],
    );

    assert_eq!(
        index.edge_count(),
        2,
        "same file, same caller, same callee, different arity = two facts"
    );
    assert_eq!(arities_of(&index, "Foo"), vec![1, 2]);
    assert_eq!(
        callers_of(&index, "Foo"),
        vec!["A".to_string(), "A".to_string()]
    );
}

#[test]
fn non_call_relations_keep_their_evidence_free_identity() {
    let mut index = WorkspaceIndex::new();
    let plain = |argc: Option<usize>| SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Component", "A").with_file("a.ts".to_string()),
        object: EntityRef::new("angular", "Service", "B").with_file("a.ts".to_string()),
        layer: "angular",
        call_evidence: argc.map(CallEvidence::new),
    };
    // Evidence on a non-call relation is not part of established identity, so
    // the second insert is a duplicate of the first.
    index.add_edges("a.ts", vec![plain(None), plain(None)]);
    assert_eq!(index.edge_count(), 1);
}

// ── RED-CALL19 / RED-CALL27: per-file occurrences ────────────────────

#[test]
fn calls_from_two_files_both_survive() {
    let mut index = WorkspaceIndex::new();
    index.add_edges("A.cs", vec![call_edge("A", "Foo", 1, "A.cs")]);
    index.add_edges("B.cs", vec![call_edge("B", "Foo", 1, "B.cs")]);

    assert_eq!(index.edge_count(), 2);
    assert_eq!(
        callers_of(&index, "Foo"),
        vec!["A".to_string(), "B".to_string()]
    );
}

#[test]
fn calls_same_name_caller_in_two_files_stay_distinct_occurrences() {
    // IDX-002 protection: two different files each containing
    // `Process --Calls(argc=1)--> Foo` must both remain present.
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "one.cs",
        vec![
            method_declaration("Process", "one.cs"),
            call_edge("Process", "Foo", 1, "one.cs"),
        ],
    );
    index.add_edges(
        "two.cs",
        vec![
            method_declaration("Process", "two.cs"),
            call_edge("Process", "Foo", 1, "two.cs"),
        ],
    );

    let reverse = index.reverse_edges_by_identity("builtin", "Method", "Foo");
    assert_eq!(
        reverse.len(),
        2,
        "one occurrence per asserting file must be retained"
    );
    assert!(
        reverse
            .iter()
            .any(|edge| edge.subject.file.as_deref() == Some("one.cs"))
            && reverse
                .iter()
                .any(|edge| edge.subject.file.as_deref() == Some("two.cs")),
        "both asserting files must be represented: {reverse:?}"
    );
}

#[test]
fn calls_three_callers_in_three_files_yield_three_reverse_occurrences() {
    let mut index = WorkspaceIndex::new();
    for file in ["a.cs", "b.cs", "c.cs"] {
        index.add_edges(
            file,
            vec![
                method_declaration("Process", file),
                call_edge("Process", "OrderBy", 1, file),
                call_edge("Project", "OrderBy", 2, file),
            ],
        );
    }

    let reverse = index.reverse_edges_by_identity("builtin", "Method", "OrderBy");
    assert_eq!(reverse.len(), 6, "two call facts per asserting file");
    assert_eq!(
        arities_of(&index, "OrderBy"),
        vec![1, 1, 1, 2, 2, 2],
        "each file contributes both arity facts"
    );
    assert_eq!(index.file_count(), 3, "three caller files are tracked");
}

// ── RED-CALL15/16/17: order independence and lifecycle ───────────────

#[test]
fn calls_compile_order_does_not_change_call_graph_state() {
    let files = [
        (
            "a.cs",
            vec![
                method_declaration("A", "a.cs"),
                call_edge("A", "Foo", 1, "a.cs"),
            ],
        ),
        (
            "b.cs",
            vec![
                method_declaration("B", "b.cs"),
                call_edge("B", "Foo", 1, "b.cs"),
            ],
        ),
        (
            "c.cs",
            vec![
                method_declaration("C", "c.cs"),
                call_edge("C", "Foo", 2, "c.cs"),
            ],
        ),
    ];

    let mut forward_order = WorkspaceIndex::new();
    for (file, edges) in files.iter() {
        forward_order.add_edges(file, edges.clone());
    }
    let mut reverse_order = WorkspaceIndex::new();
    for (file, edges) in files.iter().rev() {
        reverse_order.add_edges(file, edges.clone());
    }

    assert_eq!(forward_order.edge_count(), reverse_order.edge_count());
    assert_eq!(
        callers_of(&forward_order, "Foo"),
        callers_of(&reverse_order, "Foo")
    );
    assert_eq!(
        arities_of(&forward_order, "Foo"),
        arities_of(&reverse_order, "Foo")
    );
    assert_eq!(
        forward_order.entity_occurrence_count(),
        reverse_order.entity_occurrence_count()
    );
}

#[test]
fn removing_a_caller_file_removes_only_its_call_evidence() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "a.cs",
        vec![
            method_declaration("A", "a.cs"),
            call_edge("A", "Foo", 1, "a.cs"),
        ],
    );
    index.add_edges(
        "b.cs",
        vec![
            method_declaration("B", "b.cs"),
            call_edge("B", "Foo", 1, "b.cs"),
        ],
    );

    index.remove_file("a.cs");

    assert_eq!(
        callers_of(&index, "Foo"),
        vec!["B".to_string()],
        "only the removed file's evidence may disappear"
    );
    assert_eq!(index.edge_count(), 1);
    assert!(
        index.entities_in_file("a.cs").is_empty(),
        "the removed file contributes no entity occurrences"
    );
}

#[test]
fn recompiling_a_caller_file_updates_only_that_files_calls() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "a.cs",
        vec![
            method_declaration("A", "a.cs"),
            call_edge("A", "Foo", 1, "a.cs"),
        ],
    );
    index.add_edges(
        "b.cs",
        vec![
            method_declaration("B", "b.cs"),
            call_edge("B", "Bar", 1, "b.cs"),
        ],
    );

    // A.cs is recompiled and now calls Foo with a different arity.
    index.remove_file("a.cs");
    index.add_edges(
        "a.cs",
        vec![
            method_declaration("A", "a.cs"),
            call_edge("A", "Foo", 3, "a.cs"),
        ],
    );

    assert_eq!(arities_of(&index, "Foo"), vec![3]);
    assert_eq!(
        callers_of(&index, "Bar"),
        vec!["B".to_string()],
        "the untouched file's facts are unaffected"
    );
    assert_eq!(index.edge_count(), 2);
}

// ── Approved traversal policy ───────────────────────────────────────

#[test]
fn calls_are_not_traversed_by_transitive_dependencies() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "a.cs",
        vec![
            method_declaration("A", "a.cs"),
            call_edge("A", "Foo", 1, "a.cs"),
        ],
    );

    assert!(
        index
            .transitive_dependencies("builtin", "Method", "A", 0)
            .is_empty(),
        "Calls must stay out of transitive_dependencies"
    );
}

#[test]
fn calls_do_not_create_cycles() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "a.cs",
        vec![
            method_declaration("A", "a.cs"),
            method_declaration("B", "a.cs"),
            call_edge("A", "B", 0, "a.cs"),
            call_edge("B", "A", 0, "a.cs"),
        ],
    );

    assert!(
        !index.has_cycle(),
        "mutual call facts must not change generic has_cycle semantics"
    );
}

#[test]
fn declared_caller_is_registered_and_unresolved_callee_stays_reachable() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "caller.cs",
        vec![
            method_declaration("Process", "caller.cs"),
            call_edge("Process", "OrderBy", 2, "caller.cs"),
        ],
    );

    assert!(
        index
            .find_entities_by_name("Process")
            .iter()
            .any(|entity| entity.entity_type == "Method"),
        "the declared caller must be registered as a Method entity"
    );
    assert!(
        !index
            .reverse_edges_by_identity("builtin", "Method", "OrderBy")
            .is_empty(),
        "an unresolved callee stays reachable through the reverse edge"
    );
}
