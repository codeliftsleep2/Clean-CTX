use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{hierarchical_to_ir, try_ir_to_hierarchical};
use crate::ir::opcodes::CoreOp;

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "patterns.ts".into(),
        version: 1,
        instructions,
    }
}

#[test]
fn pattern_schema_targets_stable_ids_independent_of_shape_and_order() {
    let method_pattern = CoreOp::Pattern(
        "PROMISE".into(),
        vec!["class-two".into(), "worker-id".into(), "$Promise".into()],
    );
    let class_pattern = CoreOp::Pattern(
        "CUSTOM".into(),
        vec!["class-one".into(), "M-looking-metadata".into()],
    );
    let hierarchy = try_ir_to_hierarchical(&compiled(vec![
        method_pattern.clone(),
        class_pattern.clone(),
        CoreOp::DefMethod("class-two".into(), "worker-id".into(), "work".into()),
        CoreOp::DefClass("class-one".into(), "First".into()),
        CoreOp::DefClass("class-two".into(), "Second".into()),
    ]))
    .expect("schema-targeted patterns are valid");

    let first = hierarchy
        .classes
        .iter()
        .find(|class| class.id == "class-one")
        .expect("class-one must be projected");
    let second = hierarchy
        .classes
        .iter()
        .find(|class| class.id == "class-two")
        .expect("class-two must be projected");

    assert_eq!(
        first.patterns,
        vec![crate::ir::hierarchical::PatternEntry {
            name: "CUSTOM".into(),
            args: vec!["class-one".into(), "M-looking-metadata".into()],
        }]
    );
    assert!(second.patterns.is_empty());
    assert_eq!(second.methods[0].id, "worker-id");
    assert_eq!(
        second.methods[0].patterns,
        vec![crate::ir::hierarchical::PatternEntry {
            name: "PROMISE".into(),
            args: vec!["class-two".into(), "worker-id".into(), "$Promise".into()],
        }]
    );
}

#[test]
fn repeated_pattern_occurrences_and_operands_survive_round_trip() {
    let pattern = CoreOp::Pattern(
        "CTOR".into(),
        vec![
            "C1".into(),
            "constructor-id".into(),
            "Logger".into(),
            "Logger".into(),
        ],
    );
    let hierarchy = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Sample".into()),
        CoreOp::DefMethod("C1".into(), "constructor-id".into(), "new".into()),
        pattern.clone(),
        pattern.clone(),
    ]))
    .expect("repeated patterns are valid");

    assert_eq!(
        hierarchy.classes[0].methods[0].patterns.len(),
        2,
        "each pattern occurrence must remain distinct"
    );
    let decoded_patterns: Vec<CoreOp> = hierarchical_to_ir(&hierarchy)
        .into_iter()
        .filter(|op| matches!(op, CoreOp::Pattern(..)))
        .collect();
    assert_eq!(decoded_patterns, vec![pattern.clone(), pattern]);
}
