// proxy/src/filter_registry.rs
//
// Filter registry: manages all loaded filters, selects the best match
// for a given command using most-specific-match-wins selection.

use std::collections::HashMap;

use crate::filter_rules::CompiledFilter;

/// Registry of all loaded filters.
#[derive(Debug, Clone)]
pub struct FilterRegistry {
    /// Built-in filters shipped with the binary.
    pub builtin: HashMap<String, CompiledFilter>,

    /// Community/custom filters from .clean-ctx/filters/.
    pub community: HashMap<String, CompiledFilter>,

    /// Configuration overrides from .clean-ctx.json.
    pub overrides: HashMap<String, CompiledFilterWrapper>,
}

/// Wrapper for a compiled filter with config overrides.
#[derive(Debug, Clone)]
pub struct CompiledFilterWrapper {
    pub filter: CompiledFilter,
    pub enabled: bool,
    pub max_lines_override: Option<usize>,
}

impl FilterRegistry {
    /// Create a new, empty registry.
    pub fn new() -> Self {
        Self {
            builtin: HashMap::new(),
            community: HashMap::new(),
            overrides: HashMap::new(),
        }
    }

    /// Select the best filter for a given command string.
    ///
    /// Selection rules (from ctx-wire):
    /// 1. Most specific match wins (longest `match_command` matched span)
    /// 2. Priority breaks ties between equal-length spans
    /// 3. Disabled filters are excluded
    pub fn select_for_command(&self, command: &str) -> Option<&CompiledFilter> {
        // Collect all enabled filters
        let all_filters: Vec<&CompiledFilter> = self
            .builtin
            .values()
            .chain(self.community.values())
            .filter(|f| {
                // Check overrides for disabling
                let wrapper = self.overrides.get(&f.name);
                wrapper.map(|w| w.enabled).unwrap_or(true)
            })
            .filter(|f| f.match_command.is_match(command))
            .collect();

        if all_filters.is_empty() {
            return None;
        }

        // Most specific match wins (longest matched span)
        // For regex, we use the length of the matched string
        all_filters
            .into_iter()
            .max_by_key(|f| {
                let span = f
                    .match_command
                    .find(command)
                    .map(|m| m.len())
                    .unwrap_or(0);
                (span, f.priority)
            })
    }

    /// Get a filter by name, checking overrides first, then builtin, then community.
    pub fn get(&self, name: &str) -> Option<&CompiledFilter> {
        // Check overrides first
        if let Some(wrapper) = self.overrides.get(name) {
            if wrapper.enabled {
                // Apply any overrides (clone to apply max_lines_override)
                return Some(&wrapper.filter);
            }
            return None;
        }

        // Then builtin
        if let Some(f) = self.builtin.get(name) {
            return Some(f);
        }

        // Then community
        self.community.get(name)
    }

    /// Add a built-in filter.
    pub fn add_builtin(&mut self, filter: CompiledFilter) {
        self.builtin.insert(filter.name.clone(), filter);
    }

    /// Add a community filter.
    pub fn add_community(&mut self, filter: CompiledFilter) {
        self.community.insert(filter.name.clone(), filter);
    }

    /// Set a configuration override.
    pub fn set_override(
        &mut self,
        name: String,
        filter: CompiledFilter,
        enabled: bool,
        max_lines_override: Option<usize>,
    ) {
        self.overrides.insert(
            name,
            CompiledFilterWrapper {
                filter,
                enabled,
                max_lines_override,
            },
        );
    }

    /// Check if any filters are available for a command.
    pub fn has_filter_for(&self, command: &str) -> bool {
        self.select_for_command(command).is_some()
    }

    /// Total number of filters (builtin + community).
    pub fn count(&self) -> usize {
        self.builtin.len() + self.community.len()
    }
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter_rules::CompiledFilter;
    use regex::Regex;

    fn make_filter(name: &str, pattern: &str, priority: i32) -> CompiledFilter {
        CompiledFilter {
            name: name.to_string(),
            description: String::new(),
            match_command: Regex::new(pattern).unwrap(),
            priority,
            strip_ansi: false,
            filter_stderr: false,
            reduce_json: false,
            replace: vec![],
            match_output: vec![],
            strip_lines: vec![],
            keep_lines: vec![],
            group_by: None,
            head_lines: None,
            tail_lines: None,
            max_lines: None,
            on_empty: None,
            user_config_key: None,
        }
    }

    #[test]
    fn test_select_most_specific_wins() {
        let mut registry = FilterRegistry::new();
        registry.add_builtin(make_filter("cargo", "^cargo\\b", 0));
        registry.add_builtin(make_filter("cargo-test", "^cargo\\s+test\\b", 0));

        // cargo test should match cargo-test (more specific)
        let selected = registry.select_for_command("cargo test --lib");
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "cargo-test");
    }

    #[test]
    fn test_select_no_match() {
        let mut registry = FilterRegistry::new();
        registry.add_builtin(make_filter("cargo", "^cargo\\b", 0));

        let selected = registry.select_for_command("npm install");
        assert!(selected.is_none());
    }

    #[test]
    fn test_select_priority_breaks_ties() {
        let mut registry = FilterRegistry::new();
        registry.add_builtin(make_filter("generic", "^[a-z]+\\b", 0));
        registry.add_builtin(make_filter("specific", "^[a-z]+\\b", 10));

        let selected = registry.select_for_command("cargo build");
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "specific");
    }

    #[test]
    fn test_disabled_filter_excluded() {
        let mut registry = FilterRegistry::new();
        registry.add_builtin(make_filter("cargo", "^cargo\\b", 0));
        registry.set_override(
            "cargo".to_string(),
            make_filter("cargo", "^cargo\\b", 0),
            false, // disabled
            None,
        );

        let selected = registry.select_for_command("cargo build");
        assert!(selected.is_none());
    }

    #[test]
    fn test_get_by_name() {
        let mut registry = FilterRegistry::new();
        registry.add_builtin(make_filter("cargo", "^cargo\\b", 0));

        let filter = registry.get("cargo");
        assert!(filter.is_some());
        assert_eq!(filter.unwrap().name, "cargo");

        assert!(registry.get("npm").is_none());
    }
}