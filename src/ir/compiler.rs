// src/ir/compiler.rs
//
// IR Compiler — translates tree-sitter captures into Core IR instructions.
//
// The compiler is now an orchestration boundary. Production compilation
// stages are implemented in PassPipeline passes (src/ir/pipeline.rs).
// IRCompiler::compile_inner() constructs a PassContext, configures the
// pipeline, and delegates compilation to PassPipeline::run().
//
// Phase A: IR Core — the compiler reuses the existing capture pipeline
// (`run_capture_pipeline`) but emits `Vec<CoreOp>` instructions instead
// of formatted text strings.
//
// Phase A (FAANG remediation): Wire layers into the compile path.
//   - F-01: IRCompiler now owns language layers, meta layers, and pattern recognizers.
//   - F-02: MetaLayer::extract is called after the main compile loop.
//   - F-03: PatternRecognizer::recognize is called after meta extraction.
//   - F-27: `current_method` is tracked directly (O(1) instead of O(n) via find_last_method).
//   - F-28: Flags are accumulated in a `current_method_flags` Vec (O(1) per capture).
//   - F-29: Methods/fields without a current_class are skipped (not silently emitted with "").
//   - F-30: `compile` returns `CompileError` (a typed enum) instead of `Box<dyn Error>`.
//   - F-31: `id_counter` is `u64` (not `u32`) to avoid arithmetic overflow
//           after 4,294,967,295 instructions.

use super::layers::{LanguageLayer, PatternRecognizer};
use super::opcodes::CoreOp;
use super::pipeline::{PassContext, PassPipeline};
use crate::compression::Fidelity;

/// The compiled IR for a single file.
#[derive(Debug, Clone)]
pub struct CompiledIR {
    /// File identifier (path alias)
    pub file_id: String,
    /// Ordered instruction stream
    pub instructions: Vec<CoreOp>,
    /// Monotonic version number
    pub version: u64,
}

/// Errors that can occur during IR compilation (F-30).
///
/// Replaces the previous `Box<dyn std::error::Error>` return type so
/// callers can programmatically distinguish between parse failures,
/// "no captures" conditions, and other errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// Underlying tree-sitter / capture pipeline failure.
    Capture(String),
    /// A language / meta / pattern layer raised an error.
    Layer(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Capture(msg) => write!(f, "capture pipeline error: {msg}"),
            CompileError::Layer(msg) => write!(f, "layer error: {msg}"),
        }
    }
}

impl std::error::Error for CompileError {}

/// IR Compiler — translates tree-sitter captures into Core IR instructions.
///
/// The compiler is now primarily an orchestration boundary. It owns the
/// language layers and pattern recognizers, constructs a PassContext,
/// configures and runs the PassPipeline, and returns the resulting CompiledIR.
///
/// Individual compilation stages are implemented in the corresponding
/// IRPass implementations in src/ir/pipeline.rs.
pub struct IRCompiler {
    /// Running instruction counter for ID generation (F-31: `u64` instead
    /// of `u32` to avoid arithmetic overflow at 4,294,967,295 instructions).
    /// Used by compiler_methods.rs helpers and test code.
    id_counter: u64,
    /// Language-specific layers (Layer 2)
    language_layers: Vec<Box<dyn LanguageLayer>>,
    /// Pattern recognizers (Layer 4)
    pattern_recognizers: Vec<Box<dyn PatternRecognizer>>,
}

impl IRCompiler {
    pub fn new() -> Self {
        Self {
            id_counter: 0,
            language_layers: Vec::new(),
            pattern_recognizers: Vec::new(),
        }
    }

    /// Add a language layer (Layer 2).
    pub fn add_language_layer(&mut self, layer: Box<dyn LanguageLayer>) {
        self.language_layers.push(layer);
    }

    /// Add a pattern recognizer (Layer 4).
    pub fn add_pattern_recognizer(&mut self, layer: Box<dyn PatternRecognizer>) {
        self.pattern_recognizers.push(layer);
    }

    /// `skip_set`: optional set of symbol names to exclude from IR output.
    /// When a class, method, or field name matches an entry in this set,
    /// the capture is dropped entirely (no `DefClass`, `DefMethod`, or
    /// `DefField` emitted). Used by the CBM filter-first architecture to
    /// exclude low-importance symbols.
    ///
    /// Returns a typed `CompileError` (F-30) — callers can pattern-match
    /// on `CompileError::Capture` or `Layer`. The
    /// `mcp::tools` boundary converts to `Box<dyn Error>` via `?` if needed.
    pub fn compile(
        &mut self,
        source: &str,
        file_id: &str,
        language: tree_sitter::Language,
        query_string: &str,
        fidelity: Fidelity,
        skip_set: Option<&std::collections::HashSet<String>>,
    ) -> Result<CompiledIR, CompileError> {
        self.compile_inner(
            source,
            file_id,
            language,
            query_string,
            fidelity,
            skip_set,
            None,
        )
    }

    /// Compile with symbol targeting (`focus`).
    ///
    /// `focus`: optional set of method names that should receive full
    /// verbatim bodies at `Edit` fidelity. When `Some(set)`, only methods
    /// whose parsed name is in the set get their body extracted into the IR
    /// (`CoreOp::Body`); all other methods are emitted signature-only. This
    /// mirrors the render-time gate in `render_llm.rs` and avoids extracting
    /// and storing body text for methods that will be filtered out at render
    /// time — a memory/CPU optimization. When `None`, every method's body is
    /// extracted (legacy behavior, byte-identical to `compile`).
    ///
    /// `#[allow(clippy::too_many_arguments)]`: mirrors the existing `compile`
    /// signature (7 args) plus the `focus` set (8 total). Grouping the
    /// compile inputs into a struct would churn every call site; the allow is
    /// scoped to this one symbol-targeting entry point.
    #[allow(clippy::too_many_arguments)]
    pub fn compile_focused(
        &mut self,
        source: &str,
        file_id: &str,
        language: tree_sitter::Language,
        query_string: &str,
        fidelity: Fidelity,
        skip_set: Option<&std::collections::HashSet<String>>,
        focus: Option<&std::collections::HashSet<String>>,
    ) -> Result<CompiledIR, CompileError> {
        self.compile_inner(
            source,
            file_id,
            language,
            query_string,
            fidelity,
            skip_set,
            focus,
        )
    }

    /// Compile with a compiler-managed ID counter and no transfer of layers.
    /// Used by the compiler_methods.rs migration artifacts that still
    /// reference `IRCompiler::next_id()` for ID generation.
    #[allow(dead_code)]
    pub(super) fn next_id(&mut self, prefix: &str) -> String {
        self.id_counter += 1;
        format!("{}{}", prefix, self.id_counter)
    }

    /// Reset the ID counter (for deterministic testing).
    #[allow(dead_code)]
    pub fn reset_counter(&mut self) {
        self.id_counter = 0;
    }

    /// Shared implementation for `compile` and `compile_focused`.
    ///
    /// Constructs a PassContext, configures the PassPipeline, and delegates
    /// compilation to the pipeline. The resulting instruction stream is
    /// wrapped in a CompiledIR and returned.
    #[allow(clippy::too_many_arguments)]
    fn compile_inner(
        &mut self,
        source: &str,
        file_id: &str,
        language: tree_sitter::Language,
        query_string: &str,
        fidelity: Fidelity,
        skip_set: Option<&std::collections::HashSet<String>>,
        focus: Option<&std::collections::HashSet<String>>,
    ) -> Result<CompiledIR, CompileError> {
        // Construct PassContext with per-compilation state
        let mut ctx = PassContext::new(source.to_string(), file_id.to_string(), fidelity);
        ctx.language = Some(language);
        ctx.query_string = query_string.to_string();
        ctx.skip_set = skip_set.cloned();
        ctx.focus = focus.cloned();

        // Transfer ownership of language layers and pattern recognizers
        // to the PassContext for the duration of this compilation.
        let compiler_layers = std::mem::take(&mut self.language_layers);
        let compiler_recognizers = std::mem::take(&mut self.pattern_recognizers);
        ctx.set_language_layers(compiler_layers);
        ctx.set_pattern_recognizers(compiler_recognizers);

        // Build and run the default production pipeline
        let pipeline = PassPipeline::default_production();
        pipeline.run(&mut ctx).map_err(|e| {
            // Map PassError to the appropriate CompileError variant
            if e.pass_name == "core_ir" && e.message.contains("capture pipeline error") {
                CompileError::Capture(e.message)
            } else {
                CompileError::Layer(e.message)
            }
        })?;

        // Return ownership of language layers and pattern recognizers
        // back to the compiler for reuse in subsequent compilations.
        self.language_layers = std::mem::take(&mut ctx.language_layers);
        self.pattern_recognizers = std::mem::take(&mut ctx.pattern_recognizers);

        Ok(CompiledIR {
            file_id: file_id.to_string(),
            instructions: ctx.instructions,
            version: 1,
        })
    }
}

impl Default for IRCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/ir/compiler.rs"]
mod tests;
