// src/spring_meta/footer.rs
//
// `§ΦMAP` workspace footer formatter — Tier 2 of the Spring Boot Meta-Layer.
//
// After all files in a workspace have been compressed, the bundler
// emits a footer listing all Spring Boot bundle aliases (Φ1, Φ2, …)
// so the LLM can quickly navigate the workspace's Spring architecture.
//
// The footer is appended to the workspace manifest after the
// `§PATHMAP` footer.

use std::collections::HashMap;

/// A single bundle entry in the `§ΦMAP` footer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleEntry {
    /// The bundle alias (e.g. `"Φ1"`).
    pub alias: String,
    /// The human-readable component name (e.g. `"user-controller"`).
    pub name: String,
    /// The Spring layer (Controller, Service, Repository, Configuration).
    pub layer: crate::spring_meta::bundler::SpringLayer,
    /// Alpha-aliases of the files in this bundle.
    pub file_aliases: Vec<String>,
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
        let layer_str = match entry.layer {
            crate::spring_meta::bundler::SpringLayer::Controller => "ctrl",
            crate::spring_meta::bundler::SpringLayer::Service => "svc",
            crate::spring_meta::bundler::SpringLayer::Repository => "repo",
            crate::spring_meta::bundler::SpringLayer::Configuration => "conf",
            crate::spring_meta::bundler::SpringLayer::Unknown => "?",
        };
        let _ = writeln!(footer, "  {} = {} [{}]", entry.alias, entry.name, layer_str);
        if !entry.file_aliases.is_empty() {
            let _ = write!(footer, " [{}]", entry.file_aliases.join(", "));
            footer.push('\n');
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
/// Uses `HashMap` for the entries. Aliases are monotonically
/// increasing (`Φ1`, `Φ2`, …) so iteration order matches insertion
/// order, and `format_bundle_footer` sorts on emit for determinism.
#[derive(Debug, Clone, Default)]
pub struct FooterBuilder {
    /// Maps bundle alias → entry. HashMap for O(1) lookup.
    entries: HashMap<String, BundleEntry>,
    /// Secondary index: component name → bundle alias.
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
        layer: crate::spring_meta::bundler::SpringLayer,
        file_aliases: Vec<String>,
    ) -> String {
        self.next_index += 1;
        let alias = format!("Φ{}", self.next_index);
        self.entries.insert(
            alias.clone(),
            BundleEntry {
                alias: alias.clone(),
                name: name.clone(),
                layer,
                file_aliases,
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
        // Sort by alias (natural sort on trailing number).
        entries.sort_by(|a, b| natural_cmp(&a.alias, &b.alias));
        format_bundle_footer(&entries)
    }

    /// Look up a bundle alias by component name.
    pub fn find_by_name(&self, name: &str) -> Option<&BundleEntry> {
        self.by_name
            .get(name)
            .and_then(|alias| self.entries.get(alias))
    }
}

/// Natural-order comparator that sorts `"Φ2"` before `"Φ10"`.
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let num_a: Option<usize> = a
        .strip_prefix('Φ')
        .and_then(|s| s.parse().ok());
    let num_b: Option<usize> = b
        .strip_prefix('Φ')
        .and_then(|s| s.parse().ok());
    match (num_a, num_b) {
        (Some(na), Some(nb)) => na.cmp(&nb),
        _ => a.cmp(b),
    }
}

#[cfg(test)]
#[path = "../tests/spring_meta/footer_tests.rs"]
mod tests;
