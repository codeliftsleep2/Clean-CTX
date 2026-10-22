use super::{Resolution, TypeScriptAuthority};

#[test]
fn issue_162_import_type_boundary_private_namespace_cannot_supply_member() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    std::fs::write(path.join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    std::fs::write(path.join("leaf.ts"), "namespace Private { export interface Shape {} } export namespace Public { export interface Shape {} }").unwrap();
    let public = "import('./leaf').Public.Shape";
    let private = "import('./leaf').Private.Shape";
    let text = format!("type A = {public}; type B = {private};");
    std::fs::write(path.join("app.ts"), &text).unwrap();
    let capture = TypeScriptAuthority::capture(&path.join("app.ts"), &text, &[path.to_path_buf()]);
    let start = text.find(public).unwrap();
    assert!(
        capture
            .type_reference_sites
            .iter()
            .any(|site| site.0 == start && matches!(site.2, Resolution::Resolved(_)))
    );
    let start = text.find(private).unwrap();
    assert!(
        capture
            .type_reference_sites
            .iter()
            .any(|site| site.0 == start && matches!(site.2, Resolution::Unresolved(_)))
    );
}

#[test]
fn issue_162_import_type_boundary_invalid_mode_cannot_certify_request() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    std::fs::write(path.join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    std::fs::write(path.join("leaf.ts"), "export interface Leaf {}").unwrap();
    let text = r#"type A = import('./leaf', { with: { "resolution-mode": "invalid" } }).Leaf;"#;
    std::fs::write(path.join("app.ts"), text).unwrap();
    let capture = TypeScriptAuthority::capture(&path.join("app.ts"), text, &[path.to_path_buf()]);
    assert_eq!(capture.module_requests.len(), 1);
    assert!(capture.module_requests[0].resolved.is_err());
    assert!(
        capture
            .type_reference_sites
            .iter()
            .any(|site| matches!(site.2, Resolution::Unresolved(_)))
    );
}
