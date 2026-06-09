// src/ir/binary_wire.rs
//
// Phase II: Ultra-Compact IR — Binary Wire Format (Idea #1).
//
// Replaces JSON with a compact binary encoding for the IR instruction
// stream. Achieves ~60-70% wire byte reduction compared to positional JSON.
//
// Encoding spec:
// ┌─────────────────────────────────────────────────────────┐
// │ Header: magic(2) + version(1)                           │
// │ String Table: [count(varint), (len(varint), bytes)*]   │
// │ Instructions: [count(varint), instruction*]             │
// │                                                          │
// │ Instruction:                                             │
// │   opcode_idx: u8 (0-14 for 15 core opcodes + patterns)  │
// │   operands: [varint]* (string table indices)             │
// │   For variadic ops: operand_count as varint prefix       │
// └─────────────────────────────────────────────────────────┘
//
// Varint encoding: 7-bit groups with MSB continuation flag.
//   - Each byte: 7 data bits + 1 continuation bit
//   - Continuation bit = 1 means more bytes follow
//   - Continuation bit = 0 means last byte
//
// Magic bytes: 0xCC, 0x01 ("Clean CTX binary v1")
// Version byte: 0x01
//
// Trade-off: Not human-readable. Best used as an optional transport
// encoding. The JSON wire format remains the default for debugging
// and mixed streams.

use super::compiler::CompiledIR;
use super::opcodes::CoreOp;
use super::string_table::StringTable;

/// Magic bytes for the binary wire format: "CC" + version 01
const MAGIC: [u8; 2] = [0xCC, 0x01];
/// Current schema version
const VERSION: u8 = 0x01;

/// Opcode index assignment (0-14)
const OP_DEF_C: u8 = 0;
const OP_DEF_M: u8 = 1;
const OP_DEF_F: u8 = 2;
const OP_DEF_I: u8 = 3;
const OP_SIG: u8 = 4;     // Param
const OP_RET: u8 = 5;     // Return
const OP_FIELD_T: u8 = 6; // FieldType
const OP_FLAGS: u8 = 7;
const OP_FLAGS_C: u8 = 8; // ClassFlags
const OP_EXT: u8 = 9;     // Extends
const OP_IMPL: u8 = 10;   // Implements
const OP_INJECTS: u8 = 11;
const OP_IMP: u8 = 12;    // Import
const OP_TYPE: u8 = 13;   // TypeAlias
const OP_PAT: u8 = 14;    // Pattern

/// Opcodes that have a variable number of operands (beyond the first one).
fn is_variadic(op_idx: u8) -> bool {
    matches!(op_idx, OP_FLAGS | OP_FLAGS_C | OP_INJECTS | OP_PAT)
}

/// Convert a CoreOp to its u8 opcode index.
fn op_to_index(op: &CoreOp) -> u8 {
    match op {
        CoreOp::DefClass(..) => OP_DEF_C,
        CoreOp::DefMethod(..) => OP_DEF_M,
        CoreOp::DefField(..) => OP_DEF_F,
        CoreOp::DefInterface(..) => OP_DEF_I,
        CoreOp::Param(..) => OP_SIG,
        CoreOp::Return(..) => OP_RET,
        CoreOp::FieldType(..) => OP_FIELD_T,
        CoreOp::Flags(..) => OP_FLAGS,
        CoreOp::ClassFlags(..) => OP_FLAGS_C,
        CoreOp::Extends(..) => OP_EXT,
        CoreOp::Implements(..) => OP_IMPL,
        CoreOp::Injects(..) => OP_INJECTS,
        CoreOp::Import(..) => OP_IMP,
        CoreOp::TypeAlias(..) => OP_TYPE,
        CoreOp::Pattern(..) => OP_PAT,
    }
}


// ── Varint Encoding ──────────────────────────────────────────────

/// Encode a u64 value as a varint (unsigned, little-endian base-128).
/// Each byte: 7 data bits (LSB first) + MSB continuation flag.
fn write_varint(buf: &mut Vec<u8>, value: u64) {
    let mut v = value;
    loop {
        if v < 128 {
            buf.push(v as u8);
            break;
        } else {
            buf.push((v as u8 & 0x7F) | 0x80);
            v >>= 7;
        }
    }
}

/// Decode a varint from a byte slice, returning (value, bytes_consumed).
/// Returns None if the slice is empty or the varint is malformed.
fn read_varint(data: &[u8]) -> Option<(u64, usize)> {
    if data.is_empty() {
        return None;
    }
    let mut value: u64 = 0;
    let mut shift: u64 = 0;
    let mut consumed = 0;

    for &byte in data {
        consumed += 1;
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some((value, consumed));
        }
        shift += 7;
        // Prevent overflow for very large varints
        if shift > 63 {
            return None;
        }
    }
    // Reached end of data with continuation bit still set
    None
}

/// Encode a string as (length varint, UTF-8 bytes).
fn write_string(buf: &mut Vec<u8>, s: &str) {
    write_varint(buf, s.len() as u64);
    buf.extend_from_slice(s.as_bytes());
}

/// Decode a string from a byte slice, returning (string, bytes_consumed).
/// Returns None if the slice is too short or the length is invalid.
fn read_string(data: &[u8]) -> Option<(String, usize)> {
    let (len, consumed) = read_varint(data)?;
    let len = len as usize;
    let start = consumed;
    let end = start + len;
    if end > data.len() {
        return None;
    }
    let s = std::str::from_utf8(&data[start..end]).ok()?.to_string();
    Some((s, end))
}

// ── Binary Encoding ───────────────────────────────────────────────

/// Encode a CompiledIR into binary wire format bytes.
///
/// # Encoding Layout
///
/// 1. **Header** (3 bytes):
///    - `[0xCC, 0x01]` — magic bytes
///    - `0x01` — schema version
///
/// 2. **String table**:
///    - count: varint — number of unique strings
///    - for each string: (length: varint, bytes: UTF-8)
///
/// 3. **Instructions**:
///    - count: varint — number of instructions
///    - for each instruction:
///      - opcode: u8 — index into opcode table (0-14)
///      - [variadic count: varint — only if opcode is variadic]
///      - operands: [varint]* — string table indices
pub fn encode(ir: &CompiledIR) -> Vec<u8> {
    // Build string table from instructions
    let table = StringTable::from_instructions(&ir.instructions);
    // Map each string to its string-table index for fast lookup
    let strings: Vec<String> = table.strings().to_vec();

    let mut buf = Vec::new();

    // 1. Header
    buf.extend_from_slice(&MAGIC);
    buf.push(VERSION);

    // 2. String table
    write_varint(&mut buf, strings.len() as u64);
    for s in &strings {
        write_string(&mut buf, s);
    }

    // 3. Instructions
    write_varint(&mut buf, ir.instructions.len() as u64);

    // Build a lookup: string → table index
    use std::collections::HashMap;
    let mut str_to_idx: HashMap<&str, u64> = HashMap::new();
    for (i, s) in strings.iter().enumerate() {
        str_to_idx.insert(s.as_str(), i as u64);
    }

    // Helper to encode an operand string as its table index varint
    let encode_operand = |buf: &mut Vec<u8>, s: &str| {
        let idx = str_to_idx.get(s).copied().unwrap_or(0);
        write_varint(buf, idx);
    };

    for op in &ir.instructions {
        let op_idx = op_to_index(op);
        buf.push(op_idx);

        match op {
            CoreOp::DefClass(_, name) => {
                encode_operand(&mut buf, name);
            }
            CoreOp::DefMethod(_, mid, name) => {
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, name);
            }
            CoreOp::DefField(_, fid, name) => {
                encode_operand(&mut buf, fid);
                encode_operand(&mut buf, name);
            }
            CoreOp::DefInterface(_, name) => {
                encode_operand(&mut buf, name);
            }
            CoreOp::Param(mid, pid, ty, name) => {
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, pid);
                encode_operand(&mut buf, ty);
                encode_operand(&mut buf, name);
            }
            CoreOp::Return(mid, ty) => {
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, ty);
            }
            CoreOp::FieldType(fid, ty) => {
                encode_operand(&mut buf, fid);
                encode_operand(&mut buf, ty);
            }
            CoreOp::Flags(tid, flags) => {
                // Variadic: write count prefix, then operands
                write_varint(&mut buf, (1 + flags.len()) as u64); // tid + flags
                encode_operand(&mut buf, tid);
                for f in flags {
                    encode_operand(&mut buf, f);
                }
            }
            CoreOp::ClassFlags(cid, flags) => {
                write_varint(&mut buf, (1 + flags.len()) as u64);
                encode_operand(&mut buf, cid);
                for f in flags {
                    encode_operand(&mut buf, f);
                }
            }
            CoreOp::Extends(_, parent) => {
                encode_operand(&mut buf, parent);
            }
            CoreOp::Implements(_, iid) => {
                encode_operand(&mut buf, iid);
            }
            CoreOp::Injects(cid, deps) => {
                write_varint(&mut buf, (1 + deps.len()) as u64);
                encode_operand(&mut buf, cid);
                for d in deps {
                    encode_operand(&mut buf, d);
                }
            }
            CoreOp::Import(_, module, named) => {
                encode_operand(&mut buf, module);
                encode_operand(&mut buf, named);
            }
            CoreOp::TypeAlias(_, original) => {
                encode_operand(&mut buf, original);
            }
            CoreOp::Pattern(name, args) => {
                write_varint(&mut buf, (1 + args.len()) as u64); // name + args
                encode_operand(&mut buf, name);
                for a in args {
                    encode_operand(&mut buf, a);
                }
            }
        }
    }

    buf
}

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

    // 2. String table
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
        let (idx, consumed) = read_varint(data).ok_or_else(|| {
            BinaryDecodeError::TruncatedData("operand index".into())
        })?;
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

        if op_idx > 14 {
            return Err(BinaryDecodeError::UnknownOpcode(op_idx));
        }

        let op = if is_variadic(op_idx) {
            // Read variadic count prefix
            let (var_count, consumed) = read_varint(&data[pos..]).ok_or_else(|| {
                BinaryDecodeError::TruncatedData("variadic operand count".into())
            })?;
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
                _ => return Err(BinaryDecodeError::UnknownOpcode(op_idx)),
            }
        };

        instructions.push(op);
    }

    // Note: The binary format doesn't encode file_id or version in the
    // same way as JSON. We extract what we can from the header.
    Ok(CompiledIR {
        file_id: "bin".to_string(),
        instructions,
        version: VERSION as u64,
    })
}

/// Estimate the byte savings of binary vs. positional JSON encoding.
/// Returns `(json_chars, binary_bytes)`.
///
/// The JSON count uses the named wire format (most verbose) as baseline.
pub fn estimate_savings(ir: &CompiledIR) -> (usize, usize) {
    // Named JSON encoding
    let named = super::wire::ir_to_wire(ir);
    let json_str = serde_json::to_string(&named).unwrap_or_default();

    // Binary encoding
    let binary = encode(ir);

    (json_str.len(), binary.len())
}

/// Detect whether a byte slice starts with the binary wire magic bytes.
pub fn is_binary_wire(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == MAGIC[0] && data[1] == MAGIC[1]
}

/// Convert a CompiledIR to binary wire format, returning a JSON-compatible
/// wrapper value that contains the base64-encoded binary data.
///
/// This is used for transport over JSON-based MCP channels where raw bytes
/// cannot be sent directly.
pub fn ir_to_binary_wire_json(ir: &CompiledIR) -> serde_json::Value {
    let bytes = encode(ir);
    let b64 = base64_encode(&bytes);
    serde_json::json!({
        "file": ir.file_id,
        "v": ir.version,
        "encoding": "binary",
        "data": b64,
    })
}

/// Decode a CompiledIR from the JSON wrapper containing base64-encoded
/// binary data.
///
/// # Errors
///
/// Returns None if:
/// - Required fields are missing
/// - Base64 decoding fails
/// - Binary decoding fails
pub fn binary_wire_json_to_ir(value: &serde_json::Value) -> Option<CompiledIR> {
    let data_str = value.get("data")?.as_str()?;
    let bytes = base64_decode(data_str)?;
    decode(&bytes).ok()
}

/// Minimal base64 encoder (RFC 4648). Avoids pulling in a dependency
/// for a simple encoding.
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Minimal base64 decoder (RFC 4648).
fn base64_decode(data: &str) -> Option<Vec<u8>> {
    // Build reverse lookup table
    let decode_char = |c: u8| -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a') as u32 + 26),
            b'0'..=b'9' => Some((c - b'0') as u32 + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };

    let bytes: Vec<u8> = data.bytes().collect();
    let mut result = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits_collected = 0;

    for &b in &bytes {
        if b == b'=' {
            // Padding: handle remaining bits
            break;
        }
        let value = decode_char(b)?;
        buffer = (buffer << 6) | value;
        bits_collected += 6;
        if bits_collected >= 8 {
            bits_collected -= 8;
            result.push((buffer >> bits_collected) as u8);
            buffer &= (1 << bits_collected) - 1;
        }
    }

    Some(result)
}

#[cfg(test)]
#[path = "../tests/ir/binary_wire.rs"]
mod tests;