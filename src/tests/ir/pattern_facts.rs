use crate::compression::Fidelity;
use crate::ir::binary_wire::{decode, encode};
use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{
    ir_to_hierarchical_wire, try_ir_to_hierarchical, wire_to_ir as hierarchical_wire_to_ir,
};
use crate::ir::opcodes::{CoreOp, PatternFact};
use crate::ir::render_llm::render_hierarchical_for_llm;
use crate::ir::wire::{op_to_tuple, tuple_to_op};

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "pattern-fact-contract".into(),
        version: 1,
        instructions,
    }
}

#[test]
fn typed_pattern_fact_payload_is_closed_and_ordered() {
    let values = vec!["CTOR".into(), "GETTER".into(), "name".into(), "CTOR".into()];
    assert_eq!(
        PatternFact::parse_all(&values),
        Some(vec![
            PatternFact::Constructor,
            PatternFact::Getter("name".into()),
            PatternFact::Constructor,
        ])
    );
    assert_eq!(PatternFact::parse_all(&["GETTER".into()]), None);
    assert_eq!(PatternFact::parse_all(&["UNKNOWN".into()]), None);
}

#[test]
fn named_and_binary_wires_preserve_pattern_fact_payloads() {
    let operation = CoreOp::PatternFacts(
        "M1".into(),
        vec![
            PatternFact::Constructor,
            PatternFact::Getter("name".into()),
            PatternFact::Constructor,
        ],
    );
    let tuple = op_to_tuple(&operation);
    assert_eq!(
        tuple,
        vec!["PAT_FACT", "M1", "CTOR", "GETTER", "name", "CTOR"]
    );
    assert_eq!(tuple_to_op(&tuple), Some(operation.clone()));

    let ir = compiled(vec![operation]);
    let bytes = encode(&ir);
    assert_eq!(bytes[2], 0x03, "Phase 6C does not claim physical 0x04");
    let decoded = decode(&bytes).expect("binary round trip");
    assert_eq!(decoded.instructions, ir.instructions);
}

#[test]
fn canonical_hierarchy_and_compact_llm_pattern_views_are_distinct() {
    let ir = compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "get name".into()),
        CoreOp::PatternFacts(
            "M1".into(),
            vec![PatternFact::Getter("name".into()), PatternFact::Constructor],
        ),
    ]);
    let hierarchy = try_ir_to_hierarchical(&ir).expect("valid identity graph");
    assert_eq!(
        hierarchy.classes[0].methods[0].pattern_facts,
        vec![vec![
            PatternFact::Getter("name".into()),
            PatternFact::Constructor,
        ]]
    );
    let rendered = render_hierarchical_for_llm(&hierarchy, Fidelity::Low);
    assert!(rendered.contains("pf:GETTER(name),CTOR"), "{rendered}");
    assert_eq!(ir_to_hierarchical_wire(&ir)["hs"], 7);
}

#[test]
fn revision_five_moves_only_complete_pattern_fact_occurrences() {
    let wire = serde_json::json!({
        "file": "legacy", "v": 1, "encoding": "hierarchical", "hs": 5,
        "ir": { "c": [{
            "n": "C1", "nm": "Owner", "m": [{
                "n": "M1", "nm": "work",
                "fl": [["CTOR"], ["GETTER", "name"], ["LEGACY"]]
            }]
        }]}
    });
    let decoded = hierarchical_wire_to_ir(&wire).expect("revision five remains readable");
    assert!(decoded.instructions.contains(&CoreOp::PatternFacts(
        "M1".into(),
        vec![PatternFact::Constructor]
    )));
    assert!(decoded.instructions.contains(&CoreOp::PatternFacts(
        "M1".into(),
        vec![PatternFact::Getter("name".into())]
    )));
    assert!(
        decoded
            .instructions
            .contains(&CoreOp::Flags("M1".into(), vec!["LEGACY".into()]))
    );
}

#[test]
fn generic_flags_reject_pattern_fact_spellings() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
        CoreOp::Flags("M1".into(), vec!["CTOR".into()]),
    ]))
    .expect_err("pattern facts require their typed operation");
    assert!(
        error.to_string().contains("semantic-family operation"),
        "{error}"
    );
}
