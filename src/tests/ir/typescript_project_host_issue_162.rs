use super::project::Project;
use super::{Host, TypeScriptAuthority, Witness};
use std::path::Path;

fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[test]
fn issue_162_project_host_consumed_snapshot_cannot_be_reseeded() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class Before {}");
    let file = root.path().join("app.ts");
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let before = host.read(&file).unwrap();
    assert!(host.seed(&file, "export class After {}").is_err());
    assert_eq!(host.read(&file).unwrap(), before);
    assert!(host.seed(&file, &before).is_ok());
}

#[test]
fn issue_162_project_host_invalid_utf8_is_not_certified_as_absence() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    std::fs::write(&file, [0xff, 0xfe]).unwrap();
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(host.read(&file).is_none());
    assert!(!host.current());
    assert!(
        !host
            .witnesses
            .iter()
            .any(|witness| matches!(witness, Witness::Missing { path } if path == &file))
    );
}

#[test]
fn issue_162_project_host_failed_config_capture_retains_consulted_witnesses() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(root.path(), "tsconfig.json", "{ invalid JSON }");
    let authority = TypeScriptAuthority::capture(
        &root.path().join("app.ts"),
        "export class App {}",
        &[root.path().to_path_buf()],
    );
    assert!(authority.disposition.is_some());
    assert!(!authority.capture_current);
    assert_eq!(authority.declarations.len(), 1);
    assert!(authority.witnesses.iter().any(|witness| matches!(witness, Witness::Content { path, .. } if path.ends_with("tsconfig.json"))));
}

#[test]
fn issue_162_project_host_config_appearance_invalidates_source_only_observation() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "src/app.ts", "export class App {}");
    let file = root.path().join("src/app.ts");
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let project = Project::discover(&mut host, &file).unwrap();
    assert_eq!(project.mode, "unresolved");
    assert!(project.config.is_none());
    assert!(host.current());
    write(root.path(), "tsconfig.json", "{}");
    assert!(!host.current());
}

#[test]
fn issue_162_project_host_source_snapshot_does_not_authorize_parent() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(root.path(), "tsconfig.json", "{}");
    let file = root.path().join("app.ts");
    let evidence = TypeScriptAuthority::capture(&file, "export class App {}", &[]);
    assert!(evidence.source_only);
    assert_eq!(evidence.declarations.len(), 1);
    assert_eq!(evidence.project.as_ref().unwrap().2, "unresolved");
    assert!(!evidence.witnesses.iter().any(|witness| {
        matches!(witness, Witness::Content { path, .. } if path.ends_with("tsconfig.json"))
    }));
}

#[test]
fn issue_162_project_host_multiple_extends_merge_keys_and_preserve_origins() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "base/one.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler","baseUrl":"./sources","paths":{"@app/*":["*"]}}}"#,
    );
    write(
        root.path(),
        "base/two.json",
        r#"{"compilerOptions":{"customConditions":["test"]}}"#,
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"extends":["./base/one","./base/two"]}"#,
    );
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let project = Project::discover(&mut host, &root.path().join("app.ts")).unwrap();
    assert_eq!(project.mode, "bundler");
    assert!(project.base.ends_with("base/sources"));
    assert!(project.paths["@app/*"][0].ends_with("base/sources/*"));
    assert!(
        project
            .conditions
            .iter()
            .any(|condition| condition == "test")
    );
    write(root.path(), "base/one.json", "{}");
    assert!(!host.current());
}

#[test]
fn issue_162_project_host_package_extends_is_consumed_and_invalidated() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"extends":"@example/config"}"#,
    );
    write(
        root.path(),
        "node_modules/@example/config/package.json",
        r#"{"tsconfig":"./base.json"}"#,
    );
    write(
        root.path(),
        "node_modules/@example/config/base.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    );
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        Project::discover(&mut host, &root.path().join("app.ts"))
            .unwrap()
            .mode,
        "bundler"
    );
    assert!(host.current());
    write(
        root.path(),
        "node_modules/@example/config/package.json",
        r#"{"tsconfig":"./other.json"}"#,
    );
    assert!(!host.current());
}

#[test]
fn issue_162_project_host_directory_read_is_not_certified_as_missing() {
    let root = tempfile::tempdir().unwrap();
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(host.read(root.path()).is_none());
    assert!(
        host.witnesses
            .iter()
            .any(|witness| matches!(witness, Witness::Unreadable { .. }))
    );
    assert!(!host.current());
}

#[test]
fn issue_162_project_host_extends_cycle_and_outside_root_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(root.path(), "tsconfig.json", r#"{"extends":"./base"}"#);
    write(root.path(), "base.json", r#"{"extends":"./tsconfig"}"#);
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        Project::discover(&mut host, &root.path().join("app.ts"))
            .unwrap_err()
            .contains("Cyclic")
    );
    let foreign = tempfile::tempdir().unwrap();
    write(
        foreign.path(),
        "base.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    );
    let config = serde_json::json!({"extends": foreign.path().join("base.json")});
    write(root.path(), "tsconfig.json", &config.to_string());
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(Project::discover(&mut host, &root.path().join("app.ts")).is_err());
    assert!(
        host.witnesses
            .iter()
            .any(|witness| matches!(witness, Witness::OutsideScope { .. }))
    );
}

#[cfg(unix)]
#[test]
fn issue_162_project_host_symlink_escape_and_dangling_link_are_not_absence() {
    let root = tempfile::tempdir().unwrap();
    let foreign = tempfile::tempdir().unwrap();
    write(foreign.path(), "leaf.ts", "export class Leaf {}");
    std::os::unix::fs::symlink(foreign.path(), root.path().join("escape")).unwrap();
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(host.read(&root.path().join("escape/leaf.ts")).is_none());
    assert!(!host.current());
    std::os::unix::fs::symlink(root.path().join("missing"), root.path().join("dangling")).unwrap();
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(host.read(&root.path().join("dangling")).is_none());
    assert!(
        host.witnesses
            .iter()
            .any(|witness| matches!(witness, Witness::Unreadable { .. }))
    );
}

#[cfg(unix)]
#[test]
fn issue_162_project_host_authorized_root_alias_preserves_physical_identity() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "real/leaf.ts", "export class Leaf {}");
    let route = root.path().join("alias");
    std::os::unix::fs::symlink(root.path().join("real"), &route).unwrap();
    let mut host = Host::new(&[route.clone()]).unwrap();
    assert!(host.read(&route.join("leaf.ts")).is_some());
    assert_eq!(
        host.normalized(&route.join("leaf.ts")).unwrap(),
        root.path().join("real/leaf.ts").canonicalize().unwrap()
    );
    assert!(host.current());
}
