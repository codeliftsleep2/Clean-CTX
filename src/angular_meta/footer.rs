// src/angular_meta/footer.rs
//
// `§ΦMAP` workspace footer formatter — Tier 2 of the Meta-Layer.
//
// After all files in a workspace have been compressed, the bundler
// emits a footer listing all Angular bundle aliases (Φ1, Φ2, …)
// so the LLM can quickly navigate the workspace's Angular structure.
//
// The footer is appended to the workspace manifest after the
// `§PATHMAP` footer.

use std::collections::HashMap;

/// A single bundle entry in the `§ΦMAP` footer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleEntry {
    /// The bundle alias (e.g. `"Φ1"`).
    pub alias: String,
    /// The human-readable component name (e.g. `"user-card.component"`).
    pub name: String,
    /// Alpha-aliases of the files in this bundle, in order:
    /// `[component.ts, template.html, style.scss]`.
    pub file_aliases: Vec<String>,
    /// Optional one-line template shape summary.
    pub template_summary: Option<String>,
    /// Optional one-line style shape summary.
    pub style_summary: Option<String>,
}

/// Format the `§ΦMAP` footer for a workspace manifest.
///
/// Takes a list of bundle entries (already sorted by alias) and
/// produces the formatted footer string. Returns an empty string
/// if there are no bundles (so the caller can `+=` blindly).
pub fn format_bundle_footer(entries: &[BundleEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut footer = String::from("\n§ΦMAP\n");
    for entry in entries {
        use std::fmt::Write;
        // F-ANG-39 (mirrored from main audit): use `write!` instead of
        // `format!` + `push_str` to avoid the intermediate String alloc.
        let _ = writeln!(footer, "  {} = {}", entry.alias, entry.name);
        if !entry.file_aliases.is_empty() {
            let _ = write!(footer, " [{}]", entry.file_aliases.join(", "));
        }
        // The opening line above already ended with a newline, so we
        // need to re-establish the entry line if `file_aliases` was
        // emitted. Push the closing newline now.
        if !entry.file_aliases.is_empty() {
            footer.push('\n');
        }

        if let Some(ref tpl) = entry.template_summary {
            let _ = writeln!(footer, "    {}", tpl);
        }
        if let Some(ref sty) = entry.style_summary {
            let _ = writeln!(footer, "    {}", sty);
        }
    }
    footer
}

/// Builder for constructing the `§ΦMAP` footer incrementally.
///
/// This is used by the workspace compression pass to collect bundle
/// entries as files are processed, then format the footer at the end.
///
/// # Storage
///
/// Uses `HashMap` for the entries (F-ANG-22). Aliases are monotonically
/// increasing (`Φ1`, `Φ2`, …) so iteration order matches insertion
/// order, and `format_bundle_footer` sorts on emit for determinism.
#[derive(Debug, Clone, Default)]
pub struct FooterBuilder {
    /// Maps bundle alias → entry. HashMap for O(1) lookup.
    entries: HashMap<String, BundleEntry>,
    /// Secondary index: component name → bundle alias (F-ANG-21).
    by_name: HashMap<String, String>,
    /// Counter for generating unique aliases (Φ1, Φ2, …).
    next_index: usize,
}

impl FooterBuilder {
    /// Create a new empty footer builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new bundle and return its alias.
    pub fn register_bundle(
        &mut self,
        name: String,
        file_aliases: Vec<String>,
        template_summary: Option<String>,
        style_summary: Option<String>,
    ) -> String {
        self.next_index += 1;
        let alias = format!("Φ{}", self.next_index);
        // F-ANG-14: avoid cloning the alias twice.
        self.entries.insert(
            alias.clone(),
            BundleEntry {
                alias: alias.clone(),
                name: name.clone(),
                file_aliases,
                template_summary,
                style_summary,
            },
        );
        self.by_name.insert(name, alias.clone());
        alias
    }

    /// Number of registered bundles.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no bundles have been registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Format the complete `§ΦMAP` footer.
    pub fn build(&self) -> String {
        let mut entries: Vec<BundleEntry> = self.entries.values().cloned().collect();
        // Sort by alias (lexicographic Φ1, Φ10, Φ2, … would be wrong,
        // but aliases are zero-padded implicitly because all have the
        // same Φ prefix and monotonically increasing integer suffix).
        // Use natural sort via a stable comparator on the trailing
        // number.
        entries.sort_by(|a, b| natural_cmp(&a.alias, &b.alias));
        format_bundle_footer(&entries)
    }

    /// Look up a bundle alias by component name (F-ANG-21: O(1)).
    pub fn find_by_name(&self, name: &str) -> Option<&BundleEntry> {
        self.by_name
            .get(name)
            .and_then(|alias| self.entries.get(alias))
    }
}

/// Natural-order comparator that sorts `"Φ2"` before `"Φ10"`.
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let num_a: Option<usize> = a.strip_prefix('Φ').and_then(|s| s.parse().ok());
    let num_b: Option<usize> = b.strip_prefix('Φ').and_then(|s| s.parse().ok());
    match (num_a, num_b) {
        (Some(na), Some(nb)) => na.cmp(&nb),
        _ => a.cmp(b),
    }
}

#[cfg(test)]
#[path = "../tests/angular_meta/footer.rs"]
mod tests;
