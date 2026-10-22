use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;

fn write(root: &Path, path: &str, source: &str) {
    std::fs::write(root.join(path), source).unwrap();
}
fn project(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    );
    write(
        root,
        "leaf.ts",
        "export class Leaf {} export interface Shape {}",
    );
}

#[test]
fn issue_162_namespace_members_share_export_identity_through_aliases_and_stars() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(
        root.path(),
        "namespace.ts",
        "export * as API from './leaf'; export class Sibling {}",
    );
    write(root.path(), "barrel.ts", "export * from './namespace';");
    write(
        root.path(),
        "target.ts",
        "import * as Direct from './leaf'; import { API as Via } from './barrel';",
    );
    let target = root.path().join("target.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let direct = resolver.namespace_member(&target, "Direct", "Leaf", Namespace::Value);
    assert!(matches!(direct, Resolution::Resolved(_)));
    assert_eq!(
        direct,
        resolver.namespace_member(&target, "Via", "Leaf", Namespace::Value)
    );
    assert!(matches!(
        resolver.namespace_member(&target, "Via", "Shape", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    assert!(matches!(
        resolver.namespace_member(&target, "Via", "Shape", Namespace::Type),
        Resolution::Resolved(_)
    ));
    write(
        root.path(),
        "consumer.ts",
        "import { Sibling } from './namespace';",
    );
    assert!(matches!(
        resolver.local(
            &root.path().join("consumer.ts"),
            "Sibling",
            Namespace::Value
        ),
        Resolution::Resolved(_)
    ));
}

#[test]
fn issue_162_namespace_ambiguity_and_type_only_imports_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(root.path(), "other.ts", "export class Leaf {}");
    write(root.path(), "a.ts", "export * as API from './leaf';");
    write(root.path(), "b.ts", "export * as API from './other';");
    write(
        root.path(),
        "barrel.ts",
        "export * from './a'; export * from './b';",
    );
    write(
        root.path(),
        "target.ts",
        "import { API } from './barrel'; import type * as Types from './leaf';",
    );
    let file = root.path().join("target.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.namespace_member(&file, "API", "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    assert!(matches!(
        resolver.namespace_member(&file, "Types", "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    assert!(matches!(
        resolver.namespace_member(&file, "Types", "Leaf", Namespace::Type),
        Resolution::Resolved(_)
    ));
}

#[test]
fn issue_162_namespace_occurrence_capture_retracts_when_export_changes() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    let source = "import * as API from './leaf'; const selected = API.Leaf;";
    let file = root.path().join("target.ts");
    write(root.path(), "target.ts", source);
    let before = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let start = source.rfind("API.Leaf").unwrap();
    assert!(
        before
            .reference_sites
            .iter()
            .any(|site| site.0 == start && matches!(site.2, Resolution::Resolved(_)))
    );
    write(root.path(), "leaf.ts", "export class Replacement {}");
    assert!(before.witnesses.iter().any(|witness| !witness.current()));
    let after = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert!(
        after
            .reference_sites
            .iter()
            .any(|site| site.0 == start && matches!(site.2, Resolution::Unresolved(_)))
    );
}

#[test]
fn issue_162_duplicate_local_namespace_aliases_do_not_become_unique_by_deduplication() {
    for imports in [
        "import * as API from './leaf'; import * as API from './leaf';",
        "import { API } from './barrel'; import { API } from './barrel';",
    ] {
        let root = tempfile::tempdir().unwrap();
        project(root.path());
        write(root.path(), "barrel.ts", "export * as API from './leaf';");
        write(root.path(), "target.ts", imports);
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        for namespace in [Namespace::Value, Namespace::Type] {
            assert!(matches!(
                resolver.namespace_member(&root.path().join("target.ts"), "API", "Leaf", namespace),
                Resolution::Unresolved(_)
            ));
        }
    }
}

#[test]
fn issue_162_namespace_export_diamond_remains_proven_and_tracks_retargeting() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(root.path(), "a.ts", "export * as API from './leaf';");
    write(root.path(), "b.ts", "export { API } from './a';");
    write(
        root.path(),
        "barrel.ts",
        "export * from './a'; export * from './b';",
    );
    write(root.path(), "target.ts", "import { API } from './barrel';");
    let file = root.path().join("target.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let direct = resolver.local(&root.path().join("leaf.ts"), "Leaf", Namespace::Value);
    assert!(matches!(&direct, Resolution::Resolved(_)));
    assert_eq!(
        resolver.namespace_member(&file, "API", "Leaf", Namespace::Value),
        direct
    );
    write(root.path(), "other.ts", "export class Leaf {}");
    write(root.path(), "b.ts", "export * as API from './other';");
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        fresh.namespace_member(&file, "API", "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    write(root.path(), "b.ts", "export { API } from './a';");
    assert!(!fresh.host.current());
    let mut restored = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        restored.namespace_member(&file, "API", "Leaf", Namespace::Value),
        direct
    );
}

#[test]
fn issue_162_duplicate_explicit_namespace_exports_do_not_hide_behind_equal_targets() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(
        root.path(),
        "barrel.ts",
        "export * as API from './leaf'; export * as API from './leaf';",
    );
    write(root.path(), "target.ts", "import { API } from './barrel';");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.namespace_member(
            &root.path().join("target.ts"),
            "API",
            "Leaf",
            Namespace::Value
        ),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_chained_namespace_occurrence_retains_terminal_declaration_identity() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(root.path(), "middle.ts", "export * as Core from './leaf';");
    let source = "import * as API from './middle'; const selected = API.Core.Leaf;";
    write(root.path(), "target.ts", source);
    let file = root.path().join("target.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = resolver.local(&root.path().join("leaf.ts"), "Leaf", Namespace::Value);
    assert!(matches!(&expected, Resolution::Resolved(_)));
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let start = source.find("API.Core.Leaf").unwrap();
    assert!(
        capture.reference_sites.iter().any(|(a, b, id)| *a == start
            && *b == start + "API.Core.Leaf".len()
            && *id == expected)
    );
    write(root.path(), "leaf.ts", "export class Replacement {}");
    assert!(capture.witnesses.iter().any(|w| !w.current()));
    let changed = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert!(changed.reference_sites.iter().any(|(a, b, id)| *a == start
        && *b == start + "API.Core.Leaf".len()
        && matches!(id, Resolution::Unresolved(_))));
}

#[test]
fn issue_162_chained_namespace_does_not_treat_class_properties_as_module_exports() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(
        root.path(),
        "middle.ts",
        "export class Core { static Leaf = 1; }",
    );
    write(root.path(), "target.ts", "import * as API from './middle';");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.namespace_chain(
            &root.path().join("target.ts"),
            "API",
            &["Core", "Leaf"],
            Namespace::Value
        ),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_chained_namespace_ambiguity_and_type_only_eligibility_are_preserved() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(
        root.path(),
        "middle.ts",
        "export type * as Core from './leaf';",
    );
    write(root.path(), "target.ts", "import * as API from './middle';");
    let file = root.path().join("target.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.namespace_chain(&file, "API", &["Core", "Shape"], Namespace::Type),
        Resolution::Resolved(_)
    ));
    assert!(matches!(
        resolver.namespace_chain(&file, "API", &["Core", "Leaf"], Namespace::Value),
        Resolution::Unresolved(_)
    ));
    write(root.path(), "other.ts", "export class Leaf {}");
    write(root.path(), "a.ts", "export * as Core from './leaf';");
    write(root.path(), "b.ts", "export * as Core from './other';");
    write(
        root.path(),
        "middle.ts",
        "export * from './a'; export * from './b';",
    );
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        fresh.namespace_chain(&file, "API", &["Core", "Leaf"], Namespace::Value),
        Resolution::Unresolved(_)
    ));
}
