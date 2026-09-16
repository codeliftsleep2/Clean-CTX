// src/tests/ir/calls.rs
//
// Native call facts (`CoreOp::Call`): producer semantics and wire
// preservation.
//
// Covers RED-CALL23 (every wire/round-trip surface preserves caller, callee,
// and argc) and the producer-level contracts behind RED-CALL2/CALL5/CALL6/
// CALL9/CALL10 (arity is accumulated structurally, arity-distinct facts stay
// distinct, identical facts collapse at the INDEX, not here) and RED-CALL12
// (an invocation with no precise callable owner is never guessed into a fact).

use crate::ir::binary_wire::{decode as binary_decode, encode as binary_encode};
use crate::ir::calls::CallProducer;
use crate::ir::compiler::CompiledIR;
use crate::ir::delta::primary_key_from_tuple;
use crate::ir::hierarchical::{hierarchical_to_ir, ir_to_hierarchical};
use crate::ir::opcodes::CoreOp;
use crate::ir::wire::{op_to_tuple, tuple_to_op};

/// A document whose native call facts cover argc 0, 1, and 2.
fn call_document() -> CompiledIR {
    CompiledIR {
        file_id: "Example.cs".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "Example".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "Process".to_string()),
            CoreOp::Call("M1".to_string(), "Save".to_string(), 1),
            CoreOp::Call("M1".to_string(), "OrderBy".to_string(), 2),
            CoreOp::Call("M1".to_string(), "Reset".to_string(), 0),
        ],
        version: 1,
    }
}

// ── Producer semantics ───────────────────────────────────────────────

#[test]
fn producer_records_zero_arguments_without_argument_captures() {
    let mut producer = CallProducer::new();
    // Only the argument-less query pattern bound this invocation (`Foo()`).
    producer.record_callee(0, 10, 13, "Foo", Some("M1"));

    assert_eq!(
        producer.settle(Some("M1")),
        vec![CoreOp::Call("M1".to_string(), "Foo".to_string(), 0)],
        "zero written arguments must produce argc 0"
    );
    assert_eq!(producer.emitted(), 1);
    assert_eq!(producer.pending(), 0);
}

#[test]
fn producer_accumulates_arguments_across_matches_of_one_invocation() {
    let mut producer = CallProducer::new();
    // `Foo(a, b)`: the argument-less pattern enumerates the invocation once and
    // the arity pattern binds one argument per match. All of them bind the SAME
    // callee node, which is what joins them into one fact.
    producer.record_callee(0, 4, 7, "Foo", Some("M1"));
    producer.record_callee(1, 4, 7, "Foo", Some("M1"));
    producer.record_argument(1);
    producer.record_callee(2, 4, 7, "Foo", Some("M1"));
    producer.record_argument(2);

    assert_eq!(
        producer.settle(Some("M1")),
        vec![CoreOp::Call("M1".to_string(), "Foo".to_string(), 2)],
        "one invocation must produce exactly one fact with argc 2"
    );
}

#[test]
fn producer_keeps_arity_distinct_invocations_apart() {
    let mut producer = CallProducer::new();
    // `Foo(x); Foo(x, y);` — same callee name, different call sites AND arity.
    producer.record_callee(0, 10, 13, "Foo", Some("M1"));
    producer.record_callee(1, 10, 13, "Foo", Some("M1"));
    producer.record_argument(1);
    producer.record_callee(2, 40, 43, "Foo", Some("M1"));
    producer.record_callee(3, 40, 43, "Foo", Some("M1"));
    producer.record_argument(3);
    producer.record_callee(4, 40, 43, "Foo", Some("M1"));
    producer.record_argument(4);

    assert_eq!(
        producer.settle(Some("M1")),
        vec![
            CoreOp::Call("M1".to_string(), "Foo".to_string(), 1),
            CoreOp::Call("M1".to_string(), "Foo".to_string(), 2),
        ],
        "facts must be emitted in document order and keep both arities"
    );
}

#[test]
fn producer_emits_only_the_settled_owners_facts() {
    let mut producer = CallProducer::new();
    producer.record_callee(0, 10, 13, "Foo", Some("M1"));
    producer.record_callee(1, 40, 43, "Bar", Some("M2"));

    assert_eq!(
        producer.settle(Some("M1")),
        vec![CoreOp::Call("M1".to_string(), "Foo".to_string(), 0)],
        "settling one owner must not emit another owner's facts"
    );
    assert_eq!(producer.pending(), 1);
    assert_eq!(
        producer.settle(Some("M2")),
        vec![CoreOp::Call("M2".to_string(), "Bar".to_string(), 0)]
    );
}

#[test]
fn producer_never_guesses_ownership_for_unattributed_invocations() {
    let mut producer = CallProducer::new();
    // No callable contains this invocation (e.g. a field initializer).
    producer.record_callee(0, 10, 13, "Foo", None);

    assert!(
        producer.settle(None).is_empty(),
        "an unattributed invocation must produce no fact"
    );
    assert_eq!(producer.unattributed(), 1);
    assert_eq!(producer.emitted(), 0);
}

// ── RED-CALL23: wire preservation ────────────────────────────────────

#[test]
fn red_call23_named_wire_round_trips_caller_callee_and_argc() {
    for op in call_document().instructions {
        let Some((caller, callee, argc)) = op.call_parts() else {
            continue;
        };
        let tuple = op_to_tuple(&op);
        assert_eq!(tuple[0], "CALL");
        assert_eq!(tuple[1], caller);
        assert_eq!(tuple[2], callee);
        assert_eq!(tuple[3], argc.to_string());
        assert_eq!(
            tuple_to_op(&tuple),
            Some(op),
            "named wire must round-trip the whole fact"
        );
    }
}

#[test]
fn red_call23_binary_wire_round_trips_the_call_stream() {
    let ir = call_document();
    let bytes = binary_encode(&ir);
    let decoded = binary_decode(&bytes).expect("binary wire must decode");
    assert_eq!(
        decoded.instructions, ir.instructions,
        "binary wire must preserve caller, callee, and argc"
    );
}

#[test]
fn red_call23_hierarchical_wire_round_trips_the_call_stream() {
    let ir = call_document();
    let hir = ir_to_hierarchical(&ir);
    assert_eq!(hir.calls.len(), 3, "every call fact must be carried");
    assert_eq!(
        hir.calls
            .iter()
            .map(|call| (
                call.caller.as_str(),
                call.callee.as_str(),
                call.explicit_arg_count
            ))
            .collect::<Vec<_>>(),
        vec![("M1", "Save", 1), ("M1", "OrderBy", 2), ("M1", "Reset", 0)],
        "hierarchical transport must carry the arity explicitly"
    );
    assert_eq!(
        hierarchical_to_ir(&hir),
        ir.instructions,
        "hierarchical round-trip must restore the identical stream"
    );
}

#[test]
fn red_call23_delta_identity_includes_the_argument_count() {
    let argc_one = op_to_tuple(&CoreOp::Call("M1".to_string(), "Foo".to_string(), 1));
    let argc_two = op_to_tuple(&CoreOp::Call("M1".to_string(), "Foo".to_string(), 2));
    assert_ne!(
        primary_key_from_tuple(&argc_one),
        primary_key_from_tuple(&argc_two),
        "two calls differing only in arity must be distinct delta instructions"
    );
}
