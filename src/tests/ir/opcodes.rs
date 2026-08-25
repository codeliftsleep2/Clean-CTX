use crate::ir::opcodes::*;

#[test]
fn core_op_def_class_display() {
    let op = CoreOp::DefClass("C1".into(), "MyClass".into());
    assert_eq!(format!("{}", op), "DEF_C C1 MyClass");
}

#[test]
fn core_op_def_method_display() {
    let op = CoreOp::DefMethod("C1".into(), "M1".into(), "doStuff".into());
    assert_eq!(format!("{}", op), "DEF_M C1 M1 doStuff");
}

#[test]
fn core_op_param_display() {
    let op = CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "name".into());
    assert_eq!(format!("{}", op), "SIG M1 P1 $s name");
}

#[test]
fn core_op_return_display() {
    let op = CoreOp::Return("M1".into(), "$b".into());
    assert_eq!(format!("{}", op), "RET M1 $b");
}

#[test]
fn core_op_flags_display() {
    let op = CoreOp::Flags("M1".into(), vec!["IF".into(), "LOOP".into()]);
    assert_eq!(format!("{}", op), "FLAGS M1 IF LOOP");
}

#[test]
fn core_op_extends_display() {
    let op = CoreOp::Extends("C1".into(), "C2".into());
    assert_eq!(format!("{}", op), "EXT C1 C2");
}

#[test]
fn core_op_import_display() {
    let op = CoreOp::Import("IM1".into(), "rxjs".into(), "map".into());
    assert_eq!(format!("{}", op), "IMP IM1 rxjs map");
}

#[test]
fn core_op_type_alias_display() {
    let op = CoreOp::TypeAlias("T1".into(), "UserId".into());
    assert_eq!(format!("{}", op), "TYPE T1 UserId");
}

#[test]
fn core_op_body_display() {
    // Spans are transport metadata and must not leak into the rendered
    // form (byte-identical to the pre-span format).
    let op = CoreOp::Body(
        "M1".into(),
        "{\n  return 42;\n}".into(),
        Some(120),
        Some(138),
    );
    assert_eq!(format!("{}", op), "BODY M1 {\n  return 42;\n}");
}

#[test]
fn core_op_body_round_trips_through_wire() {
    use crate::ir::wire::{op_to_tuple, tuple_to_op};
    // Legacy span-less shape keeps the historical 3-tuple wire layout.
    let op = CoreOp::Body("M1".into(), "{\n  return 42;\n}".into(), None, None);
    let tuple = op_to_tuple(&op);
    assert_eq!(tuple[0], "BODY");
    assert_eq!(tuple[1], "M1");
    assert_eq!(tuple[2], "{\n  return 42;\n}");
    assert_eq!(tuple.len(), 3, "span-less Body must stay a 3-tuple");
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(restored, op);
}

#[test]
fn core_op_body_span_round_trips_through_wire() {
    use crate::ir::wire::{op_to_tuple, tuple_to_op};
    let op = CoreOp::Body(
        "M7".into(),
        "{\n  return 42;\n}".into(),
        Some(1024),
        Some(1048),
    );
    let tuple = op_to_tuple(&op);
    assert_eq!(
        tuple,
        vec![
            "BODY".to_string(),
            "M7".to_string(),
            "{\n  return 42;\n}".to_string(),
            "1024".to_string(),
            "1048".to_string(),
        ],
        "spanned Body serializes as a 5-tuple with decimal offsets"
    );
    let restored = tuple_to_op(&tuple).unwrap();
    assert_eq!(restored, op);
    assert_eq!(restored.body_span(), Some((1024, 1048)));
}

#[test]
fn body_span_helper_enforces_pairing() {
    // The pairing invariant: partial spans are unrepresentable through
    // `body_span`, which only fires when both fields are present.
    let none = CoreOp::Body("M1".into(), "{}".into(), None, None);
    assert_eq!(none.body_span(), None);
    let both = CoreOp::Body("M1".into(), "{}".into(), Some(0), Some(2));
    assert_eq!(both.body_span(), Some((0, 2)));
}

#[test]
fn wire_rejects_malformed_body_tuples() {
    use crate::ir::wire::tuple_to_op;
    // 4-tuple (partial span) must be rejected — never produced by
    // `op_to_tuple`, never accepted on decode.
    let four = vec![
        "BODY".to_string(),
        "M1".to_string(),
        "{}".to_string(),
        "10".to_string(),
    ];
    assert!(tuple_to_op(&four).is_none());
    // Non-numeric span fields must be rejected.
    let five_bad = vec![
        "BODY".to_string(),
        "M1".to_string(),
        "{}".to_string(),
        "abc".to_string(),
        "20".to_string(),
    ];
    assert!(tuple_to_op(&five_bad).is_none());
}

#[test]
fn arity_table_covers_all_opcodes() {
    let all_opcodes = [
        "DEF_C", "DEF_M", "DEF_F", "DEF_I", "SIG", "RET", "FIELD_T", "FLAGS", "FLAGS_C", "EXT",
        "IMPL", "INJECTS", "IMP", "TYPE", "BODY", "DATAFLOW", "CTRL", "EFFECT", "CTX",
    ];
    for opcode in &all_opcodes {
        assert!(
            arity(opcode).is_some(),
            "arity() returned None for known opcode: {}",
            opcode
        );
    }
}

#[test]
fn arity_table_variadic_opcodes() {
    assert_eq!(arity("FLAGS"), Some(-1));
    assert_eq!(arity("FLAGS_C"), Some(-1));
    assert_eq!(arity("INJECTS"), Some(-1));
    // apply_edit Phase 1: BODY has a dual shape (legacy 3-tuple or
    // spanned 5-tuple), so its arity is variadic.
    assert_eq!(arity("BODY"), Some(-1));
}

#[test]
fn arity_table_fixed_opcodes() {
    assert_eq!(arity("DEF_C"), Some(3));
    assert_eq!(arity("DEF_M"), Some(4));
    assert_eq!(arity("DEF_F"), Some(4));
    assert_eq!(arity("SIG"), Some(5));
    assert_eq!(arity("RET"), Some(3));
    assert_eq!(arity("IMP"), Some(4));
}

#[test]
fn opcode_name_body() {
    assert_eq!(
        opcode_name(&CoreOp::Body("M1".into(), "{}".into(), None, None)),
        "BODY"
    );
}

#[test]
fn arity_table_unknown_opcode() {
    assert_eq!(arity("UNKNOWN"), None);
    assert_eq!(arity(""), None);
}

#[test]
fn opcode_name_matches_variant() {
    assert_eq!(
        opcode_name(&CoreOp::DefClass("C1".into(), "X".into())),
        "DEF_C"
    );
    assert_eq!(
        opcode_name(&CoreOp::DefMethod("C1".into(), "M1".into(), "X".into())),
        "DEF_M"
    );
    assert_eq!(
        opcode_name(&CoreOp::DefField("C1".into(), "F1".into(), "X".into())),
        "DEF_F"
    );
    assert_eq!(
        opcode_name(&CoreOp::DefInterface("I1".into(), "X".into())),
        "DEF_I"
    );
    assert_eq!(
        opcode_name(&CoreOp::Param(
            "M1".into(),
            "P1".into(),
            "$s".into(),
            "x".into()
        )),
        "SIG"
    );
    assert_eq!(
        opcode_name(&CoreOp::Return("M1".into(), "$v".into())),
        "RET"
    );
    assert_eq!(
        opcode_name(&CoreOp::FieldType("F1".into(), "$n".into())),
        "FIELD_T"
    );
    assert_eq!(opcode_name(&CoreOp::Flags("M1".into(), vec![])), "FLAGS");
    assert_eq!(
        opcode_name(&CoreOp::ClassFlags("C1".into(), vec![])),
        "FLAGS_C"
    );
    assert_eq!(
        opcode_name(&CoreOp::Extends("C1".into(), "C2".into())),
        "EXT"
    );
    assert_eq!(
        opcode_name(&CoreOp::Implements("C1".into(), "I1".into())),
        "IMPL"
    );
    assert_eq!(
        opcode_name(&CoreOp::Injects("C1".into(), vec![])),
        "INJECTS"
    );
    assert_eq!(
        opcode_name(&CoreOp::Import("IM1".into(), "m".into(), "n".into())),
        "IMP"
    );
    assert_eq!(
        opcode_name(&CoreOp::TypeAlias("T1".into(), "X".into())),
        "TYPE"
    );
}

#[test]
fn core_ops_are_cloneable() {
    let op = CoreOp::DefClass("C1".into(), "Foo".into());
    let cloned = op.clone();
    assert_eq!(op, cloned);
}

#[test]
fn core_ops_support_eq() {
    let a = CoreOp::Flags("M1".into(), vec!["IF".into()]);
    let b = CoreOp::Flags("M1".into(), vec!["IF".into()]);
    let c = CoreOp::Flags("M1".into(), vec!["LOOP".into()]);
    assert_eq!(a, b);
    assert_ne!(a, c);
}
