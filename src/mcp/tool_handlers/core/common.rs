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

/// The model-visible content category a response self-reports.
///
/// Serializes to the wire strings the LLM and verification drivers read;
/// [`ContentKind::as_str`] is the single source of truth for those strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ContentKind {
    /// Structural-only: names, signatures, no byte-exact regions.
    Skeleton,
    /// Edit fidelity: every method body is byte-exact.
    SkeletonWithVerbatimBodies,
    /// Edit + `focusMethods`: only focused method bodies are byte-exact.
    SkeletonWithFocusedVerbatimBodies,
    /// Verbatim fidelity: the whole document is byte-exact.
    VerbatimDocument,
    /// The economics gate selected byte-exact raw source.
    RawPassthrough,
    /// Human-readable acknowledgement of a code-side delta payload.
    DeltaSummary,
}

impl ContentKind {
    /// Wire string for tests — the same string the derived `Serialize` impl
    /// emits. Test-only: production serializes via `Serialize`, not this method.
    #[cfg(test)]
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ContentKind::Skeleton => "skeleton",
            ContentKind::SkeletonWithVerbatimBodies => "skeleton_with_verbatim_bodies",
            ContentKind::SkeletonWithFocusedVerbatimBodies => {
                "skeleton_with_focused_verbatim_bodies"
            }
            ContentKind::VerbatimDocument => "verbatim_document",
            ContentKind::RawPassthrough => "raw_passthrough",
            ContentKind::DeltaSummary => "delta_summary",
        }
    }
}

/// Derive the contract from the body coverage that is actually visible in a
/// regenerated hierarchy. This is intentionally independent of delta
/// transport: deltas remain code-side, while this helper describes only the
/// rendered SCHEMA-v5 text returned after a lifecycle operation.
pub(crate) fn contract_fields_for_hierarchy(
    fidelity: crate::compression::Fidelity,
    hierarchy: &crate::ir::hierarchical::HierarchicalIR,
) -> (ContentKind, Vec<&'static str>) {
    if fidelity != crate::compression::Fidelity::Edit {
        return contract_fields(fidelity);
    }

    let (method_count, body_count) = hierarchy
        .classes
        .iter()
        .flat_map(|class| class.methods.iter())
        .filter(|method| {
            !method
                .modifiers
                .iter()
                .flatten()
                .any(|modifier| *modifier == crate::ir::opcodes::DeclarationModifier::Abstract)
        })
        .fold((0usize, 0usize), |(methods, bodies), method| {
            (methods + 1, bodies + usize::from(method.body.is_some()))
        });
    match (body_count, method_count) {
        (0, _) => (ContentKind::Skeleton, Vec::new()),
        (bodies, methods) if bodies == methods => (
            ContentKind::SkeletonWithVerbatimBodies,
            vec!["method_bodies"],
        ),
        _ => (
            ContentKind::SkeletonWithFocusedVerbatimBodies,
            vec!["focused_method_bodies"],
        ),
    }
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
) -> (ContentKind, Vec<&'static str>) {
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
) -> (ContentKind, Vec<&'static str>) {
    match fidelity {
        crate::compression::Fidelity::Verbatim => (ContentKind::VerbatimDocument, vec!["document"]),
        // No focus set → every method body is byte-exact (legacy behavior).
        crate::compression::Fidelity::Edit if focus.is_none() => (
            ContentKind::SkeletonWithVerbatimBodies,
            vec!["method_bodies"],
        ),
        // Focus set but EMPTY → ZERO method bodies are byte-exact. The
        // output is effectively all-signatures, so report `Skeleton`
        // with no byte-exact regions (otherwise the LLM would attempt
        // replace_in_file SEARCH on bodies that don't exist).
        crate::compression::Fidelity::Edit if focus.is_some_and(HashSet::is_empty) => {
            (ContentKind::Skeleton, Vec::new())
        }
        // Focus set with names → only the focused method bodies are
        // byte-exact. The LLM must NOT attempt SEARCH on unfocused bodies.
        crate::compression::Fidelity::Edit => (
            ContentKind::SkeletonWithFocusedVerbatimBodies,
            vec!["focused_method_bodies"],
        ),
        _ => (ContentKind::Skeleton, Vec::new()),
    }
}
