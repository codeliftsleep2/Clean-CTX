// Shared contracts and helpers for core MCP handlers.

use crate::error::{CleanCtxError, to_jsonrpc_error};
use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::HierarchicalIR;
use crate::ir::opcodes::CoreOp;
use crate::ir::wire::tuple_to_op;
use crate::protocol::send_response;
use serde_json::Value;
use std::collections::HashSet;
pub(super) fn tuples_to_coreops(tuples: Vec<Vec<String>>) -> Result<Vec<CoreOp>, String> {
    tuples
        .into_iter()
        .enumerate()
        .map(|(position, tuple)| {
            tuple_to_op(&tuple)
                .ok_or_else(|| format!("invalid canonical tuple at position {position}: {tuple:?}"))
        })
        .collect()
}

pub(super) fn compiled_from_tuples(
    file_id: String,
    version: u64,
    tuples: Vec<Vec<String>>,
) -> Result<CompiledIR, String> {
    Ok(CompiledIR {
        file_id,
        version,
        instructions: tuples_to_coreops(tuples)?,
    })
}

pub(super) fn invalid_session_ir_response(id: &Value, message: &str) -> Value {
    crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        -32603,
        format!("Invalid session canonical IR: {message}"),
        None,
    )
}

/// Run the checked semantic-identity projection and map failures through the
/// established MCP IR error contract. Callers invoke this before storing or
/// rendering newly compiled hierarchical state. Existing reset/restore
/// lifecycle ordering remains unchanged by this projection slice.
pub(super) fn checked_hierarchy_or_respond(id: &Value, ir: &CompiledIR) -> Option<HierarchicalIR> {
    match crate::ir::hierarchical::try_ir_to_hierarchical(ir) {
        Ok(hierarchy) => Some(hierarchy),
        Err(error) => {
            send_response(&projection_error_response(id, &error));
            None
        }
    }
}

/// Resolve `focusMethods` through typed ownership, then retain exact body
/// opcodes only for the resulting canonical method IDs.
pub(super) fn resolve_focus_or_respond(
    id: &Value,
    ir: &mut CompiledIR,
    focus: Option<&HashSet<String>>,
) -> Option<HierarchicalIR> {
    let hierarchy = checked_hierarchy_or_respond(id, ir)?;
    let Some(selectors) = focus else {
        return Some(hierarchy);
    };
    let method_ids = match crate::ir::focus::resolve_focus_method_ids(&hierarchy, selectors) {
        Ok(ids) => ids,
        Err(error) => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32602,
                error.to_string(),
                None,
            ));
            return None;
        }
    };
    crate::ir::focus::retain_focused_bodies(ir, &method_ids);
    checked_hierarchy_or_respond(id, ir)
}

pub(crate) fn projection_error_response(
    id: &Value,
    error: &crate::ir::hierarchical::HierarchicalProjectionError,
) -> Value {
    let projection_code = error.code();
    let mapped = CleanCtxError::Ir(format!("hierarchical projection failed: {error}"));
    let mut response = to_jsonrpc_error(id, &mapped);
    response["error"]["data"]["projection_code"] = Value::String(projection_code.to_owned());
    response
}

/// Self-reporting contract fields (Gap 5/3/6 fixes).
///
/// Returns `(content_kind, byte_exact_regions)` describing what the
/// response contains so the LLM can tell structural-only output from
/// body-inclusive output without re-parsing the text.
///
/// - `content_kind`: `"skeleton"` (structural-only), `"skeleton_with_verbatim_bodies"`
///   (Edit — method bodies are byte-exact), or `"verbatim_document"`
///   (Verbatim — entire document byte-exact).
/// - `byte_exact`: which regions are safe for `replace_in_file` SEARCH
///   blocks. Edit → `["method_bodies"]`; Verbatim → `["document"]`;
///   others → `[]`.
pub(crate) fn contract_fields(
    fidelity: crate::compression::Fidelity,
) -> (&'static str, Vec<&'static str>) {
    contract_fields_focused(fidelity, None)
}

/// Self-reporting contract fields for `provide_code_context`, accounting for
/// symbol targeting via `focusMethods`.
///
/// When `focus` is `None` (no `focusMethods` supplied), `Edit` fidelity reports
/// `"skeleton_with_verbatim_bodies"`/`["method_bodies"]` — every method's body
/// is byte-exact (legacy behavior).
///
/// When `focus` is `Some(_)` (silently ignored unless the effective fidelity is
/// `Edit`), only the focused method bodies are byte-exact. The contract reports
/// `"skeleton_with_focused_verbatim_bodies"`/`["focused_method_bodies"]` so the
/// LLM knows NOT to attempt `replace_in_file` SEARCH on unfocused method bodies.
pub(crate) fn contract_fields_focused(
    fidelity: crate::compression::Fidelity,
    focus: Option<&HashSet<String>>,
) -> (&'static str, Vec<&'static str>) {
    match fidelity {
        crate::compression::Fidelity::Verbatim => ("verbatim_document", vec!["document"]),
        // No focus set → every method body is byte-exact (legacy behavior).
        crate::compression::Fidelity::Edit if focus.is_none() => {
            ("skeleton_with_verbatim_bodies", vec!["method_bodies"])
        }
        // Focus set but EMPTY → ZERO method bodies are byte-exact. The
        // output is effectively all-signatures, so report `"skeleton"`
        // with no byte-exact regions (otherwise the LLM would attempt
        // replace_in_file SEARCH on bodies that don't exist).
        crate::compression::Fidelity::Edit if focus.is_some_and(HashSet::is_empty) => {
            ("skeleton", Vec::new())
        }
        // Focus set with names → only the focused method bodies are
        // byte-exact. The LLM must NOT attempt SEARCH on unfocused bodies.
        crate::compression::Fidelity::Edit => (
            "skeleton_with_focused_verbatim_bodies",
            vec!["focused_method_bodies"],
        ),
        _ => ("skeleton", Vec::new()),
    }
}
