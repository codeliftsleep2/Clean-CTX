// src/tests/ir/layers/patterns.rs
//
// Tests for the Pattern Recognizer (Layer 4).
// Verifies constructor injection, observable, and getter/setter pattern detection.

use crate::ir::layers::PatternRecognizer;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::opcodes::{CoreOp, DeclarationModifier, PatternFact};

fn make_defmethod(cid: &str, mid: &str, name: &str) -> CoreOp {
    CoreOp::DefMethod(cid.into(), mid.into(), name.into())
}

fn make_param(mid: &str, pid: &str, ty: &str, name: &str) -> CoreOp {
    CoreOp::Param(mid.into(), pid.into(), ty.into(), name.into())
}

fn make_ret(mid: &str, ty: &str) -> CoreOp {
    CoreOp::Return(mid.into(), ty.into())
}

fn make_modifiers(tid: &str, modifiers: Vec<DeclarationModifier>) -> CoreOp {
    CoreOp::MethodModifiers(tid.into(), modifiers)
}

// ── Constructor Pattern Tests ─────────────────────────

#[test]
fn recognize_constructor_injection() {
    let instructions = vec![
        make_defmethod("C1", "M1", "constructor"),
        make_param("M1", "P1", "$s", "input"),
        make_param("M1", "P2", "$n", "count"),
        make_ret("M1", "$v"),
    ];

    let recognizer = CodePatternRecognizer::new();
    let result = recognizer.recognize(&instructions);

    // Constructor should get a CTOR flag
    let has_ctor = result.iter().any(|op| {
        matches!(op, CoreOp::PatternFacts(m, facts) if m == "M1" && facts.contains(&PatternFact::Constructor))
    });
    assert!(
        has_ctor,
        "Constructor injection pattern should produce CTOR flag: {:?}",
        result
    );

    // Original instructions should be preserved too
    assert!(
        result.len() >= instructions.len(),
        "Pattern should not remove instructions, only add facts. result={}, expected>={}",
        result.len(),
        instructions.len()
    );
}

// ── Observable Pattern Tests ──────────────────────────

#[test]
fn observable_semantics_additive_recognizes_declared_observable() {
    let instructions = vec![
        make_defmethod("C1", "M1", "fetchData"),
        make_ret("M1", "Observable<User>"),
    ];

    let recognizer = CodePatternRecognizer::new();
    let result = recognizer.recognize(&instructions);

    let has_observable = result.iter().any(|op| {
        matches!(op, CoreOp::PatternFacts(m, facts) if m == "M1" && facts.contains(&PatternFact::Observable))
    });
    assert!(
        has_observable,
        "Observable pattern should produce OBSERVABLE flag: {:?}",
        result
    );
}

#[test]
fn observable_semantics_additive_never_borrows_evidence_from_another_method() {
    let instructions = vec![
        make_defmethod("C1", "M1", "methodB"),
        make_ret("M1", "$v"),
        make_defmethod("C1", "M2", "methodA"),
        make_ret("M2", "Observable<number>"),
    ];

    let result = CodePatternRecognizer::new().recognize(&instructions);

    assert!(
        !result.iter().any(|op| {
            matches!(op, CoreOp::PatternFacts(m, facts) if m == "M1" && facts.contains(&PatternFact::Observable))
        }),
        "a plain method must not borrow Observable evidence from a neighboring method: {result:?}"
    );
    assert!(
        result.iter().any(|op| {
            matches!(op, CoreOp::PatternFacts(m, facts) if m == "M2" && facts.contains(&PatternFact::Observable))
        }),
        "the method that owns both pieces of evidence should retain its Observable fact: {result:?}"
    );
}

#[test]
fn observable_semantics_additive_rejects_non_observable_evidence() {
    let async_promise = vec![
        make_defmethod("C1", "M1", "asyncPromise"),
        make_ret("M1", "$P"),
        make_modifiers("M1", vec![DeclarationModifier::Async]),
    ];
    let async_only = vec![
        make_defmethod("C1", "M2", "asyncOnly"),
        make_ret("M2", "$v"),
        make_modifiers("M2", vec![DeclarationModifier::Async]),
    ];
    let recognizer = CodePatternRecognizer::new();

    for instructions in [&async_promise, &async_only] {
        let result = recognizer.recognize(instructions);
        assert!(
            !result
                .iter()
                .any(|op| matches!(op, CoreOp::PatternFacts(_, facts) if facts.contains(&PatternFact::Observable))),
            "Promise or async evidence must not produce an Observable fact: {result:?}"
        );
    }
}

// ── Getter/Setter Pattern Tests ───────────────────────

#[test]
fn recognize_getter() {
    let instructions = vec![make_defmethod("C1", "M1", "get fullName")];

    let recognizer = CodePatternRecognizer::new();
    let result = recognizer.recognize(&instructions);

    let has_getter = result.iter().any(|op| {
        matches!(op, CoreOp::PatternFacts(m, facts) if m == "M1" && matches!(facts.as_slice(), [PatternFact::Getter(_)]))
    });
    assert!(
        has_getter,
        "Getter pattern should produce GETTER flag: {:?}",
        result
    );
}

#[test]
fn recognize_setter() {
    let instructions = vec![make_defmethod("C1", "M1", "set fullName")];

    let recognizer = CodePatternRecognizer::new();
    let result = recognizer.recognize(&instructions);

    let has_setter = result.iter().any(|op| {
        matches!(op, CoreOp::PatternFacts(m, facts) if m == "M1" && matches!(facts.as_slice(), [PatternFact::Setter(_)]))
    });
    assert!(
        has_setter,
        "Setter pattern should produce SETTER flag: {:?}",
        result
    );
}

// ── Pass-Through Tests ────────────────────────────────

#[test]
fn unrecognized_patterns_pass_through() {
    let instructions = vec![
        make_defmethod("C1", "M1", "doWork"),
        make_param("M1", "P1", "$s", "input"),
        make_ret("M1", "$b"),
    ];

    let recognizer = CodePatternRecognizer::new();
    let result = recognizer.recognize(&instructions);

    // All instructions should be preserved unchanged
    assert_eq!(
        result.len(),
        instructions.len(),
        "Unrecognized patterns should pass through unchanged"
    );
}

#[test]
fn empty_instructions_pass_through() {
    let instructions: Vec<CoreOp> = vec![];
    let recognizer = CodePatternRecognizer::new();
    let result = recognizer.recognize(&instructions);
    assert!(
        result.is_empty(),
        "Empty instructions should produce empty output"
    );
}

// ── PatternRecognizer Trait Tests ─────────────────────

#[test]
fn pattern_recognizer_trait_dispatch() {
    let recognizer = CodePatternRecognizer::new();
    let instructions = vec![make_defmethod("C1", "M1", "get name")];

    let result: Vec<CoreOp> = PatternRecognizer::recognize(&recognizer, &instructions);
    assert!(!result.is_empty(), "Trait dispatch should work");
}
