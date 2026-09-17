// src/tests/ir/semantic_projection.rs
//
// The language-agnostic IR fact → `SemanticEdge` projection at the compile
// boundary.
//
// Covers RED-CALL19's identity premise (arity never enters an entity name or
// entity identity), the method-registration shape, unresolved callee
// truthfulness, and the edge evidence that later becomes WorkspaceIndex
// occurrence identity.

use crate::ir::opcodes::CoreOp;
use crate::ir::semantic_projection::{
    BUILTIN_DOMAIN, BUILTIN_LAYER, METHOD_ENTITY_TYPE, project_calls, project_generic_facts,
    project_method_declarations,
};
use crate::layers::meta::semantic::SemanticRelation;

fn stream() -> Vec<CoreOp> {
    vec![
        CoreOp::DefClass("C1".into(), "Example".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "Process".into()),
        CoreOp::Call("M1".into(), "Save".into(), 1, false),
        CoreOp::Call("M1".into(), "Save".into(), 2, false),
        CoreOp::Call("M1".into(), "OrderBy".into(), 2, false),
    ]
}

#[test]
fn method_declarations_register_one_occurrence_per_callable() {
    let edges = project_method_declarations(&stream(), "C:/repo/Example.cs");
    assert_eq!(edges.len(), 1);

    let edge = &edges[0];
    assert_eq!(edge.relation, SemanticRelation::Defines);
    assert_eq!(edge.layer, BUILTIN_LAYER);
    assert_eq!(edge.subject.domain, BUILTIN_DOMAIN);
    assert_eq!(edge.subject.entity_type, METHOD_ENTITY_TYPE);
    assert_eq!(edge.subject.name, "Process");
    assert_eq!(edge.subject.file.as_deref(), Some("C:/repo/Example.cs"));
    assert_eq!(
        edge.subject, edge.object,
        "a self-referential Defines edge is the registration carrier"
    );
    assert!(
        edge.call_evidence.is_none(),
        "a declaration carries no call evidence"
    );
}

#[test]
fn calls_project_method_level_subject_and_object_with_evidence() {
    let edges = project_calls(&stream(), "C:/repo/Example.cs");
    assert_eq!(edges.len(), 3);

    let save_one = &edges[0];
    assert_eq!(save_one.relation, SemanticRelation::Calls);
    assert_eq!(save_one.subject.domain, BUILTIN_DOMAIN);
    assert_eq!(save_one.subject.entity_type, METHOD_ENTITY_TYPE);
    assert_eq!(save_one.subject.name, "Process");
    assert_eq!(save_one.object.domain, BUILTIN_DOMAIN);
    assert_eq!(save_one.object.entity_type, METHOD_ENTITY_TYPE);
    assert_eq!(save_one.object.name, "Save");
    let save_one_evidence = save_one.call_evidence.expect("call evidence");
    assert_eq!(save_one_evidence.explicit_arg_count, 1);
    assert!(
        !save_one_evidence.has_spread,
        "a call with no expanding argument is exact written-arity evidence"
    );
    let save_two_evidence = edges[1].call_evidence.expect("call evidence");
    assert_eq!(
        save_two_evidence.explicit_arg_count, 2,
        "same caller/callee with a different arity is a different fact"
    );
    assert!(!save_two_evidence.has_spread);
}

#[test]
fn spread_call_projects_the_qualifier_into_evidence() {
    let instructions = vec![
        CoreOp::DefMethod("C1".into(), "M1".into(), "Process".into()),
        CoreOp::Call("M1".into(), "Save".into(), 1, false),
        CoreOp::Call("M1".into(), "Save".into(), 1, true),
    ];
    let edges = project_calls(&instructions, "C:/repo/Example.ts");
    assert_eq!(edges.len(), 2, "both facts must project");
    let exact = edges[0].call_evidence.expect("exact evidence");
    let spread = edges[1].call_evidence.expect("spread evidence");
    assert_eq!(
        exact.explicit_arg_count, spread.explicit_arg_count,
        "both calls write one argument node"
    );
    assert!(!exact.has_spread);
    assert!(spread.has_spread, "the expanding fact must be qualified");
    assert_ne!(
        exact, spread,
        "one written argument is not the same evidence in both cases"
    );
}

#[test]
fn arity_never_enters_an_entity_name_or_identity() {
    let edges = project_calls(&stream(), "C:/repo/Example.cs");
    for edge in &edges {
        for name in [edge.subject.name.as_str(), edge.object.name.as_str()] {
            assert!(
                !name.contains('/') && !name.contains('$') && !name.contains(":2"),
                "arity must never be encoded into a semantic name: {name}"
            );
        }
    }
    // Both arities describe the same caller and the same callee entity.
    assert_eq!(edges[0].subject.name, edges[1].subject.name);
    assert_eq!(edges[0].object.name, edges[1].object.name);
}

#[test]
fn unresolved_callee_still_projects_as_the_edge_object() {
    let edges = project_calls(&stream(), "C:/repo/Example.cs");
    assert!(
        edges.iter().any(|edge| edge.object.name == "OrderBy"),
        "a callee with no declaration in the file must still be recorded"
    );
    assert!(
        !edges.iter().any(|edge| edge.object.name.is_empty()),
        "an unnamed callee must never be projected"
    );
}

#[test]
fn unknown_caller_projects_no_edge() {
    let instructions = vec![CoreOp::Call("M404".into(), "Save".into(), 1, false)];
    assert!(
        project_calls(&instructions, "C:/repo/Example.cs").is_empty(),
        "a caller that does not exist in the compiled IR projects nothing \
         (validator rule E011 owns the diagnostic)"
    );
}

#[test]
fn duplicate_identical_facts_project_equal_edges() {
    let instructions = vec![
        CoreOp::DefMethod("C1".into(), "M1".into(), "Process".into()),
        CoreOp::Call("M1".into(), "Save".into(), 1, false),
        CoreOp::Call("M1".into(), "Save".into(), 1, false),
    ];
    let edges = project_calls(&instructions, "C:/repo/Example.cs");
    assert_eq!(edges.len(), 2);
    // Per-file extraction keeps both; the WorkspaceIndex collapses identical
    // occurrences of one asserting file (RED-CALL6).
    assert_eq!(edges[0].object.name, edges[1].object.name);
    assert_eq!(edges[0].call_evidence, edges[1].call_evidence);
}

#[test]
fn generic_projection_orders_declarations_before_calls() {
    let edges = project_generic_facts(&stream(), "C:/repo/Example.cs");
    assert_eq!(edges.len(), 4);
    assert_eq!(edges[0].relation, SemanticRelation::Defines);
    assert!(
        edges[1..]
            .iter()
            .all(|edge| edge.relation == SemanticRelation::Calls),
        "declaration occurrences must precede call facts"
    );
}
