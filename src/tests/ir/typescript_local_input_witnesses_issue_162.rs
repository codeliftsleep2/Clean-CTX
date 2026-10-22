use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;
fn write(root: &Path, name: &str, text: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}
fn setup(root: &Path) -> String {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"rootDir":"src","outDir":"dist"},"files":["src/app.ts"]}"#,
    );
    write(
        root,
        "package.json",
        r##"{"imports":{"#leaf":"./dist/leaf.js"}}"##,
    );
    let source = "import { Leaf } from '#leaf'; const item = Leaf;".to_string();
    write(root, "src/app.ts", &source);
    source
}
#[test]
fn issue_162_local_input_witness_consumed_source_mutation_retracts_value_identity() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let source = setup(path);
    write(path, "src/leaf.ts", "export class Leaf {}");
    let before =
        TypeScriptAuthority::capture(&path.join("src/app.ts"), &source, &[path.to_path_buf()]);
    assert!(before.capture_current, "{:?}", before.disposition);
    assert!(
        before.module_requests[0]
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("src/leaf.ts")
    );
    let start = source.rfind("Leaf").unwrap();
    assert!(
        before
            .reference_sites
            .iter()
            .any(|site| site.0 == start && matches!(site.2, Resolution::Resolved(_)))
    );
    write(path, "src/leaf.ts", "export interface Leaf {}");
    assert!(before.witnesses.iter().any(|witness| !witness.current()));
    let mut resolver = Resolver::new(&[path.to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&path.join("src/app.ts"), "Leaf", Namespace::Value),
        Resolution::Unresolved(_)
    ));
    assert!(matches!(
        resolver.local(&path.join("src/app.ts"), "Leaf", Namespace::Type),
        Resolution::Resolved(_)
    ));
}
#[test]
fn issue_162_local_input_witness_input_appearance_invalidates_output_fallback() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let source = setup(path);
    write(path, "dist/leaf.d.ts", "export interface Leaf {}");
    let before =
        TypeScriptAuthority::capture(&path.join("src/app.ts"), &source, &[path.to_path_buf()]);
    assert!(
        before.module_requests[0]
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("dist/leaf.d.ts")
    );
    write(path, "src/leaf.ts", "export class Leaf {}");
    assert!(before.witnesses.iter().any(|witness| !witness.current()));
    let mut resolver = Resolver::new(&[path.to_path_buf()]).unwrap();
    let result = resolver.local(&path.join("src/app.ts"), "Leaf", Namespace::Value);
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("src/leaf.ts")),
        "{result:?}"
    );
}
