//! Scoped input, authority, currentness and failure falsifiers; no timing budget.
use crate::ir::typescript_resolution::Resolver;
use crate::ir::typescript_resolution::shared_acquisition::Operation;
use std::{path::Path, sync::Arc};

pub(super) fn write(root: &Path, name: &str, source: impl AsRef<[u8]>) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}
pub(super) fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"include":["*.ts"]}"#,
    );
    write(root.path(), "a.ts", "class A { peer?: Z; }");
    write(root.path(), "b.ts", "class Z {}");
    root
}
pub(super) fn resolver(root: &Path, owner: &str, op: &Arc<Operation>) -> Resolver {
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    resolver.operation = Some(Arc::clone(op));
    resolver.acquisition_policy = "controls".into();
    resolver.host.witnesses.enable();
    let path = root.join(owner);
    resolver
        .host
        .seed(&path, &std::fs::read_to_string(&path).unwrap())
        .unwrap();
    resolver
}
pub(super) fn acquire(resolver: &mut Resolver, path: &Path) {
    assert_eq!(resolver.program_units(path).unwrap().len(), 2);
}
