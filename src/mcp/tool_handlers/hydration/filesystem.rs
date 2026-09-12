use crate::mcp::McpState;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Conventional metadata, dependency, build-output, and cache directories
/// that do not participate in normal source semantic discovery. These are
/// pruned before their contents can be enumerated or read, independently of
/// user-configured exclusions.
const BUILT_IN_EXCLUDED_DIRECTORIES: &[&str] = &[
    ".git",
    "node_modules",
    "bin",
    "obj",
    "dist",
    "build",
    "target",
    ".venv",
    "venv",
    "__pycache__",
];

pub(super) struct FilesystemScan {
    pub(super) candidates: Vec<String>,
    pub(super) attempted: bool,
    pub(super) completed: bool,
}

#[cfg(all(test, feature = "rust"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TraversalStats {
    pub(crate) directories_entered: usize,
    pub(crate) supported_files_considered: usize,
    pub(crate) files_read: usize,
}

#[cfg(all(test, feature = "rust"))]
static TEST_TRAVERSAL_STATS: std::sync::Mutex<TraversalStats> =
    std::sync::Mutex::new(TraversalStats {
        directories_entered: 0,
        supported_files_considered: 0,
        files_read: 0,
    });

#[cfg(all(test, feature = "rust"))]
pub(crate) fn last_test_traversal_stats() -> TraversalStats {
    *TEST_TRAVERSAL_STATS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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
    #[cfg(all(test, feature = "rust"))]
    let mut stats = TraversalStats::default();

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
            .filter_entry(|entry| {
                !is_built_in_excluded_directory(entry)
                    && !state.config.is_excluded(&entry.path().to_string_lossy())
            });
        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    completed = false;
                    continue;
                }
            };
            #[cfg(all(test, feature = "rust"))]
            if entry.file_type().is_dir() {
                stats.directories_entered += 1;
            }
            if !entry.file_type().is_file() || !is_supported_source(entry.path()) {
                continue;
            }
            #[cfg(all(test, feature = "rust"))]
            {
                stats.supported_files_considered += 1;
                stats.files_read += 1;
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
    #[cfg(all(test, feature = "rust"))]
    {
        *TEST_TRAVERSAL_STATS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = stats;
    }
    FilesystemScan {
        candidates,
        attempted,
        completed: attempted && completed,
    }
}

fn is_built_in_excluded_directory(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && BUILT_IN_EXCLUDED_DIRECTORIES
            .iter()
            .any(|excluded| entry.file_name() == *excluded)
}

fn is_supported_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .and_then(crate::compression::language::language_for_extension)
        .is_some()
}
