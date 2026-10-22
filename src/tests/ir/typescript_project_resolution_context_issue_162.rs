use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, file: &str, text: &str) {
    let file = root.join(file);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","paths":{"@leaf":["./root-leaf.ts"]}}}"#,
    );
    write(
        root,
        "nested/tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","paths":{"@leaf":["../other-leaf.ts"]}}}"#,
    );
    write(root, "root-leaf.ts", "export class Leaf {}");
    write(root, "other-leaf.ts", "export class Leaf {}");
    write(root, "nested/barrel.ts", "export { Leaf } from '@leaf';");
    write(root, "nested/own.ts", "import { Leaf } from '@leaf';");
    write(root, "app.ts", "import { Leaf } from './nested/barrel';");
}
fn leaf(resolver: &mut Resolver, file: &Path) -> super::DeclarationId {
    match resolver.local(file, "Leaf", Namespace::Value) {
        Resolution::Resolved(id) => id,
        other => panic!("nonempty context-owned identity required: {other:?}"),
    }
}

#[test]
fn issue_162_export_chain_keeps_initiating_project_and_restores_request_context() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let app = leaf(&mut resolver, &root.path().join("app.ts"));
    assert!(app.source.ends_with("root-leaf.ts"));
    let own = leaf(&mut resolver, &root.path().join("nested/own.ts"));
    assert!(own.source.ends_with("other-leaf.ts"));
    assert_eq!(leaf(&mut resolver, &root.path().join("app.ts")), app);
}

#[test]
fn issue_162_unrelated_nested_config_is_not_an_export_chain_resolution_witness() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let app = leaf(&mut resolver, &root.path().join("app.ts"));
    assert!(app.source.ends_with("root-leaf.ts"));
    write(root.path(), "nested/tsconfig.json", "{}");
    assert!(resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(leaf(&mut fresh, &root.path().join("app.ts")), app);
    write(
        root.path(),
        "root-leaf.ts",
        "export class Leaf { changed = true; }",
    );
    assert!(!resolver.host.current());
}
