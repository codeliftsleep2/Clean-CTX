//! Automatic type admission is project-owned and distinct from written directives.
use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
}
fn setup(root: &Path, options: serde_json::Value) {
    compiler(root);
    write(root, "app.ts", "export class Control {}");
    let mut base = serde_json::json!({"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy"});
    base.as_object_mut()
        .unwrap()
        .extend(options.as_object().unwrap().clone());
    write(
        root,
        "tsconfig.json",
        &serde_json::json!({"compilerOptions":base,"files":["app.ts"]}).to_string(),
    );
}
fn compiler(root: &Path) {
    write(
        root,
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    write(
        root,
        "node_modules/typescript/lib/lib.d.ts",
        "/// <reference no-default-lib=\"true\" />\n// No library declarations participate in this admission fixture.\n",
    );
}
fn member(root: &Path, name: &str) -> (Resolver, Resolution) {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let file = root.join("app.ts");
    assert!(matches!(
        resolver.local(&file, "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    let result = resolver.namespace_member(&file, "API", name, Namespace::Value);
    (resolver, result)
}

#[test]
fn issue_162_automatic_types_explicit_allowlist_and_empty_list_retract_globals() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "node_modules/@types/one/index.d.ts",
        "declare namespace API { class One {} }",
    );
    write(
        root.path(),
        "node_modules/@types/two/index.d.ts",
        "declare namespace API { class Two {} }",
    );
    for types in [
        serde_json::json!(["one"]),
        serde_json::json!([]),
        serde_json::json!(["two"]),
    ] {
        setup(root.path(), serde_json::json!({"types":types}));
        for name in ["One", "Two"] {
            let (_, result) = member(root.path(), name);
            let expected = types
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str() == Some(if name == "One" { "one" } else { "two" }));
            if expected {
                assert!(
                    matches!(result, Resolution::Resolved(_)),
                    "{name}: {result:?}"
                );
            } else {
                assert!(
                    matches!(result, Resolution::Unresolved(_)),
                    "{name}: {result:?}"
                );
            }
        }
    }
}

#[test]
fn issue_162_automatic_types_default_visible_packages_and_inventory_invalidation() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), serde_json::json!({}));
    write(
        root.path(),
        "node_modules/@types/one/index.d.ts",
        "declare namespace API { class One {} }",
    );
    let (resolver, positive) = member(root.path(), "One");
    assert!(matches!(positive, Resolution::Resolved(_)), "{positive:?}");
    assert!(resolver.host.current());
    write(
        root.path(),
        "node_modules/@types/two/index.d.ts",
        "declare namespace API { class Two {} }",
    );
    assert!(!resolver.host.current());
    assert!(matches!(
        member(root.path(), "Two").1,
        Resolution::Resolved(_)
    ));
}

#[test]
fn issue_162_automatic_types_custom_roots_use_declaring_config_origin() {
    let root = tempfile::tempdir().unwrap();
    compiler(root.path());
    write(
        root.path(),
        "config/base.json",
        r#"{"compilerOptions":{"typeRoots":["./types"]}}"#,
    );
    write(
        root.path(),
        "config/types/support/index.d.ts",
        "declare namespace API { class Selected {} }",
    );
    write(
        root.path(),
        "node_modules/@types/foreign/index.d.ts",
        "declare namespace API { class Foreign {} }",
    );
    write(root.path(), "app.ts", "export class Control {}");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"extends":"./config/base.json","compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy"},"files":["app.ts"]}"#,
    );
    let (_, selected) = member(root.path(), "Selected");
    assert!(
        matches!(selected, Resolution::Resolved(ref id) if id.source.ends_with("config/types/support/index.d.ts")),
        "{selected:?}"
    );
    assert!(matches!(
        member(root.path(), "Foreign").1,
        Resolution::Unresolved(_)
    ));
}
