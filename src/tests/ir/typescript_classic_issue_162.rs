use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}
fn setup(root: &Path, extra: &str) {
    write(
        root,
        "tsconfig.json",
        &format!(
            r#"{{"compilerOptions":{{"module":"ESNext","moduleResolution":"classic"{extra}}}}}"#
        ),
    );
    write(root, "src/app.ts", "import { Leaf } from 'leaf';");
}
fn resolver(root: &Path) -> Resolver {
    Resolver::new(&[root.to_path_buf()]).unwrap()
}

#[test]
fn issue_162_classic_typescript_ancestor_precedes_nearer_javascript() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "");
    write(root.path(), "src/leaf.js", "export class Runtime {}");
    write(root.path(), "leaf.ts", "export class Leaf {}");
    let file = root.path().join("src/app.ts");
    let mut before = resolver(root.path());
    assert!(before.module(&file, "leaf").unwrap().ends_with("leaf.ts"));
    assert!(matches!(
        before.local(&file, "Leaf", Namespace::Value),
        Resolution::Resolved(_)
    ));
    write(root.path(), "src/leaf.ts", "export class Nearer {}");
    assert!(!before.host.current());
    assert!(
        resolver(root.path())
            .module(&file, "leaf")
            .unwrap()
            .ends_with("src/leaf.ts")
    );
}

#[test]
fn issue_162_classic_relative_files_do_not_gain_node_directory_semantics() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "");
    write(root.path(), "src/dir/index.ts", "export class Leaf {}");
    let file = root.path().join("src/app.ts");
    assert!(resolver(root.path()).module(&file, "./dir").is_err());
    write(root.path(), "src/dir.ts", "export class Leaf {}");
    assert!(
        resolver(root.path())
            .module(&file, "./dir")
            .unwrap()
            .ends_with("dir.ts")
    );
}

#[test]
fn issue_162_classic_paths_fallback_is_file_only_and_type_phase_first() {
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        r#", "paths":{"leaf":["missing","mapped","fallback"]}"#,
    );
    write(
        root.path(),
        "mapped/index.ts",
        "export class WrongDirectory {}",
    );
    write(root.path(), "mapped.js", "export class Runtime {}");
    write(root.path(), "fallback.ts", "export class Leaf {}");
    let file = root.path().join("src/app.ts");
    assert!(
        resolver(root.path())
            .module(&file, "leaf")
            .unwrap()
            .ends_with("fallback.ts")
    );
}

#[test]
fn issue_162_classic_types_packages_are_distinct_from_ordinary_node_packages() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "");
    write(
        root.path(),
        "node_modules/leaf/index.ts",
        "export class WrongNodePackage {}",
    );
    let file = root.path().join("src/app.ts");
    let mut absent = resolver(root.path());
    assert!(absent.module(&file, "leaf").is_err());
    write(
        root.path(),
        "node_modules/@types/leaf/package.json",
        r#"{"types":"./entry.d.ts","exports":"./wrong.d.ts"}"#,
    );
    write(
        root.path(),
        "node_modules/@types/leaf/entry.d.ts",
        "export declare class Leaf {}",
    );
    assert!(!absent.host.current());
    assert!(
        resolver(root.path())
            .module(&file, "leaf")
            .unwrap()
            .ends_with("entry.d.ts")
    );
    write(
        root.path(),
        "node_modules/@types/scope__pkg/index.d.ts",
        "export declare class Scoped {}",
    );
    assert!(
        resolver(root.path())
            .module(&file, "@scope/pkg")
            .unwrap()
            .ends_with("scope__pkg/index.d.ts")
    );
}

#[test]
fn issue_162_classic_custom_type_roots_and_config_changes_are_witnesses() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), r#", "typeRoots":["custom"]"#);
    write(
        root.path(),
        "custom/leaf/index.d.ts",
        "export declare class Leaf {}",
    );
    write(
        root.path(),
        "other/leaf/index.d.ts",
        "export declare class Other {}",
    );
    let file = root.path().join("src/app.ts");
    let mut before = resolver(root.path());
    assert!(
        before
            .module(&file, "leaf")
            .unwrap()
            .ends_with("custom/leaf/index.d.ts")
    );
    setup(root.path(), r#", "typeRoots":["other"]"#);
    assert!(!before.host.current());
    assert!(
        resolver(root.path())
            .module(&file, "leaf")
            .unwrap()
            .ends_with("other/leaf/index.d.ts")
    );
}

#[test]
fn issue_162_classic_virtual_roots_share_the_same_negative_snapshot_frontier() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), r#", "rootDirs":["src","generated"]"#);
    write(root.path(), "generated/leaf.ts", "export class Leaf {}");
    let file = root.path().join("src/app.ts");
    let mut before = resolver(root.path());
    assert!(
        before
            .module(&file, "./leaf")
            .unwrap()
            .ends_with("generated/leaf.ts")
    );
    write(root.path(), "src/leaf.ts", "export class Local {}");
    assert!(!before.host.current());
    assert!(
        resolver(root.path())
            .module(&file, "./leaf")
            .unwrap()
            .ends_with("src/leaf.ts")
    );
}
