use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}
fn config(root: &Path, preserve: bool) {
    write(
        root,
        "tsconfig.json",
        &format!(
            r#"{{"compilerOptions":{{"module":"ESNext","moduleResolution":"bundler","preserveSymlinks":{preserve}}}}}"#
        ),
    );
}
fn fixture(root: &Path) {
    config(root, false);
    write(root, "app.ts", "import { Leaf } from 'pkg';");
    write(root, "packages/pkg/package.json", r#"{"types":"index.ts"}"#);
    write(
        root,
        "packages/pkg/index.ts",
        "export { Leaf } from 'dep'; export class Own {}",
    );
    write(
        root,
        "packages/node_modules/dep/index.ts",
        "export class Leaf {}",
    );
    write(root, "node_modules/dep/index.ts", "export class Leaf {}");
}

#[cfg(any(unix, windows))]
#[test]
fn issue_162_preserve_symlinks_changes_lookup_context_not_canonical_declaration_identity() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let target = root.path().join("packages/pkg");
    let link = root.path().join("node_modules/pkg");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    #[cfg(windows)]
    let _junction = super::windows_host_tests::junction(&link, &target);
    let file = root.path().join("app.ts");
    let mut physical = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let Resolution::Resolved(real) = physical.local(&file, "Leaf", Namespace::Value) else {
        panic!("physical dependency");
    };
    assert!(real.source.ends_with("packages/node_modules/dep/index.ts"));
    let real_own = physical.local(&target.join("index.ts"), "Own", Namespace::Value);
    assert!(matches!(&real_own, Resolution::Resolved(_)));
    config(root.path(), true);
    assert!(!physical.host.current());
    let mut logical = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let Resolution::Resolved(alias) = logical.local(&file, "Leaf", Namespace::Value) else {
        panic!("logical dependency");
    };
    assert!(alias.source.ends_with("node_modules/dep/index.ts"));
    assert!(!alias.source.ends_with("packages/node_modules/dep/index.ts"));
    assert_ne!(real.source, alias.source);
    assert_eq!(
        logical.local(&link.join("index.ts"), "Own", Namespace::Value),
        real_own
    );
}

#[cfg(any(unix, windows))]
#[test]
fn issue_162_preserved_logical_route_does_not_authorize_outside_physical_sources() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    config(root.path(), true);
    let outside = tempfile::tempdir().unwrap();
    write(outside.path(), "index.ts", "export class Leaf {}");
    let link = root.path().join("node_modules/pkg");
    #[cfg(unix)]
    std::os::unix::fs::symlink(outside.path(), &link).unwrap();
    #[cfg(windows)]
    let _junction = super::windows_host_tests::junction(&link, outside.path());
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&root.path().join("app.ts"), "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    assert!(
        resolver
            .host
            .witnesses
            .iter()
            .any(|witness| matches!(witness, super::Witness::OutsideScope { .. }))
    );
}

#[test]
fn issue_162_preserve_symlinks_option_requires_boolean_project_evidence() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","preserveSymlinks":"true"}}"#,
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(resolver.module(&root.path().join("app.ts"), "pkg").is_err());
}
