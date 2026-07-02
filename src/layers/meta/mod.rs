// src/layers/meta/mod.rs
//
// MetaLayer trait + per-framework implementations.
// Each implementation is gated by its Cargo feature so the framework-specific
// code is only compiled when the feature is enabled.
//
// # Preferred pattern for new meta-layers
//
// New meta-layers SHOULD follow the decentralized .NET pattern:
//   - Define the MetaLayer struct + MetaLayer trait impl in the meta-layer's
//     own module (e.g. `src/dotnet_meta/mod.rs`)
//   - Register it in `src/layers/registry.rs` with a `#[cfg(feature = "...")]` guard
//   - The decentralized approach keeps the meta-layer's code co-located with
//     its detection, extraction, and marker logic
//
// Legacy: Angular and Spring Boot meta-layers have their trait impls directly
// in this file for historical reasons. New meta-layers should NOT follow
// this pattern.

use std::path::Path;
use crate::compression::Fidelity;
use crate::ir::compiler::CompiledIR;

/// A meta-layer that enriches compressed output with framework-specific
/// context (e.g. Angular decorators, Spring Boot annotations).
///
/// Meta-layers are purely additive: they append framework-specific markers
/// to the compressed output without modifying the base compression.
pub trait MetaLayer: Send + Sync {
    /// Unique identifier (e.g. "angular", "spring_boot").
    fn name(&self) -> &'static str;

    /// Check if this meta-layer applies to the given source file.
    /// Called before `enrich` to avoid unnecessary work.
    fn is_applicable(&self, source: &str, path: &Path) -> bool;

    /// Enrich the compressed output with framework-specific markers.
    ///
    /// # Arguments
    /// - `output`: The compressed output to append to (e.g. `Φ` block).
    /// - `source`: The full source text of the file being compressed.
    /// - `ir`: The compiled IR for this file (if available).
    /// - `fidelity`: The fidelity level controlling verbosity.
    fn enrich(&self, output: &mut String, source: &str, ir: &CompiledIR, fidelity: Fidelity);
}

// ── Angular Meta-Layer ────────────────────────────────────────────────

#[cfg(feature = "angular")]
pub struct AngularMetaLayer;

#[cfg(feature = "angular")]
impl AngularMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "angular")]
impl Default for AngularMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "angular")]
impl MetaLayer for AngularMetaLayer {
    fn name(&self) -> &'static str {
        "angular"
    }

    fn is_applicable(&self, source: &str, _path: &Path) -> bool {
        // Delegate to the existing Angular detection heuristic
        crate::angular_meta::detect::is_angular_file(source)
    }

    fn enrich(&self, output: &mut String, source: &str, ir: &CompiledIR, fidelity: Fidelity) {
        // Extract class names from the IR for compatibility
        let class_captures: Vec<String> = ir.instructions
            .iter()
            .filter_map(|op| {
                if let crate::ir::opcodes::CoreOp::DefClass(_, name) = op {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        
        if let Some(block) = crate::angular_meta::run_meta_layer(source, &class_captures, fidelity) {
            output.push_str(&block.render());
        }
    }
}

#[cfg(not(feature = "angular"))]
pub struct AngularMetaLayer;

#[cfg(not(feature = "angular"))]
impl AngularMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "angular"))]
impl Default for AngularMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "angular"))]
impl MetaLayer for AngularMetaLayer {
    fn name(&self) -> &'static str {
        "angular"
    }

    fn is_applicable(&self, _source: &str, _path: &Path) -> bool {
        false
    }

    fn enrich(&self, _output: &mut String, _source: &str, _ir: &CompiledIR, _fidelity: Fidelity) {
        // No-op when feature is disabled
    }
}

// ── Spring Boot Meta-Layer ────────────────────────────────────────────

#[cfg(feature = "spring_boot")]
pub struct SpringBootMetaLayer;

#[cfg(feature = "spring_boot")]
impl SpringBootMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "spring_boot")]
impl Default for SpringBootMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "spring_boot")]
impl MetaLayer for SpringBootMetaLayer {
    fn name(&self) -> &'static str {
        "spring_boot"
    }

    fn is_applicable(&self, source: &str, _path: &Path) -> bool {
        // Delegate to the existing Spring Boot detection heuristic
        crate::spring_meta::detect::is_spring_file(source)
    }

    fn enrich(&self, output: &mut String, source: &str, ir: &CompiledIR, fidelity: Fidelity) {
        // Extract class names from the IR for compatibility
        let class_captures: Vec<String> = ir.instructions
            .iter()
            .filter_map(|op| {
                if let crate::ir::opcodes::CoreOp::DefClass(_, name) = op {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        
        if let Some(block) = crate::spring_meta::run_meta_layer(source, &class_captures, fidelity) {
            output.push_str(&block.render());
        }
    }
}

#[cfg(not(feature = "spring_boot"))]
pub struct SpringBootMetaLayer;

#[cfg(not(feature = "spring_boot"))]
impl SpringBootMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "spring_boot"))]
impl Default for SpringBootMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "spring_boot"))]
impl MetaLayer for SpringBootMetaLayer {
    fn name(&self) -> &'static str {
        "spring_boot"
    }

    fn is_applicable(&self, _source: &str, _path: &Path) -> bool {
        false
    }

    fn enrich(&self, _output: &mut String, _source: &str, _ir: &CompiledIR, _fidelity: Fidelity) {
        // No-op when feature is disabled
    }
}
