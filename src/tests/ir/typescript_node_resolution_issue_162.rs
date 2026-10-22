use super::Resolver;
use std::path::Path;

fn write(root: &Path, file: &str, text: &str) {
    let file = root.join(file);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}
fn setup(root: &Path, mode: &str) {
    write(
        root,
        "tsconfig.json",
        &format!(r#"{{"compilerOptions":{{"module":"{mode}","moduleResolution":"{mode}"}}}}"#),
    );
    write(root, "app.ts", "export {};");
    write(root, "app.mts", "export {};");
    write(root, "app.cts", "export {};");
    write(root, "leaf.ts", "export class Leaf {}");
    write(root, "dir/index.ts", "export class Leaf {}");
}

#[test]
fn issue_162_node_static_import_format_controls_relative_extension_and_directory_lookup() {
    for mode in ["node16", "nodenext"] {
        let root = tempfile::tempdir().unwrap();
        setup(root.path(), mode);
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        assert!(
            resolver
                .module(&root.path().join("app.cts"), "./leaf")
                .unwrap()
                .ends_with("leaf.ts")
        );
        assert!(
            resolver
                .module(&root.path().join("app.cts"), "./dir")
                .unwrap()
                .ends_with("dir/index.ts")
        );
        assert!(
            resolver
                .module(&root.path().join("app.mts"), "./leaf")
                .is_err()
        );
        assert!(
            resolver
                .module(&root.path().join("app.mts"), "./dir")
                .is_err()
        );
        assert!(
            resolver
                .module(&root.path().join("app.mts"), "./leaf.js")
                .unwrap()
                .ends_with("leaf.ts")
        );
    }
}

#[test]
fn issue_162_node_package_type_mutation_invalidates_importing_file_format() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "node16");
    let file = root.path().join("app.ts");
    let mut before = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(before.module(&file, "./leaf").is_ok());
    write(root.path(), "package.json", r#"{"type":"module"}"#);
    assert!(!before.host.current());
    let mut after = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(after.module(&file, "./leaf").is_err());
    assert!(after.module(&file, "./leaf.js").is_ok());
    assert!(after.module(&root.path().join("app.cts"), "./leaf").is_ok());
}

#[test]
fn issue_162_node_static_imports_select_import_or_require_package_conditions() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "node16");
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{"import":"./esm.mts","require":"./cjs.cts"}}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/esm.mts",
        "export class ESM {}",
    );
    write(
        root.path(),
        "node_modules/pkg/cjs.cts",
        "export class CJS {}",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        resolver
            .module(&root.path().join("app.mts"), "pkg")
            .unwrap()
            .ends_with("esm.mts")
    );
    assert!(
        resolver
            .module(&root.path().join("app.cts"), "pkg")
            .unwrap()
            .ends_with("cjs.cts")
    );
}
