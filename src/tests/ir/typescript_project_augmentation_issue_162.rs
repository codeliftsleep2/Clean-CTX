//! Project membership, not workspace spelling, owns ambient/global contributors.
use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, file: &str, source: &str) {
    std::fs::write(root.join(file), source).unwrap();
}
fn project(root: &Path, files: &[&str]) {
    write(root, "tsconfig.json", &serde_json::json!({
        "compilerOptions": {"module": "ESNext", "moduleResolution": "bundler", "moduleDetection": "legacy"},
        "files": files
    }).to_string());
}

#[test]
fn issue_162_project_augmentation_script_namespace_merges_only_project_members() {
    let root = tempfile::tempdir().unwrap();
    project(root.path(), &["app.ts", "a.ts", "b.ts", "foreign.ts"]);
    write(root.path(), "app.ts", "class Control {}");
    write(
        root.path(),
        "a.ts",
        "namespace API { export class Leaf {} class Private {} }",
    );
    write(
        root.path(),
        "b.ts",
        "namespace API { export interface Shape {} }",
    );
    write(
        root.path(),
        "foreign.ts",
        "export namespace API { export class Foreign {} }",
    );
    write(
        root.path(),
        "excluded.ts",
        "namespace API { export class Excluded {} }",
    );
    let app = root.path().join("app.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(matches!(
        resolver.local(&app, "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    let Resolution::Resolved(leaf) =
        resolver.namespace_member(&app, "API", "Leaf", Namespace::Value)
    else {
        panic!("project script namespace must supply Leaf");
    };
    assert!(leaf.source.ends_with("a.ts"));
    let Resolution::Resolved(shape) =
        resolver.namespace_member(&app, "API", "Shape", Namespace::Type)
    else {
        panic!("second script part must supply Shape");
    };
    assert!(shape.source.ends_with("b.ts"));
    for name in ["Private", "Foreign", "Excluded"] {
        assert!(
            matches!(
                resolver.namespace_member(&app, "API", name, Namespace::Value),
                Resolution::Unresolved(_)
            ),
            "{name}"
        );
    }
}

#[test]
fn issue_162_project_augmentation_declare_global_requires_admitted_module() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class Control {}");
    write(
        root.path(),
        "globals.ts",
        "export {}; declare global { namespace API { class Leaf {} } }",
    );
    let app = root.path().join("app.ts");
    for admitted in [true, false, true] {
        project(
            root.path(),
            if admitted {
                &["app.ts", "globals.ts"]
            } else {
                &["app.ts"]
            },
        );
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        assert!(matches!(
            resolver.local(&app, "Control", Namespace::Value),
            Resolution::Resolved(_)
        ));
        let result = resolver.namespace_member(&app, "API", "Leaf", Namespace::Value);
        if admitted {
            let Resolution::Resolved(id) = result else {
                panic!("admitted global: {result:?}");
            };
            assert!(id.source.ends_with("globals.ts"));
        } else {
            assert!(matches!(result, Resolution::Unresolved(_)));
        }
    }
}

#[test]
fn issue_162_project_augmentation_module_interface_keeps_all_canonical_contributors() {
    let root = tempfile::tempdir().unwrap();
    project(root.path(), &["app.ts", "leaf.ts", "augment.ts"]);
    write(root.path(), "app.ts", "import { Leaf } from './leaf';");
    write(root.path(), "leaf.ts", "export class Leaf {}");
    write(
        root.path(),
        "augment.ts",
        "import './leaf'; declare module './leaf' { interface Leaf { extra: string; } }",
    );
    let app = root.path().join("app.ts");
    let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
    let value = resolver.local(&app, "Leaf", Namespace::Value);
    let Resolution::Resolved(class) = value else {
        panic!("runtime control: {value:?}");
    };
    let ty = resolver.local(&app, "Leaf", Namespace::Type);
    let Resolution::Merged(ids) = ty else {
        panic!("augmentation contributors: {ty:?}");
    };
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&class));
    assert!(
        ids.iter()
            .any(|id| id.kind == "interface_declaration" && id.source.ends_with("augment.ts"))
    );
}
