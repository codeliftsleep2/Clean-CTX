use super::{Namespace, Resolution, Resolver};
use std::path::Path;
fn write(root: &Path, name: &str, text: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}
fn setup(root: &Path, disabled: bool) {
    write(
        root,
        "app/tsconfig.json",
        &format!(
            r#"{{"compilerOptions":{{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"disableSourceOfProjectReferenceRedirect":{disabled}}},"files":["app.ts"],"references":[{{"path":"../lib"}}]}}"#
        ),
    );
    write(
        root,
        "app/app.ts",
        "import { Leaf } from '../lib/src/index';",
    );
    write(
        root,
        "lib/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"rootDir":"src","outDir":"dist","declarationDir":"types","module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["src/index.ts"]}"#,
    );
    write(root, "lib/src/index.ts", "export class Leaf {}");
    write(root, "lib/dist/index.d.ts", "export declare class Leaf {}");
    write(root, "lib/types/index.d.ts", "export interface Leaf {}");
}
fn lookup(root: &Path, namespace: Namespace) -> Resolution {
    let mut resolver =
        Resolver::with_project(&[root.to_path_buf()], Some(&root.join("app/tsconfig.json")))
            .unwrap();
    resolver.local(&root.join("app/app.ts"), "Leaf", namespace)
}
#[test]
fn issue_162_project_output_policy_declaration_dir_precedes_out_dir() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), true);
    let ty = lookup(root.path(), Namespace::Type);
    assert!(
        matches!(ty, Resolution::Resolved(ref id) if id.source.ends_with("types/index.d.ts")),
        "{ty:?}"
    );
    assert!(matches!(
        lookup(root.path(), Namespace::Value),
        Resolution::Unresolved(_)
    ));
}
#[test]
fn issue_162_project_output_policy_explicit_false_preserves_source_route() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), false);
    let value = lookup(root.path(), Namespace::Value);
    assert!(
        matches!(value, Resolution::Resolved(ref id) if id.source.ends_with("src/index.ts")),
        "{value:?}"
    );
}
