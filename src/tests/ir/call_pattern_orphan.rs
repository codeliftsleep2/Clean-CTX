// src/tests/ir/call_pattern_orphan.rs
//
// RED-CALL21 — IRPAT-001 for native call facts.
//
// A consumptive pattern transformation must never consume a `DefMethod(M)`
// while a surviving `CoreOp::Call(M, ...)` would be orphaned: the caller id
// cannot be re-registered from a `PatternOp` (the validator registers method
// identities from `DefMethod` only), so the pattern must DECLINE and leave the
// fully valid, fully annotated stream in place.
//
// Two independent guards are exercised:
//   1. the established CTOR/EMPTY_CTOR orphan guard
//      (`op_is_unrepresentable_method_ref`, which now includes `Call`), and
//   2. the centralized decline check in `try_compress_pattern`
//      (`trailing_region_references_call`), which covers the patterns that had
//      no orphan guard of their own (promise/observable/getter/setter/override).
//
// Each case has a control proving that the SAME region still compresses when no
// call fact references the consumed method (no global weakening of the
// transforms).

use crate::ir::compiler::CompiledIR;
use crate::ir::layers::PatternRecognizer;
use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::CompressingPatternRecognizer;
use crate::ir::validator::{DefaultValidator, IRValidator};

fn compress(instructions: &[CoreOp]) -> Vec<CoreOp> {
    CompressingPatternRecognizer::new().recognize(instructions)
}

fn ctor_stream(call: Option<CoreOp>) -> Vec<CoreOp> {
    // Production shape of a constructor-injection region:
    //   DEF_C + DEF_M(ctor) + SIG(type) + RET + INJECTS(class, [S1])
    // The pattern's `deps` payload comes ONLY from the INJECTS op (a param alone
    // contributes arity, not deps), and INJECTS requires a declared class
    // (validator E006), so the fixture carries both.
    let mut ops = vec![
        CoreOp::DefClass("C1".into(), "Example".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "ctor".into()),
        CoreOp::Param("M1".into(), "P1".into(), "S1".into(), "dep".into()),
        CoreOp::Return("M1".into(), "void".into()),
        CoreOp::Injects("C1".into(), vec!["S1".into()]),
    ];
    if let Some(op) = call {
        ops.push(op);
    }
    ops.push(CoreOp::Flags("M1".into(), vec!["CTOR".into()]));
    ops
}

/// The leading class declaration of every `ctor_stream` fixture.
fn ctor_class() -> CoreOp {
    CoreOp::DefClass("C1".into(), "Example".into())
}

fn promise_stream(call: Option<CoreOp>) -> Vec<CoreOp> {
    let mut ops = vec![
        CoreOp::DefMethod("C1".into(), "M1".into(), "load".into()),
        CoreOp::Return("M1".into(), "Promise".into()),
    ];
    if let Some(op) = call {
        ops.push(op);
    }
    ops
}

fn validate(instructions: &[CoreOp]) -> Vec<String> {
    DefaultValidator::new()
        .validate(&CompiledIR {
            file_id: "Example.cs".into(),
            instructions: instructions.to_vec(),
            version: 1,
        })
        .into_iter()
        .map(|error| error.code)
        .collect()
}

#[test]
fn red_call21_ctor_pattern_declines_when_a_call_would_be_orphaned() {
    let ops = ctor_stream(Some(CoreOp::Call("M1".into(), "Save".into(), 1)));
    let compressed = compress(&ops);

    assert_eq!(
        compressed, ops,
        "the region must stay uncompressed: its surviving CALL references the \
         DefMethod the pattern would consume"
    );
    assert_eq!(
        validate(&compressed),
        Vec::<String>::new(),
        "the preserved stream must be fully valid (no orphaned CALL)"
    );
}

#[test]
fn red_call21_ctor_control_still_compresses_without_a_call() {
    let compressed = compress(&ctor_stream(None));
    assert_eq!(
        compressed,
        vec![
            ctor_class(),
            CoreOp::Pattern("CTOR".into(), vec!["C1".into(), "M1".into(), "S1".into()]),
        ],
        "a ctor region with no call fact must keep compressing exactly as before"
    );
}

#[test]
fn red_call21_promise_pattern_declines_when_a_call_would_be_orphaned() {
    // The promise pattern has no orphan guard of its own, so the centralized
    // `trailing_region_references_call` check is what protects the fact.
    let ops = promise_stream(Some(CoreOp::Call("M1".into(), "Save".into(), 1)));
    let compressed = compress(&ops);

    assert_eq!(
        compressed, ops,
        "no consumptive pattern may orphan a surviving CALL's caller identity"
    );
    assert_eq!(validate(&compressed), Vec::<String>::new());
}

#[test]
fn red_call21_promise_control_still_compresses_without_a_call() {
    let compressed = compress(&promise_stream(None));
    assert_eq!(
        compressed,
        vec![CoreOp::Pattern(
            "PROMISE".into(),
            vec!["C1".into(), "M1".into(), "Promise".into()],
        )],
        "a promise region with no call fact must keep compressing as before"
    );
}

#[test]
fn red_call21_a_call_for_another_method_does_not_block_compression() {
    // The guard is scoped to the method being consumed: an unrelated call fact
    // (different caller id) must not disable compression globally.
    let ops = ctor_stream(Some(CoreOp::Call("M9".into(), "Save".into(), 1)));
    let compressed = compress(&ops);

    assert_eq!(
        compressed.get(1),
        Some(&CoreOp::Pattern(
            "CTOR".into(),
            vec!["C1".into(), "M1".into(), "S1".into()],
        )),
        "an unrelated call fact must not block the pattern"
    );
    assert_eq!(
        compressed
            .iter()
            .filter(|op| matches!(op, CoreOp::Call(..)))
            .count(),
        1,
        "the unrelated call fact must survive untouched"
    );
    assert!(
        validate(&compressed).contains(&"E011".to_string()),
        "the validator must still flag the unrelated orphaned caller as E011"
    );
}
