use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};

#[test]
fn issue_162_destructured_export_slots_keep_distinct_identity_and_ignore_object_keys() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("leaf.ts");
    let source = "export const { remote: Local, Plain, ...Rest } = value; use(Local); use(Plain);";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let names: Vec<_> = capture
        .declarations
        .iter()
        .filter(|id| id.kind == "binding_pattern")
        .map(|id| id.name.as_str())
        .collect();
    assert_eq!(names, ["Local", "Plain", "Rest"]);
    assert!(!capture.declarations.iter().any(|id| id.name == "remote"));
    for name in ["Local", "Plain"] {
        let start = source.rfind(&format!("use({name})")).unwrap() + 4;
        let (_, _, Resolution::Resolved(id)) = capture
            .reference_sites
            .iter()
            .find(|site| site.0 == start)
            .unwrap()
        else {
            panic!("exact binding slot required");
        };
        assert_eq!(id.kind, "binding_pattern");
        assert_eq!(&source[id.start..id.end], name);
        assert!(capture.declarations.contains(id));
    }
}

#[test]
fn issue_162_nested_array_defaults_bind_only_pattern_names_not_initializer_references() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    let source = "const fallback = 1; const [first = fallback, { key: nested }, ...rest] = values; use(first); use(nested); use(rest);";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    for name in ["first", "nested", "rest"] {
        assert_eq!(
            capture
                .declarations
                .iter()
                .filter(|id| id.name == name)
                .count(),
            1
        );
    }
    assert_eq!(
        capture
            .declarations
            .iter()
            .filter(|id| id.name == "fallback")
            .count(),
        1
    );
    assert!(!capture.declarations.iter().any(|id| id.name == "key"));
    let start = source.rfind("fallback").unwrap();
    assert!(capture.reference_sites.iter().any(|site| site.0 == start
        && matches!(&site.2, Resolution::Resolved(id) if id.kind == "variable_declarator")));
}

#[test]
fn issue_162_pattern_parameter_and_catch_slots_shadow_without_widening_legacy_adapter() {
    use crate::ir::invocation_evidence::SourceSpan;
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    let source = "function f({ key: item }: any) { use(item); } try {} catch ({ key: caught }) { use(caught); }";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    for name in ["item", "caught"] {
        let start = source.rfind(&format!("use({name})")).unwrap() + 4;
        let (_, _, Resolution::Resolved(id)) = capture
            .reference_sites
            .iter()
            .find(|site| site.0 == start)
            .unwrap()
        else {
            panic!("owned pattern binding required");
        };
        assert_eq!(id.kind, "binding_pattern");
        assert_eq!(&source[id.start..id.end], name);
        let span = SourceSpan::new(start, start + name.len()).unwrap();
        assert!(
            crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
                "app.ts", source, span
            )
            .is_none()
        );
    }
}

#[test]
fn issue_162_imported_pattern_export_uses_the_same_slot_as_local_references() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    let leaf = root.path().join("leaf.ts");
    std::fs::write(&leaf, "export const { key: Local } = value; use(Local);").unwrap();
    let app = root.path().join("app.ts");
    std::fs::write(&app, "import { Local as Selected } from './leaf';").unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let local = resolver.local(&leaf, "Local", Namespace::Value);
    assert!(matches!(local, Resolution::Resolved(_)));
    assert_eq!(resolver.local(&app, "Selected", Namespace::Value), local);
    assert!(matches!(
        resolver.local(&app, "Selected", Namespace::Type),
        Resolution::Unresolved(_)
    ));
}
