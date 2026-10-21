use super::{Host, Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
}
fn resolver(root: &Path) -> Resolver {
    if !root.join("tsconfig.json").exists() {
        write(
            root,
            "tsconfig.json",
            r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
        );
    }
    Resolver::new(&[root.to_path_buf()]).unwrap()
}
fn declaration(resolution: Resolution) -> super::DeclarationId {
    match resolution {
        Resolution::Resolved(declaration) => declaration,
        other => panic!("expected exact declaration: {other:?}"),
    }
}

#[test]
fn issue_162_native_snapshot_reuses_exact_consumed_bytes() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("leaf.ts");
    write(root.path(), "leaf.ts", "export class Before {}");
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    let before = host.read(&file).unwrap();
    write(root.path(), "leaf.ts", "export class After {}");
    assert_eq!(host.read(&file).unwrap(), before);
    assert!(!host.current());
}

#[test]
fn issue_162_native_alias_barrel_diamond_preserves_one_declaration() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "leaf.ts", "export class Leaf {}");
    write(root.path(), "a.ts", "export { Leaf } from './leaf';");
    write(root.path(), "b.ts", "export * from './leaf';");
    write(
        root.path(),
        "barrel/index.ts",
        "export * from '../a'; export * from '../b';",
    );
    write(
        root.path(),
        "target.ts",
        "import { Leaf as Selected } from './barrel';",
    );
    let mut resolver = resolver(root.path());
    let found =
        declaration(resolver.local(&root.path().join("target.ts"), "Selected", Namespace::Value));
    assert_eq!(found.name, "Leaf");
    assert!(found.source.ends_with("leaf.ts"));
    assert_eq!(
        found.source_hash,
        crate::cache::content_hash(b"export class Leaf {}")
    );
    assert!(resolver.host.current());
}

#[test]
fn issue_162_native_star_conflict_and_type_only_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "a.ts", "export class Leaf {}");
    write(root.path(), "b.ts", "export class Leaf {}");
    write(
        root.path(),
        "barrel.ts",
        "export * from './a'; export * from './b';",
    );
    write(
        root.path(),
        "target.ts",
        "import { Leaf } from './barrel'; import type { Leaf as TypeLeaf } from './a';",
    );
    let mut resolver = resolver(root.path());
    assert!(
        matches!(resolver.local(&root.path().join("target.ts"), "Leaf", Namespace::Value), Resolution::Ambiguous(ref declarations) if declarations.len() == 2)
    );
    assert!(matches!(
        resolver.local(&root.path().join("target.ts"), "TypeLeaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    let _ =
        declaration(resolver.local(&root.path().join("target.ts"), "TypeLeaf", Namespace::Type));
}

#[test]
fn issue_162_native_export_cycle_keeps_rooted_symbol() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "leaf.ts", "export class Leaf {}");
    write(
        root.path(),
        "a.ts",
        "export * from './b'; export * from './leaf';",
    );
    write(root.path(), "b.ts", "export * from './a';");
    write(root.path(), "target.ts", "import { Leaf } from './b';");
    let found = declaration(resolver(root.path()).local(
        &root.path().join("target.ts"),
        "Leaf",
        Namespace::Value,
    ));
    assert!(found.source.ends_with("leaf.ts"));
}

#[test]
fn issue_162_native_jsonc_paths_and_config_mutation_are_witnessed() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "tsconfig.json",
        "{ // actual project\n\"compilerOptions\":{\"moduleResolution\":\"bundler\",\"paths\":{\"@app/*\":[\"./src/*\",],},},}",
    );
    write(root.path(), "src/leaf.ts", "export class Leaf {}");
    write(
        root.path(),
        "target.ts",
        "import { Leaf } from '@app/leaf';",
    );
    let mut resolver = resolver(root.path());
    let found =
        declaration(resolver.local(&root.path().join("target.ts"), "Leaf", Namespace::Value));
    assert!(found.source.ends_with("src/leaf.ts"));
    write(root.path(), "tsconfig.json", "{}");
    assert!(!resolver.host.current());
}

#[test]
fn issue_162_native_package_condition_order_and_negative_lookup() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    );
    write(root.path(), "target.ts", "import { Leaf } from 'example';");
    let mut before = resolver(root.path());
    assert!(matches!(
        before.local(&root.path().join("target.ts"), "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    write(
        root.path(),
        "node_modules/example/package.json",
        r#"{"exports":{".":{"types":"./types.ts","default":"./runtime.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/example/types.ts",
        "export class Leaf {}",
    );
    write(
        root.path(),
        "node_modules/example/runtime.ts",
        "export class Wrong {}",
    );
    assert!(!before.host.current());
    let found = declaration(resolver(root.path()).local(
        &root.path().join("target.ts"),
        "Leaf",
        Namespace::Value,
    ));
    assert!(found.source.ends_with("types.ts"));
}

#[test]
fn issue_162_native_unauthorized_dependency_is_not_read() {
    let root = tempfile::tempdir().unwrap();
    let foreign = tempfile::tempdir().unwrap();
    write(foreign.path(), "leaf.ts", "export class Leaf {}");
    let file = foreign.path().join("leaf.ts");
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(host.read(&file).is_none());
    assert!(!host.current());
    assert!(
        !host.witnesses.iter().any(
            |witness| matches!(witness, super::Witness::Content { path, .. } if path == &file)
        )
    );
}

#[test]
fn issue_162_native_supplied_snapshot_does_not_certify_later_disk_bytes() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "leaf.ts", "export class New {}");
    let authority = TypeScriptAuthority::capture(
        &root.path().join("leaf.ts"),
        "export class Old {}",
        &[root.path().to_path_buf()],
    );
    assert_eq!(
        authority.source_hash,
        crate::cache::content_hash(b"export class Old {}")
    );
    assert_eq!(authority.declarations[0].name, "Old");
}
