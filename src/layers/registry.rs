// src/layers/registry.rs
//
// LayerRegistry — collects all enabled language and meta layers at startup
// and dispatches to them at runtime.
//
// When a Cargo feature is disabled, the corresponding layer is not registered,
// and the tree-sitter grammar is not linked in the binary.

use std::sync::OnceLock;
use crate::compression::Fidelity;
use crate::layers::language::LanguageLayer;
use crate::layers::meta::MetaLayer;

/// Global registry, initialized once per process.
static LAYER_REGISTRY: OnceLock<LayerRegistry> = OnceLock::new();

/// Registry of all enabled language and meta layers.
///
/// Built at startup from the available Cargo features. Provides dispatch
/// methods for language detection, compilation, and meta-layer enrichment.
pub struct LayerRegistry {
    languages: Vec<Box<dyn LanguageLayer>>,
    meta_layers: Vec<Box<dyn MetaLayer>>,
}

impl Default for LayerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerRegistry {
    /// Build a new registry from the currently enabled Cargo features.
    ///
    /// This is called once at server startup and cached in a `OnceLock`.
    pub fn new() -> Self {
        let mut reg = LayerRegistry {
            languages: Vec::new(),
            meta_layers: Vec::new(),
        };

        // Language layers (always available when feature is enabled)
        #[cfg(feature = "typescript")]
        reg.languages.push(Box::new(crate::layers::language::TypeScriptLayer::new()));

        #[cfg(feature = "csharp")]
        reg.languages.push(Box::new(crate::layers::language::CSharpLayer::new()));

        #[cfg(feature = "rust")]
        reg.languages.push(Box::new(crate::layers::language::RustLayer::new()));

        #[cfg(feature = "java")]
        reg.languages.push(Box::new(crate::layers::language::JavaLayer::new()));

        // Meta layers (only available when both the meta feature and base language are enabled)
        #[cfg(feature = "angular")]
        reg.meta_layers.push(Box::new(crate::layers::meta::AngularMetaLayer::new()));

        #[cfg(feature = "spring_boot")]
        reg.meta_layers.push(Box::new(crate::layers::meta::SpringBootMetaLayer::new()));

        #[cfg(feature = "dotnet")]
        reg.meta_layers.push(Box::new(crate::dotnet_meta::DotNetMetaLayer::new()));

        reg
    }

    /// Get or initialize the global registry.
    pub fn global() -> &'static Self {
        LAYER_REGISTRY.get_or_init(Self::new)
    }

    /// Find a language layer by name.
    pub fn language_layer(&self, name: &str) -> Option<&dyn LanguageLayer> {
        self.languages.iter().find(|l| l.name() == name).map(|l| l.as_ref())
    }

    /// Find a language layer by file extension.
    pub fn language_layer_for_extension(&self, ext: &str) -> Option<&dyn LanguageLayer> {
        self.languages.iter().find(|l| l.extensions().contains(&ext)).map(|l| l.as_ref())
    }

    /// Get all enabled language layers.
    pub fn language_layers(&self) -> &[Box<dyn LanguageLayer>] {
        &self.languages
    }

    /// Get all enabled meta layers.
    pub fn meta_layers(&self) -> &[Box<dyn MetaLayer>] {
        &self.meta_layers
    }

    /// Run all applicable meta-layers on the given source and class captures.
    ///
    /// This is the pipeline-compatible entry point. Each meta-layer checks
    /// `is_applicable` and, if true, calls `enrich` to append framework-specific
    /// markers to the output.
    ///
    /// The `ir` parameter provides class captures from the compiled IR.
    /// When called from the text pipeline, this may be an empty IR — meta-layers
    /// that need class names should extract them from `class_captures` directly.
    ///
    /// Returns a vector of `(layer_name, block_text)` tuples for the caller
    /// to integrate into the compressed output.
    pub fn run_meta_layers_pipeline(
        &self,
        source: &str,
        class_captures: &[String],
        fidelity: Fidelity,
    ) -> Vec<(String, String)> {
        let mut results = Vec::new();
        
        for layer in &self.meta_layers {
            // Use trait-based dispatch: check if this layer applies to the source
            if layer.is_applicable(source, std::path::Path::new("")) {
                let mut output = String::new();
                // Build a minimal CompiledIR with the class captures so meta-layers
                // that extract class names from instructions still work.
                let class_instructions: Vec<crate::ir::opcodes::CoreOp> = class_captures
                    .iter()
                    .map(|name| crate::ir::opcodes::CoreOp::DefClass(String::new(), name.clone()))
                    .collect();
                let ir = crate::ir::compiler::CompiledIR {
                    file_id: String::new(),
                    instructions: class_instructions,
                    version: 1,
                };
                // Pass the real source code so detection and extraction work correctly
                layer.enrich(&mut output, source, &ir, fidelity);
                if !output.is_empty() {
                    results.push((layer.name().to_string(), output));
                }
            }
        }
        
        results
    }

    /// Check if a specific meta-layer is enabled.
    pub fn has_meta_layer(&self, name: &str) -> bool {
        self.meta_layers.iter().any(|l| l.name() == name)
    }

    /// Get a list of enabled language names (for diagnostics/CLI).
    pub fn enabled_language_names(&self) -> Vec<&'static str> {
        self.languages.iter().map(|l| l.name()).collect()
    }

    /// Get a list of enabled meta-layer names (for diagnostics/CLI).
    pub fn enabled_meta_layer_names(&self) -> Vec<&'static str> {
        self.meta_layers.iter().map(|l| l.name()).collect()
    }
}