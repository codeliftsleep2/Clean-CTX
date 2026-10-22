use super::{Resolution, TypeScriptAuthority};
use crate::ir::invocation_evidence::SourceSpan;

fn capture(source: &str) -> TypeScriptAuthority {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    std::fs::write(&file, source).unwrap();
    TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()])
}
fn resolved(capture: &TypeScriptAuthority, start: usize) -> &super::DeclarationId {
    match &capture
        .reference_sites
        .iter()
        .find(|site| site.0 == start)
        .unwrap()
        .2
    {
        Resolution::Resolved(id) => id,
        other => panic!("exact callable scope identity required: {other:?}"),
    }
}

#[test]
fn issue_162_single_arrow_parameter_shadows_import_without_escaping_its_owner() {
    let source = "import { item } from './foreign'; const f = item => use(item);";
    let capture = capture(source);
    let start = source.rfind("use(item)").unwrap() + 4;
    let id = resolved(&capture, start);
    assert_eq!(id.kind, "arrow_parameter");
    assert_eq!(id.source, capture.source);
    assert_eq!(&source[id.start..id.end], "item");
    assert!(capture.declarations.contains(id));
    assert!(
        crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
            "app.ts",
            source,
            SourceSpan::new(start, start + 4).unwrap()
        )
        .is_none()
    );
}

#[test]
fn issue_162_single_arrow_parameter_does_not_replace_an_outer_binding_after_the_arrow() {
    let source = "const item = 1; const f = item => use(item); use(item);";
    let capture = capture(source);
    let inner = resolved(&capture, source.find("use(item)").unwrap() + 4);
    let outer = resolved(&capture, source.rfind("use(item)").unwrap() + 4);
    assert_eq!(inner.kind, "arrow_parameter");
    assert_eq!(outer.kind, "variable_declarator");
    assert_ne!(inner.start, outer.start);
}

#[test]
fn issue_162_named_function_expression_self_binding_is_private_to_its_expression() {
    let source = "const f = function Inner() { return Inner; }; use(Inner);";
    let capture = capture(source);
    let inside = source.find("return Inner").unwrap() + 7;
    let outside = source.rfind("Inner").unwrap();
    assert_eq!(resolved(&capture, inside).kind, "function_expression");
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == outside && matches!(site.2, Resolution::Unresolved(_)))
    );
    let declaration = source.find("Inner").unwrap();
    assert!(
        !capture
            .reference_sites
            .iter()
            .any(|site| site.0 == declaration)
    );
}

#[test]
fn issue_162_function_expression_parameter_takes_precedence_over_its_self_name() {
    let source = "const f = function Inner(Inner: unknown) { return Inner; };";
    let capture = capture(source);
    let id = resolved(&capture, source.rfind("Inner").unwrap());
    assert_eq!(id.kind, "required_parameter");
}
