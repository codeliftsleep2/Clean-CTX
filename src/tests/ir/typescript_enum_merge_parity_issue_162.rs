use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn setup(root: &Path, first: &str, second: &str) {
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["app.ts","a.ts","b.ts"]}"#).unwrap();
    std::fs::write(root.join("app.ts"), "export class Control {}").unwrap();
    std::fs::write(root.join("a.ts"), first).unwrap();
    std::fs::write(root.join("b.ts"), second).unwrap();
}
fn lookup(root: &Path, file: &str, namespace: Namespace) -> Resolution {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&root.join("app.ts"), "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    resolver.local(&root.join(file), "Mode", namespace)
}
fn merged(result: Resolution) -> Vec<super::DeclarationId> {
    let Resolution::Merged(ids) = result else {
        panic!("enum contributors: {result:?}");
    };
    assert_eq!(ids.len(), 2);
    assert!(
        ids.iter()
            .all(|id| id.kind == "enum_declaration" && id.end > id.start)
    );
    ids
}

#[test]
fn issue_162_enum_merge_parity_same_source_export_and_global() {
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        "enum Mode { One = 1 } enum Mode { Two = 2 }",
        "",
    );
    let values = merged(lookup(root.path(), "app.ts", Namespace::Value));
    assert_eq!(
        values,
        merged(lookup(root.path(), "app.ts", Namespace::Type))
    );
    std::fs::write(
        root.path().join("a.ts"),
        "export enum Mode { One = 1 } export enum Mode { Two = 2 }",
    )
    .unwrap();
    let values = merged(lookup(root.path(), "a.ts", Namespace::Value));
    assert_eq!(values, merged(lookup(root.path(), "a.ts", Namespace::Type)));
    assert!(matches!(
        lookup(root.path(), "app.ts", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_enum_merge_parity_const_parts_preserve_both_sources() {
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        "const enum Mode { One = 1 }",
        "const enum Mode { Two = 2 }",
    );
    let ids = merged(lookup(root.path(), "app.ts", Namespace::Type));
    assert!(ids.iter().any(|id| id.source.ends_with("a.ts")));
    assert!(ids.iter().any(|id| id.source.ends_with("b.ts")));
}

#[test]
fn issue_162_enum_merge_parity_deletion_restoration_replaces_snapshot() {
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        "enum Mode { One = 1 }",
        "enum Mode { Two = 2 }",
    );
    let original = merged(lookup(root.path(), "app.ts", Namespace::Type));
    std::fs::write(root.path().join("b.ts"), "").unwrap();
    assert!(matches!(
        lookup(root.path(), "app.ts", Namespace::Type),
        Resolution::Resolved(_)
    ));
    std::fs::write(root.path().join("b.ts"), "enum Mode { Three = 3 }").unwrap();
    let restored = merged(lookup(root.path(), "app.ts", Namespace::Type));
    assert_ne!(original, restored);
    assert_eq!(
        original.iter().find(|id| id.source.ends_with("a.ts")),
        restored.iter().find(|id| id.source.ends_with("a.ts"))
    );
}
