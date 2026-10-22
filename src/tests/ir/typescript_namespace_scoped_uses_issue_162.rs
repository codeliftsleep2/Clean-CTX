//! Scoped occurrences must consume namespace authority before file projection.
use super::{DeclarationId, Resolution, TypeScriptAuthority};

fn capture(source: &str) -> TypeScriptAuthority {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("fixture.ts");
    std::fs::write(&file, source).unwrap();
    TypeScriptAuthority::capture(&file, source, &[])
}

fn exact(sites: &[(usize, usize, Resolution)], start: usize, end: usize) -> DeclarationId {
    let (_, _, result) = sites
        .iter()
        .find(|site| site.0 == start && site.1 == end)
        .unwrap_or_else(|| panic!("missing exact occurrence {start}..{end}"));
    let Resolution::Resolved(id) = result else {
        panic!("unproved occurrence: {result:?}");
    };
    id.clone()
}

#[test]
fn issue_162_namespace_scoped_use_dotted_owner_retains_private_leaf() {
    let source = "class Control {} use(Control); namespace API.Core { class Leaf {} use(Leaf); }";
    let capture = capture(source);
    let control = source.find("use(Control)").unwrap() + 4;
    exact(&capture.reference_sites, control, control + 7);
    let offset = source.find("use(Leaf)").unwrap() + 4;
    let id = exact(&capture.reference_sites, offset, offset + 4);
    assert_eq!(&source[id.start..id.end], "class Leaf {}");
    assert!(capture.declarations.contains(&id));
}

#[test]
fn issue_162_namespace_scoped_use_qualified_local_namespace_beats_file_namesake() {
    let source = "namespace Core { export class Leaf {} } namespace API { export namespace Core { export class Leaf {} } use(Core.Leaf); }";
    let capture = capture(source);
    let offset = source.find("use(Core.Leaf)").unwrap() + 4;
    let id = exact(&capture.reference_sites, offset, offset + 9);
    assert_eq!(id.start, source.rfind("class Leaf").unwrap());
    assert_eq!(&source[id.start..id.end], "class Leaf {}");
    assert!(capture.declarations.contains(&id));
}

#[test]
fn issue_162_namespace_scoped_use_runtime_shadow_cannot_borrow_file_namespace() {
    let source = "namespace Core { export class Leaf {} } namespace API { const Core = {}; use(Core.Leaf); }";
    let capture = capture(source);
    let offset = source.find("use(Core.Leaf)").unwrap() + 4;
    let (_, _, result) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == offset && site.1 == offset + 9)
        .expect("retained unresolved member");
    assert!(matches!(result, Resolution::Unresolved(_)));
}

#[test]
fn issue_162_namespace_scoped_use_type_space_selects_private_owner_not_file_type() {
    let source = "interface Shape {} namespace API { interface Shape {} const item: Shape = {}; }";
    let capture = capture(source);
    let offset = source.find(": Shape").unwrap() + 2;
    let id = exact(&capture.type_reference_sites, offset, offset + 5);
    assert_eq!(id.start, source.rfind("interface Shape").unwrap());
    assert_eq!(id.kind, "interface_declaration");
    assert!(capture.declarations.contains(&id));
}
