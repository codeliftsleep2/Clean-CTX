use crate::mcp::McpState;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(super) struct FilesystemScan {
    pub(super) candidates: Vec<String>,
    pub(super) attempted: bool,
    pub(super) completed: bool,
}

pub(super) fn configured_roots(state: &McpState, workspace_root: Option<&str>) -> Vec<PathBuf> {
    let primary = workspace_root
        .map(PathBuf::from)
        .unwrap_or_else(default_root);
    let mut roots = vec![primary];
    roots.extend(state.config.additional_roots.iter().map(PathBuf::from));
    deduplicate_roots(&mut roots);
    roots
}

#[cfg(not(test))]
fn default_root() -> PathBuf {
    crate::mcp::server::find_project_root().clone()
}

#[cfg(test)]
fn default_root() -> PathBuf {
    // Unit tests must opt into traversal with an explicit temporary root. This
    // prevents unrelated handler tests from scanning the developer checkout.
    PathBuf::from("__clean_ctx_test_requires_explicit_workspace_root__")
}

pub(super) fn deduplicate_roots(roots: &mut Vec<PathBuf>) {
    let mut seen = HashSet::new();
    roots.retain(|root| seen.insert(root_key(root)));
    roots.sort_by_key(|root| root_key(root));
}

pub(super) fn root_key(root: &Path) -> String {
    crate::dictionary::path::canonical_identity_key(&root.to_string_lossy())
}

pub(super) fn scan(state: &McpState, roots: &[PathBuf], query_name: &str) -> FilesystemScan {
    let mut candidates = Vec::new();
    let mut attempted = false;
    let mut completed = true;

    for configured_root in roots {
        let root = match configured_root.canonicalize() {
            Ok(root) if root.is_dir() => root,
            _ => {
                completed = false;
                continue;
            }
        };
        attempted = true;
        let walker = WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !state.config.is_excluded(&entry.path().to_string_lossy()));
        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    completed = false;
                    continue;
                }
            };
            if !entry.file_type().is_file() || !is_supported_source(entry.path()) {
                continue;
            }
            let path = entry.path().to_string_lossy();
            match state.read_source(&path) {
                Ok(source) if source.contains(query_name) => {
                    candidates.push(root_key(entry.path()))
                }
                Ok(_) => {}
                Err(_) => completed = false,
            }
        }
    }
    candidates.sort();
    candidates.dedup();
    FilesystemScan {
        candidates,
        attempted,
        completed: attempted && completed,
    }
}

fn is_supported_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .and_then(crate::compression::language::language_for_extension)
        .is_some()
}
