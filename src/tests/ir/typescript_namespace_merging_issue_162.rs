//! Required declaration-owned namespace scope contracts; not module-path guesses.
use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    std::fs::write(root.join(name), source).unwrap();
}
fn setup(root: &Path, source: &str) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    );
    write(root, "leaf.ts", source);
    write(root, "app.ts", "import { API as Selected } from './leaf';");
}
fn member(root: &Path, resolver: &mut Resolver, name: &str, namespace: Namespace) -> Resolution {
    resolver.namespace_member(&root.join("app.ts"), "Selected", name, namespace)
}
fn exact(result: Resolution) -> super::DeclarationId {
    let Resolution::Resolved(id) = result else {
        panic!("expected nonempty exact declaration, got {result:?}");
    };
    id
}

#[test]
fn issue_162_namespace_merge_declared_scope_retains_member_source_and_span() {
    let root = tempfile::tempdir().unwrap();
    let source = "export namespace API { export class Leaf {} }";
    setup(root.path(), source);
    write(root.path(), "control.ts", "export class Control {}");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    exact(resolver.local(&root.path().join("control.ts"), "Control", Namespace::Value));
    let id = exact(member(root.path(), &mut resolver, "Leaf", Namespace::Value));
    assert!(id.source.ends_with("leaf.ts"));
    assert_eq!(id.name, "Leaf");
    assert_eq!(id.kind, "class_declaration");
    assert_eq!(&source[id.start..id.end], "class Leaf {}");
    assert_eq!(
        id.source_hash,
        crate::cache::content_hash(source.as_bytes())
    );
    assert_eq!(
        member(root.path(), &mut resolver, "Leaf", Namespace::Type),
        Resolution::Resolved(id)
    );
}

#[test]
fn issue_162_namespace_merge_same_owner_parts_expose_only_exported_members() {
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        "export namespace API { export class Leaf {} class Private {} } export namespace API { export interface Shape {} }",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        exact(member(root.path(), &mut resolver, "Leaf", Namespace::Value)).kind,
        "class_declaration"
    );
    assert_eq!(
        exact(member(root.path(), &mut resolver, "Shape", Namespace::Type)).kind,
        "interface_declaration"
    );
    assert!(matches!(
        member(root.path(), &mut resolver, "Private", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    assert!(matches!(
        member(root.path(), &mut resolver, "Shape", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_namespace_merge_nested_scopes_preserve_lexical_owner() {
    let root = tempfile::tempdir().unwrap();
    let source = "export namespace API { export namespace Core { export class Leaf {} } } export namespace Other { export class Leaf {} }";
    setup(root.path(), source);
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let id = exact(resolver.namespace_chain(
        &root.path().join("app.ts"),
        "Selected",
        &["Core", "Leaf"],
        Namespace::Value,
    ));
    assert_eq!(id.start, source.find("class Leaf").unwrap());
    assert!(matches!(
        member(root.path(), &mut resolver, "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_namespace_merge_class_function_and_enum_keep_all_symbol_contributors() {
    for declaration in [
        "export class API {}",
        "export function API() {}",
        "export enum API { One }",
    ] {
        let root = tempfile::tempdir().unwrap();
        setup(
            root.path(),
            &format!("{declaration} export namespace API {{ export class Leaf {{}} }}"),
        );
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        let value = resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value);
        assert!(
            matches!(value, Resolution::Merged(ids) if ids.len() == 2),
            "{declaration}"
        );
        assert_eq!(
            exact(member(root.path(), &mut resolver, "Leaf", Namespace::Value)).kind,
            "class_declaration"
        );
    }
}

#[test]
fn issue_162_namespace_merge_distinct_module_owners_remain_conflicting() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "export namespace API { export class Leaf {} }");
    let mut control = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    exact(member(root.path(), &mut control, "Leaf", Namespace::Value));
    write(
        root.path(),
        "other.ts",
        "export namespace API { export class Leaf {} }",
    );
    write(
        root.path(),
        "barrel.ts",
        "export * from './leaf'; export * from './other';",
    );
    write(
        root.path(),
        "app.ts",
        "import { API as Selected } from './barrel';",
    );
    let mut competing = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        member(root.path(), &mut competing, "Leaf", Namespace::Value),
        Resolution::Unresolved(_) | Resolution::Ambiguous(_)
    ));
}

#[test]
fn issue_162_namespace_merge_member_kind_deletion_and_restoration_revalidate() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "export namespace API { export class Leaf {} }");
    let mut before = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    exact(member(root.path(), &mut before, "Leaf", Namespace::Value));
    write(
        root.path(),
        "leaf.ts",
        "export namespace API { export interface Leaf {} }",
    );
    assert!(!before.host.current());
    let mut changed = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        member(root.path(), &mut changed, "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    assert_eq!(
        exact(member(root.path(), &mut changed, "Leaf", Namespace::Type)).kind,
        "interface_declaration"
    );
    std::fs::remove_file(root.path().join("leaf.ts")).unwrap();
    assert!(!changed.host.current());
    let mut missing = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        member(root.path(), &mut missing, "Leaf", Namespace::Type),
        Resolution::Unresolved(_)
    ));
    write(
        root.path(),
        "leaf.ts",
        "export namespace API { export class Leaf { restored = true; } }",
    );
    assert!(!missing.host.current());
    let mut restored = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    exact(member(root.path(), &mut restored, "Leaf", Namespace::Value));
}

#[test]
fn issue_162_namespace_merge_imported_alias_and_barrel_preserve_member_identity() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "export namespace API { export class Leaf {} }");
    let mut before = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = member(root.path(), &mut before, "Leaf", Namespace::Value);
    exact(expected.clone());
    write(
        root.path(),
        "barrel.ts",
        "export { API as Facade } from './leaf';",
    );
    write(
        root.path(),
        "app.ts",
        "import { Facade as Selected } from './barrel';",
    );
    let mut aliased = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        member(root.path(), &mut aliased, "Leaf", Namespace::Value),
        expected
    );
}
