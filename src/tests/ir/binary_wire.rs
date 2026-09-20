// src/tests/ir/binary_wire.rs
//
// Phase II: Ultra-Compact IR — Binary Wire Format tests (Idea #1).
//
// Tests cover: varint encoding, string encoding, full round-trip for
// all opcode types, error handling, base64 JSON wrapper, and
// integration with wire_to_ir_detect.

use crate::ir::binary_wire::{
    BinaryDecodeError, binary_wire_json_to_ir, decode, encode, estimate_savings,
    ir_to_binary_wire_json, is_binary_wire,
};
use crate::ir::compiler::CompiledIR;
use crate::ir::opcodes::CoreOp;
use serde_json::json;

// ── Helpers ────────────────────────────────────────────────────────

/// Create a simple single-class IR.
fn make_simple_ir() -> CompiledIR {
    CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "SampleService".to_string()),
            CoreOp::DefMethod(
                "C1".to_string(),
                "M1".to_string(),
                "processData".to_string(),
            ),
            CoreOp::Param(
                "M1".to_string(),
                "P1".to_string(),
                "$s".to_string(),
                "payload".to_string(),
            ),
            CoreOp::Return("M1".to_string(), "$b".to_string()),
            CoreOp::Flags("M1".to_string(), vec!["IF".to_string()]),
        ],
    }
}

/// Create a multi-class IR covering all opcode variants.
fn make_full_ir() -> CompiledIR {
    CompiledIR {
        file_id: "α2".to_string(),
        version: 1,
        instructions: vec![
            // Class 1
            CoreOp::DefClass("C1".to_string(), "BaseService".to_string()),
            CoreOp::ClassFlags(
                "C1".to_string(),
                vec!["EXPORT".to_string(), "ABSTRACT".to_string()],
            ),
            CoreOp::DefField("C1".to_string(), "F1".to_string(), "items".to_string()),
            CoreOp::FieldType("F1".to_string(), "$s[]".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "doWork".to_string()),
            CoreOp::Param(
                "M1".to_string(),
                "P1".to_string(),
                "$n".to_string(),
                "count".to_string(),
            ),
            CoreOp::Return("M1".to_string(), "$v".to_string()),
            CoreOp::Flags("M1".to_string(), vec!["IF".to_string(), "LOOP".to_string()]),
            // Class 2
            CoreOp::DefClass("C2".to_string(), "DerivedService".to_string()),
            CoreOp::DefInterface("IF1".to_string(), "IComparable".to_string()),
            CoreOp::Extends("C2".to_string(), "C1".to_string()),
            CoreOp::Implements("C2".to_string(), "IF1".to_string()),
            CoreOp::Injects(
                "C2".to_string(),
                vec!["DEP1".to_string(), "DEP2".to_string()],
            ),
            CoreOp::DefMethod(
                "C2".to_string(),
                "M2".to_string(),
                "handleEvent".to_string(),
            ),
            CoreOp::Return("M2".to_string(), "$b".to_string()),
            CoreOp::Flags("M2".to_string(), vec!["ASYNC".to_string()]),
            // Imports, Types, Patterns
            CoreOp::Import(
                "IM1".to_string(),
                "rxjs".to_string(),
                "Observable".to_string(),
            ),
            CoreOp::TypeAlias("T1".to_string(), "string[]".to_string()),
            CoreOp::Pattern("CTOR".to_string(), vec!["C1".to_string(), "M1".to_string()]),
        ],
    }
}

// ── Varint Tests ───────────────────────────────────────────────────

#[test]
fn test_varint_small_values() {
    // Varints are private, but we test them indirectly via encode/decode.
    // Small values (0-127) should encode as single bytes.
    let ir = make_simple_ir();
    let bytes = encode(&ir);
    // Magic(2) + version(1) = 3 bytes header
    assert!(bytes.len() > 3, "binary output should have header");
    assert_eq!(bytes[0], 0xCC, "magic byte 1");
    assert_eq!(bytes[1], 0x02, "magic byte 2");
    assert_eq!(bytes[2], 0x04, "corrected physical version byte");
}

// ── Round-trip Tests ───────────────────────────────────────────────

#[test]
fn test_round_trip_simple_ir() {
    let ir = make_simple_ir();
    let bytes = encode(&ir);
    let decoded = decode(&bytes).unwrap();
    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);
}

#[test]
fn test_round_trip_full_ir() {
    let ir = make_full_ir();
    let bytes = encode(&ir);
    let decoded = decode(&bytes).unwrap();
    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);
}

#[test]
fn test_round_trip_empty_ir() {
    let ir = CompiledIR {
        file_id: "test".to_string(),
        version: 1,
        instructions: vec![],
    };
    let bytes = encode(&ir);
    let decoded = decode(&bytes).unwrap();
    assert_eq!(decoded, ir);
}

// ── Detection Tests ────────────────────────────────────────────────

#[test]
fn test_is_binary_wire() {
    let ir = make_simple_ir();
    let bytes = encode(&ir);
    assert!(is_binary_wire(&bytes), "should detect magic bytes");
    assert!(
        !is_binary_wire(&[0x00, 0x00, 0x00]),
        "should reject non-magic"
    );
    assert!(!is_binary_wire(&[]), "should reject empty");
}

#[test]
fn test_is_binary_wire_short() {
    assert!(!is_binary_wire(&[0xCC]), "single byte should not match");
}

// ── Error Handling Tests ───────────────────────────────────────────

#[test]
fn test_decode_invalid_magic() {
    let data = vec![0x00, 0x00, 0x01];
    let result = decode(&data);
    assert!(matches!(result, Err(BinaryDecodeError::InvalidMagic)));
}

#[test]
fn test_decode_unsupported_version() {
    let data = vec![0xCC, 0x02, 0xFF];
    let result = decode(&data);
    assert!(matches!(
        result,
        Err(BinaryDecodeError::UnsupportedVersion(0xFF))
    ));
}

#[test]
fn test_decode_truncated_header() {
    let data = vec![0xCC, 0x02]; // missing version byte
    let result = decode(&data);
    assert!(matches!(result, Err(BinaryDecodeError::TruncatedData(_))));
}

#[test]
fn test_decode_empty() {
    let result = decode(&[]);
    assert!(matches!(result, Err(BinaryDecodeError::TruncatedData(_))));
}

#[test]
fn test_decode_truncated_string_table() {
    // Valid header but no string table
    let data = vec![0xCC, 0x02, 0x04];
    let result = decode(&data);
    assert!(matches!(result, Err(BinaryDecodeError::TruncatedData(_))));
}

// ── Savings Estimation Tests ──────────────────────────────────────

#[test]
fn test_estimate_savings_positive() {
    let ir = make_simple_ir();
    let (json_chars, binary_bytes) = estimate_savings(&ir);
    assert!(json_chars > 0, "JSON should have content");
    assert!(binary_bytes > 0, "binary should have content");
    // Binary should be smaller than JSON for any non-trivial IR
    assert!(
        binary_bytes < json_chars,
        "binary ({}) should be smaller than JSON ({})",
        binary_bytes,
        json_chars
    );
}

#[test]
fn test_estimate_savings_full_ir() {
    let ir = make_full_ir();
    let (json_chars, binary_bytes) = estimate_savings(&ir);
    assert!(json_chars > 0);
    assert!(binary_bytes > 0);
    assert!(
        binary_bytes < json_chars,
        "binary ({}) should be smaller than JSON ({}) for full IR",
        binary_bytes,
        json_chars
    );
}

// ── Base64 JSON Wrapper Tests ─────────────────────────────────────

#[test]
fn test_binary_wire_json_round_trip() {
    let ir = make_full_ir();
    let json_value = ir_to_binary_wire_json(&ir);

    // Verify JSON structure
    assert_eq!(
        json_value.get("encoding").and_then(|v| v.as_str()),
        Some("binary")
    );
    assert!(
        json_value.get("data").and_then(|v| v.as_str()).is_some(),
        "should contain base64 data"
    );
    assert_eq!(json_value.get("file").and_then(|v| v.as_str()), Some("α2"));
    assert_eq!(json_value.get("v").and_then(|v| v.as_u64()), Some(1));

    // Round-trip
    let decoded = binary_wire_json_to_ir(&json_value).unwrap();
    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);
}

#[test]
fn test_binary_wire_json_decode_nonexistent() {
    let value = json!({"encoding": "binary"});
    let result = binary_wire_json_to_ir(&value);
    assert!(result.is_none(), "missing data field should return None");
}

#[test]
fn test_binary_wire_json_decode_invalid_base64() {
    let value = json!({
        "encoding": "binary",
        "data": "!!!not-valid-base64!!!"
    });
    let result = binary_wire_json_to_ir(&value);
    assert!(result.is_none(), "invalid base64 should return None");
}

// ── Wire Detection Integration Tests ──────────────────────────────

#[test]
fn test_wire_to_ir_detect_binary() {
    let ir = make_simple_ir();
    let json_value = ir_to_binary_wire_json(&ir);
    let decoded = crate::ir::wire::wire_to_ir_detect(&json_value).unwrap();
    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);
}

#[test]
fn test_wire_to_ir_detect_binary_via_serde() {
    // Construct the JSON directly to ensure encoding detection works
    // when the JSON comes from a wire source
    let ir = make_full_ir();
    let json_value = ir_to_binary_wire_json(&ir);
    let json_str = serde_json::to_string(&json_value).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let decoded = crate::ir::wire::wire_to_ir_detect(&parsed).unwrap();
    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);
}

// ── Encoding Stability Tests ──────────────────────────────────────

#[test]
fn test_encoding_deterministic() {
    let ir = make_full_ir();
    let bytes1 = encode(&ir);
    let bytes2 = encode(&ir);
    assert_eq!(bytes1, bytes2, "encoding should be deterministic");
}

#[test]
fn test_binary_output_smaller_than_json() {
    // Verify the binary format is significantly more compact than JSON
    let ir = make_full_ir();
    let bytes = encode(&ir);

    // Serialize to named JSON
    let named_json = crate::ir::wire::ir_to_wire(&ir);
    let json_str = serde_json::to_string(&named_json).unwrap();

    // Binary should be less than 70% of JSON size for non-trivial IRs
    let ratio = bytes.len() as f64 / json_str.len() as f64;
    assert!(
        ratio < 0.7,
        "binary size ratio {:.2} should be < 0.7 ({} binary vs {} JSON)",
        ratio,
        bytes.len(),
        json_str.len()
    );
}

// ── Large IR Test ─────────────────────────────────────────────────

#[test]
fn test_round_trip_large_ir() {
    let mut instructions = Vec::new();
    for i in 0..100 {
        let cid = format!("C{}", i);
        instructions.push(CoreOp::DefClass(cid.clone(), format!("Class{}", i)));
        instructions.push(CoreOp::DefMethod(
            cid,
            format!("M{}", i),
            format!("method{}", i),
        ));
        instructions.push(CoreOp::Return(format!("M{}", i), "$v".to_string()));
    }
    let ir = CompiledIR {
        file_id: "large".to_string(),
        version: 1,
        instructions,
    };

    let bytes = encode(&ir);
    let decoded = decode(&bytes).unwrap();
    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);
}

// ── Zero-State Tests ──────────────────────────────────────────────

/// Regression (RED-CALL23): the decoder's forward-compatibility guard must bound
/// the **highest defined** opcode, not the last opcode that happened to exist
/// when the guard was written.
///
/// `OP_CALL` (20) is allocated after the edit-mode `OP_BODY` (19), so a guard
/// written as `op_idx > OP_BODY` reported a *defined* opcode as
/// `UnknownOpcode(20)` and made native call facts unrepresentable over the
/// binary wire — the encoder wrote them and no reader could read them. The
/// guard now bounds `OP_MAX`, so every opcode this build defines round-trips.
///
/// Physical `0x04` transports the complete canonical operation and container
/// identity while retaining the established call opcode assignments.
#[test]
fn binary_wire_round_trips_call_facts_and_every_defined_opcode() {
    let ir = CompiledIR {
        file_id: "Example.cs".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "Example".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "Process".to_string()),
            CoreOp::Call("M1".to_string(), "Save".to_string(), 1, false),
            CoreOp::Call("M1".to_string(), "OrderBy".to_string(), 2, false),
            CoreOp::Call("M1".to_string(), "Reset".to_string(), 0, false),
        ],
    };

    let decoded = decode(&encode(&ir)).expect("every defined opcode must decode");

    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);

    let opcodes = |instructions: &[CoreOp]| -> Vec<&'static str> {
        instructions
            .iter()
            .map(crate::ir::opcodes::opcode_name)
            .collect()
    };
    let facts = |instructions: &[CoreOp]| -> Vec<(String, String, usize, bool)> {
        instructions
            .iter()
            .filter_map(|op| {
                op.call_parts().map(|(caller, callee, argc, has_spread)| {
                    (caller.to_string(), callee.to_string(), argc, has_spread)
                })
            })
            .collect()
    };

    assert_eq!(
        opcodes(&decoded.instructions),
        opcodes(&ir.instructions),
        "no instruction may be lost, added, or reordered"
    );
    assert_eq!(
        facts(&decoded.instructions),
        facts(&ir.instructions),
        "caller, callee, explicit argument count, and spread qualifier must \
         survive the binary wire"
    );
}

#[test]
fn test_ir_with_only_variadic_ops() {
    let ir = CompiledIR {
        file_id: "test".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::Flags(
                "T1".to_string(),
                vec!["IF".to_string(), "LOOP".to_string(), "ASYNC".to_string()],
            ),
            CoreOp::ClassFlags("C1".to_string(), vec!["EXPORT".to_string()]),
            CoreOp::Injects(
                "C2".to_string(),
                vec!["A".to_string(), "B".to_string(), "C".to_string()],
            ),
            CoreOp::Pattern("CTOR".to_string(), vec!["C1".to_string(), "M1".to_string()]),
        ],
    };
    let bytes = encode(&ir);
    let decoded = decode(&bytes).unwrap();
    assert_eq!(decoded, ir);
}

#[test]
fn test_ir_with_only_fixed_ops() {
    let ir = CompiledIR {
        file_id: "test".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "Service".to_string()),
            CoreOp::DefInterface("IF1".to_string(), "Comparable".to_string()),
            CoreOp::Extends("C1".to_string(), "Base".to_string()),
            CoreOp::Implements("C1".to_string(), "IF1".to_string()),
            CoreOp::Import("IM1".to_string(), "module".to_string(), "Foo".to_string()),
            CoreOp::TypeAlias("T1".to_string(), "string[]".to_string()),
        ],
    };
    let bytes = encode(&ir);
    let decoded = decode(&bytes).unwrap();
    assert_eq!(decoded, ir);
}
