use super::{Resolution, TypeScriptAuthority};

#[test]
fn issue_162_program_capture_retains_contributors_and_config_currentness() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("tsconfig.json");
    std::fs::write(&config, r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy"},"files":["app.ts","a.ts","b.ts"]}"#).unwrap();
    std::fs::write(
        root.path().join("a.ts"),
        "namespace API { export class Leaf {} }",
    )
    .unwrap();
    std::fs::write(
        root.path().join("b.ts"),
        "namespace API { export interface Shape {} }",
    )
    .unwrap();
    let file = root.path().join("app.ts");
    let source = "API.Leaf;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|(_, _, result)| matches!(result, Resolution::Resolved(id) if id.name == "Leaf"))
    );
    assert_eq!(capture.programs.len(), 1);
    assert_eq!(capture.programs[0].sources.len(), 3);
    assert!(
        capture.programs[0]
            .groups
            .iter()
            .any(|group| group.len() == 2)
    );
    let restored: TypeScriptAuthority =
        serde_json::from_str(&serde_json::to_string(&capture).unwrap()).unwrap();
    assert_eq!(restored.programs[0].groups, capture.programs[0].groups);
    assert!(capture.witnesses.iter().all(|witness| witness.current()));
    std::fs::write(root.path().join("b.ts"), "namespace API {}").unwrap();
    assert!(!capture.witnesses.iter().all(|witness| witness.current()));
}

#[test]
fn issue_162_program_unavailable_contributor_cannot_certify_global_scope() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy"},"files":["app.ts","absent.ts"]}"#).unwrap();
    let file = root.path().join("app.ts");
    let source = "API.Leaf;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|(_, _, result)| matches!(result, Resolution::Unresolved(_)))
    );
    assert!(capture.programs.is_empty());
    assert!(capture.witnesses.iter().any(
        |witness| matches!(witness, super::Witness::Missing { path } if path.ends_with("absent.ts"))
    ));
}
