//! Program-global authority depends on consumed source format and compiler inputs.
use super::{Namespace, Resolution, Resolver};
use std::path::Path;

fn write(root: &Path, name: &str, source: &str) {
    std::fs::write(root.join(name), source).unwrap();
}
fn lookup(root: &Path, member: &str) -> Resolution {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let app = root.join("app.ts");
    assert!(matches!(
        resolver.local(&app, "Control", Namespace::Value),
        Resolution::Resolved(_)
    ));
    resolver.namespace_member(&app, "API", member, Namespace::Value)
}

#[test]
fn issue_162_program_detection_force_preserves_declaration_globals_only() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class Control {}");
    write(
        root.path(),
        "ambient.d.ts",
        "declare namespace API { class Ambient {} }",
    );
    write(
        root.path(),
        "script.ts",
        "namespace API { export class Runtime {} }",
    );
    for mode in ["legacy", "force", "legacy"] {
        write(root.path(), "tsconfig.json", &serde_json::json!({"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":mode},"files":["app.ts","ambient.d.ts","script.ts"]}).to_string());
        let ambient = lookup(root.path(), "Ambient");
        assert!(
            matches!(ambient, Resolution::Resolved(ref id) if id.source.ends_with("ambient.d.ts")),
            "ambient {mode}: {ambient:?}"
        );
        let runtime = lookup(root.path(), "Runtime");
        if mode == "force" {
            assert!(matches!(runtime, Resolution::Unresolved(_)));
        } else {
            assert!(
                matches!(runtime, Resolution::Resolved(_)),
                "runtime {mode}: {runtime:?}"
            );
        }
    }
}

#[test]
fn issue_162_program_detection_auto_node_package_mutation_changes_global_membership() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class Control {}");
    write(
        root.path(),
        "leaf.ts",
        "namespace API { export class Leaf {} }",
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"NodeNext","moduleResolution":"nodenext","moduleDetection":"auto"},"files":["app.ts","leaf.ts"]}"#,
    );
    for package_type in ["commonjs", "module", "commonjs"] {
        write(
            root.path(),
            "package.json",
            &serde_json::json!({"type":package_type}).to_string(),
        );
        let result = lookup(root.path(), "Leaf");
        if package_type == "module" {
            assert!(matches!(result, Resolution::Unresolved(_)));
        } else {
            assert!(
                matches!(result, Resolution::Resolved(_)),
                "{package_type}: {result:?}"
            );
        }
    }
}

#[test]
fn issue_162_program_detection_auto_jsx_excludes_automatic_runtime_module() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", "export class Control {}");
    write(
        root.path(),
        "leaf.tsx",
        "namespace API { export class Leaf {} } const element = <div />;",
    );
    for mode in ["legacy", "auto", "legacy"] {
        write(root.path(), "tsconfig.json", &serde_json::json!({"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":mode,"jsx":"react-jsx"},"files":["app.ts","leaf.tsx"]}).to_string());
        let result = lookup(root.path(), "Leaf");
        if mode == "auto" {
            assert!(matches!(result, Resolution::Unresolved(_)));
        } else {
            assert!(
                matches!(result, Resolution::Resolved(_)),
                "{mode}: {result:?}"
            );
        }
    }
}

#[test]
fn issue_162_program_detection_side_effect_import_admits_global_augmentation() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "app.ts",
        "import './augment'; export class Control {}",
    );
    write(
        root.path(),
        "augment.ts",
        "export {}; declare global { namespace API { class Leaf {} } }",
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy"},"files":["app.ts"]}"#,
    );
    let result = lookup(root.path(), "Leaf");
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("augment.ts")),
        "{result:?}"
    );
}
