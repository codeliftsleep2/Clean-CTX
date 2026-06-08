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

/// A structured delta between two IR states.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IRDelta {
    /// Target file (path alias)
    pub file: String,
    /// Baseline version this delta applies to
    #[serde(rename = "from")]
    pub from_version: u64,
    /// Version after applying this delta
    #[serde(rename = "to")]
    pub to_version: u64,
    /// Operations grouped by type
    pub ops: DeltaOps,
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

/// A modification operation: match by key, replace with new instruction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModOp {
    /// The instruction to match (opcode + id = primary key)
    #[serde(rename = "k")]
    pub key: Vec<String>,
    /// The full replacement instruction
    #[serde(rename = "r")]
    pub replace: Vec<String>,
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
                        replace: op_to_tuple(cur_insn),
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
            from_version: baseline.version,
            to_version: current.version,
            ops,
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
    }
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
        _ => tuple.join(":"),
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
        _ => tuple.to_vec(),
    }
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
#[path = "../tests/ir/delta.rs"]
mod tests;