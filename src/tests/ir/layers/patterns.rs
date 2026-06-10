// src/tests/ir/layers/patterns.rs
//
// Tests for the Pattern Recognizer (Layer 4).
// Verifies constructor injection, observable, and getter/setter pattern detection.

use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::layers::PatternRecognizer;
use crate::ir::opcodes::CoreOp;

fn make_defmethod(cid: &str, mid: &str, name: &str) -> CoreOp {
    CoreOp::DefMethod(cid.into(), mid.into(), name.into())
}

fn make_param(mid: &str, pid: &str, ty: &str, name: &str) -> CoreOp {
    CoreOp::Param(mid.into(), pid.into(), ty.into(), name.into())
}

fn make_ret(mid: &str, ty: &str) -> CoreOp {
    CoreOp::Return(mid.into(), ty.into())
}

fn make_flags(tid: &str, flags: Vec<&str>) -> CoreOp {
    CoreOp::Flags(tid.into(), flags.iter().map(|s| s.to_string()).collect())
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
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"CTOR".to_string()))
    });
    assert!(has_ctor, "Constructor injection pattern should produce CTOR flag: {:?}", result);

    // Original instructions should be preserved too
    assert!(result.len() >= instructions.len(),
        "Pattern should not remove instructions, only add flags. result={}, expected>={}",
        result.len(), instructions.len());
}

// ── Observable Pattern Tests ──────────────────────────

#[test]
fn recognize_observable_return() {
    let instructions = vec![
        make_defmethod("C1", "M1", "fetchData"),
        make_ret("M1", "$P"),
        make_flags("M1", vec!["ASYNC"]),
    ];

    let recognizer = CodePatternRecognizer::new();
    let result = recognizer.recognize(&instructions);

    let has_observable = result.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"OBSERVABLE".to_string()))
    });
    assert!(has_observable, "Observable pattern should produce OBSERVABLE flag: {:?}", result);
}

// ── Getter/Setter Pattern Tests ───────────────────────

#[test]
fn recognize_getter() {
    let instructions = vec![
        make_defmethod("C1", "M1", "get fullName"),
    ];

    let recognizer = CodePatternRecognizer::new();
    let result = recognizer.recognize(&instructions);

    let has_getter = result.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"GETTER".to_string()))
    });
    assert!(has_getter, "Getter pattern should produce GETTER flag: {:?}", result);
}

#[test]
fn recognize_setter() {
    let instructions = vec![
        make_defmethod("C1", "M1", "set fullName"),
    ];

    let recognizer = CodePatternRecognizer::new();
    let result = recognizer.recognize(&instructions);

    let has_setter = result.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"SETTER".to_string()))
    });
    assert!(has_setter, "Setter pattern should produce SETTER flag: {:?}", result);
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
    assert_eq!(result.len(), instructions.len(),
        "Unrecognized patterns should pass through unchanged");
}

#[test]
fn empty_instructions_pass_through() {
    let instructions: Vec<CoreOp> = vec![];
    let recognizer = CodePatternRecognizer::new();
    let result = recognizer.recognize(&instructions);
    assert!(result.is_empty(), "Empty instructions should produce empty output");
}

// ── PatternRecognizer Trait Tests ─────────────────────

#[test]
fn pattern_recognizer_trait_dispatch() {
    let recognizer = CodePatternRecognizer::new();
    let instructions = vec![
        make_defmethod("C1", "M1", "get name"),
    ];

    let result: Vec<CoreOp> = PatternRecognizer::recognize(&recognizer, &instructions);
    assert!(!result.is_empty(), "Trait dispatch should work");
}