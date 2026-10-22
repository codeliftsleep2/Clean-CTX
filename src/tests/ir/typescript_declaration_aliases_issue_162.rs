//! Declaration aliases and outer lexical namespaces share canonical authority.
use super::{Namespace, Resolution, Resolver};

#[test]
fn issue_162_declaration_alias_class_and_interface_preserve_namespace_eligibility() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    std::fs::write(&file, "namespace API { export class Leaf {} export interface Shape {} } import Selected = API.Leaf; import Type = API.Shape;").unwrap();
    let mut resolver = Resolver::new(&[file.clone()]).unwrap();
    for ns in [Namespace::Value, Namespace::Type] {
        let direct = resolver.namespace_member(&file, "API", "Leaf", ns);
        assert!(
            matches!(direct, Resolution::Resolved(_)),
            "direct: {direct:?}"
        );
        assert_eq!(resolver.local(&file, "Selected", ns), direct);
    }
    let shape = resolver.namespace_member(&file, "API", "Shape", Namespace::Type);
    assert!(
        matches!(shape, Resolution::Resolved(_)),
        "type control: {shape:?}"
    );
    assert_eq!(resolver.local(&file, "Type", Namespace::Type), shape);
    assert!(matches!(
        resolver.local(&file, "Type", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_declaration_alias_exported_member_uses_declaring_namespace() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    let source = "namespace Core { export class Leaf {} } namespace API { export namespace Core { export class Leaf {} } export import Selected = Core.Leaf; }";
    std::fs::write(&file, source).unwrap();
    let mut resolver = Resolver::new(&[file.clone()]).unwrap();
    let direct = resolver.namespace_chain(&file, "API", &["Core", "Leaf"], Namespace::Value);
    let Resolution::Resolved(id) = &direct else {
        panic!("control: {direct:?}");
    };
    assert_eq!(id.start, source.rfind("class Leaf").unwrap());
    assert_eq!(
        resolver.namespace_member(&file, "API", "Selected", Namespace::Value),
        direct
    );
}

#[test]
fn issue_162_declaration_alias_outward_namespace_lookup_stops_at_runtime_shadow() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    for shadow in [false, true] {
        let local = if shadow { "const Core = {};" } else { "" };
        std::fs::write(&file, format!("namespace Core {{ export class Leaf {{}} }} namespace API {{ {local} export import Selected = Core; }}")).unwrap();
        let mut resolver = Resolver::new(&[file.clone()]).unwrap();
        let direct = resolver.namespace_member(&file, "Core", "Leaf", Namespace::Value);
        assert!(
            matches!(direct, Resolution::Resolved(_)),
            "control: {direct:?}"
        );
        let selected =
            resolver.namespace_chain(&file, "API", &["Selected", "Leaf"], Namespace::Value);
        if shadow {
            assert!(matches!(selected, Resolution::Unresolved(_)));
        } else {
            assert_eq!(selected, direct);
        }
    }
}

#[test]
fn issue_162_declaration_alias_export_retarget_retracts_value_identity() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    for kind in ["class", "interface", "enum"] {
        let source = format!(
            "namespace API {{ export {kind} Leaf {{}} }} export import Selected = API.Leaf;"
        );
        std::fs::write(&file, &source).unwrap();
        let mut resolver = Resolver::new(&[file.clone()]).unwrap();
        let direct = resolver.namespace_member(&file, "API", "Leaf", Namespace::Type);
        let Resolution::Resolved(id) = &direct else {
            panic!("control: {direct:?}");
        };
        assert_eq!(
            id.source_hash,
            crate::cache::content_hash(source.as_bytes())
        );
        assert_eq!(resolver.local(&file, "Selected", Namespace::Type), direct);
        let value = resolver.local(&file, "Selected", Namespace::Value);
        if kind == "interface" {
            assert!(matches!(value, Resolution::Unresolved(_)));
        } else {
            assert_eq!(value, direct);
        }
    }
}
