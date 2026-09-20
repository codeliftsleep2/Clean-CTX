use super::*;

// ── 7. Edge Case: All R-43a Variants Only ───────────────────────

#[test]
fn round_trip_only_execution_semantics() {
    let ir = CompiledIR {
        file_id: "exec.ts".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DataFlow("M1".into(), "reads".into(), "repo".into()),
            CoreOp::DataFlow("M1".into(), "writes".into(), "cache".into()),
            CoreOp::ControlFlow("M1".into(), "if".into(), "cond".into()),
            CoreOp::ControlFlow("M1".into(), "loop".into(), "items".into()),
            CoreOp::ControlFlow("M1".into(), "match".into(), "val".into()),
            CoreOp::ControlFlow("M1".into(), "try".into(), "block".into()),
            CoreOp::ControlFlow("M1".into(), "await".into(), "promise".into()),
            CoreOp::ControlFlow("M1".into(), "return".into(), "result".into()),
            CoreOp::SideEffect("M1".into(), SideEffectKind::Pure),
            CoreOp::SideEffect("M1".into(), SideEffectKind::Io),
            CoreOp::SideEffect("M1".into(), SideEffectKind::Mutation),
            CoreOp::SideEffect("M1".into(), SideEffectKind::Async),
            CoreOp::SideEffect("M1".into(), SideEffectKind::Transaction),
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::Sync),
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::Async),
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::ThreadBound),
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::TransactionScope),
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::Realtime),
        ],
    };

    // Named wire
    let wire = ir_to_wire(&ir);
    let restored = wire_to_ir(&wire).expect("execution semantics only: named wire round-trip");
    assert_ir_eq(&ir, &restored);

    // Binary wire
    let bytes = encode(&ir);
    let restored_bin = decode(&bytes).expect("execution semantics only: binary wire round-trip");
    assert_eq!(restored_bin.instructions.len(), ir.instructions.len());
    for (i, (a, b)) in restored_bin
        .instructions
        .iter()
        .zip(ir.instructions.iter())
        .enumerate()
    {
        assert_eq!(a, b, "execution semantics binary mismatch at index {}", i);
    }
}

// ── 8. Delta Round-Trip with SemanticIntent ─────────────────────

#[test]
fn round_trip_delta_with_semantic_intent() {
    let delta = IRDelta {
        file: "test.ts".to_string(),
        from: 1,
        to: 2,
        ops: DeltaOps {
            adds: vec![vec![
                "DATAFLOW".into(),
                "M1".into(),
                "reads".into(),
                "repo".into(),
            ]],
            mods: vec![],
            dels: vec![],
        },
        intent: Some(crate::ir::delta::SemanticIntent::AddMethod {
            class: "C1".to_string(),
            method_name: "newMethod".to_string(),
        }),
    };

    // Serialize to JSON and back
    let json = serde_json::to_string(&delta).expect("serialize delta with intent");
    let restored: IRDelta = serde_json::from_str(&json).expect("deserialize delta with intent");
    assert_eq!(restored.file, delta.file);
    assert_eq!(restored.from, delta.from);
    assert_eq!(restored.to, delta.to);
    assert!(restored.intent.is_some());
    match restored.intent.unwrap() {
        crate::ir::delta::SemanticIntent::AddMethod { class, method_name } => {
            assert_eq!(class, "C1");
            assert_eq!(method_name, "newMethod");
        }
        other => panic!("expected AddMethod, got: {:?}", other),
    }
}

#[test]
fn round_trip_delta_without_intent_skips_field() {
    let delta = IRDelta {
        file: "test.ts".to_string(),
        from: 1,
        to: 2,
        ops: DeltaOps::default(),
        intent: None,
    };
    let json = serde_json::to_string(&delta).expect("serialize delta without intent");
    // The intent field should be absent (skip_serializing_if)
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(
        parsed.get("intent").is_none() || parsed["intent"].is_null(),
        "intent field should be absent when None"
    );
}

// ── 9. R-02: Type Alias IR Round-Trip Tests ─────────────────────
//
// Verifies that CoreOp::TypeAlias ops emitted by apply_type_aliases_to_ir
// survive wire-format round-trips, and that the alias mapping is
// preserved.

use crate::ir::type_aliases::apply_type_aliases_to_ir;
use std::collections::BTreeMap;

fn make_aliases(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn round_trip_type_alias_op_named_wire() {
    let original = CoreOp::TypeAlias("$uid".into(), "User".into());
    let tuple = op_to_tuple(&original);
    assert_eq!(tuple, vec!["TYPE", "$uid", "User"]);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_ir_with_type_aliases_named_wire() {
    // Build IR with type-bearing ops, apply aliases, then round-trip
    // through named wire format.
    let mut instructions = vec![
        CoreOp::DefClass("C1".into(), "UserService".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "getUser".into()),
        CoreOp::Param("M1".into(), "P1".into(), "User".into(), "id".into()),
        CoreOp::Return("M1".into(), "Promise<User>".into()),
        CoreOp::DefField("C1".into(), "F1".into(), "user".into()),
        CoreOp::FieldType("F1".into(), "User".into()),
    ];
    let aliases = make_aliases(&[("User", "$uid")]);
    apply_type_aliases_to_ir(&mut instructions, &aliases);

    // Verify substitution occurred
    assert!(instructions.iter().any(|op| matches!(op,
        CoreOp::Param(_, _, t, _) if t == "$uid")));
    assert!(instructions.iter().any(|op| matches!(op,
        CoreOp::FieldType(_, t) if t == "$uid")));
    // Verify TypeAlias op was appended
    assert!(instructions.iter().any(|op| matches!(op,
        CoreOp::TypeAlias(a, o) if a == "$uid" && o == "User")));

    // Round-trip through named wire
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        version: 1,
        instructions,
    };
    let wire = ir_to_wire(&ir);
    let restored = wire_to_ir(&wire).expect("named wire round-trip with type aliases");
    assert_ir_eq(&ir, &restored);
}

#[test]
fn round_trip_ir_with_type_aliases_binary_wire() {
    let mut instructions = vec![
        CoreOp::FieldType("F1".into(), "User".into()),
        CoreOp::Return("M1".into(), "User".into()),
    ];
    let aliases = make_aliases(&[("User", "$uid")]);
    apply_type_aliases_to_ir(&mut instructions, &aliases);

    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        version: 1,
        instructions,
    };
    let bytes = encode(&ir);
    let restored = decode(&bytes).expect("binary wire round-trip with type aliases");
    // Binary wire format uses empty strings for TypeAlias ID fields (like
    // DefClass, Import, etc.), so we verify the TYPE opcode is present
    // rather than the full alias/original content.
    assert!(
        restored
            .instructions
            .iter()
            .any(|op| matches!(op, CoreOp::TypeAlias(..))),
        "TypeAlias op should survive binary wire round-trip"
    );
    // Verify the substituted type values survived (FieldType/Return are
    // data-preserving in binary wire).
    assert!(
        restored.instructions.iter().any(|op| matches!(op,
        CoreOp::FieldType(_, t) if t == "$uid")),
        "FieldType with $uid should survive"
    );
    assert!(
        restored.instructions.iter().any(|op| matches!(op,
        CoreOp::Return(_, t) if t == "$uid")),
        "Return with $uid should survive"
    );
}

#[test]
fn round_trip_ir_with_multiple_type_aliases() {
    let mut instructions = vec![
        CoreOp::FieldType("F1".into(), "User".into()),
        CoreOp::FieldType("F2".into(), "JsonObject".into()),
        CoreOp::Return("M1".into(), "Promise<User>".into()),
    ];
    let aliases = make_aliases(&[("User", "$uid"), ("JsonObject", "$jo")]);
    apply_type_aliases_to_ir(&mut instructions, &aliases);

    // Both TypeAlias ops should be present
    let ta_count = instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::TypeAlias(..)))
        .count();
    assert_eq!(ta_count, 2, "expected 2 TypeAlias ops");

    // Round-trip through named wire
    let ir = CompiledIR {
        file_id: "multi.ts".to_string(),
        version: 1,
        instructions,
    };
    let wire = ir_to_wire(&ir);
    let restored = wire_to_ir(&wire).expect("named wire round-trip with multiple aliases");
    assert_ir_eq(&ir, &restored);
}
