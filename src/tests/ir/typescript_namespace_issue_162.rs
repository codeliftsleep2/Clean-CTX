use super::{Namespace, Resolution, Resolver};

#[test]
fn issue_162_reference_occurrences_do_not_bypass_competing_local_symbols() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("source.ts");
    let source = "import { Leaf } from './leaf'; class Leaf {} const selected = Leaf;";
    std::fs::write(&file, source).unwrap();
    std::fs::write(root.path().join("leaf.ts"), "export class Leaf {}").unwrap();
    let capture = super::TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let start = source.rfind("Leaf").unwrap();
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == start && matches!(site.2, Resolution::Unresolved(_)))
    );
}

#[test]
fn issue_162_reference_binding_rejects_text_from_a_different_snapshot() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("source.ts");
    let consumed = "import { Original as Selected } from './before'; const x = Selected;";
    let changed = "import { Original as Selected } from './after_'; const x = Selected;";
    std::fs::write(&file, consumed).unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    resolver.host.seed(&file, consumed).unwrap();
    let start = changed.rfind("Selected").unwrap();
    let span =
        crate::ir::invocation_evidence::SourceSpan::new(start, start + "Selected".len()).unwrap();
    let result = resolver.value_reference(&file, changed, span);
    assert!(matches!(result, Some(Resolution::Unresolved(reason)) if reason.contains("snapshot")));
    assert_eq!(resolver.host.read(&file).unwrap().as_ref(), consumed);
}

#[test]
fn issue_162_reference_sites_resolve_aliases_without_certifying_nested_shadows() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    std::fs::write(root.path().join("leaf.ts"), "export class Original {}").unwrap();
    let file = root.path().join("source.ts");
    let source = "import { Original as Selected } from './leaf';\nconst direct = Selected;\nfunction f(Selected: unknown) { return Selected; }";
    std::fs::write(&file, source).unwrap();
    let capture = super::TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let start = source.find("= Selected").unwrap() + 2;
    let site = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == start)
        .expect("exact top-level reference");
    let Resolution::Resolved(id) = &site.2 else {
        panic!("reference must resolve through its imported alias");
    };
    assert_eq!(id.name, "Original");
    assert!(id.source.ends_with("leaf.ts"));
    let shadow = source.rfind("Selected").unwrap();
    let (_, _, Resolution::Resolved(local)) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == shadow)
        .expect("exact shadowed parameter reference")
    else {
        panic!("the parameter must resolve locally rather than inherit the import");
    };
    assert_eq!(local.kind, "required_parameter");
    assert_eq!(local.source, capture.source);
    assert_ne!(local.source, id.source);
}

#[test]
fn issue_162_anonymous_defaults_retain_exact_snapshot_and_no_local_default_binding() {
    for (source, type_identity) in [
        ("export default class {}", true),
        ("export default function() {}", false),
    ] {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
        )
        .unwrap();
        let leaf = root.path().join("leaf.ts");
        std::fs::write(&leaf, source).unwrap();
        let target = root.path().join("target.ts");
        std::fs::write(&target, "import Selected from './leaf';").unwrap();
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        let Resolution::Resolved(id) = resolver.local(&target, "Selected", Namespace::Value) else {
            panic!("anonymous default must retain declaration identity: {source}");
        };
        assert_eq!(
            id.source_hash,
            crate::cache::content_hash(source.as_bytes())
        );
        assert_eq!(
            &source[id.start..id.end],
            source.strip_prefix("export default ").unwrap()
        );
        assert!(resolver.source_declarations(&leaf).unwrap().contains(&id));
        assert!(matches!(
            resolver.local(&leaf, "default", Namespace::Value),
            Resolution::Unresolved(_)
        ));
        assert_eq!(
            matches!(
                resolver.local(&target, "Selected", Namespace::Type),
                Resolution::Resolved(_)
            ),
            type_identity
        );
        std::fs::write(&leaf, "export default class Replacement {}").unwrap();
        assert!(!resolver.host.current());
        assert_eq!(
            resolver.local(&target, "Selected", Namespace::Value),
            Resolution::Resolved(id)
        );
    }
}

#[test]
fn issue_162_named_default_export_keeps_local_identity_without_named_export() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    let leaf = root.path().join("leaf.ts");
    std::fs::write(&leaf, "export default class Internal {}").unwrap();
    let target = root.path().join("target.ts");
    std::fs::write(
        &target,
        "import Selected from './leaf'; import { Internal as Invalid } from './leaf';",
    )
    .unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let expected = resolver.local(&leaf, "Internal", Namespace::Value);
    assert!(matches!(expected, Resolution::Resolved(_)));
    assert_eq!(
        resolver.local(&target, "Selected", Namespace::Value),
        expected
    );
    assert!(matches!(
        resolver.local(&target, "Invalid", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    assert_eq!(
        resolver.local(&target, "Selected", Namespace::Type),
        expected
    );
}

#[test]
fn issue_162_star_inside_comment_is_not_a_star_export() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    std::fs::write(
        root.path().join("leaf.ts"),
        "export class Selected {} export class Hidden {}",
    )
    .unwrap();
    std::fs::write(
        root.path().join("barrel.ts"),
        "export /* * */ { Selected } from './leaf';",
    )
    .unwrap();
    let target = root.path().join("target.ts");
    std::fs::write(&target, "import { Selected, Hidden } from './barrel';").unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&target, "Selected", Namespace::Value),
        Resolution::Resolved(_)
    ));
    assert!(matches!(
        resolver.local(&target, "Hidden", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_value_declarations_do_not_invent_type_identity() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("source.ts");
    std::fs::write(&file, "const Value = 1; function Callable() {}").unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    for name in ["Value", "Callable"] {
        assert!(matches!(
            resolver.local(&file, name, Namespace::Value),
            Resolution::Resolved(_)
        ));
        assert!(matches!(
            resolver.local(&file, name, Namespace::Type),
            Resolution::Unresolved(_)
        ));
    }
}

#[test]
fn issue_162_separate_type_value_bindings_keep_distinct_declaration_spans() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("source.ts");
    std::fs::write(&file, "interface Shared {} const Shared = 1;").unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let Resolution::Resolved(value) = resolver.local(&file, "Shared", Namespace::Value) else {
        panic!("value binding must resolve independently");
    };
    let Resolution::Resolved(ty) = resolver.local(&file, "Shared", Namespace::Type) else {
        panic!("type binding must resolve independently");
    };
    assert_eq!(value.kind, "variable_declarator");
    assert_eq!(ty.kind, "interface_declaration");
    assert_ne!(value.start, ty.start);
    assert_eq!(value.source_hash, ty.source_hash);
}

#[test]
fn issue_162_class_identity_inhabits_both_namespaces() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("source.ts");
    std::fs::write(&file, "class Shared {}").unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let value = resolver.local(&file, "Shared", Namespace::Value);
    assert!(matches!(value, Resolution::Resolved(_)));
    assert_eq!(value, resolver.local(&file, "Shared", Namespace::Type));
}
