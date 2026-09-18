use crate::compression::Fidelity;
use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{ir_to_hierarchical_wire, try_ir_to_hierarchical, wire_to_ir};
use crate::ir::opcodes::CoreOp;
use crate::ir::render_llm::render_hierarchical_for_llm;

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "alpha-1".to_string(),
        version: 7,
        instructions,
    }
}

#[test]
fn method_facts_resolve_by_identity_before_and_across_definitions() {
    let ir = compiled(vec![
        CoreOp::Flags("M2".into(), vec!["ASYNC".into(), "ASYNC".into()]),
        CoreOp::SideEffect("M1".into(), "io".into()),
        CoreOp::ExecutionContext("M2".into(), "async".into()),
        CoreOp::ControlFlow("M1".into(), "if".into(), "ready".into()),
        CoreOp::DataFlow("M2".into(), "reads".into(), "cache".into()),
        CoreOp::Body("M1".into(), "{ work(); }".into(), Some(10), Some(21)),
        CoreOp::DefMethod("C2".into(), "M2".into(), "second".into()),
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "first".into()),
        CoreOp::DefClass("C2".into(), "Second".into()),
    ]);

    let hierarchy = try_ir_to_hierarchical(&ir).expect("identity graph is valid");
    let first = hierarchy
        .classes
        .iter()
        .find(|class| class.id == "C1")
        .and_then(|class| class.methods.iter().find(|method| method.id == "M1"))
        .expect("M1 must be owned by C1");
    let second = hierarchy
        .classes
        .iter()
        .find(|class| class.id == "C2")
        .and_then(|class| class.methods.iter().find(|method| method.id == "M2"))
        .expect("M2 must be owned by C2");

    assert_eq!(first.side_effect, vec!["io"]);
    assert_eq!(first.control_flow, vec![vec!["if", "ready"]]);
    assert_eq!(first.body.as_deref(), Some("{ work(); }"));
    assert_eq!((first.body_start, first.body_end), (Some(10), Some(21)));
    assert!(first.flags.is_empty());
    assert!(first.data_flow.is_empty());
    assert!(first.execution_context.is_empty());

    assert_eq!(second.flags, vec![vec!["ASYNC", "ASYNC"]]);
    assert_eq!(second.data_flow, vec![vec!["reads", "cache"]]);
    assert_eq!(second.execution_context, vec!["async"]);
    assert!(second.side_effect.is_empty());
    assert!(second.control_flow.is_empty());
    assert!(second.body.is_none());
}

#[test]
fn repeated_method_facts_round_trip_without_union_or_replacement() {
    let ir = compiled(vec![
        CoreOp::DefClass("C1".into(), "Sample".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
        CoreOp::Flags("M1".into(), vec!["STATIC".into(), "STATIC".into()]),
        CoreOp::Flags("M1".into(), vec!["RET".into()]),
        CoreOp::ControlFlow("M1".into(), "if".into(), "first".into()),
        CoreOp::ControlFlow("M1".into(), "if".into(), "first".into()),
        CoreOp::DataFlow("M1".into(), "reads".into(), "state".into()),
        CoreOp::DataFlow("M1".into(), "reads".into(), "state".into()),
        CoreOp::SideEffect("M1".into(), "io".into()),
        CoreOp::SideEffect("M1".into(), "mutation".into()),
        CoreOp::ExecutionContext("M1".into(), "sync".into()),
        CoreOp::ExecutionContext("M1".into(), "async".into()),
    ]);

    let hierarchy = try_ir_to_hierarchical(&ir).expect("identity graph is valid");
    let method = &hierarchy.classes[0].methods[0];
    assert_eq!(method.flags, vec![vec!["STATIC", "STATIC"], vec!["RET"]]);
    assert_eq!(
        method.control_flow,
        vec![vec!["if", "first"], vec!["if", "first"]]
    );
    assert_eq!(
        method.data_flow,
        vec![vec!["reads", "state"], vec!["reads", "state"]]
    );
    assert_eq!(method.side_effect, vec!["io", "mutation"]);
    assert_eq!(method.execution_context, vec!["sync", "async"]);

    let restored = crate::ir::hierarchical::hierarchical_to_ir(&hierarchy);
    let facts = restored
        .into_iter()
        .filter(|op| {
            matches!(
                op,
                CoreOp::Flags(..)
                    | CoreOp::ControlFlow(..)
                    | CoreOp::DataFlow(..)
                    | CoreOp::SideEffect(..)
                    | CoreOp::ExecutionContext(..)
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(facts.as_slice(), &ir.instructions[2..]);
}

#[test]
fn versioned_wire_emits_occurrence_preserving_shapes() {
    let ir = compiled(vec![
        CoreOp::DefClass("C1".into(), "Sample".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
        CoreOp::Flags("M1".into(), vec!["STATIC".into()]),
        CoreOp::Flags("M1".into(), vec!["RET".into()]),
        CoreOp::SideEffect("M1".into(), "io".into()),
        CoreOp::SideEffect("M1".into(), "mutation".into()),
        CoreOp::ExecutionContext("M1".into(), "sync".into()),
        CoreOp::ExecutionContext("M1".into(), "async".into()),
    ]);

    let wire = ir_to_hierarchical_wire(&ir);
    assert_eq!(wire["hs"], 2);
    assert_eq!(
        wire["ir"]["c"][0]["m"][0]["fl"],
        serde_json::json!([["STATIC"], ["RET"]])
    );
    assert_eq!(
        wire["ir"]["c"][0]["m"][0]["se"],
        serde_json::json!(["io", "mutation"])
    );
    assert_eq!(
        wire["ir"]["c"][0]["m"][0]["ec"],
        serde_json::json!(["sync", "async"])
    );
}

#[test]
fn renderer_exposes_every_method_fact_occurrence_in_order() {
    let hierarchy = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Sample".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
        CoreOp::Flags("M1".into(), vec!["STATIC".into(), "STATIC".into()]),
        CoreOp::Flags("M1".into(), vec!["RET".into()]),
        CoreOp::SideEffect("M1".into(), "io".into()),
        CoreOp::SideEffect("M1".into(), "mutation".into()),
        CoreOp::ExecutionContext("M1".into(), "sync".into()),
        CoreOp::ExecutionContext("M1".into(), "async".into()),
    ]))
    .expect("identity graph is valid");

    let rendered = render_hierarchical_for_llm(&hierarchy, Fidelity::High);
    assert!(rendered.contains("fl:STATIC,STATIC,RET"), "{rendered}");
    assert!(rendered.contains("se:io,mutation"), "{rendered}");
    assert!(rendered.contains("ec:sync,async"), "{rendered}");
}

#[test]
fn legacy_hierarchy_without_schema_marker_still_decodes() {
    let wire = serde_json::json!({
        "file": "alpha-1",
        "v": 7,
        "encoding": "hierarchical",
        "ir": {
            "c": [{
                "n": "C1",
                "nm": "Sample",
                "m": [{
                    "n": "M1",
                    "nm": "work",
                    "fl": ["STATIC", "RET"],
                    "se": "io",
                    "ec": "async"
                }]
            }]
        }
    });

    let decoded = wire_to_ir(&wire).expect("legacy hierarchy remains readable");
    assert_eq!(
        decoded.instructions,
        vec![
            CoreOp::DefClass("C1".into(), "Sample".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
            CoreOp::Flags("M1".into(), vec!["STATIC".into(), "RET".into()]),
            CoreOp::SideEffect("M1".into(), "io".into()),
            CoreOp::ExecutionContext("M1".into(), "async".into()),
        ]
    );
}

#[test]
fn revision_two_rejects_legacy_method_fact_shapes() {
    let result = wire_to_ir(&serde_json::json!({
        "file": "alpha-1",
        "v": 7,
        "encoding": "hierarchical",
        "hs": 2,
        "ir": {
            "c": [{
                "n": "C1",
                "nm": "Sample",
                "m": [{
                    "n": "M1",
                    "nm": "work",
                    "fl": ["STATIC"],
                    "se": "io",
                    "ec": "async"
                }]
            }]
        }
    }));

    assert!(result.is_err(), "revision 2 must enforce revision-2 shapes");
}

#[test]
fn unknown_hierarchy_schema_revision_fails_loudly() {
    let error = wire_to_ir(&serde_json::json!({
        "file": "alpha-1",
        "v": 7,
        "encoding": "hierarchical",
        "hs": 3,
        "ir": { "c": [] }
    }))
    .expect_err("unknown hierarchy schema must not be guessed");

    assert!(
        error
            .to_string()
            .contains("unsupported hierarchical schema version: 3"),
        "unexpected error: {error}"
    );
}
