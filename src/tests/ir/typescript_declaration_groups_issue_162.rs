use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn fixture(root: &Path, declaration: &str) {
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    std::fs::write(root.join("leaf.ts"), declaration).unwrap();
    std::fs::write(root.join("barrel.ts"), "export * from './leaf';").unwrap();
    std::fs::write(
        root.join("app.ts"),
        "import { Item as Selected } from './barrel';",
    )
    .unwrap();
}

#[test]
fn issue_162_overload_identity_retains_every_signature_and_body() {
    let root = tempfile::tempdir().unwrap();
    fixture(
        root.path(),
        "export function Item(x: string): string; export function Item(x: number): number; export function Item(x: any): any { return x; }",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let Resolution::Merged(ids) =
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value)
    else {
        panic!("all overload contributors must survive alias/export resolution");
    };
    assert_eq!(ids.len(), 3);
    assert_eq!(
        ids.iter()
            .filter(|id| id.kind == "function_signature")
            .count(),
        2
    );
    assert!(ids.windows(2).all(|pair| pair[0] != pair[1]));
    assert!(matches!(
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Type),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_interface_group_is_type_only_and_changes_invalidate_its_snapshot() {
    let root = tempfile::tempdir().unwrap();
    fixture(
        root.path(),
        "export interface Item { a: string; } export interface Item { b: number; }",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        matches!(resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Type), Resolution::Merged(ids) if ids.len() == 2)
    );
    assert!(matches!(
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    std::fs::write(
        root.path().join("leaf.ts"),
        "export interface Item { a: string; }",
    )
    .unwrap();
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        fresh.local(&root.path().join("app.ts"), "Selected", Namespace::Type),
        Resolution::Resolved(_)
    ));
}

#[test]
fn issue_162_competing_function_bodies_are_not_an_overload_group() {
    let root = tempfile::tempdir().unwrap();
    fixture(
        root.path(),
        "export function Item() {} export function Item() {}",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value),
        Resolution::Ambiguous(_)
    ));
}

#[test]
fn issue_162_star_exports_from_distinct_interface_owners_do_not_merge() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), "export interface Item { a: string; }");
    std::fs::write(
        root.path().join("other.ts"),
        "export interface Item { b: number; }",
    )
    .unwrap();
    std::fs::write(
        root.path().join("barrel.ts"),
        "export * from './leaf'; export * from './other';",
    )
    .unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        matches!(resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Type), Resolution::Ambiguous(ids) if ids.len() == 2)
    );
}

#[test]
fn issue_162_class_interface_merge_preserves_distinct_type_and_value_groups() {
    for class in ["class", "abstract class"] {
        let root = tempfile::tempdir().unwrap();
        fixture(
            root.path(),
            &format!("export {class} Item {{}} export interface Item {{ extra: string; }}"),
        );
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        let file = root.path().join("app.ts");
        let Resolution::Resolved(value) = resolver.local(&file, "Selected", Namespace::Value)
        else {
            panic!("interface augmentation must not create a second runtime value");
        };
        let Resolution::Merged(types) = resolver.local(&file, "Selected", Namespace::Type) else {
            panic!("both type contributors must survive");
        };
        assert_eq!(types.len(), 2);
        assert!(types.contains(&value));
        assert_eq!(
            types
                .iter()
                .filter(|id| id.kind == "interface_declaration")
                .count(),
            1
        );
    }
}

#[test]
fn issue_162_export_diamond_deduplicates_routes_without_losing_group_members() {
    let root = tempfile::tempdir().unwrap();
    fixture(
        root.path(),
        "export interface Item { a: string; } export interface Item { b: number; }",
    );
    std::fs::write(
        root.path().join("left.ts"),
        "export { Item } from './leaf';",
    )
    .unwrap();
    std::fs::write(root.path().join("right.ts"), "export * from './leaf';").unwrap();
    std::fs::write(
        root.path().join("barrel.ts"),
        "export * from './left'; export * from './right';",
    )
    .unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let direct = resolver.local(&root.path().join("leaf.ts"), "Item", Namespace::Type);
    assert!(matches!(&direct, Resolution::Merged(ids) if ids.len() == 2));
    assert_eq!(
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Type),
        direct
    );
    std::fs::remove_file(root.path().join("right.ts")).unwrap();
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        fresh.local(&root.path().join("app.ts"), "Selected", Namespace::Type),
        Resolution::Unresolved(_)
    ));
    std::fs::write(root.path().join("right.ts"), "export * from './leaf';").unwrap();
    assert!(!fresh.host.current());
    let mut restored = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        restored.local(&root.path().join("app.ts"), "Selected", Namespace::Type),
        direct
    );
}

#[test]
fn issue_162_two_class_bodies_cannot_gain_authority_from_an_interface() {
    let root = tempfile::tempdir().unwrap();
    fixture(
        root.path(),
        "export class Item {} export class Item {} export interface Item { a: string; }",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    for namespace in [Namespace::Type, Namespace::Value] {
        assert!(matches!(
            resolver.local(&root.path().join("app.ts"), "Selected", namespace),
            Resolution::Ambiguous(_)
        ));
    }
}
