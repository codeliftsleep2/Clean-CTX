use super::{Namespace, Resolution, Resolver};
use std::path::Path;
fn write(root: &Path, name: &str, text: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"rootDir":"src","outDir":"dist"},"files":["src/app.ts","src/leaf.ts"]}"#,
    );
    write(
        root,
        "package.json",
        r##"{"name":"local-pkg","imports":{"#leaf":"./dist/leaf.js"},"exports":{".":{"types":"./dist/leaf.d.ts","default":"./dist/leaf.js"}}}"##,
    );
    write(root, "src/leaf.ts", "export class Leaf {}");
}
fn lookup(root: &Path, specifier: &str, namespace: Namespace) -> Resolution {
    write(
        root,
        "src/app.ts",
        &format!("import {{ Leaf }} from '{specifier}';"),
    );
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let control = resolver.local(&root.join("src/leaf.ts"), "Leaf", Namespace::Value);
    assert!(matches!(control, Resolution::Resolved(_)));
    resolver.local(&root.join("src/app.ts"), "Leaf", namespace)
}
#[test]
fn issue_162_local_package_input_private_import_maps_missing_output_to_source() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let result = lookup(root.path(), "#leaf", Namespace::Value);
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("src/leaf.ts")),
        "{result:?}"
    );
}
#[test]
fn issue_162_local_package_input_self_types_target_prefers_current_source_over_old_output() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(root.path(), "dist/leaf.d.ts", "export interface Leaf {}");
    let result = lookup(root.path(), "local-pkg", Namespace::Value);
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("src/leaf.ts")),
        "{result:?}"
    );
}
#[test]
fn issue_162_local_package_input_installed_dependency_keeps_its_declaration_identity() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/foreign/package.json",
        r#"{"name":"foreign","exports":{".":"./dist/leaf.d.ts"}}"#,
    );
    write(
        root.path(),
        "node_modules/foreign/dist/leaf.d.ts",
        "export interface Leaf {}",
    );
    let result = lookup(root.path(), "foreign", Namespace::Type);
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("foreign/dist/leaf.d.ts")),
        "{result:?}"
    );
    assert!(matches!(
        lookup(root.path(), "foreign", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}
#[test]
fn issue_162_local_package_input_absent_input_allows_existing_output_fallback() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"rootDir":"src","outDir":"dist"},"files":["src/app.ts","src/leaf.ts"]}"#,
    );
    write(
        root.path(),
        "package.json",
        r##"{"name":"local-pkg","imports":{"#leaf":"./dist/other.js"}}"##,
    );
    write(root.path(), "dist/other.d.ts", "export interface Leaf {}");
    let result = lookup(root.path(), "#leaf", Namespace::Type);
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("dist/other.d.ts")),
        "{result:?}"
    );
}
