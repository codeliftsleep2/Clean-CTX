use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}
fn setup(root: &Path, a: &str, b: &str) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["app.ts","a.d.ts","b.d.ts","control.ts"]}"#,
    );
    write(root, "control.ts", "export class Control {}");
    write(root, "a.d.ts", a);
    write(root, "b.d.ts", b);
}
fn lookup(root: &Path, specifier: &str, namespace: Namespace) -> Resolution {
    write(
        root,
        "app.ts",
        &format!("import {{ Leaf }} from '{specifier}';"),
    );
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&root.join("control.ts"), "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    resolver.local(&root.join("app.ts"), "Leaf", namespace)
}
fn one(result: Resolution) -> super::DeclarationId {
    let Resolution::Resolved(id) = result else {
        panic!("ambient conformance: {result:?}");
    };
    assert!(id.end > id.start);
    id
}

#[test]
fn issue_162_ambient_conformance_exact_declaration_precedes_installed_file() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(
        p,
        r#"declare module "virtual-api" { export class Leaf {} }"#,
        "",
    );
    write(
        p,
        "node_modules/virtual-api/package.json",
        r#"{"types":"index.d.ts"}"#,
    );
    write(
        p,
        "node_modules/virtual-api/index.d.ts",
        "export class Leaf {}",
    );
    assert!(
        one(lookup(p, "virtual-api", Namespace::Value))
            .source
            .ends_with("a.d.ts")
    );
}

#[test]
fn issue_162_ambient_conformance_wildcard_longest_prefix_and_exact_override() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(
        p,
        r#"declare module "*.asset" { export interface Leaf {} }"#,
        r#"declare module "icons/*.asset" { export class Leaf {} }"#,
    );
    let chosen = one(lookup(p, "icons/logo.asset", Namespace::Value));
    assert!(chosen.source.ends_with("b.d.ts"));
    write(
        p,
        "a.d.ts",
        r#"declare module "icons/logo.asset" { export interface Leaf {} }"#,
    );
    assert!(matches!(
        lookup(p, "icons/logo.asset", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    assert!(
        one(lookup(p, "icons/logo.asset", Namespace::Type))
            .source
            .ends_with("a.d.ts")
    );
}

#[test]
fn issue_162_ambient_conformance_wildcard_follows_existing_physical_module() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(p, r#"declare module "*" { export interface Leaf {} }"#, "");
    write(
        p,
        "node_modules/virtual-api/package.json",
        r#"{"types":"index.d.ts"}"#,
    );
    write(
        p,
        "node_modules/virtual-api/index.d.ts",
        "export class Leaf {}",
    );
    assert!(
        one(lookup(p, "virtual-api", Namespace::Value))
            .source
            .ends_with("index.d.ts")
    );
}

#[test]
fn issue_162_ambient_conformance_cross_source_interfaces_keep_all_contributors() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(
        p,
        r#"declare module "virtual-api" { export interface Leaf { a: string } }"#,
        r#"declare module "virtual-api" { export interface Leaf { b: number } }"#,
    );
    let result = lookup(p, "virtual-api", Namespace::Type);
    let Resolution::Merged(ids) = result else {
        panic!("ambient interface group: {result:?}");
    };
    assert_eq!(ids.len(), 2);
    assert!(ids.iter().any(|id| id.source.ends_with("a.d.ts")));
    assert!(ids.iter().any(|id| id.source.ends_with("b.d.ts")));
    assert!(matches!(
        lookup(p, "virtual-api", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_ambient_conformance_cross_source_namespace_parts_keep_member_owners() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(
        p,
        r#"declare module "virtual-api" { export namespace Core { export class Leaf {} } }"#,
        r#"declare module "virtual-api" { export namespace Core { export interface Shape {} } }"#,
    );
    write(p, "app.ts", "import * as API from 'virtual-api';");
    let mut resolver = Resolver::new(&[p.to_path_buf()]).unwrap();
    let file = p.join("app.ts");
    let leaf = one(resolver.namespace_chain(&file, "API", &["Core", "Leaf"], Namespace::Value));
    assert!(leaf.source.ends_with("a.d.ts"));
    let shape = one(resolver.namespace_chain(&file, "API", &["Core", "Shape"], Namespace::Type));
    assert!(shape.source.ends_with("b.d.ts"));
}
