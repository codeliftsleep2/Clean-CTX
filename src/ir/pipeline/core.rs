//! Core capture emission and language-layer finalization passes.

use super::{IRPass, PassContext, PassError, locate_method_body};
use crate::compaction::{
    extract_class_name, extract_field, extract_method_sig, extract_rust_struct_name,
};
use crate::compression::Fidelity;
use crate::compression::capture_pipeline::{CapturedNode, run_capture_pipeline_nodes};
use crate::ir::calls::{CALL_ARGUMENT_CAPTURE, CALL_CALLEE_CAPTURE, capture_query};
use crate::ir::opcodes::{CoreOp, FLAG_IF, FLAG_LOOP, FLAG_RET, FLAG_THROW};
use crate::ir::symbol_table::SymbolKind;

/// Pass 1: Core IR emission from tree-sitter captures.
pub struct CoreIRPass;

impl CoreIRPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CoreIRPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for CoreIRPass {
    fn name(&self) -> &str {
        "core_ir"
    }

    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        if state.source.is_empty() {
            return Ok(());
        }

        let language = state.language.clone().ok_or_else(|| PassError {
            pass_name: self.name().to_string(),
            message: "no tree-sitter language configured".to_string(),
        })?;
        let query_string = state.query_string.clone();
        let source = state.source.clone();
        let fidelity = state.fidelity;
        let file_id = state.file_id.clone();
        let skip_set = state.skip_set.clone();
        let focus = state.focus.clone();

        // ONE tree-sitter parse: the language's invocation-capture query (when a
        // native call producer exists) is compiled into the SAME query, so the
        // call facts are captured by the walk that already parses the file.
        let capture_query = capture_query(&query_string);

        let captures = run_capture_pipeline_nodes(
            language,
            &capture_query,
            &source,
            fidelity,
            |capture_name, raw, fidelity| match capture_name {
                "class.root" => Some(extract_class_name(raw)),
                "struct.root" | "trait.root" | "impl.root" => Some(extract_rust_struct_name(raw)),
                "enum.root" => {
                    if query_string == crate::queries::CS_QUERY {
                        Some(extract_class_name(raw))
                    } else {
                        Some(extract_rust_struct_name(raw))
                    }
                }
                "method.root" => Some(extract_method_sig(raw, fidelity)),
                "field.root" => Some(extract_field(raw, fidelity)),
                _ => Some(raw.to_string()),
            },
        )
        .map_err(|error| PassError {
            pass_name: self.name().to_string(),
            message: format!("capture pipeline error: {error}"),
        })?;

        for cap in &captures {
            // Callable-scope maintenance is independent of filtering: the walk
            // has advanced past `cap.start_byte` either way, so scopes whose
            // region closed are popped and their call facts settled.
            state.refresh_callable_owner(cap.start_byte);

            // The CBM filter-first skip test is defined on the public
            // `CapEntry` projection (it reads only the capture name and text),
            // so the richer node walk projects on demand. The projection is
            // only built on the skip path.
            if let Some(skip) = &skip_set
                && !skip.is_empty()
                && crate::compression::pipeline::should_skip_capture(&cap.to_entry(), skip)
            {
                continue;
            }
            match cap.name.as_str() {
                // Native call facts: the generic producer assembles
                // `CoreOp::Call` from the callee + argument captures of one
                // query match and settles them when the owning callable's
                // region closes.
                CALL_CALLEE_CAPTURE => state.record_call_callee(
                    cap.match_index,
                    cap.start_byte,
                    cap.end_byte,
                    &cap.text,
                ),
                CALL_ARGUMENT_CAPTURE => state.record_call_argument(cap.match_index),
                "class.root" | "interface.root" | "struct.root" | "enum.root" | "trait.root"
                | "record.root" => {
                    let class_id = state.next_id("C");
                    state
                        .instructions
                        .push(CoreOp::DefClass(class_id.clone(), cap.text.clone()));
                    state.push_type_scope(class_id.clone(), cap.end_byte);
                    state.layer_context.current_class_name = Some(cap.raw_text.clone());
                    state.layer_context.current_class_bare_name = Some(cap.text.clone());
                    state.layer_context.symbol_table_mut().register(
                        class_id,
                        cap.text.clone(),
                        SymbolKind::Class,
                        &file_id,
                    );
                    for layer in state.language_layers.iter_mut() {
                        state.instructions.extend(layer.process_capture(
                            &cap.name,
                            &cap.raw_text,
                            &mut state.layer_context,
                        ));
                    }
                }
                "impl.root" => process_impl_capture(state, cap),
                "method.root" | "constructor.root" | "func.root" | "arrow.root" => {
                    process_method_capture(state, cap, &file_id, fidelity, focus.as_ref());
                }
                "field.root" => {
                    state.refresh_type_owner(cap.start_byte);
                    let Some(class_id) = state.current_class.clone() else {
                        continue;
                    };
                    let field_id = state.next_id("F");
                    state
                        .instructions
                        .push(CoreOp::DefField(class_id, field_id, cap.text.clone()));
                    for layer in state.language_layers.iter_mut() {
                        state.instructions.extend(layer.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut state.layer_context,
                        ));
                    }
                }
                "import.root" | "package.root" => {
                    state.emit_import_ir(&cap.text);
                    for layer in state.language_layers.iter_mut() {
                        state.instructions.extend(layer.process_capture(
                            &cap.name,
                            &cap.text,
                            &mut state.layer_context,
                        ));
                    }
                }
                "type.root" => {
                    let alias_id = state.next_id("T");
                    state
                        .instructions
                        .push(CoreOp::TypeAlias(alias_id, cap.text.clone()));
                    dispatch_capture(state, cap, false);
                }
                "mod.root" => dispatch_capture(state, cap, false),
                "if.root" => push_flag(state, FLAG_IF),
                "for.root" | "while.root" | "loop.root" => push_flag(state, FLAG_LOOP),
                "return.root" => push_flag(state, FLAG_RET),
                "throw.root" => push_flag(state, FLAG_THROW),
                "do.root" | "try.root" | "switch.root" | "match.root" => {
                    push_flag(state, FLAG_IF);
                }
                _ => dispatch_capture(state, cap, false),
            }
        }

        // Settle the last callable's call facts BEFORE flushing its method
        // flags, so a caller's `CALL` ops stay adjacent to the caller's own
        // instruction region (the established trailing-Flags contract).
        state.flush_callable_calls();
        state.flush_method_flags();
        state.captures = captures.iter().map(CapturedNode::to_entry).collect();
        Ok(())
    }
}

fn process_impl_capture(state: &mut PassContext, cap: &CapturedNode) {
    let previous_class = state.current_class.clone();
    state.refresh_type_owner(cap.start_byte);
    match state.current_class.clone() {
        Some(class_id) => state.push_type_scope(class_id, cap.end_byte),
        None => {
            if let Some(class_id) = previous_class {
                state.push_type_scope(class_id, cap.end_byte);
            } else {
                let self_type = cap.text.split(':').next().unwrap_or(&cap.text).to_string();
                if !self_type.is_empty() {
                    let class_id = state.next_id("C");
                    state
                        .instructions
                        .push(CoreOp::DefClass(class_id.clone(), self_type.clone()));
                    state.push_type_scope(class_id, cap.end_byte);
                    state.layer_context.current_class_name = Some(cap.raw_text.clone());
                    state.layer_context.current_class_bare_name = Some(self_type);
                }
            }
        }
    }
    dispatch_capture(state, cap, true);
}

fn process_method_capture(
    state: &mut PassContext,
    cap: &CapturedNode,
    file_id: &str,
    fidelity: Fidelity,
    focus: Option<&std::collections::HashSet<String>>,
) {
    state.refresh_type_owner(cap.start_byte);
    let class_id = match state.current_class.clone() {
        Some(class_id) => class_id,
        None if cap.name == "func.root" || cap.name == "arrow.root" => {
            let class_id = state.next_id("C");
            let file_class = format!("__file_{file_id}");
            state
                .instructions
                .push(CoreOp::DefClass(class_id.clone(), file_class.clone()));
            state.push_file_scope(class_id.clone());
            state.layer_context.current_class_name = Some(file_class.clone());
            state.layer_context.current_class_bare_name = Some(file_class);
            class_id
        }
        None => return,
    };

    state.flush_method_flags();
    let method_id = state.next_id("M");
    state.current_method = Some(method_id.clone());
    // Native call facts: this declaration's source span owns every invocation
    // inside it (excluding any nested callable, which pushes its own scope).
    state.push_callable_scope(method_id.clone(), cap.start_byte, cap.end_byte);
    state.layer_context.current_method = Some(method_id.clone());
    state.layer_context.current_method_name = Some(cap.text.clone());
    let method_name = state.emit_method_ir(&class_id, &method_id, &cap.text);

    if fidelity == Fidelity::Edit
        && focus.is_none_or(|focused| focused.contains(&method_name))
        && let Some((body, body_offset)) = locate_method_body(&cap.raw_text)
    {
        state.instructions.push(CoreOp::Body(
            method_id,
            body,
            Some(cap.start_byte as u64 + body_offset as u64),
            Some(cap.end_byte as u64),
        ));
    }
    dispatch_capture(state, cap, true);
}

fn dispatch_capture(state: &mut PassContext, cap: &CapturedNode, use_raw_text: bool) {
    let text = if use_raw_text {
        &cap.raw_text
    } else {
        &cap.text
    };
    for layer in state.language_layers.iter_mut() {
        state
            .instructions
            .extend(layer.process_capture(&cap.name, text, &mut state.layer_context));
    }
}

fn push_flag(state: &mut PassContext, flag: &str) {
    if state.current_method.is_some() && !state.current_method_flags.iter().any(|item| item == flag)
    {
        state.current_method_flags.push(flag.to_string());
    }
}

/// Pass 2: Language layer finalization.
pub struct LanguageLayerPass;

impl LanguageLayerPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LanguageLayerPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for LanguageLayerPass {
    fn name(&self) -> &str {
        "language_layer"
    }

    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        for layer in state.language_layers.iter_mut() {
            state
                .instructions
                .extend(layer.finalize(&mut state.layer_context));
        }
        Ok(())
    }
}
