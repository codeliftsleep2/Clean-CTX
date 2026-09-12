use crate::mcp::McpState;
use ignore::{DirEntry, WalkBuilder};
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
    pub(crate) traversal_elapsed: std::time::Duration,
    pub(crate) read_match_elapsed: std::time::Duration,
    pub(crate) total_elapsed: std::time::Duration,
}

#[cfg(all(test, feature = "rust"))]
thread_local! {
    static TEST_TRAVERSAL_STATS: std::cell::Cell<TraversalStats> = const { std::cell::Cell::new(TraversalStats {
        directories_entered: 0,
        supported_files_considered: 0,
        files_read: 0,
        traversal_elapsed: std::time::Duration::ZERO,
        read_match_elapsed: std::time::Duration::ZERO,
        total_elapsed: std::time::Duration::ZERO,
    }) };
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn last_test_traversal_stats() -> TraversalStats {
    TEST_TRAVERSAL_STATS.get()
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
    #[cfg(all(test, feature = "rust"))]
    let total_started = std::time::Instant::now();
    #[cfg(all(test, feature = "rust"))]
    let traversal_started = std::time::Instant::now();
    let mut source_paths = Vec::new();
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
        let config = state.config.clone();
        let mut builder = WalkBuilder::new(&root);
        builder
            .standard_filters(false)
            .hidden(false)
            .parents(true)
            .ignore(false)
            .git_ignore(true)
            .git_global(false)
            .git_exclude(true)
            .require_git(true)
            .follow_links(false)
            .filter_entry(move |entry| {
                !is_built_in_excluded_directory(entry)
                    && !config.is_excluded(&entry.path().to_string_lossy())
            });
        for entry in builder.build() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    completed = false;
                    continue;
                }
            };
            #[cfg(all(test, feature = "rust"))]
            if entry
                .file_type()
                .is_some_and(|file_type| file_type.is_dir())
            {
                stats.directories_entered += 1;
            }
            if !entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
                || !is_supported_source(entry.path())
            {
                continue;
            }
            source_paths.push(root_key(entry.path()));
        }
    }

    source_paths.sort();
    source_paths.dedup();
    #[cfg(all(test, feature = "rust"))]
    {
        stats.supported_files_considered = source_paths.len();
        stats.files_read = source_paths.len();
        stats.traversal_elapsed = traversal_started.elapsed();
    }
    #[cfg(all(test, feature = "rust"))]
    let read_match_started = std::time::Instant::now();
    let outcomes: Vec<_> = source_paths
        .par_iter()
        .map(|path| match state.read_source(path) {
            Ok(source) => (source.contains(query_name).then(|| path.clone()), true),
            Err(_) => (None, false),
        })
        .collect();
    #[cfg(all(test, feature = "rust"))]
    {
        stats.read_match_elapsed = read_match_started.elapsed();
    }
    let mut candidates = Vec::new();
    for (candidate, read_succeeded) in outcomes {
        completed &= read_succeeded;
        if let Some(candidate) = candidate {
            candidates.push(candidate);
        }
    }
    candidates.sort();
    candidates.dedup();
    #[cfg(all(test, feature = "rust"))]
    {
        stats.total_elapsed = total_started.elapsed();
        TEST_TRAVERSAL_STATS.set(stats);
    }
    FilesystemScan {
        candidates,
        attempted,
        completed: attempted && completed,
    }
}

fn is_built_in_excluded_directory(entry: &DirEntry) -> bool {
    entry
        .file_type()
        .is_some_and(|file_type| file_type.is_dir())
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

#[cfg(all(test, feature = "rust"))]
#[path = "../../../tests/mcp/hydration_filesystem.rs"]
mod tests;
