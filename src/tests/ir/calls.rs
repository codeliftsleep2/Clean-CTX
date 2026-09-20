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

/// An EXACT call fact: every written argument is exactly one argument.
fn exact(caller: &str, callee: &str, argc: usize) -> CoreOp {
    CoreOp::Call(caller.to_string(), callee.to_string(), argc, false)
}

/// A SPREAD call fact: at least one written argument expands at run time, so
/// `argc` is a written-node count and never an exact arity.
fn spread(caller: &str, callee: &str, argc: usize) -> CoreOp {
    CoreOp::Call(caller.to_string(), callee.to_string(), argc, true)
}

/// A document whose native call facts cover argc 0, 1, and 2.
fn call_document() -> CompiledIR {
    CompiledIR {
        file_id: "Example.cs".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "Example".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "Process".to_string()),
            exact("M1", "Save", 1),
            exact("M1", "OrderBy", 2),
            exact("M1", "Reset", 0),
        ],
        version: 1,
    }
}

/// A document whose call facts cover the exact/shape distinction: `Save` is
/// called once with one written argument and once with one written argument
/// that expands, so the two facts describe the same name, the same written
/// count, and materially different evidence.
fn spread_document() -> CompiledIR {
    CompiledIR {
        file_id: "Example.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "Example".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "Process".to_string()),
            exact("M1", "Save", 1),
            spread("M1", "Save", 1),
        ],
        version: 1,
    }
}

/// Field-level `(caller, callee, argc, has_spread)` facts of a stream.
fn call_facts_of(instructions: &[CoreOp]) -> Vec<(String, String, usize, bool)> {
    instructions
        .iter()
        .filter_map(|op| {
            op.call_parts().map(|(caller, callee, argc, has_spread)| {
                (caller.to_string(), callee.to_string(), argc, has_spread)
            })
        })
        .collect()
}

/// Opcode sequence of a stream.
fn opcode_sequence(instructions: &[CoreOp]) -> Vec<&'static str> {
    instructions
        .iter()
        .map(crate::ir::opcodes::opcode_name)
        .collect()
}

// ── Producer semantics ───────────────────────────────────────────────

#[test]
fn producer_records_zero_arguments_without_argument_captures() {
    let mut producer = CallProducer::new();
    // Only the argument-less query pattern bound this invocation (`Foo()`).
    producer.record_callee(0, 10, 13, "Foo", Some("M1"));

    assert_eq!(
        producer.settle(Some("M1")),
        vec![exact("M1", "Foo", 0)],
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
        vec![exact("M1", "Foo", 2)],
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
        vec![exact("M1", "Foo", 1), exact("M1", "Foo", 2)],
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
        vec![exact("M1", "Foo", 0)],
        "settling one owner must not emit another owner's facts"
    );
    assert_eq!(producer.pending(), 1);
    assert_eq!(producer.settle(Some("M2")), vec![exact("M2", "Bar", 0)]);
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
        let Some((caller, callee, argc, has_spread)) = op.call_parts() else {
            continue;
        };
        let tuple = op_to_tuple(&op);
        assert_eq!(tuple[0], "CALL");
        assert_eq!(tuple[1], caller);
        assert_eq!(tuple[2], callee);
        assert_eq!(tuple[3], argc.to_string());
        assert_eq!(
            tuple.len(),
            4,
            "an exact call keeps the established 4-component shape"
        );
        assert!(!has_spread);
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

    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);
    assert_eq!(
        opcode_sequence(&decoded.instructions),
        opcode_sequence(&ir.instructions),
        "no instruction may be lost, added, or reordered"
    );
    assert_eq!(
        call_facts_of(&decoded.instructions),
        call_facts_of(&ir.instructions),
        "binary wire must preserve caller, callee, and argc"
    );
    assert_eq!(
        call_facts_of(&decoded.instructions).len(),
        3,
        "every call fact must survive the wire"
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
    let argc_one = op_to_tuple(&exact("M1", "Foo", 1));
    let argc_two = op_to_tuple(&exact("M1", "Foo", 2));
    assert_ne!(
        primary_key_from_tuple(&argc_one),
        primary_key_from_tuple(&argc_two),
        "two calls differing only in arity must be distinct delta instructions"
    );
}

#[test]
fn delta_identity_includes_the_spread_qualifier() {
    let exact_tuple = op_to_tuple(&exact("M1", "Foo", 1));
    let spread_tuple = op_to_tuple(&spread("M1", "Foo", 1));
    assert_eq!(primary_key_from_tuple(&exact_tuple), "CALL:M1:Foo:1");
    assert_eq!(
        primary_key_from_tuple(&spread_tuple),
        "CALL:M1:Foo:1:spread"
    );
    assert_ne!(
        primary_key_from_tuple(&exact_tuple),
        primary_key_from_tuple(&spread_tuple),
        "two calls with the same written count must not collapse when only one \
         of them expands a written argument"
    );
}

// ── Spread qualifier: producer and transport shapes ──────────────────

/// A document with exactly one call fact, in either shape.
fn single_call_document(has_spread: bool) -> CompiledIR {
    CompiledIR {
        file_id: "Example.ts".to_string(),
        instructions: vec![CoreOp::Call(
            "M1".to_string(),
            "Foo".to_string(),
            1,
            has_spread,
        )],
        version: 1,
    }
}

#[test]
fn producer_records_a_spread_argument_once_and_qualifies_the_fact() {
    // `foo(...args)`: the argument-less pattern enumerates the invocation and
    // the spread pattern binds one written argument that expands.
    let mut producer = CallProducer::new();
    producer.record_callee(0, 4, 7, "Foo", Some("M1"));
    producer.record_callee(1, 4, 7, "Foo", Some("M1"));
    producer.record_spread(1);
    assert_eq!(
        producer.settle(Some("M1")),
        vec![spread("M1", "Foo", 1)],
        "a spread argument is ONE written argument and qualifies the fact"
    );

    // `foo(a, ...rest)`: two written arguments, one of which expands.
    let mut producer = CallProducer::new();
    producer.record_callee(0, 4, 7, "Foo", Some("M1"));
    producer.record_callee(1, 4, 7, "Foo", Some("M1"));
    producer.record_argument(1);
    producer.record_callee(2, 4, 7, "Foo", Some("M1"));
    producer.record_spread(2);
    assert_eq!(producer.settle(Some("M1")), vec![spread("M1", "Foo", 2)]);
}

#[test]
fn spread_document_keeps_two_facts_with_one_written_argument() {
    assert_eq!(
        call_facts_of(&spread_document().instructions),
        vec![
            ("M1".to_string(), "Save".to_string(), 1, false),
            ("M1".to_string(), "Save".to_string(), 1, true),
        ],
        "the same written count with different expansion evidence is two facts"
    );
}

#[test]
fn named_wire_dual_shape_carries_the_spread_qualifier() {
    let exact_tuple = op_to_tuple(&exact("M1", "Foo", 1));
    let spread_tuple = op_to_tuple(&spread("M1", "Foo", 1));
    assert_eq!(exact_tuple.len(), 4, "exact shape is the established one");
    assert_eq!(spread_tuple.len(), 5);
    assert_eq!(spread_tuple[4], "spread");
    assert_eq!(tuple_to_op(&exact_tuple), Some(exact("M1", "Foo", 1)));
    assert_eq!(tuple_to_op(&spread_tuple), Some(spread("M1", "Foo", 1)));

    // Strictness: an unrecognized qualifier, an extra component, and a
    // non-numeric count are all malformed rather than silently exact.
    let mut unknown_qualifier = spread_tuple.clone();
    unknown_qualifier[4] = "expanded".to_string();
    assert_eq!(tuple_to_op(&unknown_qualifier), None);
    let mut too_long = spread_tuple.clone();
    too_long.push("extra".to_string());
    assert_eq!(tuple_to_op(&too_long), None);
    let mut bad_count = exact_tuple.clone();
    bad_count[3] = "many".to_string();
    assert_eq!(tuple_to_op(&bad_count), None);
}

#[test]
fn binary_wire_carries_the_spread_qualifier_in_its_own_opcode() {
    // The qualifier is carried by the OPCODE, not an operand: the decoder
    // reads the same [caller, callee, argc] layout for either opcode, so an
    // exact call keeps its established OP_CALL encoding while a qualified call
    // is a distinct opcode that a predating reader rejects instead of
    // decoding as an exact one.
    //
    // The two streams are deliberately NOT byte-length identical, and that is
    // not a defect: the binary wire shares its string table with the
    // string-table transport (`StringTable::from_instructions` interns every
    // component of the canonical tuple), so the qualifier occupies one extra
    // table entry. No operand in the binary layout references that entry —
    // which is exactly what this test pins.
    let exact_bytes = binary_encode(&single_call_document(false));
    let spread_bytes = binary_encode(&single_call_document(true));
    assert_ne!(
        exact_bytes, spread_bytes,
        "a spread call must never encode to the exact call's bytes"
    );

    // Both shapes round-trip, and the qualifier survives the decode.
    assert_eq!(
        call_facts_of(
            &binary_decode(&spread_bytes)
                .expect("spread bytes must decode")
                .instructions
        ),
        vec![("M1".to_string(), "Foo".to_string(), 1, true)]
    );
    assert_eq!(
        call_facts_of(
            &binary_decode(&exact_bytes)
                .expect("exact bytes must decode")
                .instructions
        ),
        vec![("M1".to_string(), "Foo".to_string(), 1, false)]
    );
}

#[test]
fn hierarchical_wire_round_trips_the_spread_qualifier() {
    let ir = spread_document();
    let hir = ir_to_hierarchical(&ir);
    assert_eq!(
        hir.calls
            .iter()
            .map(|call| call.has_spread)
            .collect::<Vec<_>>(),
        vec![false, true],
        "the flat call table must carry the qualifier"
    );
    assert_eq!(
        hierarchical_to_ir(&hir),
        ir.instructions,
        "hierarchical round-trip must restore the identical stream"
    );

    // An exact call keeps the pre-qualifier serialized shape; a spread call
    // publishes the qualifier so its written count cannot be read as an arity.
    let exact_json = serde_json::to_value(&hir.calls[0]).expect("exact call serializes");
    assert!(
        exact_json.get("s").is_none(),
        "an exact call must omit the qualifier: {exact_json}"
    );
    let spread_json = serde_json::to_value(&hir.calls[1]).expect("spread call serializes");
    assert_eq!(spread_json.get("s"), Some(&serde_json::json!(true)));
}

#[test]
fn positional_and_string_table_wire_round_trip_both_call_shapes() {
    use crate::ir::positional::{PositionalConfig, decode_op, encode_op};
    use crate::ir::string_table::{
        StringTable, decode_op as table_decode, encode_op as table_encode,
    };

    for op in [exact("M1", "Foo", 1), spread("M1", "Foo", 1)] {
        // Positional transport: the opcode name is unchanged, so the variadic
        // arity table plus the strict tuple decoder carry the qualifier.
        let tagged = encode_op(&op, PositionalConfig::tagged());
        assert_eq!(tagged[0], "CALL");
        assert_eq!(
            decode_op("CALL", &tagged[1..]),
            Some(op.clone()),
            "positional transport must round-trip the whole fact"
        );

        // String-table transport is tuple-generic: every component (including
        // the qualifier) is interned, so no call-specific change is needed.
        let table = StringTable::from_instructions(std::slice::from_ref(&op));
        let indices = table_encode(&op, &table);
        assert_eq!(
            table_decode(&indices, &table),
            Some(op.clone()),
            "string-table transport must round-trip the whole fact"
        );
    }
}
