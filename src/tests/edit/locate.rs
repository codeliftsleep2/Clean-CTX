// src/tests/edit/locate.rs
//
// Unit tests for unit-table construction and target resolution:
// qualified-name lookup, fingerprint disambiguation of same-named
// methods, method-id keys, and span-less exclusion.

use crate::edit::locate::{LocateError, UnitTable};
use crate::ir::opcodes::CoreOp;

fn sample_instructions() -> Vec<CoreOp> {
    vec![
        CoreOp::DefClass("C1".into(), "SampleService".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "processOrder".into()),
        CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "order".into()),
        CoreOp::Body(
            "M1".into(),
            "{\n  return true;\n}".into(),
            Some(40),
            Some(58),
        ),
        // Second class with a SAME-NAMED method — bare-name lookup on
        // "processOrder" must be ambiguous, not silently first-match.
        CoreOp::DefClass("C2".into(), "OtherService".into()),
        CoreOp::DefMethod("C2".into(), "M2".into(), "processOrder".into()),
        CoreOp::Param("M2".into(), "P1".into(), "$b".into(), "flag".into()),
        CoreOp::Body(
            "M2".into(),
            "{\n  return false;\n}".into(),
            Some(200),
            Some(219),
        ),
        // Span-less body (legacy wire state) must be excluded.
        CoreOp::DefClass("C3".into(), "LegacyService".into()),
        CoreOp::DefMethod("C3".into(), "M3".into(), "legacy".into()),
        CoreOp::Body("M3".into(), "{}".into(), None, None),
    ]
}

#[test]
fn qualified_name_resolves_unique_unit() {
    let table = UnitTable::from_instructions(&sample_instructions());
    let rec = table.resolve("SampleService.processOrder").unwrap();
    assert_eq!(rec.method_id, "M1");
    assert_eq!(rec.start_byte, 40);
    assert_eq!(rec.end_byte, 58);
    assert_eq!(rec.text, "{\n  return true;\n}");
    assert_eq!(rec.fingerprint, "SampleService($s)");
}

#[test]
fn duplicate_bare_names_are_ambiguous() {
    let table = UnitTable::from_instructions(&sample_instructions());
    match table.resolve("processOrder") {
        Err(LocateError::Ambiguous { candidates, .. }) => {
            assert_eq!(candidates.len(), 2);
            assert!(candidates[0].contains("SampleService"));
            assert!(candidates[1].contains("OtherService"));
        }
        other => panic!("expected ambiguity, got {other:?}"),
    }
}

#[test]
fn method_id_is_a_valid_key() {
    let table = UnitTable::from_instructions(&sample_instructions());
    let rec = table.resolve("M2").unwrap();
    assert_eq!(rec.qualified_name, "OtherService.processOrder");
}

#[test]
fn unknown_target_is_not_found() {
    let table = UnitTable::from_instructions(&sample_instructions());
    assert!(matches!(
        table.resolve("MissingService.nope"),
        Err(LocateError::NotFound(_))
    ));
}

#[test]
fn spanless_bodies_are_excluded() {
    let table = UnitTable::from_instructions(&sample_instructions());
    assert_eq!(table.len(), 2, "span-less M3 must not appear");
    assert!(table.resolve("LegacyService.legacy").is_err());
}

#[test]
fn empty_instructions_yield_empty_table() {
    let table = UnitTable::from_instructions(&[]);
    assert!(table.is_empty());
}
