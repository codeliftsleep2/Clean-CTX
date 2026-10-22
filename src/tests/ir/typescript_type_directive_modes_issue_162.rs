use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, file: &str, source: &str) {
    let file = root.join(file);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"NodeNext","moduleResolution":"nodenext","moduleDetection":"legacy","noLib":true,"types":[]},"files":["app.ts"]}"#,
    );
    write(root, "package.json", r#"{"type":"commonjs"}"#);
    write(
        root,
        "node_modules/@types/support/legacy.d.ts",
        "declare namespace API { class Selected {} }",
    );
    write(
        root,
        "node_modules/@types/support/import.d.ts",
        "declare namespace API { class Selected {} }",
    );
    write(
        root,
        "node_modules/@types/support/require.d.ts",
        "declare namespace API { class Selected {} }",
    );
}
fn manifest(root: &Path, conditional: bool) {
    let mut json = serde_json::json!({"name":"@types/support","types":"legacy.d.ts"});
    if conditional {
        json["exports"] =
            serde_json::json!({".":{"import":"./import.d.ts","require":"./require.d.ts"}});
    }
    write(
        root,
        "node_modules/@types/support/package.json",
        &json.to_string(),
    );
}
fn request(root: &Path, mode: Option<&str>) -> Resolution {
    let attribute = mode
        .map(|mode| format!(" resolution-mode=\"{mode}\""))
        .unwrap_or_default();
    write(
        root,
        "app.ts",
        &format!("/// <reference types=\"support\"{attribute} />\nexport class Control {{}}"),
    );
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let file = root.join("app.ts");
    assert!(matches!(
        resolver.local(&file, "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    resolver.namespace_member(&file, "API", "Selected", Namespace::Value)
}
fn source(result: Resolution, suffix: &str) {
    let Resolution::Resolved(id) = result else {
        panic!("directive positive: {result:?}");
    };
    assert!(id.source.ends_with(suffix), "{id:?}");
}

#[test]
fn issue_162_type_directive_mode_explicit_import_require_select_export_conditions() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    manifest(root.path(), false);
    source(request(root.path(), None), "legacy.d.ts");
    manifest(root.path(), true);
    source(request(root.path(), Some("import")), "import.d.ts");
    source(request(root.path(), Some("require")), "require.d.ts");
}

#[test]
fn issue_162_type_directive_mode_default_tracks_containing_package_format() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    manifest(root.path(), false);
    source(request(root.path(), None), "legacy.d.ts");
    manifest(root.path(), true);
    for (format, expected) in [
        ("commonjs", "require.d.ts"),
        ("module", "import.d.ts"),
        ("commonjs", "require.d.ts"),
    ] {
        write(
            root.path(),
            "package.json",
            &serde_json::json!({"type":format}).to_string(),
        );
        source(request(root.path(), None), expected);
    }
}

#[test]
fn issue_162_type_directive_mode_invalid_attribute_refuses_after_positive_control() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    manifest(root.path(), false);
    source(request(root.path(), None), "legacy.d.ts");
    for invalid in ["IMPORT", "node", "", "require import"] {
        assert!(
            matches!(
                request(root.path(), Some(invalid)),
                Resolution::Unresolved(_)
            ),
            "{invalid}"
        );
    }
}
