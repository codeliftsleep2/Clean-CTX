use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;

fn setup(root: &Path) {
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["app.ts","a.ts","b.ts"]}"#).unwrap();
}

#[test]
fn issue_162_global_binding_capture_competing_classes_are_not_merged_or_first_matched() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let file = root.path().join("app.ts");
    std::fs::write(&file, "export class Control {}").unwrap();
    std::fs::write(root.path().join("a.ts"), "class Shared {}").unwrap();
    std::fs::write(root.path().join("b.ts"), "").unwrap();
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&file, "Shared", Namespace::Value),
        Resolution::Resolved(_)
    ));
    std::fs::write(root.path().join("b.ts"), "class Shared {}").unwrap();
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let result = fresh.local(&file, "Shared", Namespace::Value);
    let Resolution::Ambiguous(ids) = result else {
        panic!("competing globals: {result:?}");
    };
    assert_eq!(ids.len(), 2);
    assert!(ids.iter().any(|id| id.source.ends_with("a.ts")));
    assert!(ids.iter().any(|id| id.source.ends_with("b.ts")));
}

#[test]
fn issue_162_global_binding_capture_type_site_retains_canonical_group_before_projection() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    std::fs::write(root.path().join("a.ts"), "interface Shape { a: string; }").unwrap();
    std::fs::write(root.path().join("b.ts"), "interface Shape { b: number; }").unwrap();
    let file = root.path().join("app.ts");
    let source = "export class Control {} let item: Shape;";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let offset = source.find(": Shape").unwrap() + 2;
    let (_, _, result) = capture
        .type_reference_sites
        .iter()
        .find(|site| site.0 == offset)
        .expect("global type occurrence");
    let Resolution::Merged(ids) = result else {
        panic!("retained global group: {result:?}");
    };
    assert_eq!(ids.len(), 2);
    assert!(
        capture
            .programs
            .iter()
            .any(|program| program.groups.contains(ids))
    );
    let restored: TypeScriptAuthority =
        serde_json::from_str(&serde_json::to_string(&capture).unwrap()).unwrap();
    assert!(
        restored
            .type_reference_sites
            .iter()
            .any(|site| site.0 == offset && site.2 == *result)
    );
}
