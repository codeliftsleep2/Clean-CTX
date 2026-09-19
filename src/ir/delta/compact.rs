// src/ir/delta/compact.rs
//
// Compact delta encoding (Idea #6): the abbreviated-opcode JSON form of an
// IRDelta and its inverse.
//
// Split out of `src/ir/delta.rs` (active-file size policy): the module had
// exceeded the 615-line ceiling. This is a pure relocation -- the code below
// is byte-for-byte the previous implementation, and `delta` re-exports the
// public entry points (`compact_encode`, `compact_decode`, `CompactDelta`,
// `CompactOps`) so every existing path keeps resolving.
//
// Child-module access: a private item of an ancestor module is visible in its
// descendants, so `use super::*` supplies `IRDelta`, `DeltaOps`, `ModOp`,
// `FieldPatch`, `compute_field_patches`, and `SemanticIntent`.

use super::*;

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
    /// R-43a: optional semantic intent metadata, preserved through
    /// the compact encode → decode round-trip.
    #[serde(rename = "i", skip_serializing_if = "Option::is_none")]
    pub intent: Option<SemanticIntent>,
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
        "MOD_M" => "MM",
        "MOD_C" => "MC",
        "EXT" => "E",
        "IMPL" => "IM",
        "INJECTS" => "IJ",
        "IMP" => "IP",
        "TYPE" => "T",
        "PAT" => "P",
        // Edit Mode: Verbatim Method Bodies
        "BODY" => "BD",
        // R-43a: compact abbreviations
        "DATAFLOW" => "DF",
        "CTRL" => "CT",
        "EFFECT" => "EF",
        "CTX" => "CX",
        // Structural invocations (native call graph)
        "CALL" => "CL",
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
        "MM" => "MOD_M",
        "MC" => "MOD_C",
        "E" => "EXT",
        "IM" => "IMPL",
        "IJ" => "INJECTS",
        "IP" => "IMP",
        "T" => "TYPE",
        "P" => "PAT",
        // Edit Mode: Verbatim Method Bodies
        "BD" => "BODY",
        // R-43a: compact abbreviations
        "DF" => "DATAFLOW",
        "CT" => "CTRL",
        "EF" => "EFFECT",
        "CX" => "CTX",
        // Structural invocations (native call graph)
        "CL" => "CALL",
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
        intent: delta.intent.clone(),
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
        intent: compact.intent.clone(),
    })
}
