use super::{Resolution, TypeScriptAuthority};
use std::path::Path;
fn write(root: &Path, name: &str, text: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}
fn setup(root: &Path, extra: &str) -> String {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","types":[]},"files":["app.ts","ambient.d.ts"]}"#,
    );
    write(
        root,
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    write(
        root,
        "ambient.d.ts",
        r#"declare module "virtual-api" { export class Leaf {} }"#,
    );
    let source = format!("import {{ Leaf }} from 'virtual-api'; {extra} const item = Leaf;");
    write(root, "app.ts", &source);
    source
}
fn capture(root: &Path, source: &str) -> TypeScriptAuthority {
    let capture = TypeScriptAuthority::capture(&root.join("app.ts"), source, &[root.to_path_buf()]);
    let start = source.rfind("Leaf").unwrap();
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == start && matches!(site.2, Resolution::Resolved(_))),
        "{:?}",
        capture.disposition
    );
    assert!(capture.capture_current);
    capture
}
#[test]
fn issue_162_ambient_acquisition_preservation_all_ambient_requests_still_select_default_library() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let source = setup(path, "");
    write(
        path,
        "node_modules/typescript/lib/lib.d.ts",
        "/// <reference no-default-lib=\"true\"/>\ninterface LibraryControl {}",
    );
    let capture = capture(path, &source);
    assert!(!capture.compiler_libraries.is_empty());
    assert!(capture.compiler_libraries.iter().any(|request| {
        request
            .resolved
            .as_ref()
            .is_ok_and(|file| file.ends_with("lib.d.ts"))
    }));
}
#[test]
fn issue_162_ambient_acquisition_preservation_imported_no_default_lib_precedes_default_selection() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let source = setup(path, "import './stopper';");
    write(
        path,
        "stopper.ts",
        "/// <reference no-default-lib=\"true\"/>\nexport {};",
    );
    let capture = capture(path, &source);
    assert!(capture.compiler_libraries.is_empty());
}
