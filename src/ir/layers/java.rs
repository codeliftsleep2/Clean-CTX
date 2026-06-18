// src/ir/layers/java.rs
//
// FAANG audit: NEW — Java Language Layer (Layer 2)
// Translates Java-specific captures into additional IR instructions
// on top of the Core IR.
//
// Extracts:
//   - Class/Interface/Enum/Record inheritance (extends) from declarations
//   - Interface implementations
//   - Class-level flags (public, abstract, static)
//   - Method-level flags (async, static, abstract, visibility)
//   - Constructor injection patterns

use super::{LanguageLayer, LayerContext};
use crate::ir::opcodes::{
    CoreOp, FLAG_ABSTRACT, FLAG_EXPORT, FLAG_PRIVATE, FLAG_PROTECTED, FLAG_STATIC,
};

/// Java language layer (Layer 2).
/// Processes Java-specific captures and emits additional CoreOp instructions.
pub struct JavaLayer;

impl JavaLayer {
    pub fn new() -> Self {
        Self
    }

    /// Extract extends/implements from a Java class/interface/enum head.
    /// Parses: "public class MyService extends BaseService implements Serializable"
    fn extract_class_relationships(class_head: &str) -> (Option<String>, Vec<String>) {
        let mut base: Option<String> = None;
        let mut interfaces: Vec<String> = Vec::new();

        // Find "extends" keyword
        if let Some(ext_pos) = class_head.find("extends") {
            let after_ext = class_head[ext_pos + 7..].trim_start();
            // Take up to next keyword or opening brace
            let base_name = after_ext.split_once("implements")
                .map(|(name, _)| name.trim())
                .or_else(|| after_ext.split_once('{').map(|(name, _)| name.trim()))
                .unwrap_or(after_ext.trim());
            // Strip generic parameters
            let bare = base_name.split('<').next().unwrap_or(base_name).trim().to_string();
            if !bare.is_empty() {
                base = Some(bare);
            }
        }

        // Find "implements" keyword
        if let Some(imp_pos) = class_head.find("implements") {
            let after_imp = class_head[imp_pos + 10..].trim_start();
            let mut current = String::new();
            for ch in after_imp.chars() {
                if ch == ',' {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        interfaces.push(trimmed.split('<').next().unwrap_or(&trimmed).trim().to_string());
                    }
                    current.clear();
                } else if ch == '{' {
                    break;
                } else {
                    current.push(ch);
                }
            }
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                interfaces.push(trimmed.split('<').next().unwrap_or(&trimmed).trim().to_string());
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

    /// Extract method-level flags (static, abstract, visibility).
    fn extract_method_flags(raw_sig: &str) -> Vec<String> {
        let mut flags = Vec::new();
        if raw_sig.contains("public") && !raw_sig.contains("native") {
            flags.push(FLAG_EXPORT.to_string());
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

impl Default for JavaLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageLayer for JavaLayer {
    fn name(&self) -> &str {
        "java"
    }

    fn process_capture(
        &mut self,
        capture_name: &str,
        raw_text: &str,
        context: &mut LayerContext,
    ) -> Vec<CoreOp> {
        let mut ops = Vec::new();

        match capture_name {
            "class.root" | "interface.root" | "enum.root" | "record.root" => {
                // Extract extends/implements from raw text
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
            "method.root" | "constructor.root" => {
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