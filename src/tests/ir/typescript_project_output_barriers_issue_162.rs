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
        "app/tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"disableSourceOfProjectReferenceRedirect":true},"files":["app.ts"],"references":[{"path":"../lib"}]}"#,
    );
    write(
        root,
        "lib/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"rootDir":"src","outDir":"dist","module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["src/index.ts"]}"#,
    );
    write(root, "lib/src/index.ts", "export class Leaf {}");
    let source = "import { Leaf } from '../lib/src/index'; const item = Leaf;".to_string();
    write(root, "app/app.ts", &source);
    source
}
fn lookup(root: &Path) -> Resolution {
    let mut resolver =
        Resolver::with_project(&[root.to_path_buf()], Some(&root.join("app/tsconfig.json")))
            .unwrap();
    resolver.local(&root.join("app/app.ts"), "Leaf", Namespace::Value)
}
#[test]
fn issue_162_project_output_barrier_missing_selected_output_cannot_fall_through_to_directory() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    setup(path);
    write(path, "lib/dist/index.d.ts", "export declare class Leaf {}");
    assert!(
        matches!(lookup(path), Resolution::Resolved(ref id) if id.source.ends_with("dist/index.d.ts"))
    );
    std::fs::remove_file(path.join("lib/dist/index.d.ts")).unwrap();
    write(path, "lib/src/index/index.ts", "export class Leaf {}");
    let result = lookup(path);
    assert!(
        matches!(result, Resolution::Unresolved(_)),
        "directory namesake bypassed selected output: {result:?}"
    );
}
#[test]
fn issue_162_project_output_barrier_inherited_output_paths_keep_declaring_origin() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    setup(path);
    write(
        path,
        "config/base.json",
        r#"{"compilerOptions":{"composite":true,"rootDir":"../lib/src","declarationDir":"../lib/generated","module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#,
    );
    write(
        path,
        "lib/tsconfig.json",
        r#"{"extends":"../config/base.json","files":["src/index.ts"]}"#,
    );
    write(
        path,
        "lib/generated/index.d.ts",
        "export declare class Leaf {}",
    );
    let result = lookup(path);
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("generated/index.d.ts")),
        "{result:?}"
    );
}
#[test]
fn issue_162_project_output_barrier_output_appearance_invalidates_negative_witness() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let source = setup(path);
    assert!(matches!(lookup(path), Resolution::Unresolved(_)));
    let before = TypeScriptAuthority::capture_in_project(
        &path.join("app/app.ts"),
        &source,
        &[path.to_path_buf()],
        Some(&path.join("app/tsconfig.json")),
    );
    write(path, "lib/dist/index.d.ts", "export declare class Leaf {}");
    assert!(before.witnesses.iter().any(|witness| !witness.current()));
    assert!(
        matches!(lookup(path), Resolution::Resolved(ref id) if id.source.ends_with("dist/index.d.ts"))
    );
}
