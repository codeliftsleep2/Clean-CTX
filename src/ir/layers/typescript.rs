// src/ir/layers/typescript.rs
//
// Phase F: Layered Encoding — TypeScript Language Layer (Layer 2)
// Translates TypeScript-specific captures into additional IR instructions
// on top of the Core IR.
//
// Extracts:
//   - Class extends/implements from class declarations
//   - async/generator flags from method signatures
//   - Abstract/export class-level flags
//   - Visibility modifiers (private, protected, static)
//   - Constructor injection patterns
//
// R-43a: Execution semantics extraction:
//   - .subscribe() → DataFlow("reads", "observable")
//   - .pipe() with tap → SideEffect("io")
//   - .pipe() with map/filter → DataFlow("reads", "observable")
//   - async keyword → SideEffect("async") + ExecutionContext("async")
//   - new Observable() → DataFlow("writes", "observable")
//   - @Injectable() → ExecutionContext("di_scope")

use super::{LanguageLayer, LayerContext};
use crate::ir::opcodes::{
    CoreOp, FLAG_ABSTRACT, FLAG_ASYNC, FLAG_EXPORT, FLAG_GEN, FLAG_PRIVATE, FLAG_PROTECTED,
    FLAG_STATIC,
    DATAFLOW_READ, DATAFLOW_WRITE, EFFECT_ASYNC, EFFECT_IO,
    CTX_ASYNC, CTRL_AWAIT, CTRL_TRY,
};

/// TypeScript language layer (Layer 2).
/// Processes TypeScript-specific captures and emits additional CoreOp instructions.
pub struct TypeScriptLayer;

impl TypeScriptLayer {
    pub fn new() -> Self {
        Self
    }

    /// Extract extends/implements from a class head string.
    /// Processes text like "class Foo extends Bar implements Baz, Qux"
    fn extract_class_relationships(class_head: &str) -> (Option<String>, Vec<String>) {
        let mut base: Option<String> = None;
        let mut interfaces: Vec<String> = Vec::new();

        // Find "extends" keyword
        if let Some(ext_pos) = class_head.find("extends") {
            let after_ext = class_head[ext_pos + 7..].trim_start();
            // Find the end of the extended class (next keyword or boundary)
            let mut end_pos = 0;
            for (i, ch) in after_ext.char_indices() {
                if ch == ',' || ch == '{' {
                    end_pos = i;
                    break;
                }
                if ch.is_whitespace() {
                    // Check if next word is "implements"
                    let rest = after_ext[i..].trim_start();
                    if rest.starts_with("implements") {
                        end_pos = i;
                        break;
                    }
                }
            }
            if end_pos > 0 {
                base = Some(after_ext[..end_pos].trim().to_string());
            } else {
                base = Some(after_ext.trim().to_string());
            }
        }

        // Find "implements" keyword
        if let Some(imp_pos) = class_head.find("implements") {
            let after_imp = class_head[imp_pos + 10..].trim_start();
            // Split by comma for multiple interfaces
            let mut current = String::new();
            for ch in after_imp.chars() {
                if ch == ',' {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        interfaces.push(trimmed);
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
                interfaces.push(trimmed);
            }
        }

        (base, interfaces)
    }

    /// Extract class-level flags (export, abstract) from class head.
    fn extract_class_flags(class_head: &str) -> Vec<String> {
        let mut flags = Vec::new();
        if class_head.contains("export ") || class_head.starts_with("export") {
            flags.push(FLAG_EXPORT.to_string());
        }
        if class_head.contains("abstract ") || class_head.starts_with("abstract") {
            flags.push(FLAG_ABSTRACT.to_string());
        }
        flags
    }

    /// Extract method-level flags (async, generator, visibility) from method signature.
    fn extract_method_flags(raw_sig: &str) -> Vec<String> {
        let mut flags = Vec::new();
        if raw_sig.contains("async") {
            flags.push(FLAG_ASYNC.to_string());
        }
        if raw_sig.contains('*') && raw_sig.contains("function") {
            flags.push(FLAG_GEN.to_string());
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
        flags
    }

    /// R-43a: Extract execution semantics from TypeScript method body text.
    fn extract_method_execution_semantics(method_id: &str, raw_sig: &str) -> Vec<CoreOp> {
        let mut ops = Vec::new();

        // Detect async method
        if raw_sig.contains("async") {
            ops.push(CoreOp::SideEffect(method_id.to_string(), EFFECT_ASYNC.to_string()));
            ops.push(CoreOp::ExecutionContext(method_id.to_string(), CTX_ASYNC.to_string()));
        }

        // RxJS: detect .subscribe() → DataFlow("reads", "observable")
        if raw_sig.contains(".subscribe(") || raw_sig.contains(".subscribe (") {
            ops.push(CoreOp::DataFlow(
                method_id.to_string(),
                DATAFLOW_READ.to_string(),
                "observable".to_string(),
            ));
        }

        // RxJS: detect .pipe() with tap → SideEffect("io")
        if raw_sig.contains(".pipe(") && raw_sig.contains("tap(") {
            ops.push(CoreOp::SideEffect(method_id.to_string(), EFFECT_IO.to_string()));
        }

        // RxJS: detect .pipe() with map/filter → DataFlow("reads", "observable")
        if raw_sig.contains(".pipe(") && (raw_sig.contains("map(") || raw_sig.contains("filter(")) {
            ops.push(CoreOp::DataFlow(
                method_id.to_string(),
                DATAFLOW_READ.to_string(),
                "observable".to_string(),
            ));
        }

        // RxJS: detect new Observable() → DataFlow("writes", "observable")
        if raw_sig.contains("new Observable(") || raw_sig.contains("new Observable<") {
            ops.push(CoreOp::DataFlow(
                method_id.to_string(),
                DATAFLOW_WRITE.to_string(),
                "observable".to_string(),
            ));
        }

        // Angular: detect @Injectable() → ExecutionContext("di_scope")
        if raw_sig.contains("@Injectable") {
            ops.push(CoreOp::ExecutionContext(
                method_id.to_string(),
                "di_scope".to_string(),
            ));
        }

        // Detect await
        if raw_sig.contains("await ") {
            ops.push(CoreOp::ControlFlow(
                method_id.to_string(),
                CTRL_AWAIT.to_string(),
                "await".to_string(),
            ));
        }

        // Detect try/catch
        if raw_sig.contains("try ") || raw_sig.contains("try{") || raw_sig.contains("try\n") {
            ops.push(CoreOp::ControlFlow(
                method_id.to_string(),
                CTRL_TRY.to_string(),
                "try".to_string(),
            ));
        }

        ops
    }
}

impl Default for TypeScriptLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageLayer for TypeScriptLayer {
    fn name(&self) -> &str {
        "typescript"
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
                // Extract extends/implements from raw text
                let (base, interfaces) = Self::extract_class_relationships(raw_text);
                if let Some(class_id) = &context.current_class {
                    // Emit Extends
                    if let Some(base_id) = base {
                        // Look up the base class in the symbol table
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

                    // R-43a: Detect @Injectable() on class → ExecutionContext("di_scope")
                    if raw_text.contains("@Injectable") {
                        ops.push(CoreOp::ExecutionContext(
                            class_id.clone(),
                            "di_scope".to_string(),
                        ));
                    }
                }
            }
            "method.root" => {
                // Extract method-level flags (async, generator, visibility)
                let method_flags = Self::extract_method_flags(raw_text);
                if let Some(method_id) = &context.current_method {
                    if !method_flags.is_empty() {
                        ops.push(CoreOp::Flags(method_id.clone(), method_flags));
                    }

                    // R-43a: Extract execution semantics
                    let exec_ops = Self::extract_method_execution_semantics(method_id, raw_text);
                    ops.extend(exec_ops);
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