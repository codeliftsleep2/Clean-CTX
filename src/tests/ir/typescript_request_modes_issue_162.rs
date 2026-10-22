use super::module_resolution::ModuleRequestKind;
use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"NodeNext","moduleResolution":"nodenext"}}"#,
    );
    write(
        root,
        "node_modules/pkg/package.json",
        r#"{"exports":{"import":"./esm.mts","require":"./cjs.cts"}}"#,
    );
    write(root, "node_modules/pkg/esm.mts", "export class Leaf {}");
    write(
        root,
        "node_modules/pkg/cjs.cts",
        "class Leaf {} export = Leaf;",
    );
}

#[test]
fn issue_162_dynamic_import_uses_import_conditions_in_commonjs_source() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let file = root.path().join("app.cts");
    let source = "import Selected = require('pkg'); const lazy = import('pkg');";
    write(root.path(), "app.cts", source);
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert_eq!(capture.module_requests.len(), 2);
    assert!(
        capture
            .module_requests
            .iter()
            .any(|r| r.kind == ModuleRequestKind::Require
                && r.resolved.as_ref().unwrap().ends_with("cjs.cts"))
    );
    assert!(
        capture
            .module_requests
            .iter()
            .any(|r| r.kind == ModuleRequestKind::Import
                && r.resolved.as_ref().unwrap().ends_with("esm.mts"))
    );
}

#[test]
fn issue_162_require_request_in_esm_source_retains_require_extension_rules() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(root.path(), "app.mts", "export {};");
    write(root.path(), "leaf.ts", "export class Leaf {}");
    let file = root.path().join("app.mts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(resolver.module(&file, "./leaf").is_err());
    assert!(
        resolver
            .module_request(&file, "./leaf", ModuleRequestKind::Require)
            .unwrap()
            .ends_with("leaf.ts")
    );
    assert!(
        resolver
            .module_request(&file, "pkg", ModuleRequestKind::Require)
            .unwrap()
            .ends_with("cjs.cts")
    );
    assert!(
        resolver
            .module_request(&file, "pkg", ModuleRequestKind::Import)
            .unwrap()
            .ends_with("esm.mts")
    );
}

#[test]
fn issue_162_import_equals_follows_export_assignment_declaration_identity() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(root.path(), "app.mts", "import Selected = require('pkg');");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = resolver.local(
        &root.path().join("node_modules/pkg/cjs.cts"),
        "Leaf",
        Namespace::Value,
    );
    assert!(matches!(&expected, Resolution::Resolved(_)));
    for namespace in [Namespace::Value, Namespace::Type] {
        assert_eq!(
            resolver.local(&root.path().join("app.mts"), "Selected", namespace),
            expected
        );
    }
}

#[test]
fn issue_162_quoted_export_name_cannot_impersonate_an_export_assignment() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/cjs.cts",
        "class Leaf {} export { Leaf as 'export=' };",
    );
    write(root.path(), "app.cts", "import Selected = require('pkg');");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&root.path().join("app.cts"), "Selected", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_import_equals_namespace_members_use_require_selected_module() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "node_modules/pkg/cjs.cts",
        "export class Leaf {}",
    );
    write(root.path(), "app.mts", "import API = require('pkg');");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let Resolution::Resolved(id) = resolver.namespace_member(
        &root.path().join("app.mts"),
        "API",
        "Leaf",
        Namespace::Value,
    ) else {
        panic!("require namespace identity");
    };
    assert!(id.source.ends_with("cjs.cts"));
}

#[test]
fn issue_162_require_spelled_calls_are_not_syntax_owned_module_requests() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let source = "function require(name: string) { return name; } const x = require('pkg'); const y = import('pkg');";
    let file = root.path().join("app.cts");
    write(root.path(), "app.cts", source);
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert_eq!(capture.module_requests.len(), 1);
    assert_eq!(capture.module_requests[0].kind, ModuleRequestKind::Import);
    assert_eq!(
        &source[capture.module_requests[0].start..capture.module_requests[0].end],
        "'pkg'"
    );
}

#[test]
fn issue_162_require_assignment_cycles_and_type_only_imports_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "app.cts",
        "import type Selected = require('pkg');",
    );
    let file = root.path().join("app.cts");
    let mut before = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        before.local(&file, "Selected", Namespace::Type),
        Resolution::Resolved(_)
    ));
    assert!(matches!(
        before.local(&file, "Selected", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    write(
        root.path(),
        "node_modules/pkg/cjs.cts",
        "import Self = require('pkg'); export = Self;",
    );
    assert!(!before.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        fresh.local(&file, "Selected", Namespace::Type),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_bundler_preserve_mixed_request_syntax_uses_distinct_conditions() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"preserve","moduleResolution":"bundler"}}"#,
    );
    let source = "import { Leaf } from 'pkg'; import Selected = require('pkg');";
    let file = root.path().join("app.ts");
    write(root.path(), "app.ts", source);
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert_eq!(capture.module_requests.len(), 2);
    assert!(
        capture.module_requests[0]
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("esm.mts")
    );
    assert!(
        capture.module_requests[1]
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("cjs.cts")
    );
}
