// src/ir/binary_wire/decode.rs
//
// Binary wire DECODING: the bytes -> CompiledIR inverse of `encode`.
//
// Split out of `src/ir/binary_wire.rs` (active-file size policy): the module
// had exceeded the 615-line ceiling. This is a pure relocation -- the code
// below is byte-for-byte the previous implementation, and `binary_wire`
// re-exports `decode` and `BinaryDecodeError` so every existing path
// (`crate::ir::binary_wire::decode`) keeps resolving.
//
// Child-module access: a private item of an ancestor module is visible in its
// descendants, so `use super::*` supplies the opcode index table
// (`OP_*`), the version constants, and the varint/string primitives declared
// by the parent.

use super::*;

// ── Binary Decoding ───────────────────────────────────────────────

/// Errors that can occur during binary wire decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryDecodeError {
    /// Invalid magic bytes (not a binary wire file)
    InvalidMagic,
    /// Unsupported schema version
    UnsupportedVersion(u8),
    /// Unexpected end of data
    TruncatedData(String),
    /// Invalid opcode index
    UnknownOpcode(u8),
    /// String table index out of bounds
    InvalidStringIndex(u64),
    /// UTF-8 decoding failure
    InvalidUtf8(String),
    /// Unknown declaration modifier in a typed modifier opcode
    InvalidDeclarationModifier(String),
    /// Unknown control summary in the typed summary opcode
    InvalidControlSummary(String),
}

impl std::fmt::Display for BinaryDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryDecodeError::InvalidMagic => write!(f, "invalid magic bytes"),
            BinaryDecodeError::UnsupportedVersion(v) => {
                write!(f, "unsupported binary version: {}", v)
            }
            BinaryDecodeError::TruncatedData(msg) => write!(f, "truncated data: {}", msg),
            BinaryDecodeError::UnknownOpcode(idx) => write!(f, "unknown opcode index: {}", idx),
            BinaryDecodeError::InvalidStringIndex(idx) => {
                write!(f, "invalid string table index: {}", idx)
            }
            BinaryDecodeError::InvalidUtf8(msg) => write!(f, "invalid UTF-8: {}", msg),
            BinaryDecodeError::InvalidDeclarationModifier(value) => {
                write!(f, "invalid declaration modifier: {value}")
            }
            BinaryDecodeError::InvalidControlSummary(value) => {
                write!(f, "invalid control summary: {value}")
            }
        }
    }
}

impl std::error::Error for BinaryDecodeError {}

/// Decode binary wire format bytes back into a CompiledIR.
///
/// # Errors
///
/// Returns `Err(BinaryDecodeError)` if:
/// - The magic bytes don't match
/// - The version is unsupported
/// - The data is truncated
/// - An opcode index is out of range
/// - A string table index is out of bounds
pub fn decode(data: &[u8]) -> Result<CompiledIR, BinaryDecodeError> {
    // 1. Header
    if data.len() < 3 {
        return Err(BinaryDecodeError::TruncatedData("header too short".into()));
    }
    if data[0] != MAGIC[0] || data[1] != MAGIC[1] {
        return Err(BinaryDecodeError::InvalidMagic);
    }
    if data[2] != VERSION && data[2] != VERSION_PRE_SPAN && data[2] != VERSION_LEGACY {
        return Err(BinaryDecodeError::UnsupportedVersion(data[2]));
    }
    let mut pos = 3;

    // 2. IR version (edit sequence number) — only in VERSION (0x02)
    // Versions 0x02 and 0x03 carry the IR edit-sequence number; only
    // VERSION_LEGACY (0x01) omits it.
    let ir_version = if data[2] != VERSION_LEGACY {
        let (ver, consumed) = read_varint(&data[pos..])
            .ok_or_else(|| BinaryDecodeError::TruncatedData("IR version".into()))?;
        pos += consumed;
        ver
    } else {
        // VERSION_LEGACY (0x01) — no IR version stored
        0
    };

    // 3. String table
    let (table_len, consumed) = read_varint(&data[pos..])
        .ok_or_else(|| BinaryDecodeError::TruncatedData("string table count".into()))?;
    pos += consumed;

    let mut strings: Vec<String> = Vec::with_capacity(table_len as usize);
    for i in 0..table_len {
        if pos >= data.len() {
            return Err(BinaryDecodeError::TruncatedData(format!(
                "string table entry {}",
                i
            )));
        }
        let (s, consumed) = read_string(&data[pos..]).ok_or_else(|| {
            BinaryDecodeError::TruncatedData(format!("string data for entry {}", i))
        })?;
        pos += consumed;
        strings.push(s);
    }

    // 3. Instructions
    let (inst_count, consumed) = read_varint(&data[pos..])
        .ok_or_else(|| BinaryDecodeError::TruncatedData("instruction count".into()))?;
    pos += consumed;

    // Helper: read a string table index varint and return the string
    let read_operand = |data: &[u8], pos: &mut usize| -> Result<String, BinaryDecodeError> {
        let (idx, consumed) = read_varint(data)
            .ok_or_else(|| BinaryDecodeError::TruncatedData("operand index".into()))?;
        *pos += consumed;
        let idx_usize = idx as usize;
        if idx_usize >= strings.len() {
            return Err(BinaryDecodeError::InvalidStringIndex(idx));
        }
        Ok(strings[idx_usize].clone())
    };

    let mut instructions = Vec::with_capacity(inst_count as usize);

    for _ in 0..inst_count {
        if pos >= data.len() {
            return Err(BinaryDecodeError::TruncatedData("opcode byte".into()));
        }
        let op_idx = data[pos];
        pos += 1;

        if op_idx > OP_MAX {
            return Err(BinaryDecodeError::UnknownOpcode(op_idx));
        }

        let op = if is_variadic(op_idx) {
            // Read variadic count prefix
            let (var_count, consumed) = read_varint(&data[pos..])
                .ok_or_else(|| BinaryDecodeError::TruncatedData("variadic operand count".into()))?;
            pos += consumed;

            let mut operands: Vec<String> = Vec::with_capacity(var_count as usize);
            for _ in 0..var_count {
                let operand = read_operand(&data[pos..], &mut pos)?;
                operands.push(operand);
            }

            match op_idx {
                OP_FLAGS => {
                    if operands.is_empty() {
                        return Err(BinaryDecodeError::TruncatedData(
                            "FLAGS needs at least target_id".into(),
                        ));
                    }
                    let tid = operands.remove(0);
                    CoreOp::Flags(tid, operands)
                }
                OP_FLAGS_C => {
                    if operands.is_empty() {
                        return Err(BinaryDecodeError::TruncatedData(
                            "FLAGS_C needs at least class_id".into(),
                        ));
                    }
                    let cid = operands.remove(0);
                    CoreOp::ClassFlags(cid, operands)
                }
                OP_INJECTS => {
                    if operands.is_empty() {
                        return Err(BinaryDecodeError::TruncatedData(
                            "INJECTS needs at least class_id".into(),
                        ));
                    }
                    let cid = operands.remove(0);
                    CoreOp::Injects(cid, operands)
                }
                OP_PAT => {
                    if operands.is_empty() {
                        return Err(BinaryDecodeError::TruncatedData(
                            "PAT needs at least pattern_name".into(),
                        ));
                    }
                    let name = operands.remove(0);
                    CoreOp::Pattern(name, operands)
                }
                OP_MOD_M | OP_MOD_C => {
                    if operands.len() < 2 {
                        let name = if op_idx == OP_MOD_M { "MOD_M" } else { "MOD_C" };
                        return Err(BinaryDecodeError::TruncatedData(format!(
                            "{name} needs a target_id and at least one modifier"
                        )));
                    }
                    let target = operands.remove(0);
                    let modifiers = operands
                        .into_iter()
                        .map(|value| {
                            DeclarationModifier::from_serialized(&value).ok_or_else(|| {
                                BinaryDecodeError::InvalidDeclarationModifier(value.clone())
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if op_idx == OP_MOD_M {
                        CoreOp::MethodModifiers(target, modifiers)
                    } else {
                        CoreOp::ClassModifiers(target, modifiers)
                    }
                }
                OP_CTRL_SUM => {
                    if operands.len() < 2 {
                        return Err(BinaryDecodeError::TruncatedData(
                            "CTRL_SUM needs a method_id and at least one summary".into(),
                        ));
                    }
                    let method_id = operands.remove(0);
                    let summaries = operands
                        .into_iter()
                        .map(|value| {
                            ControlSummary::from_serialized(&value).ok_or_else(|| {
                                BinaryDecodeError::InvalidControlSummary(value.clone())
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    CoreOp::ControlSummary(method_id, summaries)
                }
                _ => unreachable!(),
            }
        } else {
            match op_idx {
                OP_DEF_C => {
                    let name = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::DefClass(String::new(), name)
                }
                OP_DEF_M => {
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let name = read_operand(&data[pos..], &mut pos)?;
                    // Need class_id too — use "C0" as placeholder since binary
                    // doesn't store class_id redundantly
                    CoreOp::DefMethod(String::new(), mid, name)
                }
                OP_DEF_F => {
                    let fid = read_operand(&data[pos..], &mut pos)?;
                    let name = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::DefField(String::new(), fid, name)
                }
                OP_DEF_I => {
                    let name = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::DefInterface(String::new(), name)
                }
                OP_SIG => {
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let pid = read_operand(&data[pos..], &mut pos)?;
                    let ty = read_operand(&data[pos..], &mut pos)?;
                    let name = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::Param(mid, pid, ty, name)
                }
                OP_RET => {
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let ty = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::Return(mid, ty)
                }
                OP_FIELD_T => {
                    let fid = read_operand(&data[pos..], &mut pos)?;
                    let ty = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::FieldType(fid, ty)
                }
                OP_EXT => {
                    let parent = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::Extends(String::new(), parent)
                }
                OP_IMPL => {
                    let iid = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::Implements(String::new(), iid)
                }
                OP_IMP => {
                    let module = read_operand(&data[pos..], &mut pos)?;
                    let named = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::Import(String::new(), module, named)
                }
                OP_TYPE => {
                    let original = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::TypeAlias(String::new(), original)
                }
                // Edit Mode: Verbatim Method Bodies
                // v0x03 streams append a has-span flag varint plus two raw
                // byte-offset varints after the string-table operands;
                // pre-span streams stop at the operands and decode
                // span-less (apply_edit plan Phase 1 compat gate).
                OP_BODY => {
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let text = read_operand(&data[pos..], &mut pos)?;
                    if data[2] == VERSION {
                        let (has_span, consumed) = read_varint(&data[pos..]).ok_or_else(|| {
                            BinaryDecodeError::TruncatedData("BODY span flag".into())
                        })?;
                        pos += consumed;
                        if has_span == 1 {
                            let (start, consumed) = read_varint(&data[pos..]).ok_or_else(|| {
                                BinaryDecodeError::TruncatedData("BODY start_byte".into())
                            })?;
                            pos += consumed;
                            let (end, consumed) = read_varint(&data[pos..]).ok_or_else(|| {
                                BinaryDecodeError::TruncatedData("BODY end_byte".into())
                            })?;
                            pos += consumed;
                            CoreOp::Body(mid, text, Some(start), Some(end))
                        } else {
                            CoreOp::Body(mid, text, None, None)
                        }
                    } else {
                        CoreOp::Body(mid, text, None, None)
                    }
                }
                // Structural invocations (native call graph).
                // [caller_idx, callee_idx, argc_varint] — the count is a raw
                // varint following the two string-table operands. The spread
                // qualifier is carried by the OPCODE, not an operand, so an
                // exact call keeps its established encoding and a spread call
                // is never decoded as an exact one.
                OP_CALL | OP_CALL_SPREAD => {
                    let caller = read_operand(&data[pos..], &mut pos)?;
                    let callee = read_operand(&data[pos..], &mut pos)?;
                    let (argc, consumed) = read_varint(&data[pos..]).ok_or_else(|| {
                        BinaryDecodeError::TruncatedData("CALL explicit_arg_count".into())
                    })?;
                    pos += consumed;
                    CoreOp::Call(caller, callee, argc as usize, op_idx == OP_CALL_SPREAD)
                }
                // R-43a: Execution Semantics
                OP_DATAFLOW => {
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let direction = read_operand(&data[pos..], &mut pos)?;
                    let target = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::DataFlow(mid, direction, target)
                }
                OP_CTRL => {
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let kind = read_operand(&data[pos..], &mut pos)?;
                    let target = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::ControlFlow(mid, kind, target)
                }
                OP_EFFECT => {
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let effect_type = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::SideEffect(mid, effect_type)
                }
                OP_CTX => {
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let context_type = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::ExecutionContext(mid, context_type)
                }
                _ => return Err(BinaryDecodeError::UnknownOpcode(op_idx)),
            }
        };

        instructions.push(op);
    }

    // Note: The binary format doesn't encode file_id or the IR version
    // (edit sequence count). The caller (e.g. sqlite_store) is responsible
    // for setting file_id and version from the DB columns.
    Ok(CompiledIR {
        file_id: "bin".to_string(),
        instructions,
        version: ir_version,
    })
}
