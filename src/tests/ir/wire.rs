use crate::ir::compiler::CompiledIR;
use crate::ir::opcodes::CoreOp;
use crate::ir::wire::{op_to_tuple, tuple_to_op, ir_to_wire, wire_to_ir, DecodeError};

#[test]
fn op_to_tuple_def_class() {
    let tuple = op_to_tuple(&CoreOp::DefClass("C1".into(), "Foo".into()));
    assert_eq!(tuple, vec!["DEF_C", "C1", "Foo"]);
}

#[test]
fn op_to_tuple_def_method() {
    let tuple = op_to_tuple(&CoreOp::DefMethod("C1".into(), "M1".into(), "bar".into()));
    assert_eq!(tuple, vec!["DEF_M", "C1", "M1", "bar"]);
}

#[test]
fn op_to_tuple_def_field() {
    let tuple = op_to_tuple(&CoreOp::DefField("C1".into(), "F1".into(), "x".into()));
    assert_eq!(tuple, vec!["DEF_F", "C1", "F1", "x"]);
}

#[test]
fn op_to_tuple_def_interface() {
    let tuple = op_to_tuple(&CoreOp::DefInterface("I1".into(), "IFoo".into()));
    assert_eq!(tuple, vec!["DEF_I", "I1", "IFoo"]);
}

#[test]
fn op_to_tuple_param() {
    let tuple = op_to_tuple(&CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "name".into()));
    assert_eq!(tuple, vec!["SIG", "M1", "P1", "$s", "name"]);
}

#[test]
fn op_to_tuple_return() {
    let tuple = op_to_tuple(&CoreOp::Return("M1".into(), "$b".into()));
    assert_eq!(tuple, vec!["RET", "M1", "$b"]);
}

#[test]
fn op_to_tuple_field_type() {
    let tuple = op_to_tuple(&CoreOp::FieldType("F1".into(), "$n".into()));
    assert_eq!(tuple, vec!["FIELD_T", "F1", "$n"]);
}

#[test]
fn op_to_tuple_flags() {
    let tuple = op_to_tuple(&CoreOp::Flags("M1".into(), vec!["IF".into(), "LOOP".into()]));
    assert_eq!(tuple, vec!["FLAGS", "M1", "IF", "LOOP"]);
}

#[test]
fn op_to_tuple_class_flags() {
    let tuple = op_to_tuple(&CoreOp::ClassFlags("C1".into(), vec!["EXPORT".into()]));
    assert_eq!(tuple, vec!["FLAGS_C", "C1", "EXPORT"]);
}

#[test]
fn op_to_tuple_extends() {
    let tuple = op_to_tuple(&CoreOp::Extends("C1".into(), "C2".into()));
    assert_eq!(tuple, vec!["EXT", "C1", "C2"]);
}

#[test]
fn op_to_tuple_implements() {
    let tuple = op_to_tuple(&CoreOp::Implements("C1".into(), "I1".into()));
    assert_eq!(tuple, vec!["IMPL", "C1", "I1"]);
}

#[test]
fn op_to_tuple_injects() {
    let tuple = op_to_tuple(&CoreOp::Injects("C1".into(), vec!["S1".into(), "S2".into()]));
    assert_eq!(tuple, vec!["INJECTS", "C1", "S1", "S2"]);
}

#[test]
fn op_to_tuple_import() {
    let tuple = op_to_tuple(&CoreOp::Import("IM1".into(), "rxjs".into(), "map".into()));
    assert_eq!(tuple, vec!["IMP", "IM1", "rxjs", "map"]);
}

#[test]
fn op_to_tuple_type_alias() {
    let tuple = op_to_tuple(&CoreOp::TypeAlias("T1".into(), "UserId".into()));
    assert_eq!(tuple, vec!["TYPE", "T1", "UserId"]);
}

// Round-trip tests: CoreOp -> tuple -> CoreOp

#[test]
fn round_trip_def_class() {
    let original = CoreOp::DefClass("C1".into(), "Foo".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_def_method() {
    let original = CoreOp::DefMethod("C1".into(), "M1".into(), "bar".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_def_field() {
    let original = CoreOp::DefField("C1".into(), "F1".into(), "x".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_def_interface() {
    let original = CoreOp::DefInterface("I1".into(), "IFoo".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_param() {
    let original = CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "name".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_return() {
    let original = CoreOp::Return("M1".into(), "$b".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_field_type() {
    let original = CoreOp::FieldType("F1".into(), "$n".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_flags() {
    let original = CoreOp::Flags("M1".into(), vec!["IF".into(), "LOOP".into()]);
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_class_flags() {
    let original = CoreOp::ClassFlags("C1".into(), vec!["EXPORT".into(), "ABSTRACT".into()]);
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_extends() {
    let original = CoreOp::Extends("C1".into(), "C2".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_implements() {
    let original = CoreOp::Implements("C1".into(), "I1".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_injects() {
    let original = CoreOp::Injects("C1".into(), vec!["S1".into(), "S2".into()]);
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_import() {
    let original = CoreOp::Import("IM1".into(), "rxjs".into(), "map".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn round_trip_type_alias() {
    let original = CoreOp::TypeAlias("T1".into(), "UserId".into());
    let tuple = op_to_tuple(&original);
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(original, restored);
}

// tuple_to_op edge cases

#[test]
fn tuple_to_op_empty_returns_none() {
    assert!(tuple_to_op(&[]).is_none());
}

#[test]
fn tuple_to_op_unknown_opcode_returns_none() {
    assert!(tuple_to_op(&["UNKNOWN".into(), "x".into()]).is_none());
}

#[test]
fn tuple_to_op_too_short_returns_none() {
    assert!(tuple_to_op(&["DEF_C".into(), "C1".into()]).is_none());
    assert!(tuple_to_op(&["DEF_M".into(), "C1".into(), "M1".into()]).is_none());
}

// Wire format (JSON) tests

fn sample_compiled_ir() -> CompiledIR {
    CompiledIR {
        file_id: "a1".into(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "SampleService".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "processData".into()),
            CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "input".into()),
            CoreOp::Return("M1".into(), "$b".into()),
            CoreOp::Flags("M1".into(), vec!["IF".into()]),
        ],
        version: 1,
    }
}

#[test]
fn ir_to_wire_json_structure() {
    let ir = sample_compiled_ir();
    let wire = ir_to_wire(&ir);
    assert_eq!(wire["file"], "a1");
    assert_eq!(wire["v"], 1);
    assert!(wire["ir"].is_array());
    assert_eq!(wire["ir"].as_array().unwrap().len(), 5);
}

#[test]
fn ir_to_wire_tuple_format() {
    let ir = sample_compiled_ir();
    let wire = ir_to_wire(&ir);
    let tuples = wire["ir"].as_array().unwrap();
    assert_eq!(tuples[0][0], "DEF_C");
    assert_eq!(tuples[0][1], "C1");
    assert_eq!(tuples[0][2], "SampleService");
    assert_eq!(tuples[1][0], "DEF_M");
    assert_eq!(tuples[1][3], "processData");
    assert_eq!(tuples[4][0], "FLAGS");
    assert_eq!(tuples[4][2], "IF");
}

#[test]
fn wire_round_trip() {
    let original = sample_compiled_ir();
    let wire = ir_to_wire(&original);
    let restored = wire_to_ir(&wire).expect("should deserialize");
    assert_eq!(restored.file_id, original.file_id);
    assert_eq!(restored.version, original.version);
    assert_eq!(restored.instructions.len(), original.instructions.len());
    for (a, b) in original.instructions.iter().zip(restored.instructions.iter()) {
        assert_eq!(a, b);
    }
}

#[test]
fn wire_to_ir_empty_instructions() {
    let wire = serde_json::json!({
        "file": "empty",
        "v": 1,
        "ir": []
    });
    let restored = wire_to_ir(&wire).expect("should deserialize");
    assert_eq!(restored.file_id, "empty");
    assert_eq!(restored.instructions.len(), 0);
}

#[test]
fn wire_to_ir_missing_file_returns_err() {
    let wire = serde_json::json!({
        "v": 1,
        "ir": []
    });
    let err = wire_to_ir(&wire).expect_err("should return error for missing file");
    assert!(matches!(err, DecodeError::MissingField(_)));
}

#[test]
fn wire_to_ir_missing_version_returns_err() {
    let wire = serde_json::json!({
        "file": "f",
        "ir": []
    });
    let err = wire_to_ir(&wire).expect_err("should return error for missing version");
    assert!(matches!(err, DecodeError::MissingField(_)));
}

#[test]
fn wire_to_ir_unknown_opcode_returns_err() {
    let wire = serde_json::json!({
        "file": "f",
        "v": 1,
        "ir": [
            ["DEF_C", "C1", "Foo"],
            ["UNKNOWN_OPCODE", "x"],
            ["DEF_M", "C1", "M1", "bar"]
        ]
    });
    // Unknown opcode should return an error (F-19: no silent swallowing)
    let err = wire_to_ir(&wire).expect_err("should return error for unknown opcode");
    assert!(matches!(err, DecodeError::UnknownOpcode(_)));
}

#[test]
fn wire_to_ir_malformed_tuple_returns_err() {
    // Tuple with a known opcode but insufficient operands is silently dropped
    // by tuple_to_op returning None. The decoder then reports it as an
    // unknown opcode (because tuple_to_op cannot tell us "too short" vs
    // "unknown").
    let wire = serde_json::json!({
        "file": "f",
        "v": 1,
        "ir": [
            ["DEF_C"] // too short for DEF_C (needs 3 elements)
        ]
    });
    let err = wire_to_ir(&wire).expect_err("should return error for malformed tuple");
    // Either MalformedTuple (if the decoder can tell it's a known opcode)
    // or UnknownOpcode (if the decoder treats the short tuple as unknown).
    assert!(
        matches!(err, DecodeError::MalformedTuple(_) | DecodeError::UnknownOpcode(_)),
        "expected MalformedTuple or UnknownOpcode, got: {:?}", err
    );
}

// ─── F-FINAL-07: Round-trip stability tests ──────────────────────
//
// The audit flagged that the IR wire format has *individual* encode
// and decode tests but no end-to-end "ir → wire → ir" round-trip
// assertion. A round-trip test that does NOT use `verify_round_trip`
// (which is tagged with its own caveat) is the only way to catch
// drift between the encoder, the JSON envelope, and the decoder.
//
// The tests below cover:
//   1. A small hand-built CompiledIR (5 ops).
//   2. An empty CompiledIR (boundary).
//   3. A mixed-type CompiledIR with every CoreOp variant represented.
//   4. A version-stability test (re-encode and check the byte count
//      is deterministic — guards against HashMap iteration leakage).

/// F-FINAL-07: Round-trip a small hand-built CompiledIR through
/// `ir_to_wire` and back. Asserts the result is byte-identical to
/// the input.
#[test]
fn round_trip_small_ir_is_stable() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        version: 7,
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Foo".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "bar".into()),
            CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "name".into()),
            CoreOp::Return("M1".into(), "$v".into()),
            CoreOp::Flags("M1".into(), vec!["IF".into()]),
        ],
    };
    let wire = ir_to_wire(&ir);
    let decoded = wire_to_ir(&wire).expect("round-trip should succeed");
    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions.len(), ir.instructions.len());
    for (a, b) in decoded.instructions.iter().zip(ir.instructions.iter()) {
        assert_eq!(a, b, "instruction round-trip mismatch");
    }
}

/// F-FINAL-07: An empty CompiledIR (no instructions) must round-trip
/// without error. This catches decoder-side "empty tuple" panic paths.
#[test]
fn round_trip_empty_ir_is_stable() {
    let ir = CompiledIR {
        file_id: "empty.ts".to_string(),
        version: 0,
        instructions: vec![],
    };
    let wire = ir_to_wire(&ir);
    let decoded = wire_to_ir(&wire).expect("empty IR should round-trip");
    assert_eq!(decoded.file_id, "empty.ts");
    assert_eq!(decoded.version, 0);
    assert!(decoded.instructions.is_empty());
}

/// F-FINAL-07: A mixed-type IR with every CoreOp variant represented
/// must round-trip correctly. This is the strongest stability check
/// — if any single variant fails to encode/decode symmetrically,
/// the assertion will fail.
#[test]
fn round_trip_all_variants_is_stable() {
    let ir = CompiledIR {
        file_id: "all.ts".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Foo".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "ctor".into()),
            CoreOp::DefField("C1".into(), "F1".into(), "x".into()),
            CoreOp::DefInterface("I1".into(), "IFoo".into()),
            CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "name".into()),
            CoreOp::Return("M1".into(), "$v".into()),
            CoreOp::FieldType("F1".into(), "$n".into()),
            CoreOp::Flags("M1".into(), vec!["IF".into(), "LOOP".into()]),
            CoreOp::ClassFlags("C1".into(), vec!["EXPORT".into()]),
            CoreOp::Extends("C1".into(), "C2".into()),
            CoreOp::Implements("C1".into(), "I1".into()),
            CoreOp::Injects("C1".into(), vec!["S1".into(), "S2".into()]),
            CoreOp::Import("IM1".into(), "fs".into(), "readFile".into()),
            CoreOp::TypeAlias("T1".into(), "string".into()),
            CoreOp::Pattern("CTOR".into(), vec!["C1".into(), "M1".into(), "S1".into()]),
        ],
    };
    let wire = ir_to_wire(&ir);
    let decoded = wire_to_ir(&wire).expect("all-variants round-trip should succeed");
    assert_eq!(decoded.instructions.len(), ir.instructions.len());
    for (i, (a, b)) in decoded.instructions.iter().zip(ir.instructions.iter()).enumerate() {
        assert_eq!(a, b, "mismatch at instruction {}", i);
    }
}

/// F-FINAL-07: Re-encoding the same CompiledIR must produce
/// byte-identical JSON. This guards against HashMap iteration
/// ordering leaking into the wire format.
#[test]
fn ir_to_wire_is_deterministic() {
    let ir = CompiledIR {
        file_id: "determinism.ts".to_string(),
        version: 3,
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Foo".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "bar".into()),
        ],
    };
    let wire1 = serde_json::to_string(&ir_to_wire(&ir)).unwrap();
    let wire2 = serde_json::to_string(&ir_to_wire(&ir)).unwrap();
    assert_eq!(wire1, wire2, "ir_to_wire must be deterministic");
}
