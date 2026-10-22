use super::Resolver;
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    );
    write(root, "app.ts", "export {};");
    compiler(root, "5.9.3");
    write(
        root,
        "node_modules/pkg/index.d.ts",
        "export interface Legacy {}",
    );
    write(
        root,
        "node_modules/pkg/new/index.d.ts",
        "export interface Modern {}",
    );
}
fn compiler(root: &Path, version: &str) {
    write(
        root,
        "node_modules/typescript/package.json",
        &format!(r#"{{"name":"typescript","version":"{version}"}}"#),
    );
}
fn resolver(root: &Path) -> Resolver {
    Resolver::new(&[root.to_path_buf()]).unwrap()
}

#[test]
fn issue_162_native_version_ranges_observe_stable_comparator_and_partial_rules() {
    for (range, version, expected) in [
        (">=5.0 <6", "5.9.3", true),
        (">= 5.0 < 6", "6.0.0", false),
        ("4.x || ^5.1", "5.9.3", true),
        ("~5.8", "5.9.3", false),
        ("5.1 - 5.9", "5.9.3", true),
        ("5.1 - 5.9.2", "5.9.3", false),
        ("^0.2.3", "0.3.0", false),
        ("^0.0.3", "0.0.4", false),
        ("^0", "0.9.1", true),
        ("<=5.8", "5.9.0", false),
        (">5.8", "5.9.0", true),
        ("*", "5.9.3", true),
    ] {
        assert_eq!(
            super::version_ranges::matches(range, version).unwrap(),
            expected,
            "{range} {version}"
        );
    }
    for range in ["garbage", ">=", "01.2", "5.9.3.1"] {
        assert!(
            super::version_ranges::matches(range, "5.9.3").is_err(),
            "{range}"
        );
    }
}

#[test]
fn issue_162_types_versions_retargets_with_compiler_manifest_and_keeps_order() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"types":"./index.d.ts","typesVersions":{">=5.5":{"*":["new/*"]},">=4":{"*":["*"]}}}"#,
    );
    let file = root.path().join("app.ts");
    let mut before = resolver(root.path());
    assert!(
        before
            .module(&file, "pkg")
            .unwrap()
            .ends_with("new/index.d.ts")
    );
    compiler(root.path(), "5.4.5");
    assert!(!before.host.current());
    assert!(
        resolver(root.path())
            .module(&file, "pkg")
            .unwrap()
            .ends_with("pkg/index.d.ts")
    );
}

#[test]
fn issue_162_active_exports_outrank_types_versions_but_disabled_exports_do_not() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":"./index.d.ts","types":"./index.d.ts","typesVersions":{"*":{"*":["new/*"]}}}"#,
    );
    let file = root.path().join("app.ts");
    assert!(
        resolver(root.path())
            .module(&file, "pkg")
            .unwrap()
            .ends_with("pkg/index.d.ts")
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","resolvePackageJsonExports":false}}"#,
    );
    assert!(
        resolver(root.path())
            .module(&file, "pkg")
            .unwrap()
            .ends_with("new/index.d.ts")
    );
}

#[test]
fn issue_162_types_versions_subpaths_respect_exact_keys_and_negative_fallbacks() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"typesVersions":{"*":{"special":["missing.d.ts","new/index.d.ts"],"*":["index.d.ts"]}}}"#,
    );
    let file = root.path().join("app.ts");
    let mut before = resolver(root.path());
    assert!(
        before
            .module(&file, "pkg/special")
            .unwrap()
            .ends_with("new/index.d.ts")
    );
    write(
        root.path(),
        "node_modules/pkg/missing.d.ts",
        "export interface Appeared {}",
    );
    assert!(!before.host.current());
    assert!(
        resolver(root.path())
            .module(&file, "pkg/special")
            .unwrap()
            .ends_with("missing.d.ts")
    );
}

#[test]
fn issue_162_versioned_types_conditions_share_compiler_evidence_and_import_maps() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{"types@>=5.5":"./new/index.d.ts","types":"./index.d.ts"}}"#,
    );
    write(
        root.path(),
        "package.json",
        r##"{"imports":{"#api":{"types@>=5.5":"./modern.ts","types":"./legacy.ts"}}}"##,
    );
    write(root.path(), "modern.ts", "export class Modern {}");
    write(root.path(), "legacy.ts", "export class Legacy {}");
    let file = root.path().join("app.ts");
    let mut before = resolver(root.path());
    assert!(
        before
            .module(&file, "pkg")
            .unwrap()
            .ends_with("new/index.d.ts")
    );
    assert!(before.module(&file, "#api").unwrap().ends_with("modern.ts"));
    compiler(root.path(), "5.4.5");
    assert!(!before.host.current());
    let mut after = resolver(root.path());
    assert!(
        after
            .module(&file, "pkg")
            .unwrap()
            .ends_with("pkg/index.d.ts")
    );
    assert!(after.module(&file, "#api").unwrap().ends_with("legacy.ts"));
}

#[test]
fn issue_162_versioned_types_require_actual_compiler_evidence() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{"types@>=5":"./new/index.d.ts","types":"./index.d.ts"}}"#,
    );
    std::fs::remove_file(root.path().join("node_modules/typescript/package.json")).unwrap();
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "pkg")
            .is_err()
    );
}

#[test]
fn issue_162_versioned_directory_modules_share_the_package_redirect_engine() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "dir/package.json",
        r#"{"types":"index.d.ts","typesVersions":{"*":{"*":["new/*"]}}}"#,
    );
    write(root.path(), "dir/index.d.ts", "export interface Legacy {}");
    write(
        root.path(),
        "dir/new/index.d.ts",
        "export interface Current {}",
    );
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "./dir")
            .unwrap()
            .ends_with("dir/new/index.d.ts")
    );
}

#[test]
fn issue_162_cyclic_types_versions_do_not_recurse_without_bound() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"typesVersions":{"*":{"*":["."]}}}"#,
    );
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "pkg")
            .is_err()
    );
}

#[test]
fn issue_162_unvisited_version_condition_does_not_require_missing_compiler_evidence() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    std::fs::remove_file(root.path().join("node_modules/typescript/package.json")).unwrap();
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{"default":"./index.d.ts","types@>=5":"./new/index.d.ts"}}"#,
    );
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "pkg")
            .unwrap()
            .ends_with("pkg/index.d.ts")
    );
}

#[test]
fn issue_162_types_versions_keep_canonical_declaration_identity_across_compiler_retarget() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    // This fixture supplies only a compiler version witness for package routing.
    // Explicitly exclude unrelated compiler-library and automatic-global inputs.
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"types":"index.d.ts","typesVersions":{">=5.5":{"*":["new/*"]}}}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/index.d.ts",
        "export declare class Leaf {}",
    );
    write(
        root.path(),
        "node_modules/pkg/new/index.d.ts",
        "export declare class Leaf {}",
    );
    write(
        root.path(),
        "app.ts",
        "import { Leaf as Selected } from 'pkg';",
    );
    let file = root.path().join("app.ts");
    let mut before = resolver(root.path());
    let super::Resolution::Resolved(modern) =
        before.local(&file, "Selected", super::Namespace::Value)
    else {
        panic!("modern declaration control");
    };
    assert!(modern.source.ends_with("new/index.d.ts"));
    compiler(root.path(), "5.4.5");
    assert!(!before.host.current());
    let super::Resolution::Resolved(legacy) =
        resolver(root.path()).local(&file, "Selected", super::Namespace::Value)
    else {
        panic!("legacy declaration control");
    };
    assert!(legacy.source.ends_with("pkg/index.d.ts"));
    assert_eq!(modern.name, legacy.name);
    assert_ne!(modern.source, legacy.source);
}

#[test]
fn issue_162_prerelease_ranges_follow_typescript_order_and_partial_floor_rules() {
    for (range, version, expected) in [
        (">=5.9.0-beta.2 <5.9.0", "5.9.0-beta.10", true),
        (">=5.9.0", "5.9.0-beta.10", false),
        ("5.9", "5.9.0-beta.10", true),
        ("<5.9", "5.9.0-beta.10", false),
        ("<5.9.0", "5.9.0-beta.10", true),
        ("^5.9.0", "6.0.0-beta.1", true),
        ("5.x.2", "5.9.3", true),
        ("5.9 ||", "5.8.0", false),
        ("* - *", "0.0.0-alpha", true),
        (">=5.9.0-alpha", "5.9.0-1", false),
    ] {
        assert_eq!(
            super::version_ranges::matches(range, version).unwrap(),
            expected,
            "{range} {version}"
        );
    }
}

#[test]
fn issue_162_build_metadata_does_not_change_version_precedence() {
    for version in ["5.9.3+build.001", "5.9.3+different", "5.9.3"] {
        assert!(super::version_ranges::matches("=5.9.3+baseline", version).unwrap());
    }
    for version in [
        "5.9.3-beta.01",
        "5.9.3-beta..1",
        "5.9.3+",
        "5.9.3-",
        "5.9.3+bad/value",
        "v5.9.3",
    ] {
        assert!(
            super::version_ranges::matches("*", version).is_err(),
            "{version}"
        );
    }
}

#[test]
fn issue_162_prerelease_compiler_manifest_drives_type_conditions_and_mutation() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{"types@>=5.9.0-beta.2 <5.9.0":"./new/index.d.ts","types":"./index.d.ts"}}"#,
    );
    compiler(root.path(), "5.9.0-beta.10+local");
    let file = root.path().join("app.ts");
    let mut before = resolver(root.path());
    assert!(
        before
            .module(&file, "pkg")
            .unwrap()
            .ends_with("new/index.d.ts")
    );
    compiler(root.path(), "5.9.0+release");
    assert!(!before.host.current());
    assert!(
        resolver(root.path())
            .module(&file, "pkg")
            .unwrap()
            .ends_with("pkg/index.d.ts")
    );
}

#[test]
fn issue_162_prerelease_types_versions_use_the_same_version_matcher() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"types":"index.d.ts","typesVersions":{"5.9":{"*":["new/*"]}}}"#,
    );
    compiler(root.path(), "5.9.0-beta.10");
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "pkg")
            .unwrap()
            .ends_with("new/index.d.ts")
    );
}
