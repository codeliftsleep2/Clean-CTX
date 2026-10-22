use super::module_resolution::ModuleRequestKind;
use super::{Namespace, Resolution, Resolver, TypeScriptAuthority};
use std::path::Path;

fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"NodeNext","moduleResolution":"nodenext","moduleDetection":"legacy","noLib":true,"types":[]}}"#,
    );
    write(
        root,
        "node_modules/pkg/package.json",
        r#"{"exports":{"import":"./esm.d.mts","require":"./cjs.d.cts"}}"#,
    );
    write(root, "node_modules/pkg/esm.d.mts", "export class Leaf {}");
    write(root, "node_modules/pkg/cjs.d.cts", "export class Leaf {}");
}
fn source(mode: &str) -> String {
    format!(
        "import type {{ Leaf }} from 'pkg' with {{ \"resolution-mode\": \"{mode}\" }}; let item: Leaf;"
    )
}
fn expected(root: &Path, name: &str) -> super::DeclarationId {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let result = resolver.local(
        &root.join(format!("node_modules/pkg/{name}")),
        "Leaf",
        Namespace::Type,
    );
    let Resolution::Resolved(id) = result else {
        panic!("direct control: {result:?}");
    };
    assert!(id.end > id.start);
    id
}

#[test]
fn issue_162_import_type_mode_explicit_conditions_override_containing_format() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    for (file, mode, target) in [
        ("app.cts", "import", "esm.d.mts"),
        ("app.mts", "require", "cjs.d.cts"),
    ] {
        let text = source(mode);
        write(root.path(), file, &text);
        let control = expected(root.path(), target);
        let mut resolver = Resolver::new(&[root.path().to_path_buf()]).unwrap();
        assert_eq!(
            resolver.local(&root.path().join(file), "Leaf", Namespace::Type),
            Resolution::Resolved(control)
        );
        assert!(matches!(
            resolver.local(&root.path().join(file), "Leaf", Namespace::Value),
            Resolution::Unresolved(_)
        ));
    }
}

#[test]
fn issue_162_import_type_mode_capture_retains_selected_request_and_endpoint() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let text = source("require");
    write(root.path(), "app.mts", &text);
    let control = expected(root.path(), "cjs.d.cts");
    let capture = TypeScriptAuthority::capture(
        &root.path().join("app.mts"),
        &text,
        &[root.path().to_path_buf()],
    );
    assert_eq!(capture.module_requests.len(), 1);
    let request = &capture.module_requests[0];
    assert_eq!(request.kind, ModuleRequestKind::Require);
    assert!(request.resolved.as_ref().unwrap().ends_with("cjs.d.cts"));
    assert!(
        capture
            .type_reference_sites
            .iter()
            .any(|site| site.2 == Resolution::Resolved(control.clone()))
    );
    let json = serde_json::to_value(&capture).unwrap();
    let restored: TypeScriptAuthority = serde_json::from_value(json).unwrap();
    assert_eq!(restored.module_requests[0].kind, ModuleRequestKind::Require);
}

#[test]
fn issue_162_import_type_mode_attribute_mutation_retracts_prior_endpoint() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path());
    let file = root.path().join("app.cts");
    let text = source("import");
    write(root.path(), "app.cts", &text);
    let before = TypeScriptAuthority::capture(&file, &text, &[root.path().to_path_buf()]);
    assert!(before.capture_current);
    assert!(
        before.module_requests[0]
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("esm.d.mts")
    );
    let changed = source("require");
    write(root.path(), "app.cts", &changed);
    // Supplied target bytes are certified separately from disk dependencies.
    assert_ne!(
        before.source_hash,
        crate::cache::content_hash(&std::fs::read(&file).unwrap())
    );
    let after = TypeScriptAuthority::capture(&file, &changed, &[root.path().to_path_buf()]);
    assert!(
        after.module_requests[0]
            .resolved
            .as_ref()
            .unwrap()
            .ends_with("cjs.d.cts")
    );
    assert_ne!(before.source_hash, after.source_hash);
    assert_eq!(
        after.source_hash,
        crate::cache::content_hash(changed.as_bytes())
    );
    assert!(before.witnesses.iter().all(|witness| witness.current()));
    write(
        root.path(),
        "node_modules/pkg/esm.d.mts",
        "export interface Leaf {}",
    );
    assert!(before.witnesses.iter().any(|witness| !witness.current()));
}
