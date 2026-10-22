use super::TypeScriptAuthority;

#[test]
fn issue_162_automatic_type_capture_retains_resolved_request_and_source_witness() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","types":["one"]},"files":["app.ts"]}"#).unwrap();
    let package = root.path().join("node_modules/@types/one");
    std::fs::create_dir_all(&package).unwrap();
    let declaration = package.join("index.d.ts");
    std::fs::write(&declaration, "declare namespace API { class One {} }").unwrap();
    let file = root.path().join("app.ts");
    let source = "export class Control {} API.One;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert_eq!(capture.automatic_types.len(), 1);
    let request = &capture.automatic_types[0];
    assert_eq!(request.name, "one");
    assert!(
        request
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("one/index.d.ts")
    );
    let restored: TypeScriptAuthority =
        serde_json::from_str(&serde_json::to_string(&capture).unwrap()).unwrap();
    assert_eq!(restored.automatic_types[0].name, request.name);
    assert!(capture.witnesses.iter().all(|witness| witness.current()));
    std::fs::write(declaration, "declare namespace API {}").unwrap();
    assert!(!capture.witnesses.iter().all(|witness| witness.current()));
}

#[test]
fn issue_162_automatic_type_capture_retains_missing_required_package_request() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","types":["absent"]},"files":["app.ts"]}"#).unwrap();
    let file = root.path().join("app.ts");
    let source = "export class Control {} API.One;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert_eq!(capture.automatic_types.len(), 1);
    assert_eq!(capture.automatic_types[0].name, "absent");
    assert!(capture.automatic_types[0].resolved.is_err());
    assert!(capture.programs.is_empty());
    assert!(capture.witnesses.iter().any(|witness| matches!(witness,
        super::Witness::Missing { path } if path.to_string_lossy().contains("absent"))));
}
