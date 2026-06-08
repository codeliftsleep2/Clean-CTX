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

use std::collections::BTreeMap;

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
        footer.push_str(&format!("  {} = {}", entry.alias, entry.name));
        if !entry.file_aliases.is_empty() {
            footer.push_str(&format!(" [{}]", entry.file_aliases.join(", ")));
        }
        footer.push('\n');

        if let Some(ref tpl) = entry.template_summary {
            footer.push_str(&format!("    {}\n", tpl));
        }
        if let Some(ref sty) = entry.style_summary {
            footer.push_str(&format!("    {}\n", sty));
        }
    }
    footer
}

/// Builder for constructing the `§ΦMAP` footer incrementally.
///
/// This is used by the workspace compression pass to collect bundle
/// entries as files are processed, then format the footer at the end.
#[derive(Debug, Clone, Default)]
pub struct FooterBuilder {
    /// Maps bundle alias → entry. Uses BTreeMap for deterministic
    /// output order.
    entries: BTreeMap<String, BundleEntry>,
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
        self.entries.insert(
            alias.clone(),
            BundleEntry {
                alias: alias.clone(),
                name,
                file_aliases,
                template_summary,
                style_summary,
            },
        );
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
        let entries: Vec<BundleEntry> = self.entries.values().cloned().collect();
        format_bundle_footer(&entries)
    }

    /// Look up a bundle alias by component name.
    pub fn find_by_name(&self, name: &str) -> Option<&BundleEntry> {
        self.entries.values().find(|e| e.name == name)
    }
}

#[cfg(test)]
#[path = "../tests/angular_meta/footer.rs"]
mod tests;