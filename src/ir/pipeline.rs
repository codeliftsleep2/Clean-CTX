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

use std::sync::Mutex;

use super::compiler::CompiledIR;
use super::compiler_methods::resolve_forward_aliases;
use super::inference_layer::InferenceLayer;
use super::layers::{LanguageLayer, LayerContext, PatternRecognizer};
use super::opcodes::*;
use super::program_graph::{GraphBuilder, ProgramGraph};
use super::symbol_table::SymbolKind;
use super::validator::{DefaultValidator, IRValidator};
use crate::cbm::bridge::GraphBridge;
use crate::compaction::method::find_method_params;
use crate::compaction::modifiers::{MODIFIERS_LOW, strip_csharp_attributes, strip_modifiers};
use crate::compaction::{
    extract_class_name, extract_field, extract_method_sig, extract_rust_struct_name,
};
use crate::compression::Fidelity;
use crate::compression::capture_pipeline::{CapEntry, run_capture_pipeline};

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
    /// Current method's accumulated flags (F-28).
    pub current_method_flags: Vec<String>,
    /// Current class ID (set when processing a class capture).
    pub current_class: Option<String>,
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
            source,
            file_id,
            fidelity,
            id_counter: 0,
            language_layers: Vec::new(),
            pattern_recognizers: Vec::new(),
            captures: Vec::new(),
            current_method: None,
            current_method_flags: Vec::new(),
            current_class: None,
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

    /// Flush accumulated method flags into a FLAGS instruction (F-28).
    fn flush_method_flags(&mut self) {
        if let Some(method_id) = self.current_method.take() {
            if !self.current_method_flags.is_empty() {
                let flags = std::mem::take(&mut self.current_method_flags);
                self.instructions.push(CoreOp::Flags(method_id, flags));
            }
        }
        self.current_method_flags.clear();
    }

    /// Parse a method signature string into a `MethodSig`.
    fn parse_method_sig(&self, sig: &str) -> MethodSig {
        let sig = sig.trim();

        let (name, params_str, return_type) = if let Some((ps, pe)) = find_method_params(sig) {
            let raw_name = sig[..ps].trim();
            let last_token = raw_name.split_whitespace().last().unwrap_or(raw_name);
            let name = last_token.trim().to_string();
            let params = sig[ps + 1..pe].trim().to_string();
            let rt = sig[pe + 1..].trim();
            let rt = if let Some(stripped) = rt.strip_prefix(':') {
                stripped.trim().to_string()
            } else if rt.is_empty() {
                TYPE_VOID.to_string()
            } else {
                rt.to_string()
            };
            (name, params, rt)
        } else {
            (sig.to_string(), String::new(), TYPE_VOID.to_string())
        };

        MethodSig {
            name,
            params_str,
            return_type: if return_type.is_empty() {
                TYPE_VOID.to_string()
            } else {
                return_type
            },
        }
    }

    /// Emit a method's IR (DefMethod + Param + Return) and return the method name.
    fn emit_method_ir(&mut self, class_id: &str, method_id: &str, raw_sig: &str) -> String {
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
        let sig = self.parse_method_sig(&sig_text);
        let name = strip_modifiers(&sig.name, MODIFIERS_LOW);
        let params_str = sig.params_str;
        let return_type = sig.return_type;

        self.instructions.push(CoreOp::DefMethod(
            class_id.to_string(),
            method_id.to_string(),
            name.clone(),
        ));

        if !params_str.is_empty() {
            for param in params_str.split(',') {
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
    fn emit_import_ir(&mut self, raw: &str) {
        let trimmed = raw.trim();

        if let Some(rest) = trimmed.strip_prefix("$im ") {
            if let Some(fm_pos) = rest.find(".$fm") {
                let named = rest[..fm_pos].trim().to_string();
                let module = rest[fm_pos + 4..].trim().to_string();
                let alias = self.next_id("IM");
                self.instructions.push(CoreOp::Import(alias, module, named));
                return;
            }
            let named = rest.trim().to_string();
            let alias = self.next_id("IM");
            self.instructions
                .push(CoreOp::Import(alias, String::new(), named));
            return;
        }

        if let Some(from_pos) = trimmed.find(" from ") {
            let named_part = trimmed[..from_pos].trim();
            let module_part = trimmed[from_pos + 6..]
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

/// Parsed method signature — the result of parsing the string returned
/// by `compaction::extract_method_sig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSig {
    pub name: String,
    pub params_str: String,
    pub return_type: String,
}

/// Locate the byte index of the brace that opens a method body.
pub(crate) fn find_body_start_in(raw_method: &str) -> Option<usize> {
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut pending_return_brace = false;
    for (i, ch) in raw_method.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth = (paren_depth - 1).max(0);
                if paren_depth == 0 {
                    pending_return_brace = false;
                }
            }
            ':' if paren_depth == 0 && brace_depth == 0 => {
                pending_return_brace = true;
            }
            '{' if paren_depth == 0 && brace_depth == 0 && !pending_return_brace => {
                return Some(i);
            }
            '{' if paren_depth == 0 && pending_return_brace => {
                brace_depth += 1;
                pending_return_brace = false;
            }
            '}' if paren_depth == 0 && brace_depth > 0 => {
                brace_depth -= 1;
            }
            _ if paren_depth == 0 && pending_return_brace && !ch.is_whitespace() => {
                pending_return_brace = false;
            }
            _ => {}
        }
    }
    None
}

/// Extract the verbatim method body from a raw method capture.
pub(crate) fn extract_method_body(raw_method: &str) -> Option<String> {
    let stripped = strip_csharp_attributes(raw_method);
    if let Some(i) = find_body_start_in(stripped) {
        let line_start = stripped[..i].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let prefix = &stripped[line_start..i];
        if prefix.trim().is_empty() {
            return Some(stripped[line_start..].to_string());
        }
        return Some(stripped[i..].to_string());
    }

    if let Some(arrow_idx) = raw_method.rfind("=>") {
        let expr = &raw_method[arrow_idx + 2..];
        let trimmed = expr.trim();
        if !trimmed.is_empty() && trimmed != ";" {
            return Some(expr.to_string());
        }
    }

    None
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
    fn name(&self) -> &str {
        "core_ir"
    }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        // Empty source is valid — produces an empty instruction stream.
        if state.source.is_empty() {
            return Ok(());
        }

        // Extract immutable state upfront to avoid borrow conflicts with
        // the mutable `state` operations in the main capture loop.
        let language = state.language.clone().ok_or_else(|| PassError {
            pass_name: self.name().to_string(),
            message: "no tree-sitter language configured".to_string(),
        })?;
        let query_string = state.query_string.clone();
        let source = state.source.clone();
        let fidelity = state.fidelity;
        let file_id = state.file_id.clone();
        let skip_set = state.skip_set.clone();
        let focus = state.focus.clone();

        let captures = run_capture_pipeline(
            language,
            &query_string,
            &source,
            fidelity,
            |capture_name, raw, fidelity| match capture_name {
                "class.root" => Some(extract_class_name(raw)),
                "struct.root" | "enum.root" | "trait.root" | "impl.root" => {
                    Some(extract_rust_struct_name(raw))
                }
                "method.root" => Some(extract_method_sig(raw, fidelity)),
                "field.root" => Some(extract_field(raw, fidelity)),
                _ => Some(raw.to_string()),
            },
        )
        .map_err(|e| PassError {
            pass_name: self.name().to_string(),
            message: format!("capture pipeline error: {}", e),
        })?;

        // ── Main capture loop: Core IR emission + per-capture language dispatch ──
        for cap in &captures {
            // CBM filter-first: skip low-importance symbols
            if let Some(ref skip) = skip_set {
                if !skip.is_empty() && crate::compression::pipeline::should_skip_capture(cap, skip)
                {
                    continue;
                }
            }
            match cap.name.as_str() {
                "class.root" | "interface.root" | "struct.root" | "enum.root" | "trait.root"
                | "record.root" => {
                    let class_id = state.next_id("C");
                    state
                        .instructions
                        .push(CoreOp::DefClass(class_id.clone(), cap.text.clone()));
                    state.current_class = Some(class_id.clone());
                    state.layer_context.current_class = Some(class_id.clone());
                    state.layer_context.current_class_name = Some(cap.raw_text.clone());
                    state.layer_context.current_class_bare_name = Some(cap.text.clone());

                    state.layer_context.symbol_table_mut().register(
                        class_id.clone(),
                        cap.text.clone(),
                        SymbolKind::Class,
                        &file_id,
                    );

                    for ll in state.language_layers.iter_mut() {
                        let layer_ops =
                            ll.process_capture(&cap.name, &cap.raw_text, &mut state.layer_context);
                        state.instructions.extend(layer_ops);
                    }
                }
                "impl.root" => {
                    if state.current_class.is_none() {
                        let self_type = cap.text.split(':').next().unwrap_or(&cap.text).to_string();
                        if !self_type.is_empty() {
                            let class_id = state.next_id("C");
                            state
                                .instructions
                                .push(CoreOp::DefClass(class_id.clone(), self_type.clone()));
                            state.current_class = Some(class_id.clone());
                            state.layer_context.current_class = Some(class_id);
                            state.layer_context.current_class_name = Some(cap.raw_text.clone());
                            state.layer_context.current_class_bare_name = Some(self_type);
                        }
                    }

                    for ll in state.language_layers.iter_mut() {
                        let layer_ops =
                            ll.process_capture(&cap.name, &cap.raw_text, &mut state.layer_context);
                        state.instructions.extend(layer_ops);
                    }
                }
                "method.root" | "constructor.root" | "func.root" | "arrow.root" => {
                    let class_id = match &state.current_class {
                        Some(cid) => cid.clone(),
                        None => {
                            if cap.name == "func.root" || cap.name == "arrow.root" {
                                let synt_id = state.next_id("C");
                                state.instructions.push(CoreOp::DefClass(
                                    synt_id.clone(),
                                    format!("__file_{}", file_id),
                                ));
                                state.current_class = Some(synt_id.clone());
                                state.layer_context.current_class = Some(synt_id.clone());
                                state.layer_context.current_class_name =
                                    Some(format!("__file_{}", file_id));
                                state.layer_context.current_class_bare_name =
                                    Some(format!("__file_{}", file_id));
                                synt_id
                            } else {
                                continue;
                            }
                        }
                    };

                    state.flush_method_flags();

                    let method_id = state.next_id("M");
                    state.current_method = Some(method_id.clone());
                    state.layer_context.current_method = Some(method_id.clone());
                    state.layer_context.current_method_name = Some(cap.text.clone());

                    let method_name = state.emit_method_ir(&class_id, &method_id, &cap.text);

                    if fidelity == Fidelity::Edit
                        && focus.as_ref().is_none_or(|f| f.contains(&method_name))
                    {
                        if let Some(body) = extract_method_body(&cap.raw_text) {
                            state
                                .instructions
                                .push(CoreOp::Body(method_id.clone(), body));
                        }
                    }

                    for ll in state.language_layers.iter_mut() {
                        let layer_ops =
                            ll.process_capture(&cap.name, &cap.raw_text, &mut state.layer_context);
                        state.instructions.extend(layer_ops);
                    }
                }
                "field.root" => {
                    let class_id = match &state.current_class {
                        Some(cid) => cid.clone(),
                        None => continue,
                    };
                    let field_id = state.next_id("F");
                    state.instructions.push(CoreOp::DefField(
                        class_id,
                        field_id.clone(),
                        cap.text.clone(),
                    ));

                    for ll in state.language_layers.iter_mut() {
                        let layer_ops =
                            ll.process_capture(&cap.name, &cap.text, &mut state.layer_context);
                        state.instructions.extend(layer_ops);
                    }
                }
                "import.root" | "package.root" => {
                    state.emit_import_ir(&cap.text);

                    for ll in state.language_layers.iter_mut() {
                        let layer_ops =
                            ll.process_capture(&cap.name, &cap.text, &mut state.layer_context);
                        state.instructions.extend(layer_ops);
                    }
                }
                "type.root" => {
                    let alias_id = state.next_id("T");
                    state
                        .instructions
                        .push(CoreOp::TypeAlias(alias_id, cap.text.clone()));

                    for ll in state.language_layers.iter_mut() {
                        let layer_ops =
                            ll.process_capture(&cap.name, &cap.text, &mut state.layer_context);
                        state.instructions.extend(layer_ops);
                    }
                }
                "mod.root" => {
                    for ll in state.language_layers.iter_mut() {
                        let layer_ops =
                            ll.process_capture(&cap.name, &cap.text, &mut state.layer_context);
                        state.instructions.extend(layer_ops);
                    }
                }
                "if.root" => {
                    if state.current_method.is_some()
                        && !state.current_method_flags.contains(&FLAG_IF.to_string())
                    {
                        state.current_method_flags.push(FLAG_IF.to_string());
                    }
                }
                "for.root" | "while.root" | "loop.root" => {
                    if state.current_method.is_some()
                        && !state.current_method_flags.contains(&FLAG_LOOP.to_string())
                    {
                        state.current_method_flags.push(FLAG_LOOP.to_string());
                    }
                }
                "return.root" => {
                    if state.current_method.is_some()
                        && !state.current_method_flags.contains(&FLAG_RET.to_string())
                    {
                        state.current_method_flags.push(FLAG_RET.to_string());
                    }
                }
                "throw.root" => {
                    if state.current_method.is_some()
                        && !state.current_method_flags.contains(&FLAG_THROW.to_string())
                    {
                        state.current_method_flags.push(FLAG_THROW.to_string());
                    }
                }
                "do.root" | "try.root" | "switch.root" | "match.root" => {
                    if state.current_method.is_some()
                        && !state.current_method_flags.contains(&FLAG_IF.to_string())
                    {
                        state.current_method_flags.push(FLAG_IF.to_string());
                    }
                }
                _ => {
                    for ll in state.language_layers.iter_mut() {
                        let layer_ops =
                            ll.process_capture(&cap.name, &cap.text, &mut state.layer_context);
                        state.instructions.extend(layer_ops);
                    }
                }
            }
        }

        // Flush any remaining method flags (F-28)
        state.flush_method_flags();

        // C-22: Persist the canonical capture identity for MetaLayerPass.
        // The loop above borrowed the local `captures` while `state` was
        // mutated; once the loop ends the owned batch is MOVED into the
        // existing `PassContext.captures` field (no clone, no parallel
        // vector). MetaLayerPass derives the meta-layer class sources from
        // these CapEntry spans.
        state.captures = captures;

        Ok(())
    }
}

/// Pass 2: Language layer finalization.
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
    fn name(&self) -> &str {
        "language_layer"
    }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        for ll in state.language_layers.iter_mut() {
            let layer_ops = ll.finalize(&mut state.layer_context);
            state.instructions.extend(layer_ops);
        }
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
    fn name(&self) -> &str {
        "meta_layer"
    }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        // C-22: Derive each class capture's canonical SOURCE SPAN from the
        // persisted capture identity (`state.captures`) — NOT from the
        // compacted `CoreOp::DefClass.name`. The meta-layer extractors need
        // the decorator/annotation/attribute-inclusive class text to detect
        // framework semantics (Angular @Component, Spring @RestController,
        // .NET [ApiController]).
        let class_captures: Vec<String> = state
            .captures
            .iter()
            .filter(|cap| {
                matches!(
                    cap.name.as_str(),
                    "class.root"
                        | "interface.root"
                        | "struct.root"
                        | "enum.root"
                        | "trait.root"
                        | "record.root"
                        | "impl.root"
                )
            })
            .map(|cap| crate::meta_util::class_source_from_capture(&state.source, cap).to_string())
            .collect();

        let meta_results = crate::layers::LayerRegistry::global().run_meta_layers_pipeline(
            &state.source,
            &class_captures,
            state.fidelity,
            None,
        );
        for output in &meta_results {
            for line in output.rendered.lines() {
                let line = line.trim();
                if line.is_empty() || !line.starts_with('Φ') {
                    continue;
                }
                let content = line.strip_prefix('Φ').unwrap_or(line);
                if let Some((prefix, text)) = content.split_once(':') {
                    let alias = format!("@{}", prefix);
                    state
                        .instructions
                        .push(CoreOp::TypeAlias(alias, text.to_string()));
                }
            }
        }
        Ok(())
    }
}

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
        for pr in state.pattern_recognizers.iter() {
            let pattern_ops = pr.recognize(&state.instructions);
            state.instructions = pattern_ops;
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

/// Pass 6: Execution semantics extraction (optional — NOT in default pipeline).
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

/// Pass 5 (optional): Program graph construction.
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
        let graph = GraphBuilder::build_from_instructions(&state.instructions);
        state.program_graph = Some(graph);
        Ok(())
    }
}

/// Pass 6 (optional): Inference layer (CBM enrichment + derived analysis).
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
        let mut guard = self.cbm_bridge.lock().unwrap_or_else(|p| p.into_inner());
        match layer.enrich_from_cbm(guard.as_mut()) {
            Ok(()) => {}
            // F11: enrichment failures propagate out of the layer and are
            // owned here. CBM is a strictly-additive enrichment source
            // (invariant C1/C2): a graph hiccup must never fail compilation,
            // but it must also never be mistaken for "no enrichment data".
            // Log loudly and continue with the un-enriched layer.
            Err(e) => {
                eprintln!(
                    "[clean-ctx-ir] CBM inference enrichment failed — continuing without enrichment: {e}"
                );
            }
        }
        state.inference_layer = Some(layer);
        Ok(())
    }
}

/// Pass 6: Validation (structural + consistency checks).
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
        let validator = DefaultValidator::new();
        let ir = CompiledIR {
            file_id: state.file_id.clone(),
            instructions: state.instructions.clone(),
            version: 1,
        };
        let errors = validator.validate(&ir);
        if !errors.is_empty() {
            let summary = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(PassError {
                pass_name: self.name().to_string(),
                message: format!("validation failed: {}", summary),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/ir/pipeline.rs"]
mod tests;
