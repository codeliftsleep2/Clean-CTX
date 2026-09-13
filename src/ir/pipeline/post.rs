//! Pipeline passes that operate after framework meta-layer enrichment.

use std::sync::Mutex;

use super::{IRPass, PassContext, PassError};
use crate::cbm::bridge::GraphBridge;
use crate::ir::compiler::CompiledIR;
use crate::ir::compiler_methods::resolve_forward_aliases;
use crate::ir::inference_layer::InferenceLayer;
use crate::ir::program_graph::GraphBuilder;
use crate::ir::validator::{DefaultValidator, IRValidator};

/// Pass 4: Pattern recognition (consumptive compression).
pub struct PatternRecognitionPass;

impl PatternRecognitionPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PatternRecognitionPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for PatternRecognitionPass {
    fn name(&self) -> &str {
        "pattern_recognition"
    }

    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        for recognizer in &state.pattern_recognizers {
            state.instructions = recognizer.recognize(&state.instructions);
        }
        Ok(())
    }
}

/// Pass 5: Forward-declaration alias resolution.
pub struct AliasResolutionPass;

impl AliasResolutionPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AliasResolutionPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for AliasResolutionPass {
    fn name(&self) -> &str {
        "alias_resolution"
    }

    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        resolve_forward_aliases(&mut state.instructions);
        Ok(())
    }
}

/// Optional execution-semantics pass.
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
    fn name(&self) -> &str {
        "execution_semantics"
    }

    fn run(&self, _state: &mut PassContext) -> Result<(), PassError> {
        Ok(())
    }
}

/// Optional program-graph construction pass.
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
    fn name(&self) -> &str {
        "program_graph"
    }

    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        state.program_graph = Some(GraphBuilder::build_from_instructions(&state.instructions));
        Ok(())
    }
}

/// Optional inference-layer pass (CBM enrichment + derived analysis).
pub struct InferenceLayerPass {
    cbm_bridge: Mutex<Option<GraphBridge>>,
}

impl InferenceLayerPass {
    pub fn new() -> Self {
        Self {
            cbm_bridge: Mutex::new(None),
        }
    }

    pub fn with_cbm(bridge: Option<GraphBridge>) -> Self {
        Self {
            cbm_bridge: Mutex::new(bridge),
        }
    }
}

impl Default for InferenceLayerPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for InferenceLayerPass {
    fn name(&self) -> &str {
        "inference_layer"
    }

    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        let mut layer = InferenceLayer::new();
        let mut guard = self
            .cbm_bridge
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Err(error) = layer.enrich_from_cbm(guard.as_mut()) {
            eprintln!(
                "[clean-ctx-ir] CBM inference enrichment failed — continuing without enrichment: {error}"
            );
        }
        for edge in state.semantic_edges.drain(..) {
            layer.add_semantic_edge(edge);
        }
        state.inference_layer = Some(layer);
        Ok(())
    }
}

/// Pass 6: structural and consistency validation.
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
    fn name(&self) -> &str {
        "validation"
    }

    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        let ir = CompiledIR {
            file_id: state.file_id.clone(),
            instructions: state.instructions.clone(),
            version: 1,
        };
        let errors = DefaultValidator::new().validate(&ir);
        if errors.is_empty() {
            return Ok(());
        }
        Err(PassError {
            pass_name: self.name().to_string(),
            message: format!(
                "validation failed: {}",
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        })
    }
}
