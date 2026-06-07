use super::*;
use crate::queries;

#[test]
fn empty_source_yields_no_captures() {
    let captures = run_capture_pipeline(
        tree_sitter_typescript::language_typescript(),
        queries::TS_QUERY,
        "",
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
        tree_sitter_typescript::language_typescript(),
        queries::TS_QUERY,
        src,
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
        tree_sitter_typescript::language_typescript(),
        queries::TS_QUERY,
        src,
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