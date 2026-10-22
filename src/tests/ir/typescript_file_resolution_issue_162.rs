use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
}
fn setup(root: &Path, extra: &str) {
    write(
        root,
        "tsconfig.json",
        &format!(
            r#"{{"compilerOptions":{{"module":"ESNext","moduleResolution":"bundler"{extra}}}}}"#
        ),
    );
    write(root, "target.ts", "export {};");
}

#[test]
fn issue_162_package_exports_control_changes_authoritative_entry_and_invalidates() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "");
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":"./modern.ts","types":"./legacy.d.ts"}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/modern.ts",
        "export class Modern {}",
    );
    write(
        root.path(),
        "node_modules/pkg/legacy.d.ts",
        "export declare class Legacy {}",
    );
    let target = root.path().join("target.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        resolver
            .module(&target, "pkg")
            .unwrap()
            .ends_with("modern.ts")
    );
    setup(root.path(), r#", "resolvePackageJsonExports":false"#);
    assert!(!resolver.host.current());
    let mut disabled = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        disabled
            .module(&target, "pkg")
            .unwrap()
            .ends_with("legacy.d.ts")
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"node10"}}"#,
    );
    let mut legacy = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        legacy
            .module(&target, "pkg")
            .unwrap()
            .ends_with("legacy.d.ts")
    );
}

#[test]
fn issue_162_package_imports_disabled_does_not_use_private_mapping() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "");
    write(
        root.path(),
        "package.json",
        r##"{"imports":{"#leaf":"./private.ts"}}"##,
    );
    write(root.path(), "private.ts", "export class Leaf {}");
    let target = root.path().join("target.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        resolver
            .module(&target, "#leaf")
            .unwrap()
            .ends_with("private.ts")
    );
    setup(root.path(), r#", "resolvePackageJsonImports":false"#);
    assert!(!resolver.host.current());
    let mut disabled = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(disabled.module(&target, "#leaf").is_err());
}

#[test]
fn issue_162_javascript_extensions_select_typescript_before_runtime_files() {
    for (specifier, preferred) in [
        ("./leaf.js", "leaf.ts"),
        ("./leaf.mjs", "leaf.mts"),
        ("./leaf.cjs", "leaf.cts"),
    ] {
        let root = tempfile::tempdir().unwrap();
        setup(root.path(), "");
        write(root.path(), &specifier[2..], "export class Runtime {}");
        write(root.path(), preferred, "export class Preferred {}");
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        let found = resolver
            .module(&root.path().join("target.ts"), specifier)
            .unwrap();
        assert!(found.ends_with(preferred), "{found:?}");
    }
}

#[test]
fn issue_162_missing_extension_substitute_appearance_invalidates_previous_choice() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "");
    write(root.path(), "leaf.js", "export class Runtime {}");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        resolver
            .module(&root.path().join("target.ts"), "./leaf.js")
            .unwrap()
            .ends_with("leaf.js")
    );
    assert!(resolver.host.current());
    write(root.path(), "leaf.ts", "export class Preferred {}");
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        fresh
            .module(&root.path().join("target.ts"), "./leaf.js")
            .unwrap()
            .ends_with("leaf.ts")
    );
}

#[test]
fn issue_162_extensionless_lookup_does_not_invent_mts_or_cts() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "");
    write(root.path(), "leaf.mts", "export class Leaf {}");
    write(root.path(), "leaf.cts", "export class Leaf {}");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        resolver
            .module(&root.path().join("target.ts"), "./leaf")
            .is_err()
    );
}

#[test]
fn issue_162_module_suffix_order_and_omitted_empty_suffix_are_authoritative() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), r#", "moduleSuffixes":[".native", ""]"#);
    write(root.path(), "leaf.native.ts", "export class Native {}");
    write(root.path(), "leaf.ts", "export class Plain {}");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        resolver
            .module(&root.path().join("target.ts"), "./leaf.js")
            .unwrap()
            .ends_with("leaf.native.ts")
    );
    setup(root.path(), r#", "moduleSuffixes":[".missing"]"#);
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        fresh
            .module(&root.path().join("target.ts"), "./leaf.js")
            .is_err()
    );
}

#[test]
fn issue_162_directory_package_entry_precedes_index_without_export_leakage() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "");
    write(
        root.path(),
        "dir/package.json",
        r#"{"types":"./entry.d.ts","exports":"./wrong.ts"}"#,
    );
    write(
        root.path(),
        "dir/entry.d.ts",
        "export declare class Leaf {}",
    );
    write(root.path(), "dir/index.ts", "export class Wrong {}");
    write(root.path(), "dir/wrong.ts", "export class Wrong {}");
    write(root.path(), "target.ts", "import { Leaf } from './dir';");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let Resolution::Resolved(id) =
        resolver.local(&root.path().join("target.ts"), "Leaf", Namespace::Value)
    else {
        panic!("directory types entry must resolve");
    };
    assert!(id.source.ends_with("entry.d.ts"));
}

#[test]
fn issue_162_explicit_base_url_dot_is_not_confused_with_no_base_url() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), r#", "baseUrl":".""#);
    write(root.path(), "local.ts", "export class Leaf {}");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        resolver
            .module(&root.path().join("target.ts"), "local")
            .unwrap()
            .ends_with("local.ts")
    );
}
