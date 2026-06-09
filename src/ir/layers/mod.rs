// src/ir/layers/mod.rs
//
// Phase F: Layered Encoding
// Separate language-specific and framework-specific logic into pluggable
// layers that emit additional ops on top of the Core IR.
//
// 4-Layer Architecture:
//   Layer 1: Core IR (language-agnostic) — in opcodes.rs
//   Layer 2: Language Layer (TS, C#, etc.) — LanguageLayer trait
//   Layer 3: Meta-Layer (framework-specific) — MetaLayer trait
//   Layer 4: Application Patterns (positional encoding) — PatternRecognizer trait

use super::opcodes::CoreOp;
use super::symbol_table::GlobalSymbolTable;
use crate::compression::Fidelity;

pub mod typescript;
pub mod csharp;
pub mod angular;
pub mod patterns;

/// Context passed to layer processing functions.
/// Provides current class/method context and symbol table access.
///
/// F-46: `LayerContext` is constructed fresh per `IRCompiler::compile`
/// call. The `GlobalSymbolTable` is owned (not borrowed), so mutations
/// inside `process_capture` are visible to subsequent layers but not
/// to the caller's copy. This is intentional for the current single-shot
/// compile path. A future enhancement could propagate a `&mut
/// GlobalSymbolTable` from `McpState` through to the compiler for
/// cross-compile symbol registration.
#[derive(Debug, Clone)]
pub struct LayerContext {
    /// Current class ID (set when processing a class capture)
    pub current_class: Option<String>,
    /// Current class original name (set when processing a class capture)
    pub current_class_name: Option<String>,
    /// Current method ID (set when processing a method capture)
    pub current_method: Option<String>,
    /// Current method original name (set when processing a method capture)
    pub current_method_name: Option<String>,
    /// Global symbol table reference
    pub symbol_table: GlobalSymbolTable,
    /// Fidelity level
    pub fidelity: Fidelity,
    /// Source code being compiled
    pub source: String,
}

impl LayerContext {
    pub fn new(source: &str, fidelity: Fidelity) -> Self {
        Self {
            current_class: None,
            current_class_name: None,
            current_method: None,
            current_method_name: None,
            symbol_table: GlobalSymbolTable::new(),
            fidelity,
            source: source.to_string(),
        }
    }

    /// Get a mutable reference to the symbol table.
    /// Allows the compiler and layers to register/resolve class aliases.
    ///
    /// NF-05: Exposed so `IRCompiler::compile` can register class aliases
    /// after emitting `DefClass`, enabling language layers (TypeScript, C#)
    /// to find the alias of extended/implemented interfaces via `alias_for`.
    pub fn symbol_table_mut(&mut self) -> &mut GlobalSymbolTable {
        &mut self.symbol_table
    }
}

/// Language-specific IR layer (Layer 2).
/// Translates language-specific captures into additional IR instructions.
pub trait LanguageLayer {
    /// Language name (e.g., "typescript", "csharp")
    fn name(&self) -> &str;

    /// Process a capture and emit additional IR instructions.
    /// Called for each capture from the tree-sitter pipeline.
    fn process_capture(
        &mut self,
        capture_name: &str,
        raw_text: &str,
        context: &mut LayerContext,
    ) -> Vec<CoreOp>;

    /// Post-processing: emit any cross-cutting instructions
    /// after all captures have been processed.
    fn finalize(&mut self, context: &mut LayerContext) -> Vec<CoreOp> {
        let _ = context;
        Vec::new()
    }
}

/// Framework-specific IR layer (Layer 3).
/// Extracts framework patterns (decorators, annotations, etc.)
pub trait MetaLayer {
    /// Framework name (e.g., "angular", "react", "ngrx")
    fn name(&self) -> &str;

    /// Extract framework-specific ops from the full source and class list.
    fn extract(
        &mut self,
        source: &str,
        classes: &[String],
        fidelity: Fidelity,
    ) -> Vec<CoreOp>;
}

/// Pattern recognizer (Layer 4).
/// Identifies common code patterns and compresses them to single ops.
pub trait PatternRecognizer {
    /// Analyze the instruction stream and compress recognized patterns.
    fn recognize(&self, instructions: &[CoreOp]) -> Vec<CoreOp>;
}

#[cfg(test)]
#[path = "../../tests/ir/layers/mod.rs"]
mod tests;
