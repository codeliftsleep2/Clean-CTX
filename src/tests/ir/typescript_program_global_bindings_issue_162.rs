use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    std::fs::write(root.join(name), source).unwrap();
}
fn setup(root: &Path, files: &[&str]) {
    write(root, "tsconfig.json", &serde_json::json!({"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","types":[],"noLib":true},"files":files}).to_string());
}

#[test]
fn issue_162_program_global_binding_value_uses_project_source_not_external_namesake() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), &["app.ts", "global.ts", "foreign.ts"]);
    let source = "export class Control {} use(Control); use(Shared);";
    write(root.path(), "app.ts", source);
    write(root.path(), "global.ts", "class Shared {}");
    write(root.path(), "foreign.ts", "export class Shared {}");
    write(root.path(), "excluded.ts", "class Shared {}");
    let file = root.path().join("app.ts");
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let control = source.find("use(Control)").unwrap() + 4;
    assert!(
        capture.reference_sites.iter().any(
            |(start, _, result)| *start == control && matches!(result, Resolution::Resolved(_))
        )
    );
    let offset = source.find("use(Shared)").unwrap() + 4;
    let (_, _, result) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == offset)
        .expect("global occurrence");
    let Resolution::Resolved(id) = result else {
        panic!("global identity: {result:?}");
    };
    assert!(id.source.ends_with("global.ts"));
    assert_eq!(
        id.source_hash,
        crate::cache::content_hash(b"class Shared {}")
    );
}

#[test]
fn issue_162_program_global_binding_interface_merge_retains_all_type_contributors() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), &["app.ts", "a.ts", "b.ts"]);
    write(root.path(), "app.ts", "export class Control {}");
    write(root.path(), "a.ts", "interface Shape { a: string; }");
    write(root.path(), "b.ts", "interface Shape { b: number; }");
    let file = root.path().join("app.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&file, "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    let ty = resolver.local(&file, "Shape", Namespace::Type);
    let Resolution::Merged(ids) = ty else {
        panic!("global type contributors: {ty:?}");
    };
    assert_eq!(ids.len(), 2);
    assert!(ids.iter().any(|id| id.source.ends_with("a.ts")));
    assert!(ids.iter().any(|id| id.source.ends_with("b.ts")));
    assert!(matches!(
        resolver.local(&file, "Shape", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_program_global_binding_lexical_parameter_shadows_global_without_leakage() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), &["app.ts", "global.ts"]);
    let source =
        "export class Control {} function f(Shared: unknown) { use(Shared); } use(Shared);";
    write(root.path(), "app.ts", source);
    write(root.path(), "global.ts", "class Shared {}");
    let file = root.path().join("app.ts");
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let sites: Vec<_> = source
        .match_indices("use(Shared)")
        .map(|(at, _)| at + 4)
        .collect();
    let (_, _, Resolution::Resolved(parameter)) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == sites[0])
        .unwrap()
    else {
        panic!("parameter control");
    };
    assert!(parameter.source.ends_with("app.ts"));
    let (_, _, result) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == sites[1])
        .unwrap();
    let Resolution::Resolved(global) = result else {
        panic!("global: {result:?}");
    };
    assert!(global.source.ends_with("global.ts"));
    assert_ne!(global, parameter);
}

#[test]
fn issue_162_program_global_binding_kind_mutation_retracts_value_and_invalidates_witness() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), &["app.ts", "global.ts"]);
    write(root.path(), "app.ts", "export class Control {}");
    write(root.path(), "global.ts", "class Shared {}");
    let file = root.path().join("app.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&file, "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    let initial = resolver.local(&file, "Shared", Namespace::Value);
    assert!(matches!(initial, Resolution::Resolved(_)), "{initial:?}");
    assert!(resolver.host.current());
    write(root.path(), "global.ts", "interface Shared {}");
    assert!(!resolver.host.current());
    let mut fresh = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        fresh.local(&file, "Shared", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    let ty = fresh.local(&file, "Shared", Namespace::Type);
    assert!(
        matches!(ty, Resolution::Resolved(ref id) if id.kind == "interface_declaration"),
        "{ty:?}"
    );
}
