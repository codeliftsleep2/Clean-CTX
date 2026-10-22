use super::TypeScriptAuthority;

#[test]
fn issue_162_library_capture_retains_compiler_source_request_and_currentness() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","types":[],"lib":["dom"]},"files":["app.ts"]}"#).unwrap();
    let compiler = root.path().join("node_modules/typescript");
    std::fs::create_dir_all(compiler.join("lib")).unwrap();
    std::fs::write(
        compiler.join("package.json"),
        r#"{"name":"typescript","version":"5.9.3"}"#,
    )
    .unwrap();
    let library = compiler.join("lib/lib.dom.d.ts");
    std::fs::write(&library, "declare namespace API { class Dom {} }").unwrap();
    let file = root.path().join("app.ts");
    let source = "export class Control {} API.Dom;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let request = capture
        .compiler_libraries
        .first()
        .expect("retained library source request");
    assert_eq!(request.version, "5.9.3");
    assert_eq!(request.name, "dom");
    assert!(request.resolved.as_ref().unwrap().ends_with("lib.dom.d.ts"));
    let restored: TypeScriptAuthority =
        serde_json::from_str(&serde_json::to_string(&capture).unwrap()).unwrap();
    assert_eq!(restored.compiler_libraries[0].manifest, request.manifest);
    assert!(capture.witnesses.iter().all(|witness| witness.current()));
    std::fs::write(library, "declare namespace API {}").unwrap();
    assert!(!capture.witnesses.iter().all(|witness| witness.current()));
}
