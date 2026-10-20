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

use super::declaration::{declaration_head, has_modifier};
use super::{LanguageLayer, LayerContext};
use crate::ir::opcodes::{
    CTRL_AWAIT, CTRL_TRY, CoreOp, DATAFLOW_READ, DATAFLOW_WRITE, DeclarationModifier,
    ExecutionContextKind, SideEffectKind,
};

/// Increment when identical TypeScript input can emit different canonical IR.
pub(crate) const CANONICAL_PRODUCER_GENERATION: u32 = 2;
/// Increment when TypeScript canonical facts change meaning for projection.
pub(crate) const SEMANTIC_INPUT_GENERATION: u32 = 3;

/// TypeScript language layer (Layer 2).
/// Processes TypeScript-specific captures and emits additional CoreOp instructions.
pub struct TypeScriptLayer;

impl TypeScriptLayer {
    pub fn new() -> Self {
        Self
    }

    /// Extract heritage from parser-owned clause nodes. Type-parameter
    /// constraints are outside class_heritage and cannot become heritage.
    fn extract_relationships(
        declaration_source: &str,
        interface: bool,
    ) -> (Vec<String>, Vec<String>) {
        let Some(language) = crate::compression::language::safe_typescript_language() else {
            return (Vec::new(), Vec::new());
        };
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&language).is_err() {
            return (Vec::new(), Vec::new());
        }
        let Some(tree) = parser.parse(declaration_source, None) else {
            return (Vec::new(), Vec::new());
        };
        if tree.root_node().has_error() {
            return (Vec::new(), Vec::new());
        }
        let kinds = if interface {
            &["interface_declaration"][..]
        } else {
            &["class_declaration", "abstract_class_declaration"][..]
        };
        let Some(declaration) = first_descendant(tree.root_node(), kinds) else {
            return (Vec::new(), Vec::new());
        };
        let mut declaration_cursor = declaration.walk();
        let heritage_kind = if interface {
            "extends_type_clause"
        } else {
            "class_heritage"
        };
        let Some(heritage) = declaration
            .named_children(&mut declaration_cursor)
            .find(|child| child.kind() == heritage_kind)
        else {
            return (Vec::new(), Vec::new());
        };
        let mut extends = Vec::new();
        let mut implements = Vec::new();
        if interface {
            extend_clause_values(&mut extends, heritage, declaration_source);
            return (extends, implements);
        }
        let mut cursor = heritage.walk();
        for clause in heritage.named_children(&mut cursor) {
            let target = match clause.kind() {
                "extends_clause" => &mut extends,
                "implements_clause" => &mut implements,
                _ => continue,
            };
            extend_clause_values(target, clause, declaration_source);
        }
        (extends, implements)
    }
    /// Extract class-level flags (export, abstract) from class head.
    fn extract_class_modifiers(class_head: &str) -> Vec<DeclarationModifier> {
        let head = declaration_head(class_head);
        let mut flags = Vec::new();
        if has_modifier(head, "export") {
            flags.push(DeclarationModifier::Export);
        }
        if has_modifier(head, "abstract") {
            flags.push(DeclarationModifier::Abstract);
        }
        flags
    }

    /// Extract method-level flags (async, generator, visibility) from method signature.
    fn extract_method_modifiers(raw_sig: &str) -> Vec<DeclarationModifier> {
        let head = declaration_head(raw_sig);
        let mut flags = Vec::new();
        if has_modifier(head, "async") {
            flags.push(DeclarationModifier::Async);
        }
        if head.contains('*') && has_modifier(head, "function") {
            flags.push(DeclarationModifier::Generator);
        }
        if has_modifier(head, "private") {
            flags.push(DeclarationModifier::Private);
        }
        if has_modifier(head, "protected") {
            flags.push(DeclarationModifier::Protected);
        }
        if has_modifier(head, "static") {
            flags.push(DeclarationModifier::Static);
        }
        flags
    }

    /// R-43a: Extract execution semantics from TypeScript method body text.
    fn extract_method_execution_semantics(method_id: &str, raw_sig: &str) -> Vec<CoreOp> {
        let mut ops = Vec::new();

        // Detect async method
        if raw_sig.contains("async") {
            ops.push(CoreOp::SideEffect(
                method_id.to_string(),
                SideEffectKind::Async,
            ));
            ops.push(CoreOp::ExecutionContext(
                method_id.to_string(),
                ExecutionContextKind::Async,
            ));
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
            ops.push(CoreOp::SideEffect(
                method_id.to_string(),
                SideEffectKind::Io,
            ));
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

fn extend_clause_values(target: &mut Vec<String>, clause: tree_sitter::Node<'_>, source: &str) {
    let Some(text) = clause.utf8_text(source.as_bytes()).ok().and_then(|text| {
        text.split_once(char::is_whitespace)
            .map(|(_, values)| values)
    }) else {
        return;
    };
    target.extend(
        crate::meta_util::split_top_level(text.trim(), ',')
            .into_iter()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty()),
    );
}
fn first_descendant<'tree>(
    node: tree_sitter::Node<'tree>,
    kinds: &[&str],
) -> Option<tree_sitter::Node<'tree>> {
    if kinds.contains(&node.kind()) {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Some(found) = first_descendant(child, kinds) {
            return Some(found);
        }
    }
    None
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
                let (bases, interfaces) = Self::extract_relationships(raw_text, false);
                if let Some(class_id) = &context.current_class {
                    if let Some(base_id) = bases.into_iter().next() {
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
                    let modifiers = Self::extract_class_modifiers(raw_text);
                    if !modifiers.is_empty() {
                        ops.push(CoreOp::ClassModifiers(class_id.clone(), modifiers));
                    }
                }
            }
            "interface.root" => {
                if let Some(interface_id) = &context.current_interface {
                    let (parents, _) = Self::extract_relationships(raw_text, true);
                    for parent in parents {
                        ops.push(CoreOp::InterfaceExtends(interface_id.clone(), parent));
                    }
                    let modifiers = Self::extract_class_modifiers(raw_text);
                    if !modifiers.is_empty() {
                        ops.push(CoreOp::InterfaceModifiers(interface_id.clone(), modifiers));
                    }
                }
            }
            "method.root" => {
                // Extract method-level flags (async, generator, visibility)
                let modifiers = Self::extract_method_modifiers(raw_text);
                if let Some(method_id) = &context.current_method {
                    if !modifiers.is_empty() {
                        ops.push(CoreOp::MethodModifiers(method_id.clone(), modifiers));
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
