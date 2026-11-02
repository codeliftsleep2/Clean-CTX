//! Shared neutral selection fixtures through the second explicit demand origin.
pub(super) use super::typescript_lazy_selection_support::{call, capture, proofs};
use std::path::Path;
pub(super) fn setup(form: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = super::typescript_lazy_selection_support::setup(form);
    for file in ["one.ts", "two.ts"] {
        let p = root.path().join(file);
        let source = std::fs::read_to_string(&p)
            .unwrap()
            .replace("loadComponent:", "loadChildren:");
        std::fs::write(p, source).unwrap();
    }
    (root, config)
}
pub(super) fn rewrite(p: &Path, callback: &str) {
    super::typescript_lazy_selection_support::rewrite(p, callback);
    let file = p.join("one.ts");
    let source = std::fs::read_to_string(&file)
        .unwrap()
        .replace("loadComponent:", "loadChildren:");
    std::fs::write(file, source).unwrap();
}
