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
fn arity_table_covers_all_opcodes() {
    let all_opcodes = [
        "DEF_C", "DEF_M", "DEF_F", "DEF_I", "SIG", "RET", "FIELD_T",
        "FLAGS", "FLAGS_C", "EXT", "IMPL", "INJECTS", "IMP", "TYPE",
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
fn arity_table_unknown_opcode() {
    assert_eq!(arity("UNKNOWN"), None);
    assert_eq!(arity(""), None);
}

#[test]
fn opcode_name_matches_variant() {
    assert_eq!(opcode_name(&CoreOp::DefClass("C1".into(), "X".into())), "DEF_C");
    assert_eq!(opcode_name(&CoreOp::DefMethod("C1".into(), "M1".into(), "X".into())), "DEF_M");
    assert_eq!(opcode_name(&CoreOp::DefField("C1".into(), "F1".into(), "X".into())), "DEF_F");
    assert_eq!(opcode_name(&CoreOp::DefInterface("I1".into(), "X".into())), "DEF_I");
    assert_eq!(opcode_name(&CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "x".into())), "SIG");
    assert_eq!(opcode_name(&CoreOp::Return("M1".into(), "$v".into())), "RET");
    assert_eq!(opcode_name(&CoreOp::FieldType("F1".into(), "$n".into())), "FIELD_T");
    assert_eq!(opcode_name(&CoreOp::Flags("M1".into(), vec![])), "FLAGS");
    assert_eq!(opcode_name(&CoreOp::ClassFlags("C1".into(), vec![])), "FLAGS_C");
    assert_eq!(opcode_name(&CoreOp::Extends("C1".into(), "C2".into())), "EXT");
    assert_eq!(opcode_name(&CoreOp::Implements("C1".into(), "I1".into())), "IMPL");
    assert_eq!(opcode_name(&CoreOp::Injects("C1".into(), vec![])), "INJECTS");
    assert_eq!(opcode_name(&CoreOp::Import("IM1".into(), "m".into(), "n".into())), "IMP");
    assert_eq!(opcode_name(&CoreOp::TypeAlias("T1".into(), "X".into())), "TYPE");
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