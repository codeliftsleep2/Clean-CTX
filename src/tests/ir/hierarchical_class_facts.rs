use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{
    HierarchicalProjectionError, ProjectionIdentityKind, hierarchical_to_ir,
    ir_to_hierarchical_wire, try_ir_to_hierarchical, wire_to_ir,
};
use crate::ir::opcodes::{CoreOp, DeclarationModifier};

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "alpha-1".to_string(),
        version: 7,
        instructions,
    }
}

#[test]
fn class_facts_resolve_by_identity_before_and_across_definitions() {
    let ir = compiled(vec![
        CoreOp::ClassModifiers(
            "C2".into(),
            vec![DeclarationModifier::Export, DeclarationModifier::Export],
        ),
        CoreOp::Injects("C1".into(), vec!["Logger".into(), "Logger".into()]),
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::Implements("C2".into(), "Runnable".into()),
        CoreOp::Extends("C1".into(), "Base".into()),
        CoreOp::DefClass("C2".into(), "Second".into()),
    ]);

    let hierarchy = try_ir_to_hierarchical(&ir).expect("identity graph is valid");
    let first = hierarchy
        .classes
        .iter()
        .find(|class| class.id == "C1")
        .expect("C1 must be projected");
    let second = hierarchy
        .classes
        .iter()
        .find(|class| class.id == "C2")
        .expect("C2 must be projected");

    assert_eq!(first.extends.as_deref(), Some("Base"));
    assert_eq!(first.injects, vec![vec!["Logger", "Logger"]]);
    assert!(first.modifiers.is_empty());
    assert!(first.implements.is_empty());

    assert_eq!(
        second.modifiers,
        vec![vec![
            DeclarationModifier::Export,
            DeclarationModifier::Export
        ]]
    );
    assert_eq!(second.implements, vec!["Runnable"]);
    assert!(second.extends.is_none());
    assert!(second.injects.is_empty());
}

#[test]
fn repeated_class_facts_round_trip_without_union_or_replacement() {
    let ir = compiled(vec![
        CoreOp::DefClass("C1".into(), "Sample".into()),
        CoreOp::ClassModifiers(
            "C1".into(),
            vec![DeclarationModifier::Export, DeclarationModifier::Export],
        ),
        CoreOp::ClassModifiers("C1".into(), vec![DeclarationModifier::Abstract]),
        CoreOp::Extends("C1".into(), "Base".into()),
        CoreOp::Implements("C1".into(), "Readable".into()),
        CoreOp::Implements("C1".into(), "Readable".into()),
        CoreOp::Injects("C1".into(), vec!["Logger".into(), "Clock".into()]),
        CoreOp::Injects("C1".into(), vec!["Logger".into()]),
        CoreOp::Injects("C1".into(), vec![]),
    ]);

    let hierarchy = try_ir_to_hierarchical(&ir).expect("identity graph is valid");
    let class = &hierarchy.classes[0];
    assert_eq!(
        class.modifiers,
        vec![
            vec![DeclarationModifier::Export, DeclarationModifier::Export],
            vec![DeclarationModifier::Abstract]
        ]
    );
    assert_eq!(class.extends.as_deref(), Some("Base"));
    assert_eq!(class.implements, vec!["Readable", "Readable"]);
    assert_eq!(
        class.injects,
        vec![vec!["Logger", "Clock"], vec!["Logger"], vec![]]
    );

    assert_eq!(hierarchical_to_ir(&hierarchy), ir.instructions);
}

#[test]
fn revision_four_wire_emits_occurrence_preserving_class_shapes() {
    let ir = compiled(vec![
        CoreOp::DefClass("C1".into(), "Sample".into()),
        CoreOp::ClassModifiers("C1".into(), vec![DeclarationModifier::Export]),
        CoreOp::ClassModifiers("C1".into(), vec![DeclarationModifier::Abstract]),
        CoreOp::Injects("C1".into(), vec!["Logger".into(), "Clock".into()]),
        CoreOp::Injects("C1".into(), vec!["Logger".into()]),
    ]);

    let wire = ir_to_hierarchical_wire(&ir);
    assert_eq!(wire["hs"], 7);
    assert_eq!(
        wire["ir"]["c"][0]["mo"],
        serde_json::json!([["EXPORT"], ["ABSTRACT"]])
    );
    assert_eq!(
        wire["ir"]["c"][0]["ij"],
        serde_json::json!([["Logger", "Clock"], ["Logger"]])
    );
}

#[test]
fn revision_two_class_shapes_remain_readable() {
    let wire = serde_json::json!({
        "file": "alpha-1",
        "v": 7,
        "encoding": "hierarchical",
        "hs": 2,
        "ir": {
            "c": [{
                "n": "C1",
                "nm": "Sample",
                "fl": ["EXPORT", "ABSTRACT"],
                "x": "Base",
                "im": ["Readable", "Readable"],
                "ij": ["Logger", "Clock"]
            }]
        }
    });

    let decoded = wire_to_ir(&wire).expect("revision 2 remains readable");
    assert_eq!(
        decoded.instructions,
        vec![
            CoreOp::DefClass("C1".into(), "Sample".into()),
            CoreOp::ClassFlags("C1".into(), vec!["EXPORT".into(), "ABSTRACT".into()],),
            CoreOp::Extends("C1".into(), "Base".into()),
            CoreOp::Implements("C1".into(), "Readable".into()),
            CoreOp::Implements("C1".into(), "Readable".into()),
            CoreOp::Injects("C1".into(), vec!["Logger".into(), "Clock".into()]),
        ]
    );
}

#[test]
fn unversioned_class_shapes_remain_readable() {
    let wire = serde_json::json!({
        "file": "alpha-1",
        "v": 7,
        "encoding": "hierarchical",
        "ir": {
            "c": [{
                "n": "C1",
                "nm": "Sample",
                "fl": ["EXPORT"],
                "ij": ["Logger", "Clock"]
            }]
        }
    });

    let decoded = wire_to_ir(&wire).expect("unversioned hierarchy remains readable");
    assert_eq!(
        decoded.instructions,
        vec![
            CoreOp::DefClass("C1".into(), "Sample".into()),
            CoreOp::ClassFlags("C1".into(), vec!["EXPORT".into()]),
            CoreOp::Injects("C1".into(), vec!["Logger".into(), "Clock".into()]),
        ]
    );
}

#[test]
fn revision_three_rejects_revision_two_class_shapes() {
    let result = wire_to_ir(&serde_json::json!({
        "file": "alpha-1",
        "v": 7,
        "encoding": "hierarchical",
        "hs": 3,
        "ir": {
            "c": [{
                "n": "C1",
                "nm": "Sample",
                "fl": ["EXPORT"],
                "ij": ["Logger"]
            }]
        }
    }));

    assert!(result.is_err(), "revision 3 must enforce revision-3 shapes");
}

#[test]
fn class_fact_wrong_kind_target_is_a_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Sample".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
        CoreOp::ClassModifiers("M1".into(), vec![DeclarationModifier::Export]),
    ]))
    .expect_err("method identity must not satisfy a class fact");

    assert!(matches!(
        error,
        HierarchicalProjectionError::KindMismatch {
            operation: "MOD_C",
            ref id,
            expected: ProjectionIdentityKind::Class,
            actual: ProjectionIdentityKind::Method,
            instruction: 2,
        } if id == "M1"
    ));
}
