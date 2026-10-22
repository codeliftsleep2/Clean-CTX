use super::{Host, project::Project};
use std::path::Path;

fn write(root: &Path, file: &str, source: &str) {
    let file = root.join(file);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
}

fn project(root: &Path, file: &str) -> Result<Project, String> {
    Project::discover(
        &mut Host::new(&[root.to_path_buf()]).unwrap(),
        &root.join(file),
    )
}

#[test]
fn issue_162_project_ts6_defaults_are_versioned_and_config_sensitive() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"6.0.2"}"#,
    );
    for (module, expected) in [
        (None, "bundler"),
        (Some("commonjs"), "bundler"),
        (Some("esnext"), "bundler"),
        (Some("nodenext"), "nodenext"),
        (Some("amd"), "classic"),
    ] {
        let config = module.map_or_else(
            || serde_json::json!({}),
            |module| serde_json::json!({"compilerOptions":{"module": module}}),
        );
        write(root.path(), "tsconfig.json", &config.to_string());
        assert_eq!(project(root.path(), "app.ts").unwrap().mode, expected);
    }
}

#[test]
fn issue_162_project_module_resolution_conflict_does_not_produce_context() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    for config in [
        r#"{"compilerOptions":{"module":"nodenext","moduleResolution":"bundler"}}"#,
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"node16"}}"#,
        r#"{"compilerOptions":{"moduleResolution":"node10","customConditions":["custom"]}}"#,
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","allowJs":"yes"}}"#,
    ] {
        write(root.path(), "tsconfig.json", config);
        assert!(project(root.path(), "app.ts").is_err(), "{config}");
    }
}

#[test]
fn issue_162_project_commonjs_bundler_requires_compatible_version() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"commonjs","moduleResolution":"bundler"}}"#,
    );
    for (version, accepted) in [("5.9.3", false), ("6.0.2", true), ("6.0.2-beta", false)] {
        write(
            root.path(),
            "node_modules/typescript/package.json",
            &serde_json::json!({"name":"typescript","version":version}).to_string(),
        );
        assert_eq!(project(root.path(), "app.ts").is_ok(), accepted);
    }
}

#[test]
fn issue_162_project_wildcards_distinguish_explicit_and_implicit_dependency_paths() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "node_modules/pkg/leaf.ts",
        "export class Leaf {}",
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler"},"exclude":[]}"#,
    );
    assert!(
        !project(root.path(), "node_modules/pkg/leaf.ts")
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler"},"include":["node_modules/pkg/**/*"],"exclude":[]}"#,
    );
    assert!(
        project(root.path(), "node_modules/pkg/leaf.ts")
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
}

#[test]
fn issue_162_project_exclude_wildcards_match_hidden_paths_and_directory_prefixes() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), ".hidden/leaf.ts", "export class Leaf {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler"},"include":[".hidden/**/*"],"exclude":[]}"#,
    );
    assert!(
        project(root.path(), ".hidden/leaf.ts")
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler"},"include":[".hidden/**/*"],"exclude":["**/.hidden"]}"#,
    );
    assert!(
        !project(root.path(), ".hidden/leaf.ts")
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
}

#[test]
fn issue_162_project_checkjs_and_minified_file_patterns_are_not_name_guesses() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "src/app.js", "export class App {}");
    write(root.path(), "src/app.min.js", "export class App {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","checkJs":true}}"#,
    );
    assert!(
        project(root.path(), "src/app.js")
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
    assert!(
        !project(root.path(), "src/app.min.js")
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","checkJs":true},"include":["src/app.min.js"]}"#,
    );
    assert!(
        project(root.path(), "src/app.min.js")
            .unwrap()
            .ownership
            .unwrap()
            .root_member
    );
}

#[test]
fn issue_162_project_configdir_uses_consuming_config_not_inherited_directory() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "src/app.ts", "export class App {}");
    write(
        root.path(),
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    write(
        root.path(),
        "base/base.json",
        r#"{"include":["${configDir}/src/**/*"],"compilerOptions":{"module":"esnext","moduleResolution":"bundler","paths":{"@app/*":["${configDir}/src/*"]}}}"#,
    );
    write(root.path(), "tsconfig.json", r#"{"extends":"./base/base"}"#);
    let project = project(root.path(), "src/app.ts").unwrap();
    assert!(project.ownership.unwrap().root_member);
    assert_eq!(
        project.paths["@app/*"][0]
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap(),
        root.path().join("src").canonicalize().unwrap()
    );
    assert_eq!(project.paths["@app/*"][0].file_name().unwrap(), "*");
}

#[test]
fn issue_162_project_configdir_without_version_and_invalid_wildcards_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    for config in [
        r#"{"include":["${configDir}/*"]}"#,
        r#"{"include":["src/**"]}"#,
        r#"{"include":["src/**/../*"]}"#,
        r#"{"compilerOptions":{"paths":{"@a*":["a**"]}}}"#,
    ] {
        write(root.path(), "tsconfig.json", config);
        assert!(project(root.path(), "app.ts").is_err(), "{config}");
    }
}

#[test]
fn issue_162_project_configdir_embedded_text_is_literal_and_prefix_expands_once() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "src/app.ts", "export class App {}");
    write(
        root.path(),
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","paths":{"@literal":["./prefix/${configDir}/leaf.ts"],"@prefix":["${configDir}src/../src/app.ts"]}}}"#,
    );
    let context = project(root.path(), "src/app.ts").unwrap();
    assert!(
        context.paths["@literal"][0]
            .to_string_lossy()
            .contains("${configDir}")
    );
    assert_eq!(
        context.paths["@prefix"][0].canonicalize().unwrap(),
        root.path().join("src/app.ts").canonicalize().unwrap()
    );
}

#[test]
fn issue_162_project_embedded_configdir_does_not_require_version_evidence() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","paths":{"@literal":["./prefix/${configDir}/leaf.ts"]}}}"#,
    );
    let context = project(root.path(), "app.ts").unwrap();
    assert!(context.compiler_version.is_none());
    assert!(
        context.paths["@literal"][0]
            .to_string_lossy()
            .contains("${configDir}")
    );
}

#[test]
fn issue_162_project_legacy_compiler_defaults_and_newer_options_are_distinct() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    for version in ["4.0.8", "4.6.4"] {
        write(
            root.path(),
            "node_modules/typescript/package.json",
            &serde_json::json!({"name":"typescript","version":version}).to_string(),
        );
        for (module, expected) in [("commonjs", "node10"), ("es2015", "classic")] {
            write(
                root.path(),
                "tsconfig.json",
                &serde_json::json!({"compilerOptions":{"module":module}}).to_string(),
            );
            assert_eq!(project(root.path(), "app.ts").unwrap().mode, expected);
        }
        write(
            root.path(),
            "tsconfig.json",
            r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler"}}"#,
        );
        assert!(project(root.path(), "app.ts").is_err());
        write(
            root.path(),
            "tsconfig.json",
            r#"{"compilerOptions":{"module":"node16","moduleResolution":"node16"}}"#,
        );
        assert!(project(root.path(), "app.ts").is_err());
    }
}

#[test]
fn issue_162_project_bom_config_certifies_original_consumed_bytes() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class App {}");
    let config = format!(
        "\u{feff}{}",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler"}}"#
    );
    write(root.path(), "tsconfig.json", &config);
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        Project::discover(&mut host, &root.path().join("app.ts"))
            .unwrap()
            .mode,
        "bundler"
    );
    assert!(host.witnesses.iter().any(|witness| {
        matches!(witness, super::Witness::Content { path, hash } if path.ends_with("tsconfig.json") && hash == &crate::cache::content_hash(config.as_bytes()))
    }));
    write(
        root.path(),
        "tsconfig.json",
        config.trim_start_matches('\u{feff}'),
    );
    assert!(!host.current());
}
