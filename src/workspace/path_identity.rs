//! Shared filesystem-identity containment for workspace-owned state.

use std::path::Path;

/// Return whether `candidate` is the root itself or a descendant.
///
/// Both inputs use the repository's canonical identity authority when the
/// filesystem can resolve them. Component comparison follows platform
/// filesystem case semantics and never uses a raw string prefix.
pub(crate) fn is_within_root(candidate: &str, root: &str) -> bool {
    let candidate = canonical_components(candidate);
    let root = canonical_components(root);
    components_are_within(&candidate, &root)
}

pub(crate) fn canonical_components(path: &str) -> Vec<String> {
    let canonical = crate::dictionary::path::canonical_identity_key(path);
    path_components(Path::new(&canonical))
}

pub(crate) fn path_components(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect()
}

pub(crate) fn components_are_within(candidate: &[String], root: &[String]) -> bool {
    candidate.len() >= root.len()
        && root
            .iter()
            .zip(candidate)
            .all(|(root_part, candidate_part)| component_eq(root_part, candidate_part))
}

pub(crate) fn components_equal(left: &[String], right: &[String]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| component_eq(left, right))
}

#[cfg(windows)]
fn component_eq(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

#[cfg(not(windows))]
fn component_eq(left: &str, right: &str) -> bool {
    left == right
}
