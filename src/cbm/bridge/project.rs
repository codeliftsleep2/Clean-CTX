//! CBM project identity.
//!
//! The canonical path-derived project slug and the authoritative
//! root -> project-id map. Split out of `bridge.rs`; no behavior changed.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

/// Derive CBM's canonical project ID from a repository path.
///
/// Verified against the CBM 0.8.1 wire contract (`index_repository` response +
/// `list_projects`):
///   - `C:/Users/MNasty/Desktop/RustContextLayerAI`
///     â†’ `C-Users-MNasty-Desktop-RustContextLayerAI`
///   - `C:/Users/MNasty/AppData/Local/Temp/CleanCtx_Probe.Repo`
///     â†’ `C-Users-MNasty-AppData-Local-Temp-CleanCtx_Probe.Repo` (dots/underscores kept)
///   - `C:/Users/MNasty/AppData/Local/Temp/My space_probe`
///     â†’ `C-Users-MNasty-AppData-Local-Temp-My-space_probe` (space â†’ dash)
///
/// Algorithm: every character outside `[A-Za-z0-9._-]` becomes `-`, runs of
/// `-` collapse, and leading/trailing `-` are trimmed. The directory basename
/// is NEVER used as a project ID.
pub(crate) fn cbm_project_slug(canonical_root: &Path) -> String {
    let raw = canonical_root.to_string_lossy();
    let mut out = String::with_capacity(raw.len());
    let mut last_dash = false;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
            out.push(c);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        } else if out.is_empty() {
            // Leading separator — skip without emitting a dash.
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "default".to_string()
    } else {
        out
    }
}

/// Register a repository root in the bridge's authoritative identity maps.
///
/// Returns the canonical CBM slug for `root`. Both maps are updated:
/// `project_ids` (root â†’ slug) and `project_paths` (slug â†’ root), so indexing,
/// readiness checks, queries, and proxy calls all resolve to one identity.
pub(crate) fn insert_cbm_project(
    project_ids: &mut HashMap<PathBuf, String>,
    project_paths: &mut HashMap<String, PathBuf>,
    root: &Path,
) -> String {
    let slug = cbm_project_slug(root);
    project_paths.insert(slug.clone(), root.to_path_buf());
    project_ids.insert(root.to_path_buf(), slug.clone());
    slug
}
