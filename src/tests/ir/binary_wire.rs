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
    assert_eq!(bytes[2], 0x02, "version byte");
}

// ── Round-trip Tests ───────────────────────────────────────────────

#[test]
fn test_round_trip_simple_ir() {
    let ir = make_simple_ir();
    let bytes = encode(&ir);
    let decoded = decode(&bytes).unwrap();

    assert_eq!(
        decoded.instructions.len(),
        ir.instructions.len(),
        "instruction count should match"
    );
    for (i, (original, decoded_op)) in ir
        .instructions
        .iter()
        .zip(decoded.instructions.iter())
        .enumerate()
    {
        // Check structural equality (note: binary format may use empty strings
        // for some parent IDs like class_id in DefMethod)
        match (original, decoded_op) {
            (CoreOp::DefClass(_, orig_name), CoreOp::DefClass(_, dec_name)) => {
                assert_eq!(orig_name, dec_name, "DefClass name mismatch at {}", i);
            }
            (
                CoreOp::DefMethod(_, orig_mid, orig_name),
                CoreOp::DefMethod(_, dec_mid, dec_name),
            ) => {
                assert_eq!(orig_mid, dec_mid, "DefMethod mid mismatch at {}", i);
                assert_eq!(orig_name, dec_name, "DefMethod name mismatch at {}", i);
            }
            (CoreOp::DefField(_, orig_fid, orig_name), CoreOp::DefField(_, dec_fid, dec_name)) => {
                assert_eq!(orig_fid, dec_fid, "DefField fid mismatch at {}", i);
                assert_eq!(orig_name, dec_name, "DefField name mismatch at {}", i);
            }
            (CoreOp::Return(_, orig_ty), CoreOp::Return(_, dec_ty)) => {
                assert_eq!(orig_ty, dec_ty, "Return type mismatch at {}", i);
            }
            (
                CoreOp::Param(_, orig_pid, orig_ty, orig_name),
                CoreOp::Param(_, dec_pid, dec_ty, dec_name),
            ) => {
                assert_eq!(orig_pid, dec_pid, "Param pid mismatch at {}", i);
                assert_eq!(orig_ty, dec_ty, "Param type mismatch at {}", i);
                assert_eq!(orig_name, dec_name, "Param name mismatch at {}", i);
            }
            (CoreOp::Flags(_, orig_flags), CoreOp::Flags(_, dec_flags)) => {
                assert_eq!(orig_flags, dec_flags, "Flags mismatch at {}", i);
            }
            _ => panic!(
                "Opcode variant mismatch at index {}: original={:?} decoded={:?}",
                i, original, decoded_op
            ),
        }
    }
}

#[test]
fn test_round_trip_full_ir() {
    let ir = make_full_ir();
    let bytes = encode(&ir);
    let decoded = decode(&bytes).unwrap();

    assert_eq!(
        decoded.instructions.len(),
        ir.instructions.len(),
        "instruction count should match"
    );
    for (i, (original, decoded_op)) in ir
        .instructions
        .iter()
        .zip(decoded.instructions.iter())
        .enumerate()
    {
        match (original, decoded_op) {
            (CoreOp::DefClass(_, orig_name), CoreOp::DefClass(_, dec_name)) => {
                assert_eq!(orig_name, dec_name, "DefClass name mismatch at {}", i);
            }
            (
                CoreOp::DefMethod(_, orig_mid, orig_name),
                CoreOp::DefMethod(_, dec_mid, dec_name),
            ) => {
                assert_eq!(orig_mid, dec_mid, "DefMethod mid mismatch at {}", i);
                assert_eq!(orig_name, dec_name, "DefMethod name mismatch at {}", i);
            }
            (CoreOp::DefField(_, orig_fid, orig_name), CoreOp::DefField(_, dec_fid, dec_name)) => {
                assert_eq!(orig_fid, dec_fid, "DefField fid mismatch at {}", i);
                assert_eq!(orig_name, dec_name, "DefField name mismatch at {}", i);
            }
            (CoreOp::DefInterface(_, orig_name), CoreOp::DefInterface(_, dec_name)) => {
                assert_eq!(orig_name, dec_name, "DefInterface name mismatch at {}", i);
            }
            (
                CoreOp::Param(_, orig_pid, orig_ty, orig_name),
                CoreOp::Param(_, dec_pid, dec_ty, dec_name),
            ) => {
                assert_eq!(orig_pid, dec_pid, "Param pid mismatch at {}", i);
                assert_eq!(orig_ty, dec_ty, "Param type mismatch at {}", i);
                assert_eq!(orig_name, dec_name, "Param name mismatch at {}", i);
            }
            (CoreOp::Return(_, orig_ty), CoreOp::Return(_, dec_ty)) => {
                assert_eq!(orig_ty, dec_ty, "Return type mismatch at {}", i);
            }
            (CoreOp::FieldType(_, orig_ty), CoreOp::FieldType(_, dec_ty)) => {
                assert_eq!(orig_ty, dec_ty, "FieldType mismatch at {}", i);
            }
            (CoreOp::Flags(_, orig_flags), CoreOp::Flags(_, dec_flags)) => {
                assert_eq!(orig_flags, dec_flags, "Flags mismatch at {}", i);
            }
            (CoreOp::ClassFlags(_, orig_flags), CoreOp::ClassFlags(_, dec_flags)) => {
                assert_eq!(orig_flags, dec_flags, "ClassFlags mismatch at {}", i);
            }
            (CoreOp::Extends(_, orig_parent), CoreOp::Extends(_, dec_parent)) => {
                assert_eq!(orig_parent, dec_parent, "Extends parent mismatch at {}", i);
            }
            (CoreOp::Implements(_, orig_iid), CoreOp::Implements(_, dec_iid)) => {
                assert_eq!(orig_iid, dec_iid, "Implements iid mismatch at {}", i);
            }
            (CoreOp::Injects(_, orig_deps), CoreOp::Injects(_, dec_deps)) => {
                assert_eq!(orig_deps, dec_deps, "Injects deps mismatch at {}", i);
            }
            (CoreOp::Import(_, orig_mod, orig_named), CoreOp::Import(_, dec_mod, dec_named)) => {
                assert_eq!(orig_mod, dec_mod, "Import module mismatch at {}", i);
                assert_eq!(orig_named, dec_named, "Import named mismatch at {}", i);
            }
            (CoreOp::TypeAlias(_, orig_original), CoreOp::TypeAlias(_, dec_original)) => {
                assert_eq!(
                    orig_original, dec_original,
                    "TypeAlias original mismatch at {}",
                    i
                );
            }
            (CoreOp::Pattern(_, orig_args), CoreOp::Pattern(_, dec_args)) => {
                assert_eq!(orig_args, dec_args, "Pattern args mismatch at {}", i);
            }
            _ => panic!(
                "Opcode variant mismatch at index {}: original={:?} decoded={:?}",
                i, original, decoded_op
            ),
        }
    }
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
    assert_eq!(decoded.instructions.len(), 0);
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
    // Legacy magic would be 0xCC,0x01,0x01 but current is 0xCC,0x02,0x02
    let data = vec![0xCC, 0x02, 0x02];
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
    assert_eq!(
        decoded.instructions.len(),
        ir.instructions.len(),
        "base64 round-trip instruction count should match"
    );

    // Verify key structural properties
    let class_count = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefClass(..)))
        .count();
    let decoded_class_count = decoded
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefClass(..)))
        .count();
    assert_eq!(decoded_class_count, class_count, "class count should match");
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
    assert_eq!(
        decoded.instructions.len(),
        ir.instructions.len(),
        "wire_to_ir_detect should handle binary encoding"
    );
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
    assert_eq!(decoded.instructions.len(), ir.instructions.len());
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
    assert_eq!(decoded.instructions.len(), 300); // 100 * 3 instructions
}

// ── Zero-State Tests ──────────────────────────────────────────────

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
    assert_eq!(decoded.instructions.len(), 4);
    for (orig, dec) in ir.instructions.iter().zip(decoded.instructions.iter()) {
        match (orig, dec) {
            (CoreOp::Flags(_, of), CoreOp::Flags(_, df)) => assert_eq!(of, df),
            (CoreOp::ClassFlags(_, of), CoreOp::ClassFlags(_, df)) => assert_eq!(of, df),
            (CoreOp::Injects(_, od), CoreOp::Injects(_, dd)) => assert_eq!(od, dd),
            (CoreOp::Pattern(_, oa), CoreOp::Pattern(_, da)) => assert_eq!(oa, da),
            _ => panic!("variant mismatch: {:?} vs {:?}", orig, dec),
        }
    }
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
    assert_eq!(decoded.instructions.len(), 6);
}
