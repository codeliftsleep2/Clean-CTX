use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, file: &str, source: &str) {
    let path = root.join(file);
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
}
fn resolver(root: &Path) -> Resolver {
    Resolver::new(&[root.to_path_buf()]).unwrap()
}

#[test]
fn issue_162_nested_unmatched_conditions_fall_through_but_null_stays_blocked() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/leaf.ts",
        "export class Leaf {}",
    );
    let file = root.path().join("app.ts");
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{"types":{"foreign":"./absent.ts"},"default":"./leaf.ts"}}"#,
    );
    assert!(
        resolver(root.path())
            .module(&file, "pkg")
            .unwrap()
            .ends_with("leaf.ts")
    );
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{"types":null,"default":"./leaf.ts"}}"#,
    );
    assert!(resolver(root.path()).module(&file, "pkg").is_err());
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{".":"./leaf.ts","default":"./leaf.ts"}}"#,
    );
    assert!(resolver(root.path()).module(&file, "pkg").is_err());
}

#[test]
fn issue_162_null_root_exports_differ_from_a_null_exported_subpath() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/index.ts",
        "export class Leaf {}",
    );
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":null}"#,
    );
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "pkg")
            .unwrap()
            .ends_with("index.ts")
    );
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{".":null}}"#,
    );
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "pkg")
            .is_err()
    );
}

#[test]
fn issue_162_package_self_reference_owns_identity_and_blocks_installed_fallback() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "package.json",
        r#"{"name":"@app/core","exports":{".":"./own.ts"}}"#,
    );
    write(root.path(), "own.ts", "export class Leaf {}");
    write(
        root.path(),
        "node_modules/@app/core/package.json",
        r#"{"exports":{".":"./other.ts","./hidden":"./other.ts"}}"#,
    );
    write(
        root.path(),
        "node_modules/@app/core/other.ts",
        "export class Leaf {}",
    );
    let mut resolve = resolver(root.path());
    let app = root.path().join("app.ts");
    assert!(
        resolve
            .module(&app, "@app/core")
            .unwrap()
            .ends_with("own.ts")
    );
    assert!(resolve.module(&app, "@app/core/hidden").is_err());
    assert!(resolve.host.current());
}

#[test]
fn issue_162_package_self_export_retarget_invalidates_unchanged_importer() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "package.json",
        r#"{"name":"self","exports":"./before.ts"}"#,
    );
    write(root.path(), "before.ts", "export class Leaf {}");
    write(root.path(), "after.ts", "export class Leaf {}");
    write(root.path(), "app.ts", "import { Leaf } from 'self';");
    let mut before = resolver(root.path());
    let Resolution::Resolved(old) =
        before.local(&root.path().join("app.ts"), "Leaf", Namespace::Value)
    else {
        panic!("self declaration control");
    };
    write(
        root.path(),
        "package.json",
        r#"{"name":"self","exports":"./after.ts"}"#,
    );
    assert!(!before.host.current());
    let Resolution::Resolved(new) =
        resolver(root.path()).local(&root.path().join("app.ts"), "Leaf", Namespace::Value)
    else {
        panic!("retargeted self declaration");
    };
    assert_ne!(old.source, new.source);
}

#[test]
fn issue_162_external_package_import_target_uses_package_export_authority() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "package.json",
        r##"{"imports":{"#leaf":"pkg/leaf"}}"##,
    );
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{"./leaf":"./real.ts"}}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/real.ts",
        "export class Leaf {}",
    );
    let mut resolve = resolver(root.path());
    assert!(
        resolve
            .module(&root.path().join("app.ts"), "#leaf")
            .unwrap()
            .ends_with("real.ts")
    );
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{"./leaf":null}}"#,
    );
    assert!(!resolve.host.current());
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "#leaf")
            .is_err()
    );
}

#[test]
fn issue_162_package_import_alias_cycle_fails_closed_and_retarget_restores_identity() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "package.json",
        r##"{"imports":{"#a":"#b","#b":"#a"}}"##,
    );
    let mut cyclic = resolver(root.path());
    assert!(
        cyclic
            .module(&root.path().join("app.ts"), "#a")
            .unwrap_err()
            .contains("Cyclic")
    );
    write(root.path(), "leaf.ts", "export class Leaf {}");
    write(
        root.path(),
        "package.json",
        r##"{"imports":{"#a":"#b","#b":"./leaf.ts"}}"##,
    );
    assert!(!cyclic.host.current());
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "#a")
            .unwrap()
            .ends_with("leaf.ts")
    );
}

#[test]
fn issue_162_invalid_nearest_manifest_does_not_grant_foreign_package_identity() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(root.path(), "package.json", "{invalid");
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":"./leaf.ts"}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/leaf.ts",
        "export class Leaf {}",
    );
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "pkg")
            .is_err()
    );
}

#[test]
fn issue_162_export_maps_do_not_guess_directory_index_or_escape_package() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/dir/index.ts",
        "export class Leaf {}",
    );
    write(root.path(), "outside.ts", "export class Leaf {}");
    for target in ["./dir", "./../../outside.ts", "./%2e%2e/outside.ts"] {
        write(
            root.path(),
            "node_modules/pkg/package.json",
            &format!(r#"{{"exports":"{target}"}}"#),
        );
        assert!(
            resolver(root.path())
                .module(&root.path().join("app.ts"), "pkg")
                .is_err(),
            "{target}"
        );
    }
}

#[test]
fn issue_162_package_without_manifest_can_resolve_index_and_manifest_appearance_invalidates() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/index.ts",
        "export class Leaf {}",
    );
    let mut before = resolver(root.path());
    assert!(
        before
            .module(&root.path().join("app.ts"), "pkg")
            .unwrap()
            .ends_with("index.ts")
    );
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":{".":null}}"#,
    );
    assert!(!before.host.current());
    assert!(
        resolver(root.path())
            .module(&root.path().join("app.ts"), "pkg")
            .is_err()
    );
}

#[test]
fn issue_162_package_maps_cannot_turn_local_targets_into_host_absolute_paths() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(root.path(), "outside.ts", "export class Leaf {}");
    write(
        root.path(),
        "node_modules/pkg/leaf.ts",
        "export class Leaf {}",
    );
    let outside = root
        .path()
        .join("outside.ts")
        .to_str()
        .unwrap()
        .replace('\\', "/");
    let file = root.path().join("app.ts");
    // Both candidate sources remain inside the authorized workspace. Package
    // locality is an additional authority boundary, independent of root scope.
    for target in [
        format!("./{outside}"),
        ".//outside.ts".into(),
        "./C:/outside.ts".into(),
    ] {
        write(
            root.path(),
            "node_modules/pkg/package.json",
            &format!(r#"{{"exports":"{target}"}}"#),
        );
        let mut invalid = resolver(root.path());
        assert!(invalid.module(&file, "pkg").is_err(), "{target}");
    }
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"exports":"./leaf.ts"}"#,
    );
    let mut valid = resolver(root.path());
    assert!(valid.module(&file, "pkg").unwrap().ends_with("leaf.ts"));
    write(
        root.path(),
        "node_modules/pkg/package.json",
        &format!(r#"{{"exports":"./{outside}"}}"#),
    );
    assert!(!valid.host.current());
    assert!(resolver(root.path()).module(&file, "pkg").is_err());
}

#[test]
fn issue_162_private_import_targets_share_package_local_validation() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(root.path(), "leaf.ts", "export class Leaf {}");
    let file = root.path().join("app.ts");
    write(
        root.path(),
        "package.json",
        r##"{"imports":{"#leaf":"./leaf.ts"}}"##,
    );
    let mut valid = resolver(root.path());
    assert!(valid.module(&file, "#leaf").unwrap().ends_with("leaf.ts"));
    write(
        root.path(),
        "package.json",
        r##"{"imports":{"#leaf":"./C:/leaf.ts"}}"##,
    );
    assert!(!valid.host.current());
    assert!(resolver(root.path()).module(&file, "#leaf").is_err());
}

#[test]
fn issue_162_invalid_primitive_targets_allow_ordered_fallback_but_blocks_do_not() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/leaf.ts",
        "export class Leaf {}",
    );
    let file = root.path().join("app.ts");
    for exports in [
        r#"[7,false,"./leaf.ts"]"#,
        r#"{"types":7,"default":"./leaf.ts"}"#,
        r#"[{"foreign":"./absent.ts"},true,"./leaf.ts"]"#,
    ] {
        write(
            root.path(),
            "node_modules/pkg/package.json",
            &format!(r#"{{"exports":{exports}}}"#),
        );
        let mut current = resolver(root.path());
        assert!(
            current.module(&file, "pkg").unwrap().ends_with("leaf.ts"),
            "{exports}"
        );
        write(
            root.path(),
            "node_modules/pkg/package.json",
            r#"{"exports":[null,"./leaf.ts"]}"#,
        );
        assert!(!current.host.current());
        assert!(resolver(root.path()).module(&file, "pkg").is_err());
    }
    for exports in [
        r#"[[],"./leaf.ts"]"#,
        r#"{"types":[],"default":"./leaf.ts"}"#,
    ] {
        write(
            root.path(),
            "node_modules/pkg/package.json",
            &format!(r#"{{"exports":{exports}}}"#),
        );
        assert!(
            resolver(root.path()).module(&file, "pkg").is_err(),
            "{exports}"
        );
    }
}

#[test]
fn issue_162_selected_missing_target_does_not_fall_through_to_another_file() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/leaf.ts",
        "export class Leaf {}",
    );
    let file = root.path().join("app.ts");
    for exports in [
        r#"["./absent.ts","./leaf.ts"]"#,
        r#"{"types":"./absent.ts","default":"./leaf.ts"}"#,
    ] {
        write(
            root.path(),
            "node_modules/pkg/package.json",
            &format!(r#"{{"exports":{exports}}}"#),
        );
        let mut missing = resolver(root.path());
        assert!(missing.module(&file, "pkg").is_err());
        write(
            root.path(),
            "node_modules/pkg/absent.ts",
            "export class Present {}",
        );
        assert!(!missing.host.current());
        assert!(
            resolver(root.path())
                .module(&file, "pkg")
                .unwrap()
                .ends_with("absent.ts")
        );
        std::fs::remove_file(root.path().join("node_modules/pkg/absent.ts")).unwrap();
    }
}

#[test]
fn issue_162_private_imports_share_invalid_primitive_fallback_rules() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(root.path(), "leaf.ts", "export class Leaf {}");
    write(
        root.path(),
        "package.json",
        r##"{"imports":{"#leaf":[false,"./leaf.ts"]}}"##,
    );
    let file = root.path().join("app.ts");
    let mut valid = resolver(root.path());
    assert!(valid.module(&file, "#leaf").unwrap().ends_with("leaf.ts"));
    write(
        root.path(),
        "package.json",
        r##"{"imports":{"#leaf":[null,"./leaf.ts"]}}"##,
    );
    assert!(!valid.host.current());
    assert!(resolver(root.path()).module(&file, "#leaf").is_err());
}
