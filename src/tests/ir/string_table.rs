// src/tests/ir/string_table.rs
//
// Phase I: Ultra-Compact IR — String Table + Relative Referencing tests.

use crate::ir::string_table::{
    self, StringTable, encode_op, decode_op, ir_to_string_table_wire, wire_to_ir,
};
use crate::ir::compiler::CompiledIR;
use crate::ir::opcodes::CoreOp;
use serde_json::json;

/// Helper to create a simple CompiledIR for testing.
fn make_test_ir(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "test".to_string(),
        instructions,
        version: 1,
    }
}

/// Returns a canonical set of test instructions.
fn sample_instructions() -> Vec<CoreOp> {
    vec![
        CoreOp::DefClass("C1".into(), "SampleService".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "processComplexData".into()),
        CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "payload".into()),
        CoreOp::Return("M1".into(), "$b".into()),
        CoreOp::Flags("C1".into(), vec!["IF".into()]),
        CoreOp::DefField("C1".into(), "F1".into(), "serviceName".into()),
        CoreOp::FieldType("F1".into(), "$s".into()),
        CoreOp::Extends("C1".into(), "BaseService".into()),
        CoreOp::Implements("C1".into(), "IDisposable".into()),
        CoreOp::Import("$".into(), "rxjs".into(), "Observable".into()),
    ]
}

// ── StringTable Tests ─────────────────────────────────────────────

#[test]
fn test_string_table_empty() {
    let table = StringTable::new();
    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
}

#[test]
fn test_string_table_intern_and_lookup() {
    let mut table = StringTable::new();
    assert_eq!(table.intern("hello"), 0);
    assert_eq!(table.intern("world"), 1);
    assert_eq!(table.intern("hello"), 0); // dedup
    assert_eq!(table.len(), 2);
    assert_eq!(table.lookup(0), Some("hello"));
    assert_eq!(table.lookup(1), Some("world"));
    assert_eq!(table.lookup(99), None);
}

#[test]
fn test_string_table_from_instructions() {
    let instructions = sample_instructions();
    let table = StringTable::from_instructions(&instructions);
    // Should contain "C1", "SampleService", "M1", "$s", etc.
    assert!(table.len() > 0);
    // Verify strings exist via lookup - use the inverse: find indices of expected strings
    let strings: Vec<&str> = table.strings().iter().map(|s| s.as_str()).collect();
    assert!(strings.contains(&"C1"));
    assert!(strings.contains(&"$s"));
    assert!(strings.contains(&"M1"));
    assert!(strings.contains(&"SampleService"));
}

#[test]
fn test_string_table_json_round_trip() {
    let mut table = StringTable::new();
    table.intern("a");
    table.intern("b");
    table.intern("c");
    let json = table.to_json();
    let parsed = StringTable::from_json(&json).unwrap();
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed.lookup(0), Some("a"));
    assert_eq!(parsed.lookup(1), Some("b"));
    assert_eq!(parsed.lookup(2), Some("c"));
}

// ── Encode/Decode Tests ──────────────────────────────────────────

#[test]
fn test_encode_decode_all_opcodes() {
    let instructions = sample_instructions();
    let table = StringTable::from_instructions(&instructions);

    for op in &instructions {
        let indices = encode_op(op, &table);
        let decoded = decode_op(&indices, &table).unwrap();
        assert_eq!(&decoded, op, "encode/decode round-trip failed for {:?}", op);
    }
}

#[test]
fn test_decode_invalid_index_returns_none() {
    let table = StringTable::new();
    let result = decode_op(&[999], &table);
    assert!(result.is_none());
}

#[test]
fn test_decode_empty_indices() {
    let table = StringTable::new();
    let result = decode_op(&[], &table);
    assert!(result.is_none());
}

// ── Wire Format Tests ────────────────────────────────────────────

#[test]
fn test_wire_round_trip() {
    let ir = make_test_ir(sample_instructions());
    let wire = ir_to_string_table_wire(&ir);
    let decoded = wire_to_ir(&wire).unwrap();

    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions.len(), ir.instructions.len());
    for (a, b) in decoded.instructions.iter().zip(ir.instructions.iter()) {
        assert_eq!(a, b, "instruction mismatch after round-trip");
    }
}

#[test]
fn test_wire_format_structure() {
    let ir = make_test_ir(sample_instructions());
    let wire = ir_to_string_table_wire(&ir);

    assert_eq!(wire.get("encoding").and_then(|v| v.as_str()), Some("string_table"));
    assert!(wire.get("t").and_then(|v| v.as_array()).is_some());
    assert!(wire.get("ir").and_then(|v| v.as_array()).is_some());
    assert_eq!(wire.get("file").and_then(|v| v.as_str()), Some("test"));
    assert_eq!(wire.get("v").and_then(|v| v.as_u64()), Some(1));
}

#[test]
fn test_wire_uses_integer_indices() {
    let instructions = sample_instructions();
    let ir = make_test_ir(instructions);
    let wire = ir_to_string_table_wire(&ir);
    let ir_array = wire.get("ir").and_then(|v| v.as_array()).unwrap();

    for tuple in ir_array {
        for elem in tuple.as_array().unwrap() {
            assert!(elem.is_number(), "expected integer, got {:?}", elem);
        }
    }
}

#[test]
fn test_wire_missing_fields() {
    // Missing "t" field
    let bad = json!({"file": "x", "v": 1, "ir": [[0, 1]]});
    assert!(wire_to_ir(&bad).is_none());

    // Missing "ir" field
    let bad = json!({"file": "x", "v": 1, "t": ["a", "b"]});
    assert!(wire_to_ir(&bad).is_none());

    // Missing "file" field
    let bad = json!({"v": 1, "t": ["a"], "ir": [[0]]});
    assert!(wire_to_ir(&bad).is_none());
}

#[test]
fn test_wire_empty_instructions() {
    let ir = make_test_ir(vec![]);
    let wire = ir_to_string_table_wire(&ir);
    let decoded = wire_to_ir(&wire).unwrap();
    assert!(decoded.instructions.is_empty());
}

#[test]
fn test_wire_single_instruction() {
    let ir = make_test_ir(vec![CoreOp::DefClass("C1".into(), "Foo".into())]);
    let wire = ir_to_string_table_wire(&ir);
    let decoded = wire_to_ir(&wire).unwrap();
    assert_eq!(decoded.instructions.len(), 1);
    assert_eq!(
        decoded.instructions[0],
        CoreOp::DefClass("C1".into(), "Foo".into())
    );
}

// ── Deduplication Test ───────────────────────────────────────────

#[test]
fn test_string_table_deduplicates() {
    // "C1" appears in multiple instructions — should only appear once in the table
    let instructions = vec![
        CoreOp::DefClass("C1".into(), "Service".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "doWork".into()),
        CoreOp::DefField("C1".into(), "F1".into(), "name".into()),
        CoreOp::Flags("C1".into(), vec!["IF".into()]),
    ];
    let table = StringTable::from_instructions(&instructions);
    // "C1" should appear exactly once in the table
    let c1_count = table.strings().iter().filter(|s| s.as_str() == "C1").count();
    assert_eq!(c1_count, 1, "C1 should appear exactly once in the table");
}

// ── Savings Estimation Test ──────────────────────────────────────

#[test]
fn test_string_table_is_smaller_than_named() {
    let mut instructions = sample_instructions();
    // Add enough repeated-string instructions so the table overhead is
    // amortised. The sample set has many unique strings (low dedup ratio),
    // so we add 30 more methods reusing "C1" to boost repetition.
    for i in 0..30 {
        instructions.push(CoreOp::DefMethod(
            "C1".into(),
            format!("M{}", i + 10),
            format!("method_{}", i),
        ));
        instructions.push(CoreOp::Return(format!("M{}", i + 10), "$s".into()));
        instructions.push(CoreOp::Flags("C1".into(), vec!["IF".into()]));
    }
    let ir = make_test_ir(instructions);
    let (named_chars, table_chars) = string_table::estimate_savings(&ir);
    assert!(
        table_chars < named_chars,
        "string_table ({}) should be smaller than named ({})",
        table_chars,
        named_chars
    );
}

#[test]
fn test_savings_with_large_file() {
    let mut instructions = sample_instructions();
    // Add many methods with highly-repeated strings (C1, $s, IF) to
    // simulate a realistic large file where the string table shines.
    for i in 0..60 {
        instructions.push(CoreOp::DefMethod(
            "C1".into(),
            format!("M{}", i + 10),
            format!("method_{}", i),
        ));
        instructions.push(CoreOp::Return(format!("M{}", i + 10), "$s".into()));
        instructions.push(CoreOp::Flags("C1".into(), vec!["IF".into()]));
        instructions.push(CoreOp::Param(
            format!("M{}", i + 10),
            format!("P{}", i),
            "$s".into(),
            "payload".into(),
        ));
    }
    let ir = make_test_ir(instructions);
    let (named_chars, table_chars) = string_table::estimate_savings(&ir);
    let savings_pct = (named_chars.saturating_sub(table_chars) * 100) / named_chars;
    assert!(
        savings_pct >= 20,
        "expected >=20% savings, got {}% (named={}, table={})",
        savings_pct,
        named_chars,
        table_chars
    );
}

// ── Edge Case Tests ──────────────────────────────────────────────

#[test]
fn test_intern_empty_string() {
    let mut table = StringTable::new();
    let idx = table.intern("");
    assert_eq!(idx, 0);
    assert_eq!(table.lookup(0), Some(""));
}

#[test]
fn test_from_json_invalid() {
    assert!(StringTable::from_json(&json!("not_an_array")).is_none());
    assert!(StringTable::from_json(&json!(42)).is_none());
    assert!(StringTable::from_json(&json!(null)).is_none());
}

#[test]
fn test_wire_to_ir_invalid_json() {
    // null value
    assert!(wire_to_ir(&json!(null)).is_none());
    // number value
    assert!(wire_to_ir(&json!(42)).is_none());
}

#[test]
fn test_wire_invalid_indices() {
    // Index out of bounds
    let bad = json!({
        "file": "x", "v": 1, "encoding": "string_table",
        "t": ["a", "b"],
        "ir": [[0, 5]]  // 5 is out of bounds
    });
    assert!(wire_to_ir(&bad).is_none());
}

#[test]
fn test_wire_non_integer_element() {
    // String where integer expected
    let bad = json!({
        "file": "x", "v": 1, "encoding": "string_table",
        "t": ["a", "b"],
        "ir": [["a", "b"]]  // strings instead of ints
    });
    assert!(wire_to_ir(&bad).is_none());
}