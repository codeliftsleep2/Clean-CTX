// src/dictionary/path.rs
//
// Maps absolute file paths to short aliases (e.g., α1, α2).

use std::collections::BTreeMap;

pub struct PathDictionary {
    mappings: BTreeMap<String, String>,
}

impl Default for PathDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl PathDictionary {
    pub fn new() -> Self {
        Self { mappings: BTreeMap::new() }
    }

    pub fn get_or_create_alias(&mut self, absolute_path: String) -> String {
        if let Some(alias) = self.mappings.iter().find(|(_, p)| **p == absolute_path).map(|(a, _)| a.clone()) {
            alias
        } else {
            let alias = format!("α{}", self.mappings.len() + 1);
            self.mappings.insert(alias.clone(), absolute_path);
            alias
        }
    }

    pub fn format_footer(&self) -> String {
        let mut footer = String::from("\n§PATHMAP\n");
        for (alias, real_path) in &self.mappings {
            footer.push_str(&format!("  {} = {}\n", alias, real_path));
        }
        footer
    }
}
