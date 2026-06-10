// src/ir/layers/csharp.rs
//
// Phase F: Layered Encoding — C# Language Layer (Layer 2)
// Translates C#-specific captures into additional IR instructions
// on top of the Core IR.
//
// Extracts:
//   - Class inheritance (extends) from class declarations
//   - Interface implementations
//   - Attributes/annotations
//   - Nullable type markers
//   - async/static/abstract/virtual flags

use super::{LanguageLayer, LayerContext};
use crate::ir::opcodes::{
    CoreOp, FLAG_ABSTRACT, FLAG_ASYNC, FLAG_EXPORT, FLAG_PRIVATE, FLAG_PROTECTED, FLAG_STATIC,
};

/// C# language layer (Layer 2).
/// Processes C#-specific captures and emits additional CoreOp instructions.
pub struct CSharpLayer;

impl CSharpLayer {
    pub fn new() -> Self {
        Self
    }

    /// Extract class inheritance from a C# class declaration.
    /// Parses: "public class MyClass : BaseClass, IInterface1, IInterface2"
    fn extract_class_relationships(class_head: &str) -> (Option<String>, Vec<String>) {
        let mut base: Option<String> = None;
        let mut interfaces: Vec<String> = Vec::new();

        // Find ":" separator (C# uses colon for inheritance)
        if let Some(colon_pos) = class_head.find(':') {
            let after_colon = class_head[colon_pos + 1..].trim_start();
            // Split by comma
            let mut current = String::new();
            let mut first = true;
            for ch in after_colon.chars() {
                if ch == ',' {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        if first {
                            base = Some(trimmed);
                            first = false;
                        } else {
                            interfaces.push(trimmed);
                        }
                    }
                    current.clear();
                } else if ch == '{' || ch == '\n' || ch == '\r' {
                    break;
                } else {
                    current.push(ch);
                }
            }
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                if first {
                    base = Some(trimmed);
                } else {
                    interfaces.push(trimmed);
                }
            }
        }

        (base, interfaces)
    }

    /// Extract class-level flags (public/abstract/static).
    fn extract_class_flags(class_head: &str) -> Vec<String> {
        let mut flags = Vec::new();
        if class_head.starts_with("public ") || class_head.contains(" public ") {
            flags.push(FLAG_EXPORT.to_string());
        }
        if class_head.contains("abstract ") {
            flags.push(FLAG_ABSTRACT.to_string());
        }
        if class_head.contains("static ") {
            flags.push(FLAG_STATIC.to_string());
        }
        flags
    }

    /// Extract method-level flags (async, static, virtual, override, visibility).
    fn extract_method_flags(raw_sig: &str) -> Vec<String> {
        let mut flags = Vec::new();
        if raw_sig.contains("async") {
            flags.push(FLAG_ASYNC.to_string());
        }
        if raw_sig.contains("private") {
            flags.push(FLAG_PRIVATE.to_string());
        }
        if raw_sig.contains("protected") {
            flags.push(FLAG_PROTECTED.to_string());
        }
        if raw_sig.contains("static") {
            flags.push(FLAG_STATIC.to_string());
        }
        if raw_sig.contains("abstract") {
            flags.push(FLAG_ABSTRACT.to_string());
        }
        flags
    }
}

impl Default for CSharpLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageLayer for CSharpLayer {
    fn name(&self) -> &str {
        "csharp"
    }

    fn process_capture(
        &mut self,
        capture_name: &str,
        raw_text: &str,
        context: &mut LayerContext,
    ) -> Vec<CoreOp> {
        let mut ops = Vec::new();

        match capture_name {
            "class.root" => {
                // Extract inheritance from raw text
                let (base, interfaces) = Self::extract_class_relationships(raw_text);
                if let Some(class_id) = &context.current_class {
                    // Emit Extends
                    if let Some(base_id) = base {
                        let base_alias = context
                            .symbol_table
                            .alias_for(&base_id)
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| base_id.clone());
                        ops.push(CoreOp::Extends(class_id.clone(), base_alias));
                    }
                    // Emit Implements for each interface
                    for iface in &interfaces {
                        let iface_alias = context
                            .symbol_table
                            .alias_for(iface)
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| iface.clone());
                        ops.push(CoreOp::Implements(class_id.clone(), iface_alias));
                    }

                    // Emit class-level flags
                    let class_flags = Self::extract_class_flags(raw_text);
                    if !class_flags.is_empty() {
                        ops.push(CoreOp::ClassFlags(class_id.clone(), class_flags));
                    }
                }
            }
            "method.root" => {
                // Extract method-level flags
                let method_flags = Self::extract_method_flags(raw_text);
                if let Some(method_id) = &context.current_method {
                    if !method_flags.is_empty() {
                        ops.push(CoreOp::Flags(method_id.clone(), method_flags));
                    }
                }
            }
            _ => {}
        }

        ops
    }

    fn finalize(&mut self, context: &mut LayerContext) -> Vec<CoreOp> {
        let _ = context;
        Vec::new()
    }
}