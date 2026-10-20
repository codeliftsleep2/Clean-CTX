use crate::mcp::McpState;
use crate::mcp::acquisition::{
    InventoryEntry, InventoryManifest, SourceKind, type_ref_candidate_owners,
};
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
    pub(super) inventories: Vec<RootInventoryManifest>,
}

pub(super) struct RootInventoryManifest {
    pub(super) root: String,
    pub(super) manifest: InventoryManifest,
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

// Test-only: number of `scan` invocations on this thread.
//
// Structural discovery-cost instrumentation. `TraversalStats` describes the
// last scan only, so it cannot tell "the scan did not run" from "the scan ran
// and read nothing". The discovery-cache regressions assert on this counter
// instead of wall-clock timing. Thread-local so that concurrently running test
// binaries cannot perturb an assertion.
#[cfg(all(test, feature = "rust"))]
thread_local! {
    static TEST_SCAN_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn test_scan_calls() -> usize {
    TEST_SCAN_CALLS.get()
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn reset_test_scan_calls() {
    TEST_SCAN_CALLS.set(0);
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn total_test_scan_calls(state: &McpState) -> usize {
    state
        .hydration_scan_calls
        .load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn reset_total_test_scan_calls(state: &McpState) {
    state
        .hydration_scan_calls
        .store(0, std::sync::atomic::Ordering::SeqCst);
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

/// Discover supported source files containing a literal in the effective roots.
/// Angular ownership uses this only to narrow TypeScript files that are then
/// verified by structured decorator extraction.
pub(crate) fn source_candidates_containing(
    state: &McpState,
    workspace_root: Option<&str>,
    literal: &str,
) -> Vec<String> {
    let roots = configured_roots(state, workspace_root);
    scan(state, &roots, literal).candidates
}

pub(crate) fn effective_refresh_roots(state: &McpState, workspace_root: &str) -> Vec<String> {
    configured_roots(state, Some(workspace_root))
        .iter()
        .map(|root| root_key(root))
        .collect()
}

pub(super) fn scan(state: &McpState, roots: &[PathBuf], query_name: &str) -> FilesystemScan {
    #[cfg(all(test, feature = "rust"))]
    {
        TEST_SCAN_CALLS.set(TEST_SCAN_CALLS.get() + 1);
        state
            .hydration_scan_calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
    #[cfg(all(test, feature = "rust"))]
    let total_started = std::time::Instant::now();
    #[cfg(all(test, feature = "rust"))]
    let traversal_started = std::time::Instant::now();
    #[cfg(all(test, feature = "rust"))]
    let mut stats = TraversalStats::default();
    let (source_paths, attempted, traversal_completed, _directories_entered) =
        enumerate_source_paths(state, roots);
    #[cfg(all(test, feature = "rust"))]
    {
        stats.directories_entered = _directories_entered;
        stats.supported_files_considered = source_paths.len();
        stats.files_read = source_paths.len();
        stats.traversal_elapsed = traversal_started.elapsed();
    }
    #[cfg(all(test, feature = "rust"))]
    let read_match_started = std::time::Instant::now();
    let manifest = build_inventory(state, source_paths, attempted, traversal_completed);
    #[cfg(all(test, feature = "rust"))]
    {
        stats.read_match_elapsed = read_match_started.elapsed();
    }
    let candidates = type_ref_candidate_owners(&manifest, query_name);
    let inventories = root_inventories(roots, &manifest);
    #[cfg(all(test, feature = "rust"))]
    {
        stats.total_elapsed = total_started.elapsed();
        TEST_TRAVERSAL_STATS.set(stats);
    }
    FilesystemScan {
        candidates,
        attempted: manifest.attempted,
        completed: manifest.complete,
        inventories,
    }
}

fn root_inventories(roots: &[PathBuf], combined: &InventoryManifest) -> Vec<RootInventoryManifest> {
    roots
        .iter()
        .map(|root| {
            let root = root_key(root);
            let entries = combined
                .entries
                .iter()
                .filter(|entry| Path::new(&entry.owner).starts_with(Path::new(&root)))
                .cloned()
                .collect();
            RootInventoryManifest {
                root,
                manifest: InventoryManifest::new(entries, combined.attempted, combined.complete),
            }
        })
        .collect()
}

#[cfg(any(feature = "csharp", all(test, feature = "rust")))]
pub(crate) fn source_inventory(state: &McpState, roots: &[PathBuf]) -> InventoryManifest {
    let (paths, attempted, traversal_completed, _) = enumerate_source_paths(state, roots);
    build_inventory(state, paths, attempted, traversal_completed)
}

fn enumerate_source_paths(state: &McpState, roots: &[PathBuf]) -> (Vec<String>, bool, bool, usize) {
    let mut source_paths = Vec::new();
    let mut attempted = false;
    let mut completed = true;
    let mut directories_entered = 0;
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
            if entry
                .file_type()
                .is_some_and(|file_type| file_type.is_dir())
            {
                directories_entered += 1;
            }
            if entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
                && is_supported_source(entry.path())
            {
                source_paths.push(root_key(entry.path()));
            }
        }
    }
    source_paths.sort();
    source_paths.dedup();
    (source_paths, attempted, completed, directories_entered)
}

fn build_inventory(
    state: &McpState,
    source_paths: Vec<String>,
    attempted: bool,
    traversal_completed: bool,
) -> InventoryManifest {
    let outcomes: Vec<_> = source_paths
        .par_iter()
        .map(|owner| {
            let source_kind = SourceKind::from_path(Path::new(owner));
            // Inventory observation is request-local proof input, not semantic
            // admission. Do not populate the shared source cache merely by
            // enumerating a file: request-local catalogs consume that cache as
            // admitted semantic evidence.
            let source = std::fs::read_to_string(owner).map(std::sync::Arc::new);
            match (source_kind, source) {
                (Some(source_kind), Ok(source)) => Some(InventoryEntry {
                    owner: state.semantic_owner_path(owner),
                    source_kind,
                    source_hash: crate::cache::content_hash(source.as_bytes()),
                    source,
                }),
                _ => None,
            }
        })
        .collect();
    let complete = traversal_completed && outcomes.iter().all(Option::is_some);
    InventoryManifest::new(
        outcomes.into_iter().flatten().collect(),
        attempted,
        complete,
    )
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
