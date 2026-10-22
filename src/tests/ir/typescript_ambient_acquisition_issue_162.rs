use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;

fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}
fn setup(root: &Path, name: &str, installed: &str, reversed: bool) -> String {
    let files = if reversed {
        r#"["ambient.d.ts","app.ts","control.ts"]"#
    } else {
        r#"["app.ts","control.ts","ambient.d.ts"]"#
    };
    write(
        root,
        "tsconfig.json",
        &format!(
            r#"{{"compilerOptions":{{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}},"files":{files}}}"#
        ),
    );
    write(root, "control.ts", "export class Control {}");
    write(
        root,
        "ambient.d.ts",
        &format!(r#"declare module "{name}" {{ export class Leaf {{}} }}"#),
    );
    write(
        root,
        "node_modules/virtual-api/package.json",
        r#"{"types":"index.d.ts"}"#,
    );
    write(root, "node_modules/virtual-api/index.d.ts", installed);
    let source = "import { Leaf } from 'virtual-api'; const item = Leaf;".to_string();
    write(root, "app.ts", &source);
    source
}
fn lookup(root: &Path) -> Resolution {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&root.join("control.ts"), "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    resolver.local(&root.join("app.ts"), "Leaf", Namespace::Value)
}

#[test]
fn issue_162_ambient_acquisition_exact_scope_ignores_invalid_shadowed_file_in_both_orders() {
    for reversed in [false, true] {
        let root = tempfile::tempdir().unwrap();
        setup(root.path(), "virtual-api", "export class {", reversed);
        let result = lookup(root.path());
        assert!(
            matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("ambient.d.ts")),
            "order {reversed}: {result:?}"
        );
    }
}

#[test]
fn issue_162_ambient_acquisition_shadowed_file_is_not_a_consumed_content_dependency() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let source = setup(path, "virtual-api", "export class Leaf {}", true);
    let before = TypeScriptAuthority::capture(&path.join("app.ts"), &source, &[path.to_path_buf()]);
    assert!(before.capture_current);
    assert_eq!(before.module_requests[0].ambient_declarations.len(), 1);
    write(
        path,
        "node_modules/virtual-api/index.d.ts",
        "export class {",
    );
    assert!(
        before.witnesses.iter().all(|witness| witness.current()),
        "shadowed file was consumed"
    );
    assert!(
        matches!(lookup(path), Resolution::Resolved(ref id) if id.source.ends_with("ambient.d.ts"))
    );
}

#[test]
fn issue_162_ambient_acquisition_wildcard_cannot_rescue_invalid_selected_file() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    setup(path, "*", "export class Leaf {}", false);
    assert!(
        matches!(lookup(path), Resolution::Resolved(ref id) if id.source.ends_with("index.d.ts"))
    );
    write(
        path,
        "node_modules/virtual-api/index.d.ts",
        "export class {",
    );
    assert!(matches!(lookup(path), Resolution::Unresolved(_)));
}
