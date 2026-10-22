use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;
fn write(root: &Path, name: &str, text: &str) {
    std::fs::write(root.join(name), text).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#,
    );
    write(
        root,
        "leaf.ts",
        "export namespace Core { export namespace Child { export function make() {} } }",
    );
}
fn expected(root: &Path) -> Resolution {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let result = resolver.namespace_chain(
        &root.join("leaf.ts"),
        "Core",
        &["Child", "make"],
        Namespace::Value,
    );
    assert!(matches!(result, Resolution::Resolved(_)));
    result
}
fn capture(root: &Path, source: &str) -> TypeScriptAuthority {
    write(root, "app.ts", source);
    TypeScriptAuthority::capture(&root.join("app.ts"), source, &[root.to_path_buf()])
}
fn query(capture: &TypeScriptAuthority, source: &str, text: &str) -> Resolution {
    let start = source.find(text).unwrap();
    capture
        .type_reference_sites
        .iter()
        .find(|site| site.0 == start && site.1 == start + text.len())
        .unwrap_or_else(|| {
            panic!(
                "missing erased namespace query: {:?}",
                capture.type_reference_sites
            )
        })
        .2
        .clone()
}
#[test]
fn issue_162_erased_namespace_query_named_import_keeps_endpoint_without_runtime_promotion() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(p);
    let source = "import type { Core as API } from './leaf'; type Selected = typeof API.Child.make; API.Child.make();";
    let capture = capture(p, source);
    let expected = expected(p);
    assert_eq!(query(&capture, source, "typeof API.Child.make"), expected);
    let runtime = source.rfind("API.Child.make()").unwrap();
    assert!(
        !capture
            .reference_sites
            .iter()
            .any(|site| site.0 == runtime && site.2 == expected)
    );
}
#[test]
fn issue_162_erased_namespace_query_type_only_barrel_retracts_on_member_mutation() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(p);
    write(
        p,
        "barrel.ts",
        "export type { Core as Public } from './leaf';",
    );
    let source =
        "import type { Public as API } from './barrel'; type Selected = typeof API.Child.make;";
    let before = capture(p, source);
    assert_eq!(query(&before, source, "typeof API.Child.make"), expected(p));
    write(
        p,
        "leaf.ts",
        "export namespace Core { export namespace Child { export interface make {} } }",
    );
    assert!(before.witnesses.iter().any(|witness| !witness.current()));
    let after = capture(p, source);
    assert!(matches!(
        query(&after, source, "typeof API.Child.make"),
        Resolution::Unresolved(_)
    ));
}
#[test]
fn issue_162_erased_namespace_query_lexical_shadow_cannot_borrow_imported_members() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(p);
    let source = "import type { Core as API } from './leaf'; type Outer = typeof API.Child.make; function f(API: number) { type Inner = typeof API.Child.make; }";
    let capture = capture(p, source);
    assert_eq!(
        query(&capture, source, "typeof API.Child.make"),
        expected(p)
    );
    let inner = source.rfind("typeof API.Child.make").unwrap();
    assert!(
        capture
            .type_reference_sites
            .iter()
            .any(|site| site.0 == inner && matches!(site.2, Resolution::Unresolved(_))),
        "{:?}",
        capture.type_reference_sites
    );
}
