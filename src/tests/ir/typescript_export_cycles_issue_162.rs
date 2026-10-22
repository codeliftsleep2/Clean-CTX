use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    std::fs::write(root.join(name), source).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    );
    write(root, "leaf.ts", "export class Leaf {}");
    write(root, "app.ts", "import { Leaf as Selected } from './a';");
}
fn selected(root: &Path, resolver: &mut Resolver) -> Resolution {
    resolver.local(&root.join("app.ts"), "Selected", Namespace::Value)
}

#[test]
fn issue_162_export_star_cycles_retain_grounded_identity_in_either_traversal_order() {
    for a in [
        "export * from './b'; export * from './leaf';",
        "export * from './leaf'; export * from './b';",
    ] {
        let root = tempfile::tempdir().unwrap();
        setup(root.path());
        write(root.path(), "a.ts", a);
        write(root.path(), "b.ts", "export * from './c';");
        write(root.path(), "c.ts", "export * from './a';");
        write(
            root.path(),
            "via_b.ts",
            "import { Leaf as Selected } from './b';",
        );
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        let expected = resolver.local(&root.path().join("leaf.ts"), "Leaf", Namespace::Value);
        assert!(matches!(&expected, Resolution::Resolved(_)));
        assert_eq!(selected(root.path(), &mut resolver), expected);
        assert_eq!(
            resolver.local(&root.path().join("via_b.ts"), "Selected", Namespace::Value),
            expected
        );
        assert!(matches!(
            resolver.local(&root.path().join("b.ts"), "Leaf", Namespace::Value),
            Resolution::Unresolved(_)
        ));
        assert_eq!(selected(root.path(), &mut resolver), expected);
    }
}

#[test]
fn issue_162_cycle_without_grounded_declaration_does_not_invent_one() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(root.path(), "a.ts", "export * from './b';");
    write(root.path(), "b.ts", "export * from './a';");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        selected(root.path(), &mut resolver),
        Resolution::Unresolved(_)
    ));
    write(
        root.path(),
        "b.ts",
        "export * from './a'; export * from './leaf';",
    );
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        selected(root.path(), &mut fresh),
        Resolution::Resolved(_)
    ));
}

#[test]
fn issue_162_cycle_with_competing_grounded_declarations_remains_ambiguous() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(root.path(), "other.ts", "export class Leaf {}");
    write(
        root.path(),
        "a.ts",
        "export * from './b'; export * from './leaf';",
    );
    write(
        root.path(),
        "b.ts",
        "export * from './a'; export * from './other';",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let Resolution::Ambiguous(ids) = selected(root.path(), &mut resolver) else {
        panic!("competing grounded declarations");
    };
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0].source, ids[1].source);
}

#[test]
fn issue_162_explicit_export_owns_precedence_over_competing_star_cycle() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(root.path(), "other.ts", "export class Leaf {}");
    write(
        root.path(),
        "a.ts",
        "export { Leaf } from './leaf'; export * from './b';",
    );
    write(
        root.path(),
        "b.ts",
        "export * from './a'; export * from './other';",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = resolver.local(&root.path().join("leaf.ts"), "Leaf", Namespace::Value);
    assert!(matches!(&expected, Resolution::Resolved(_)));
    assert_eq!(selected(root.path(), &mut resolver), expected);
}

#[test]
fn issue_162_cycle_branch_deletion_and_restoration_invalidate_grounded_proof() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "a.ts",
        "export * from './b'; export * from './leaf';",
    );
    write(root.path(), "b.ts", "export * from './a';");
    let mut before = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = selected(root.path(), &mut before);
    assert!(matches!(&expected, Resolution::Resolved(_)));
    std::fs::remove_file(root.path().join("b.ts")).unwrap();
    assert!(!before.host.current());
    let mut missing = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        selected(root.path(), &mut missing),
        Resolution::Unresolved(_)
    ));
    write(root.path(), "b.ts", "export * from './a';");
    assert!(!missing.host.current());
    let mut restored = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(selected(root.path(), &mut restored), expected);
}

#[test]
fn issue_162_type_only_cycle_keeps_type_identity_without_value_promotion() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "a.ts",
        "export type * from './b'; export type * from './leaf';",
    );
    write(root.path(), "b.ts", "export type * from './a';");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Type),
        Resolution::Resolved(_)
    ));
    assert!(matches!(
        selected(root.path(), &mut resolver),
        Resolution::Unresolved(_)
    ));
}
