use std::path::{Path, PathBuf};

pub(super) fn discovery_status(
    attempted: bool,
    completed: bool,
    has_partial_coverage: bool,
) -> &'static str {
    if completed {
        "completed"
    } else if attempted || has_partial_coverage {
        "partial"
    } else {
        "failed"
    }
}

#[cfg(all(test, feature = "rust"))]
pub(super) fn append_project_paths(
    candidates: &mut Vec<String>,
    root: &Path,
    nodes: Vec<crate::cbm::GraphNode>,
) {
    append_rooted_paths(
        candidates,
        root,
        nodes.into_iter().map(|node| node.file).collect(),
    );
}

pub(super) fn append_rooted_paths(
    candidates: &mut Vec<String>,
    root: &Path,
    paths: Vec<String>,
) {
    candidates.extend(paths.into_iter().filter_map(|file| {
        if file.is_empty() {
            return None;
        }
        let path = PathBuf::from(file);
        let rooted = if path.is_absolute() {
            path
        } else {
            root.join(path)
        };
        Some(crate::dictionary::path::canonical_identity_key(
            &rooted.to_string_lossy(),
        ))
    }));
}
