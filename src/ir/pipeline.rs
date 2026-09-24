// src/ir/pipeline.rs
//
// R-43b: Explicit Pass Pipeline
//
// This pipeline is now the ACTIVE production compilation path.
// Production compilation occurs through PassPipeline, which replaces
// the previous monolithic IRCompiler::compile_inner() implementation.
//
// The composable pass pipeline was designed to replace the monolithic
// `compile()` function with a sequence of composable IRPass implementations.
// Each pass has a clear input/output contract. Adding new languages,
// meta-layers, or analysis passes becomes mechanical.
//
// Pipeline order (default production pipeline):
//   Pass 1: Core IR            (tree-sitter → CoreOp stream + per-capture language dispatch)
//   Pass 2: Language Finalize  (language-layer finalize())
//   Pass 3: Meta Layer         (framework-specific markers)
//   Pass 4: Pattern Recognition (consumptive pattern compression)
//   Pass 5: Alias Resolution   (forward-declaration alias resolution)
//   Pass 6: Validation         (structural + consistency checks)
//
// Optional passes (NOT part of the default production pipeline):
//   ExecutionSemanticsPass  — execution semantics are language-specific, extracted inside language layers
//   ProgramGraphPass        — on-demand local program graph construction
//   InferenceLayerPass      — on-demand CBM enrichment + derived analysis

use super::calls::{CallProducer, CallableScope};
use super::inference_layer::InferenceLayer;
use super::layers::{LanguageLayer, LayerContext, PatternRecognizer};
use super::opcodes::*;
use super::program_graph::ProgramGraph;
use crate::compaction::modifiers::{MODIFIERS_LOW, strip_csharp_attributes, strip_modifiers};
use crate::compaction::signature::split_parameters;
use crate::compression::Fidelity;
use crate::compression::capture_pipeline::CapEntry;
use crate::layers::meta::semantic::SemanticEdge;

mod core;
mod meta_layer;
mod post;
mod signature;

pub use core::{CoreIRPass, LanguageLayerPass};
pub use meta_layer::MetaLayerPass;
pub use post::{
    AliasResolutionPass, ExecutionSemanticsPass, InferenceLayerPass, PatternRecognitionPass,
    ProgramGraphPass, ValidationPass,
};
// Declaration-level helpers relocated to `pipeline/signature.rs`; the
// established public paths (`crate::ir::pipeline::{MethodSig,
// find_body_start_in, locate_method_body}`) are re-exported here so every
// existing consumer resolves unchanged.
pub use signature::MethodSig;
use signature::parse_method_sig;
pub(crate) use signature::{find_body_start_in, locate_method_body};

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

/// Active nested-type scope: the class id plus the source span of the
/// type declaration that owns it.
#[derive(Debug, Clone)]
pub struct TypeScope {
    pub owner: TypeOwner,
    /// Byte offset one past the end of the type declaration node.
    /// Scopes are pruned once the walk passes this offset.
    pub end_byte: usize,
}

#[derive(Debug, Clone)]
pub enum TypeOwner {
    Class(String),
    Interface(String),
}

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
    /// Meta-layer semantic edges accumulated by Pass 3 (MetaLayerPass) and
    /// consumed by the inference layer (Pass 8; semantic plan Phase 2).
    pub semantic_edges: Vec<SemanticEdge>,
    /// Source code and metadata
    pub source: String,
    pub file_id: String,
    /// Durable canonical file identity (for EntityRef.file provenance).
    /// Set by compile_inner; distinct from file_id (which is αN alias).
    pub canonical_path: Option<String>,
    pub fidelity: Fidelity,
    /// Monotonic instruction ID counter.
    pub id_counter: u64,
    /// Language layers (Layer 2) — mutable per-compilation state.
    pub language_layers: Vec<Box<dyn LanguageLayer>>,
    /// Pattern recognizers (Layer 4).
    pub pattern_recognizers: Vec<Box<dyn PatternRecognizer>>,
    /// Tree-sitter captures produced by the capture pipeline.
    pub captures: Vec<CapEntry>,
    /// Current method being processed (F-27: O(1) tracking).
    pub current_method: Option<String>,
    /// Current method's accumulated typed control summaries (F-28).
    pub current_control_summaries: Vec<ControlSummary>,
    /// Current class ID (set when processing a class capture).
    /// Nested-type aware: mirrors the innermost entry of `type_scopes`.
    pub current_class: Option<String>,
    pub current_interface: Option<String>,
    /// Open type-declaration scopes keyed by source span.
    /// A declaration is owned by the innermost scope whose
    /// `[start_byte, end_byte)` window contains it.
    pub type_scopes: Vec<TypeScope>,
    /// Open callable-declaration scopes keyed by source span.
    /// An invocation is owned by the innermost callable whose
    /// `[start_byte, end_byte)` window contains it (native call facts).
    pub callable_scopes: Vec<CallableScope>,
    /// Active callable's `DefMethod` id — the innermost entry of
    /// `callable_scopes`, or `None` outside every supported callable.
    pub current_callable: Option<String>,
    /// Native call-fact producer (invocation captures → `CoreOp::Call`).
    pub call_producer: CallProducer,
    /// Tree-sitter language for capture pipeline.
    pub language: Option<tree_sitter::Language>,
    /// Query string for capture pipeline.
    pub query_string: String,
    /// Optional skip-set for CBM filter-first.
    pub skip_set: Option<std::collections::HashSet<String>>,
    /// Optional focus set for symbol-targeting.
    pub focus: Option<std::collections::HashSet<String>>,
}

impl PassContext {
    /// Create a new pass context.
    pub fn new(source: String, file_id: String, fidelity: Fidelity) -> Self {
        Self {
            instructions: Vec::new(),
            layer_context: LayerContext::new(&source, fidelity),
            program_graph: None,
            inference_layer: None,
            semantic_edges: Vec::new(),
            source,
            file_id,
            canonical_path: None,
            fidelity,
            id_counter: 0,
            language_layers: Vec::new(),
            pattern_recognizers: Vec::new(),
            captures: Vec::new(),
            current_method: None,
            current_control_summaries: Vec::new(),
            current_class: None,
            current_interface: None,
            type_scopes: Vec::new(),
            callable_scopes: Vec::new(),
            current_callable: None,
            call_producer: CallProducer::new(),
            language: None,
            query_string: String::new(),
            skip_set: None,
            focus: None,
        }
    }

    /// Generate the next instruction ID with the given prefix.
    pub fn next_id(&mut self, prefix: &str) -> String {
        self.id_counter += 1;
        format!("{}{}", prefix, self.id_counter)
    }

    /// Set the language layers for this compilation.
    pub fn set_language_layers(&mut self, layers: Vec<Box<dyn LanguageLayer>>) {
        self.language_layers = layers;
    }

    /// Set the pattern recognizers for this compilation.
    pub fn set_pattern_recognizers(&mut self, recognizers: Vec<Box<dyn PatternRecognizer>>) {
        self.pattern_recognizers = recognizers;
    }

    /// Flush accumulated method summaries into a typed instruction (F-28).
    pub(super) fn flush_control_summaries(&mut self) {
        if let Some(method_id) = self.current_method.take() {
            if !self.current_control_summaries.is_empty() {
                let summaries = std::mem::take(&mut self.current_control_summaries);
                self.instructions
                    .push(CoreOp::ControlSummary(method_id, summaries));
            }
        }
        self.current_control_summaries.clear();
    }

    /// Push a type-declaration scope. Closed scopes are pruned lazily by
    /// `refresh_type_owner`, so entering a nested type never orphans the
    /// enclosing type's later members.
    pub(super) fn push_type_scope(&mut self, class_id: String, end_byte: usize) {
        self.type_scopes.push(TypeScope {
            owner: TypeOwner::Class(class_id.clone()),
            end_byte,
        });
        self.current_class = Some(class_id.clone());
        self.current_interface = None;
        self.layer_context.current_class = Some(class_id);
        self.layer_context.current_interface = None;
    }

    pub(super) fn push_interface_scope(&mut self, interface_id: String, end_byte: usize) {
        self.type_scopes.push(TypeScope {
            owner: TypeOwner::Interface(interface_id.clone()),
            end_byte,
        });
        self.current_class = None;
        self.current_interface = Some(interface_id.clone());
        self.layer_context.current_class = None;
        self.layer_context.current_interface = Some(interface_id);
    }

    /// Resolve the innermost type scope whose `[start_byte, end_byte)`
    /// window contains `at`. Scopes that ended at or before `at` are popped.
    /// `current_class` (and its layer-context mirror) track the result so
    /// every member attaches to its true enclosing type.
    pub(super) fn refresh_type_owner(&mut self, at: usize) {
        while let Some(scope) = self.type_scopes.last() {
            if scope.end_byte <= at {
                self.type_scopes.pop();
            } else {
                break;
            }
        }
        let owner = self.type_scopes.last().map(|scope| scope.owner.clone());
        self.current_class = match &owner {
            Some(TypeOwner::Class(id)) => Some(id.clone()),
            _ => None,
        };
        self.current_interface = match &owner {
            Some(TypeOwner::Interface(id)) => Some(id.clone()),
            _ => None,
        };
        self.layer_context
            .current_class
            .clone_from(&self.current_class);
        self.layer_context
            .current_interface
            .clone_from(&self.current_interface);
    }

    /// Push the file-wide synthetic scope for top-level functions
    /// (`func.root`/`arrow.root` with no enclosing class). It must remain
    /// the innermost owner for all subsequent top-level functions in this
    /// file, so it sits ABOVE any other scope and is never pruned.
    pub(super) fn push_file_scope(&mut self, class_id: String) {
        self.type_scopes.push(TypeScope {
            owner: TypeOwner::Class(class_id.clone()),
            end_byte: usize::MAX,
        });
        self.current_class = Some(class_id.clone());
        self.current_interface = None;
        self.layer_context.current_class = Some(class_id);
        self.layer_context.current_interface = None;
    }

    /// Push the callable-declaration scope for `method_id`.
    ///
    /// Mirrors `push_type_scope`: the scope carries the declaration's source
    /// span so a later invocation is attributed by span containment rather
    /// than by whichever method happened to be processed last.
    pub(super) fn push_callable_scope(
        &mut self,
        method_id: String,
        start_byte: usize,
        end_byte: usize,
    ) {
        self.callable_scopes.push(CallableScope {
            method_id: method_id.clone(),
            start_byte,
            end_byte,
        });
        self.current_callable = Some(method_id);
    }

    /// Resolve the innermost callable scope whose `[start_byte, end_byte)`
    /// window contains `at`. Scopes that ended at or before `at` are popped.
    ///
    /// When the active callable changes, the previous callable's pending call
    /// facts are settled into the instruction stream (they are complete: the
    /// walk is in document order and every invocation inside that callable has
    /// already been visited).
    pub(super) fn refresh_callable_owner(&mut self, at: usize) {
        let previous = self.current_callable.clone();
        while let Some(scope) = self.callable_scopes.last() {
            if scope.end_byte <= at {
                self.callable_scopes.pop();
            } else {
                break;
            }
        }
        let owner = self.callable_scopes.last().map(|s| s.method_id.clone());
        if owner != previous {
            let emitted = self.call_producer.settle(previous.as_deref());
            self.instructions.extend(emitted);
            self.current_callable = owner;
        }
    }

    /// Record the callee capture of one invocation query match.
    pub(super) fn record_call_callee(
        &mut self,
        match_index: usize,
        start_byte: usize,
        end_byte: usize,
        callee: &str,
    ) {
        let owner = self.current_callable.clone();
        self.call_producer.record_callee(
            match_index,
            start_byte,
            end_byte,
            callee,
            owner.as_deref(),
        );
    }

    /// Record one explicitly written argument capture of one query match.
    pub(super) fn record_call_argument(&mut self, match_index: usize) {
        self.call_producer.record_argument(match_index);
    }

    /// Record one explicitly written argument capture of one query match that
    /// expands at run time (a TypeScript spread element): it contributes one
    /// written argument AND marks the invocation's count as non-exact.
    pub(super) fn record_call_spread(&mut self, match_index: usize) {
        self.call_producer.record_spread(match_index);
    }

    /// Settle every remaining call fact after the capture walk (the last
    /// callable's region never "closes" during the walk).
    pub(super) fn flush_callable_calls(&mut self) {
        let owner = self.current_callable.take();
        let emitted = self.call_producer.settle(owner.as_deref());
        self.instructions.extend(emitted);
        self.callable_scopes.clear();
    }

    /// Emit a method's IR (DefMethod + Param + Return) and return the method name.
    pub(super) fn emit_method_ir(
        &mut self,
        owner: &TypeOwner,
        method_id: &str,
        raw_sig: &str,
    ) -> String {
        let stripped = strip_csharp_attributes(raw_sig);
        let sig_text = match find_body_start_in(stripped) {
            Some(i) => stripped[..i].trim_end().to_string(),
            None => {
                if let Some(arrow_idx) = stripped.rfind("=>") {
                    stripped[..arrow_idx].trim_end().to_string()
                } else {
                    stripped.to_string()
                }
            }
        };
        let sig = parse_method_sig(&sig_text);
        let name = strip_modifiers(&sig.name, MODIFIERS_LOW);
        let params_str = sig.params_str;
        let return_type = sig.return_type;

        match owner {
            TypeOwner::Class(class_id) => self.instructions.push(CoreOp::DefMethod(
                class_id.clone(),
                method_id.to_string(),
                name.clone(),
            )),
            TypeOwner::Interface(interface_id) => {
                self.instructions.push(CoreOp::DefInterfaceMethod(
                    interface_id.clone(),
                    method_id.to_string(),
                    name.clone(),
                ))
            }
        }

        if !params_str.is_empty() {
            // The formal parameters are the ones the declaration wrote: a
            // comma inside a generic argument list
            // (`Expression<Func<A, B>> key`) does not separate parameters.
            for param in split_parameters(&params_str) {
                let param = param.trim();
                if param.is_empty() {
                    continue;
                }
                let (param_name, param_type) = if let Some(colon_pos) = param.find(':') {
                    let pname = param[..colon_pos].trim().to_string();
                    let ptype = param[colon_pos + 1..].trim().to_string();
                    (pname, ptype)
                } else {
                    (param.to_string(), TYPE_VOID.to_string())
                };

                let param_id = self.next_id("P");
                self.instructions.push(CoreOp::Param(
                    method_id.to_string(),
                    param_id,
                    param_type,
                    param_name,
                ));
            }
        }

        self.instructions
            .push(CoreOp::Return(method_id.to_string(), return_type));

        name
    }

    /// Emit import IR from a raw import line.
    ///
    /// The IR capture closure passes the raw source line for
    /// `import.root`/`package.root` captures (it does not run
    /// `compact_import`), so imports arrive as source text with a literal
    /// ` from ` separator, never as legacy `$im … .$fm …` text. The
    /// vestigial `$im…$fm` decode/strip branch was removed in Phase C0 as
    /// unreachable — nothing produces that form.
    pub(super) fn emit_import_ir(&mut self, raw: &str) {
        let trimmed = raw.trim();

        if let Some(from_pos) = trimmed.find(" from ") {
            let named_part = trimmed[..from_pos].trim();
            let module_part = trimmed[from_pos + 6..]
                .trim()
                .trim_end_matches(';')
                .trim()
                .trim_matches('\'')
                .trim_matches('"');
            let named = if let Some(start) = named_part.find('{') {
                if let Some(end) = named_part.find('}') {
                    named_part[start + 1..end].trim().to_string()
                } else {
                    named_part.to_string()
                }
            } else {
                named_part.to_string()
            };
            let alias = self.next_id("IM");
            self.instructions
                .push(CoreOp::Import(alias, module_part.to_string(), named));
            return;
        }

        let alias = self.next_id("IM");
        self.instructions
            .push(CoreOp::Import(alias, String::new(), trimmed.to_string()));
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

    /// Get the names of all registered passes in order.
    /// Used for architectural ordering verification.
    pub fn pass_names(&self) -> Vec<String> {
        self.passes.iter().map(|p| p.name().to_string()).collect()
    }

    /// Create the default production pipeline with all six stages.
    pub fn default_production() -> Self {
        let mut pipeline = Self::new();
        pipeline.add_pass(Box::new(CoreIRPass::new()));
        pipeline.add_pass(Box::new(LanguageLayerPass::new()));
        pipeline.add_pass(Box::new(MetaLayerPass::new()));
        pipeline.add_pass(Box::new(PatternRecognitionPass::new()));
        pipeline.add_pass(Box::new(AliasResolutionPass::new()));
        pipeline.add_pass(Box::new(ValidationPass::new()));
        pipeline
    }
}

impl Default for PassPipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ── Built-in Passes ──────────────────────────────────────────────

#[cfg(test)]
#[path = "../tests/ir/pipeline.rs"]
mod tests;

// Structural method-identity regressions: a declaration's name, its generic
// type parameters, its parameters, and its return type are read from
// structure (`compaction::signature`), never from whitespace-token position.
// The child module probes the other languages through the same shared
// boundary, so a shared-regression cannot hide behind the C# fixtures.
#[cfg(all(test, feature = "csharp"))]
#[path = "../tests/ir/method_signature_shape.rs"]
mod signature_shape_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/ir/typescript_export_ownership.rs"]
mod typescript_export_ownership_tests;
