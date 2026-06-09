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

use crate::compaction::{extract_class_name, extract_field, extract_method_sig};
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::Fidelity;
use super::layers::{LanguageLayer, LayerContext, MetaLayer, PatternRecognizer};
use super::opcodes::*;
use super::symbol_table::SymbolKind;

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

    fn next_id(&mut self, prefix: &str) -> String {
        self.id_counter += 1;
        format!("{}{}", prefix, self.id_counter)
    }

    /// Parse a method signature and emit DefMethod + Param + Return instructions.
    ///
    /// Accepts signatures in the format produced by `extract_method_sig`:
    ///   - `methodName(param1:$t,param2:$t):$t`
    ///   - `methodName():$t`
    fn emit_method_ir(
        &mut self,
        instructions: &mut Vec<CoreOp>,
        class_id: &str,
        method_id: &str,
        raw_sig: &str,
    ) {
        // Parse the method signature: "name(params):return_type"
        let sig = parse_method_sig(raw_sig);
        let name = sig.name;
        let params_str = sig.params_str;
        let return_type = sig.return_type;

        // Emit DefMethod
        instructions.push(CoreOp::DefMethod(
            class_id.to_string(),
            method_id.to_string(),
            name,
        ));

        // Emit Param instructions for each parameter
        if !params_str.is_empty() {
            for param in params_str.split(',') {
                let param = param.trim();
                if param.is_empty() {
                    continue;
                }
                // Parse "name:$type" or just "name"
                let (param_name, param_type) = if let Some(colon_pos) = param.find(':') {
                    let pname = param[..colon_pos].trim().to_string();
                    let ptype = param[colon_pos + 1..].trim().to_string();
                    (pname, ptype)
                } else {
                    (param.to_string(), TYPE_VOID.to_string())
                };

                let param_id = self.next_id("P");
                instructions.push(CoreOp::Param(
                    method_id.to_string(),
                    param_id,
                    param_type,
                    param_name,
                ));
            }
        }

        // Emit Return
        instructions.push(CoreOp::Return(
            method_id.to_string(),
            return_type,
        ));
    }

    /// Emit import IR from a raw import line.
    fn emit_import_ir(&mut self, instructions: &mut Vec<CoreOp>, raw: &str) {
        // Raw import looks like: `import { Foo } from 'module'`
        // or compacted: `$im Foo.$fmmodule` / `$im $fm module`
        // We try to parse the compacted format first.
        let trimmed = raw.trim();

        // Try to parse: "$im <named>.$fm <module>" pattern
        if let Some(rest) = trimmed.strip_prefix("$im ") {
            if let Some(fm_pos) = rest.find(".$fm") {
                let named = rest[..fm_pos].trim().to_string();
                let module = rest[fm_pos + 4..].trim().to_string();
                let alias = self.next_id("IM");
                instructions.push(CoreOp::Import(alias, module, named));
                return;
            }
            // Just "$im something" without .$fm
            let named = rest.trim().to_string();
            let alias = self.next_id("IM");
            instructions.push(CoreOp::Import(alias, String::new(), named));
            return;
        }

        // Try to parse standard ES import: "import { X } from 'module'"
        if let Some(from_pos) = trimmed.find(" from ") {
            let named_part = trimmed[..from_pos].trim();
            let module_part = trimmed[from_pos + 6..].trim().trim_matches('\'').trim_matches('"');
            // Extract named imports: "import { Foo, Bar } from ..."
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
            instructions.push(CoreOp::Import(alias, module_part.to_string(), named));
            return;
        }

        // Fallback: just emit as-is
        let alias = self.next_id("IM");
        instructions.push(CoreOp::Import(alias, String::new(), trimmed.to_string()));
    }
}

/// Parsed method signature — the result of parsing the string returned
/// by `compaction::extract_method_sig`.
///
/// F-26: This struct formalizes the output shape that `parse_method_sig`
/// returns, making the two-field `(name, params_str, return_type)` tuple
/// self-documenting. The upstream `extract_method_sig` (in `compaction/*`)
/// returns a string of the form `name(params):return_type`, and this
/// function re-parses that string into a structured form. This is an
/// acknowledged duplication — a future refactor could have
/// `extract_method_sig` return this struct directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSig {
    /// Method name (e.g., "processComplexData")
    pub name: String,
    /// Raw parameter list string (e.g., "payload:$s[],payload2:$n")
    pub params_str: String,
    /// Return type opcode (e.g., "$b", "$v")
    pub return_type: String,
}

/// Parse a method signature string into a `MethodSig`.
///
/// Input: "processComplexData(payload:$s[],payload2:$n):$b"
/// Output: MethodSig { name: "processComplexData", params_str: "payload:$s[],payload2:$n", return_type: "$b" }
fn parse_method_sig(sig: &str) -> MethodSig {
    let sig = sig.trim();

    // Find the return type separator "):"
    // We need to handle nested parens in param types like "items:string[]"
    let mut paren_depth = 0i32;
    let mut params_start = None;
    let mut params_end = None;

    for (i, ch) in sig.char_indices() {
        match ch {
            '(' if params_start.is_none() => {
                params_start = Some(i);
                paren_depth = 1;
            }
            '(' => paren_depth += 1,
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    params_end = Some(i);
                }
            }
            _ => {}
        }
    }

    let (name, params_str, return_type) = if let (Some(ps), Some(pe)) = (params_start, params_end) {
        let name = sig[..ps].trim().to_string();
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
        // No parens found — treat entire string as method name
        (sig.to_string(), String::new(), TYPE_VOID.to_string())
    };

    MethodSig {
        name,
        params_str,
        return_type: if return_type.is_empty() { TYPE_VOID.to_string() } else { return_type },
    }
}

impl Default for IRCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// F-FULL-08: Post-process the IR stream to resolve forward-declared class
/// aliases. When class B extends class A, but A is defined later in the file,
/// the TypeScript layer emits `Extends("C2", "A")` where "A" is a raw class
/// name (not an alias ID). This function builds a mapping from class name →
/// alias ID from the `DefClass` ops in the stream, then rewrites any
/// `Extends`/`Implements` ops that reference a raw class name.
fn resolve_forward_aliases(instructions: &mut Vec<CoreOp>) {
    // First pass: build the class-name → alias-id mapping from DefClass ops.
    let mut name_to_alias: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for op in instructions.iter() {
        if let CoreOp::DefClass(alias_id, name) = op {
            // name is the extracted class name like "FooService" (no modifiers).
            // Store the mapping so we can resolve Extends("C1", "Foo") → Extends("C1", "C2").
            if !name_to_alias.contains_key(name) {
                name_to_alias.insert(name.clone(), alias_id.clone());
            }
        }
    }

    // Second pass: rewrite Extends/Implements ops that reference raw class names.
    for op in instructions.iter_mut() {
        match op {
            CoreOp::Extends(_, target) => {
                // If target looks like a raw class name (not an alias ID starting with "C"),
                // try to resolve it to the alias ID.
                if !target.starts_with('C') {
                    if let Some(alias) = name_to_alias.get(target.as_str()) {
                        *target = alias.clone();
                    }
                }
            }
            CoreOp::Implements(_, target) => {
                if !target.starts_with('C') {
                    if let Some(alias) = name_to_alias.get(target.as_str()) {
                        *target = alias.clone();
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "../tests/ir/compiler.rs"]
mod tests;
           