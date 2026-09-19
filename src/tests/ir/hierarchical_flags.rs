// src/tests/ir/hierarchical_flags.rs
//
// RED-FLAG1..RED-FLAG6, RED-FLAG11, RED-FLAG12 plus the round-trip, wire and
// flat-stream nonregressions: repeated typed method facts for ONE method id
// must remain distinct occurrences at the hierarchical projection instead of
// being overwritten, merged, or deduplicated.
//
// Declaration modifiers and residual flags have distinct typed operations:
// language layers emit `MethodModifiers`, while the core pipeline emits typed
// `ControlSummary` facts. Residual `Flags` carry pattern classifications.
//
// Each family can carry repeated operations. The hierarchy stores one inner
// vector per occurrence in its distinct `modifiers` and `flags` fields.
//
// Projection semantics pinned here: occurrence order, payload order, and
// duplicate values are all preserved.
//
// ── ROUND-TRIP SCOPE (deliberate) ───────────────────────────────────────
// Flat → hierarchical → flat restores each original `Flags` occurrence.
//
// ── NOT changed by this fix (pinned below) ──────────────────────────────
// The flat instruction stream itself — what pattern recognition and the
// named/binary wire consume — keeps each typed operation separate.

use crate::compression::Fidelity;
use crate::ir::binary_wire::{decode, encode};
use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{HierarchicalIR, hierarchical_to_ir, ir_to_hierarchical};
use crate::ir::opcodes::{ControlSummary, CoreOp, DeclarationModifier};
use crate::ir::render_llm::render_hierarchical_for_llm;
use crate::ir::wire::{ir_to_wire, op_to_tuple, tuple_to_op, wire_to_ir};

// ── Flat-stream / hierarchical helpers ─────────────────────────────────

/// A flat `CompiledIR` around one instruction stream.
fn flat_ir(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions,
    }
}

/// One typed control-summary op, or a residual pattern-fact op.
fn flags(mid: &str, values: &[&str]) -> CoreOp {
    let summaries = values
        .iter()
        .map(|value| ControlSummary::from_serialized(value))
        .collect::<Option<Vec<_>>>();
    match summaries {
        Some(summaries) => CoreOp::ControlSummary(mid.to_string(), summaries),
        None => CoreOp::Flags(
            mid.to_string(),
            values.iter().map(|value| value.to_string()).collect(),
        ),
    }
}

fn modifiers(mid: &str, values: &[DeclarationModifier]) -> CoreOp {
    CoreOp::MethodModifiers(mid.to_string(), values.to_vec())
}

/// Every method node in the hierarchical IR as `(id, name, flags)`.
fn methods(hir: &HierarchicalIR) -> Vec<(String, String, Vec<String>)> {
    hir.classes
        .iter()
        .flat_map(|class| class.methods.iter())
        .map(|method| {
            (
                method.id.clone(),
                method.name.clone(),
                method
                    .modifiers
                    .iter()
                    .flatten()
                    .map(|modifier| modifier.as_str().to_string())
                    .chain(
                        method
                            .control_summaries
                            .iter()
                            .flatten()
                            .map(|summary| summary.as_str().to_string()),
                    )
                    .chain(method.flags.iter().flatten().cloned())
                    .collect(),
            )
        })
        .collect()
}

/// The flags of the single method named `name` in the hierarchical IR.
fn method_flags(hir: &HierarchicalIR, name: &str) -> Vec<String> {
    let all = methods(hir);
    all.iter()
        .find(|(_, declared, _)| declared == name)
        .map(|(_, _, values)| values.clone())
        .unwrap_or_else(|| {
            panic!(
                "no method named {name}; declared: {:?}",
                all.iter()
                    .map(|(_, declared, _)| declared.as_str())
                    .collect::<Vec<_>>()
            )
        })
}

/// Every method fact in the flat stream that references `mid`, in stream order.
fn flag_ops_for(ir: &CompiledIR, mid: &str) -> Vec<Vec<String>> {
    ir.instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::Flags(id, values) if id == mid => Some(values.clone()),
            CoreOp::ControlSummary(id, values) if id == mid => Some(
                values
                    .iter()
                    .map(|summary| summary.as_str().to_string())
                    .collect(),
            ),
            _ => None,
        })
        .collect()
}

fn modifier_ops_for(ir: &CompiledIR, mid: &str) -> Vec<Vec<DeclarationModifier>> {
    ir.instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::MethodModifiers(id, values) if id == mid => Some(values.clone()),
            _ => None,
        })
        .collect()
}

/// The rendered skeleton of a hierarchical IR.
fn render(hir: &HierarchicalIR, fidelity: Fidelity) -> String {
    render_hierarchical_for_llm(hir, fidelity)
}

// ── RED-FLAG1: two `Flags` ops preserve both families ──────────────────

#[test]
fn red_flag1_two_flag_ops_preserve_both_families() {
    // The reported shape: the layer's declaration op, then the flushed
    // control-flow op.
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "work".to_string()),
        modifiers("M1", &[DeclarationModifier::Static]),
        flags("M1", &["RET"]),
    ]);

    let hir = ir_to_hierarchical(&ir);
    assert_eq!(
        method_flags(&hir, "work"),
        vec!["STATIC".to_string(), "RET".to_string()],
        "both producers' families must survive the projection"
    );
}

// ── RED-FLAG2: the last write no longer wins ───────────────────────────

#[test]
fn red_flag2_last_write_no_longer_wins() {
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "load".to_string()),
        modifiers("M1", &[DeclarationModifier::Async]),
        flags("M1", &["IF", "RET"]),
    ]);

    let hir = ir_to_hierarchical(&ir);
    assert_eq!(
        method_flags(&hir, "load"),
        vec!["ASYNC".to_string(), "IF".to_string(), "RET".to_string()],
        "the second op must extend the first, not replace it"
    );
}

// ── RED-FLAG3: duplicates remain semantic occurrences ─────────────────

#[test]
fn red_flag3_duplicate_values_are_preserved() {
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "save".to_string()),
        modifiers(
            "M1",
            &[DeclarationModifier::Private, DeclarationModifier::Static],
        ),
        modifiers("M1", &[DeclarationModifier::Static]),
        flags("M1", &["RET"]),
    ]);

    let hir = ir_to_hierarchical(&ir);
    let merged = method_flags(&hir, "save");
    assert_eq!(
        merged,
        vec![
            "PRIVATE".to_string(),
            "STATIC".to_string(),
            "STATIC".to_string(),
            "RET".to_string()
        ],
        "payload order and duplicate values must survive"
    );
    assert_eq!(
        merged
            .iter()
            .filter(|flag| flag.as_str() == "STATIC")
            .count(),
        2,
        "both STATIC occurrences must survive: {merged:?}"
    );
}

// ── RED-FLAG4: three producer-style writes for one method ──────────────

#[test]
fn red_flag4_three_producer_style_writes_remain_ordered() {
    // Three writes in the order the flat stream can carry them: a
    // declaration modifier, a pattern-additive annotation, then the
    // control-flow family.
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "stream".to_string()),
        modifiers("M1", &[DeclarationModifier::Private]),
        flags("M1", &["OBSERVABLE"]),
        flags("M1", &["IF", "RET", "LOOP", "RET"]),
    ]);

    let hir = ir_to_hierarchical(&ir);
    assert_eq!(
        method_flags(&hir, "stream"),
        vec![
            "PRIVATE".to_string(),
            "OBSERVABLE".to_string(),
            "IF".to_string(),
            "RET".to_string(),
            "LOOP".to_string(),
            "RET".to_string()
        ],
        "every write contributes in occurrence and payload order"
    );
}

// ── RED-FLAG5: method isolation ────────────────────────────────────────

#[test]
fn red_flag5_method_isolation() {
    // The stream order production really emits: the first method's flushed
    // control-flow op lands immediately before the next declaration, in the
    // same class.
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Pairing".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "first".to_string()),
        modifiers("M1", &[DeclarationModifier::Static]),
        flags("M1", &["RET"]),
        CoreOp::DefMethod("C1".to_string(), "M2".to_string(), "second".to_string()),
        modifiers("M2", &[DeclarationModifier::Async]),
        flags("M2", &["THROW"]),
    ]);

    let hir = ir_to_hierarchical(&ir);
    assert_eq!(
        method_flags(&hir, "first"),
        vec!["STATIC".to_string(), "RET".to_string()],
        "M1 keeps its own two families"
    );
    assert_eq!(
        method_flags(&hir, "second"),
        vec!["ASYNC".to_string(), "THROW".to_string()],
        "M2 is not contaminated by M1 and keeps its own two families"
    );
}

// ── RED-FLAG6: the rendered line carries both families ─────────────────

#[test]
fn red_flag6_rendered_line_carries_declaration_and_control_flow() {
    // A real compressed-method shape: one parameter, a return type, the
    // layer's declaration op, then the flushed control-flow op.
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "pick".to_string()),
        CoreOp::Param(
            "M1".to_string(),
            "P1".to_string(),
            "$s[]".to_string(),
            "values".to_string(),
        ),
        CoreOp::Return("M1".to_string(), "$n".to_string()),
        modifiers("M1", &[DeclarationModifier::Static]),
        flags("M1", &["IF", "RET"]),
    ]);

    let hir = ir_to_hierarchical(&ir);
    let out = render(&hir, Fidelity::Medium);
    // The LLM projection stays compact without mirroring canonical structure.
    assert!(out.contains(" mod:STATIC"), "{out}");
    assert!(out.contains(" ctl:IF,RET"), "{out}");
    assert!(out.contains("STATIC"), "{out}");
    assert!(out.contains("RET"), "{out}");
}

// ── RED-FLAG11: a declaration-only method is unchanged ────────────────

#[test]
fn red_flag11_declaration_only_method_unchanged() {
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "getUser".to_string()),
        modifiers("M1", &[DeclarationModifier::Static]),
    ]);

    let hir = ir_to_hierarchical(&ir);
    assert_eq!(method_flags(&hir, "getUser"), vec!["STATIC".to_string()]);

    let out = render(&hir, Fidelity::Low);
    assert!(out.contains("mod:STATIC"), "{out}");
    // Accumulation must not invent values: a single op stays a single value.
    assert!(!out.contains("mod:STATIC,"), "{out}");
}

// ── RED-FLAG12: a control-flow-only method is unchanged ────────────────

#[test]
fn red_flag12_control_flow_only_method_unchanged() {
    // A method with no declaration/modifier op at all: exactly the previous
    // behavior must remain.
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "loop".to_string()),
        flags("M1", &["RET", "IF"]),
    ]);

    let hir = ir_to_hierarchical(&ir);
    assert_eq!(
        method_flags(&hir, "loop"),
        vec!["RET".to_string(), "IF".to_string()]
    );

    let out = render(&hir, Fidelity::Low);
    assert!(out.contains("ctl:RET,IF"), "{out}");
}

// ── Round trip: information and op multiplicity are preserved ──────────

#[test]
fn round_trip_preserves_flag_occurrences() {
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "work".to_string()),
        modifiers("M1", &[DeclarationModifier::Static]),
        flags("M1", &["RET"]),
    ]);

    // flat → hierarchical: one ordered entry for each producer occurrence.
    let hir = ir_to_hierarchical(&ir);
    assert_eq!(
        method_flags(&hir, "work"),
        vec!["STATIC".to_string(), "RET".to_string()]
    );

    let method = &hir.classes[0].methods[0];
    assert_eq!(method.modifiers, vec![vec![DeclarationModifier::Static]]);
    assert_eq!(method.control_summaries, vec![vec![ControlSummary::Return]]);

    // hierarchical → flat: each occurrence is restored independently.
    let restored = hierarchical_to_ir(&hir);
    let projected = methods(&ir_to_hierarchical(&flat_ir(restored.clone())))[0]
        .2
        .clone();
    assert_eq!(
        projected,
        vec!["STATIC".to_string(), "RET".to_string()],
        "both typed families must survive"
    );

    // No semantic flag value was lost anywhere in the round trip.
    let mut values: Vec<String> = Vec::new();
    for value in &projected {
        if !values.iter().any(|seen| seen == value) {
            values.push(value.clone());
        }
    }
    assert_eq!(values, vec!["STATIC".to_string(), "RET".to_string()]);

    // The projection never rewrites the stream it reads: the original flat
    // stream still holds the two separate ops.
    assert_eq!(flag_ops_for(&ir, "M1").len(), 1);
    assert_eq!(
        restored.len(),
        4,
        "DefClass + DefMethod + modifier and flag ops: {restored:?}"
    );
}

// ── Flat-stream nonregression (patterns + wire) ────────────────────────

#[test]
fn hierarchical_encode_does_not_rewrite_the_flat_stream() {
    // Pattern recognition (`ir::layers::patterns`, the consumptive
    // recognizer and IRPAT-001's trailing-Flags runs) consumes the FLAT
    // stream; the projection must not touch the input it reads.
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "work".to_string()),
        modifiers("M1", &[DeclarationModifier::Static]),
        flags("M1", &["RET"]),
    ]);
    let before = ir.instructions.clone();

    let _hir = ir_to_hierarchical(&ir);

    assert_eq!(ir.instructions, before, "the flat stream is the input");
    assert_eq!(
        flag_ops_for(&ir, "M1"),
        vec![vec!["RET".to_string()]],
        "the typed summary op must remain unchanged"
    );
}

#[test]
fn named_wire_pins_separate_modifier_and_control_summary_ops() {
    // Each semantic family serializes as its own named tuple.
    assert_eq!(
        op_to_tuple(&modifiers("M1", &[DeclarationModifier::Static])),
        vec!["MOD_M", "M1", "STATIC"]
    );
    assert_eq!(
        op_to_tuple(&flags("M1", &["RET"])),
        vec!["CTRL_SUM", "M1", "RET"]
    );

    // ...each tuple still round-trips on its own...
    for op in [
        modifiers("M1", &[DeclarationModifier::Static]),
        flags("M1", &["RET"]),
    ] {
        let tuple = op_to_tuple(&op);
        assert_eq!(tuple_to_op(&tuple), Some(op));
    }

    // ...and an IR carrying BOTH ops round-trips through the named wire with
    // both ops independently present.
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "work".to_string()),
        modifiers("M1", &[DeclarationModifier::Static]),
        flags("M1", &["RET"]),
    ]);
    let restored = wire_to_ir(&ir_to_wire(&ir)).expect("named wire round trip");
    assert_eq!(
        methods(&ir_to_hierarchical(&restored))[0].2,
        vec!["STATIC".to_string(), "RET".to_string()]
    );
}

#[test]
fn binary_wire_round_trips_separate_modifier_and_control_summary_ops() {
    // The typed family has its own additive opcode under physical 0x03.
    let ir = flat_ir(vec![
        CoreOp::DefClass("C1".to_string(), "Sample".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "work".to_string()),
        modifiers("M1", &[DeclarationModifier::Static]),
        flags("M1", &["RET"]),
    ]);
    let bytes = encode(&ir);
    let decoded = decode(&bytes).expect("binary wire round trip");
    assert!(
        decoded
            .instructions
            .contains(&modifiers("M1", &[DeclarationModifier::Static]))
    );
    assert!(decoded.instructions.contains(&flags("M1", &["RET"])));
}

// ── Cross-language pipeline probes ─────────────────────────────────────
// RED-FLAG7..RED-FLAG10 compile REAL fixtures through each language's
// production compiler configuration, so they live in a child module (each
// file respects the active size ceiling) and share this module's helpers
// through `use super::*`.
#[path = "hierarchical_flags_languages.rs"]
mod languages;
