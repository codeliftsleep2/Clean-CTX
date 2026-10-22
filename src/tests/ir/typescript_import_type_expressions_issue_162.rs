use super::module_resolution::ModuleRequestKind;
use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;

fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"NodeNext","moduleResolution":"nodenext","moduleDetection":"legacy","noLib":true,"types":[]}}"#,
    );
    write(
        root,
        "node_modules/pkg/package.json",
        r#"{"exports":{"import":"./esm.d.mts","require":"./cjs.d.cts"}}"#,
    );
    for name in ["esm.d.mts", "cjs.d.cts"] {
        write(
            root,
            &format!("node_modules/pkg/{name}"),
            "export class Leaf {} export namespace Core { export interface Shape {} }",
        );
    }
}
fn control(root: &Path, file: &str) -> super::DeclarationId {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let result = resolver.local(
        &root.join(format!("node_modules/pkg/{file}")),
        "Leaf",
        Namespace::Type,
    );
    let Resolution::Resolved(id) = result else {
        panic!("nonempty control: {result:?}");
    };
    assert!(id.end > id.start);
    id
}

#[test]
fn issue_162_import_type_expression_explicit_modes_retain_exact_type_occurrence() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    for (mode, target, kind) in [
        ("require", "cjs.d.cts", ModuleRequestKind::Require),
        ("import", "esm.d.mts", ModuleRequestKind::Import),
    ] {
        let expression =
            format!("import(\"pkg\", {{ with: {{ \"resolution-mode\": \"{mode}\" }} }}).Leaf");
        let text = format!("type Selected = {expression};");
        write(root.path(), "app.mts", &text);
        let expected = control(root.path(), target);
        let capture = TypeScriptAuthority::capture(
            &root.path().join("app.mts"),
            &text,
            &[root.path().to_path_buf()],
        );
        assert_eq!(capture.module_requests.len(), 1);
        assert_eq!(capture.module_requests[0].kind, kind);
        assert!(
            capture.module_requests[0]
                .resolved
                .as_ref()
                .unwrap()
                .ends_with(target)
        );
        let start = text.find(&expression).unwrap();
        assert!(
            capture
                .type_reference_sites
                .iter()
                .any(|site| site.0 == start
                    && site.1 == start + expression.len()
                    && site.2 == Resolution::Resolved(expected.clone())),
            "missing exact import-type occurrence: {:?}",
            capture.type_reference_sites
        );
        assert!(
            !capture
                .reference_sites
                .iter()
                .any(|site| site.0 == start && site.2 == Resolution::Resolved(expected.clone()))
        );
    }
}

#[test]
fn issue_162_import_type_expression_default_uses_source_format_while_runtime_is_import() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let text = "type Selected = import('pkg').Leaf; const lazy = import('pkg');";
    write(root.path(), "app.cts", text);
    let capture = TypeScriptAuthority::capture(
        &root.path().join("app.cts"),
        text,
        &[root.path().to_path_buf()],
    );
    assert_eq!(capture.module_requests.len(), 2);
    assert!(
        capture.module_requests[0]
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("cjs.d.cts")
    );
    assert_eq!(capture.module_requests[1].kind, ModuleRequestKind::Import);
    assert!(
        capture.module_requests[1]
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("esm.d.mts")
    );
}

#[test]
fn issue_162_import_type_expression_qualified_namespace_retains_member_identity() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let expression = "import('pkg').Core.Shape";
    let text = format!("interface Shape {{}} type Selected = {expression};");
    write(root.path(), "app.mts", &text);
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = resolver.namespace_member(
        &root.path().join("node_modules/pkg/esm.d.mts"),
        "Core",
        "Shape",
        Namespace::Type,
    );
    assert!(matches!(expected, Resolution::Resolved(_)));
    let capture = TypeScriptAuthority::capture(
        &root.path().join("app.mts"),
        &text,
        &[root.path().to_path_buf()],
    );
    let start = text.find(expression).unwrap();
    assert!(
        capture
            .type_reference_sites
            .iter()
            .any(|site| site.0 == start
                && site.1 == start + expression.len()
                && site.2 == expected),
        "missing qualified import type: {:?}",
        capture.type_reference_sites
    );
}
