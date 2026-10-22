//! Compiler library declarations are consumed authority, not built-in name catalogs.
use super::{Namespace, Resolution, Resolver, Witness};
use std::path::Path;

fn write(root: &Path, file: &str, source: &str) {
    let file = root.join(file);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
}
fn setup(root: &Path, options: serde_json::Value, prefix: &str) {
    write(
        root,
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    write(
        root,
        "node_modules/typescript/lib/lib.d.ts",
        "/// <reference no-default-lib=\"true\" />\ndeclare namespace API { class Core {} }",
    );
    write(
        root,
        "node_modules/typescript/lib/lib.dom.d.ts",
        "/// <reference no-default-lib=\"true\" />\ndeclare namespace API { class Dom {} }",
    );
    write(
        root,
        "app.ts",
        &format!("{prefix}\nexport class Control {{}}"),
    );
    let mut base = serde_json::json!({"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","target":"ES5","types":[]});
    base.as_object_mut()
        .unwrap()
        .extend(options.as_object().unwrap().clone());
    write(
        root,
        "tsconfig.json",
        &serde_json::json!({"compilerOptions":base,"files":["app.ts"]}).to_string(),
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
fn issue_162_compiler_library_default_source_and_no_lib_retraction() {
    let root = tempfile::tempdir().unwrap();
    for no_lib in [false, true, false] {
        setup(root.path(), serde_json::json!({"noLib":no_lib}), "");
        let (_, result) = member(root.path(), "Core");
        if no_lib {
            assert!(matches!(result, Resolution::Unresolved(_)));
        } else {
            assert!(
                matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("typescript/lib/lib.d.ts")),
                "{result:?}"
            );
        }
    }
}

#[test]
fn issue_162_compiler_library_explicit_list_replaces_default_selection() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), serde_json::json!({"lib":["dom"]}), "");
    let (_, dom) = member(root.path(), "Dom");
    assert!(
        matches!(dom, Resolution::Resolved(ref id) if id.source.ends_with("lib.dom.d.ts")),
        "{dom:?}"
    );
    assert!(matches!(
        member(root.path(), "Core").1,
        Resolution::Unresolved(_)
    ));
    setup(root.path(), serde_json::json!({"lib":[]}), "");
    assert!(matches!(
        member(root.path(), "Dom").1,
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_compiler_library_directive_survives_no_default_lib_marker() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), serde_json::json!({}), "");
    assert!(matches!(
        member(root.path(), "Core").1,
        Resolution::Resolved(_)
    ));
    setup(
        root.path(),
        serde_json::json!({}),
        "/// <reference no-default-lib=\"true\" />\n/// <reference lib=\"dom\" />",
    );
    let (_, dom) = member(root.path(), "Dom");
    assert!(matches!(dom, Resolution::Resolved(_)), "{dom:?}");
    assert!(matches!(
        member(root.path(), "Core").1,
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_compiler_library_deletion_records_witness_and_restores_new_kind() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), serde_json::json!({"lib":["dom"]}), "");
    assert!(matches!(
        member(root.path(), "Dom").1,
        Resolution::Resolved(_)
    ));
    let library = root.path().join("node_modules/typescript/lib/lib.dom.d.ts");
    std::fs::remove_file(&library).unwrap();
    let (resolver, missing) = member(root.path(), "Dom");
    assert!(matches!(missing, Resolution::Unresolved(_)));
    assert!(resolver.host.witnesses.iter().any(
        |witness| matches!(witness, Witness::Missing { path } if path.ends_with("lib.dom.d.ts"))
    ));
    std::fs::write(library, "declare namespace API { enum Dom {} }").unwrap();
    let (_, restored) = member(root.path(), "Dom");
    assert!(
        matches!(restored, Resolution::Resolved(ref id) if id.kind == "enum_declaration"),
        "{restored:?}"
    );
    assert!(!resolver.host.current());
}
