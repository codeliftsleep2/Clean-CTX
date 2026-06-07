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
}
