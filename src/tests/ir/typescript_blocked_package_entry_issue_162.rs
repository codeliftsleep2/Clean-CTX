use super::super::module_resolution::{ModuleRequestKind, ModuleTarget};
use crate::ir::typescript_resolution::Resolver;

#[test]
fn issue_162_explicit_blocked_package_entry_is_absent_but_unproved_maps_remain_errors() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    std::fs::create_dir_all(p.join("node_modules/pkg")).unwrap();
    std::fs::write(p.join("tsconfig.json"),r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[]}}"#).unwrap();
    std::fs::write(
        p.join("node_modules/pkg/index.d.ts"),
        "export interface RootApi {}",
    )
    .unwrap();
    let file = p.join("app.ts");
    std::fs::write(&file, "export const unrelated=0;").unwrap();
    let manifest = p.join("node_modules/pkg/package.json");
    std::fs::write(&manifest,r#"{"name":"pkg","version":"1.0.0","exports":{".":{"types":"./index.d.ts"},"./blocked":null}}"#).unwrap();
    let mut resolver = Resolver::new(&[p.to_path_buf()]).unwrap();
    assert!(
        matches!(
            resolver.module_request_target(&file, "pkg", ModuleRequestKind::StaticImport),
            Ok(ModuleTarget::Found(_))
        ),
        "nonempty root export control"
    );
    assert!(
        matches!(
            resolver.module_request_target(&file, "pkg/blocked", ModuleRequestKind::StaticImport),
            Ok(ModuleTarget::Absent(_))
        ),
        "exact null entry has witnessed absence"
    );
    assert!(
        resolver
            .module_request_target(&file, "pkg/unknown", ModuleRequestKind::StaticImport)
            .is_err(),
        "unproved entry forms retain their error contract"
    );
    for text in [
        r#"{"name":"pkg","exports":{"./blocked":null,"default":"./index.d.ts"}}"#,
        r#"{"name":"pkg","exports":{"./blocked":123}}"#,
        r#"{"name":"pkg","name":"other","exports":{"./blocked":null}}"#,
    ] {
        std::fs::write(&manifest, text).unwrap();
        let mut resolver = Resolver::new(&[p.to_path_buf()]).unwrap();
        assert!(
            resolver
                .module_request_target(&file, "pkg/blocked", ModuleRequestKind::StaticImport)
                .is_err(),
            "invalid/unproved package map must not become proved absence: {text}"
        );
    }
}
