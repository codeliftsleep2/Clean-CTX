use super::{Host, TypeScriptAuthority, project::Project};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
}

#[test]
fn issue_162_jsconfig_default_and_tsconfig_appearance_change_context() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.js", "export class App {}");
    write(
        root.path(),
        "jsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler"}}"#,
    );
    let file = root.path().join("app.js");
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let project = Project::discover(&mut host, &file).unwrap();
    assert!(project.config.unwrap().ends_with("jsconfig.json"));
    assert!(project.ownership.unwrap().root_member);
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler"}}"#,
    );
    assert!(!host.current());
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let project = Project::discover(&mut host, &file).unwrap();
    assert!(project.config.unwrap().ends_with("tsconfig.json"));
    assert!(!project.ownership.unwrap().root_member);
}

#[test]
fn issue_162_jsconfig_defaults_override_base_but_explicit_false_wins() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.js", "export class App {}");
    write(
        root.path(),
        "base.json",
        r#"{"compilerOptions":{"allowJs":false,"module":"esnext","moduleResolution":"bundler"}}"#,
    );
    write(root.path(), "jsconfig.json", r#"{"extends":"./base"}"#);
    let file = root.path().join("app.js");
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        Project::discover(&mut host, &file)
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
    write(
        root.path(),
        "jsconfig.json",
        r#"{"extends":"./base","compilerOptions":{"allowJs":false}}"#,
    );
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        !Project::discover(&mut host, &file)
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
}

#[test]
fn issue_162_nullable_roots_inherit_and_null_options_reset() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "src/app.ts", "export class App {}");
    write(
        root.path(),
        "base/base.json",
        r#"{"include":["./src/**/*"],"compilerOptions":{"module":"esnext","moduleResolution":"bundler","paths":{"@base":["./other.ts"]}}}"#,
    );
    write(root.path(), "base/src/leaf.ts", "export class Leaf {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"extends":"./base/base","include":null,"files":null,"exclude":null,"references":null,"compilerOptions":{"paths":null}}"#,
    );
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let project = Project::discover(&mut host, &root.path().join("base/src/leaf.ts")).unwrap();
    assert!(project.ownership.unwrap().root_member);
    assert!(project.paths.is_empty());
    assert_eq!(project.mode, "bundler");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"extends":"./base/base","compilerOptions":null}"#,
    );
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        !Project::discover(&mut host, &root.path().join("base/src/leaf.ts"))
            .unwrap()
            .paths
            .is_empty()
    );
}

#[test]
fn issue_162_explicit_context_resolves_overlap_without_first_match() {
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
    let file = root.path().join("app.ts");
    let inferred =
        TypeScriptAuthority::capture(&file, "export class App {}", &[root.path().to_path_buf()]);
    assert!(
        inferred
            .disposition
            .as_deref()
            .unwrap()
            .contains("Ambiguous")
    );
    for name in ["one.json", "two.json"] {
        let explicit = TypeScriptAuthority::capture_in_project(
            &file,
            "export class App {}",
            &[root.path().to_path_buf()],
            Some(&root.path().join(name)),
        );
        assert!(explicit.disposition.is_none(), "{:?}", explicit.disposition);
        assert!(explicit.project.unwrap().1.unwrap().ends_with(name));
        assert!(explicit.project_ownership.unwrap().selected_explicitly);
    }
}

#[test]
fn issue_162_explicit_context_never_authorizes_foreign_config() {
    let root = tempfile::tempdir().unwrap();
    let foreign = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        foreign.path(),
        "project.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    );
    let evidence = TypeScriptAuthority::capture_in_project(
        &root.path().join("app.ts"),
        "export class App {}",
        &[root.path().to_path_buf()],
        Some(&foreign.path().join("project.json")),
    );
    assert!(evidence.disposition.is_some());
    assert!(!evidence.capture_current);
    assert!(!evidence.witnesses.iter().any(|witness| matches!(witness, super::Witness::Content { path, .. } if path.ends_with("project.json"))));
}

#[test]
fn issue_162_null_option_reset_survives_multiple_extends_layers() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "one.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","paths":{"@old":["./old.ts"]}}}"#,
    );
    write(
        root.path(),
        "two.json",
        r#"{"extends":"./one","compilerOptions":{"paths":null}}"#,
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"extends":["./one","./two"]}"#,
    );
    let file = root.path().join("app.ts");
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let project = Project::discover(&mut host, &file).unwrap();
    assert!(project.paths.is_empty());
    assert_eq!(project.mode, "bundler");
    assert!(host.current());
    write(root.path(), "two.json", r#"{"extends":"./one"}"#);
    assert!(!host.current());
}

#[test]
fn issue_162_capture_retains_effective_options_and_their_source_origins() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "base/base.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","preserveSymlinks":true,"rootDirs":["./sources","./generated"],"resolvePackageJsonExports":false}}"#,
    );
    write(root.path(), "tsconfig.json", r#"{"extends":"./base/base"}"#);
    let evidence = TypeScriptAuthority::capture(
        &root.path().join("app.ts"),
        "export class App {}",
        &[root.path().to_path_buf()],
    );
    assert!(evidence.disposition.is_none());
    assert_eq!(
        evidence.compiler_options["preserveSymlinks"].value,
        serde_json::json!(true)
    );
    assert_eq!(
        evidence.compiler_options["resolvePackageJsonExports"].value,
        serde_json::json!(false)
    );
    assert!(
        evidence.compiler_options["rootDirs"]
            .directory
            .ends_with("base")
    );
    let encoded = serde_json::to_vec(&evidence).unwrap();
    let retained: TypeScriptAuthority = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(
        retained.compiler_options["rootDirs"].value,
        serde_json::json!(["./sources", "./generated"])
    );
    write(root.path(), "base/base.json", "{}");
    assert!(!retained.witnesses.iter().all(super::Witness::current));
}

#[cfg(unix)]
#[test]
fn issue_162_capture_alias_preserves_route_but_has_one_physical_declaration_identity() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "real/app.ts", "export class App {}");
    std::os::unix::fs::symlink(root.path().join("real"), root.path().join("alias")).unwrap();
    let route = root.path().join("alias/app.ts");
    let physical = root.path().join("real/app.ts").canonicalize().unwrap();
    let evidence =
        TypeScriptAuthority::capture(&route, "export class App {}", &[root.path().to_path_buf()]);
    assert_eq!(evidence.source, physical);
    assert_eq!(evidence.source_route, route);
    assert_eq!(evidence.declarations[0].source, physical);
    assert!(
        evidence
            .references
            .iter()
            .any(|(name, _, resolution)| name == "App"
                && matches!(resolution, super::Resolution::Resolved(id) if id.source == physical))
    );
}

#[cfg(unix)]
#[test]
fn issue_162_host_does_not_merge_lossy_physical_root_identities() {
    use std::os::unix::ffi::OsStringExt;
    let root = tempfile::tempdir().unwrap();
    let physical = root
        .path()
        .join(std::ffi::OsString::from_vec(vec![b'r', 0xff]));
    std::fs::create_dir(&physical).unwrap();
    let alias = root.path().join("alias");
    std::os::unix::fs::symlink(&physical, &alias).unwrap();
    assert!(Host::new(&[alias]).is_err());
}
