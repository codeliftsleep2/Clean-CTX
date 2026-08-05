// src/tests/ir/type_aliases.rs
//
// R-02 Phase 3 tests: type-alias substitution in the IR path.
//
// Verifies that `apply_type_aliases_to_ir` correctly substitutes
// type names in FieldType, Return, and Param ops, and emits
// CoreOp::TypeAlias ops for used aliases.

use std::collections::BTreeMap;
use crate::ir::opcodes::CoreOp;
use crate::ir::type_aliases::apply_type_aliases_to_ir;

fn aliases(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn empty_aliases_noop() {
    let mut instructions = vec![
        CoreOp::FieldType("F1".into(), "User".into()),
        CoreOp::Return("M1".into(), "User".into()),
    ];
    let original = instructions.clone();
    apply_type_aliases_to_ir(&mut instructions, &BTreeMap::new());
    assert_eq!(instructions, original);
}

#[test]
fn field_type_substituted() {
    let mut instructions = vec![
        CoreOp::FieldType("F1".into(), "User".into()),
    ];
    apply_type_aliases_to_ir(&mut instructions, &aliases(&[("User", "$uid")]));
    assert_eq!(instructions[0], CoreOp::FieldType("F1".into(), "$uid".into()));
    // TypeAlias op appended
    assert!(instructions.iter().any(|op| matches!(op, CoreOp::TypeAlias(a, o) if a == "$uid" && o == "User")));
}

#[test]
fn return_type_substituted() {
    let mut instructions = vec![
        CoreOp::Return("M1".into(), "Promise<User>".into()),
    ];
    apply_type_aliases_to_ir(&mut instructions, &aliases(&[("User", "$uid")]));
    assert_eq!(instructions[0], CoreOp::Return("M1".into(), "Promise<$uid>".into()));
}

#[test]
fn param_type_substituted() {
    let mut instructions = vec![
        CoreOp::Param("M1".into(), "P1".into(), "User".into(), "id".into()),
    ];
    apply_type_aliases_to_ir(&mut instructions, &aliases(&[("User", "$uid")]));
    assert_eq!(instructions[0], CoreOp::Param("M1".into(), "P1".into(), "$uid".into(), "id".into()));
}

#[test]
fn multiple_aliases_used() {
    let mut instructions = vec![
        CoreOp::FieldType("F1".into(), "User".into()),
        CoreOp::Return("M1".into(), "JsonObject".into()),
    ];
    apply_type_aliases_to_ir(
        &mut instructions,
        &aliases(&[("User", "$uid"), ("JsonObject", "$jo")]),
    );
    assert_eq!(instructions[0], CoreOp::FieldType("F1".into(), "$uid".into()));
    assert_eq!(instructions[1], CoreOp::Return("M1".into(), "$jo".into()));
    // Both TypeAlias ops appended
    let ta_ops: Vec<_> = instructions.iter()
        .filter_map(|op| match op {
            CoreOp::TypeAlias(a, o) => Some((a.clone(), o.clone())),
            _ => None,
        })
        .collect();
    assert!(ta_ops.contains(&("$uid".to_string(), "User".to_string())));
    assert!(ta_ops.contains(&("$jo".to_string(), "JsonObject".to_string())));
}

#[test]
fn unused_alias_not_emitted() {
    let mut instructions = vec![
        CoreOp::FieldType("F1".into(), "Service".into()),
    ];
    apply_type_aliases_to_ir(
        &mut instructions,
        &aliases(&[("User", "$uid"), ("JsonObject", "$jo")]),
    );
    // "Service" doesn't match any alias — no substitution
    assert_eq!(instructions[0], CoreOp::FieldType("F1".into(), "Service".into()));
    // No TypeAlias ops appended
    assert!(!instructions.iter().any(|op| matches!(op, CoreOp::TypeAlias(..))));
}

#[test]
fn primitive_types_not_substituted() {
    // Primitive types like $s, $n are already short — they won't match
    // because the original must be ≥ 4 chars.
    let mut instructions = vec![
        CoreOp::FieldType("F1".into(), "$s".into()),
        CoreOp::Return("M1".into(), "$n".into()),
    ];
    let original = instructions.clone();
    apply_type_aliases_to_ir(&mut instructions, &aliases(&[("$s", "$str"), ("$n", "$num")]));
    assert_eq!(instructions, original);
}

#[test]
fn nested_generics_substituted() {
    let mut instructions = vec![
        CoreOp::Return("M1".into(), "Map<string,User>".into()),
    ];
    apply_type_aliases_to_ir(&mut instructions, &aliases(&[("User", "$uid")]));
    assert_eq!(instructions[0], CoreOp::Return("M1".into(), "Map<string,$uid>".into()));
}

#[test]
fn no_partial_match() {
    // "User" must NOT match inside "UserService"
    let mut instructions = vec![
        CoreOp::FieldType("F1".into(), "UserService".into()),
    ];
    apply_type_aliases_to_ir(&mut instructions, &aliases(&[("User", "$uid")]));
    assert_eq!(instructions[0], CoreOp::FieldType("F1".into(), "UserService".into()));
    assert!(!instructions.iter().any(|op| matches!(op, CoreOp::TypeAlias(..))));
}

#[test]
fn non_type_ops_untouched() {
    let mut instructions = vec![
        CoreOp::DefClass("C1".into(), "UserService".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "getUser".into()),
        CoreOp::Extends("C1".into(), "User".into()),
        CoreOp::Implements("C1".into(), "IUser".into()),
    ];
    let original = instructions.clone();
    apply_type_aliases_to_ir(&mut instructions, &aliases(&[("User", "$uid")]));
    // DefClass, DefMethod, Extends, Implements are NOT type-bearing ops
    // — they should be untouched.
    assert_eq!(instructions, original);
}

#[test]
fn deterministic_output() {
    let mut instructions1 = vec![
        CoreOp::FieldType("F1".into(), "User".into()),
        CoreOp::Return("M1".into(), "Promise<User>".into()),
    ];
    let mut instructions2 = instructions1.clone();
    let cfg = aliases(&[("User", "$uid")]);
    apply_type_aliases_to_ir(&mut instructions1, &cfg);
    apply_type_aliases_to_ir(&mut instructions2, &cfg);
    assert_eq!(instructions1, instructions2);
}