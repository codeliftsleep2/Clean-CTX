//! Directive syntax and source-relative closure belong to the canonical program.
use super::{Namespace, Resolution, Resolver, Witness};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    std::fs::write(root.join(name), source).unwrap();
}
fn setup(root: &Path, no_resolve: bool) {
    write(
        root,
        "tsconfig.json",
        &serde_json::json!({"compilerOptions":{
        "module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy",
        "types":[],"noResolve":no_resolve},"files":["app.ts"]})
        .to_string(),
    );
}
fn lookup(root: &Path) -> (Resolver, Resolution) {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let file = root.join("app.ts");
    assert!(matches!(
        resolver.local(&file, "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    let result = resolver.namespace_member(&file, "API", "Leaf", Namespace::Value);
    (resolver, result)
}

#[test]
fn issue_162_program_reference_path_transitive_closure_and_no_resolve() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "app.ts",
        "/// <reference path=\"bridge.d.ts\" />\nexport class Control {}",
    );
    write(
        root.path(),
        "bridge.d.ts",
        "/// <reference path=\"leaf.d.ts\" />\n",
    );
    write(
        root.path(),
        "leaf.d.ts",
        "declare namespace API { class Leaf {} }",
    );
    for no_resolve in [false, true, false] {
        setup(root.path(), no_resolve);
        let (_, result) = lookup(root.path());
        if no_resolve {
            assert!(matches!(result, Resolution::Unresolved(_)));
        } else {
            assert!(
                matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("leaf.d.ts")),
                "{result:?}"
            );
        }
    }
}

#[test]
fn issue_162_program_reference_only_leading_comment_directives_admit_sources() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), false);
    write(
        root.path(),
        "leaf.d.ts",
        "declare namespace API { class Leaf {} }",
    );
    for (source, positive) in [
        (
            "/* preceding comment */\n/// <reference path=\"leaf.d.ts\" />\nexport class Control {}",
            true,
        ),
        (
            "export class Control {}\n/// <reference path=\"leaf.d.ts\" />",
            false,
        ),
        (
            "export class Control {} const text = '/// <reference path=\"leaf.d.ts\" />';",
            false,
        ),
    ] {
        write(root.path(), "app.ts", source);
        let (_, result) = lookup(root.path());
        if positive {
            assert!(matches!(result, Resolution::Resolved(_)), "{result:?}");
        } else {
            assert!(matches!(result, Resolution::Unresolved(_)));
        }
    }
}

#[test]
fn issue_162_program_reference_types_directive_adds_exact_declaration_package() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), false);
    write(
        root.path(),
        "app.ts",
        "/// <reference path=\"env.d.ts\" />\nexport class Control {}",
    );
    write(
        root.path(),
        "env.d.ts",
        "/// <reference types=\"support\" />\n",
    );
    let package = root.path().join("node_modules/@types/support");
    std::fs::create_dir_all(&package).unwrap();
    write(
        &package,
        "package.json",
        r#"{"name":"@types/support","types":"index.d.ts"}"#,
    );
    write(
        &package,
        "index.d.ts",
        "declare namespace API { class Leaf {} }",
    );
    let (_, result) = lookup(root.path());
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("support/index.d.ts")),
        "{result:?}"
    );
}

#[test]
fn issue_162_program_reference_missing_target_records_negative_witness_and_restores() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), false);
    write(
        root.path(),
        "app.ts",
        "/// <reference path=\"leaf.d.ts\" />\nexport class Control {}",
    );
    write(
        root.path(),
        "leaf.d.ts",
        "declare namespace API { class Leaf {} }",
    );
    assert!(matches!(lookup(root.path()).1, Resolution::Resolved(_)));
    std::fs::remove_file(root.path().join("leaf.d.ts")).unwrap();
    let (resolver, absent) = lookup(root.path());
    assert!(matches!(absent, Resolution::Unresolved(_)));
    assert!(resolver.host.witnesses.iter().any(
        |witness| matches!(witness, Witness::Missing { path } if path.ends_with("leaf.d.ts"))
    ));
    write(
        root.path(),
        "leaf.d.ts",
        "declare namespace API { enum Leaf {} }",
    );
    let (_, restored) = lookup(root.path());
    assert!(
        matches!(restored, Resolution::Resolved(ref id) if id.kind == "enum_declaration"),
        "{restored:?}"
    );
    assert!(!resolver.host.current());
}
