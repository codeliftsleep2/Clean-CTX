// src/layers/language/mod.rs
//
// LanguageLayer trait + per-language implementations.
// Each implementation is gated by its Cargo feature so the tree-sitter
// grammar is only compiled when the feature is enabled.

use tree_sitter::Language;
use crate::compression::Fidelity;

/// Result of compiling source through a language layer.
/// Mirrors the existing pipeline output so callers don't need to change.
pub struct CompileOutput {
    pub body: String,
    pub class_count: usize,
    pub method_count: usize,
    pub import_count: usize,
}

/// A language layer that can parse source code and produce compressed output.
///
/// Implementations are feature-gated: when the feature is disabled, the
/// corresponding tree-sitter grammar is not linked, and `language_ptr()`
/// returns `None`.
pub trait LanguageLayer: Send + Sync {
    /// Unique identifier (e.g. "typescript", "csharp", "rust", "java").
    fn name(&self) -> &'static str;

    /// File extensions this layer handles (e.g. `["ts", "tsx"]`).
    fn extensions(&self) -> &'static [&'static str];

    /// Tree-sitter language pointer, or `None` if the grammar is not available.
    fn language_ptr(&self) -> Option<Language>;

    /// Compile source code into compressed output.
    fn compile(&self, source: &str, fidelity: Fidelity) -> Result<CompileOutput, CompileError>;
}

/// Errors that can occur during language layer compilation.
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("tree-sitter grammar not available (feature disabled)")]
    GrammarUnavailable,
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("compression error: {0}")]
    CompressionError(String),
}

// ── TypeScript Layer ──────────────────────────────────────────────────

#[cfg(feature = "typescript")]
pub struct TypeScriptLayer;

#[cfg(feature = "typescript")]
impl TypeScriptLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "typescript")]
impl Default for TypeScriptLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "typescript")]
impl LanguageLayer for TypeScriptLayer {
    fn name(&self) -> &'static str {
        "typescript"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["ts", "tsx"]
    }

    fn language_ptr(&self) -> Option<Language> {
        crate::compression::language::safe_typescript_language()
    }

    fn compile(&self, source: &str, fidelity: Fidelity) -> Result<CompileOutput, CompileError> {
        // Delegate to the existing pipeline
        let result = crate::compression::pipeline::compress_text(
            source,
            "ts",
            fidelity,
            "ts",
            None,
        ).map_err(|e| CompileError::CompressionError(e.to_string()))?;

        let (body_lines, _full_output) = result;
        let body = body_lines.join("\n");

        // Count structures from body
        let class_count = body_lines.iter().filter(|l| l.contains("class ")).count();
        let method_count = body_lines.iter().filter(|l| l.contains("fn ") || l.contains("def ")).count();
        let import_count = body_lines.iter().filter(|l| l.starts_with("import ") || l.starts_with("from ")).count();

        Ok(CompileOutput {
            body,
            class_count,
            method_count,
            import_count,
        })
    }
}

#[cfg(not(feature = "typescript"))]
pub struct TypeScriptLayer;

#[cfg(not(feature = "typescript"))]
impl TypeScriptLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "typescript"))]
impl Default for TypeScriptLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "typescript"))]
impl LanguageLayer for TypeScriptLayer {
    fn name(&self) -> &'static str {
        "typescript"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["ts", "tsx"]
    }

    fn language_ptr(&self) -> Option<Language> {
        None
    }

    fn compile(&self, _source: &str, _fidelity: Fidelity) -> Result<CompileOutput, CompileError> {
        Err(CompileError::GrammarUnavailable)
    }
}

// ── C# Layer ──────────────────────────────────────────────────────────

#[cfg(feature = "csharp")]
pub struct CSharpLayer;

#[cfg(feature = "csharp")]
impl CSharpLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "csharp")]
impl Default for CSharpLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "csharp")]
impl LanguageLayer for CSharpLayer {
    fn name(&self) -> &'static str {
        "csharp"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["cs"]
    }

    fn language_ptr(&self) -> Option<Language> {
        crate::compression::language::safe_csharp_language()
    }

    fn compile(&self, source: &str, fidelity: Fidelity) -> Result<CompileOutput, CompileError> {
        let result = crate::compression::pipeline::compress_text(
            source,
            "cs",
            fidelity,
            "cs",
            None,
        ).map_err(|e| CompileError::CompressionError(e.to_string()))?;

        let (body_lines, _full_output) = result;
        let body = body_lines.join("\n");

        Ok(CompileOutput {
            body,
            class_count: 0,
            method_count: 0,
            import_count: 0,
        })
    }
}

#[cfg(not(feature = "csharp"))]
pub struct CSharpLayer;

#[cfg(not(feature = "csharp"))]
impl CSharpLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "csharp"))]
impl Default for CSharpLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "csharp"))]
impl LanguageLayer for CSharpLayer {
    fn name(&self) -> &'static str {
        "csharp"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["cs"]
    }

    fn language_ptr(&self) -> Option<Language> {
        None
    }

    fn compile(&self, _source: &str, _fidelity: Fidelity) -> Result<CompileOutput, CompileError> {
        Err(CompileError::GrammarUnavailable)
    }
}

// ── Rust Layer ────────────────────────────────────────────────────────

#[cfg(feature = "rust")]
pub struct RustLayer;

#[cfg(feature = "rust")]
impl RustLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "rust")]
impl Default for RustLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rust")]
impl LanguageLayer for RustLayer {
    fn name(&self) -> &'static str {
        "rust"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn language_ptr(&self) -> Option<Language> {
        crate::compression::language::safe_rust_language()
    }

    fn compile(&self, source: &str, fidelity: Fidelity) -> Result<CompileOutput, CompileError> {
        let result = crate::compression::pipeline::compress_text(
            source,
            "rs",
            fidelity,
            "rs",
            None,
        ).map_err(|e| CompileError::CompressionError(e.to_string()))?;

        let (body_lines, _full_output) = result;
        let body = body_lines.join("\n");

        Ok(CompileOutput {
            body,
            class_count: 0,
            method_count: 0,
            import_count: 0,
        })
    }
}

#[cfg(not(feature = "rust"))]
pub struct RustLayer;

#[cfg(not(feature = "rust"))]
impl RustLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "rust"))]
impl Default for RustLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "rust"))]
impl LanguageLayer for RustLayer {
    fn name(&self) -> &'static str {
        "rust"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn language_ptr(&self) -> Option<Language> {
        None
    }

    fn compile(&self, _source: &str, _fidelity: Fidelity) -> Result<CompileOutput, CompileError> {
        Err(CompileError::GrammarUnavailable)
    }
}

// ── Java Layer ────────────────────────────────────────────────────────

#[cfg(feature = "java")]
pub struct JavaLayer;

#[cfg(feature = "java")]
impl JavaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "java")]
impl Default for JavaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "java")]
impl LanguageLayer for JavaLayer {
    fn name(&self) -> &'static str {
        "java"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["java"]
    }

    fn language_ptr(&self) -> Option<Language> {
        crate::compression::language::safe_java_language()
    }

    fn compile(&self, source: &str, fidelity: Fidelity) -> Result<CompileOutput, CompileError> {
        let result = crate::compression::pipeline::compress_text(
            source,
            "java",
            fidelity,
            "java",
            None,
        ).map_err(|e| CompileError::CompressionError(e.to_string()))?;

        let (body_lines, _full_output) = result;
        let body = body_lines.join("\n");

        Ok(CompileOutput {
            body,
            class_count: 0,
            method_count: 0,
            import_count: 0,
        })
    }
}

#[cfg(not(feature = "java"))]
pub struct JavaLayer;

#[cfg(not(feature = "java"))]
impl JavaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "java"))]
impl Default for JavaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "java"))]
impl LanguageLayer for JavaLayer {
    fn name(&self) -> &'static str {
        "java"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["java"]
    }

    fn language_ptr(&self) -> Option<Language> {
        None
    }

    fn compile(&self, _source: &str, _fidelity: Fidelity) -> Result<CompileOutput, CompileError> {
        Err(CompileError::GrammarUnavailable)
    }
}