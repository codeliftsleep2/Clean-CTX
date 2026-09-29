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
use super::opcodes::{
    ControlSummary, CoreOp, DeclarationModifier, ExecutionContextKind, PatternFact, SideEffectKind,
};
use super::wire::DecodeError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const HIERARCHICAL_SCHEMA_REVISION_2: u64 = 2;
const HIERARCHICAL_SCHEMA_REVISION_3: u64 = 3;
const HIERARCHICAL_SCHEMA_REVISION_4: u64 = 4;
const HIERARCHICAL_SCHEMA_REVISION_5: u64 = 5;
const HIERARCHICAL_SCHEMA_REVISION_6: u64 = 6;
const PREVIOUS_HIERARCHICAL_SCHEMA_VERSION: u64 = 7;
const HIERARCHICAL_SCHEMA_VERSION: u64 = 8;

mod decode;
mod encode;
mod migrate;
mod reduce;
use migrate::upgrade_revision_5_pattern_facts;
pub(crate) use reduce::hierarchy_to_wire_reduced;

// Re-exported so the established public paths (`crate::ir::hierarchical::
// ir_to_hierarchical`, `::hierarchical_to_ir`) are unchanged by the split
// into `hierarchical/encode.rs` and `hierarchical/decode.rs`.
pub use super::identity::{
    IdentityError as HierarchicalProjectionError, IdentityKind as ProjectionIdentityKind,
};
pub use decode::hierarchical_to_ir;
pub use encode::{ir_to_hierarchical, try_ir_to_hierarchical};

/// Top-level hierarchical IR container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchicalIR {
    /// Classes (each contains methods, fields, relationships, etc.)
    #[serde(rename = "c")]
    pub classes: Vec<ClassNode>,

    #[serde(rename = "if", default, skip_serializing_if = "Vec::is_empty")]
    pub interfaces: Vec<InterfaceNode>,

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

/// A semantically distinct interface container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceNode {
    #[serde(rename = "n")]
    pub id: String,
    #[serde(rename = "nm")]
    pub name: String,
    #[serde(rename = "m", default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<MethodNode>,
    #[serde(rename = "f", default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldNode>,
    #[serde(rename = "mo", default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<Vec<DeclarationModifier>>,
    #[serde(rename = "x", default, skip_serializing_if = "Vec::is_empty")]
    pub extends: Vec<String>,
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

    /// Ordered class declaration-modifier occurrences.
    #[serde(rename = "mo", default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<Vec<DeclarationModifier>>,

    /// Residual class-metadata flag occurrences (for example CFG and GP).
    ///
    /// Each inner vector is one `CoreOp::ClassFlags` occurrence. The hierarchy
    /// preserves occurrence order, payload order, and duplicate values.
    #[serde(rename = "fl", default, skip_serializing_if = "Vec::is_empty")]
    pub class_flags: Vec<Vec<String>>,

    /// Extends (parent class alias ID)
    #[serde(rename = "x", default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Implements (interface alias IDs)
    #[serde(rename = "im", default, skip_serializing_if = "Vec::is_empty")]
    pub implements: Vec<String>,

    /// Injection occurrences (dependency aliases).
    ///
    /// Each inner vector is one `CoreOp::Injects` occurrence. Operation
    /// boundaries, payload order, and duplicate values are preserved.
    #[serde(rename = "ij", default, skip_serializing_if = "Vec::is_empty")]
    pub injects: Vec<Vec<String>>,

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

    /// Ordered method declaration-modifier occurrences.
    #[serde(rename = "mo", default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<Vec<DeclarationModifier>>,

    /// Ordered typed control-summary occurrences.
    #[serde(rename = "cs", default, skip_serializing_if = "Vec::is_empty")]
    pub control_summaries: Vec<Vec<ControlSummary>>,

    /// Ordered typed pattern-fact occurrences.
    #[serde(rename = "pf", default, skip_serializing_if = "Vec::is_empty")]
    pub pattern_facts: Vec<Vec<PatternFact>>,

    /// Unknown legacy method-metadata occurrences.
    ///
    /// Each inner vector is one `CoreOp::Flags` occurrence. The hierarchy
    /// preserves occurrence order, payload order, and duplicate values.
    #[serde(rename = "fl", default, skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<Vec<String>>,

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

    /// Side-effect annotations (R-43a), in canonical occurrence order.
    /// effect_type: "pure" | "io" | "mutation" | "async" | "transaction"
    #[serde(rename = "se", default, skip_serializing_if = "Vec::is_empty")]
    pub side_effect: Vec<SideEffectKind>,

    /// Execution context annotations (R-43a), in canonical occurrence order.
    /// context_type: "sync" | "async" | "thread_bound" | "transaction_scope" | "realtime"
    #[serde(rename = "ec", default, skip_serializing_if = "Vec::is_empty")]
    pub execution_context: Vec<ExecutionContextKind>,
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
///   "file": "α1", "v": 1, "encoding": "hierarchical", "hs": 3,
///   "ir": {
///     "c": [{
///       "n": "C1", "nm": "SampleService",
///       "m": [{
///         "n": "M1", "nm": "processComplexData",
///         "p": [["P1", "$s", "payload"]],
///         "r": "$b", "fl": [["IF"]]
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
    hierarchy_to_wire(ir, &hir)
}

/// Wrap an already-checked hierarchy in the current wire envelope.
///
/// Production callers use this after `try_ir_to_hierarchical` so projection
/// failures remain structured MCP errors rather than entering the panic-based
/// convenience path above.
pub(crate) fn hierarchy_to_wire(ir: &CompiledIR, hir: &HierarchicalIR) -> Value {
    json!({
        "file": ir.file_id,
        "v": ir.version,
        "encoding": "hierarchical",
        "hs": HIERARCHICAL_SCHEMA_VERSION,
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

    let schema_version = value
        .get("hs")
        .map(|schema_version| {
            schema_version.as_u64().ok_or_else(|| {
                DecodeError::InvalidInput("hierarchical schema version must be an integer".into())
            })
        })
        .transpose()?;
    match schema_version {
        None
        | Some(HIERARCHICAL_SCHEMA_REVISION_2)
        | Some(HIERARCHICAL_SCHEMA_REVISION_3)
        | Some(HIERARCHICAL_SCHEMA_REVISION_4)
        | Some(HIERARCHICAL_SCHEMA_REVISION_5)
        | Some(HIERARCHICAL_SCHEMA_REVISION_6)
        | Some(PREVIOUS_HIERARCHICAL_SCHEMA_VERSION)
        | Some(HIERARCHICAL_SCHEMA_VERSION) => {}
        Some(unsupported) => {
            return Err(DecodeError::InvalidInput(format!(
                "unsupported hierarchical schema version: {unsupported}"
            )));
        }
    }

    let mut ir_val = value
        .get("ir")
        .cloned()
        .ok_or_else(|| DecodeError::MissingField("ir".into()))?;
    match schema_version {
        None => upgrade_legacy_hierarchy(&mut ir_val)?,
        Some(HIERARCHICAL_SCHEMA_REVISION_2) => {
            upgrade_revision_2_hierarchy(&mut ir_val)?;
            upgrade_revision_4_control_summaries(&mut ir_val)?;
            upgrade_revision_5_pattern_facts(&mut ir_val)?;
        }
        Some(HIERARCHICAL_SCHEMA_REVISION_3) | Some(HIERARCHICAL_SCHEMA_REVISION_4) => {
            upgrade_revision_4_control_summaries(&mut ir_val)?;
            upgrade_revision_5_pattern_facts(&mut ir_val)?;
        }
        Some(HIERARCHICAL_SCHEMA_REVISION_5) => upgrade_revision_5_pattern_facts(&mut ir_val)?,
        Some(HIERARCHICAL_SCHEMA_REVISION_6)
        | Some(PREVIOUS_HIERARCHICAL_SCHEMA_VERSION)
        | Some(HIERARCHICAL_SCHEMA_VERSION) => {}
        Some(_) => unreachable!("unsupported revisions returned above"),
    }

    // Deserialize via serde
    let hir: HierarchicalIR = serde_json::from_value(ir_val)
        .map_err(|e| DecodeError::InvalidInput(format!("hierarchical decode: {}", e)))?;

    let instructions = hierarchical_to_ir(&hir);

    Ok(CompiledIR {
        file_id,
        instructions,
        version,
    })
}

/// Upgrade the established unversioned hierarchy shape at the wire boundary.
///
/// This adapter is invoked only when the envelope has no `hs` marker. It
/// changes containers, never their string contents.
fn upgrade_legacy_hierarchy(ir: &mut Value) -> Result<(), DecodeError> {
    let Some(classes) = ir.get_mut("c").and_then(Value::as_array_mut) else {
        return Ok(());
    };

    for class in classes {
        upgrade_flat_occurrences(class, "fl", "class flags")?;
        upgrade_flat_occurrences(class, "ij", "injections")?;
        let Some(methods_value) = class.get_mut("m") else {
            continue;
        };
        let methods = methods_value.as_array_mut().ok_or_else(|| {
            DecodeError::InvalidInput("legacy hierarchical method list must be an array".into())
        })?;
        for method in methods {
            if let Some(flags) = method.get_mut("fl") {
                let payload = flags.as_array().ok_or_else(|| {
                    DecodeError::InvalidInput("legacy hierarchical flags must be an array".into())
                })?;
                if !payload.iter().all(Value::is_string) {
                    return Err(DecodeError::InvalidInput(
                        "legacy hierarchical flags must contain strings".into(),
                    ));
                }
                let payload = std::mem::take(flags);
                *flags = Value::Array(vec![payload]);
            }
            upgrade_legacy_scalar(method, "se", "side effect")?;
            upgrade_legacy_scalar(method, "ec", "execution context")?;
        }
    }
    Ok(())
}

/// Upgrade strict revision-2 containers to the occurrence-aware shape used by
/// later revisions. Typed semantic-family fields are absent and default empty.
///
/// Revision 2 already has occurrence-aware method facts, but its class flags
/// and injections are flat and therefore cannot represent repeated operation
/// boundaries. No other shape is accepted through this compatibility path.
fn upgrade_revision_2_hierarchy(ir: &mut Value) -> Result<(), DecodeError> {
    let Some(classes) = ir.get_mut("c").and_then(Value::as_array_mut) else {
        return Ok(());
    };
    for class in classes {
        upgrade_flat_occurrences(class, "fl", "class flags")?;
        upgrade_flat_occurrences(class, "ij", "injections")?;
    }
    Ok(())
}

/// Revisions 2 through 4 carried control summaries in the residual `fl`
/// channel. Move only the closed vocabulary into `cs`; patterns remain `fl`.
fn upgrade_revision_4_control_summaries(ir: &mut Value) -> Result<(), DecodeError> {
    let Some(classes) = ir.get_mut("c").and_then(Value::as_array_mut) else {
        return Ok(());
    };
    for class in classes {
        let Some(methods) = class.get_mut("m").and_then(Value::as_array_mut) else {
            continue;
        };
        for method in methods {
            let Some(flag_occurrences) = method.get_mut("fl").and_then(Value::as_array_mut) else {
                continue;
            };
            let mut summaries = Vec::new();
            for occurrence in flag_occurrences.iter_mut() {
                let Some(values) = occurrence.as_array_mut() else {
                    continue;
                };
                let mut typed = Vec::new();
                values.retain(|value| {
                    let Some(raw) = value.as_str() else {
                        return true;
                    };
                    if ControlSummary::from_serialized(raw).is_some() {
                        typed.push(value.clone());
                        false
                    } else {
                        true
                    }
                });
                if !typed.is_empty() {
                    summaries.push(Value::Array(typed));
                }
            }
            flag_occurrences.retain(|occurrence| {
                occurrence
                    .as_array()
                    .is_none_or(|values| !values.is_empty())
            });
            if !summaries.is_empty() {
                method["cs"] = Value::Array(summaries);
            }
        }
    }
    Ok(())
}

fn upgrade_flat_occurrences(
    owner: &mut Value,
    field: &str,
    description: &str,
) -> Result<(), DecodeError> {
    let Some(value) = owner.get_mut(field) else {
        return Ok(());
    };
    let payload = value.as_array().ok_or_else(|| {
        DecodeError::InvalidInput(format!(
            "hierarchical revision-2 {description} must be an array"
        ))
    })?;
    if !payload.iter().all(Value::is_string) {
        return Err(DecodeError::InvalidInput(format!(
            "hierarchical revision-2 {description} must contain strings"
        )));
    }
    if payload.is_empty() && field == "ij" {
        return Ok(());
    }
    let payload = std::mem::take(value);
    *value = Value::Array(vec![payload]);
    Ok(())
}

fn upgrade_legacy_scalar(
    method: &mut Value,
    field: &str,
    description: &str,
) -> Result<(), DecodeError> {
    let Some(value) = method.get_mut(field) else {
        return Ok(());
    };
    if !value.is_string() {
        return Err(DecodeError::InvalidInput(format!(
            "legacy hierarchical {description} must be a string"
        )));
    }
    let legacy_value = std::mem::take(value);
    *value = Value::Array(vec![legacy_value]);
    Ok(())
}

/// Estimate character savings of hierarchical format vs. positional encoding.
///
/// Returns `(positional_chars, hierarchical_chars, savings_pct)`. The
/// percentage is negative when the hierarchical envelope is larger.
pub fn estimate_savings(ir: &CompiledIR) -> (usize, usize, f64) {
    use super::wire::ir_to_wire;
    let positional = ir_to_wire(ir);
    let pos_str = serde_json::to_string(&positional).unwrap_or_default();

    let hier = ir_to_hierarchical_wire(ir);
    let hier_str = serde_json::to_string(&hier).unwrap_or_default();

    let pos_chars = pos_str.len();
    let hier_chars = hier_str.len();

    let savings = if pos_chars > 0 {
        ((pos_chars as f64 - hier_chars as f64) / pos_chars as f64) * 100.0
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

#[cfg(test)]
#[path = "../tests/ir/hierarchical_identity.rs"]
mod identity_tests;

#[cfg(test)]
#[path = "../tests/ir/hierarchical_field_identity.rs"]
mod field_identity_tests;

// Typed declaration modifiers and residual flags remain separate ordered
// occurrences, including cross-language and wire/pattern regressions.
#[cfg(test)]
#[path = "../tests/ir/hierarchical_flags.rs"]
mod flags_tests;

#[cfg(test)]
#[path = "../tests/ir/hierarchical_method_facts.rs"]
mod method_fact_tests;

#[cfg(test)]
#[path = "../tests/ir/hierarchical_class_facts.rs"]
mod class_fact_tests;

#[cfg(test)]
#[path = "../tests/ir/hierarchical_patterns.rs"]
mod pattern_tests;

#[cfg(test)]
#[path = "../tests/ir/interface_identity.rs"]
mod interface_identity_tests;
