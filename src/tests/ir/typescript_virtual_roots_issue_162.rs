use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, file: &str, source: &str) {
    let file = root.join(file);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
}
fn setup(root: &Path, roots: &str) {
    write(
        root,
        "tsconfig.json",
        &format!(
            r#"{{"compilerOptions":{{"module":"ESNext","moduleResolution":"bundler","rootDirs":{roots}}}}}"#
        ),
    );
    write(root, "src/views/app.ts", "import { Leaf } from './leaf';");
}
fn selected(resolver: &mut Resolver, root: &Path) -> super::DeclarationId {
    match resolver.local(&root.join("src/views/app.ts"), "Leaf", Namespace::Value) {
        Resolution::Resolved(id) => id,
        other => panic!("nonempty virtual-root declaration required: {other:?}"),
    }
}

#[test]
fn issue_162_virtual_root_fallback_invalidates_when_local_candidate_appears() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), r#"["src", "generated"]"#);
    write(
        root.path(),
        "generated/views/leaf.ts",
        "export class Leaf {}",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        selected(&mut resolver, root.path())
            .source
            .ends_with("generated/views/leaf.ts")
    );
    assert!(resolver.host.current());
    write(root.path(), "src/views/leaf.ts", "export class Leaf {}");
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        selected(&mut fresh, root.path())
            .source
            .ends_with("src/views/leaf.ts")
    );
}

#[test]
fn issue_162_virtual_roots_use_longest_prefix_and_inherited_option_origin() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "config/base.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","rootDirs":["../src","../src/views","../generated"]}}"#,
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"extends":"./config/base.json"}"#,
    );
    write(
        root.path(),
        "src/views/app.ts",
        "import { Leaf } from './leaf';",
    );
    write(root.path(), "generated/leaf.ts", "export class Leaf {}");
    write(
        root.path(),
        "generated/views/leaf.ts",
        "export class Wrong {}",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(
        selected(&mut resolver, root.path())
            .source
            .ends_with("generated/leaf.ts")
    );
    assert!(resolver.host.current());
}

#[test]
fn issue_162_virtual_root_config_retarget_retracts_old_declaration() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), r#"["src", "before"]"#);
    write(root.path(), "before/views/leaf.ts", "export class Leaf {}");
    write(root.path(), "after/views/leaf.ts", "export class Leaf {}");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let before = selected(&mut resolver, root.path());
    setup(root.path(), r#"["src", "after"]"#);
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let after = selected(&mut fresh, root.path());
    assert_ne!(before.source, after.source);
    assert!(after.source.ends_with("after/views/leaf.ts"));
}
