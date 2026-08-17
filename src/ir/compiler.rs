// src/ir/compiler.rs
//
// IR Compiler — translates tree-sitter captures into Core IR instructions.
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

use crate::compaction::{extract_class_name, extract_field, extract_method_sig, extract_rust_struct_name};
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::Fidelity;
use super::layers::{LanguageLayer, LayerContext, PatternRecognizer};
use super::opcodes::*;
use super::symbol_table::SymbolKind;
use super::compiler_methods::{extract_method_body, resolve_forward_aliases};

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
    /// Source produced no `class.root` / `method.root` captures.
    /// Not necessarily fatal — the call site may treat it as an
    /// empty (but valid) compile result.
    NoCaptures,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Capture(msg) => write!(f, "capture pipeline error: {}", msg),
            CompileError::Layer(msg) => write!(f, "layer error: {}", msg),
            CompileError::NoCaptures => write!(f, "source produced no captures"),
        }
    }
}

impl std::error::Error for CompileError {}

/// IR Compiler — translates tree-sitter captures into Core IR instructions.
///
/// The compiler now owns pluggable layers (F-01, F-02, F-03):
///   - Language layers process language-specific captures (extends, implements, flags)
///   - Meta layers extract framework-specific patterns (Angular decorators, etc.)
///   - Pattern recognizers compress instruction streams into compact ops
pub struct IRCompiler {
    /// Running instruction counter for ID generation (F-31: `u64` instead
    /// of `u32` to avoid arithmetic overflow at 4,294,967,295 instructions).
    id_counter: u64,
    /// Current method being processed (F-27: O(1) tracking instead of O(n) search)
    current_method: Option<String>,
    /// Current method's accumulated flags (F-28: O(1) push instead of O(n) search)
    current_method_flags: Vec<String>,
    /// Current class ID (set when processing a class capture)
    current_class: Option<String>,
    /// Language-specific layers (Layer 2)
    language_layers: Vec<Box<dyn LanguageLayer>>,
    /// Pattern recognizers (Layer 4)
    pattern_recognizers: Vec<Box<dyn PatternRecognizer>>,
}

impl IRCompiler {
    pub fn new() -> Self {
        Self {
            id_counter: 0,
            current_method: None,
            current_method_flags: Vec::new(),
            current_class: None,
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
    /// on `CompileError::Capture`, `Layer`, or `NoCaptures`. The
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
        // Reset per-compilation state
        self.current_method = None;
        self.current_method_flags = Vec::new();
        self.current_class = None;
        let mut layer_context = LayerContext::new(source, fidelity);

        let captures = run_capture_pipeline(
            language,
            query_string,
            source,
            fidelity,
            |capture_name, raw, fidelity| {
                // Use the existing compaction functions to normalize captured
                // text, same as the text-based pipeline does. This ensures
                // class names are extracted, method signatures are compacted,
                // and fields are properly formatted.
                match capture_name {
                    "class.root" => Some(extract_class_name(raw)),
                    // Rust type declarations: extract clean names
                    "struct.root" | "enum.root" | "trait.root" | "impl.root" => {
                        Some(extract_rust_struct_name(raw))
                    }
                    "method.root" => Some(extract_method_sig(raw, fidelity)),
                    "field.root" => Some(extract_field(raw, fidelity)),
                    _ => Some(raw.to_string()),
                }
            },
        )
        .map_err(|e| CompileError::Capture(e.to_string()))?;

        let mut instructions = Vec::new();

        // ── Layer 1: Core IR emission ──────────────────────────────────
        // Also invokes Layer 2 (LanguageLayer::process_capture) for each capture.
        for cap in &captures {
            // CBM filter-first: skip low-importance symbols before emitting
            // any IR ops. Uses the same should_skip_capture logic as the
            // text compression pipeline for consistent behavior.
            if let Some(skip) = skip_set {
                if !skip.is_empty() && crate::compression::pipeline::should_skip_capture(cap, skip) {
                    continue;
                }
            }
            match cap.name.as_str() {
                // ── Type/capture-based actions ───────────────────────
                // Each arm either:
                //   A) Emits DefClass and sets up class context (for type-like captures)
                //   B) Emits DefMethod (for method/constructor captures)
                //   C) Emits DefField (for field captures)
                //   D) Emits Import/TypeAlias (for import/type captures)
                //   E) Accumulates flags (for control flow captures)
                //   F) Passes through to language layers (for other captures)
                //
                // Type-like captures (emit DefClass):
                //   class.root        - TS/C#/Java classes
                //   interface.root    - TS/C#/Java interfaces
                //   struct.root       - Rust structs, C# structs
                //   enum.root         - TS/Rust/Java/C# enums
                //   trait.root        - Rust traits
                //   record.root       - Java/C# records
                //
                // Method-like captures (emit DefMethod):
                //   method.root       - methods in all languages
                //   constructor.root  - Java/C# constructors
                //   func.root         - TS/JS standalone functions
                //   arrow.root        - TS/JS arrow functions

                // ── Type-like: DefClass + class context setup ────────
                "class.root" | "interface.root" | "struct.root" | "enum.root" | "trait.root" | "record.root" => {
                    let class_id = self.next_id("C");
                    instructions.push(CoreOp::DefClass(
                        class_id.clone(),
                        cap.text.clone(),
                    ));
                    self.current_class = Some(class_id.clone());
                    layer_context.current_class = Some(class_id.clone());
                    layer_context.current_class_name = Some(cap.raw_text.clone());
                    layer_context.current_class_bare_name = Some(cap.text.clone());

                    layer_context.symbol_table_mut().register(
                        class_id.clone(),
                        cap.text.clone(),
                        SymbolKind::Class,
                        file_id,
                    );

                    // Invoke language layers for type-like captures.
                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.raw_text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // Rust impl blocks: emit trait impls with a class context.
                // If no current class exists, emit a DefClass for the self-type.
                "impl.root" => {
                    if self.current_class.is_none() {
                        let self_type = cap.text.split(':').next()
                            .unwrap_or(&cap.text)
                            .to_string();
                        if !self_type.is_empty() {
                            let class_id = self.next_id("C");
                            instructions.push(CoreOp::DefClass(
                                class_id.clone(),
                                self_type.clone(),
                            ));
                            self.current_class = Some(class_id.clone());
                            layer_context.current_class = Some(class_id);
                            layer_context.current_class_name = Some(cap.raw_text.clone());
                            layer_context.current_class_bare_name = Some(self_type);
                        }
                    }

                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.raw_text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // ── Method-like: DefMethod + method context setup ────
                "method.root" | "constructor.root" | "func.root" | "arrow.root" => {
                    let class_id = match &self.current_class {
                        Some(cid) => cid.clone(),
                        None => {
                            // No class context: for standalone functions (func.root),
                            // emit as a standalone method with a synthetic class.
                            if cap.name == "func.root" || cap.name == "arrow.root" {
                                // Create a synthetic class for the file
                                let synt_id = self.next_id("C");
                                instructions.push(CoreOp::DefClass(
                                    synt_id.clone(),
                                    format!("__file_{}", file_id),
                                ));
                                self.current_class = Some(synt_id.clone());
                                layer_context.current_class = Some(synt_id.clone());
                                layer_context.current_class_name = Some(format!("__file_{}", file_id));
                                layer_context.current_class_bare_name = Some(format!("__file_{}", file_id));
                                synt_id
                            } else {
                                // F-29: Skip method captures outside a class
                                continue;
                            }
                        }
                    };

                    // Flush pending flags from previous method (F-28)
                    self.flush_method_flags(&mut instructions);

                    let method_id = self.next_id("M");
                    self.current_method = Some(method_id.clone());
                    layer_context.current_method = Some(method_id.clone());
                    layer_context.current_method_name = Some(cap.text.clone());

                    self.emit_method_ir(
                        &mut instructions,
                        &class_id,
                        &method_id,
                        &cap.text,
                    );

                    // Edit Mode: emit verbatim method body when fidelity is Edit.
                    // The raw_text for method.root captures the full method
                    // including the body. We extract everything from the first
                    // '{' to the end (inclusive) as the byte-exact body.
                    if fidelity == Fidelity::Edit {
                        if let Some(body) = extract_method_body(&cap.raw_text) {
                            instructions.push(CoreOp::Body(method_id.clone(), body));
                        }
                    }

                    // Invoke language layers for method-like captures.
                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.raw_text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // ── Field captures ───────────────────────────────────
                "field.root" => {
                    let class_id = match &self.current_class {
                        Some(cid) => cid.clone(),
                        None => {
                            // F-29: Skip field captures outside a class
                            continue;
                        }
                    };
                    let field_id = self.next_id("F");
                    instructions.push(CoreOp::DefField(
                        class_id,
                        field_id.clone(),
                        cap.text.clone(),
                    ));

                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // ── Import and package captures ──────────────────────
                "import.root" | "package.root" => {
                    self.emit_import_ir(&mut instructions, &cap.text);

                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // ── Type alias captures (Rust `type Foo = Bar` and TS `type Foo = ...`) ──
                "type.root" => {
                    let alias_id = self.next_id("T");
                    instructions.push(CoreOp::TypeAlias(
                        alias_id,
                        cap.text.clone(),
                    ));

                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // ── Rust mod declarations: structural pass-through ──
                "mod.root" => {
                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // ── Control flow captures → FLAGS accumulation ──────
                "if.root" => {
                    if self.current_method.is_some()
                        && !self.current_method_flags.contains(&FLAG_IF.to_string())
                    {
                        self.current_method_flags.push(FLAG_IF.to_string());
                    }
                }
                "for.root" | "while.root" | "loop.root" => {
                    if self.current_method.is_some()
                        && !self.current_method_flags.contains(&FLAG_LOOP.to_string())
                    {
                        self.current_method_flags.push(FLAG_LOOP.to_string());
                    }
                }
                "return.root" => {
                    if self.current_method.is_some()
                        && !self.current_method_flags.contains(&FLAG_RET.to_string())
                    {
                        self.current_method_flags.push(FLAG_RET.to_string());
                    }
                }
                "throw.root" => {
                    if self.current_method.is_some()
                        && !self.current_method_flags.contains(&FLAG_THROW.to_string())
                    {
                        self.current_method_flags.push(FLAG_THROW.to_string());
                    }
                }
                "do.root" | "try.root" | "switch.root" | "match.root" => {
                    // Additional control flow: do-while, try-catch, switch,
                    // Rust match expressions. Accumulate IF flag as a general
                    // branching marker.
                    if self.current_method.is_some()
                        && !self.current_method_flags.contains(&FLAG_IF.to_string())
                    {
                        self.current_method_flags.push(FLAG_IF.to_string());
                    }
                }
                // ── Pass-through to language layers for any other capture ──
                _ => {
                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
            }
        }

        // Flush any remaining method flags (F-28)
        self.flush_method_flags(&mut instructions);

        // Call LanguageLayer::finalize for each language layer
        for ll in self.language_layers.iter_mut() {
            let layer_ops = ll.finalize(&mut layer_context);
            instructions.extend(layer_ops);
        }

        // ── Layer 3: Meta-layer pass (F-02) ────────────────────────────
        // Collect class names from the instruction stream for meta extraction
        let class_names: Vec<String> = instructions
            .iter()
            .filter_map(|op| {
                if let CoreOp::DefClass(_, name) = op {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        // P0-4: Use canonical LayerRegistry instead of removed ir::layers::MetaLayer trait.
        // Meta-layers are registered in src/layers/meta/ and wired via McpState -> LayerRegistry.
        let meta_results = crate::layers::LayerRegistry::global()
            .run_meta_layers_pipeline(source, &class_names, fidelity);
        for (_layer_name, block_text) in &meta_results {
            // Parse Φ marker lines into CoreOp instructions for IR enrichment.
            for line in block_text.lines() {
                let line = line.trim();
                if line.is_empty() || !line.starts_with('Φ') {
                    continue;
                }
                let content = line.strip_prefix('Φ').unwrap_or(line);
                if let Some((prefix, text)) = content.split_once(':') {
                    let alias = format!("@{}", prefix);
                    instructions.push(CoreOp::TypeAlias(alias, text.to_string()));
                }
            }
        }

        // ── Layer 4: Pattern recognition (F-03) ────────────────────────
        for pr in self.pattern_recognizers.iter() {
            let pattern_ops = pr.recognize(&instructions);
            // Replace instructions with recognized output
            instructions = pattern_ops;
        }

        // ── F-FULL-08: Forward-declaration alias resolution ────────────
        // Post-process the IR stream to resolve Extends/Implements ops
        // whose target is a raw class name (no alias ID prefix like "C1").
        // This handles forward references where class B extends class A
        // but A is defined later in the file.
        resolve_forward_aliases(&mut instructions);

        Ok(CompiledIR {
            file_id: file_id.to_string(),
            instructions,
            version: 1,
        })
    }

    /// Flush accumulated method flags into a FLAGS instruction (F-28).
    fn flush_method_flags(&mut self, instructions: &mut Vec<CoreOp>) {
        if let Some(method_id) = self.current_method.take() {
            if !self.current_method_flags.is_empty() {
                let flags = std::mem::take(&mut self.current_method_flags);
                instructions.push(CoreOp::Flags(method_id, flags));
            }
        }
        self.current_method_flags.clear();
    }

    /// Reset the ID counter (for deterministic testing).
    pub fn reset_counter(&mut self) {
        self.id_counter = 0;
    }

    pub(super) fn next_id(&mut self, prefix: &str) -> String {
        self.id_counter += 1;
        format!("{}{}", prefix, self.id_counter)
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