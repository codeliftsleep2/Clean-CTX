use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};

#[test]
fn issue_162_declared_scope_capture_retains_parent_and_member_identity_before_projection() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    let source =
        "export namespace API { export namespace Core { export class Leaf {} class Hidden {} } }";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[]);
    let api = capture
        .namespace_scopes
        .iter()
        .find(|scope| scope.declaration.name == "API")
        .unwrap();
    let core = capture
        .namespace_scopes
        .iter()
        .find(|scope| scope.declaration.name == "Core")
        .unwrap();
    assert_eq!(core.parent.as_ref(), Some(&api.declaration));
    assert!(api.parent.is_none());
    let (_, _, Resolution::Resolved(leaf)) = core
        .exports
        .iter()
        .find(|(name, ns, _)| name == "Leaf" && *ns == Namespace::Value)
        .unwrap()
    else {
        panic!("retained member");
    };
    assert_eq!(&source[leaf.start..leaf.end], "class Leaf {}");
    assert!(capture.declarations.contains(leaf));
    assert!(!core.exports.iter().any(|(name, _, _)| name == "Hidden"));
    let restored: TypeScriptAuthority =
        serde_json::from_str(&serde_json::to_string(&capture).unwrap()).unwrap();
    assert_eq!(
        restored.namespace_scopes[1].parent,
        capture.namespace_scopes[1].parent
    );
}

#[test]
fn issue_162_namespace_augmentation_before_its_class_does_not_grant_a_merge() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    std::fs::write(&file, "namespace API { export class Leaf {} } class API {}").unwrap();
    let mut resolver = Resolver::new(&[file.clone()]).unwrap();
    assert!(matches!(
        resolver.local(&file, "API", Namespace::Value),
        Resolution::Ambiguous(_) | Resolution::Unresolved(_)
    ));
    assert!(matches!(
        resolver.namespace_member(&file, "API", "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_distinct_nested_namespace_scopes_do_not_collapse_same_spelling_members() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    let source = "namespace A { export class Leaf {} } namespace B { export class Leaf {} }";
    std::fs::write(&file, source).unwrap();
    let mut resolver = Resolver::new(&[file.clone()]).unwrap();
    let Resolution::Resolved(a) = resolver.namespace_member(&file, "A", "Leaf", Namespace::Value)
    else {
        panic!("A.Leaf");
    };
    let Resolution::Resolved(b) = resolver.namespace_member(&file, "B", "Leaf", Namespace::Value)
    else {
        panic!("B.Leaf");
    };
    assert_eq!(a.name, b.name);
    assert_eq!(a.source, b.source);
    assert_ne!(a.start, b.start);
    assert_ne!(a, b);
}
