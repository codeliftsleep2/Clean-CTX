use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;

fn setup(root: &Path, source: &str) {
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"NodeNext","moduleResolution":"nodenext","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    std::fs::write(root.join("app.cts"), source).unwrap();
}
fn query(capture: &TypeScriptAuthority, source: &str, expression: &str) -> Resolution {
    let start = source.find(expression).unwrap();
    capture
        .type_reference_sites
        .iter()
        .find(|site| site.0 == start && site.1 == start + expression.len())
        .unwrap_or_else(|| {
            panic!(
                "missing type query {expression}: {:?}",
                capture.type_reference_sites
            )
        })
        .2
        .clone()
}
fn capture(root: &Path, source: &str) -> TypeScriptAuthority {
    TypeScriptAuthority::capture(&root.join("app.cts"), source, &[root.to_path_buf()])
}

#[test]
fn issue_162_type_query_value_identity_is_distinct_from_same_spelling_type() {
    let root = tempfile::tempdir().unwrap();
    let source = "interface Item {} const Item = 1; type Selected = typeof Item;";
    setup(root.path(), source);
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let value = resolver.local(&root.path().join("app.cts"), "Item", Namespace::Value);
    let ty = resolver.local(&root.path().join("app.cts"), "Item", Namespace::Type);
    assert!(matches!(value, Resolution::Resolved(_)));
    assert!(matches!(ty, Resolution::Resolved(_)));
    assert_ne!(value, ty);
    let capture = capture(root.path(), source);
    assert_eq!(query(&capture, source, "typeof Item"), value);
    let operand = source.find("typeof Item").unwrap() + "typeof ".len();
    assert!(!capture.reference_sites.iter().any(|site| site.0 == operand));
}

#[test]
fn issue_162_type_query_parameter_shadow_uses_exact_lexical_owner() {
    let root = tempfile::tempdir().unwrap();
    let source = "const Item = 1; function f(Item: number) { type Selected = typeof Item; }";
    setup(root.path(), source);
    let capture = capture(root.path(), source);
    let Resolution::Resolved(id) = query(&capture, source, "typeof Item") else {
        panic!("parameter query unresolved");
    };
    assert_eq!(&source[id.start..id.end], "Item: number");
    assert!(id.start > source.find("function").unwrap());
}

#[test]
fn issue_162_type_query_imported_namespace_member_keeps_value_declaration() {
    let root = tempfile::tempdir().unwrap();
    let source = "import * as API from './leaf'; type Selected = typeof API.Core.make;";
    setup(root.path(), source);
    std::fs::write(
        root.path().join("leaf.ts"),
        "export namespace Core { export function make() {} }",
    )
    .unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = resolver.namespace_member(
        &root.path().join("leaf.ts"),
        "Core",
        "make",
        Namespace::Value,
    );
    assert!(matches!(expected, Resolution::Resolved(_)));
    assert_eq!(
        query(
            &capture(root.path(), source),
            source,
            "typeof API.Core.make"
        ),
        expected
    );
}

#[test]
fn issue_162_type_query_type_only_import_allows_query_without_runtime_authority() {
    let root = tempfile::tempdir().unwrap();
    let source = "import type { make } from './leaf'; type Selected = typeof make; make();";
    setup(root.path(), source);
    std::fs::write(root.path().join("leaf.ts"), "export function make() {}").unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = resolver.local(&root.path().join("leaf.ts"), "make", Namespace::Value);
    assert!(matches!(expected, Resolution::Resolved(_)));
    let capture = capture(root.path(), source);
    assert_eq!(query(&capture, source, "typeof make"), expected);
    let runtime = source.rfind("make()").unwrap();
    assert!(
        !capture
            .reference_sites
            .iter()
            .any(|site| site.0 == runtime && matches!(site.2, Resolution::Resolved(_)))
    );
}

#[test]
fn issue_162_type_query_import_expression_default_tracks_containing_format() {
    let root = tempfile::tempdir().unwrap();
    let source = "type Selected = typeof import('pkg').make;";
    setup(root.path(), source);
    std::fs::create_dir_all(root.path().join("node_modules/pkg")).unwrap();
    std::fs::write(
        root.path().join("node_modules/pkg/package.json"),
        r#"{"exports":{"import":"./esm.d.mts","require":"./cjs.d.cts"}}"#,
    )
    .unwrap();
    for name in ["esm.d.mts", "cjs.d.cts"] {
        std::fs::write(
            root.path().join("node_modules/pkg").join(name),
            "export function make(): void;",
        )
        .unwrap();
    }
    let capture = capture(root.path(), source);
    assert_eq!(capture.module_requests.len(), 1);
    assert!(
        capture.module_requests[0]
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("cjs.d.cts")
    );
    let Resolution::Resolved(id) = query(&capture, source, "typeof import('pkg').make") else {
        panic!("import value query unresolved");
    };
    assert!(id.source.ends_with("cjs.d.cts"));
    assert_eq!(id.kind, "function_signature");
}
