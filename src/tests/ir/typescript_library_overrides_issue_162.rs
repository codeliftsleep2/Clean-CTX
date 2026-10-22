use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}
fn setup(root: &Path, name: &str, directive: bool) {
    write(
        root,
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    write(
        root,
        &format!("node_modules/typescript/lib/lib.{name}.d.ts"),
        "/// <reference no-default-lib=\"true\" />\ndeclare namespace API { class Builtin {} }",
    );
    let source = if directive {
        format!("/// <reference lib=\"{name}\" />\nexport class Control {{}}")
    } else {
        "export class Control {}".into()
    };
    write(root, "app.ts", &source);
    let libs = if directive {
        Vec::<String>::new()
    } else {
        vec![name.to_string()]
    };
    write(root, "tsconfig.json", &serde_json::json!({"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","types":[],"lib":libs},"files":["app.ts"]}).to_string());
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
fn issue_162_library_override_package_appearance_replaces_installed_source() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "dom", false);
    let (resolver, builtin) = member(root.path(), "Builtin");
    assert!(
        matches!(builtin, Resolution::Resolved(_)),
        "control: {builtin:?}"
    );
    write(
        root.path(),
        "node_modules/@typescript/lib-dom/package.json",
        r#"{"name":"@types/web","types":"index.d.ts"}"#,
    );
    write(
        root.path(),
        "node_modules/@typescript/lib-dom/index.d.ts",
        "declare namespace API { class Override {} }",
    );
    assert!(!resolver.host.current());
    let (_, replacement) = member(root.path(), "Override");
    assert!(
        matches!(replacement, Resolution::Resolved(ref id) if id.source.ends_with("@typescript/lib-dom/index.d.ts")),
        "{replacement:?}"
    );
    assert!(matches!(
        member(root.path(), "Builtin").1,
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_library_override_directive_manifest_retarget_preserves_canonical_source() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "dom", true);
    assert!(matches!(
        member(root.path(), "Builtin").1,
        Resolution::Resolved(_)
    ));
    for (target, kind) in [("first.d.ts", "class"), ("second.d.ts", "enum")] {
        write(
            root.path(),
            &format!("node_modules/@typescript/lib-dom/{target}"),
            &format!("declare namespace API {{ {kind} Override {{}} }}"),
        );
        write(
            root.path(),
            "node_modules/@typescript/lib-dom/package.json",
            &serde_json::json!({"name":"@typescript/lib-dom","types":target}).to_string(),
        );
        let (_, result) = member(root.path(), "Override");
        let Resolution::Resolved(id) = result else {
            panic!("override: {result:?}");
        };
        assert!(id.source.ends_with(target));
        assert_eq!(
            id.kind,
            if kind == "class" {
                "class_declaration"
            } else {
                "enum_declaration"
            }
        );
    }
}

#[test]
fn issue_162_library_override_component_name_selects_package_subpath() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "dom.iterable", false);
    assert!(matches!(
        member(root.path(), "Builtin").1,
        Resolution::Resolved(_)
    ));
    write(
        root.path(),
        "node_modules/@typescript/lib-dom/package.json",
        r#"{"name":"@typescript/lib-dom","types":"index.d.ts"}"#,
    );
    write(
        root.path(),
        "node_modules/@typescript/lib-dom/index.d.ts",
        "declare namespace API { class WrongRoot {} }",
    );
    write(
        root.path(),
        "node_modules/@typescript/lib-dom/iterable.d.ts",
        "declare namespace API { class Override {} }",
    );
    let (_, result) = member(root.path(), "Override");
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("lib-dom/iterable.d.ts")),
        "{result:?}"
    );
    assert!(matches!(
        member(root.path(), "WrongRoot").1,
        Resolution::Unresolved(_)
    ));
}
