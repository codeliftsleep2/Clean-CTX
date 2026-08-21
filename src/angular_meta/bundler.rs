// src/angular_meta/bundler.rs
//
// File-triplet resolver — Tier 2 of the Meta-Layer.
//
// Given a `.component.ts` file, resolve its sibling template and
// style files (`*.component.html`, `*.component.scss`, etc.).
// This enables the LLM to see "these three files are one logical
// unit" without burning tokens on raw HTML/SCSS.
//
// Workspace-mode only.

use std::path::{Path, PathBuf};

/// Supported Angular-adjacent style extensions, in priority order.
const STYLE_EXTENSIONS: &[&str] = &["scss", "css", "sass", "less"];

/// Template extension.
const TEMPLATE_EXTENSION: &str = "html";

/// A resolved file triplet for an Angular component.
///
/// The component `.ts` file is always present; the template and
/// style files are optional (some components use inline templates
/// or no styles).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTriplet {
    /// Absolute path to the `.component.ts` file.
    pub component_ts: PathBuf,
    /// Absolute path to the template `.html` file, if found.
    pub template: Option<PathBuf>,
    /// Absolute path to the style file, if found.
    pub style: Option<PathBuf>,
}

/// Bundle group output — a component plus its resolved siblings,
/// suitable for emitting `ΦBUNDLE` lines in the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleGroup {
    /// The component name (filename without extension, e.g.
    /// `"user-card.component"`).
    pub name: String,
    /// The resolved file triplet.
    pub triplet: FileTriplet,
}

impl BundleGroup {
    /// Returns `true` if the triplet has at least one sibling
    /// (template or style file).
    pub fn has_siblings(&self) -> bool {
        self.triplet.template.is_some() || self.triplet.style.is_some()
    }
}

/// Check if the given path looks like an Angular component `.ts` file.
///
/// A file is considered a component if its filename ends with
/// `.component.ts` (case-sensitive). This is the standard Angular
/// convention and avoids false positives on service files,
/// directives, pipes, etc.
pub fn is_component_ts(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name.ends_with(".component.ts"))
}

/// Resolve the file triplet for a `.component.ts` file.
///
/// Searches the parent directory for matching sibling files:
/// - `<name>.component.html` (template)
/// - `<name>.component.scss` / `.css` / `.sass` / `.less` (style)
///
/// The first matching style extension wins (scss > css > sass > less).
///
/// Returns `None` if the input is not a `.component.ts` file.
pub fn resolve_triplet(component_path: &Path) -> Option<FileTriplet> {
    if !is_component_ts(component_path) {
        return None;
    }

    let parent = component_path.parent()?;
    let stem = component_path.file_stem()?.to_str()?; // "foo.component"

    // The base name is the full stem (e.g. "foo.component").
    // Sibling files follow the Angular convention:
    //   foo.component.html, foo.component.scss, etc.
    let base = stem;

    let template = find_sibling(parent, base, TEMPLATE_EXTENSION);
    let style = find_first_style_sibling(parent, base);

    Some(FileTriplet {
        component_ts: component_path.to_path_buf(),
        template,
        style,
    })
}

/// Resolve the bundle group for a `.component.ts` file.
///
/// Returns `Some(BundleGroup)` only for component files. Non-component
/// files (services, pipes, directives) return `None` — they are not
/// bundled.
pub fn resolve_bundle_group(component_path: &Path) -> Option<BundleGroup> {
    if !is_component_ts(component_path) {
        return None;
    }

    let name = component_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let triplet = resolve_triplet(component_path)?;

    Some(BundleGroup { name, triplet })
}

/// Find a sibling file with the given base name and extension.
fn find_sibling(parent: &Path, base: &str, ext: &str) -> Option<PathBuf> {
    let candidate = parent.join(format!("{}.{}", base, ext));
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

/// Find the first matching style sibling, trying extensions in
/// priority order: scss > css > sass > less.
fn find_first_style_sibling(parent: &Path, base: &str) -> Option<PathBuf> {
    for ext in STYLE_EXTENSIONS {
        if let Some(path) = find_sibling(parent, base, ext) {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
#[path = "../tests/angular_meta/bundler.rs"]
mod tests;
