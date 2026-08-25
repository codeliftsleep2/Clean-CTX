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
    /// path → alias (the reverse index for O(1) lookup).
    ///
    /// Keys are CANONICAL paths (see `normalize_alias_key`) so that
    /// different caller-supplied spellings of the same physical file
    /// converge onto one stable alias.
    reverse: HashMap<String, String>,
    /// bundle alias → component name (e.g. "Φ1" → "user-card.component")
    bundle_aliases: HashMap<String, String>,
    /// component name → bundle alias (reverse index for O(1) lookup)
    bundle_reverse: HashMap<String, String>,
}

/// Normalize a caller-supplied path to THE canonical file identity used by
/// every in-session stateful consumer.
///
/// Non-CBM audit 2026-08-25 #3: handlers mix absolute paths and
/// workspace-relative/joined spellings of the SAME file (`resolve_file_path_checked`
/// deliberately returns the caller-shaped path), so keying on the raw
/// argument fragmented identity — one file could hold two aliases
/// (visible as duplicate `α` entries in `§PATHMAP`), silently splitting
/// every alias-keyed state (IR context, text-delta baselines, LLM cache).
///
/// IDENT-001 extension: `SessionStats` keys through this same helper so
/// stats aggregate one physical file into one row. The SQLite persistence
/// layer is the DELIBERATE exception (durable rows keep caller-shaped
/// keys; migrating would orphan historical baselines — pinned by the
/// `persistence_keys_are_caller_shaped_by_contract` test).
///
/// Resolution: `fs::canonicalize` when possible; on Windows the verbatim
/// (`\\?\`) prefix is stripped so stored keys stay human-readable.
/// Unresolvable paths (deleted files, synthetic strings used by tests)
/// fall back to the raw argument unchanged.
pub(crate) fn canonical_identity_key(path: &str) -> String {
    match std::fs::canonicalize(path) {
        Ok(canon) => {
            let s = canon.to_string_lossy();
            #[cfg(windows)]
            {
                if let Some(unc) = s.strip_prefix(r"\\?\UNC\") {
                    format!(r"\\{unc}")
                } else {
                    s.strip_prefix(r"\\?\").unwrap_or(&s).to_string()
                }
            }
            #[cfg(not(windows))]
            {
                s.into_owned()
            }
        }
        Err(_) => path.to_string(),
    }
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
        // Identity invariant: ONE physical file ⇒ ONE stable alias,
        // regardless of the path form the caller supplies. Fast-path the
        // exact-string hit first so repeat calls with identical arguments
        // stay O(1) without touching the filesystem; otherwise resolve to
        // the canonical key before consulting the map.
        // Non-CBM audit 2026-08-25 #3.
        if let Some(alias) = self.reverse.get(&absolute_path).cloned() {
            return alias;
        }
        let key = canonical_identity_key(&absolute_path);
        if let Some(alias) = self.reverse.get(&key).cloned() {
            alias
        } else {
            let alias = format!("α{}", self.forward.len() + 1);
            self.reverse.insert(key.clone(), alias.clone());
            self.forward.insert(alias.clone(), key);
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
