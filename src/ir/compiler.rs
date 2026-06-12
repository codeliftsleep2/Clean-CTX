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
use super::layers::{LanguageLayer, LayerContext, MetaLayer, PatternRecognizer};
use super::opcodes::*;
use super::symbol_table::SymbolKind;
use super::compiler_methods::resolve_forward_aliases;

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
    /// Framework meta layers (Layer 3)
    meta_layers: Vec<Box<dyn MetaLayer>>,
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
            meta_layers: Vec::new(),
            pattern_recognizers: Vec::new(),
        }
    }

    /// Add a language layer (Layer 2).
    pub fn add_language_layer(&mut self, layer: Box<dyn LanguageLayer>) {
        self.language_layers.push(layer);
    }

    /// Add a meta layer (Layer 3).
    pub fn add_meta_layer(&mut self, layer: Box<dyn MetaLayer>) {
        self.meta_layers.push(layer);
    }

    /// Add a pattern recognizer (Layer 4).
    pub fn add_pattern_recognizer(&mut self, layer: Box<dyn PatternRecognizer>) {
        self.pattern_recognizers.push(layer);
    }

    /// Compile source code into IR.
    /// Reuses the existing capture pipeline but emits CoreOp instructions
    /// instead of formatted text strings.
    ///
    /// The compile pipeline runs in four layers (F-01, F-02, F-03):
    ///   1. Core IR emission (always runs)
    ///   2. Language layer translation (TS/C# specific ops)
    ///   3. Meta-layer pass (framework-specific extraction)
    ///   4. Pattern recognition (instruction stream compression)
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
            match cap.name.as_str() {
                "class.root" => {
                    let class_id = self.next_id("C");
                    instructions.push(CoreOp::DefClass(
                        class_id.clone(),
                        cap.text.clone(),
                    ));
                    self.current_class = Some(class_id.clone());
                    layer_context.current_class = Some(class_id.clone());
                    // F-FULL-14: Use cap.raw_text for current_class_name so the
                    // class name includes any `extends`/`implements` suffixes
                    // that language layers may need. Previously used cap.text
                    // which is the *extracted* name (modifiers stripped).
                    // For the Angular layer's phi-line parsing, store the
                    // bare class name too (derived from cap.text).
                    layer_context.current_class_name = Some(cap.raw_text.clone());
                    layer_context.current_class_bare_name = Some(cap.text.clone());

                    // NF-05: Register the class alias in the symbol table so
                    // language layers (TypeScript, C#) can resolve aliases for
                    // EXT/IMPL ops via `symbol_table.alias_for(&base_id)`.
                    layer_context.symbol_table_mut().register(
                        class_id.clone(),
                        cap.text.clone(),
                        SymbolKind::Class,
                        file_id,
                    );

                    // Invoke language layers for class captures.
                    // Pass raw_text so layers can parse extends/implements
                    // from the full class head declaration.
                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.raw_text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // Rust type declarations: struct, enum, trait
                // These are treated as class definitions in the IR.
                "struct.root" | "enum.root" | "trait.root" => {
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

                    // Invoke language layers for Rust type captures.
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
                // If no current class exists (standalone impl block without
                // a preceding struct/enum/trait), emit a DefClass for the
                // self-type so methods inside the impl are not silently lost.
                // This mirrors the text pipeline's behavior where impl.root
                // creates a structural entry via format_rust_type_entry.
                //
                // Phase C: The extracted cap.text for impl.root has the form
                // "Type:Trait" (trait impl) or "Type" (inherent impl) after
                // extract_rust_struct_name processing. We use only the type
                // part (before ':') for the class name.
                "impl.root" => {
                    if self.current_class.is_none() {
                        // Extract the self-type name from the impl head.
                        // cap.text is already processed through extract_rust_struct_name:
                        //   "impl Trait for Type" → "Type:Trait"
                        //   "impl Type" → "Type"
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
                "method.root" => {
                    let class_id = match &self.current_class {
                        Some(cid) => cid.clone(),
                        None => {
                            // F-29: Skip method captures outside a class
                            // instead of silently emitting with empty class id
                            continue;
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

                    // Invoke language layers for method captures.
                    // Pass raw_text so layers can parse modifiers (async, static, etc.)
                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.raw_text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
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

                    // Invoke language layers for field captures
                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                "import.root" => {
                    self.emit_import_ir(&mut instructions, &cap.text);

                    // Invoke language layers for import captures
                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // Rust type aliases: type Foo = Bar; → TypeAlias
                "type.root" => {
                    let alias_id = self.next_id("T");
                    // The extracted text is the alias name (e.g., "UserId")
                    instructions.push(CoreOp::TypeAlias(
                        alias_id,
                        cap.text.clone(),
                    ));

                    // Invoke language layers for type alias captures
                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // Rust mod declarations: treated as structural imports
                "mod.root" => {
                    // Invoke language layers for mod captures
                    for ll in self.language_layers.iter_mut() {
                        let layer_ops = ll.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut layer_context,
                        );
                        instructions.extend(layer_ops);
                    }
                }
                // Control flow captures → FLAGS on the most recent method
                "if.root" => {
                    // F-28: Accumulate in current_method_flags (O(1))
                    if self.current_method.is_some()
                        && !self.current_method_flags.contains(&FLAG_IF.to_string())
                    {
                        self.current_method_flags.push(FLAG_IF.to_string());
                    }
                }
                "for.root" | "while.root" => {
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
                _ => {
                    // Invoke language layers for any other captures
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

        for ml in self.meta_layers.iter_mut() {
            let meta_ops = ml.extract(source, &class_names, fidelity);
            instructions.extend(meta_ops);
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