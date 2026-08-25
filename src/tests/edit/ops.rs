// src/tests/edit/ops.rs
//
// Unit tests for the EditOperation model: serde tagging, accessors,
// and the batch bound constant.

use crate::edit::ops::EditOperation;

#[test]
fn replace_body_serde_round_trip() {
    let op: EditOperation = serde_json::from_value(serde_json::json!({
        "type": "replace_body",
        "target": "UserService.processOrder",
        "expectedOldText": "{\n  return true;\n}",
        "newText": "{\n  return false;\n}"
    }))
    .expect("tagged json must parse");
    match &op {
        EditOperation::ReplaceBody {
            target,
            expected_old_text,
            new_text,
        } => {
            assert_eq!(target, "UserService.processOrder");
            assert_eq!(expected_old_text, "{\n  return true;\n}");
            assert_eq!(new_text, "{\n  return false;\n}");
        }
        other => panic!("wrong variant: {other:?}"),
    }
    assert_eq!(op.kind(), "replace_body");
    assert_eq!(op.target(), "UserService.processOrder");
}

#[test]
fn insert_delete_variants_parse_with_snake_case_tags() {
    let insert_after: EditOperation = serde_json::from_value(serde_json::json!({
        "type": "insert_after", "anchor": "Svc.a", "unitText": "\n  b() {}"
    }))
    .unwrap();
    assert_eq!(insert_after.kind(), "insert_after");
    assert_eq!(insert_after.target(), "Svc.a");

    let insert_before: EditOperation = serde_json::from_value(serde_json::json!({
        "type": "insert_before", "anchor": "Svc.a", "unitText": "pre() {}"
    }))
    .unwrap();
    assert_eq!(insert_before.kind(), "insert_before");

    let del: EditOperation = serde_json::from_value(serde_json::json!({
        "type": "delete", "target": "S.m", "expectedOldText": "{}"
    }))
    .unwrap();
    assert_eq!(del.kind(), "delete");
    assert_eq!(del.target(), "S.m");
}

#[test]
fn unknown_tag_is_rejected() {
    assert!(
        serde_json::from_value::<EditOperation>(serde_json::json!({
            "type": "rename_symbol", "target": "x"
        }))
        .is_err()
    );
}
