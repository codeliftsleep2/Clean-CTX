//! Internal import aliases denote canonical namespace/declaration identities.
use super::{Namespace, Resolution, Resolver};

#[test]
fn issue_162_namespace_alias_qualified_target_preserves_both_namespaces() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    let source = "namespace API.Core { export class Leaf {} } import Selected = API.Core;";
    std::fs::write(&file, source).unwrap();
    let mut resolver = Resolver::new(std::slice::from_ref(&file)).unwrap();
    for ns in [Namespace::Value, Namespace::Type] {
        let direct = resolver.namespace_chain(&file, "API", &["Core", "Leaf"], ns);
        assert!(
            matches!(direct, Resolution::Resolved(_)),
            "direct control: {direct:?}"
        );
        assert_eq!(
            resolver.namespace_member(&file, "Selected", "Leaf", ns),
            direct
        );
    }
}

#[test]
fn issue_162_namespace_alias_exported_nested_alias_keeps_lexical_owner() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    let source = "namespace Core { export class Leaf {} } namespace API { export namespace Core { export class Leaf {} } export import Selected = Core; }";
    std::fs::write(&file, source).unwrap();
    let mut resolver = Resolver::new(std::slice::from_ref(&file)).unwrap();
    let direct = resolver.namespace_chain(&file, "API", &["Core", "Leaf"], Namespace::Value);
    let Resolution::Resolved(id) = &direct else {
        panic!("direct control: {direct:?}");
    };
    assert_eq!(id.start, source.rfind("class Leaf").unwrap());
    assert_eq!(
        resolver.namespace_chain(&file, "API", &["Selected", "Leaf"], Namespace::Value),
        direct
    );
}

#[test]
fn issue_162_namespace_alias_cycles_refuse_and_retarget_restores_identity() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    let prefix = "namespace Core { export class Leaf {} } ";
    for (suffix, positive) in [
        ("import Selected = Core;", true),
        ("import Selected = Other; import Other = Selected;", false),
        ("const Other = {}; import Selected = Other;", false),
        ("import Other = Core; import Selected = Other;", true),
    ] {
        std::fs::write(&file, format!("{prefix}{suffix}")).unwrap();
        let mut resolver = Resolver::new(std::slice::from_ref(&file)).unwrap();
        let result = resolver.namespace_member(&file, "Selected", "Leaf", Namespace::Value);
        if positive {
            let direct = resolver.namespace_member(&file, "Core", "Leaf", Namespace::Value);
            assert!(
                matches!(direct, Resolution::Resolved(_)),
                "direct control: {direct:?}"
            );
            assert_eq!(result, direct);
        } else {
            assert!(matches!(result, Resolution::Unresolved(_)), "{result:?}");
        }
    }
}
