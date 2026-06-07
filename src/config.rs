// src/config.rs — Project-level configuration for Clean-CTX
// Reads .clean-ctx.json from the project root for custom settings

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Project-level configuration loaded from `.clean-ctx.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanCtxConfig {
    /// Custom type aliases: short_name → original_type
    #[serde(default)]
    pub type_aliases: BTreeMap<String, String>,

    /// Fidelity override per file extension
    #[serde(default)]
    pub fidelity_overrides: BTreeMap<String, String>,

    /// File/directory patterns to exclude from compression
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Custom behavior markers: marker → description
    #[serde(default)]
    pub custom_markers: BTreeMap<String, String>,

    /// Default fidelity level if not specified
    #[serde(default = "default_fidelity")]
    pub default_fidelity: String,

    /// Whether to enable diff-aware compression
    #[serde(default = "default_true")]
    pub diff_compression: bool,

    /// Whether to enable workspace-wide type detection
    #[serde(default = "default_true")]
    pub workspace_type_detection: bool,
}

fn default_fidelity() -> String {
    "low".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for CleanCtxConfig {
    fn default() -> Self {
        Self {
            type_aliases: BTreeMap::new(),
            fidelity_overrides: BTreeMap::new(),
            exclude_patterns: Vec::new(),
            custom_markers: BTreeMap::new(),
            default_fidelity: default_fidelity(),
            diff_compression: default_true(),
            workspace_type_detection: default_true(),
        }
    }
}

impl CleanCtxConfig {
    /// Load configuration from the project directory, walking up to find `.clean-ctx.json`
    pub fn load(start_dir: &Path) -> Self {
        if let Some(config_path) = Self::find_config(start_dir) {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => {
                            eprintln!("[clean-ctx] Loaded config from: {}", config_path.display());
                            config
                        }
                        Err(e) => {
                            eprintln!("[clean-ctx] Warning: Failed to parse {}: {}", config_path.display(), e);
                            Self::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[clean-ctx] Warning: Failed to read {}: {}", config_path.display(), e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    /// Walk up from start_dir looking for `.clean-ctx.json`
    fn find_config(start_dir: &Path) -> Option<PathBuf> {
        let mut current = start_dir.to_path_buf();
        loop {
            let config_path = current.join(".clean-ctx.json");
            if config_path.exists() {
                return Some(config_path);
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    /// Check if a file path should be excluded
    pub fn is_excluded(&self, path: &str) -> bool {
        for pattern in &self.exclude_patterns {
            if path.contains(pattern.as_str()) {
                return true;
            }
        }
        false
    }

    /// Get fidelity override for a file extension
    pub fn get_fidelity_for_extension(&self, ext: &str) -> Option<&str> {
        self.fidelity_overrides.get(ext).map(|s| s.as_str())
    }

    /// Generate a default config file content
    pub fn default_config_content() -> String {
        let default = Self::default();
        serde_json::to_string_pretty(&default).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CleanCtxConfig::default();
        assert_eq!(config.default_fidelity, "low");
        assert!(config.diff_compression);
        assert!(config.type_aliases.is_empty());
    }

    #[test]
    fn test_exclusion() {
        let mut config = CleanCtxConfig::default();
        config.exclude_patterns.push("node_modules".to_string());
        config.exclude_patterns.push(".test.".to_string());
        
        assert!(config.is_excluded("src/node_modules/file.ts"));
        assert!(config.is_excluded("src/file.test.ts"));
        assert!(!config.is_excluded("src/file.ts"));
    }
}
