// proxy/src/community_filters.rs
//
// Community filter loading: discovers and loads user-defined filter TOML files
// from `.clean-ctx/filters/` directory.
//
// This allows users to add custom filters without modifying the proxy code.
// Community filters are loaded at startup and merged with built-in filters.

use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::filter_registry::FilterRegistry;
use crate::filter_rules::{compile_filter_file, FilterFile};

/// Error loading community filters.
#[derive(Debug)]
pub enum CommunityFilterError {
    Io(std::io::Error),
    Parse(String),
    Compile(String),
}

impl std::fmt::Display for CommunityFilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Parse(e) => write!(f, "Parse error: {e}"),
            Self::Compile(e) => write!(f, "Compile error: {e}"),
        }
    }
}

/// Result of loading community filters.
pub struct CommunityFiltersResult {
    pub filters: Vec<crate::filter_rules::CompiledFilter>,
    pub errors: Vec<CommunityFilterError>,
}

/// Get the default community filter directory.
pub fn default_community_dir() -> PathBuf {
    PathBuf::from(".clean-ctx/filters")
}

/// Load all community filters from the given directory.
pub fn load_community_filters(dir: &Path) -> CommunityFiltersResult {
    let mut result = CommunityFiltersResult {
        filters: Vec::new(),
        errors: Vec::new(),
    };

    if !dir.exists() {
        info!(
            "[community_filters] No community filter directory found at {:?}",
            dir
        );
        return result;
    }

    info!(
        "[community_filters] Loading community filters from {:?}",
        dir
    );

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            result.errors.push(CommunityFilterError::Io(e));
            return result;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }

        match load_single_filter_file(&path) {
            Ok(filters) => {
                for filter in filters {
                    info!(
                        "[community_filters] Loaded filter: {} ({})",
                        filter.name, filter.description
                    );
                    result.filters.push(filter);
                }
            }
            Err(e) => {
                warn!("[community_filters] Skipping {}: {e}", path.display());
                result.errors.push(e);
            }
        }
    }

    info!(
        "[community_filters] Loaded {} community filters",
        result.filters.len()
    );
    result
}

/// Load and compile filters from a single TOML file.
fn load_single_filter_file(
    path: &Path,
) -> Result<Vec<crate::filter_rules::CompiledFilter>, CommunityFilterError> {
    let content = std::fs::read_to_string(path).map_err(CommunityFilterError::Io)?;

    let file: FilterFile = toml::from_str(&content)
        .map_err(|e| CommunityFilterError::Parse(format!("{}: {e}", path.display())))?;

    let compiled = compile_filter_file(&file).map_err(|errs| {
        let msg = errs
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        CommunityFilterError::Compile(msg)
    })?;

    Ok(compiled.into_iter().map(|(f, _)| f).collect())
}

/// Merge community filters into a registry.
pub fn merge_community_filters(registry: &mut FilterRegistry, community_dir: &Path) {
    let result = load_community_filters(community_dir);
    for filter in result.filters {
        registry.add_community(filter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_community_filters_nonexistent_dir() {
        let dir = Path::new("/nonexistent/path");
        let result = load_community_filters(dir);
        assert!(result.filters.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_load_community_filters_empty_dir() {
        let dir = std::env::temp_dir().join("clean-ctx-test-community-filters-empty");
        let _ = std::fs::create_dir_all(&dir);
        let result = load_community_filters(&dir);
        assert!(result.filters.is_empty());
        assert!(result.errors.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_single_valid_filter() {
        let dir = std::env::temp_dir().join("clean-ctx-test-community-filters-valid");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("test.toml");

        let toml_content = r#"
[filters.test-filter]
description = "Test filter"
match_command = "^test"
        "#;

        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(file, "{}", toml_content).unwrap();
        drop(file);

        let result = load_single_filter_file(&file_path);
        assert!(result.is_ok(), "Failed: {:?}", result.err());
        let filters = result.unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].name, "test-filter");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_single_invalid_toml() {
        let dir = std::env::temp_dir().join("clean-ctx-test-community-filters-invalid");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("bad.toml");

        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(file, "this is not valid toml [[[").unwrap();
        drop(file);

        let result = load_single_filter_file(&file_path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommunityFilterError::Parse(_)
        ));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_default_community_dir() {
        let dir = default_community_dir();
        assert_eq!(dir, PathBuf::from(".clean-ctx/filters"));
    }
}
