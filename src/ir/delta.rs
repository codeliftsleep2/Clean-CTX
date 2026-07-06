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

use std::collections::BTreeMap;
use super::compiler::CompiledIR;
use super::opcodes::CoreOp;
use super::wire::op_to_tuple;

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
    pub fn compute(
        &self,
        baseline: &CompiledIR,
        current: &CompiledIR,
    ) -> Option<IRDelta> {
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

        Some(IRDelta {
            file: current.file_id.clone(),
            from: baseline.version,
            to: current.version,
            ops,
            intent: None,
        })
    }
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
        CoreOp::Flags(tid, _) => format!("FLAGS:{}", tid),
        CoreOp::ClassFlags(cid, _) => format!("FLAGS_C:{}", cid),
        CoreOp::Extends(child, _) => format!("EXT:{}", child),
        CoreOp::Implements(cid, iid) => format!("IMPL:{}:{}", cid, iid),
        CoreOp::Injects(cid, _) => format!("INJECTS:{}", cid),
        CoreOp::Import(alias, _, _) => format!("IMP:{}", alias),
        CoreOp::TypeAlias(alias, _) => format!("TYPE:{}", alias),
        CoreOp::Pattern(name, args) => {
            format!("PAT:{}:{}", name, args.first().map(|s| s.as_str()).unwrap_or("?"))
        }
        // R-43a: Execution Semantics
        CoreOp::DataFlow(mid, _, _) => format!("DATAFLOW:{}", mid),
        CoreOp::ControlFlow(mid, _, _) => format!("CTRL:{}", mid),
        CoreOp::SideEffect(mid, _) => format!("EFFECT:{}", mid),
        CoreOp::ExecutionContext(mid, _) => format!("CTX:{}", mid),
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
        // R-43a: Execution Semantics
        CoreOp::DataFlow(mid, _, _) => vec!["DATAFLOW".into(), mid.clone()],
        CoreOp::ControlFlow(mid, _, _) => vec!["CTRL".into(), mid.clone()],
        CoreOp::SideEffect(mid, _) => vec!["EFFECT".into(), mid.clone()],
        CoreOp::ExecutionContext(mid, _) => vec!["CTX".into(), mid.clone()],
    }
}

/// Compute field patches between two CoreOp tuples.
///
/// Returns None if the tuples have different opcodes or the same full content.
/// Returns Some(patches) with an empty vec if the tuples are identical.
/// Only non-identical fields are included, skipping the opcode (index 0).
pub fn compute_field_patches(base_tuple: &[String], cur_tuple: &[String]) -> Option<Vec<FieldPatch>> {
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

// ── Compact Delta Encoding (Idea #6) ────────────────────────────

/// A compact delta that uses abbreviated field names and opcode abbreviations.
///
/// This is a wrapper around IRDelta that provides alternative serialization.
/// The compact format uses:
/// - `f` instead of `file`
/// - `"5→6"` version range instead of separate `from`/`to` fields
/// - Abbreviated opcodes in instruction tuples (single-char where unambiguous)
/// - Field-patch format for all modifications (Idea #3)
///
/// The compact format is **always lossless** and decodes to the same IRDelta.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompactDelta {
    /// File alias
    #[serde(rename = "f")]
    pub file: String,
    /// Version as "from→to" string
    #[serde(rename = "v")]
    pub version_range: String,
    /// Operations
    #[serde(rename = "o")]
    pub ops: CompactOps,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CompactOps {
    /// Additions — tuples with abbreviated opcodes
    #[serde(rename = "+", skip_serializing_if = "Vec::is_empty")]
    pub adds: Vec<Vec<String>>,
    /// Modifications — encoded as [key_tuple, field_index, new_value, ...]
    /// e.g. ["C1:M1", 3, "renamedMethod"]
    #[serde(rename = "~", skip_serializing_if = "Vec::is_empty")]
    pub mods: Vec<serde_json::Value>,
    /// Deletions — key tuples with abbreviated opcodes
    #[serde(rename = "-", skip_serializing_if = "Vec::is_empty")]
    pub dels: Vec<Vec<String>>,
}

/// Abbreviate an opcode string to a compact single-char or short form.
fn abbreviate_opcode(opcode: &str) -> &str {
    match opcode {
        "DEF_C" => "C",
        "DEF_M" => "M",
        "DEF_F" => "F",
        "DEF_I" => "I",
        "SIG" => "S",
        "RET" => "R",
        "FIELD_T" => "FT",
        "FLAGS" => "FL",
        "FLAGS_C" => "FC",
        "EXT" => "E",
        "IMPL" => "IM",
        "INJECTS" => "IJ",
        "IMP" => "IP",
        "TYPE" => "T",
        "PAT" => "P",
        // R-43a: compact abbreviations
        "DATAFLOW" => "DF",
        "CTRL" => "CT",
        "EFFECT" => "EF",
        "CTX" => "CX",
        _ => opcode,
    }
}

/// Expand an abbreviated opcode back to its full form.
fn expand_opcode(abbrev: &str) -> &str {
    match abbrev {
        "C" => "DEF_C",
        "M" => "DEF_M",
        "F" => "DEF_F",
        "I" => "DEF_I",
        "S" => "SIG",
        "R" => "RET",
        "FT" => "FIELD_T",
        "FL" => "FLAGS",
        "FC" => "FLAGS_C",
        "E" => "EXT",
        "IM" => "IMPL",
        "IJ" => "INJECTS",
        "IP" => "IMP",
        "T" => "TYPE",
        "P" => "PAT",
        // R-43a: compact abbreviations
        "DF" => "DATAFLOW",
        "CT" => "CTRL",
        "EF" => "EFFECT",
        "CX" => "CTX",
        _ => abbrev,
    }
}

/// Encode an IRDelta into the compact format.
pub fn compact_encode(delta: &IRDelta) -> CompactDelta {
    let version_range = format!("{}→{}", delta.from, delta.to);

    let mut ops = CompactOps::default();

    // Encode additions with abbreviated opcodes
    for add in &delta.ops.adds {
        let mut compact = add.clone();
        if !compact.is_empty() {
            compact[0] = abbreviate_opcode(&compact[0]).to_string();
        }
        ops.adds.push(compact);
    }

    // Encode modifications as field patches
    for mod_op in &delta.ops.mods {
        if let Some(replacement) = &mod_op.replace {
            // Compute field patches and encode compactly
            let base_tuple = &mod_op.key;
            let patches = compute_field_patches(base_tuple, replacement);
            if let Some(patch_list) = patches {
                // Format as [key_joined, field1, val1, field2, val2, ...]
                let key_str = base_tuple.join(":");
                let mut compact_mod = vec![serde_json::Value::String(key_str)];
                for patch in &patch_list {
                    compact_mod.push(serde_json::Value::Number(patch.field_index.into()));
                    compact_mod.push(serde_json::Value::String(patch.new_value.clone()));
                }
                ops.mods.push(serde_json::Value::Array(compact_mod));
            }
        } else if let Some(patches) = &mod_op.patches {
            let key_str = mod_op.key.join(":");
            let mut compact_mod = vec![serde_json::Value::String(key_str)];
            for patch in patches {
                compact_mod.push(serde_json::Value::Number(patch.field_index.into()));
                compact_mod.push(serde_json::Value::String(patch.new_value.clone()));
            }
            ops.mods.push(serde_json::Value::Array(compact_mod));
        }
    }

    // Encode deletions with abbreviated opcodes
    for del in &delta.ops.dels {
        let mut compact = del.clone();
        if !compact.is_empty() {
            compact[0] = abbreviate_opcode(&compact[0]).to_string();
        }
        ops.dels.push(compact);
    }

    CompactDelta {
        file: delta.file.clone(),
        version_range,
        ops,
    }
}

/// Decode a CompactDelta back into an IRDelta.
pub fn compact_decode(compact: &CompactDelta) -> Option<IRDelta> {
    // Parse version range "from→to"
    let parts: Vec<&str> = compact.version_range.split('→').collect();
    if parts.len() != 2 {
        return None;
    }
    let from: u64 = parts[0].parse().ok()?;
    let to: u64 = parts[1].parse().ok()?;

    let mut adds = Vec::new();
    let mut mods = Vec::new();
    let mut dels = Vec::new();

    // Decode additions — expand abbreviated opcodes
    for add in &compact.ops.adds {
        let mut full = add.clone();
        if !full.is_empty() {
            full[0] = expand_opcode(&full[0]).to_string();
        }
        adds.push(full);
    }

    // Decode modifications — reconstruct ModOp from patches
    for mod_val in &compact.ops.mods {
        let arr = mod_val.as_array()?;
        if arr.is_empty() {
            return None;
        }
        let key_str = arr[0].as_str()?;
        let key_parts: Vec<&str> = key_str.split(':').collect();
        let key: Vec<String> = key_parts.iter().map(|s| s.to_string()).collect();

        // Reconstruct patches from alternating [field, value] pairs
        let mut patches = Vec::new();
        let mut i = 1;
        while i + 1 < arr.len() {
            let field_idx = arr[i].as_u64()? as usize;
            let new_val = arr[i + 1].as_str()?.to_string();
            patches.push(FieldPatch {
                field_index: field_idx,
                new_value: new_val,
            });
            i += 2;
        }

        mods.push(ModOp::new_patches(key, patches));
    }

    // Decode deletions — expand abbreviated opcodes
    for del in &compact.ops.dels {
        let mut full = del.clone();
        if !full.is_empty() {
            full[0] = expand_opcode(&full[0]).to_string();
        }
        dels.push(full);
    }

    Some(IRDelta {
        file: compact.file.clone(),
        from,
        to,
        ops: DeltaOps { adds, mods, dels },
        intent: None,
    })
}

// ── Public helpers for external consumers (replay, etc.) ─────────

/// Extract the primary key from an instruction tuple (Vec<String>).
/// Used by state replay to match instructions by key.
pub fn primary_key_from_tuple(tuple: &[String]) -> String {
    if tuple.is_empty() {
        return String::new();
    }
    match tuple[0].as_str() {
        "DEF_C" => format!("DEF_C:{}", tuple.get(1).unwrap_or(&String::new())),
        "DEF_M" => format!(
            "DEF_M:{}:{}",
            tuple.get(1).unwrap_or(&String::new()),
            tuple.get(2).unwrap_or(&String::new())
        ),
        "DEF_F" => format!(
            "DEF_F:{}:{}",
            tuple.get(1).unwrap_or(&String::new()),
            tuple.get(2).unwrap_or(&String::new())
        ),
        "DEF_I" => format!("DEF_I:{}", tuple.get(1).unwrap_or(&String::new())),
        "SIG" => format!(
            "SIG:{}:{}",
            tuple.get(1).unwrap_or(&String::new()),
            tuple.get(2).unwrap_or(&String::new())
        ),
        "RET" => format!("RET:{}", tuple.get(1).unwrap_or(&String::new())),
        "FIELD_T" => format!("FIELD_T:{}", tuple.get(1).unwrap_or(&String::new())),
        "FLAGS" => format!("FLAGS:{}", tuple.get(1).unwrap_or(&String::new())),
        "FLAGS_C" => format!("FLAGS_C:{}", tuple.get(1).unwrap_or(&String::new())),
        "EXT" => format!("EXT:{}", tuple.get(1).unwrap_or(&String::new())),
        "IMPL" => format!(
            "IMPL:{}:{}",
            tuple.get(1).unwrap_or(&String::new()),
            tuple.get(2).unwrap_or(&String::new())
        ),
        "INJECTS" => format!("INJECTS:{}", tuple.get(1).unwrap_or(&String::new())),
        "IMP" => format!("IMP:{}", tuple.get(1).unwrap_or(&String::new())),
        "TYPE" => format!("TYPE:{}", tuple.get(1).unwrap_or(&String::new())),
        // R-43a: Execution Semantics
        "DATAFLOW" => format!("DATAFLOW:{}", tuple.get(1).unwrap_or(&String::new())),
        "CTRL" => format!("CTRL:{}", tuple.get(1).unwrap_or(&String::new())),
        "EFFECT" => format!("EFFECT:{}", tuple.get(1).unwrap_or(&String::new())),
        "CTX" => format!("CTX:{}", tuple.get(1).unwrap_or(&String::new())),
        _ => {
            // F-16: Unknown opcode — fallback produces a key from the full tuple.
            if cfg!(debug_assertions) {
                eprintln!("[warn] primary_key_from_tuple: unknown opcode '{}'", tuple[0]);
            }
            tuple.join(":")
        }
    }
}

/// Extract the key tuple from an instruction tuple (Vec<String>).
/// Returns the opcode + identifying operands.
pub fn key_tuple_from_tuple(tuple: &[String]) -> Vec<String> {
    if tuple.is_empty() {
        return Vec::new();
    }
    match tuple[0].as_str() {
        "DEF_C" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "DEF_M" => vec![
            tuple[0].clone(),
            tuple.get(1).cloned().unwrap_or_default(),
            tuple.get(2).cloned().unwrap_or_default(),
        ],
        "DEF_F" => vec![
            tuple[0].clone(),
            tuple.get(1).cloned().unwrap_or_default(),
            tuple.get(2).cloned().unwrap_or_default(),
        ],
        "DEF_I" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "SIG" => vec![
            tuple[0].clone(),
            tuple.get(1).cloned().unwrap_or_default(),
            tuple.get(2).cloned().unwrap_or_default(),
        ],
        "RET" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "FIELD_T" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "FLAGS" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "FLAGS_C" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "EXT" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "IMPL" => vec![
            tuple[0].clone(),
            tuple.get(1).cloned().unwrap_or_default(),
            tuple.get(2).cloned().unwrap_or_default(),
        ],
        "INJECTS" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "IMP" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "TYPE" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        // R-43a: Execution Semantics
        "DATAFLOW" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "CTRL" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "EFFECT" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "CTX" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        _ => {
            // F-17: Unknown opcode — fallback returns the full instruction body.
            if cfg!(debug_assertions) {
                eprintln!("[warn] key_tuple_from_tuple: unknown opcode '{}'", tuple[0]);
            }
            tuple.to_vec()
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
#[path = "../tests/ir/delta.rs"]
mod tests;