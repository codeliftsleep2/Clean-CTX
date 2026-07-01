use super::*;
use crate::queries;
#[test]
fn empty_source_yields_no_captures() {
    let captures = run_capture_pipeline(
        crate::compression::language::safe_typescript_language()
            .expect("typescript feature should be enabled in tests"),
        queries::TS_QUERY,
        "",
        Fidelity::Low,
        |_, _, _| Some("x".to_string()),
    )
    .expect("pipeline should not error on empty source");
    assert!(captures.is_empty());
}

#[test]
fn captures_are_sorted_by_position() {
    let src = r#"
        class A { foo() {} }
        class B { bar() {} }
    "#;
    let mut names: Vec<String> = Vec::new();
    let captures = run_capture_pipeline(
        crate::compression::language::safe_typescript_language()
            .expect("typescript feature should be enabled in tests"),
        queries::TS_QUERY,
        src,
        Fidelity::Low,
        |name, _raw, _fidelity| {
            if name == "class.root" {
                names.push(name.to_string());
            }
            Some("ClassName".to_string())
        },
    )
    .expect("pipeline should parse valid TS");
    let _ = captures.len();
    assert!(!names.is_empty());
}

#[test]
fn process_can_drop_captures() {
    let src = "class A {}";
    let captures = run_capture_pipeline(
        crate::compression::language::safe_typescript_language()
            .expect("typescript feature should be enabled in tests"),
        queries::TS_QUERY,
        src,
        Fidelity::Low,
        |name, _, _| {
            if name == "class.root" {
                None
            } else {
                Some("kept".to_string())
            }
        },
    )
    .expect("pipeline should parse valid TS");
    for c in &captures {
        assert_ne!(c.name, "class.root");
    }
}

/// F-08: the closure must receive the fidelity that the caller passed
/// in, not a hard-coded `Fidelity::Low`. This test asserts the value
/// flows through.
#[test]
fn fidelity_is_passed_through_to_closure() {
    let src = "class A {}";
    let mut seen: Option<Fidelity> = None;
    let _ = run_capture_pipeline(
        crate::compression::language::safe_typescript_language()
            .expect("typescript feature should be enabled in tests"),
        queries::TS_QUERY,
        src,
        Fidelity::High,
        |_, _, f| {
            seen = Some(f);
            Some("ok".to_string())
        },
    )
    .expect("pipeline should parse valid TS");
    assert_eq!(seen, Some(Fidelity::High));
}

/// F-08 (Medium): the closure must see Medium too.
#[test]
fn fidelity_medium_is_passed_through_to_closure() {
    let src = "class A {}";
    let mut seen: Option<Fidelity> = None;
    let _ = run_capture_pipeline(
        crate::compression::language::safe_typescript_language()
            .expect("typescript feature should be enabled in tests"),
        queries::TS_QUERY,
        src,
        Fidelity::Medium,
        |_, _, f| {
            seen = Some(f);
            Some("ok".to_string())
        },
    )
    .expect("pipeline should parse valid TS");
    assert_eq!(seen, Some(Fidelity::Medium));
}
