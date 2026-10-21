//! Composition-local source snapshots, including negative resolution witnesses.
use crate::layers::meta::semantic::SemanticDependency;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Default)]
pub(super) struct Contributors {
    sources: BTreeMap<PathBuf, (Option<Arc<str>>, SemanticDependency)>,
}

#[cfg(test)]
#[path = "../../tests/angular_meta/module_metadata_sources_issue_157.rs"]
mod tests;

impl Contributors {
    pub(super) fn read(&mut self, path: &Path) -> Option<Arc<str>> {
        self.sources
            .entry(path.to_path_buf())
            .or_insert_with(|| {
                let bytes = std::fs::read(path).ok();
                // The hash certifies exactly the bytes parsed below, never a
                // second independent read. Keep unreadable/absent witnesses.
                let dependency = SemanticDependency {
                    file_path: path.to_string_lossy().into_owned(),
                    source_hash: bytes.as_deref().map(crate::cache::content_hash),
                };
                let source = bytes
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .map(Arc::<str>::from);
                (source, dependency)
            })
            .0
            .clone()
    }

    pub(super) fn resolve_module(&mut self, path: &Path, module: &str) -> Option<PathBuf> {
        if !(module.starts_with("./") || module.starts_with("../")) {
            return None;
        }
        let target = path.parent()?.join(module);
        // Preserve exact-file precedence. Its absence must remain a witness
        // even when the .ts fallback succeeds, so later shadowing invalidates.
        let _ = self.read(&target);
        if target.is_file() {
            return Some(target);
        }
        let mut typescript_path = target.into_os_string();
        typescript_path.push(".ts");
        let typescript_target = PathBuf::from(typescript_path);
        let _ = self.read(&typescript_target);
        typescript_target.is_file().then_some(typescript_target)
    }

    pub(super) fn dependencies(self) -> Vec<SemanticDependency> {
        self.sources
            .into_values()
            .map(|(_, dependency)| dependency)
            .collect()
    }
}
