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

#[test]
fn issue_136_same_owner_overloads_require_signature_selectors() {
    let table = UnitTable::from_instructions(&[
        CoreOp::DefClass("C1".into(), "Service".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "Run".into()),
        CoreOp::Param("M1".into(), "P1".into(), "int".into(), "value".into()),
        CoreOp::Body("M1".into(), "{ one(); }".into(), Some(10), Some(20)),
        CoreOp::DefMethod("C1".into(), "M2".into(), "Run".into()),
        CoreOp::Param("M2".into(), "P2".into(), "string".into(), "value".into()),
        CoreOp::Body("M2".into(), "{ two(); }".into(), Some(30), Some(40)),
        CoreOp::DefMethod("C1".into(), "M3".into(), "Run".into()),
        CoreOp::Param("M3".into(), "P3".into(), "bool".into(), "value".into()),
        CoreOp::Body("M3".into(), "{ three(); }".into(), Some(50), Some(62)),
    ]);

    assert_eq!(table.resolve("Service.Run(int)").unwrap().method_id, "M1");
    assert_eq!(
        table.resolve("Service.Run(string)").unwrap().method_id,
        "M2"
    );
    assert_eq!(table.resolve("Service.Run(bool)").unwrap().method_id, "M3");
    assert!(matches!(
        table.resolve("Service.Run"),
        Err(LocateError::Ambiguous { candidates, .. }) if candidates.len() == 3
    ));
}

#[test]
fn issue_136_signature_collision_also_fails_closed() {
    let table = UnitTable::from_instructions(&[
        CoreOp::DefClass("C1".into(), "Service".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "Run".into()),
        CoreOp::Param("M1".into(), "P1".into(), "string".into(), "a".into()),
        CoreOp::Body("M1".into(), "{}".into(), Some(10), Some(12)),
        CoreOp::DefMethod("C1".into(), "M2".into(), "Run".into()),
        CoreOp::Param("M2".into(), "P2".into(), "string".into(), "b".into()),
        CoreOp::Body("M2".into(), "{}".into(), Some(20), Some(22)),
    ]);
    assert!(matches!(
        table.resolve("Service.Run(string)"),
        Err(LocateError::Ambiguous { candidates, .. }) if candidates.len() == 2
    ));
}

#[test]
fn issue_136_generic_method_accepts_bare_name_signature_alias_when_unique() {
    let table = UnitTable::from_instructions(&[
        CoreOp::DefClass("C1".into(), "Service".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "Get<T>".into()),
        CoreOp::Param("M1".into(), "P1".into(), "T".into(), "value".into()),
        CoreOp::Body("M1".into(), "{}".into(), Some(10), Some(12)),
    ]);
    assert_eq!(table.resolve("Service.Get(T)").unwrap().method_id, "M1");
    assert_eq!(table.resolve("Service.Get").unwrap().method_id, "M1");
}

#[test]
fn issue_136_base_generic_alias_refuses_collision_with_non_generic_member() {
    let table = UnitTable::from_instructions(&[
        CoreOp::DefClass("C1".into(), "Service".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "Get<T>".into()),
        CoreOp::Param("M1".into(), "P1".into(), "T".into(), "value".into()),
        CoreOp::Body("M1".into(), "{}".into(), Some(10), Some(12)),
        CoreOp::DefMethod("C1".into(), "M2".into(), "Get".into()),
        CoreOp::Param("M2".into(), "P2".into(), "T".into(), "value".into()),
        CoreOp::Body("M2".into(), "{}".into(), Some(20), Some(22)),
    ]);
    assert!(matches!(
        table.resolve("Service.Get(T)"),
        Err(LocateError::Ambiguous { .. })
    ));
}
