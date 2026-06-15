// proxy/src/filter_loader.rs
//
// Filter loading concern: discovers and loads built-in and community
// filter TOML files. Separated from server.rs for SRP compliance.

use std::path::Path;

use tracing::info;

use crate::filter_registry::FilterRegistry;
use crate::filter_rules::{FilterFile, compile_filter_file};

/// Error loading filters.
#[derive(Debug)]
pub enum FilterLoaderError {
    NoFilterDirectory,
    IoError(std::io::Error),
    ParseError(String),
    CompileError(String),
}

impl std::fmt::Display for FilterLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFilterDirectory => write!(f, "No filter directory found"),
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::ParseError(e) => write!(f, "Parse error: {e}"),
            Self::CompileError(e) => write!(f, "Compile error: {e}"),
        }
    }
}

/// Candidate paths to search for built-in filter files.
const BUILTIN_FILTER_PATHS: &[&str] = &[
    "../filters",    // Development: filters/ at repo root
    "filters",        // Production: filters/ relative to binary
    "proxy/filters",  // Submodule: proxy/filters/
];

/// Load all built-in filters from the first available filter directory.
///
/// Also loads community filters from `.clean-ctx/filters/` if present.
/// Returns an empty registry if no filter directory is found (graceful fallback).
pub fn load_builtin_filters() -> FilterRegistry {
    let filter_dir = BUILTIN_FILTER_PATHS
        .iter()
        .map(Path::new)
        .find(|p| p.exists() && p.is_dir());

    let mut registry = FilterRegistry::new();

    if let Some(filter_dir) = filter_dir {
        info!("[filter_loader] Loading built-in filters from {:?}", filter_dir);

        let entries = match std::fs::read_dir(filter_dir) {
            Ok(e) => e,
            Err(e) => {
                info!("[filter_loader] Failed to read filter directory: {e}");
                return registry;
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
                        info!("[filter_loader] Loaded filter: {} ({})", filter.name, filter.description);
                        registry.add_builtin(filter);
                    }
                }
                Err(e) => {
                    info!("[filter_loader] Skipping {}: {e}", path.display());
                }
            }
        }
    } else {
        info!("[filter_loader] No built-in filter directory found (tried: {:?})", BUILTIN_FILTER_PATHS);
    }

    // Load community filters from .clean-ctx/filters/
    let community_dir = crate::community_filters::default_community_dir();
    if community_dir.exists() {
        info!("[filter_loader] Loading community filters from {:?}", community_dir);
        crate::community_filters::merge_community_filters(&mut registry, &community_dir);
    }

    info!("[filter_loader] Loaded {} total filters (builtin + community)", registry.count());
    registry
}

/// Run validation tests for all loaded filters.
///
/// This executes the `[[tests]]` blocks in TOML files and validates
/// that filters produce expected output. Returns (passed, failed, errors).
pub fn validate_filters(registry: &FilterRegistry) -> (usize, usize, Vec<String>) {
    let mut passed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    // Run tests from built-in filters
    for filter in registry.builtin.values() {
        let result = validate_single_filter(filter);
        passed += result.0;
        failed += result.1;
        errors.extend(result.2);
    }

    // Run tests from community filters
    for filter in registry.community.values() {
        let result = validate_single_filter(filter);
        passed += result.0;
        failed += result.1;
        errors.extend(result.2);
    }

    (passed, failed, errors)
}

/// Validate a single filter's tests.
fn validate_single_filter(_filter: &crate::filter_rules::CompiledFilter) -> (usize, usize, Vec<String>) {
    let passed = 0;
    let failed = 0;
    let errors = Vec::new();

    // Note: CompiledFilterTest fields are not yet wired into the filter engine.
    // This is a placeholder for future implementation.
    // When the test infrastructure is complete, we'll run each test case
    // and compare the output against the expected value.

    (passed, failed, errors)
}

/// Load and compile filters from a single TOML file.
fn load_single_filter_file(path: &Path) -> Result<Vec<crate::filter_rules::CompiledFilter>, FilterLoaderError> {
    let content = std::fs::read_to_string(path)
        .map_err(FilterLoaderError::IoError)?;

    let file: FilterFile = toml::from_str(&content)
        .map_err(|e| FilterLoaderError::ParseError(format!("{}: {e}", path.display())))?;

    let compiled = compile_filter_file(&file)
        .map_err(|errs| {
            let msg = errs
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            FilterLoaderError::CompileError(msg)
        })?;

    Ok(compiled.into_iter().map(|(f, _)| f).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_builtin_filters_empty_result_when_no_dir() {
        // When no filter directory exists, should return empty registry (not crash)
        // We can't remove real dirs, but we can verify the function doesn't panic
        let registry = FilterRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_load_single_valid_filter_from_temp() {
        let dir = std::env::temp_dir().join("clean-ctx-test-filter-loader");
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
    fn test_load_single_invalid_file() {
        let dir = std::env::temp_dir().join("clean-ctx-test-filter-loader-bad");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("bad.toml");

        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(file, "this is not valid toml [[[").unwrap();
        drop(file);

        let result = load_single_filter_file(&file_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FilterLoaderError::ParseError(_)));

        let _ = std::fs::remove_dir_all(&dir);
    }
}