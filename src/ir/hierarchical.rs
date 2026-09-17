// src/ir/hierarchical.rs
//
// Phase II: Scoped Hierarchical IR (Idea #4).
//
// Replaces the flat instruction array with a class→method→param tree,
// eliminating all opcode strings and parent ID repetitions. The same
// CoreOp semantics are preserved — this is purely a wire format change.
//
// Estimated savings: 40-60% reduction in wire bytes vs. positional encoding.

use super::compiler::CompiledIR;
use super::opcodes::CoreOp;
use super::wire::DecodeError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

mod decode;
mod encode;

// Re-exported so the established public paths (`crate::ir::hierarchical::
// ir_to_hierarchical`, `::hierarchical_to_ir`) are unchanged by the split
// into `hierarchical/encode.rs` and `hierarchical/decode.rs`.
pub use decode::hierarchical_to_ir;
pub use encode::ir_to_hierarchical;

/// Top-level hierarchical IR container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchicalIR {
    /// Classes (each contains methods, fields, relationships, etc.)
    #[serde(rename = "c")]
    pub classes: Vec<ClassNode>,

    /// Top-level imports — flat array of [alias, module, named]
    #[serde(rename = "i", default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<Vec<String>>,

    /// Type aliases — flat array of [alias, original]
    #[serde(rename = "t", default, skip_serializing_if = "Vec::is_empty")]
    pub type_aliases: Vec<Vec<String>>,

    /// Structural invocations (native call graph).
    ///
    /// Flat, like `imports` and `type_aliases`: the caller is already an
    /// explicit method id, so the class→method nesting adds no shared
    /// context to compress, and the flat entry mirrors `CoreOp::Call`
    /// losslessly (caller, callee NAME, written argument count, and the
    /// spread qualifier when the written count is not an exact arity). Typed
    /// (not a string tuple) so a malformed document fails decoding loudly
    /// instead of silently defaulting an argument count.
    #[serde(rename = "ca", default, skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<HierarchicalCall>,
}

/// One structural invocation in the flat hierarchical call table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchicalCall {
    /// Caller method alias id (e.g. "M1").
    #[serde(rename = "c")]
    pub caller: String,

    /// Callee textual name as written at the call site (never a resolved
    /// declaration identity).
    #[serde(rename = "n")]
    pub callee: String,

    /// Written argument count at the call site.
    #[serde(rename = "a")]
    pub explicit_arg_count: usize,

    /// Whether at least one written argument expands at run time.
    ///
    /// `true` means `explicit_arg_count` counts argument NODES and is never the
    /// runtime/declared arity, so a consumer must not read it as exact-arity
    /// evidence. `false` for every exact call (C#, Java, and TypeScript calls
    /// with no spread argument). Absent in the serialized form when `false`,
    /// so an exact call keeps the pre-qualifier shape byte-for-byte, and an
    /// older document that omits the field decodes as exact.
    #[serde(rename = "s", default, skip_serializing_if = "spread_is_absent")]
    pub has_spread: bool,
}

/// `skip_serializing_if` predicate for [`HierarchicalCall::has_spread`]: an
/// exact call must serialize exactly as it did before the qualifier existed.
fn spread_is_absent(has_spread: &bool) -> bool {
    !has_spread
}

/// A single class node — the top-level structural container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassNode {
    /// Class alias ID (e.g., "C1")
    #[serde(rename = "n")]
    pub id: String,

    /// Original class name
    #[serde(rename = "nm")]
    pub name: String,

    /// Methods within this class
    #[serde(rename = "m", default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<MethodNode>,

    /// Fields within this class
    #[serde(rename = "f", default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldNode>,

    /// Class-level flags (EXPORT, ABSTRACT, etc.)
    #[serde(rename = "fl", default, skip_serializing_if = "Option::is_none")]
    pub class_flags: Option<Vec<String>>,

    /// Extends (parent class alias ID)
    #[serde(rename = "x", default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Implements (interface alias IDs)
    #[serde(rename = "im", default, skip_serializing_if = "Vec::is_empty")]
    pub implements: Vec<String>,

    /// Injections (dependency aliases)
    #[serde(rename = "ij", default, skip_serializing_if = "Vec::is_empty")]
    pub injects: Vec<String>,

    /// Class-level pattern ops (e.g., CTOR)
    #[serde(rename = "p", default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<PatternEntry>,

    /// True if this class was synthesized (no DefClass in original stream).
    /// Synthetic classes emit NO DefClass instruction during hierarchical_to_ir.
    #[serde(rename = "sy", default, skip_serializing_if = "is_false")]
    pub synthetic: bool,
}

/// Helper serde skip for false booleans.
fn is_false(v: &bool) -> bool {
    !*v
}

/// A single method node — nested inside a class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodNode {
    /// Method alias ID (e.g., "M1")
    #[serde(rename = "n")]
    pub id: String,

    /// Original method name
    #[serde(rename = "nm")]
    pub name: String,

    /// Parameters — array of [param_id, type, name]
    #[serde(rename = "p", default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Vec<String>>,

    /// Return type
    #[serde(rename = "r", default, skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,

    /// Method-level flags (IF, LOOP, RET, etc.)
    #[serde(rename = "fl", default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,

    /// Method-level pattern ops
    #[serde(rename = "pa", default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<PatternEntry>,

    /// Verbatim method body text — byte-exact copy from source.
    /// Only present when `Fidelity::Edit` was used to compile.
    #[serde(rename = "b", default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// apply_edit plan Phase 1: absolute byte offset of the body slice's
    /// start within the source that produced this IR. Present iff `body`
    /// is present AND the producing IR carried spans (legacy span-less
    /// bodies decode without offsets).
    #[serde(rename = "bs", default, skip_serializing_if = "Option::is_none")]
    pub body_start: Option<u64>,

    /// Absolute byte offset one past the end of the body slice. Present
    /// iff `body_start` is present (pairing invariant).
    #[serde(rename = "be", default, skip_serializing_if = "Option::is_none")]
    pub body_end: Option<u64>,

    /// Control-flow metadata: [kind, target] tuples (R-43a).
    /// kind: "if" | "loop" | "match" | "try" | "await" | "return"
    #[serde(rename = "cf", default, skip_serializing_if = "Vec::is_empty")]
    pub control_flow: Vec<Vec<String>>,

    /// Data-flow metadata: [direction, target] tuples (R-43a).
    /// direction: "reads" | "writes"
    #[serde(rename = "df", default, skip_serializing_if = "Vec::is_empty")]
    pub data_flow: Vec<Vec<String>>,

    /// Side-effect annotation (R-43a).
    /// effect_type: "pure" | "io" | "mutation" | "async" | "transaction"
    #[serde(rename = "se", default, skip_serializing_if = "Option::is_none")]
    pub side_effect: Option<String>,

    /// Execution context annotation (R-43a).
    /// context_type: "sync" | "async" | "thread_bound" | "transaction_scope" | "realtime"
    #[serde(rename = "ec", default, skip_serializing_if = "Option::is_none")]
    pub execution_context: Option<String>,
}

/// A single field node — nested inside a class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldNode {
    /// Field alias ID (e.g., "F1")
    #[serde(rename = "n")]
    pub id: String,

    /// Original field name
    #[serde(rename = "nm")]
    pub name: String,

    /// Field type
    #[serde(rename = "t", default, skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
}

/// A pattern entry — compressed structural pattern (CTOR, GETTER, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternEntry {
    /// Pattern name (e.g., "CTOR", "GETTER")
    #[serde(rename = "n")]
    pub name: String,

    /// Pattern args (metadata) — stored as-is from the original Pattern op.
    /// The hierarchical format does NOT add/remove CID/MID prefixes.
    #[serde(rename = "a", default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

/// Encode a compiled IR into the hierarchical wire format (JSON).
///
/// Example output:
/// ```json
/// {
///   "file": "α1", "v": 1, "encoding": "hierarchical",
///   "ir": {
///     "c": [{
///       "n": "C1", "nm": "SampleService",
///       "m": [{
///         "n": "M1", "nm": "processComplexData",
///         "p": [["P1", "$s", "payload"]],
///         "r": "$b", "fl": ["IF"]
///       }],
///       "f": [{"n": "F1", "nm": "items", "t": "$s[]"}],
///       "im": ["IF1"]
///     }],
///     "i": [["IM1", "./module", "Foo"]]
///   }
/// }
/// ```
pub fn ir_to_hierarchical_wire(ir: &CompiledIR) -> Value {
    let hir = ir_to_hierarchical(ir);
    json!({
        "file": ir.file_id,
        "v": ir.version,
        "encoding": "hierarchical",
        "ir": hir
    })
}

/// Decode a hierarchical wire format value back into a `CompiledIR`.
///
/// Returns `Err(DecodeError)` if the input is malformed.
pub fn wire_to_ir(value: &Value) -> Result<CompiledIR, DecodeError> {
    let file_id = value
        .get("file")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| DecodeError::MissingField("file".into()))?;
    let version = value
        .get("v")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| DecodeError::MissingField("v".into()))?;

    let ir_val = value
        .get("ir")
        .ok_or_else(|| DecodeError::MissingField("ir".into()))?;

    // Deserialize via serde
    let hir: HierarchicalIR = serde_json::from_value(ir_val.clone())
        .map_err(|e| DecodeError::InvalidInput(format!("hierarchical decode: {}", e)))?;

    let instructions = hierarchical_to_ir(&hir);

    Ok(CompiledIR {
        file_id,
        instructions,
        version,
    })
}

/// Estimate character savings of hierarchical format vs. positional encoding.
///
/// Returns (positional_chars, hierarchical_chars, savings_pct).
pub fn estimate_savings(ir: &CompiledIR) -> (usize, usize, f64) {
    use super::wire::ir_to_wire;
    let positional = ir_to_wire(ir);
    let pos_str = serde_json::to_string(&positional).unwrap_or_default();

    let hier = ir_to_hierarchical_wire(ir);
    let hier_str = serde_json::to_string(&hier).unwrap_or_default();

    let pos_chars = pos_str.len();
    let hier_chars = hier_str.len();

    let savings = if pos_chars > 0 {
        ((pos_chars - hier_chars) as f64 / pos_chars as f64) * 100.0
    } else {
        0.0
    };

    (pos_chars, hier_chars, savings)
}

/// Helper: find a class index by its alias ID.
fn find_class_by_id(classes: &[ClassNode], id: &str) -> Option<usize> {
    classes.iter().position(|c| c.id == id)
}

#[cfg(test)]
#[path = "../tests/ir/hierarchical.rs"]
mod tests;
