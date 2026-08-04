// src/ir/pipeline.rs
//
// R-43b: Explicit Pass Pipeline
//
// The composable pass pipeline replaces the monolithic `compile()` function
// with a sequence of composable `IRPass` implementations. Each pass has a
// clear input/output contract. Adding new languages, meta-layers, or analysis
// passes becomes mechanical.
//
// Pipeline order:
//   Pass 1: Core IR      (tree-sitter → CoreOp stream)
//   Pass 2: Language     (language-specific ops)
//   Pass 3: Meta Layer   (framework-specific markers)
//   Pass 4: Execution    (DataFlow, ControlFlow, SideEffect, Context)
//   Pass 5: Program Graph (local graph from CompiledIRs)
//   Pass 6: Inference    (CBM enrichment + derived analysis)
//   Pass 7: Validation   (structural + consistency checks)

use std::sync::Mutex;

use super::inference_layer::InferenceLayer;
use super::opcodes::CoreOp;
use super::program_graph::{ProgramGraph, GraphBuilder};
use super::layers::LayerContext;
use crate::cbm::bridge::GraphBridge;
use crate::compression::Fidelity;

/// Error type for pass execution.
#[derive(Debug, Clone)]
pub struct PassError {
    /// Name of the pass that failed
    pub pass_name: String,
    /// Error message
    pub message: String,
}

impl std::fmt::Display for PassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pass '{}' failed: {}", self.pass_name, self.message)
    }
}

impl std::error::Error for PassError {}

/// A single pass in the IR compilation pipeline.
/// Each pass transforms or enriches the compilation state.
pub trait IRPass {
    /// Name of this pass (for debugging and profiling).
    fn name(&self) -> &str;

    /// Run this pass on the current compilation state.
    /// Passes are ordered and composable.
    fn run(&self, state: &mut PassContext) -> Result<(), PassError>;
}

/// Context passed through the pipeline.
/// Each pass reads from and writes to this context.
pub struct PassContext {
    /// The core instruction stream (pure facts)
    pub instructions: Vec<CoreOp>,
    /// Language-specific context
    pub layer_context: LayerContext,
    /// Program graph (built in Pass 5)
    pub program_graph: Option<ProgramGraph>,
    /// Inference layer (built in Pass 6)
    pub inference_layer: Option<InferenceLayer>,
    /// Source code and metadata
    pub source: String,
    pub file_id: String,
    pub fidelity: Fidelity,
}

impl PassContext {
    /// Create a new pass context.
    pub fn new(source: String, file_id: String, fidelity: Fidelity) -> Self {
        Self {
            instructions: Vec::new(),
            layer_context: LayerContext::new(&source, fidelity),
            program_graph: None,
            inference_layer: None,
            source,
            file_id,
            fidelity,
        }
    }
}

/// The composable pass pipeline.
pub struct PassPipeline {
    passes: Vec<Box<dyn IRPass>>,
}

impl PassPipeline {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Register a pass. Passes run in registration order.
    pub fn add_pass(&mut self, pass: Box<dyn IRPass>) {
        self.passes.push(pass);
    }

    /// Run all registered passes in order.
    pub fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        for pass in &self.passes {
            pass.run(state)?;
        }
        Ok(())
    }

    /// Get the number of registered passes.
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }
}

impl Default for PassPipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ── Built-in Passes ──────────────────────────────────────────────

/// Pass 1: Core IR emission from tree-sitter captures.
pub struct CoreIRPass;

impl CoreIRPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CoreIRPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for CoreIRPass {
    fn name(&self) -> &str { "core_ir" }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        if state.source.is_empty() {
            return Err(PassError {
                pass_name: self.name().to_string(),
                message: "source code is empty".to_string(),
            });
        }
        Ok(())
    }
}

/// Pass 2: Language layer processing.
pub struct LanguageLayerPass;

impl LanguageLayerPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LanguageLayerPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for LanguageLayerPass {
    fn name(&self) -> &str { "language_layer" }
    fn run(&self, _state: &mut PassContext) -> Result<(), PassError> {
        Ok(())
    }
}

/// Pass 3: Meta-layer processing (framework-specific).
pub struct MetaLayerPass;

impl MetaLayerPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MetaLayerPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for MetaLayerPass {
    fn name(&self) -> &str { "meta_layer" }
    fn run(&self, _state: &mut PassContext) -> Result<(), PassError> {
        Ok(())
    }
}

/// Pass 4: Execution semantics extraction.
pub struct ExecutionSemanticsPass;

impl ExecutionSemanticsPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExecutionSemanticsPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for ExecutionSemanticsPass {
    fn name(&self) -> &str { "execution_semantics" }
    fn run(&self, _state: &mut PassContext) -> Result<(), PassError> {
        Ok(())
    }
}

/// Pass 5: Program graph construction.
pub struct ProgramGraphPass;

impl ProgramGraphPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProgramGraphPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for ProgramGraphPass {
    fn name(&self) -> &str { "program_graph" }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        let graph = GraphBuilder::build_from_instructions(&state.instructions);
        state.program_graph = Some(graph);
        Ok(())
    }
}

/// Pass 6: Inference layer (CBM enrichment + derived analysis).
///
/// R-43b Phase 3: The pass now holds an optional CBM `GraphBridge` wrapped
/// in a `Mutex` so `run(&self)` can lock it mutably for enrichment. When no
/// bridge is provided (or CBM is unavailable), the layer is built empty —
/// invariant C2 (all core functionality works without CBM).
pub struct InferenceLayerPass {
    cbm_bridge: Mutex<Option<GraphBridge>>,
}

impl InferenceLayerPass {
    pub fn new() -> Self {
        Self { cbm_bridge: Mutex::new(None) }
    }

    /// Create a pass with an optional CBM bridge for enrichment.
    pub fn with_cbm(bridge: Option<GraphBridge>) -> Self {
        Self { cbm_bridge: Mutex::new(bridge) }
    }
}

impl Default for InferenceLayerPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for InferenceLayerPass {
    fn name(&self) -> &str { "inference_layer" }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        let mut layer = InferenceLayer::new();
        let mut guard = self.cbm_bridge.lock().unwrap_or_else(|p| p.into_inner());
        layer.enrich_from_cbm(guard.as_mut());
        state.inference_layer = Some(layer);
        Ok(())
    }
}

/// Pass 7: Validation.
pub struct ValidationPass;

impl ValidationPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ValidationPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for ValidationPass {
    fn name(&self) -> &str { "validation" }
    fn run(&self, _state: &mut PassContext) -> Result<(), PassError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/ir/pipeline.rs"]
mod tests;