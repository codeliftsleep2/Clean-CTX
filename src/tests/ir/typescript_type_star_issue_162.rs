use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    std::fs::write(root.join(name), source).unwrap();
}

#[test]
fn issue_162_type_star_forms_keep_type_authority_and_retract_runtime_authority() {
    for export in [
        "export type * from './leaf';",
        "export /*a*/ type /*b*/ * from './leaf';",
    ] {
        let root = tempfile::tempdir().unwrap();
        write(
            root.path(),
            "tsconfig.json",
            r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
        );
        write(root.path(), "leaf.ts", "export class Leaf {}");
        write(root.path(), "barrel.ts", export);
        write(root.path(), "app.ts", "import { Leaf } from './barrel';");
        let file = root.path().join("app.ts");
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        assert!(matches!(
            resolver.local(&file, "Leaf", Namespace::Type),
            Resolution::Resolved(_)
        ));
        assert!(matches!(
            resolver.local(&file, "Leaf", Namespace::Value),
            Resolution::Unresolved(_)
        ));
        write(root.path(), "barrel.ts", "export * from './leaf';");
        assert!(!resolver.host.current());
        let mut changed = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        assert!(matches!(
            changed.local(&file, "Leaf", Namespace::Value),
            Resolution::Resolved(_)
        ));
    }
}

#[test]
fn issue_162_type_star_frontend_keeps_original_declaration_offsets_and_hash() {
    let source = "export type * as Core from './leaf';\nexport class Local {}";
    let unit = super::syntax::Unit::parse(Path::new("fixture.ts"), source).unwrap();
    let super::syntax::Binding::Declaration(id, _) = &unit.locals["Local"][0] else {
        panic!("local class");
    };
    assert_eq!(&source[id.start..id.end], "class Local {}");
    assert_eq!(
        id.source_hash,
        crate::cache::content_hash(source.as_bytes())
    );
    assert!(matches!(
        &unit.exports["Core"][0],
        super::syntax::Binding::Namespace {
            type_only: true,
            ..
        }
    ));
}

#[test]
fn issue_162_type_star_frontend_does_not_repair_other_malformed_syntax() {
    for source in [
        "export type * as Core from './leaf'; const = ;",
        "export type * as Core;",
        "function f() { export type * from './leaf'; }",
        "const text = 'export type *'; const = ;",
    ] {
        assert!(
            super::syntax::Unit::parse(Path::new("fixture.ts"), source).is_none(),
            "{source}"
        );
    }
}
