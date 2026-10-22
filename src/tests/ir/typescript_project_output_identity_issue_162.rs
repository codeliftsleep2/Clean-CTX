use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;
fn write(root: &Path, name: &str, text: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}
fn setup(root: &Path, declaration: Option<&str>) -> String {
    write(
        root,
        "app/tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"disableSourceOfProjectReferenceRedirect":true},"files":["app.ts"],"references":[{"path":"../lib"}]}"#,
    );
    write(
        root,
        "lib/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"rootDir":"src","outDir":"dist","module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["src/index.ts"]}"#,
    );
    write(root, "lib/src/index.ts", "export class Leaf {}");
    if let Some(declaration) = declaration {
        write(root, "lib/dist/index.d.ts", declaration);
    }
    let source = "import { Leaf } from '../lib/src/index'; const item = Leaf;".to_string();
    write(root, "app/app.ts", &source);
    source
}
fn lookup(root: &Path, namespace: Namespace) -> Resolution {
    let mut resolver =
        Resolver::with_project(&[root.to_path_buf()], Some(&root.join("app/tsconfig.json")))
            .unwrap();
    resolver.local(&root.join("app/app.ts"), "Leaf", namespace)
}

#[test]
fn issue_162_project_output_identity_explicit_no_redirect_uses_declaration_bytes() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let declaration = "export interface Leaf {}";
    setup(path, Some(declaration));
    let result = lookup(path, Namespace::Type);
    let Resolution::Resolved(id) = result else {
        panic!("referenced output identity: {result:?}");
    };
    assert_eq!(
        std::fs::canonicalize(&id.source).unwrap(),
        std::fs::canonicalize(path.join("lib/dist/index.d.ts")).unwrap()
    );
    assert_eq!(
        id.source_hash,
        crate::cache::content_hash(declaration.as_bytes())
    );
    assert_eq!(&declaration[id.start..id.end], "interface Leaf {}");
    assert!(matches!(
        lookup(path, Namespace::Value),
        Resolution::Unresolved(_)
    ));
}

#[test]
fn issue_162_project_output_identity_missing_output_refuses_source_substitution_and_restores() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    setup(path, None);
    assert!(
        matches!(lookup(path, Namespace::Value), Resolution::Unresolved(_)),
        "source cannot replace missing declaration output"
    );
    write(path, "lib/dist/index.d.ts", "export declare class Leaf {}");
    let result = lookup(path, Namespace::Value);
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("dist/index.d.ts")),
        "{result:?}"
    );
}

#[test]
fn issue_162_project_output_identity_invalidation_tracks_selected_output_not_source() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let source = setup(path, Some("export declare class Leaf {}"));
    let capture = TypeScriptAuthority::capture_in_project(
        &path.join("app/app.ts"),
        &source,
        &[path.to_path_buf()],
        Some(&path.join("app/tsconfig.json")),
    );
    let start = source.rfind("Leaf").unwrap();
    assert!(capture.reference_sites.iter().any(|site| site.0 == start && matches!(site.2, Resolution::Resolved(ref id) if id.source.ends_with("dist/index.d.ts"))));
    write(path, "lib/src/index.ts", "export interface Leaf {}");
    assert!(
        capture.witnesses.iter().all(|witness| witness.current()),
        "unselected implementation content became authority"
    );
    write(path, "lib/dist/index.d.ts", "export interface Leaf {}");
    assert!(capture.witnesses.iter().any(|witness| !witness.current()));
    assert!(matches!(
        lookup(path, Namespace::Value),
        Resolution::Unresolved(_)
    ));
}
