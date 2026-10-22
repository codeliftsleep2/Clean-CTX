use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};

#[test]
fn issue_162_jsx_sources_keep_canonical_export_identity_and_snapshot_invalidation() {
    for extension in ["tsx", "jsx"] {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","allowJs":true,"jsx":"preserve"}}"#).unwrap();
        let leaf = root.path().join(format!("leaf.{extension}"));
        let source = "export class Leaf { render() { return <section data-id='leaf' />; } }";
        std::fs::write(&leaf, source).unwrap();
        let app = root.path().join("app.ts");
        std::fs::write(&app, "import { Leaf as Selected } from './leaf';").unwrap();
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        let Resolution::Resolved(id) = resolver.local(&app, "Selected", Namespace::Value) else {
            panic!("JSX syntax must not erase a proved exported declaration");
        };
        // Compare physical identities in the same OS representation: Windows
        // canonicalize returns a verbatim path, while Host stores shaped paths.
        assert_eq!(
            std::fs::canonicalize(&id.source).unwrap(),
            std::fs::canonicalize(&leaf).unwrap()
        );
        assert_eq!(
            id.source_hash,
            crate::cache::content_hash(source.as_bytes())
        );
        assert_eq!(
            &source[id.start..id.end],
            source.strip_prefix("export ").unwrap()
        );
        assert_eq!(
            resolver.local(&app, "Selected", Namespace::Type),
            Resolution::Resolved(id)
        );
        std::fs::write(
            &leaf,
            "export class Replacement { render() { return <aside />; } }",
        )
        .unwrap();
        assert!(!resolver.host.current());
        let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        assert!(matches!(
            fresh.local(&app, "Selected", Namespace::Value),
            Resolution::Unresolved(_)
        ));
    }
}

#[test]
fn issue_162_jsx_capture_uses_supplied_bytes_and_keeps_local_declarations() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("view.tsx");
    std::fs::write(&file, "export class DiskOnly {}").unwrap();
    let source = "export const View = () => <main />;";
    let capture = TypeScriptAuthority::capture(&file, source, &[]);
    assert!(
        capture.declarations.iter().any(|id| id.name == "View"
            && id.source_hash == crate::cache::content_hash(source.as_bytes()))
    );
    assert!(!capture.declarations.iter().any(|id| id.name == "DiskOnly"));
}

#[test]
fn issue_162_typescript_extension_does_not_silently_enable_jsx_grammar() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("view.ts");
    let source = "export const View = () => <main />;";
    std::fs::write(&file, source).unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&file, "View", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}
