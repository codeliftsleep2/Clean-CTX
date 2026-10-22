use super::{Resolution, TypeScriptAuthority};
use crate::ir::invocation_evidence::SourceSpan;

fn capture(source: &str) -> TypeScriptAuthority {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    std::fs::write(&file, source).unwrap();
    TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()])
}

#[test]
fn issue_162_unproved_lexical_collision_does_not_fall_back_to_partial_module_bindings() {
    let source = "const item = 1; for (var item of values) {} use(item);";
    let capture = capture(source);
    let reference = source.rfind("use(item)").unwrap() + 4;
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == reference && matches!(site.2, Resolution::Unresolved(_)))
    );
}

#[test]
fn issue_162_let_const_iteration_bindings_do_not_escape_their_loop() {
    for kind in ["let", "const"] {
        for operator in ["in", "of"] {
            let source = format!("for ({kind} item {operator} values) {{ use(item); }} use(item);");
            let capture = capture(&source);
            let inside = source.find("use(item)").unwrap() + 4;
            let outside = source.rfind("use(item)").unwrap() + 4;
            let (_, _, Resolution::Resolved(id)) = capture
                .reference_sites
                .iter()
                .find(|site| site.0 == inside)
                .unwrap()
            else {
                panic!("loop lexical identity required");
            };
            assert_eq!(id.kind, "loop_binding");
            assert_eq!(&source[id.start..id.end], "item");
            assert!(
                capture
                    .reference_sites
                    .iter()
                    .any(|site| site.0 == outside && matches!(site.2, Resolution::Unresolved(_)))
            );
            let span = SourceSpan::new(inside, inside + 4).unwrap();
            assert!(
                crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
                    "app.ts", &source, span
                )
                .is_none()
            );
        }
    }
}

#[test]
fn issue_162_var_iteration_binding_survives_loop_but_not_function_owner() {
    let source = "function f() { for (var item of values) {} use(item); } use(item);";
    let capture = capture(source);
    let inside = source.find("use(item)").unwrap() + 4;
    let outside = source.rfind("use(item)").unwrap() + 4;
    let (_, _, Resolution::Resolved(id)) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == inside)
        .unwrap()
    else {
        panic!("function-owned iteration var required");
    };
    assert_eq!(id.kind, "loop_binding");
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == outside && matches!(site.2, Resolution::Unresolved(_)))
    );
}

#[test]
fn issue_162_assignment_iteration_does_not_create_a_new_declaration() {
    let source = "let item = 0; for (item of values) { use(item); }";
    let capture = capture(source);
    let reference = source.rfind("use(item)").unwrap() + 4;
    let (_, _, Resolution::Resolved(id)) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == reference)
        .unwrap()
    else {
        panic!("existing outer binding required");
    };
    assert_eq!(id.kind, "variable_declarator");
    assert_eq!(&source[id.start..id.end], "item = 0");
}

#[test]
fn issue_162_destructured_iteration_keeps_binding_slots_separate_from_keys() {
    let source = "for (const { key: item } of values) { use(item); }";
    let capture = capture(source);
    let reference = source.rfind("use(item)").unwrap() + 4;
    let (_, _, Resolution::Resolved(id)) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == reference)
        .unwrap()
    else {
        panic!("exact iteration pattern slot required");
    };
    assert_eq!(id.kind, "loop_binding");
    assert_eq!(&source[id.start..id.end], "item");
    assert!(!capture.declarations.iter().any(|id| id.name == "key"));
}
