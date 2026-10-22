use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;

fn write(root: &Path, name: &str, text: &str) {
    std::fs::write(root.join(name), text).unwrap();
}
fn setup(root: &Path, declaration: &str, consumer: &str) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["app.ts","ambient.d.ts","control.ts"]}"#,
    );
    write(root, "ambient.d.ts", declaration);
    write(root, "control.ts", "export class Control {}");
    write(root, "app.ts", consumer);
}
fn resolver(root: &Path) -> Resolver {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&root.join("control.ts"), "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    resolver
}
fn resolved(result: Resolution) -> super::DeclarationId {
    let Resolution::Resolved(id) = result else {
        panic!("ambient identity: {result:?}");
    };
    assert!(id.end > id.start);
    id
}

#[test]
fn issue_162_ambient_module_named_import_owns_exact_source_hash_span() {
    let root = tempfile::tempdir().unwrap();
    let declaration = r#"declare module "virtual-api" { export class Leaf {} }"#;
    let source = "import { Leaf as Selected } from 'virtual-api'; const item = Selected;";
    setup(root.path(), declaration, source);
    let mut resolver = resolver(root.path());
    let value = resolved(resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value));
    assert_eq!(
        std::fs::canonicalize(&value.source).unwrap(),
        std::fs::canonicalize(root.path().join("ambient.d.ts")).unwrap()
    );
    assert_eq!(
        value.source_hash,
        crate::cache::content_hash(declaration.as_bytes())
    );
    assert_eq!(&declaration[value.start..value.end], "class Leaf {}");
    assert_eq!(
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Type),
        Resolution::Resolved(value)
    );
}

#[test]
fn issue_162_ambient_module_parts_merge_namespace_members_without_name_rescue() {
    let root = tempfile::tempdir().unwrap();
    let declaration = r#"declare module "virtual-api" { export namespace Core { export class Leaf {} } }
declare module "virtual-api" { export namespace Core { export interface Shape {} } }"#;
    setup(
        root.path(),
        declaration,
        "import * as API from 'virtual-api';",
    );
    let mut resolver = resolver(root.path());
    let file = root.path().join("app.ts");
    let leaf =
        resolved(resolver.namespace_chain(&file, "API", &["Core", "Leaf"], Namespace::Value));
    assert_eq!(&declaration[leaf.start..leaf.end], "class Leaf {}");
    let shape =
        resolved(resolver.namespace_chain(&file, "API", &["Core", "Shape"], Namespace::Type));
    assert_eq!(&declaration[shape.start..shape.end], "interface Shape {}");
    assert!(matches!(
        resolver.namespace_chain(&file, "API", &["Core", "Shape"], Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_ambient_module_external_augmentation_cannot_invent_missing_module() {
    let root = tempfile::tempdir().unwrap();
    let declaration = r#"declare module "virtual-api" { export class Leaf {} }"#;
    let source = "import { Leaf } from 'virtual-api';";
    setup(root.path(), declaration, source);
    assert!(matches!(
        resolver(root.path()).local(&root.path().join("app.ts"), "Leaf", Namespace::Value),
        Resolution::Resolved(_)
    ));
    write(
        root.path(),
        "ambient.d.ts",
        &format!("export {{}}; {declaration}"),
    );
    assert!(matches!(
        resolver(root.path()).local(&root.path().join("app.ts"), "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_ambient_module_member_kind_mutation_invalidates_consumed_source() {
    let root = tempfile::tempdir().unwrap();
    let declaration = r#"declare module "virtual-api" { export class Leaf {} }"#;
    let source = "import { Leaf } from 'virtual-api'; const item = Leaf;";
    setup(root.path(), declaration, source);
    let file = root.path().join("app.ts");
    let before = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let start = source.rfind("Leaf").unwrap();
    assert!(
        before
            .reference_sites
            .iter()
            .any(|site| site.0 == start && matches!(site.2, Resolution::Resolved(_)))
    );
    write(
        root.path(),
        "ambient.d.ts",
        r#"declare module "virtual-api" { export interface Leaf {} }"#,
    );
    assert!(before.witnesses.iter().any(|witness| !witness.current()));
    assert!(matches!(
        resolver(root.path()).local(&file, "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    assert!(matches!(
        resolver(root.path()).local(&file, "Leaf", Namespace::Type),
        Resolution::Resolved(_)
    ));
    write(root.path(), "ambient.d.ts", declaration);
    assert!(matches!(
        resolver(root.path()).local(&file, "Leaf", Namespace::Value),
        Resolution::Resolved(_)
    ));
}
