//! A default classification is compiler-version evidence, not a wall-clock rule.
use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn setup(root: &Path) {
    // Library files are data, not installed-compiler identity. The absent-manifest
    // test still has no compiler version authority until it writes a manifest.
    let library = root.join("node_modules/typescript/lib");
    std::fs::create_dir_all(&library).unwrap();
    std::fs::write(
        library.join("lib.esnext.full.d.ts"),
        "/// <reference no-default-lib=\"true\" />\n// No library declarations participate in this detection fixture.\n",
    )
    .unwrap();
    std::fs::write(root.join("app.ts"), "export class Control {}").unwrap();
    std::fs::write(
        root.join("leaf.ts"),
        "namespace API { export class Leaf {} }",
    )
    .unwrap();
    std::fs::write(root.join("package.json"), r#"{"type":"commonjs"}"#).unwrap();
}
fn config(root: &Path, detection: Option<&str>) {
    let mut options = serde_json::json!({"module":"NodeNext","moduleResolution":"nodenext"});
    if let Some(detection) = detection {
        options["moduleDetection"] = detection.into();
    }
    std::fs::write(
        root.join("tsconfig.json"),
        serde_json::json!({"compilerOptions":options,"files":["app.ts","leaf.ts"]}).to_string(),
    )
    .unwrap();
}
fn result(root: &Path) -> (Resolver, Resolution) {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let file = root.join("app.ts");
    assert!(matches!(
        resolver.local(&file, "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    let result = resolver.namespace_member(&file, "API", "Leaf", Namespace::Value);
    (resolver, result)
}

#[test]
fn issue_162_detection_default_node59_is_force_with_explicit_auto_control() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let compiler = root.path().join("node_modules/typescript");
    std::fs::create_dir_all(&compiler).unwrap();
    std::fs::write(
        compiler.join("package.json"),
        r#"{"name":"typescript","version":"5.9.3"}"#,
    )
    .unwrap();
    for detection in [Some("auto"), None, Some("auto")] {
        config(root.path(), detection);
        let (_, result) = result(root.path());
        if detection.is_some() {
            assert!(
                matches!(result, Resolution::Resolved(_)),
                "explicit auto: {result:?}"
            );
        } else {
            assert!(
                matches!(result, Resolution::Unresolved(_)),
                "version-backed default: {result:?}"
            );
        }
    }
}

#[test]
fn issue_162_detection_default_without_compiler_evidence_cannot_grant_globals() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    config(root.path(), Some("auto"));
    assert!(matches!(result(root.path()).1, Resolution::Resolved(_)));
    config(root.path(), None);
    assert!(matches!(result(root.path()).1, Resolution::Unresolved(_)));
}

#[test]
fn issue_162_detection_default_compiler_snapshot_mutation_invalidates_evidence() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let compiler = root.path().join("node_modules/typescript");
    std::fs::create_dir_all(&compiler).unwrap();
    let manifest = compiler.join("package.json");
    std::fs::write(&manifest, r#"{"name":"typescript","version":"5.9.3"}"#).unwrap();
    config(root.path(), Some("auto"));
    let (resolver, positive) = result(root.path());
    assert!(matches!(positive, Resolution::Resolved(_)));
    assert!(resolver.host.current());
    std::fs::write(&manifest, r#"{"name":"typescript","version":"5.9.2"}"#).unwrap();
    assert!(!resolver.host.current());
}
