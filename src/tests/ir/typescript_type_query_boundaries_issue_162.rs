use super::{Resolution, TypeScriptAuthority};

#[test]
fn issue_162_type_query_boundary_runtime_typeof_stays_a_value_occurrence() {
    let root = tempfile::tempdir().unwrap();
    let source = "const Item = 1; type Selected = typeof Item; const result = typeof Item;";
    std::fs::write(root.path().join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    let file = root.path().join("app.ts");
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let runtime = source.rfind("typeof Item").unwrap() + "typeof ".len();
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == runtime && matches!(site.2, Resolution::Resolved(_)))
    );
    let ty = source.find("typeof Item").unwrap();
    assert!(
        capture
            .type_reference_sites
            .iter()
            .any(|site| site.0 == ty && matches!(site.2, Resolution::Resolved(_)))
    );
    assert!(
        !capture
            .reference_sites
            .iter()
            .any(|site| site.0 == ty + "typeof ".len())
    );
}

#[test]
fn issue_162_type_query_boundary_interface_cannot_supply_value_identity() {
    let root = tempfile::tempdir().unwrap();
    let source = "interface Item {} type Selected = typeof Item;";
    std::fs::write(root.path().join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    let file = root.path().join("app.ts");
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let start = source.find("typeof Item").unwrap();
    assert!(
        capture
            .type_reference_sites
            .iter()
            .any(|site| site.0 == start && matches!(site.2, Resolution::Unresolved(_)))
    );
}
