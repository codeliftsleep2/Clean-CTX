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
    /// Malformed typed pattern-fact payload
    InvalidPatternFact,
    /// Unknown side-effect value in the typed side-effect opcode
    InvalidSideEffect(String),
    /// Unknown execution-context value in the typed execution-context opcode
    InvalidExecutionContext(String),
    /// BODY span flag is not the canonical 0 or 1 value
    InvalidBodySpanFlag(u64),
    /// A decoded integer cannot be represented by the target Rust type
    IntegerOverflow(&'static str),
    /// Bytes remain after the declared instruction stream
    TrailingData(usize),
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
            BinaryDecodeError::InvalidPatternFact => f.write_str("invalid pattern fact payload"),
            BinaryDecodeError::InvalidSideEffect(value) => {
                write!(f, "invalid side effect: {value}")
            }
            BinaryDecodeError::InvalidExecutionContext(value) => {
                write!(f, "invalid execution context: {value}")
            }
            BinaryDecodeError::InvalidBodySpanFlag(value) => {
                write!(f, "invalid BODY span flag: {value}")
            }
            BinaryDecodeError::IntegerOverflow(field) => {
                write!(f, "integer overflow decoding {field}")
            }
            BinaryDecodeError::TrailingData(bytes) => {
                write!(f, "trailing data after instruction stream: {bytes} bytes")
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
    if data[2] != VERSION {
        return Err(BinaryDecodeError::UnsupportedVersion(data[2]));
    }
    let mut pos = 3;

    // 2. IR version (edit sequence number)
    let (ir_version, consumed) = read_varint(&data[pos..])
        .ok_or_else(|| BinaryDecodeError::TruncatedData("IR version".into()))?;
    pos += consumed;

    // 3. String table
    let (table_len, consumed) = read_varint(&data[pos..])
        .ok_or_else(|| BinaryDecodeError::TruncatedData("string table count".into()))?;
    pos += consumed;

    let table_capacity = usize::try_from(table_len)
        .map_err(|_| BinaryDecodeError::IntegerOverflow("string table count"))?;
    let mut strings: Vec<String> = Vec::with_capacity(table_capacity.min(data.len()));
    for i in 0..table_len {
        if pos >= data.len() {
            return Err(BinaryDecodeError::TruncatedData(format!(
                "string table entry {}",
                i
            )));
        }
        let (len, consumed) = read_varint(&data[pos..]).ok_or_else(|| {
            BinaryDecodeError::TruncatedData(format!("string length for entry {i}"))
        })?;
        pos += consumed;
        let len = usize::try_from(len)
            .map_err(|_| BinaryDecodeError::IntegerOverflow("string byte length"))?;
        let end = pos
            .checked_add(len)
            .ok_or(BinaryDecodeError::IntegerOverflow("string byte range"))?;
        let bytes = data.get(pos..end).ok_or_else(|| {
            BinaryDecodeError::TruncatedData(format!("string data for entry {i}"))
        })?;
        let value = std::str::from_utf8(bytes)
            .map_err(|error| BinaryDecodeError::InvalidUtf8(error.to_string()))?;
        strings.push(value.to_owned());
        pos = end;
    }

    // Helper: read a string table index varint and return the string
    let read_operand = |data: &[u8], pos: &mut usize| -> Result<String, BinaryDecodeError> {
        let (idx, consumed) = read_varint(data)
            .ok_or_else(|| BinaryDecodeError::TruncatedData("operand index".into()))?;
        *pos += consumed;
        let idx_usize = usize::try_from(idx)
            .map_err(|_| BinaryDecodeError::IntegerOverflow("string table index"))?;
        if idx_usize >= strings.len() {
            return Err(BinaryDecodeError::InvalidStringIndex(idx));
        }
        Ok(strings[idx_usize].clone())
    };

    // 4. File identity and instructions
    let file_id = read_operand(&data[pos..], &mut pos)?;
    let (inst_count, consumed) = read_varint(&data[pos..])
        .ok_or_else(|| BinaryDecodeError::TruncatedData("instruction count".into()))?;
    pos += consumed;

    let instruction_capacity = usize::try_from(inst_count)
        .map_err(|_| BinaryDecodeError::IntegerOverflow("instruction count"))?;
    let mut instructions = Vec::with_capacity(instruction_capacity.min(data.len()));

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

            let operand_capacity = usize::try_from(var_count)
                .map_err(|_| BinaryDecodeError::IntegerOverflow("variadic operand count"))?;
            let mut operands: Vec<String> = Vec::with_capacity(operand_capacity.min(data.len()));
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
                OP_MOD_M | OP_MOD_C | OP_MOD_I => {
                    if operands.len() < 2 {
                        let name = match op_idx {
                            OP_MOD_M => "MOD_M",
                            OP_MOD_C => "MOD_C",
                            _ => "MOD_I",
                        };
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
                    match op_idx {
                        OP_MOD_M => CoreOp::MethodModifiers(target, modifiers),
                        OP_MOD_C => CoreOp::ClassModifiers(target, modifiers),
                        _ => CoreOp::InterfaceModifiers(target, modifiers),
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
                OP_PAT_FACT => {
                    if operands.len() < 2 {
                        return Err(BinaryDecodeError::TruncatedData(
                            "PAT_FACT needs a method_id and at least one fact".into(),
                        ));
                    }
                    let method_id = operands.remove(0);
                    let facts = PatternFact::parse_all(&operands)
                        .ok_or(BinaryDecodeError::InvalidPatternFact)?;
                    CoreOp::PatternFacts(method_id, facts)
                }
                _ => unreachable!(),
            }
        } else {
            match op_idx {
                OP_DEF_C => {
                    let cid = read_operand(&data[pos..], &mut pos)?;
                    let name = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::DefClass(cid, name)
                }
                OP_DEF_M => {
                    let cid = read_operand(&data[pos..], &mut pos)?;
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let name = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::DefMethod(cid, mid, name)
                }
                OP_DEF_F => {
                    let cid = read_operand(&data[pos..], &mut pos)?;
                    let fid = read_operand(&data[pos..], &mut pos)?;
                    let name = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::DefField(cid, fid, name)
                }
                OP_DEF_I => {
                    let iid = read_operand(&data[pos..], &mut pos)?;
                    let name = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::DefInterface(iid, name)
                }
                OP_DEF_IM => {
                    let iid = read_operand(&data[pos..], &mut pos)?;
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let name = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::DefInterfaceMethod(iid, mid, name)
                }
                OP_DEF_IF => {
                    let iid = read_operand(&data[pos..], &mut pos)?;
                    let fid = read_operand(&data[pos..], &mut pos)?;
                    let name = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::DefInterfaceField(iid, fid, name)
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
                    let child = read_operand(&data[pos..], &mut pos)?;
                    let parent = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::Extends(child, parent)
                }
                OP_EXT_I => {
                    let child = read_operand(&data[pos..], &mut pos)?;
                    let parent = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::InterfaceExtends(child, parent)
                }
                OP_IMPL => {
                    let cid = read_operand(&data[pos..], &mut pos)?;
                    let iid = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::Implements(cid, iid)
                }
                OP_IMP => {
                    let alias = read_operand(&data[pos..], &mut pos)?;
                    let module = read_operand(&data[pos..], &mut pos)?;
                    let named = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::Import(alias, module, named)
                }
                OP_TYPE => {
                    let alias = read_operand(&data[pos..], &mut pos)?;
                    let original = read_operand(&data[pos..], &mut pos)?;
                    CoreOp::TypeAlias(alias, original)
                }
                // Edit Mode: Verbatim Method Bodies
                // v0x04: [mid, text, has_span, start?, end?].
                OP_BODY => {
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let text = read_operand(&data[pos..], &mut pos)?;
                    let (has_span, consumed) = read_varint(&data[pos..])
                        .ok_or_else(|| BinaryDecodeError::TruncatedData("BODY span flag".into()))?;
                    pos += consumed;
                    match has_span {
                        1 => {
                            let (start, consumed) = read_varint(&data[pos..]).ok_or_else(|| {
                                BinaryDecodeError::TruncatedData("BODY start_byte".into())
                            })?;
                            pos += consumed;
                            let (end, consumed) = read_varint(&data[pos..]).ok_or_else(|| {
                                BinaryDecodeError::TruncatedData("BODY end_byte".into())
                            })?;
                            pos += consumed;
                            CoreOp::Body(mid, text, Some(start), Some(end))
                        }
                        0 => CoreOp::Body(mid, text, None, None),
                        value => return Err(BinaryDecodeError::InvalidBodySpanFlag(value)),
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
                    let argc = usize::try_from(argc)
                        .map_err(|_| BinaryDecodeError::IntegerOverflow("CALL argument count"))?;
                    CoreOp::Call(caller, callee, argc, op_idx == OP_CALL_SPREAD)
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
                    let effect = SideEffectKind::from_serialized(&effect_type)
                        .ok_or(BinaryDecodeError::InvalidSideEffect(effect_type))?;
                    CoreOp::SideEffect(mid, effect)
                }
                OP_CTX => {
                    let mid = read_operand(&data[pos..], &mut pos)?;
                    let context_type = read_operand(&data[pos..], &mut pos)?;
                    let context = ExecutionContextKind::from_serialized(&context_type)
                        .ok_or(BinaryDecodeError::InvalidExecutionContext(context_type))?;
                    CoreOp::ExecutionContext(mid, context)
                }
                _ => return Err(BinaryDecodeError::UnknownOpcode(op_idx)),
            }
        };

        instructions.push(op);
    }

    if pos != data.len() {
        return Err(BinaryDecodeError::TrailingData(data.len() - pos));
    }

    Ok(CompiledIR {
        file_id,
        instructions,
        version: ir_version,
    })
}
