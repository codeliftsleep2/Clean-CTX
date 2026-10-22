use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, file: &str, source: &str) {
    std::fs::write(root.join(file), source).unwrap();
}
fn project(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    );
}

#[test]
fn issue_162_default_identifier_export_preserves_imported_declaration_identity() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(root.path(), "leaf.ts", "export class Leaf {}");
    write(
        root.path(),
        "barrel.ts",
        "import { Leaf as Local } from './leaf'; export default Local;",
    );
    write(root.path(), "app.ts", "import Selected from './barrel';");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = resolver.local(&root.path().join("leaf.ts"), "Leaf", Namespace::Value);
    assert!(matches!(expected, Resolution::Resolved(_)));
    for namespace in [Namespace::Value, Namespace::Type] {
        assert_eq!(
            resolver.local(&root.path().join("app.ts"), "Selected", namespace),
            expected
        );
    }
    write(root.path(), "barrel.ts", "export default Missing;");
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        fresh.local(&root.path().join("app.ts"), "Selected", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_written_type_identifier_is_not_a_type_only_import_keyword() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(root.path(), "leaf.ts", "export const type = 1;");
    write(
        root.path(),
        "app.ts",
        "import { type as Selected } from './leaf';",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let Resolution::Resolved(id) =
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value)
    else {
        panic!("ordinary import named type must remain value evidence");
    };
    assert_eq!(id.kind, "variable_declarator");
    assert_eq!(id.name, "type");
    assert!(matches!(
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Type),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_named_and_default_generators_have_value_only_declaration_identity() {
    for declaration in [
        "export function* Sequence() { yield 1; }",
        "export default function* Sequence() { yield 1; }",
    ] {
        let root = tempfile::tempdir().unwrap();
        project(root.path());
        write(root.path(), "leaf.ts", declaration);
        write(
            root.path(),
            "app.ts",
            if declaration.contains("default") {
                "import Selected from './leaf';"
            } else {
                "import { Sequence as Selected } from './leaf';"
            },
        );
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        assert!(matches!(
            resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value),
            Resolution::Resolved(_)
        ));
        assert!(matches!(
            resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Type),
            Resolution::Unresolved(_)
        ));
    }
}

#[test]
fn issue_162_duplicate_explicit_export_aliases_fail_closed_and_recover_after_removal() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(root.path(), "leaf.ts", "export class Leaf {}");
    write(
        root.path(),
        "app.ts",
        "import { Selected } from './barrel';",
    );
    let file = root.path().join("app.ts");
    for duplicate in [
        "export { Leaf as Selected } from './leaf'; export { Leaf as Selected } from './leaf';",
        "import { Leaf } from './leaf'; export { Leaf as Selected, Leaf as Selected };",
    ] {
        write(root.path(), "barrel.ts", duplicate);
        let mut invalid = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        assert!(matches!(
            invalid.local(&file, "Selected", Namespace::Value),
            Resolution::Unresolved(_)
        ));
        write(
            root.path(),
            "barrel.ts",
            "export { Leaf as Selected } from './leaf';",
        );
        assert!(!invalid.host.current());
        let mut valid = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        assert!(matches!(
            valid.local(&file, "Selected", Namespace::Value),
            Resolution::Resolved(_)
        ));
    }
}

#[test]
fn issue_162_quoted_export_names_and_escaped_module_paths_preserve_identity() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(
        root.path(),
        "leaf.ts",
        "class Leaf {} export { Leaf as 'quoted-name' };",
    );
    write(
        root.path(),
        "barrel.ts",
        r#"export { 'quoted-name' as "selected-name" } from './le\u0061f';"#,
    );
    write(
        root.path(),
        "app.ts",
        r#"import { "selected-name" as Selected } from './barrel';"#,
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let direct = resolver.local(&root.path().join("leaf.ts"), "Leaf", Namespace::Value);
    assert!(matches!(&direct, Resolution::Resolved(_)));
    assert_eq!(
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value),
        direct
    );
}

#[test]
fn issue_162_escaped_export_name_matches_its_decoded_symbol_not_raw_spelling() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(
        root.path(),
        "leaf.ts",
        r#"class Leaf {} export { Leaf as "item\u002dname" };"#,
    );
    write(
        root.path(),
        "app.ts",
        "import { 'item-name' as Selected } from './leaf';",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value),
        Resolution::Resolved(_)
    ));
    write(
        root.path(),
        "leaf.ts",
        "class Leaf {} export { Leaf as 'other-name' };",
    );
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        fresh.local(&root.path().join("app.ts"), "Selected", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_undecodable_external_module_never_borrows_a_local_export() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(
        root.path(),
        "app.ts",
        "import { Selected } from './barrel';",
    );
    for module in [r"./\uD800", r"./\u{110000}", r"./\01"] {
        write(
            root.path(),
            "barrel.ts",
            &format!("class Leaf {{}} export {{ Leaf as Selected }} from '{module}';"),
        );
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        assert!(
            matches!(
                resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value),
                Resolution::Unresolved(_)
            ),
            "{module}"
        );
    }
    write(
        root.path(),
        "barrel.ts",
        "class Leaf {} export { Leaf as Selected };",
    );
    let mut local = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        local.local(&root.path().join("app.ts"), "Selected", Namespace::Value),
        Resolution::Resolved(_)
    ));
}

#[test]
fn issue_162_unicode_export_spellings_converge_on_one_canonical_symbol() {
    let root = tempfile::tempdir().unwrap();
    project(root.path());
    write(
        root.path(),
        "leaf.ts",
        r#"class Leaf {} export { Leaf as "\uD83D\uDE80" };"#,
    );
    write(
        root.path(),
        "barrel.ts",
        r#"export { "\u{1F680}" as "selected\x2dname" } from './leaf';"#,
    );
    write(
        root.path(),
        "app.ts",
        "import { 'selected-name' as Selected } from './barrel';",
    );
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = resolver.local(&root.path().join("leaf.ts"), "Leaf", Namespace::Value);
    assert!(matches!(&expected, Resolution::Resolved(_)));
    assert_eq!(
        resolver.local(&root.path().join("app.ts"), "Selected", Namespace::Value),
        expected
    );
}
