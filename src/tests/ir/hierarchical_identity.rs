// Stable-identity contracts for the first checked hierarchical projection slice.

use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{
    HierarchicalProjectionError, ProjectionIdentityKind, try_ir_to_hierarchical,
};
use crate::ir::opcodes::CoreOp;

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "identity.rs".to_string(),
        instructions,
        version: 1,
    }
}

#[test]
fn params_and_returns_follow_method_identity_across_legal_reordering() {
    let ir = compiled(vec![
        CoreOp::Param("M2".into(), "P2".into(), "$n".into(), "count".into()),
        CoreOp::Return("M1".into(), "$s".into()),
        CoreOp::DefMethod("C2".into(), "M2".into(), "second".into()),
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "first".into()),
        CoreOp::Param("M1".into(), "P1".into(), "$b".into(), "enabled".into()),
        CoreOp::DefClass("C2".into(), "Second".into()),
        CoreOp::Param("M1".into(), "P3".into(), "$s".into(), "label".into()),
        CoreOp::Return("M2".into(), "$v".into()),
    ]);

    let hierarchy = try_ir_to_hierarchical(&ir).expect("reordered identities are valid");
    let first = hierarchy
        .classes
        .iter()
        .find(|class| class.id == "C1")
        .unwrap();
    let second = hierarchy
        .classes
        .iter()
        .find(|class| class.id == "C2")
        .unwrap();
    let first_method = first
        .methods
        .iter()
        .find(|method| method.id == "M1")
        .unwrap();
    let second_method = second
        .methods
        .iter()
        .find(|method| method.id == "M2")
        .unwrap();

    assert_eq!(
        first_method.params,
        vec![
            vec!["P1".to_string(), "$b".to_string(), "enabled".to_string()],
            vec!["P3".to_string(), "$s".to_string(), "label".to_string()],
        ]
    );
    assert_eq!(first_method.return_type.as_deref(), Some("$s"));
    assert_eq!(
        second_method.params,
        vec![vec![
            "P2".to_string(),
            "$n".to_string(),
            "count".to_string()
        ]]
    );
    assert_eq!(second_method.return_type.as_deref(), Some("$v"));
    assert!(hierarchy.classes.iter().all(|class| !class.synthetic));
}

#[test]
fn typed_projection_preserves_existing_serialized_string_shape() {
    let hierarchy = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "one".into()),
        CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "value".into()),
        CoreOp::Return("M1".into(), "$v".into()),
    ]))
    .expect("identity graph is valid");
    let serialized = serde_json::to_value(hierarchy).expect("hierarchy serializes");

    assert_eq!(serialized["c"][0]["n"], "C1");
    assert_eq!(serialized["c"][0]["m"][0]["n"], "M1");
    assert_eq!(serialized["c"][0]["m"][0]["p"][0][0], "P1");
    assert_eq!(serialized["c"][0]["m"][0]["r"], "$v");
}

#[test]
fn duplicate_class_identity_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefClass("C1".into(), "Duplicate".into()),
    ]))
    .expect_err("duplicate class must fail");

    assert_eq!(
        error,
        HierarchicalProjectionError::DuplicateIdentity {
            operation: "DEF_C",
            kind: ProjectionIdentityKind::Class,
            id: "C1".into(),
            owner: None,
            first_instruction: 0,
            duplicate_instruction: 1,
        }
    );
}

#[test]
fn empty_class_identity_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![CoreOp::DefClass(
        String::new(),
        "MissingId".into(),
    )]))
    .expect_err("empty class identity must fail");

    assert_eq!(
        error,
        HierarchicalProjectionError::InvalidIdentity {
            operation: "DEF_C",
            kind: ProjectionIdentityKind::Class,
            id: String::new(),
            owner: None,
            instruction: 0,
            reason: "identity must not be empty",
        }
    );
}

#[test]
fn duplicate_method_identity_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "one".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "two".into()),
    ]))
    .expect_err("duplicate method must fail");

    assert!(matches!(
        error,
        HierarchicalProjectionError::DuplicateIdentity {
            operation: "DEF_M",
            kind: ProjectionIdentityKind::Method,
            ref id,
            owner: Some(ref owner),
            first_instruction: 1,
            duplicate_instruction: 2,
        } if id == "M1" && owner == "C1"
    ));
}

#[test]
fn unresolved_method_owner_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![CoreOp::DefMethod(
        "C404".into(),
        "M1".into(),
        "orphan".into(),
    )]))
    .expect_err("unknown owner must fail");

    assert!(matches!(
        error,
        HierarchicalProjectionError::UnresolvedIdentity {
            operation: "DEF_M owner",
            expected: ProjectionIdentityKind::Class,
            ref id,
            instruction: 0,
        } if id == "C404"
    ));
}

#[test]
fn method_identity_used_as_method_owner_is_kind_mismatch() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "owner".into()),
        CoreOp::DefMethod("M1".into(), "M2".into(), "child".into()),
    ]))
    .expect_err("method identity cannot own another method");

    assert_eq!(
        error,
        HierarchicalProjectionError::KindMismatch {
            operation: "DEF_M owner",
            id: "M1".into(),
            expected: ProjectionIdentityKind::Class,
            actual: ProjectionIdentityKind::Method,
            instruction: 2,
        }
    );
}

#[test]
fn unresolved_param_target_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::Param("M404".into(), "P1".into(), "$s".into(), "value".into()),
    ]))
    .expect_err("unknown method target must fail");

    assert!(matches!(
        error,
        HierarchicalProjectionError::UnresolvedIdentity {
            operation: "SIG",
            expected: ProjectionIdentityKind::Method,
            ref id,
            instruction: 1,
        } if id == "M404"
    ));
}

#[test]
fn class_identity_used_as_return_target_is_kind_mismatch() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::Return("C1".into(), "$v".into()),
    ]))
    .expect_err("class identity is not a method target");

    assert_eq!(
        error,
        HierarchicalProjectionError::KindMismatch {
            operation: "RET",
            id: "C1".into(),
            expected: ProjectionIdentityKind::Method,
            actual: ProjectionIdentityKind::Class,
            instruction: 1,
        }
    );
}

#[test]
fn duplicate_parameter_identity_within_method_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "one".into()),
        CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "first".into()),
        CoreOp::Param("M1".into(), "P1".into(), "$n".into(), "second".into()),
    ]))
    .expect_err("duplicate parameter identity must fail");

    assert!(matches!(
        error,
        HierarchicalProjectionError::DuplicateIdentity {
            operation: "SIG",
            kind: ProjectionIdentityKind::Parameter,
            ref id,
            owner: Some(ref owner),
            first_instruction: 2,
            duplicate_instruction: 3,
        } if id == "P1" && owner == "M1"
    ));
}

#[test]
fn empty_parameter_identity_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "one".into()),
        CoreOp::Param("M1".into(), String::new(), "$s".into(), "value".into()),
    ]))
    .expect_err("empty parameter identity must fail");

    assert_eq!(
        error,
        HierarchicalProjectionError::InvalidIdentity {
            operation: "SIG",
            kind: ProjectionIdentityKind::Parameter,
            id: String::new(),
            owner: Some("M1".into()),
            instruction: 2,
            reason: "identity must not be empty",
        }
    );
}

#[test]
fn duplicate_return_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "one".into()),
        CoreOp::Return("M1".into(), "$s".into()),
        CoreOp::Return("M1".into(), "$n".into()),
    ]))
    .expect_err("return is singular per method");

    assert_eq!(
        error,
        HierarchicalProjectionError::DuplicateReturn {
            method_id: "M1".into(),
            first_instruction: 2,
            duplicate_instruction: 3,
        }
    );
}
