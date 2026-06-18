// src/spring_meta/bundler.rs
//
// Layer bundler — Tier 2 of the Spring Boot Meta-Layer.
//
// Resolves the Spring Boot layer for a given class:
// - Controller → Service → Repository (typical call chain)
// - Configuration (standalone)
// - Service/Repository (middle layer)
//
// This enables the LLM to see "these classes form a layered architecture"
// without burning tokens on boilerplate.
//
// Workspace-mode only.

use std::path::{Path, PathBuf};

/// A resolved layer bundle for a Spring Boot component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerBundle {
    /// The component name (filename without extension).
    pub name: String,
    /// The absolute path to the file.
    pub path: PathBuf,
    /// The Spring layer this class belongs to.
    pub layer: SpringLayer,
}

/// Spring Boot architectural layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpringLayer {
    Controller,
    Service,
    Repository,
    Configuration,
    Unknown,
}

impl SpringLayer {
    /// Returns the layer priority for ordering (lower = higher in architecture).
    pub fn priority(&self) -> u8 {
        match self {
            SpringLayer::Controller => 1,
            SpringLayer::Service => 2,
            SpringLayer::Repository => 3,
            SpringLayer::Configuration => 4,
            SpringLayer::Unknown => 5,
        }
    }
}

/// Check if the given path looks like a Spring Boot Java file.
pub fn is_spring_java(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "java")
        .unwrap_or(false)
}

/// Resolve the Spring Boot layer for a Java file based on its annotations.
pub fn resolve_layer(source: &str) -> SpringLayer {
    if source.contains("@RestController") || source.contains("@Controller") {
        return SpringLayer::Controller;
    }
    if source.contains("@Service") {
        return SpringLayer::Service;
    }
    if source.contains("@Repository") {
        return SpringLayer::Repository;
    }
    if source.contains("@Configuration") || source.contains("@SpringBootApplication") {
        return SpringLayer::Configuration;
    }
    SpringLayer::Unknown
}

/// Resolve the layer bundle for a Spring Boot Java file.
pub fn resolve_bundle(path: &Path, source: &str) -> Option<LayerBundle> {
    if !is_spring_java(path) {
        return None;
    }

    let name = path.file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let layer = resolve_layer(source);

    Some(LayerBundle {
        name,
        path: path.to_path_buf(),
        layer,
    })
}

#[cfg(test)]
#[path = "../tests/spring_meta/bundler_tests.rs"]
mod tests;
