// src/tests/edit/apply.rs
//
// Unit tests for the splice engine and the syntax gate: verification,
// byte-exact replacement/deletion/insertion, overlap rejection, bounded
// mismatch payloads, and tree-sitter pre-commit rejection.

use crate::edit::apply::{self, EditError};
use crate::edit::locate::UnitTable;
use crate::ir::opcodes::CoreOp;

/// One method whose body occupies bytes 20..38 of the source.
fn fixture() -> (String, UnitTable) {
    let source = "class Svc {\n  run() {\n    return 1;\n  }\n}\n";
    let body_start = source.find("{\n    return 1;").unwrap();
    let body_end = source.find("\n  }\n}").unwrap() + "\n  }".len();
    // Real IR shape: DefClass → DefMethod → spanned Body.
    let units = UnitTable::from_instructions(&[
        CoreOp::DefClass("C1".into(), "Svc".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "run".into()),
        CoreOp::Body(
            "M1".into(),
            source[body_start..body_end].to_string(),
            Some(body_start as u64),
            Some(body_end as u64),
        ),
    ]);
    (source.to_string(), units)
}

#[test]
fn replace_body_splices_expected_range() {
    let (source, units) = fixture();
    let old = units.resolve("Svc.run").unwrap().text.clone();
    let report = apply::apply(
        &source,
        &units,
        &[crate::edit::ops::EditOperation::ReplaceBody {
            target: "Svc.run".into(),
            expected_old_text: old.clone(),
            new_text: "{\n    return 42;\n  }".into(),
        }],
    )
    .unwrap();
    assert_eq!(report.operations.len(), 1);
    assert_eq!(report.operations[0].byte_delta, 1);
    assert!(report.new_source.contains("return 42;"));
    // Everything outside the spliced range is untouched.
    assert!(report.new_source.starts_with("class Svc {\n  run() "));
}

#[test]
fn mismatch_rejects_with_bounded_payload() {
    let (source, units) = fixture();
    let err = apply::apply(
        &source,
        &units,
        &[crate::edit::ops::EditOperation::ReplaceBody {
            target: "Svc.run".into(),
            expected_old_text: "{\n    STALE;\n  }".into(),
            new_text: "{}".into(),
        }],
    )
    .unwrap_err();
    match &err {
        EditError::Mismatch {
            actual_snippet,
            actual_len,
            ..
        } => {
            assert!(actual_snippet.contains("return 1"));
            // Fixture body slice is "{\n    return 1;\n  }" = 19 bytes.
            assert_eq!(*actual_len, 19);
            let data = err.structured();
            assert_eq!(data["kind"], "unit_mismatch");
        }
        other => panic!("expected mismatch, got {other:?}"),
    }
    // Nothing was applied.
    assert!(source.contains("return 1;"));
}

#[test]
fn delete_removes_unit_span() {
    let (source, units) = fixture();
    let old = units.resolve("M1").unwrap().text.clone();
    let report = apply::apply(
        &source,
        &units,
        &[crate::edit::ops::EditOperation::Delete {
            target: "M1".into(),
            expected_old_text: old,
        }],
    )
    .unwrap();
    assert_eq!(report.operations[0].kind, "delete");
    assert!(!report.new_source.contains("return 1;"));
    // Deleting [20..39) leaves the pre-body prefix + the trailing
    // "\n}\n" — note the space after `run()` is preserved.
    assert_eq!(report.new_source, "class Svc {\n  run() \n}\n");
}

#[test]
fn insert_after_lands_at_unit_end() {
    let (source, units) = fixture();
    let report = apply::apply(
        &source,
        &units,
        &[crate::edit::ops::EditOperation::InsertAfter {
            anchor: "Svc.run".into(),
            unit_text: "\n\n  helper() {}".into(),
        }],
    )
    .unwrap();
    assert!(report.new_source.contains("helper() {}"));
    // Insertion lands at the anchor's end byte (39) and spans exactly the
    // inserted text's length.
    let outcome = &report.operations[0];
    assert_eq!(outcome.start_byte, 39);
    assert_eq!(
        outcome.end_byte - outcome.start_byte,
        "\n\n  helper() {}".len() as u64
    );
}

#[test]
fn overlapping_replacements_are_rejected() {
    let source = "class A {\n  x() {\n    return 1;\n  }\n  y() {\n    return 2;\n  }\n}\n";
    // Real captures never intersect, so hand-craft two Body ops whose
    // spans deliberately overlap ([16..40) vs [35..60)) to exercise the
    // planner's disjointness guard. Texts are the real slices so the
    // expected-text verification passes before overlap is checked.
    let units = UnitTable::from_instructions(&[
        CoreOp::DefClass("C0".into(), "A".into()),
        CoreOp::DefMethod("C0".into(), "M1".into(), "x".into()),
        CoreOp::DefMethod("C0".into(), "M2".into(), "y".into()),
        CoreOp::Body("M1".into(), source[16..40].to_string(), Some(16), Some(40)),
        CoreOp::Body("M2".into(), source[35..60].to_string(), Some(35), Some(60)),
    ]);
    let mk = |t: &str, o: &str| crate::edit::ops::EditOperation::ReplaceBody {
        target: t.into(),
        expected_old_text: o.into(),
        new_text: "{}".into(),
    };
    let err = apply::apply(
        source,
        &units,
        &[mk("A.x", &source[16..40]), mk("A.y", &source[35..60])],
    )
    .unwrap_err();
    assert!(
        matches!(err, EditError::Overlap { .. }),
        "expected overlap rejection, got {err:?}"
    );
}

// ── Syntax gate ──────────────────────────────────────────────────────

#[test]
fn syntax_gate_accepts_valid_typescript() {
    let src = "export class S {\n  run(): number {\n    return 1;\n  }\n}\n";
    assert!(apply::verify_syntax(src, "ts").is_ok());
}

#[test]
fn syntax_gate_rejects_malformed_splice() {
    // Simulates a bad splice: unbalanced braces.
    let broken = "export class S {\n  run(): number {\n    return 1;\n}\n";
    let err = apply::verify_syntax(broken, "ts").unwrap_err();
    match err {
        EditError::SyntaxGateRejected { line, message, .. } => {
            assert!(line >= 1);
            assert!(message.contains("parse error"));
        }
        other => panic!("expected syntax rejection, got {other:?}"),
    }
}

#[test]
fn syntax_gate_rejects_unsupported_extension() {
    let err = apply::verify_syntax("whatever", "xy").unwrap_err();
    assert!(matches!(err, EditError::UnsupportedExtension(_)));
}
