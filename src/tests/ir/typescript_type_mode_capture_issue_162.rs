use super::{Resolution, TypeScriptAuthority, module_resolution::ModuleRequestKind};
use std::path::Path;

fn write(root: &Path, path: &str, source: &str) {
    let file = root.join(path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"NodeNext","moduleResolution":"nodenext","moduleDetection":"legacy","noLib":true,"types":[]},"files":["app.ts"]}"#,
    );
    write(
        root,
        "node_modules/@types/support/import.d.ts",
        "declare namespace API { class Selected {} }",
    );
    write(
        root,
        "node_modules/@types/support/legacy.d.ts",
        "declare namespace API { class Selected {} }",
    );
}

#[test]
fn issue_162_type_mode_capture_preserves_typed_mode_and_consumed_target() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/@types/support/package.json",
        r#"{"name":"@types/support","types":"legacy.d.ts","exports":{".":{"import":"./import.d.ts"}}}"#,
    );
    let file = root.path().join("app.ts");
    let source = "/// <reference types=\"support\" resolution-mode=\"import\" />\nexport class Control {} API.Selected;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let request = capture.program_directives.first().unwrap();
    assert!(matches!(
        request.resolution_mode,
        Some(Ok(ModuleRequestKind::Import))
    ));
    assert!(request.resolved.as_ref().unwrap().ends_with("import.d.ts"));
    let restored: TypeScriptAuthority =
        serde_json::from_str(&serde_json::to_string(&capture).unwrap()).unwrap();
    assert!(matches!(
        restored.program_directives[0].resolution_mode,
        Some(Ok(ModuleRequestKind::Import))
    ));
    assert!(capture.witnesses.iter().all(|witness| witness.current()));
    write(
        root.path(),
        "node_modules/@types/support/import.d.ts",
        "declare namespace API { enum Selected {} }",
    );
    assert!(!capture.witnesses.iter().all(|witness| witness.current()));
}

#[test]
fn issue_162_type_mode_capture_blocked_export_does_not_borrow_legacy_entry() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let file = root.path().join("app.ts");
    let source = "/// <reference types=\"support\" />\nexport class Control {} API.Selected;";
    std::fs::write(&file, source).unwrap();
    write(
        root.path(),
        "node_modules/@types/support/package.json",
        r#"{"name":"@types/support","types":"legacy.d.ts"}"#,
    );
    let control = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert!(control.program_directives[0].resolved.is_ok());
    write(
        root.path(),
        "node_modules/@types/support/package.json",
        r#"{"name":"@types/support","types":"legacy.d.ts","exports":{".":null}}"#,
    );
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert!(capture.program_directives[0].resolved.is_err());
    let offset = source.find("API.Selected").unwrap();
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|(start, _, result)| *start == offset
                && matches!(result, Resolution::Unresolved(_)))
    );
}
