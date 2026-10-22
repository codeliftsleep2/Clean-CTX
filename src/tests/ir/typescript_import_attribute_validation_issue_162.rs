use super::{Namespace, Resolution, Resolver};

#[test]
fn issue_162_import_attribute_validation_invalid_modes_refuse_authority() {
    for attributes in [
        r#"{ "resolution-mode": "other" }"#,
        r#"{ "resolution-mode": "import", "resolution-mode": "require" }"#,
        r#"{ "resolution-mode": choice }"#,
    ] {
        let source = format!("import type {{ Leaf }} from 'pkg' with {attributes};");
        assert!(
            super::syntax::Unit::parse(std::path::Path::new("app.mts"), &source).is_none(),
            "{source}"
        );
    }
    let runtime = r#"import { Leaf } from 'pkg' with { "resolution-mode": "require" };"#;
    assert!(super::syntax::Unit::parse(std::path::Path::new("app.mts"), runtime).is_none());
}

#[test]
fn issue_162_import_attribute_validation_namespace_binding_uses_selected_mode() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    std::fs::create_dir_all(path.join("node_modules/pkg")).unwrap();
    std::fs::write(path.join("tsconfig.json"), r#"{"compilerOptions":{"module":"NodeNext","moduleResolution":"nodenext","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    std::fs::write(
        path.join("node_modules/pkg/package.json"),
        r#"{"exports":{"import":"./esm.d.mts","require":"./cjs.d.cts"}}"#,
    )
    .unwrap();
    for name in ["esm.d.mts", "cjs.d.cts"] {
        std::fs::write(
            path.join("node_modules/pkg").join(name),
            "export class Leaf {}",
        )
        .unwrap();
    }
    std::fs::write(
        path.join("app.mts"),
        r#"import type * as API from 'pkg' with { "resolution-mode": "require" };"#,
    )
    .unwrap();
    let mut resolver = Resolver::new(&[path.to_path_buf()]).unwrap();
    let expected = resolver.local(
        &path.join("node_modules/pkg/cjs.d.cts"),
        "Leaf",
        Namespace::Type,
    );
    assert!(matches!(expected, Resolution::Resolved(_)));
    assert_eq!(
        resolver.namespace_member(&path.join("app.mts"), "API", "Leaf", Namespace::Type),
        expected
    );
    assert!(matches!(
        resolver.namespace_member(&path.join("app.mts"), "API", "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}
