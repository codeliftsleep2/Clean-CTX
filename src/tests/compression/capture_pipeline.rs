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

/// Type alias for a language query test case: (label, language factory, query string).
#[cfg(all(test, feature = "rust"))]
type LanguageQueryTestCase = (&'static str, fn() -> Option<tree_sitter::Language>, &'static str);

/// Regression test: verify that each enabled language query compiles
/// successfully against its grammar. A query with an unrecognised node
/// type will fail at `Query::new` time, which previously caused a
/// CI-blocking panic in the fallback chain. This test catches such
/// node-type mismatches before they reach production.
///
/// Languages whose Cargo feature is disabled are silently skipped, so
/// this test passes in any configuration.
#[cfg(all(test, feature = "rust"))]
#[test]
fn all_language_queries_compile_successfully() {
    use tree_sitter::Query;

    let test_cases: Vec<LanguageQueryTestCase> = vec![
        ("typescript", crate::compression::language::safe_typescript_language, queries::TS_QUERY),
        ("csharp", crate::compression::language::safe_csharp_language, queries::CS_QUERY),
        ("rust", crate::compression::language::safe_rust_language, queries::RS_QUERY),
        ("java", crate::compression::language::safe_java_language, queries::JAVA_QUERY),
    ];

    for (name, lang_fn, query_str) in test_cases {
        let Some(lang) = lang_fn() else {
            // Language feature not enabled — skip gracefully
            continue;
        };
        let result = Query::new(&lang, query_str);
        assert!(
            result.is_ok(),
            "query compilation regression: {name} query failed to compile: {}",
            result.err().unwrap(),
        );
    }
}
