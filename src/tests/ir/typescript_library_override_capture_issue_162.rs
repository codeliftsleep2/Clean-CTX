use super::{Resolution, TypeScriptAuthority};
use std::path::Path;

fn write(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}
fn setup(root: &Path, library: &str) {
    write(
        root,
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    write(root, "tsconfig.json", &serde_json::json!({"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","types":[],"lib":[library]},"files":["app.ts"]}).to_string());
}

#[test]
fn issue_162_library_override_capture_multisegment_path_and_provenance_survive() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "es2015.symbol.wellknown");
    write(
        root.path(),
        "node_modules/@typescript/lib-es2015/package.json",
        r#"{"name":"@typescript/lib-es2015","types":"index.d.ts"}"#,
    );
    let declaration = "declare namespace API { class Override {} }";
    write(
        root.path(),
        "node_modules/@typescript/lib-es2015/symbol-wellknown.d.ts",
        declaration,
    );
    let file = root.path().join("app.ts");
    let source = "export class Control {} API.Override;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let request = capture
        .compiler_libraries
        .first()
        .expect("retained override request");
    assert_eq!(
        request.override_specifier.as_deref(),
        Some("@typescript/lib-es2015/symbol-wellknown")
    );
    assert!(
        request
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("symbol-wellknown.d.ts")
    );
    assert!(capture.reference_sites.iter().any(|(_, _, result)| matches!(result,
        Resolution::Resolved(id) if id.name == "Override" && id.source_hash == crate::cache::content_hash(declaration.as_bytes()))));
    let restored: TypeScriptAuthority =
        serde_json::from_str(&serde_json::to_string(&capture).unwrap()).unwrap();
    assert_eq!(
        restored.compiler_libraries[0].override_specifier,
        request.override_specifier
    );
}

#[test]
fn issue_162_library_override_capture_invalid_manifest_cannot_borrow_installed_fact() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "dom");
    write(
        root.path(),
        "node_modules/typescript/lib/lib.dom.d.ts",
        "declare namespace API { class Builtin {} }",
    );
    write(
        root.path(),
        "node_modules/@typescript/lib-dom/package.json",
        "{invalid",
    );
    let file = root.path().join("app.ts");
    let source = "export class Control {} API.Builtin;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert!(
        capture
            .compiler_libraries
            .first()
            .expect("failed override evidence")
            .resolved
            .is_err()
    );
    let offset = source.find("API.Builtin").unwrap();
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|(start, _, result)| *start == offset
                && matches!(result, Resolution::Unresolved(_)))
    );
}
