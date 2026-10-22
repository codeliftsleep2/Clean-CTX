use super::project::Project;
use super::{Host, TypeScriptAuthority};
use std::path::Path;

fn write(root: &Path, name: &str, text: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}

#[test]
fn issue_162_project_reference_selects_owner_and_retarget_invalidates() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "src/app.ts", "export class App {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./projects/app.json"}]}"#,
    );
    write(
        root.path(),
        "projects/app.json",
        r#"{"include":["../src/**/*.ts"],"compilerOptions":{"module":"esnext","moduleResolution":"bundler"}}"#,
    );
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let project = Project::discover(&mut host, &root.path().join("src/app.ts")).unwrap();
    assert!(project.config.unwrap().ends_with("projects/app.json"));
    let ownership = project.ownership.unwrap();
    assert!(ownership.root_member);
    assert_eq!(ownership.configurations.len(), 2);
    assert_eq!(ownership.root_candidates.len(), 1);
    assert!(host.current());
    write(root.path(), "projects/app.json", r#"{"files":[]}"#);
    assert!(!host.current());
}

#[test]
fn issue_162_project_overlapping_references_do_not_choose_first() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./one.json"},{"path":"./two.json"}]}"#,
    );
    for name in ["one.json", "two.json"] {
        write(
            root.path(),
            name,
            r#"{"files":["app.ts"],"compilerOptions":{"module":"esnext","moduleResolution":"bundler"}}"#,
        );
    }
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        Project::discover(&mut host, &root.path().join("app.ts"))
            .unwrap_err()
            .contains("Ambiguous")
    );
}

#[test]
fn issue_162_project_inherited_root_patterns_and_explicit_files_keep_origins() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "base/src/app.ts", "export class App {}");
    write(
        root.path(),
        "base/base.json",
        r#"{"include":["./src"],"exclude":["./src/ignored"],"compilerOptions":{"module":"esnext","moduleResolution":"bundler"}}"#,
    );
    write(root.path(), "tsconfig.json", r#"{"extends":"./base/base"}"#);
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let project = Project::discover(&mut host, &root.path().join("base/src/app.ts")).unwrap();
    assert!(project.ownership.unwrap().root_member);
    write(
        root.path(),
        "base/src/ignored/app.ts",
        "export class Ignored {}",
    );
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let project =
        Project::discover(&mut host, &root.path().join("base/src/ignored/app.ts")).unwrap();
    assert!(!project.ownership.unwrap().root_member);
    write(
        root.path(),
        "tsconfig.json",
        r#"{"extends":"./base/base","files":["./base/src/ignored/app.ts"]}"#,
    );
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        Project::discover(&mut host, &root.path().join("base/src/ignored/app.ts"))
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
}

#[test]
fn issue_162_project_references_are_not_inherited_and_cycles_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "base.json",
        r#"{"references":[{"path":"./absent.json"}],"compilerOptions":{"module":"esnext","moduleResolution":"bundler"}}"#,
    );
    write(root.path(), "tsconfig.json", r#"{"extends":"./base"}"#);
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        Project::discover(&mut host, &root.path().join("app.ts"))
            .unwrap()
            .ownership
            .unwrap()
            .configurations
            .len(),
        1
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"references":[{"path":"./tsconfig.json"}]}"#,
    );
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        Project::discover(&mut host, &root.path().join("app.ts"))
            .unwrap_err()
            .contains("Cyclic")
    );
}

#[test]
fn issue_162_project_mode_default_requires_installed_compiler_and_tracks_version() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"preserve"}}"#,
    );
    let file = root.path().join("app.ts");
    let mut before = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        Project::discover(&mut before, &file).unwrap().mode,
        "unresolved"
    );
    write(
        root.path(),
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    assert!(!before.current());
    let authority =
        TypeScriptAuthority::capture(&file, "export class App {}", &[root.path().to_path_buf()]);
    assert_eq!(authority.project.unwrap().2, "bundler");
    assert_eq!(authority.compiler_version.as_deref(), Some("5.9.3"));
    write(
        root.path(),
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.3.3"}"#,
    );
    assert!(!authority.witnesses.iter().all(super::Witness::current));
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        Project::discover(&mut host, &file).is_err(),
        "TypeScript 5.3 does not admit module preserve"
    );
}

#[test]
fn issue_162_project_target_default_uses_versioned_module_rule() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    for (config, expected) in [
        ("{}", "node10"),
        (r#"{"compilerOptions":{"target":"ES2020"}}"#, "classic"),
    ] {
        write(root.path(), "tsconfig.json", config);
        let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
        assert_eq!(
            Project::discover(&mut host, &root.path().join("app.ts"))
                .unwrap()
                .mode,
            expected
        );
    }
    write(
        root.path(),
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"99.0.0"}"#,
    );
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        Project::discover(&mut host, &root.path().join("app.ts"))
            .unwrap()
            .mode,
        "unresolved"
    );
}

#[test]
fn issue_162_project_missing_reference_restoration_revalidates_context() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./owner.json"}]}"#,
    );
    let mut before = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(Project::discover(&mut before, &root.path().join("app.ts")).is_err());
    write(
        root.path(),
        "owner.json",
        r#"{"files":["app.ts"],"compilerOptions":{"module":"esnext","moduleResolution":"bundler"}}"#,
    );
    assert!(!before.current());
    let mut fresh = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        Project::discover(&mut fresh, &root.path().join("app.ts"))
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
    std::fs::remove_file(root.path().join("owner.json")).unwrap();
    assert!(!fresh.current());
}

#[cfg(windows)]
#[test]
fn issue_162_project_host_windows_extended_and_case_alias_share_identity() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "Leaf.ts", "export class Leaf {}");
    let file = root.path().join("Leaf.ts");
    let extended = file.canonicalize().unwrap();
    let lower = std::path::PathBuf::from(file.to_string_lossy().to_lowercase());
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(host.normalized(&file), host.normalized(&extended));
    assert_eq!(host.normalized(&file), host.normalized(&lower));
    assert_eq!(host.read(&file), host.read(&extended));
    assert!(host.current());
}
