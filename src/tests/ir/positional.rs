// src/tests/ir/positional.rs
//
// Tests for Phase H: Positional Encoding.

use crate::ir::opcodes::CoreOp;
use crate::ir::wire::op_to_tuple;
use crate::ir::positional::{
    PositionalConfig, encode_op, decode_op, encode_stream, ir_to_positional_wire,
    estimate_savings, positional_char_count, verify_round_trip,
};

fn defclass(id: &str, name: &str) -> CoreOp {
    CoreOp::DefClass(id.into(), name.into())
}

fn defmethod(cid: &str, mid: &str, name: &str) -> CoreOp {
    CoreOp::DefMethod(cid.into(), mid.into(), name.into())
}

fn deffield(cid: &str, fid: &str, name: &str) -> CoreOp {
    CoreOp::DefField(cid.into(), fid.into(), name.into())
}

fn param(mid: &str, pid: &str, ty: &str, name: &str) -> CoreOp {
    CoreOp::Param(mid.into(), pid.into(), ty.into(), name.into())
}

fn ret(mid: &str, ty: &str) -> CoreOp {
    CoreOp::Return(mid.into(), ty.into())
}

fn flags(tid: &str, fs: &[&str]) -> CoreOp {
    CoreOp::Flags(tid.into(), fs.iter().map(|s| s.to_string()).collect())
}

fn class_flags(cid: &str, fs: &[&str]) -> CoreOp {
    CoreOp::ClassFlags(cid.into(), fs.iter().map(|s| s.to_string()).collect())
}

fn extends(child: &str, parent: &str) -> CoreOp {
    CoreOp::Extends(child.into(), parent.into())
}

fn implements(cid: &str, iid: &str) -> CoreOp {
    CoreOp::Implements(cid.into(), iid.into())
}

fn injects(cid: &str, deps: &[&str]) -> CoreOp {
    CoreOp::Injects(cid.into(), deps.iter().map(|s| s.to_string()).collect())
}

fn import(alias: &str, module: &str, named: &str) -> CoreOp {
    CoreOp::Import(alias.into(), module.into(), named.into())
}

fn type_alias(alias: &str, original: &str) -> CoreOp {
    CoreOp::TypeAlias(alias.into(), original.into())
}

fn field_type(fid: &str, ty: &str) -> CoreOp {
    CoreOp::FieldType(fid.into(), ty.into())
}

fn definterface(id: &str, name: &str) -> CoreOp {
    CoreOp::DefInterface(id.into(), name.into())
}

// ── Config ────────────────────────────────────────────────────

#[test]
fn config_default_is_stripped() {
    let cfg = PositionalConfig::default();
    assert!(!cfg.tagged, "default config should strip opcodes");
    assert_eq!(cfg.tagged, PositionalConfig::stripped().tagged);
    assert_ne!(cfg.tagged, PositionalConfig::tagged().tagged);
}

#[test]
fn config_constructors_distinct() {
    let s = PositionalConfig::stripped();
    let t = PositionalConfig::tagged();
    assert!(!s.tagged);
    assert!(t.tagged);
    assert_ne!(s, t);
}

// ── Stripped encoding ─────────────────────────────────────────

#[test]
fn stripped_strips_opcode_for_def_c() {
    let op = defclass("C1", "SampleService");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["C1", "SampleService"]);
}

#[test]
fn stripped_strips_opcode_for_def_m() {
    let op = defmethod("C1", "M1", "processComplexData");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["C1", "M1", "processComplexData"]);
}

#[test]
fn stripped_strips_opcode_for_def_f() {
    let op = deffield("C1", "F1", "payload");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["C1", "F1", "payload"]);
}

#[test]
fn stripped_strips_opcode_for_sig() {
    let op = param("M1", "P1", "$s", "payload");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["M1", "P1", "$s", "payload"]);
}

#[test]
fn stripped_strips_opcode_for_ret() {
    let op = ret("M1", "$b");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["M1", "$b"]);
}

#[test]
fn stripped_strips_opcode_for_field_t() {
    let op = field_type("F1", "$n");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["F1", "$n"]);
}

#[test]
fn stripped_strips_opcode_for_def_i() {
    let op = definterface("I1", "ISample");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["I1", "ISample"]);
}

#[test]
fn stripped_strips_opcode_for_ext() {
    let op = extends("C1", "C2");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["C1", "C2"]);
}

#[test]
fn stripped_strips_opcode_for_impl() {
    let op = implements("C1", "I1");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["C1", "I1"]);
}

#[test]
fn stripped_strips_opcode_for_flags_variadic() {
    let op = flags("M1", &["IF", "LOOP", "RET"]);
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["M1", "IF", "LOOP", "RET"]);
}

#[test]
fn stripped_strips_opcode_for_class_flags_variadic() {
    let op = class_flags("C1", &["EXPORT", "ABSTRACT"]);
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["C1", "EXPORT", "ABSTRACT"]);
}

#[test]
fn stripped_strips_opcode_for_injects_variadic() {
    let op = injects("C1", &["S1", "S2", "S3"]);
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["C1", "S1", "S2", "S3"]);
}

#[test]
fn stripped_strips_opcode_for_imp() {
    let op = import("$im", "rxjs", "map");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["$im", "rxjs", "map"]);
}

#[test]
fn stripped_strips_opcode_for_type() {
    let op = type_alias("$uid", "UserId");
    let enc = encode_op(&op, PositionalConfig::stripped());
    assert_eq!(enc, vec!["$uid", "UserId"]);
}

// ── Tagged encoding ───────────────────────────────────────────

#[test]
fn tagged_preserves_opcode() {
    let op = defclass("C1", "SampleService");
    let enc = encode_op(&op, PositionalConfig::tagged());
    assert_eq!(enc, vec!["DEF_C", "C1", "SampleService"]);
}

#[test]
fn tagged_matches_op_to_tuple() {
    let op = defmethod("C1", "M1", "processComplexData");
    let tagged = encode_op(&op, PositionalConfig::tagged());
    let from_wire = op_to_tuple(&op);
    assert_eq!(tagged, from_wire, "tagged encoding should match op_to_tuple");
}

// ── Decode ────────────────────────────────────────────────────

#[test]
fn decode_stripped_def_c() {
    let op = decode_op("DEF_C", &["C1".into(), "SampleService".into()]);
    assert_eq!(op, Some(defclass("C1", "SampleService")));
}

#[test]
fn decode_stripped_def_m() {
    let op = decode_op("DEF_M", &["C1".into(), "M1".into(), "processComplexData".into()]);
    assert_eq!(op, Some(defmethod("C1", "M1", "processComplexData")));
}

#[test]
fn decode_stripped_sig() {
    let op = decode_op("SIG", &["M1".into(), "P1".into(), "$s".into(), "payload".into()]);
    assert_eq!(op, Some(param("M1", "P1", "$s", "payload")));
}

#[test]
fn decode_stripped_flags_variadic() {
    let op = decode_op("FLAGS", &["M1".into(), "IF".into(), "LOOP".into()]);
    assert_eq!(op, Some(flags("M1", &["IF", "LOOP"])));
}

#[test]
fn decode_stripped_injects_variadic() {
    let op = decode_op("INJECTS", &["C1".into(), "S1".into(), "S2".into()]);
    assert_eq!(op, Some(injects("C1", &["S1", "S2"])));
}

#[test]
fn decode_unknown_opcode_returns_none() {
    let op = decode_op("UNKNOWN", &["a".into(), "b".into()]);
    assert_eq!(op, None);
}

#[test]
fn decode_arity_mismatch_fixed_returns_none() {
    let op = decode_op("DEF_C", &["C1".into()]);
    assert_eq!(op, None);
}

#[test]
fn decode_arity_mismatch_variadic_returns_none() {
    let op = decode_op("FLAGS", &[]);
    assert_eq!(op, None);
}

#[test]
fn decode_ret_three_operands_invalid() {
    let op = decode_op("RET", &["M1".into(), "$b".into(), "extra".into()]);
    assert_eq!(op, None);
}

// ── Round-trip ────────────────────────────────────────────────

#[test]
fn round_trip_stripped_all_variants() {
    let ops = vec![
        defclass("C1", "SampleService"),
        defmethod("C1", "M1", "processComplexData"),
        param("M1", "P1", "$s", "payload"),
        ret("M1", "$b"),
        flags("M1", &["IF"]),
        class_flags("C1", &["EXPORT"]),
        extends("C1", "C2"),
        implements("C1", "I1"),
        injects("C1", &["S1"]),
        import("$im", "rxjs", "map"),
        type_alias("$uid", "UserId"),
        field_type("F1", "$n"),
        deffield("C1", "F1", "payload"),
        definterface("I1", "ISample"),
    ];

    for op in &ops {
        let enc = encode_op(op, PositionalConfig::stripped());
        let opcode = op_to_tuple(op).remove(0);
        let decoded = decode_op(&opcode, &enc)
            .unwrap_or_else(|| panic!("decode failed for {:?}", op));
        assert_eq!(&decoded, op, "round-trip mismatch for {:?}", op);
    }
}

#[test]
fn round_trip_tagged_all_variants() {
    let ops = vec![
        defclass("C1", "SampleService"),
        defmethod("C1", "M1", "processComplexData"),
        param("M1", "P1", "$s", "payload"),
        ret("M1", "$b"),
        flags("M1", &["IF", "LOOP"]),
        class_flags("C1", &["EXPORT"]),
        extends("C1", "C2"),
        implements("C1", "I1"),
        injects("C1", &["S1", "S2"]),
        import("$im", "rxjs", "map"),
        type_alias("$uid", "UserId"),
        field_type("F1", "$n"),
        deffield("C1", "F1", "payload"),
        definterface("I1", "ISample"),
    ];

    for op in &ops {
        let enc = encode_op(op, PositionalConfig::tagged());
        let opcode = enc[0].clone();
        let operands: Vec<String> = enc[1..].to_vec();
        let decoded = decode_op(&opcode, &operands)
            .unwrap_or_else(|| panic!("decode failed for {:?}", op));
        assert_eq!(&decoded, op, "round-trip mismatch for {:?}", op);
    }
}

// ── Stream / wire ─────────────────────────────────────────────

#[test]
fn encode_stream_preserves_order() {
    let ops = vec![
        defclass("C1", "Foo"),
        defmethod("C1", "M1", "doWork"),
        ret("M1", "$v"),
    ];
    let s = encode_stream(&ops, PositionalConfig::stripped());
    assert_eq!(s.len(), 3);
    assert_eq!(s[0], vec!["C1", "Foo"]);
    assert_eq!(s[1], vec!["C1", "M1", "doWork"]);
    assert_eq!(s[2], vec!["M1", "$v"]);
}

#[test]
fn encode_stream_empty() {
    let empty: Vec<CoreOp> = Vec::new();
    let s = encode_stream(&empty, PositionalConfig::stripped());
    assert!(s.is_empty());
}

#[test]
fn ir_to_positional_wire_shape() {
    let ops = vec![defclass("C1", "Foo"), defmethod("C1", "M1", "doWork")];
    let wire = ir_to_positional_wire("α1", 1, &ops, PositionalConfig::stripped());
    assert_eq!(wire["file"], "α1");
    assert_eq!(wire["v"], 1);
    assert_eq!(wire["encoding"], "positional");
    let ir = wire["ir"].as_array().expect("ir should be array");
    assert_eq!(ir.len(), 2);
    assert_eq!(ir[0], serde_json::json!(["C1", "Foo"]));
    assert_eq!(ir[1], serde_json::json!(["C1", "M1", "doWork"]));
}

#[test]
fn ir_to_positional_wire_tagged_shape() {
    let ops = vec![defclass("C1", "Foo")];
    let wire = ir_to_positional_wire("α1", 7, &ops, PositionalConfig::tagged());
    assert_eq!(wire["encoding"], "tagged");
    assert_eq!(wire["v"], 7);
    assert_eq!(wire["ir"][0], serde_json::json!(["DEF_C", "C1", "Foo"]));
}

// ── Savings estimation ────────────────────────────────────────

#[test]
fn stripped_is_shorter_than_named() {
    let ops = vec![
        defclass("C1", "SampleService"),
        defmethod("C1", "M1", "processComplexData"),
        param("M1", "P1", "$s", "payload"),
        ret("M1", "$b"),
    ];
    let (named, positional) = estimate_savings(&ops);
    assert!(positional < named,
        "positional should be smaller: named={} positional={}", named, positional);
}

#[test]
fn estimate_savings_zero_ops() {
    let empty: Vec<CoreOp> = Vec::new();
    let (named, positional) = estimate_savings(&empty);
    assert_eq!(named, 0);
    assert_eq!(positional, 0);
}

#[test]
fn positional_char_count_includes_envelope() {
    let ops = vec![defclass("C1", "Foo")];
    let count = positional_char_count(&ops, PositionalConfig::stripped());
    assert!(count > 12);
}

// ── verify_round_trip ─────────────────────────────────────────

#[test]
fn verify_round_trip_match() {
    let ops = vec![defclass("C1", "Foo"), defmethod("C1", "M1", "doWork")];
    let tagged: Vec<Vec<String>> = ops.iter().map(op_to_tuple).collect();
    assert_eq!(verify_round_trip(&ops, &tagged), None);
}

#[test]
fn verify_round_trip_length_mismatch() {
    let ops = vec![defclass("C1", "Foo")];
    let tagged: Vec<Vec<String>> = vec![];
    let result = verify_round_trip(&ops, &tagged);
    assert_eq!(result, Some(0));
}

#[test]
fn verify_round_trip_mismatch_detected() {
    let ops = vec![defclass("C1", "Foo"), defmethod("C1", "M1", "doWork")];
    let mut tagged: Vec<Vec<String>> = ops.iter().map(op_to_tuple).collect();
    tagged[1][0] = "WRONG".to_string();
    let result = verify_round_trip(&ops, &tagged);
    assert_eq!(result, Some(1));
}

#[test]
fn verify_round_trip_empty_tuple() {
    let ops = vec![defclass("C1", "Foo")];
    let tagged: Vec<Vec<String>> = vec![vec![]];
    let result = verify_round_trip(&ops, &tagged);
    assert_eq!(result, Some(0));
}
