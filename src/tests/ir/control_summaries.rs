use crate::compression::Fidelity;
use crate::ir::binary_wire::{decode, encode};
use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{
    ir_to_hierarchical_wire, try_ir_to_hierarchical, wire_to_ir as hierarchical_wire_to_ir,
};
use crate::ir::opcodes::{ControlSummary, CoreOp, PatternFact};
use crate::ir::render_llm::render_hierarchical_for_llm;
use crate::ir::wire::{op_to_tuple, tuple_to_op};

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "control-summary-contract".into(),
        version: 1,
        instructions,
    }
}

#[test]
fn typed_vocabulary_is_closed_and_serialization_is_stable() {
    let cases = [
        (ControlSummary::Branch, "IF"),
        (ControlSummary::Loop, "LOOP"),
        (ControlSummary::Return, "RET"),
        (ControlSummary::Throw, "THROW"),
    ];
    for (summary, serialized) in cases {
        assert_eq!(summary.as_str(), serialized);
        assert_eq!(ControlSummary::from_serialized(serialized), Some(summary));
    }
    assert_eq!(ControlSummary::from_serialized("CTOR"), None);
}

#[test]
fn named_and_binary_wires_preserve_order_and_duplicates() {
    let operation = CoreOp::ControlSummary(
        "M1".into(),
        vec![
            ControlSummary::Branch,
            ControlSummary::Return,
            ControlSummary::Branch,
        ],
    );
    let tuple = op_to_tuple(&operation);
    assert_eq!(tuple, vec!["CTRL_SUM", "M1", "IF", "RET", "IF"]);
    assert_eq!(tuple_to_op(&tuple), Some(operation.clone()));
    assert_eq!(
        tuple_to_op(&["CTRL_SUM".into(), "M1".into(), "CTOR".into()]),
        None
    );

    let ir = compiled(vec![operation]);
    let bytes = encode(&ir);
    assert_eq!(bytes[2], 0x04, "Phase 8 corrected physical version");
    let decoded = decode(&bytes).expect("binary round trip");
    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);
}

#[test]
fn canonical_hierarchy_and_compact_llm_projection_are_distinct() {
    let ir = compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
        CoreOp::ControlSummary(
            "M1".into(),
            vec![ControlSummary::Branch, ControlSummary::Return],
        ),
        CoreOp::PatternFacts("M1".into(), vec![PatternFact::Observable]),
    ]);
    let hierarchy = try_ir_to_hierarchical(&ir).expect("valid identity graph");
    let method = &hierarchy.classes[0].methods[0];
    assert_eq!(
        method.control_summaries,
        vec![vec![ControlSummary::Branch, ControlSummary::Return]]
    );
    assert_eq!(method.pattern_facts, vec![vec![PatternFact::Observable]]);

    let rendered = render_hierarchical_for_llm(&hierarchy, Fidelity::Low);
    assert!(rendered.contains("ctl:IF,RET"), "{rendered}");
    assert!(rendered.contains("pf:OBSERVABLE"), "{rendered}");

    let wire = ir_to_hierarchical_wire(&ir);
    assert_eq!(wire["hs"], 7);
    assert_eq!(
        wire["ir"]["c"][0]["m"][0]["cs"],
        serde_json::json!([["IF", "RET"]])
    );
}

#[test]
fn revision_four_moves_only_control_vocabulary_out_of_residual_flags() {
    let wire = serde_json::json!({
        "file": "legacy",
        "v": 1,
        "encoding": "hierarchical",
        "hs": 4,
        "ir": { "c": [{
            "n": "C1", "nm": "Owner", "m": [{
                "n": "M1", "nm": "work", "fl": [["IF", "RET"], ["CTOR"]]
            }]
        }]}
    });
    let decoded = hierarchical_wire_to_ir(&wire).expect("revision four remains readable");
    assert!(decoded.instructions.contains(&CoreOp::ControlSummary(
        "M1".into(),
        vec![ControlSummary::Branch, ControlSummary::Return]
    )));
    assert!(decoded.instructions.contains(&CoreOp::PatternFacts(
        "M1".into(),
        vec![PatternFact::Constructor]
    )));
}

#[test]
fn generic_flags_reject_control_summary_spellings() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
        CoreOp::Flags("M1".into(), vec!["IF".into()]),
    ]))
    .expect_err("control summaries require their typed operation");
    assert!(
        error.to_string().contains("semantic-family operation"),
        "{error}"
    );
}
