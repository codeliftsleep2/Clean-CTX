//! Dotted namespaces must retain every declaration-owned qualifier scope.
use super::{Namespace, Resolution, Resolver};

fn exact(result: Resolution) -> super::DeclarationId {
    let Resolution::Resolved(id) = result else {
        panic!("expected exact declaration, got {result:?}");
    };
    id
}

#[test]
fn issue_162_dotted_namespace_retains_member_identity_and_private_boundary() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    let source = "class Control {} namespace API.Core { export class Leaf {} class Hidden {} }";
    std::fs::write(&file, source).unwrap();
    let mut resolver = Resolver::new(&[file.clone()]).unwrap();
    exact(resolver.local(&file, "Control", Namespace::Value));
    let leaf = exact(resolver.namespace_chain(&file, "API", &["Core", "Leaf"], Namespace::Value));
    assert_eq!(&source[leaf.start..leaf.end], "class Leaf {}");
    assert_eq!(
        leaf.source_hash,
        crate::cache::content_hash(source.as_bytes())
    );
    assert_eq!(
        resolver.namespace_chain(&file, "API", &["Core", "Leaf"], Namespace::Type),
        Resolution::Resolved(leaf)
    );
    assert!(matches!(
        resolver.namespace_chain(&file, "API", &["Core", "Hidden"], Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_dotted_namespace_merges_with_explicit_scope_without_name_conflation() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    let source = "namespace API.Core { export class Leaf {} } namespace API { export namespace Core { export interface Shape {} } } namespace Other.Core { export class Leaf {} }";
    std::fs::write(&file, source).unwrap();
    let mut resolver = Resolver::new(&[file.clone()]).unwrap();
    let a = exact(resolver.namespace_chain(&file, "API", &["Core", "Leaf"], Namespace::Value));
    let b = exact(resolver.namespace_chain(&file, "Other", &["Core", "Leaf"], Namespace::Value));
    assert_ne!(a, b);
    let shape = exact(resolver.namespace_chain(&file, "API", &["Core", "Shape"], Namespace::Type));
    assert_eq!(shape.kind, "interface_declaration");
    assert!(matches!(
        resolver.namespace_chain(&file, "API", &["Core", "Shape"], Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_dotted_namespace_imported_member_mutation_and_restoration() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    let leaf = root.path().join("leaf.ts");
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    std::fs::write(&file, "import { API as Selected } from './leaf';").unwrap();
    for (source, expected) in [
        (
            "export namespace API.Core { export class Leaf {} }",
            Some("class_declaration"),
        ),
        (
            "export namespace API.Core { export interface Leaf {} }",
            None,
        ),
        (
            "export namespace API.Core { export enum Leaf {} }",
            Some("enum_declaration"),
        ),
    ] {
        std::fs::write(&leaf, source).unwrap();
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        let result =
            resolver.namespace_chain(&file, "Selected", &["Core", "Leaf"], Namespace::Value);
        if let Some(kind) = expected {
            let id = exact(result);
            assert_eq!(id.kind, kind);
            assert_eq!(
                id.source_hash,
                crate::cache::content_hash(source.as_bytes())
            );
            assert!(id.source.ends_with("leaf.ts"));
        } else {
            assert!(matches!(result, Resolution::Unresolved(_)));
        }
    }
}
