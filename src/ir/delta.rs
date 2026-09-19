// src/ir/delta.rs
//
// Phase C: Delta Transport — instruction-level diffing between two CompiledIR
// states, producing a structured delta envelope for transport.
//
// Instead of computing text diffs between CapturedStructure snapshots, the
// delta engine computes instruction-level deltas between CompiledIR states.
//
// Delta Wire Format:
// ```json
// {
//   "file": "<path_alias>",
//   "from": <baseline_version>,
//   "to": <current_version>,
//   "ops": {
//     "+": [ [<instruction>, ...], ... ],
//     "~": [ {"k": [<key_tuple>], "r": [<replacement>]}, ... ],
//     "-": [ [<instruction>, ...], ... ]
//   }
// }
// ```

use super::compiler::CompiledIR;
use super::opcodes::CoreOp;
use super::wire::op_to_tuple;
use std::collections::BTreeMap;

mod compact;
mod tuples;

// Glob re-exports keep the established public paths unchanged by the split
// (`crate::ir::delta::compact_encode`, `::CompactDelta`, `::CompactOps`,
// `::primary_key_from_tuple`, `::key_tuple_from_tuple`). Only `pub` items are
// re-exported, so the private abbreviation helpers stay module-local.
pub use compact::*;
pub use tuples::*;

/// R-43a: High-level semantic intent of a delta operation.
/// Provides human-readable context for what changed, beyond the structural diff.
/// Empty (None) by default — wire format ready for Phase 4 enrichment.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticIntent {
    RenameSymbol {
        old_name: String,
        new_name: String,
        kind: String, // "class", "method", "field"
    },
    AddMethod {
        class: String,
        method_name: String,
    },
    RemoveMethod {
        class: String,
        method_name: String,
    },
    ChangeSignature {
        method: String,
        field_changed: String, // "return_type", "param_type", "param_name"
    },
    AddInjection {
        class: String,
        dependency: String,
    },
    ChangeReturnType {
        method: String,
        old_type: String,
        new_type: String,
    },
}

/// A structured delta between two IR states.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IRDelta {
    /// Target file (path alias)
    pub file: String,
    /// Baseline version this delta applies to
    pub from: u64,
    /// Version after applying this delta
    pub to: u64,
    /// Operations grouped by type
    pub ops: DeltaOps,
    /// R-43a: optional semantic intent metadata
    /// Empty (None) by default — wire format ready for Phase 4 enrichment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<SemanticIntent>,
}

/// Grouped delta operations.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DeltaOps {
    /// New instructions to insert
    #[serde(rename = "+")]
    pub adds: Vec<Vec<String>>,
    /// In-place modifications
    #[serde(rename = "~")]
    pub mods: Vec<ModOp>,
    /// Instructions to remove (matched by opcode + primary key)
    #[serde(rename = "-")]
    pub dels: Vec<Vec<String>>,
}

/// A single field patch: change the value at `field_index` to `new_value`.
///
/// Used in the compact delta format (Idea #3).
/// Field index is 0-based position in the instruction tuple.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldPatch {
    /// 0-based index of the field to change
    #[serde(rename = "i")]
    pub field_index: usize,
    /// New value for the field
    #[serde(rename = "v")]
    pub new_value: String,
}

/// A modification operation: match by key, replace with new instruction.
///
/// Two formats supported:
/// 1. Full replacement (`"r"` field) — the original format
/// 2. Field patches (`"d"` field) — compact format (Idea #3)
///
/// When both `r` and `d` are present, `r` takes precedence (full replacement).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModOp {
    /// The instruction to match (opcode + id = primary key)
    #[serde(rename = "k")]
    pub key: Vec<String>,
    /// Full replacement instruction (original format, optional)
    #[serde(rename = "r", skip_serializing_if = "Option::is_none")]
    pub replace: Option<Vec<String>>,
    /// Field patches (compact format, optional — Idea #3)
    #[serde(rename = "d", skip_serializing_if = "Option::is_none")]
    pub patches: Option<Vec<FieldPatch>>,
}

impl ModOp {
    /// Create a new ModOp with full replacement format.
    pub fn new_replace(key: Vec<String>, replace: Vec<String>) -> Self {
        Self {
            key,
            replace: Some(replace),
            patches: None,
        }
    }

    /// Create a new ModOp with field-patch format.
    pub fn new_patches(key: Vec<String>, patches: Vec<FieldPatch>) -> Self {
        Self {
            key,
            replace: None,
            patches: Some(patches),
        }
    }
}

/// Delta computation engine.
/// Compares two CompiledIR states and produces an IRDelta.
pub struct DeltaComputer;

impl DeltaComputer {
    pub fn new() -> Self {
        Self
    }

    /// Compute the delta between baseline and current IR.
    /// Returns None if both IRs are identical.
    pub fn compute(&self, baseline: &CompiledIR, current: &CompiledIR) -> Option<IRDelta> {
        let base_indexed = index_instructions(&baseline.instructions);
        let cur_indexed = index_instructions(&current.instructions);

        let mut ops = DeltaOps::default();

        // Additions: in current but not baseline
        for (key, insn) in &cur_indexed {
            if !base_indexed.contains_key(key) {
                ops.adds.push(op_to_tuple(insn));
            }
        }

        // Removals: in baseline but not current
        for (key, insn) in &base_indexed {
            if !cur_indexed.contains_key(key) {
                ops.dels.push(op_to_tuple(insn));
            }
        }

        // Modifications: in both but different
        for (key, base_insn) in &base_indexed {
            if let Some(cur_insn) = cur_indexed.get(key) {
                if op_to_tuple(base_insn) != op_to_tuple(cur_insn) {
                    ops.mods.push(ModOp {
                        key: key_tuple(base_insn),
                        replace: Some(op_to_tuple(cur_insn)),
                        patches: None,
                    });
                }
            }
        }

        // Return None if no changes
        if ops.adds.is_empty() && ops.mods.is_empty() && ops.dels.is_empty() {
            return None;
        }

        let intent = detect_semantic_intent(baseline, current);

        Some(IRDelta {
            file: current.file_id.clone(),
            from: baseline.version,
            to: current.version,
            ops,
            intent,
        })
    }
}

/// Detect the high-level semantic intent of a delta between two IR states.
///
/// This examines the structural diff (adds/mods/dels) and produces a
/// human-readable `SemanticIntent` describing *what* changed, beyond the
/// raw instruction-level diff. Returns `None` when no single intent can
/// be confidently identified (e.g. a mixed change with no dominant signal).
///
/// R-43a: Phase 1 detection is purely structural and deterministic — it
/// inspects `CoreOp` streams directly. Phase 4 (R-43b) may enrich this
/// with CBM cross-file impact metadata.
///
/// Detection precedence (first match wins):
///   1. Rename — a DEF_* op with the same primary key but a different name
///   2. AddInjection — an INJECTS op present in `current` but not `baseline`
///   3. AddMethod — a DEF_M op present in `current` but not `baseline`
///   4. RemoveMethod — a DEF_M op present in `baseline` but not `current`
///   5. ChangeReturnType — a RET op whose type changed
///   6. ChangeSignature — a SIG or RET op modified
///
/// Because the delta grouping is computed from BTreeMap-indexed instruction
/// streams, iteration order is deterministic (invariant S4).
fn detect_semantic_intent(baseline: &CompiledIR, current: &CompiledIR) -> Option<SemanticIntent> {
    let base_indexed = index_instructions(&baseline.instructions);
    let cur_indexed = index_instructions(&current.instructions);

    // 1. Rename detection: same primary key, different name operand.
    //    DEF_C / DEF_M / DEF_F are the named structural ops.
    for (key, base_insn) in &base_indexed {
        if let Some(cur_insn) = cur_indexed.get(key) {
            let intent = match (base_insn, cur_insn) {
                (CoreOp::DefClass(_, old_name), CoreOp::DefClass(_, new_name))
                    if old_name != new_name =>
                {
                    Some(SemanticIntent::RenameSymbol {
                        old_name: old_name.clone(),
                        new_name: new_name.clone(),
                        kind: "class".to_string(),
                    })
                }
                (CoreOp::DefMethod(_, _, old_name), CoreOp::DefMethod(_, _, new_name))
                    if old_name != new_name =>
                {
                    Some(SemanticIntent::RenameSymbol {
                        old_name: old_name.clone(),
                        new_name: new_name.clone(),
                        kind: "method".to_string(),
                    })
                }
                (CoreOp::DefField(_, _, old_name), CoreOp::DefField(_, _, new_name))
                    if old_name != new_name =>
                {
                    Some(SemanticIntent::RenameSymbol {
                        old_name: old_name.clone(),
                        new_name: new_name.clone(),
                        kind: "field".to_string(),
                    })
                }
                _ => None,
            };
            if intent.is_some() {
                return intent;
            }
        }
    }

    // 2. AddInjection: an INJECTS op gained a dependency. This covers both
    //    the first-ever injection (key not in baseline) and a dependency
    //    added to an existing INJECTS op (key in both, deps list grew).
    for (key, cur_insn) in &cur_indexed {
        if let CoreOp::Injects(class, cur_deps) = cur_insn {
            let newly_added = match base_indexed.get(key) {
                Some(CoreOp::Injects(_, base_deps)) => {
                    // Existing INJECTS op — find a dep not in the baseline list.
                    cur_deps.iter().find(|d| !base_deps.contains(d))
                }
                _ => {
                    // First-ever injection — report the first dep.
                    cur_deps.first()
                }
            };
            if let Some(dep) = newly_added {
                return Some(SemanticIntent::AddInjection {
                    class: class.clone(),
                    dependency: dep.clone(),
                });
            }
        }
    }

    // 3. AddMethod: DEF_M present in current but not baseline.
    for (key, cur_insn) in &cur_indexed {
        if !base_indexed.contains_key(key) {
            if let CoreOp::DefMethod(class, _, method_name) = cur_insn {
                return Some(SemanticIntent::AddMethod {
                    class: class.clone(),
                    method_name: method_name.clone(),
                });
            }
        }
    }

    // 4. RemoveMethod: DEF_M present in baseline but not current.
    for (key, base_insn) in &base_indexed {
        if !cur_indexed.contains_key(key) {
            if let CoreOp::DefMethod(class, _, method_name) = base_insn {
                return Some(SemanticIntent::RemoveMethod {
                    class: class.clone(),
                    method_name: method_name.clone(),
                });
            }
        }
    }

    // 5. ChangeReturnType: RET op whose type changed.
    for (key, base_insn) in &base_indexed {
        if let Some(cur_insn) = cur_indexed.get(key) {
            if let (CoreOp::Return(method, old_type), CoreOp::Return(_, new_type)) =
                (base_insn, cur_insn)
            {
                if old_type != new_type {
                    return Some(SemanticIntent::ChangeReturnType {
                        method: method.clone(),
                        old_type: old_type.clone(),
                        new_type: new_type.clone(),
                    });
                }
            }
        }
    }

    // 6. ChangeSignature: SIG (Param) op modified. Return-type changes are
    //    already reported as ChangeReturnType in step 5, so they are not
    //    re-matched here (the Return arm would be unreachable dead code).
    for (key, base_insn) in &base_indexed {
        if let Some(cur_insn) = cur_indexed.get(key) {
            if let (CoreOp::Param(method, _, _, _), CoreOp::Param(_, _, _, _)) =
                (base_insn, cur_insn)
            {
                // Determine which field changed.
                let field_changed = match (base_insn, cur_insn) {
                    (CoreOp::Param(_, _, base_ty, _), CoreOp::Param(_, _, cur_ty, _))
                        if base_ty != cur_ty =>
                    {
                        "param_type"
                    }
                    (CoreOp::Param(_, _, _, base_name), CoreOp::Param(_, _, _, cur_name))
                        if base_name != cur_name =>
                    {
                        "param_name"
                    }
                    _ => continue,
                };
                return Some(SemanticIntent::ChangeSignature {
                    method: method.clone(),
                    field_changed: field_changed.to_string(),
                });
            }
        }
    }

    None
}

impl Default for DeltaComputer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Indexing Helpers ────────────────────────────────────────────

/// Index instructions by their primary key (opcode + identifying operands).
/// Uses BTreeMap for deterministic iteration order.
fn index_instructions(instructions: &[CoreOp]) -> BTreeMap<String, CoreOp> {
    instructions
        .iter()
        .map(|op| {
            let key = primary_key(op);
            (key, op.clone())
        })
        .collect()
}

/// Extract the primary key from an instruction.
/// Used for matching in deltas. The key uniquely identifies an instruction
/// by its opcode and the identifying IDs (class_id, method_id, etc.).
fn primary_key(op: &CoreOp) -> String {
    match op {
        CoreOp::DefClass(id, _) => format!("DEF_C:{}", id),
        CoreOp::DefMethod(cid, mid, _) => format!("DEF_M:{}:{}", cid, mid),
        CoreOp::DefField(cid, fid, _) => format!("DEF_F:{}:{}", cid, fid),
        CoreOp::DefInterface(id, _) => format!("DEF_I:{}", id),
        CoreOp::Param(mid, pid, _, _) => format!("SIG:{}:{}", mid, pid),
        CoreOp::Return(mid, _) => format!("RET:{}", mid),
        CoreOp::FieldType(fid, _) => format!("FIELD_T:{}", fid),
        CoreOp::MethodModifiers(mid, _) => format!("MOD_M:{}", mid),
        CoreOp::ClassModifiers(cid, _) => format!("MOD_C:{}", cid),
        CoreOp::Flags(tid, _) => format!("FLAGS:{}", tid),
        CoreOp::ClassFlags(cid, _) => format!("FLAGS_C:{}", cid),
        CoreOp::Extends(child, _) => format!("EXT:{}", child),
        CoreOp::Implements(cid, iid) => format!("IMPL:{}:{}", cid, iid),
        CoreOp::Injects(cid, _) => format!("INJECTS:{}", cid),
        CoreOp::Import(alias, _, _) => format!("IMP:{}", alias),
        CoreOp::TypeAlias(alias, _) => format!("TYPE:{}", alias),
        CoreOp::Pattern(name, args) => {
            format!(
                "PAT:{}:{}",
                name,
                args.first().map(|s| s.as_str()).unwrap_or("?")
            )
        }
        // Edit Mode: Verbatim Method Bodies
        CoreOp::Body(mid, ..) => format!("BODY:{}", mid),
        // R-43a: Execution Semantics
        CoreOp::DataFlow(mid, _, _) => format!("DATAFLOW:{}", mid),
        CoreOp::ControlFlow(mid, _, _) => format!("CTRL:{}", mid),
        CoreOp::SideEffect(mid, _) => format!("EFFECT:{}", mid),
        CoreOp::ExecutionContext(mid, _) => format!("CTX:{}", mid),
        // Structural invocations (native call graph): identity carries the
        // caller, the callee name, the explicit argument count AND the spread
        // qualifier, so two invocations that differ only in arity — or only in
        // whether a written argument expands — are distinct instructions.
        // Without the qualifier, `A.foo(a)` and `A.foo(...args)` (both written
        // as argc 1) would collapse into one delta entry, reproducing the
        // evidence loss the workspace index already refuses to make.
        CoreOp::Call(caller, callee, argc, has_spread) => {
            if *has_spread {
                format!("CALL:{}:{}:{}:spread", caller, callee, argc)
            } else {
                format!("CALL:{}:{}:{}", caller, callee, argc)
            }
        }
    }
}

/// Extract the key tuple from an instruction (for ModOp matching).
/// Returns the opcode + identifying operands as a Vec<String>.
fn key_tuple(op: &CoreOp) -> Vec<String> {
    match op {
        CoreOp::DefClass(id, _) => vec!["DEF_C".into(), id.clone()],
        CoreOp::DefMethod(cid, mid, _) => vec!["DEF_M".into(), cid.clone(), mid.clone()],
        CoreOp::DefField(cid, fid, _) => vec!["DEF_F".into(), cid.clone(), fid.clone()],
        CoreOp::DefInterface(id, _) => vec!["DEF_I".into(), id.clone()],
        CoreOp::Param(mid, pid, _, _) => vec!["SIG".into(), mid.clone(), pid.clone()],
        CoreOp::Return(mid, _) => vec!["RET".into(), mid.clone()],
        CoreOp::FieldType(fid, _) => vec!["FIELD_T".into(), fid.clone()],
        CoreOp::MethodModifiers(mid, _) => vec!["MOD_M".into(), mid.clone()],
        CoreOp::ClassModifiers(cid, _) => vec!["MOD_C".into(), cid.clone()],
        CoreOp::Flags(tid, _) => vec!["FLAGS".into(), tid.clone()],
        CoreOp::ClassFlags(cid, _) => vec!["FLAGS_C".into(), cid.clone()],
        CoreOp::Extends(child, _) => vec!["EXT".into(), child.clone()],
        CoreOp::Implements(cid, iid) => vec!["IMPL".into(), cid.clone(), iid.clone()],
        CoreOp::Injects(cid, _) => vec!["INJECTS".into(), cid.clone()],
        CoreOp::Import(alias, _, _) => vec!["IMP".into(), alias.clone()],
        CoreOp::TypeAlias(alias, _) => vec!["TYPE".into(), alias.clone()],
        CoreOp::Pattern(name, args) => {
            let mut v = vec!["PAT".into(), name.clone()];
            if let Some(first_arg) = args.first() {
                v.push(first_arg.clone());
            }
            v
        }
        // Edit Mode: Verbatim Method Bodies
        CoreOp::Body(mid, ..) => vec!["BODY".into(), mid.clone()],
        // R-43a: Execution Semantics
        CoreOp::DataFlow(mid, _, _) => vec!["DATAFLOW".into(), mid.clone()],
        CoreOp::ControlFlow(mid, _, _) => vec!["CTRL".into(), mid.clone()],
        CoreOp::SideEffect(mid, _) => vec!["EFFECT".into(), mid.clone()],
        CoreOp::ExecutionContext(mid, _) => vec!["CTX".into(), mid.clone()],
        // Structural invocations: every identifying operand is part of the key
        // (the argument count keeps arity-distinct calls apart, and the spread
        // qualifier keeps an exact call apart from a call whose count is a
        // written-only count).
        CoreOp::Call(caller, callee, argc, has_spread) => {
            let mut tuple = vec![
                "CALL".into(),
                caller.clone(),
                callee.clone(),
                argc.to_string(),
            ];
            if *has_spread {
                tuple.push("spread".into());
            }
            tuple
        }
    }
}

/// Compute field patches between two CoreOp tuples.
///
/// Returns None if the tuples have different opcodes or the same full content.
/// Returns Some(patches) with an empty vec if the tuples are identical.
/// Only non-identical fields are included, skipping the opcode (index 0).
pub fn compute_field_patches(
    base_tuple: &[String],
    cur_tuple: &[String],
) -> Option<Vec<FieldPatch>> {
    if base_tuple.is_empty() || cur_tuple.is_empty() {
        return None;
    }
    // Must be same opcode
    if base_tuple[0] != cur_tuple[0] {
        return None;
    }
    let mut patches = Vec::new();
    let max_len = base_tuple.len().max(cur_tuple.len());
    for i in 1..max_len {
        let base_val = base_tuple.get(i).map(|s| s.as_str()).unwrap_or("");
        let cur_val = cur_tuple.get(i).map(|s| s.as_str()).unwrap_or("");
        if base_val != cur_val {
            patches.push(FieldPatch {
                field_index: i,
                new_value: cur_val.to_string(),
            });
        }
    }
    if patches.is_empty() {
        None // no difference
    } else {
        Some(patches)
    }
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
#[path = "../tests/ir/delta.rs"]
mod tests;
