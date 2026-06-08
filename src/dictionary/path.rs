// src/dictionary/path.rs
//
// Maps absolute file paths to short aliases (e.g., α1, α2).
//
// F-36 (FAANG audit): uses a `HashMap` for the reverse lookup (path →
// alias) so `get_or_create_alias` is O(1) instead of O(n).

use std::collections::HashMap;

pub struct PathDictionary {
    /// alias → path (e.g. "α1" → "/project/src/main.ts")
    forward: HashMap<String, String>,
    /// path → alias (the reverse index for O(1) lookup)
    reverse: HashMap<String, String>,
    /// bundle alias → component name (e.g. "Φ1" → "user-card.component")
    bundle_aliases: HashMap<String, String>,
    /// component name → bundle alias (reverse index for O(1) lookup)
    bundle_reverse: HashMap<String, String>,
}

impl Default for PathDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl PathDictionary {
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            reverse: HashMap::new(),
            bundle_aliases: HashMap::new(),
            bundle_reverse: HashMap::new(),
        }
    }

    pub fn get_or_create_alias(&mut self, absolute_path: String) -> String {
        if let Some(alias) = self.reverse.get(&absolute_path).cloned() {
            alias
        } else {
            let alias = format!("α{}", self.forward.len() + 1);
            self.reverse.insert(absolute_path.clone(), alias.clone());
            self.forward.insert(alias.clone(), absolute_path);
            alias
        }
    }

    pub fn format_footer(&self) -> String {
        let mut footer = String::from("\n§PATHMAP\n");
        for (alias, real_path) in &self.forward {
            footer.push_str(&format!("  {} = {}\n", alias, real_path));
        }
        footer
    }

    // ------------------------------------------------------------------
    // Bundle aliases (Φ1, Φ2, …) — Tier 2 of the Angular Meta-Layer.
    // ------------------------------------------------------------------

    /// Register a new bundle alias for a component name.
    ///
    /// Returns the bundle alias (e.g. `"Φ1"`). If the component
    /// already has a bundle alias, the existing one is returned.
    pub fn get_or_create_bundle_alias(&mut self, component_name: String) -> String {
        if let Some(alias) = self.bundle_reverse.get(&component_name).cloned() {
            alias
        } else {
            let alias = format!("Φ{}", self.bundle_aliases.len() + 1);
            self.bundle_reverse
                .insert(component_name.clone(), alias.clone());
            self.bundle_aliases.insert(alias.clone(), component_name);
            alias
        }
    }

    /// Look up the bundle alias for a component name, if one exists.
    pub fn get_bundle_alias(&self, component_name: &str) -> Option<&str> {
        self.bundle_reverse.get(component_name).map(|s| s.as_str())
    }
}
