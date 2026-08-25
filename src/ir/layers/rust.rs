// src/ir/layers/rust.rs
//
// Phase F: Layered Encoding — Rust Language Layer (Layer 2)
// Translates Rust-specific captures into additional IR instructions
// on top of the Core IR.
//
// Extracts:
//   - Struct/Enum/Trait declarations
//   - Trait implementations (impl Trait for Type)
//   - Method visibility and safety flags (pub, unsafe, async)
//   - Self kind (&self, &mut self, self, or associated function)
//
// R-43a: Execution semantics extraction:
//   - async fn → SideEffect("async") + ExecutionContext("async")
//   - unsafe block/fn → SideEffect("mutation")
//   - references to fields → DataFlow("reads"/"writes", field_name)
//   - match/loop/if/return patterns → ControlFlow

use super::{LanguageLayer, LayerContext};
use crate::compression::Fidelity;
use crate::ir::opcodes::{
    CTRL_IF, CTRL_LOOP, CTRL_MATCH, CTRL_RETURN, CTX_ASYNC, CoreOp, EFFECT_ASYNC, EFFECT_IO,
    EFFECT_MUTATION, FLAG_ASYNC, FLAG_EXPORT, FLAG_PRIVATE, FLAG_UNSAFE,
};

/// Rust visibility enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustVisibility {
    Public,  // pub
    Crate,   // pub(crate)
    Super,   // pub(super)
    Private, // (default)
}

/// Self kind for methods
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelfKind {
    None,   // associated function (no self)
    Ref,    // &self
    RefMut, // &mut self
    Owned,  // self (by value)
}

/// Rust type kind
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustTypeKind {
    Struct,
    Enum,
    Trait,
    Impl, // inherent impl or trait impl
}

/// Struct/Enum/Trait entry
#[derive(Debug, Clone)]
pub struct RustTypeEntry {
    pub kind: RustTypeKind,
    pub visibility: RustVisibility,
    pub derives: Vec<String>,
    pub generic_params: Option<String>,
    pub is_unsafe: bool,
}

/// Method entry
#[derive(Debug, Clone)]
pub struct RustMethodEntry {
    pub is_unsafe: bool,
    pub is_async: bool,
    pub visibility: RustVisibility,
    pub self_kind: SelfKind,
    pub generic_bounds: Option<String>,
}

/// Impl block entry
#[derive(Debug, Clone)]
pub struct RustImplEntry {
    pub trait_name: Option<String>, // None for inherent impl
    pub self_type: String,
    pub is_unsafe: bool,
    pub generic_params: Option<String>,
}

/// Rust language layer (Layer 2).
/// Processes Rust-specific captures and emits additional CoreOp instructions.
pub struct RustLayer;

impl RustLayer {
    pub fn new() -> Self {
        Self
    }

    /// Extract visibility from a Rust declaration.
    fn extract_visibility(decl: &str) -> RustVisibility {
        if decl.contains("pub(crate)") {
            RustVisibility::Crate
        } else if decl.contains("pub(super)") {
            RustVisibility::Super
        } else if decl.contains("pub ") || decl.starts_with("pub ") {
            RustVisibility::Public
        } else {
            RustVisibility::Private
        }
    }

    /// Extract trait implementations from impl blocks.
    /// Parses: "impl<T> Trait for Type"
    pub fn extract_impl_relationships(impl_head: &str) -> (Option<String>, Vec<String>) {
        let mut self_type = None;
        let mut traits = Vec::new();

        // Find " for " separator to identify trait impl
        if let Some(for_pos) = impl_head.find(" for ") {
            let trait_part = impl_head[..for_pos].trim();
            let type_part = impl_head[for_pos + 5..].trim();

            // Extract type (up to where or {)
            self_type = Some(
                type_part
                    .split_whitespace()
                    .next()
                    .unwrap_or(type_part)
                    .to_string(),
            );

            // Extract trait (after "impl<T>" or "impl")
            let after_impl = trait_part.strip_prefix("impl").unwrap_or(trait_part).trim();

            // Strategy: skip all generic parameters <...> (which may be nested),
            // then take the next identifier as the trait name.
            let mut depth = 0i32;
            let mut past_generics = after_impl;
            for (i, ch) in after_impl.char_indices() {
                match ch {
                    '<' => depth += 1,
                    '>' => {
                        depth -= 1;
                        if depth == 0 {
                            past_generics = after_impl[i + 1..].trim();
                            break;
                        }
                    }
                    _ => {}
                }
            }

            // If depth > 0, we never closed all generics - just use the whole thing
            if depth > 0 {
                past_generics = after_impl;
            }

            // Now extract the trait name: it's the identifier (up to < or whitespace or end)
            let trait_name = past_generics
                .split(|c: char| c == '<' || c.is_whitespace())
                .next()
                .unwrap_or(past_generics)
                .trim();

            if !trait_name.is_empty() {
                traits.push(trait_name.to_string());
            }
        }

        (self_type, traits)
    }

    /// Extract method-level flags (unsafe, async, visibility).
    fn extract_method_flags(raw_sig: &str) -> Vec<String> {
        let mut flags = Vec::new();
        if raw_sig.contains("unsafe") {
            flags.push(FLAG_UNSAFE.to_string());
        }
        if raw_sig.contains("async") {
            flags.push(FLAG_ASYNC.to_string());
        }
        if raw_sig.contains("pub ") || raw_sig.starts_with("pub ") {
            flags.push(FLAG_EXPORT.to_string());
        }
        flags
    }

    /// R-43a: Extract execution semantics from method body text.
    /// Returns (dataflow_ops, controlflow_ops, side_effect_op, context_op).
    fn extract_execution_semantics(method_id: &str, body: &str) -> Vec<CoreOp> {
        let mut ops = Vec::new();

        // SideEffect: detect unsafe keyword
        if body.contains("unsafe ") || body.contains("unsafe{") || body.contains("unsafe\n") {
            ops.push(CoreOp::SideEffect(
                method_id.to_string(),
                EFFECT_MUTATION.to_string(),
            ));
        }

        // SideEffect: detect I/O patterns (stdout, file operations, network)
        if body.contains("std::io")
            || body.contains("std::fs")
            || body.contains("std::net")
            || body.contains("println!")
            || body.contains("eprintln!")
            || body.contains("File::")
            || body.contains("TcpStream")
        {
            ops.push(CoreOp::SideEffect(
                method_id.to_string(),
                EFFECT_IO.to_string(),
            ));
        }

        // ControlFlow: detect match expressions
        if body.contains("match ") && !body.trim_start().starts_with("//") {
            ops.push(CoreOp::ControlFlow(
                method_id.to_string(),
                CTRL_MATCH.to_string(),
                "match_expr".to_string(),
            ));
        }

        // ControlFlow: detect loops
        let has_loop = body.contains("loop ")
            || body.contains("loop {")
            || body.contains("while ")
            || body.contains("for ");
        if has_loop {
            ops.push(CoreOp::ControlFlow(
                method_id.to_string(),
                CTRL_LOOP.to_string(),
                "loop".to_string(),
            ));
        }

        // ControlFlow: detect if/else
        if body.contains("if ") && !body.trim_start().starts_with("//") {
            ops.push(CoreOp::ControlFlow(
                method_id.to_string(),
                CTRL_IF.to_string(),
                "if".to_string(),
            ));
        }

        // ControlFlow: detect return
        if body.contains("return ") || body.contains("\nreturn") || body.starts_with("return") {
            ops.push(CoreOp::ControlFlow(
                method_id.to_string(),
                CTRL_RETURN.to_string(),
                "return".to_string(),
            ));
        }

        ops
    }

    /// Extract self kind from a Rust method signature.
    pub fn extract_self_kind(params: &str) -> SelfKind {
        if params.contains("&mut self") {
            SelfKind::RefMut
        } else if params.contains("&self") {
            SelfKind::Ref
        } else if params.starts_with("self")
            || params.contains(", self")
            || params.contains(",self")
        {
            SelfKind::Owned
        } else {
            SelfKind::None
        }
    }

    /// Extract derive attributes from source text.
    pub fn extract_derives(source: &str, type_start: usize) -> Vec<String> {
        // Look for #[derive(...)] before the type declaration
        let before = &source[..type_start];
        if let Some(derive_pos) = before.rfind("#[derive(") {
            let after_bracket = &before[derive_pos + 9..];
            // Find the closing paren of derive(...) before the ]
            if let Some(close_paren) = after_bracket.find(')') {
                let derives_str = &after_bracket[..close_paren];
                return derives_str
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    /// Extract cfg attributes from source text before a type declaration.
    /// Scans backward from `type_start` for `#[cfg(...)]`.
    /// Returns the cfg predicate(s) if found, e.g. "feature = \"unstable\"".
    pub fn extract_cfg(source: &str, type_start: usize) -> Option<String> {
        let before = &source[..type_start];
        if let Some(cfg_pos) = before.rfind("#[cfg(") {
            let after_bracket = &before[cfg_pos + 6..];
            // Find the closing paren of cfg(...)
            if let Some(close_paren) = after_bracket.find(')') {
                let cfg_str = after_bracket[..close_paren].trim();
                if !cfg_str.is_empty() {
                    return Some(cfg_str.to_string());
                }
            }
        }
        None
    }

    /// Extract generic parameter string from a Rust declaration.
    /// Finds the <...> segment in the text.
    /// For example: "MyStruct<T, U>" → Some("<T, U>").
    pub fn extract_generic_params(text: &str) -> Option<String> {
        if let Some(angle_start) = text.find('<') {
            let after = &text[angle_start + 1..];
            // Track nested angle brackets to find matching >
            let mut depth = 1u32;
            for (i, ch) in after.char_indices() {
                match ch {
                    '<' => depth += 1,
                    '>' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(format!("<{}>", &after[..i]));
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }
}

impl Default for RustLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageLayer for RustLayer {
    fn name(&self) -> &str {
        "rust"
    }

    fn process_capture(
        &mut self,
        capture_name: &str,
        raw_text: &str,
        context: &mut LayerContext,
    ) -> Vec<CoreOp> {
        let mut ops = Vec::new();

        match capture_name {
            "struct.root" | "enum.root" | "trait.root" => {
                if let Some(class_id) = &context.current_class {
                    // Emit class-level flags (visibility, safety)
                    let mut flags = Self::extract_method_flags(raw_text);

                    // Wire P2: Use extract_visibility for precise visibility flags.
                    let vis = Self::extract_visibility(raw_text);
                    match vis {
                        RustVisibility::Public => {
                            if !flags.contains(&FLAG_EXPORT.to_string()) {
                                flags.push(FLAG_EXPORT.to_string());
                            }
                        }
                        RustVisibility::Crate | RustVisibility::Super => {
                            if !flags.contains(&FLAG_EXPORT.to_string()) {
                                flags.push(FLAG_EXPORT.to_string());
                            }
                        }
                        RustVisibility::Private => {
                            if !flags.contains(&FLAG_PRIVATE.to_string()) {
                                flags.push(FLAG_PRIVATE.to_string());
                            }
                        }
                    }

                    // Check for unsafe trait specifically
                    if raw_text.contains("unsafe trait")
                        && !flags.contains(&FLAG_UNSAFE.to_string())
                    {
                        flags.push(FLAG_UNSAFE.to_string());
                    }

                    // ── Phase B (P3): Wire extract_cfg ────────────────────
                    if let Some(pos) = context.source.find(raw_text) {
                        if let Some(cfg_str) = Self::extract_cfg(&context.source, pos) {
                            let cfg_flag = format!("CFG({})", cfg_str);
                            if !flags.contains(&cfg_flag) {
                                flags.push(cfg_flag);
                            }
                        }
                    }

                    // ── Phase B (P4): Wire extract_generic_params ─────────
                    if context.fidelity != Fidelity::Low {
                        if let Some(generic_params) = Self::extract_generic_params(raw_text) {
                            let gp_flag = format!("GP{}", generic_params);
                            if !flags.contains(&gp_flag) {
                                flags.push(gp_flag);
                            }
                        }
                    }

                    if !flags.is_empty() {
                        ops.push(CoreOp::ClassFlags(class_id.clone(), flags));
                    }
                }
            }
            "impl.root" => {
                // Distinguish trait impls from inherent impls
                let (_self_type, traits) = Self::extract_impl_relationships(raw_text);
                if let Some(class_id) = &context.current_class {
                    // Emit Implements for trait implementations
                    for trait_name in &traits {
                        let trait_alias = context
                            .symbol_table
                            .alias_for(trait_name)
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| trait_name.clone());
                        ops.push(CoreOp::Implements(class_id.clone(), trait_alias));
                    }

                    // Emit class-level flags for unsafe impl
                    let mut flags = Self::extract_method_flags(raw_text);
                    if raw_text.contains("unsafe impl") && !flags.contains(&FLAG_UNSAFE.to_string())
                    {
                        flags.push(FLAG_UNSAFE.to_string());
                    }
                    if !flags.is_empty() {
                        ops.push(CoreOp::ClassFlags(class_id.clone(), flags));
                    }
                }
            }
            "method.root" => {
                // Extract method-level flags
                let method_flags = Self::extract_method_flags(raw_text);
                if let Some(method_id) = &context.current_method {
                    let is_async = method_flags.contains(&FLAG_ASYNC.to_string());
                    if !method_flags.is_empty() {
                        ops.push(CoreOp::Flags(method_id.clone(), method_flags));
                    }

                    // R-43a: Extract execution semantics from method body
                    // The raw_text here is the method signature + body.
                    // We detect async to emit SideEffect + ExecutionContext.
                    if is_async {
                        ops.push(CoreOp::SideEffect(
                            method_id.clone(),
                            EFFECT_ASYNC.to_string(),
                        ));
                        ops.push(CoreOp::ExecutionContext(
                            method_id.clone(),
                            CTX_ASYNC.to_string(),
                        ));
                    }

                    // Extract additional execution semantics from the signature+body text
                    let exec_ops = Self::extract_execution_semantics(method_id, raw_text);
                    ops.extend(exec_ops);
                }
            }
            // Rust mod declarations — structural, no IR ops needed
            "mod.root" | "type.root" | "import.root" => {}
            _ => {}
        }

        ops
    }

    fn finalize(&mut self, context: &mut LayerContext) -> Vec<CoreOp> {
        let _ = context;
        Vec::new()
    }
}

#[cfg(test)]
#[path = "../../tests/ir/layers/rust.rs"]
mod tests;
