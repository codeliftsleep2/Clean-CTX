use super::TypeScriptAuthority;

#[test]
fn issue_162_directive_capture_retains_exact_source_span_hash_and_resolution() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","types":[]},"files":["app.ts"]}"#).unwrap();
    std::fs::write(
        root.path().join("leaf.d.ts"),
        "declare namespace API { class Leaf {} }",
    )
    .unwrap();
    let file = root.path().join("app.ts");
    let source = "/// <reference path=\"leaf.d.ts\" />\nexport class Control {} API.Leaf;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let directive = capture
        .program_directives
        .first()
        .expect("retained producer directive");
    assert_eq!(
        &source[directive.start..directive.end],
        "/// <reference path=\"leaf.d.ts\" />"
    );
    assert_eq!(
        directive.source_hash,
        crate::cache::content_hash(source.as_bytes())
    );
    assert!(directive.resolved.as_ref().unwrap().ends_with("leaf.d.ts"));
    assert!(!directive.ignored);
    let restored: TypeScriptAuthority =
        serde_json::from_str(&serde_json::to_string(&capture).unwrap()).unwrap();
    assert_eq!(
        restored.program_directives[0].source_hash,
        directive.source_hash
    );
    assert!(capture.witnesses.iter().all(|witness| witness.current()));
    std::fs::write(root.path().join("leaf.d.ts"), "declare namespace API {}").unwrap();
    assert!(!capture.witnesses.iter().all(|witness| witness.current()));
}

#[test]
fn issue_162_directive_capture_no_resolve_retains_ignored_without_missing_probe() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","types":[],"noResolve":true},"files":["app.ts"]}"#).unwrap();
    let file = root.path().join("app.ts");
    let source = "/// <reference path=\"absent.d.ts\" />\nexport class Control {} API.Leaf;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let directive = capture
        .program_directives
        .first()
        .expect("retained ignored directive");
    assert!(directive.ignored);
    assert!(directive.resolved.is_err());
    assert!(!capture.witnesses.iter().any(|witness| matches!(witness,
        super::Witness::Missing { path } if path.ends_with("absent.d.ts"))));
}
