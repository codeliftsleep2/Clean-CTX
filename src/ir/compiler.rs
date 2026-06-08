// src/ir/compiler.rs
//
// IR Compiler — translates tree-sitter captures into Core IR instructions.
//
// Phase A: IR Core — the compiler reuses the existing capture pipeline
// (`run_capture_pipeline`) but emits `Vec<CoreOp>` instructions instead
// of formatted text strings.

use crate::compaction::{extract_class_name, extract_field, extract_method_sig};
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::Fidelity;
use super::opcodes::*;

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

/// IR Compiler — translates tree-sitter captures into Core IR instructions.
pub struct IRCompiler {
    /// Running instruction counter for ID generation
    id_counter: u32,
}

impl IRCompiler {
    pub fn new() -> Self {
        Self { id_counter: 0 }
    }

    /// Compile source code into IR.
    /// Reuses the existing capture pipeline but emits CoreOp instructions
    /// instead of formatted text strings.
    pub fn compile(
        &mut self,
        source: &str,
        file_id: &str,
        language: tree_sitter::Language,
        query_string: &str,
        fidelity: Fidelity,
    ) -> Result<CompiledIR, Box<dyn std::error::Error>> {
        let captures = run_capture_pipeline(
            language,
            query_string,
            source,
            fidelity,
            |capture_name, raw, f| {
                // Use the existing compaction functions to normalize captured
                // text, same as the text-based pipeline does. This ensures
                // class names are extracted, method signatures are compacted,
                // and fields are properly formatted.
                match capture_name {
                    "class.root" => Some(extract_class_name(raw)),
                    "method.root" => Some(extract_method_sig(raw, f)),
                    "field.root" => Some(extract_field(raw, f)),
                    _ => Some(raw.to_string()),
                }
            },
        )?;

        let mut instructions = Vec::new();
        let mut current_class: Option<String> = None;

        for cap in &captures {
            match cap.name.as_str() {
                "class.root" => {
                    let class_id = self.next_id("C");
                    instructions.push(CoreOp::DefClass(
                        class_id.clone(),
                        cap.text.clone(),
                    ));
                    current_class = Some(class_id);
                }
                "method.root" => {
                    let class_id = current_class.clone().unwrap_or_default();
                    let method_id = self.next_id("M");
                    self.emit_method_ir(
                        &mut instructions,
                        &class_id,
                        &method_id,
                        &cap.text,
                    );
                }
                "field.root" => {
                    let class_id = current_class.clone().unwrap_or_default();
                    let field_id = self.next_id("F");
                    instructions.push(CoreOp::DefField(
                        class_id,
                        field_id.clone(),
                        cap.text.clone(),
                    ));
                }
                "import.root" => {
                    self.emit_import_ir(&mut instructions, &cap.text);
                }
                // Control flow captures → FLAGS on the most recent method
                "if.root" => {
                    if let Some(method_id) = find_last_method(&instructions) {
                        push_flag(&mut instructions, &method_id, FLAG_IF);
                    }
                }
                "for.root" | "while.root" => {
                    if let Some(method_id) = find_last_method(&instructions) {
                        push_flag(&mut instructions, &method_id, FLAG_LOOP);
                    }
                }
                "return.root" => {
                    if let Some(method_id) = find_last_method(&instructions) {
                        push_flag(&mut instructions, &method_id, FLAG_RET);
                    }
                }
                "throw.root" => {
                    if let Some(method_id) = find_last_method(&instructions) {
                        push_flag(&mut instructions, &method_id, FLAG_THROW);
                    }
                }
                _ => {}
            }
        }

        Ok(CompiledIR {
            file_id: file_id.to_string(),
            instructions,
            version: 1,
        })
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
        let (name, params_str, return_type) = parse_method_sig(raw_sig);

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

/// Parse a method signature string into (name, params_str, return_type).
///
/// Input: "processComplexData(payload:$s[],payload2:$n):$b"
/// Output: ("processComplexData", "payload:$s[],payload2:$n", "$b")
fn parse_method_sig(sig: &str) -> (String, String, String) {
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

    (name, params_str, if return_type.is_empty() { TYPE_VOID.to_string() } else { return_type })
}

/// Find the most recent DefMethod's ID in the instruction stream.
fn find_last_method(instructions: &[CoreOp]) -> Option<String> {
    instructions.iter().rev().find_map(|op| {
        if let CoreOp::DefMethod(_, id, _) = op {
            Some(id.clone())
        } else {
            None
        }
    })
}

/// Append a flag to an existing FLAGS instruction, or create a new one.
fn push_flag(instructions: &mut Vec<CoreOp>, target_id: &str, flag: &str) {
    // Check if there's already a FLAGS instruction for this target
    for op in instructions.iter_mut() {
        if let CoreOp::Flags(tid, flags) = op {
            if tid == target_id {
                if !flags.contains(&flag.to_string()) {
                    flags.push(flag.to_string());
                }
                return;
            }
        }
    }
    // No existing FLAGS — create a new one
    instructions.push(CoreOp::Flags(
        target_id.to_string(),
        vec![flag.to_string()],
    ));
}

impl Default for IRCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/ir/compiler.rs"]
mod tests;