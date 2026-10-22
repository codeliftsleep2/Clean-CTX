use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn setup(root: &Path, first: &str, second: &str) {
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["app.ts","a.ts","b.ts"]}"#).unwrap();
    std::fs::write(root.join("app.ts"), "export class Control {}").unwrap();
    std::fs::write(root.join("a.ts"), first).unwrap();
    std::fs::write(root.join("b.ts"), second).unwrap();
}
fn lookup(root: &Path, name: &str, namespace: Namespace) -> Resolution {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let file = root.join("app.ts");
    assert!(matches!(
        resolver.local(&file, "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    resolver.local(&file, name, namespace)
}
fn group(result: Resolution, count: usize) -> Vec<super::DeclarationId> {
    let Resolution::Merged(ids) = result else {
        panic!("canonical global group: {result:?}");
    };
    assert_eq!(ids.len(), count);
    ids
}

#[test]
fn issue_162_global_merge_group_ambient_overloads_retain_both_sources() {
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        "declare function run(x: string): string;",
        "declare function run(x: number): number;",
    );
    let ids = group(lookup(root.path(), "run", Namespace::Value), 2);
    assert!(ids.iter().all(|id| id.kind == "function_signature"));
    assert!(ids.iter().any(|id| id.source.ends_with("a.ts")));
    assert!(ids.iter().any(|id| id.source.ends_with("b.ts")));
    assert!(matches!(
        lookup(root.path(), "run", Namespace::Type),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_global_merge_group_function_namespace_preserves_runtime_and_member() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "function Service() {}", "");
    assert!(matches!(
        lookup(root.path(), "Service", Namespace::Value),
        Resolution::Resolved(_)
    ));
    std::fs::write(
        root.path().join("b.ts"),
        "namespace Service { export class Child {} }",
    )
    .unwrap();
    let ids = group(lookup(root.path(), "Service", Namespace::Value), 2);
    assert!(ids.iter().any(|id| id.kind == "function_declaration"));
    assert!(ids.iter().any(|id| id.kind == "internal_module"));
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let child = resolver.namespace_member(
        &root.path().join("app.ts"),
        "Service",
        "Child",
        Namespace::Value,
    );
    assert!(
        matches!(child, Resolution::Resolved(ref id) if id.source.ends_with("b.ts")),
        "{child:?}"
    );
}

#[test]
fn issue_162_global_merge_group_explicit_enum_members_retain_both_namespaces() {
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        "enum Mode { One = 1 }",
        "enum Mode { Two = 2 }",
    );
    let values = group(lookup(root.path(), "Mode", Namespace::Value), 2);
    let types = group(lookup(root.path(), "Mode", Namespace::Type), 2);
    assert_eq!(values, types);
    assert!(values.iter().all(|id| id.kind == "enum_declaration"));
}

#[test]
fn issue_162_global_merge_group_competing_function_bodies_stay_ambiguous() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "function run() {}", "");
    assert!(matches!(
        lookup(root.path(), "run", Namespace::Value),
        Resolution::Resolved(_)
    ));
    std::fs::write(root.path().join("b.ts"), "function run() {}").unwrap();
    let result = lookup(root.path(), "run", Namespace::Value);
    let Resolution::Ambiguous(ids) = result else {
        panic!("competing implementations: {result:?}");
    };
    assert_eq!(ids.len(), 2);
}
