use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};

fn setup(path: &std::path::Path, declaration: &str) -> String {
    std::fs::write(path.join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["app.ts","ambient.d.ts"]}"#).unwrap();
    std::fs::write(path.join("ambient.d.ts"), declaration).unwrap();
    let source = "import { Leaf } from 'virtual-api'; const item = Leaf;".to_string();
    std::fs::write(path.join("app.ts"), &source).unwrap();
    source
}

#[test]
fn issue_162_ambient_scope_evidence_retains_declaration_target_without_fake_file() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let declaration = r#"declare module "virtual-api" { export class Leaf {} }"#;
    let source = setup(path, declaration);
    let capture =
        TypeScriptAuthority::capture(&path.join("app.ts"), &source, &[path.to_path_buf()]);
    assert_eq!(capture.module_requests.len(), 1);
    let request = &capture.module_requests[0];
    assert!(request.resolved.is_err());
    assert_eq!(request.ambient_declarations.len(), 1);
    let id = &request.ambient_declarations[0];
    assert_eq!(id.name, "virtual-api");
    assert_eq!(id.kind, "ambient_module");
    assert_eq!(
        id.source_hash,
        crate::cache::content_hash(declaration.as_bytes())
    );
    assert!(id.source.ends_with("ambient.d.ts"));
    assert_eq!(
        &declaration[id.start..id.end],
        r#"module "virtual-api" { export class Leaf {} }"#
    );
    let restored: TypeScriptAuthority =
        serde_json::from_value(serde_json::to_value(&capture).unwrap()).unwrap();
    assert_eq!(
        restored.module_requests[0].ambient_declarations,
        request.ambient_declarations
    );
}

#[test]
fn issue_162_ambient_scope_evidence_self_export_cycle_refuses_after_positive() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    setup(
        path,
        r#"declare module "virtual-api" { export class Leaf {} }"#,
    );
    let file = path.join("app.ts");
    let mut resolver = Resolver::new(&[path.to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&file, "Leaf", Namespace::Value),
        Resolution::Resolved(_)
    ));
    std::fs::write(
        path.join("ambient.d.ts"),
        r#"declare module "virtual-api" { export { Leaf } from "virtual-api"; }"#,
    )
    .unwrap();
    let mut resolver = Resolver::new(&[path.to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&file, "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
}
