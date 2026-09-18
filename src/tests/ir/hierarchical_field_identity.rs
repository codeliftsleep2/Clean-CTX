// Stable field-identity contracts for the second checked projection slice.

use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{
    HierarchicalProjectionError, ProjectionIdentityKind, try_ir_to_hierarchical,
};
use crate::ir::opcodes::CoreOp;

fn compiled(instructions: Vec<CoreOp>) -> CompiledIR {
    CompiledIR {
        file_id: "field_identity.rs".to_string(),
        instructions,
        version: 1,
    }
}

#[test]
fn fields_and_types_follow_identity_across_legal_reordering() {
    let hierarchy = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::FieldType("F2".into(), "$n".into()),
        CoreOp::DefField("C2".into(), "F2".into(), "count".into()),
        CoreOp::DefClass("C1".into(), "First".into()),
        CoreOp::DefField("C1".into(), "F1".into(), "label".into()),
        CoreOp::DefClass("C2".into(), "Second".into()),
        CoreOp::FieldType("F1".into(), "$s".into()),
    ]))
    .expect("reordered field identities are valid");

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

    assert_eq!(first.fields[0].id, "F1");
    assert_eq!(first.fields[0].field_type.as_deref(), Some("$s"));
    assert_eq!(second.fields[0].id, "F2");
    assert_eq!(second.fields[0].field_type.as_deref(), Some("$n"));
    assert!(hierarchy.classes.iter().all(|class| !class.synthetic));
}

#[test]
fn fields_declared_before_their_class_preserve_definition_order() {
    let hierarchy = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefField("C1".into(), "F1".into(), "first".into()),
        CoreOp::DefField("C1".into(), "F2".into(), "second".into()),
        CoreOp::DefClass("C1".into(), "Owner".into()),
    ]))
    .expect("pre-definition fields are valid when their owner resolves");

    let ids: Vec<&str> = hierarchy.classes[0]
        .fields
        .iter()
        .map(|field| field.id.as_str())
        .collect();
    assert_eq!(ids, vec!["F1", "F2"]);
}

#[test]
fn typed_field_projection_preserves_serialized_string_shape() {
    let hierarchy = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefField("C1".into(), "F1".into(), "value".into()),
        CoreOp::FieldType("F1".into(), "$s".into()),
    ]))
    .expect("field identity graph is valid");
    let serialized = serde_json::to_value(hierarchy).expect("hierarchy serializes");

    assert_eq!(serialized["c"][0]["n"], "C1");
    assert_eq!(serialized["c"][0]["f"][0]["n"], "F1");
    assert_eq!(serialized["c"][0]["f"][0]["t"], "$s");
}

#[test]
fn duplicate_field_identity_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefField("C1".into(), "F1".into(), "first".into()),
        CoreOp::DefField("C1".into(), "F1".into(), "second".into()),
    ]))
    .expect_err("duplicate field must fail");

    assert_eq!(
        error,
        HierarchicalProjectionError::DuplicateIdentity {
            operation: "DEF_F",
            kind: ProjectionIdentityKind::Field,
            id: "F1".into(),
            owner: Some("C1".into()),
            first_instruction: 1,
            duplicate_instruction: 2,
        }
    );
}

#[test]
fn empty_field_identity_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefField("C1".into(), String::new(), "value".into()),
    ]))
    .expect_err("empty field identity must fail");

    assert_eq!(
        error,
        HierarchicalProjectionError::InvalidIdentity {
            operation: "DEF_F",
            kind: ProjectionIdentityKind::Field,
            id: String::new(),
            owner: Some("C1".into()),
            instruction: 1,
            reason: "identity must not be empty",
        }
    );
}

#[test]
fn unresolved_field_owner_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![CoreOp::DefField(
        "C404".into(),
        "F1".into(),
        "orphan".into(),
    )]))
    .expect_err("unknown owner must fail");

    assert_eq!(
        error,
        HierarchicalProjectionError::UnresolvedIdentity {
            operation: "DEF_F owner",
            expected: ProjectionIdentityKind::Class,
            id: "C404".into(),
            instruction: 0,
        }
    );
}

#[test]
fn method_identity_used_as_field_owner_is_kind_mismatch() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "method".into()),
        CoreOp::DefField("M1".into(), "F1".into(), "orphan".into()),
    ]))
    .expect_err("method identity cannot own a field");

    assert_eq!(
        error,
        HierarchicalProjectionError::KindMismatch {
            operation: "DEF_F owner",
            id: "M1".into(),
            expected: ProjectionIdentityKind::Class,
            actual: ProjectionIdentityKind::Method,
            instruction: 2,
        }
    );
}

#[test]
fn class_identity_used_as_field_identity_is_kind_mismatch() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Conflicting".into()),
        CoreOp::DefClass("C2".into(), "Owner".into()),
        CoreOp::DefField("C2".into(), "C1".into(), "value".into()),
    ]))
    .expect_err("a class identity cannot also define a field");

    assert_eq!(
        error,
        HierarchicalProjectionError::KindMismatch {
            operation: "DEF_F identity",
            id: "C1".into(),
            expected: ProjectionIdentityKind::Field,
            actual: ProjectionIdentityKind::Class,
            instruction: 2,
        }
    );
}

#[test]
fn unresolved_field_type_target_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![CoreOp::FieldType(
        "F404".into(),
        "$s".into(),
    )]))
    .expect_err("unknown field target must fail");

    assert_eq!(
        error,
        HierarchicalProjectionError::UnresolvedIdentity {
            operation: "FIELD_T",
            expected: ProjectionIdentityKind::Field,
            id: "F404".into(),
            instruction: 0,
        }
    );
}

#[test]
fn empty_field_type_target_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![CoreOp::FieldType(
        String::new(),
        "$s".into(),
    )]))
    .expect_err("empty field target must fail");

    assert_eq!(
        error,
        HierarchicalProjectionError::InvalidIdentity {
            operation: "FIELD_T",
            kind: ProjectionIdentityKind::Field,
            id: String::new(),
            owner: None,
            instruction: 0,
            reason: "identity must not be empty",
        }
    );
}

#[test]
fn class_identity_used_as_field_type_target_is_kind_mismatch() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::FieldType("C1".into(), "$s".into()),
    ]))
    .expect_err("class identity is not a field target");

    assert_eq!(
        error,
        HierarchicalProjectionError::KindMismatch {
            operation: "FIELD_T",
            id: "C1".into(),
            expected: ProjectionIdentityKind::Field,
            actual: ProjectionIdentityKind::Class,
            instruction: 1,
        }
    );
}

#[test]
fn duplicate_field_type_is_structured_failure() {
    let error = try_ir_to_hierarchical(&compiled(vec![
        CoreOp::DefClass("C1".into(), "Owner".into()),
        CoreOp::DefField("C1".into(), "F1".into(), "value".into()),
        CoreOp::FieldType("F1".into(), "$s".into()),
        CoreOp::FieldType("F1".into(), "$n".into()),
    ]))
    .expect_err("field type is singular per field");

    assert_eq!(
        error,
        HierarchicalProjectionError::DuplicateFieldType {
            field_id: "F1".into(),
            first_instruction: 2,
            duplicate_instruction: 3,
        }
    );
}
