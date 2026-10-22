use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};

fn setup(root: &std::path::Path, first: &str, second: &str) -> String {
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["app.ts","a.d.ts","b.d.ts"]}"#).unwrap();
    std::fs::write(root.join("a.d.ts"), first).unwrap();
    std::fs::write(root.join("b.d.ts"), second).unwrap();
    let source = "import type { Leaf } from 'virtual-api'; let item: Leaf;".to_string();
    std::fs::write(root.join("app.ts"), &source).unwrap();
    source
}

#[test]
fn issue_162_ambient_merge_evidence_type_occurrence_and_program_group_survive_serde() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let source = setup(
        path,
        r#"declare module "virtual-api" { export interface Leaf { a: string } }"#,
        r#"declare module "virtual-api" { export interface Leaf { b: number } }"#,
    );
    let capture =
        TypeScriptAuthority::capture(&path.join("app.ts"), &source, &[path.to_path_buf()]);
    let start = source.rfind("Leaf").unwrap();
    let (_, _, Resolution::Merged(ids)) = capture
        .type_reference_sites
        .iter()
        .find(|site| site.0 == start)
        .unwrap()
    else {
        panic!(
            "missing merged type evidence: {:?}",
            capture.type_reference_sites
        );
    };
    assert_eq!(ids.len(), 2);
    assert!(
        capture
            .programs
            .iter()
            .any(|program| program.groups.contains(ids))
    );
    let restored: TypeScriptAuthority =
        serde_json::from_value(serde_json::to_value(&capture).unwrap()).unwrap();
    assert_eq!(restored.type_reference_sites, capture.type_reference_sites);
    assert_eq!(restored.module_requests[0].ambient_declarations.len(), 2);
}

#[test]
fn issue_162_ambient_merge_evidence_competing_class_bodies_remain_ambiguous() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    setup(
        path,
        r#"declare module "virtual-api" { export class Leaf {} }"#,
        "",
    );
    let file = path.join("app.ts");
    let mut resolver = Resolver::new(&[path.to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&file, "Leaf", Namespace::Type),
        Resolution::Resolved(_)
    ));
    std::fs::write(
        path.join("b.d.ts"),
        r#"declare module "virtual-api" { export class Leaf {} }"#,
    )
    .unwrap();
    let mut resolver = Resolver::new(&[path.to_path_buf()]).unwrap();
    let result = resolver.local(&file, "Leaf", Namespace::Type);
    assert!(
        matches!(result, Resolution::Ambiguous(ref ids) if ids.len() == 2),
        "{result:?}"
    );
}
