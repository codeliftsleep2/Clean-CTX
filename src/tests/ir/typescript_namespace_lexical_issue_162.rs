//! Namespace lexical uses must select declaration-owned scopes, not file names.
use super::{DeclarationId, Resolution, TypeScriptAuthority};

fn capture(source: &str) -> TypeScriptAuthority {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    std::fs::write(&file, source).unwrap();
    TypeScriptAuthority::capture(&file, source, &[])
}

fn at(capture: &TypeScriptAuthority, offset: usize) -> DeclarationId {
    let (_, _, result) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == offset)
        .unwrap_or_else(|| panic!("missing reference at {offset}"));
    let Resolution::Resolved(id) = result else {
        panic!("unproved use at {offset}: {result:?}");
    };
    assert!(capture.declarations.contains(id));
    id.clone()
}

#[test]
fn issue_162_namespace_lexical_private_member_has_exact_declaration_identity() {
    let source = "class Control {} use(Control); namespace API { class Private {} use(Private); } use(Private);";
    let capture = capture(source);
    assert_eq!(
        at(&capture, source.find("use(Control)").unwrap() + 4).name,
        "Control"
    );
    let id = at(&capture, source.find("use(Private)").unwrap() + 4);
    assert_eq!(&source[id.start..id.end], "class Private {}");
    assert_eq!(
        id.source_hash,
        crate::cache::content_hash(source.as_bytes())
    );
    let outside = source.rfind("use(Private)").unwrap() + 4;
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == outside && matches!(site.2, Resolution::Unresolved(_)))
    );
}

#[test]
fn issue_162_namespace_lexical_nested_owner_shadows_outer_same_name() {
    let source = "class Leaf {} namespace API { class Leaf {} namespace Core { class Leaf {} use(Leaf); } use(Leaf); } use(Leaf);";
    let capture = capture(source);
    let offsets: Vec<_> = source
        .match_indices("use(Leaf)")
        .map(|(at, _)| at + 4)
        .collect();
    let inner = at(&capture, offsets[0]);
    let parent = at(&capture, offsets[1]);
    let outer = at(&capture, offsets[2]);
    assert_ne!(inner, parent);
    assert_ne!(parent, outer);
    assert_ne!(inner, outer);
    assert_eq!(inner.start, source.rfind("class Leaf").unwrap());
    assert_eq!(
        parent.start,
        source.match_indices("class Leaf").nth(1).unwrap().0
    );
    assert_eq!(outer.start, 0);
}

#[test]
fn issue_162_namespace_lexical_merged_parts_share_exports_but_not_private_locals() {
    let source = "namespace API { export class Shared {} class Private {} use(Private); } namespace API { use(Shared); use(Private); }";
    let capture = capture(source);
    let private = at(&capture, source.find("use(Private)").unwrap() + 4);
    assert_eq!(&source[private.start..private.end], "class Private {}");
    let shared = at(&capture, source.find("use(Shared)").unwrap() + 4);
    assert_eq!(&source[shared.start..shared.end], "class Shared {}");
    let other_part = source.rfind("use(Private)").unwrap() + 4;
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == other_part && matches!(site.2, Resolution::Unresolved(_)))
    );
}
