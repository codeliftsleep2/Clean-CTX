// src/ir/wire.rs
//
// IR → Wire format serialization.
//
// Phase A: IR Core — serializes CoreOp instructions and CompiledIR
// to positional JSON tuple format for transport.
//
// Every instruction is a JSON array where position determines meaning:
//   [op0, op1, op2, ...]
//    ^     ^     ^
//    |     |     └── operand (type depends on opcode)
//    |     └── target/class/method id
//    └── opcode (always first element)

use super::compiler::CompiledIR;
use super::opcodes::CoreOp;
use serde_json::{json, Value};

/// Serialize a single CoreOp to its positional tuple representation.
pub fn op_to_tuple(op: &CoreOp) -> Vec<String> {
    match op {
        CoreOp::DefClass(id, name) => vec!["DEF_C".into(), id.clone(), name.clone()],
        CoreOp::DefMethod(cid, mid, name) => {
            vec![
                "DEF_M".into(),
                cid.clone(),
                mid.clone(),
                name.clone(),
            ]
        }
        CoreOp::DefField(cid, fid, name) => {
            vec![
                "DEF_F".into(),
                cid.clone(),
                fid.clone(),
                name.clone(),
            ]
        }
        CoreOp::DefInterface(id, name) => vec!["DEF_I".into(), id.clone(), name.clone()],
        CoreOp::Param(mid, pid, ty, name) => {
            vec![
                "SIG".into(),
                mid.clone(),
                pid.clone(),
                ty.clone(),
                name.clone(),
            ]
        }
        CoreOp::Return(mid, ty) => vec!["RET".into(), mid.clone(), ty.clone()],
        CoreOp::FieldType(fid, ty) => vec!["FIELD_T".into(), fid.clone(), ty.clone()],
        CoreOp::Flags(tid, flags) => {
            let mut v = vec!["FLAGS".into(), tid.clone()];
            v.extend(flags.iter().cloned());
            v
        }
        CoreOp::ClassFlags(cid, flags) => {
            let mut v = vec!["FLAGS_C".into(), cid.clone()];
            v.extend(flags.iter().cloned());
            v
        }
        CoreOp::Extends(child, parent) => vec!["EXT".into(), child.clone(), parent.clone()],
        CoreOp::Implements(cid, iid) => vec!["IMPL".into(), cid.clone(), iid.clone()],
        CoreOp::Injects(cid, deps) => {
            let mut v = vec!["INJECTS".into(), cid.clone()];
            v.extend(deps.iter().cloned());
            v
        }
        CoreOp::Import(alias, module, named) => {
            vec![
                "IMP".into(),
                alias.clone(),
                module.clone(),
                named.clone(),
            ]
        }
        CoreOp::TypeAlias(alias, original) => vec!["TYPE".into(), alias.clone(), original.clone()],
    }
}

/// Deserialize a positional tuple back to a CoreOp.
/// Returns None if the tuple doesn't match any known opcode.
pub fn tuple_to_op(tuple: &[String]) -> Option<CoreOp> {
    if tuple.is_empty() {
        return None;
    }
    match tuple[0].as_str() {
        "DEF_C" => {
            if tuple.len() >= 3 {
                Some(CoreOp::DefClass(tuple[1].clone(), tuple[2].clone()))
            } else {
                None
            }
        }
        "DEF_M" => {
            if tuple.len() >= 4 {
                Some(CoreOp::DefMethod(
                    tuple[1].clone(),
                    tuple[2].clone(),
                    tuple[3].clone(),
                ))
            } else {
                None
            }
        }
        "DEF_F" => {
            if tuple.len() >= 4 {
                Some(CoreOp::DefField(
                    tuple[1].clone(),
                    tuple[2].clone(),
                    tuple[3].clone(),
                ))
            } else {
                None
            }
        }
        "DEF_I" => {
            if tuple.len() >= 3 {
                Some(CoreOp::DefInterface(tuple[1].clone(), tuple[2].clone()))
            } else {
                None
            }
        }
        "SIG" => {
            if tuple.len() >= 5 {
                Some(CoreOp::Param(
                    tuple[1].clone(),
                    tuple[2].clone(),
                    tuple[3].clone(),
                    tuple[4].clone(),
                ))
            } else {
                None
            }
        }
        "RET" => {
            if tuple.len() >= 3 {
                Some(CoreOp::Return(tuple[1].clone(), tuple[2].clone()))
            } else {
                None
            }
        }
        "FIELD_T" => {
            if tuple.len() >= 3 {
                Some(CoreOp::FieldType(tuple[1].clone(), tuple[2].clone()))
            } else {
                None
            }
        }
        "FLAGS" => {
            if tuple.len() >= 3 {
                Some(CoreOp::Flags(
                    tuple[1].clone(),
                    tuple[2..].to_vec(),
                ))
            } else {
                None
            }
        }
        "FLAGS_C" => {
            if tuple.len() >= 3 {
                Some(CoreOp::ClassFlags(
                    tuple[1].clone(),
                    tuple[2..].to_vec(),
                ))
            } else {
                None
            }
        }
        "EXT" => {
            if tuple.len() >= 3 {
                Some(CoreOp::Extends(tuple[1].clone(), tuple[2].clone()))
            } else {
                None
            }
        }
        "IMPL" => {
            if tuple.len() >= 3 {
                Some(CoreOp::Implements(tuple[1].clone(), tuple[2].clone()))
            } else {
                None
            }
        }
        "INJECTS" => {
            if tuple.len() >= 3 {
                Some(CoreOp::Injects(
                    tuple[1].clone(),
                    tuple[2..].to_vec(),
                ))
            } else {
                None
            }
        }
        "IMP" => {
            if tuple.len() >= 4 {
                Some(CoreOp::Import(
                    tuple[1].clone(),
                    tuple[2].clone(),
                    tuple[3].clone(),
                ))
            } else {
                None
            }
        }
        "TYPE" => {
            if tuple.len() >= 3 {
                Some(CoreOp::TypeAlias(tuple[1].clone(), tuple[2].clone()))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Serialize a CompiledIR to the wire format.
pub fn ir_to_wire(ir: &CompiledIR) -> Value {
    let tuples: Vec<Vec<String>> = ir.instructions.iter().map(op_to_tuple).collect();

    json!({
        "file": ir.file_id,
        "v": ir.version,
        "ir": tuples
    })
}

/// Deserialize a CompiledIR from the wire format.
pub fn wire_to_ir(value: &Value) -> Option<CompiledIR> {
    let file_id = value.get("file")?.as_str()?.to_string();
    let version = value.get("v")?.as_u64()?;
    let ir_array = value.get("ir")?.as_array()?;

    let mut instructions = Vec::new();
    for tuple_val in ir_array {
        let tuple: Vec<String> = tuple_val
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        if let Some(op) = tuple_to_op(&tuple) {
            instructions.push(op);
        }
    }

    Some(CompiledIR {
        file_id,
        instructions,
        version,
    })
}

#[cfg(test)]
#[path = "../tests/ir/wire.rs"]
mod tests;