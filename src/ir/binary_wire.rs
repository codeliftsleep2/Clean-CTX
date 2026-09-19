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
// │ String Table: [count(varint), (len(varint), bytes)*]    │
// │ Instructions: [count(varint), instruction*]             │
// │                                                         │
// │ Instruction:                                            │
// │   opcode_idx: u8 (0-25)                                 │
// │   operands: [varint]* (string table indices)            │
// │   For variadic ops: operand_count as varint prefix      │
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
use super::opcodes::{ControlSummary, CoreOp, DeclarationModifier, PatternFact};
use super::string_table::StringTable;

mod decode;

// Re-exported so the established public paths
// (`crate::ir::binary_wire::decode`, `::BinaryDecodeError`) are unchanged by
// the decode-side split.
pub use decode::BinaryDecodeError;
pub use decode::decode;

/// Magic bytes for the binary wire format: "CC" + version marker
const MAGIC: [u8; 2] = [0xCC, 0x02];
/// Binary wire format version:
/// 0x01 = Original (long TYPE op names like "NG_COMPONENT_Foo")
/// 0x02 = Abbreviated (@-prefixed TYPE ops like "@cmp")
/// 0x03 = Body spans (apply_edit plan Phase 1): OP_BODY gains a presence
///        flag varint plus two raw byte-offset varints when a body span is
///        present. All other opcodes are encoded identically to 0x02.
const VERSION: u8 = 0x03;
/// Version 0x02 (pre-span bodies) — still supported for decode; OP_BODY
/// carries exactly two string-table operands and decodes span-less.
const VERSION_PRE_SPAN: u8 = 0x02;
/// Legacy version 0x01 (long TYPE op names) — still supported for decode.
const VERSION_LEGACY: u8 = 0x01;

/// Opcode index assignment (0-25)
const OP_DEF_C: u8 = 0;
const OP_DEF_M: u8 = 1;
const OP_DEF_F: u8 = 2;
const OP_DEF_I: u8 = 3;
const OP_SIG: u8 = 4; // Param
const OP_RET: u8 = 5; // Return
const OP_FIELD_T: u8 = 6; // FieldType
const OP_FLAGS: u8 = 7;
const OP_FLAGS_C: u8 = 8; // ClassFlags
const OP_EXT: u8 = 9; // Extends
const OP_IMPL: u8 = 10; // Implements
const OP_INJECTS: u8 = 11;
const OP_IMP: u8 = 12; // Import
const OP_TYPE: u8 = 13; // TypeAlias
const OP_PAT: u8 = 14; // Pattern
// R-43a: Execution Semantics
const OP_DATAFLOW: u8 = 15;
const OP_CTRL: u8 = 16;
const OP_EFFECT: u8 = 17;
const OP_CTX: u8 = 18;
// Edit Mode: Verbatim Method Bodies
const OP_BODY: u8 = 19;
// Structural invocations (native call graph). Additive opcode under the
// existing 0x03 scheme: a reader that predates it fails loudly with
// `UnknownOpcode(20)` rather than mis-decoding an invocation fact, so no
// version-byte change is required and previously persisted 0x03 streams
// keep their established interpretation.
const OP_CALL: u8 = 20;
// Structural invocation whose written argument count is NOT exact: at least
// one written argument expands at run time (`foo(...args)`). It carries the
// same operand layout as `OP_CALL`, and it exists as a separate opcode — not
// an extra operand on `OP_CALL` — precisely so a reader that predates the
// qualifier fails loudly with `UnknownOpcode(21)` instead of decoding a
// spread call as an exact one. An exact call keeps its byte-identical
// `OP_CALL` encoding.
//
// The shared string table is derived from the canonical tuple for every
// transport, so it also interns the qualifier string; no binary operand
// references that entry, which is why the two opcodes still decouple cleanly
// even though a qualified stream is not length-identical to an exact one.
const OP_CALL_SPREAD: u8 = 21;
// Phase 6A declaration-modifier families. These are additive under physical
// 0x03, matching CALL's fail-loud forward-compatibility policy. Physical 0x04
// remains reserved for the complete corrected-format migration.
const OP_MOD_M: u8 = 22;
const OP_MOD_C: u8 = 23;
const OP_CTRL_SUM: u8 = 24;
const OP_PAT_FACT: u8 = 25;

/// Highest defined opcode index.
///
/// The decoder's forward-compatibility guard rejects only indices ABOVE this
/// value, so every opcode this build defines decodes. `OP_CALL` (and the
/// additive `OP_CALL_SPREAD`) are allocated after the edit-mode `OP_BODY`, so
/// a guard bounded at `OP_BODY` would report a defined opcode as unknown and
/// make CALL facts unrepresentable over the binary wire.
const OP_MAX: u8 = OP_PAT_FACT;

/// Opcodes that have a variable number of operands (beyond the first one).
fn is_variadic(op_idx: u8) -> bool {
    matches!(
        op_idx,
        OP_FLAGS
            | OP_FLAGS_C
            | OP_INJECTS
            | OP_PAT
            | OP_MOD_M
            | OP_MOD_C
            | OP_CTRL_SUM
            | OP_PAT_FACT
    )
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
        CoreOp::MethodModifiers(..) => OP_MOD_M,
        CoreOp::ClassModifiers(..) => OP_MOD_C,
        CoreOp::ControlSummary(..) => OP_CTRL_SUM,
        CoreOp::PatternFacts(..) => OP_PAT_FACT,
        CoreOp::Flags(..) => OP_FLAGS,
        CoreOp::ClassFlags(..) => OP_FLAGS_C,
        CoreOp::Extends(..) => OP_EXT,
        CoreOp::Implements(..) => OP_IMPL,
        CoreOp::Injects(..) => OP_INJECTS,
        CoreOp::Import(..) => OP_IMP,
        CoreOp::TypeAlias(..) => OP_TYPE,
        CoreOp::Pattern(..) => OP_PAT,
        // Edit Mode: Verbatim Method Bodies
        CoreOp::Body(..) => OP_BODY,
        // R-43a: Execution Semantics
        CoreOp::DataFlow(..) => OP_DATAFLOW,
        CoreOp::ControlFlow(..) => OP_CTRL,
        CoreOp::SideEffect(..) => OP_EFFECT,
        CoreOp::ExecutionContext(..) => OP_CTX,
        // Structural invocations (native call graph). The spread qualifier
        // selects the additive opcode so a predating reader cannot decode a
        // spread call as an exact one.
        CoreOp::Call(_, _, _, false) => OP_CALL,
        CoreOp::Call(_, _, _, true) => OP_CALL_SPREAD,
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
///      - opcode: u8 — index into opcode table (0-25)
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

    // 2. IR version (edit sequence number)
    write_varint(&mut buf, ir.version);

    // 3. String table
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
            CoreOp::MethodModifiers(mid, modifiers) => {
                write_varint(&mut buf, (1 + modifiers.len()) as u64);
                encode_operand(&mut buf, mid);
                for modifier in modifiers {
                    encode_operand(&mut buf, modifier.as_str());
                }
            }
            CoreOp::ClassModifiers(cid, modifiers) => {
                write_varint(&mut buf, (1 + modifiers.len()) as u64);
                encode_operand(&mut buf, cid);
                for modifier in modifiers {
                    encode_operand(&mut buf, modifier.as_str());
                }
            }
            CoreOp::ControlSummary(mid, summaries) => {
                write_varint(&mut buf, (1 + summaries.len()) as u64);
                encode_operand(&mut buf, mid);
                for summary in summaries {
                    encode_operand(&mut buf, summary.as_str());
                }
            }
            CoreOp::PatternFacts(mid, facts) => {
                let mut operands = Vec::new();
                for fact in facts {
                    fact.append_serialized(&mut operands);
                }
                write_varint(&mut buf, (1 + operands.len()) as u64);
                encode_operand(&mut buf, mid);
                for operand in operands {
                    encode_operand(&mut buf, &operand);
                }
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
            // Edit Mode: Verbatim Method Bodies
            // v0x03+: [mid_idx, text_idx, has_span_flag, start?, end?].
            // Spans are raw varints (not string-table entries) so repeated
            // offsets don't pollute the table. Span-less bodies (legacy
            // decoded state) write flag=0 and no offsets, keeping the
            // encoding self-describing without a schema change elsewhere.
            CoreOp::Body(mid, text, start, end) => {
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, text);
                match (start, end) {
                    (Some(s), Some(e)) => {
                        write_varint(&mut buf, 1);
                        write_varint(&mut buf, *s);
                        write_varint(&mut buf, *e);
                    }
                    _ => {
                        write_varint(&mut buf, 0);
                    }
                }
            }
            // R-43a: Execution Semantics
            CoreOp::DataFlow(mid, direction, target) => {
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, direction);
                encode_operand(&mut buf, target);
            }
            CoreOp::ControlFlow(mid, kind, target) => {
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, kind);
                encode_operand(&mut buf, target);
            }
            CoreOp::SideEffect(mid, effect_type) => {
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, effect_type);
            }
            CoreOp::ExecutionContext(mid, context_type) => {
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, context_type);
            }
            // Structural invocations (native call graph).
            // [caller_idx, callee_idx, argc_varint] — the explicit argument
            // count is a raw varint (like BODY spans) rather than a string
            // table entry, so repeated small counts never pollute the table.
            // The spread qualifier is carried by the OPCODE (see
            // `op_to_index`), so both shapes share this exact layout.
            CoreOp::Call(caller, callee, argc, _) => {
                encode_operand(&mut buf, caller);
                encode_operand(&mut buf, callee);
                write_varint(&mut buf, *argc as u64);
            }
        }
    }

    buf
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
