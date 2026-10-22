use super::{Resolution, TypeScriptAuthority};

fn capture(source: &str) -> TypeScriptAuthority {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    std::fs::write(&file, source).unwrap();
    TypeScriptAuthority::capture(&file, source, &[])
}
fn at<'a>(capture: &'a TypeScriptAuthority, source: &str, needle: &str) -> &'a Resolution {
    let start = source.rfind(needle).unwrap();
    &capture
        .type_reference_sites
        .iter()
        .find(|(a, b, _)| *a == start && *b == start + needle.len())
        .expect("exact type occurrence")
        .2
}

#[test]
fn issue_162_generic_type_parameter_shadows_outer_type_and_retains_its_span() {
    let source = "interface Item {} function consume<Item>(value: Item) {}";
    let capture = capture(source);
    let Resolution::Resolved(id) = at(&capture, source, "Item") else {
        panic!("generic parameter identity");
    };
    assert_eq!(id.kind, "type_parameter");
    assert_eq!(&source[id.start..id.end], "Item");
    assert!(capture.declarations.contains(id));
    assert!(
        !capture
            .type_reference_sites
            .iter()
            .any(|(start, _, _)| *start == source.find("Item>").unwrap())
    );
}

#[test]
fn issue_162_block_local_types_do_not_replace_outer_type_outside_their_scope() {
    let source = "interface Item {} function f() { type Item = string; let inside: Item; } let outside: Item;";
    let capture = capture(source);
    let Resolution::Resolved(outer) = at(&capture, source, "Item") else {
        panic!("outer identity");
    };
    assert_eq!(outer.kind, "interface_declaration");
    let inner = source.find("inside: Item").unwrap() + "inside: ".len();
    let Resolution::Resolved(id) = &capture
        .type_reference_sites
        .iter()
        .find(|site| site.0 == inner)
        .unwrap()
        .2
    else {
        panic!("inner type");
    };
    assert_eq!(id.kind, "type_alias_declaration");
    assert_ne!(id.start, outer.start);
}

#[test]
fn issue_162_type_only_import_occurrences_are_canonical_and_dependency_current() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    std::fs::write(root.path().join("leaf.ts"), "export interface Shape {}").unwrap();
    let file = root.path().join("app.ts");
    let source = "import type { Shape as Selected } from './leaf'; let value: Selected;";
    std::fs::write(&file, source).unwrap();
    let before = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let Resolution::Resolved(id) = at(&before, source, "Selected") else {
        panic!("import type identity");
    };
    assert!(id.source.ends_with("leaf.ts"));
    assert_eq!(id.kind, "interface_declaration");
    std::fs::write(
        root.path().join("leaf.ts"),
        "export interface Replacement {}",
    )
    .unwrap();
    assert!(before.witnesses.iter().any(|w| !w.current()));
    let after = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert!(matches!(
        at(&after, source, "Selected"),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_competing_generic_parameters_do_not_choose_first_identity() {
    let source = "function consume<Item, Item>(value: Item) {}";
    let capture = capture(source);
    assert!(matches!(at(&capture, source, "Item"), Resolution::Ambiguous(ids) if ids.len()==2));
}

#[test]
fn issue_162_type_occurrence_capture_survives_serialization_without_value_promotion() {
    let source = "interface Item {} let value: Item;";
    let capture = capture(source);
    let expected = at(&capture, source, "Item").clone();
    assert!(matches!(&expected, Resolution::Resolved(_)));
    let restored: TypeScriptAuthority =
        serde_json::from_str(&serde_json::to_string(&capture).unwrap()).unwrap();
    assert_eq!(*at(&restored, source, "Item"), expected);
    let start = source.rfind("Item").unwrap();
    assert!(!restored.reference_sites.iter().any(|site| site.0 == start));
}

#[test]
fn issue_162_qualified_type_occurrences_follow_namespace_identity_before_leaf_selection() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    std::fs::write(root.path().join("leaf.ts"), "export interface Shape {}").unwrap();
    std::fs::write(
        root.path().join("middle.ts"),
        "export type * as Core from './leaf';",
    )
    .unwrap();
    let file = root.path().join("app.ts");
    let source = "import * as API from './middle'; let value: API.Core.Shape;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let Resolution::Resolved(id) = at(&capture, source, "API.Core.Shape") else {
        panic!("qualified type identity");
    };
    assert_eq!(id.name, "Shape");
    assert!(id.source.ends_with("leaf.ts"));
    let terminal = source.rfind("Shape").unwrap();
    assert!(
        !capture
            .type_reference_sites
            .iter()
            .any(|site| site.0 == terminal)
    );
}

#[test]
fn issue_162_generic_type_shadow_cannot_borrow_an_imported_namespace() {
    let source = "function consume<API>(value: API.Shape) {}";
    let capture = capture(source);
    assert!(matches!(
        at(&capture, source, "API.Shape"),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_mapped_key_identity_is_scoped_to_mapping_and_value_not_constraint() {
    let source = "interface Key {} type Map = { [Key in keyof Key]: Key }; let outer: Key;";
    let capture = capture(source);
    let constraint = source.find("keyof Key").unwrap() + "keyof ".len();
    let value = source.find("]: Key").unwrap() + "]: ".len();
    let Resolution::Resolved(outer) = at(&capture, source, "Key") else {
        panic!("outer Key");
    };
    assert_eq!(outer.kind, "interface_declaration");
    let constraint_id = &capture
        .type_reference_sites
        .iter()
        .find(|site| site.0 == constraint)
        .unwrap()
        .2;
    assert_eq!(*constraint_id, Resolution::Resolved(outer.clone()));
    let Resolution::Resolved(mapped) = &capture
        .type_reference_sites
        .iter()
        .find(|site| site.0 == value)
        .unwrap()
        .2
    else {
        panic!("mapped Key");
    };
    assert_eq!(mapped.kind, "mapped_type_clause");
    assert_ne!(mapped.start, outer.start);
}

#[test]
fn issue_162_conditional_infer_identity_does_not_leak_to_alternative_branch() {
    let source = "interface Item {} type Result<T> = T extends Array<infer Item> ? Item : Item;";
    let capture = capture(source);
    let positive = source.find("? Item").unwrap() + "? ".len();
    let Resolution::Resolved(inferred) = &capture
        .type_reference_sites
        .iter()
        .find(|site| site.0 == positive)
        .unwrap()
        .2
    else {
        panic!("infer identity");
    };
    assert_eq!(inferred.kind, "infer_type");
    assert_eq!(&source[inferred.start..inferred.end], "infer Item");
    let Resolution::Resolved(outer) = at(&capture, source, "Item") else {
        panic!("alternative outer identity");
    };
    assert_eq!(outer.kind, "interface_declaration");
    assert_ne!(inferred.start, outer.start);
    let declaration = source.find("infer Item").unwrap() + "infer ".len();
    assert!(
        !capture
            .type_reference_sites
            .iter()
            .any(|site| site.0 == declaration)
    );
}
