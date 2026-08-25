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
use serde_json::{Value, json};

/// Errors during wire format decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Missing required field
    MissingField(String),
    /// Invalid field type (expected a different JSON type)
    InvalidFieldType(String),
    /// Unknown opcode in instruction tuple
    UnknownOpcode(String),
    /// Malformed tuple (wrong arity for the opcode)
    MalformedTuple(String),
    /// Input was not a valid JSON object/array
    InvalidInput(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::MissingField(name) => write!(f, "missing field: {}", name),
            DecodeError::InvalidFieldType(name) => write!(f, "invalid field type: {}", name),
            DecodeError::UnknownOpcode(op) => write!(f, "unknown opcode: {}", op),
            DecodeError::MalformedTuple(msg) => write!(f, "malformed tuple: {}", msg),
            DecodeError::InvalidInput(msg) => write!(f, "invalid input: {}", msg),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Serialize a single CoreOp to its positional tuple representation.
pub fn op_to_tuple(op: &CoreOp) -> Vec<String> {
    match op {
        CoreOp::DefClass(id, name) => vec!["DEF_C".into(), id.clone(), name.clone()],
        CoreOp::DefMethod(cid, mid, name) => {
            vec!["DEF_M".into(), cid.clone(), mid.clone(), name.clone()]
        }
        CoreOp::DefField(cid, fid, name) => {
            vec!["DEF_F".into(), cid.clone(), fid.clone(), name.clone()]
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
            vec!["IMP".into(), alias.clone(), module.clone(), named.clone()]
        }
        CoreOp::TypeAlias(alias, original) => vec!["TYPE".into(), alias.clone(), original.clone()],
        CoreOp::Pattern(name, args) => {
            let mut v = vec!["PAT".into(), name.clone()];
            v.extend(args.iter().cloned());
            v
        }
        // Edit Mode: Verbatim Method Bodies
        CoreOp::Body(mid, text) => {
            vec!["BODY".into(), mid.clone(), text.clone()]
        }
        // R-43a: Execution Semantics
        CoreOp::DataFlow(mid, direction, target) => {
            vec![
                "DATAFLOW".into(),
                mid.clone(),
                direction.clone(),
                target.clone(),
            ]
        }
        CoreOp::ControlFlow(mid, kind, target) => {
            vec!["CTRL".into(), mid.clone(), kind.clone(), target.clone()]
        }
        CoreOp::SideEffect(mid, effect_type) => {
            vec!["EFFECT".into(), mid.clone(), effect_type.clone()]
        }
        CoreOp::ExecutionContext(mid, context_type) => {
            vec!["CTX".into(), mid.clone(), context_type.clone()]
        }
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
                Some(CoreOp::Flags(tuple[1].clone(), tuple[2..].to_vec()))
            } else {
                None
            }
        }
        "FLAGS_C" => {
            if tuple.len() >= 3 {
                Some(CoreOp::ClassFlags(tuple[1].clone(), tuple[2..].to_vec()))
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
                Some(CoreOp::Injects(tuple[1].clone(), tuple[2..].to_vec()))
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
        "PAT" => {
            if tuple.len() >= 3 {
                Some(CoreOp::Pattern(tuple[1].clone(), tuple[2..].to_vec()))
            } else {
                None
            }
        }
        // Edit Mode: Verbatim Method Bodies
        "BODY" => {
            if tuple.len() >= 3 {
                Some(CoreOp::Body(tuple[1].clone(), tuple[2].clone()))
            } else {
                None
            }
        }
        // R-43a: Execution Semantics
        "DATAFLOW" => {
            if tuple.len() >= 4 {
                Some(CoreOp::DataFlow(
                    tuple[1].clone(),
                    tuple[2].clone(),
                    tuple[3].clone(),
                ))
            } else {
                None
            }
        }
        "CTRL" => {
            if tuple.len() >= 4 {
                Some(CoreOp::ControlFlow(
                    tuple[1].clone(),
                    tuple[2].clone(),
                    tuple[3].clone(),
                ))
            } else {
                None
            }
        }
        "EFFECT" => {
            if tuple.len() >= 3 {
                Some(CoreOp::SideEffect(tuple[1].clone(), tuple[2].clone()))
            } else {
                None
            }
        }
        "CTX" => {
            if tuple.len() >= 3 {
                Some(CoreOp::ExecutionContext(tuple[1].clone(), tuple[2].clone()))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Serialize a CompiledIR to the wire format.
///
/// NF-07: Now emits `"encoding": "named"` for consistency with the
/// positional and tagged wire formats. Previously the `encoding` field
/// was absent, making it impossible for readers to distinguish the
/// three formats without knowing the ingestion path.
pub fn ir_to_wire(ir: &CompiledIR) -> Value {
    let tuples: Vec<Vec<String>> = ir.instructions.iter().map(op_to_tuple).collect();

    json!({
        "file": ir.file_id,
        "v": ir.version,
        "encoding": "named",
        "ir": tuples
    })
}

/// Detect the encoding format from a wire value and decode accordingly.
///
/// Supports all wire formats: "named", "positional", "tagged", "string_table",
/// "hierarchical", and "binary".
/// Returns None if the encoding is unrecognized or decoding fails.
pub fn wire_to_ir_detect(value: &Value) -> Option<super::compiler::CompiledIR> {
    let encoding = value
        .get("encoding")
        .and_then(|v| v.as_str())
        .unwrap_or("named");
    match encoding {
        "string_table" => super::string_table::wire_to_ir(value),
        "hierarchical" => super::hierarchical::wire_to_ir(value).ok(),
        "binary" => super::binary_wire::binary_wire_json_to_ir(value),
        "named" | "tagged" => wire_to_ir(value).ok(),
        _ => wire_to_ir(value).ok(), // fallback to named
    }
}

/// Deserialize a CompiledIR from the wire format.
///
/// # Errors
///
/// Returns `Err(DecodeError)` if:
/// - Required fields (`file`, `v`, `ir`) are missing
/// - A tuple cannot be decoded (unknown opcode or malformed arity)
///
/// This ensures no input corruption is silently swallowed (F-19).
pub fn wire_to_ir(value: &Value) -> Result<CompiledIR, DecodeError> {
    let file_id = value
        .get("file")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| DecodeError::MissingField("file".into()))?;
    let version = value
        .get("v")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| DecodeError::MissingField("v".into()))?;
    let ir_array = value
        .get("ir")
        .and_then(|v| v.as_array())
        .ok_or_else(|| DecodeError::MissingField("ir".into()))?;

    let mut instructions = Vec::new();
    for (i, tuple_val) in ir_array.iter().enumerate() {
        let tuple: Vec<String> = tuple_val
            .as_array()
            .ok_or_else(|| DecodeError::InvalidFieldType(format!("ir[{}]: expected array", i)))?
            .iter()
            .map(|v| {
                v.as_str().map(|s| s.to_string()).ok_or_else(|| {
                    DecodeError::InvalidFieldType(format!("ir[{}]: expected string element", i))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let op = tuple_to_op(&tuple).ok_or_else(|| {
            if tuple.is_empty() {
                DecodeError::MalformedTuple(format!("ir[{}]: empty tuple", i))
            } else {
                DecodeError::UnknownOpcode(format!("ir[{}]: unknown opcode '{}'", i, tuple[0]))
            }
        })?;
        instructions.push(op);
    }

    Ok(CompiledIR {
        file_id,
        instructions,
        version,
    })
}

#[cfg(test)]
#[path = "../tests/ir/wire.rs"]
mod tests;

/// Comprehensive round-trip and randomized property tests for all wire formats.
/// Covers: named, binary, hierarchical, compact delta, and SemanticIntent.
#[cfg(test)]
#[path = "../tests/ir/round_trip.rs"]
mod round_trip_tests;
