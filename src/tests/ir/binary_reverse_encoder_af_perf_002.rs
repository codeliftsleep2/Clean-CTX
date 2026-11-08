use super::table::StringTable;
use crate::ir::CompiledIR;
use crate::ir::CoreOp;

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Rebuild,
    Reuse,
}

/// Magic bytes for the binary wire format: "CC" + version marker
const MAGIC: [u8; 2] = [0xCC, 0x02];
/// Binary wire format version:
/// 0x04 = complete CompiledIR identity and semantic operands.
const VERSION: u8 = 0x04;

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
// Structural invocations (native call graph). Exact and spread calls retain
// distinct opcodes in the corrected format.
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
// Typed semantic families retain their established opcode assignments.
const OP_MOD_M: u8 = 22;
const OP_MOD_C: u8 = 23;
const OP_CTRL_SUM: u8 = 24;
const OP_PAT_FACT: u8 = 25;
const OP_DEF_IM: u8 = 26;
const OP_DEF_IF: u8 = 27;
const OP_MOD_I: u8 = 28;
const OP_EXT_I: u8 = 29;
// Neutral C# first-base-list reference whose target kind is unresolved.
const OP_BASE_REF: u8 = 30;

/// Convert a CoreOp to its u8 opcode index.
fn op_to_index(op: &CoreOp) -> u8 {
    match op {
        CoreOp::DefClass(..) => OP_DEF_C,
        CoreOp::DefMethod(..) => OP_DEF_M,
        CoreOp::DefField(..) => OP_DEF_F,
        CoreOp::DefInterface(..) => OP_DEF_I,
        CoreOp::DefInterfaceMethod(..) => OP_DEF_IM,
        CoreOp::DefInterfaceField(..) => OP_DEF_IF,
        CoreOp::Param(..) => OP_SIG,
        CoreOp::Return(..) => OP_RET,
        CoreOp::FieldType(..) => OP_FIELD_T,
        CoreOp::MethodModifiers(..) => OP_MOD_M,
        CoreOp::ClassModifiers(..) => OP_MOD_C,
        CoreOp::InterfaceModifiers(..) => OP_MOD_I,
        CoreOp::ControlSummary(..) => OP_CTRL_SUM,
        CoreOp::PatternFacts(..) => OP_PAT_FACT,
        CoreOp::Flags(..) => OP_FLAGS,
        CoreOp::ClassFlags(..) => OP_FLAGS_C,
        CoreOp::Extends(..) => OP_EXT,
        CoreOp::BaseTypeRef(..) => OP_BASE_REF,
        CoreOp::InterfaceExtends(..) => OP_EXT_I,
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
/// Encode a string as (length varint, UTF-8 bytes).
fn write_string(buf: &mut Vec<u8>, s: &str) {
    write_varint(buf, s.len() as u64);
    buf.extend_from_slice(s.as_bytes());
}

// ── Binary Encoding ───────────────────────────────────────────────

/// Encode a CompiledIR into binary wire format bytes.
///
/// # Encoding Layout
///
/// 1. **Header** (3 bytes):
///    - `[0xCC, 0x02]` — magic bytes
///    - `0x04` — schema version
///
/// 2. **String table**:
///    - count: varint — number of unique strings
///    - for each string: (length: varint, bytes: UTF-8)
///
/// 3. **File identity**: string-table index varint
///
/// 4. **Instructions**:
///    - count: varint — number of instructions
///    - for each instruction:
///      - opcode: u8 — index into opcode table (0-25)
///      - [variadic count: varint — only if opcode is variadic]
///      - operands: [varint]* — string table indices
pub(super) fn encode(ir: &CompiledIR, mode: Mode) -> Vec<u8> {
    // Build string table from instructions
    let mut table = StringTable::from_instructions(&ir.instructions);
    table.intern(&ir.file_id);
    // Map each string to its string-table index for fast lookup
    let strings = table.strings();

    let mut buf = Vec::new();

    // 1. Header
    buf.extend_from_slice(&MAGIC);
    buf.push(VERSION);

    // 2. IR version (edit sequence number)
    write_varint(&mut buf, ir.version);

    // 3. String table
    write_varint(&mut buf, strings.len() as u64);
    for s in strings {
        write_string(&mut buf, s);
    }

    let capacities = table.capacities();
    super::observe(|w| {
        w.table_strings = table.len();
        w.string_capacity = capacities.0;
        w.table_capacity = capacities.1;
        w.common_container_header_bytes = std::mem::size_of::<StringTable>();
    });
    let rebuilt = if matches!(mode, Mode::Rebuild) {
        let mut map = std::collections::HashMap::new();
        for (i, s) in strings.iter().enumerate() {
            super::observe(|w| {
                w.reverse_entries += 1;
                w.reverse_key_bytes += s.len();
            });
            map.insert(s.as_str(), i as u64);
        }
        super::observe(|w| {
            w.reverse_capacity = map.capacity();
            w.extra_map_header_bytes = std::mem::size_of_val(&map);
        });
        Some(map)
    } else {
        None
    };
    let encode_operand = |buf: &mut Vec<u8>, s: &str| {
        super::observe(|w| {
            w.lookups += 1;
            w.lookup_key_bytes += s.len();
        });
        let idx = match &rebuilt {
            Some(map) => map.get(s).copied(),
            None => table.index_of(s).map(|i| i as u64),
        }
        .expect("canonical binary operand missing from string table");
        write_varint(buf, idx);
    };
    // 4. File identity and instructions
    encode_operand(&mut buf, &ir.file_id);
    write_varint(&mut buf, ir.instructions.len() as u64);

    for op in &ir.instructions {
        let op_idx = op_to_index(op);
        buf.push(op_idx);

        match op {
            CoreOp::DefClass(cid, name) => {
                encode_operand(&mut buf, cid);
                encode_operand(&mut buf, name);
            }
            CoreOp::DefMethod(cid, mid, name) => {
                encode_operand(&mut buf, cid);
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, name);
            }
            CoreOp::DefField(cid, fid, name) => {
                encode_operand(&mut buf, cid);
                encode_operand(&mut buf, fid);
                encode_operand(&mut buf, name);
            }
            CoreOp::DefInterface(iid, name) => {
                encode_operand(&mut buf, iid);
                encode_operand(&mut buf, name);
            }
            CoreOp::DefInterfaceMethod(iid, mid, name) => {
                encode_operand(&mut buf, iid);
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, name);
            }
            CoreOp::DefInterfaceField(iid, fid, name) => {
                encode_operand(&mut buf, iid);
                encode_operand(&mut buf, fid);
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
            CoreOp::InterfaceModifiers(iid, modifiers) => {
                write_varint(&mut buf, (1 + modifiers.len()) as u64);
                encode_operand(&mut buf, iid);
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
            CoreOp::Extends(child, parent) => {
                encode_operand(&mut buf, child);
                encode_operand(&mut buf, parent);
            }
            CoreOp::BaseTypeRef(child, written_type) => {
                encode_operand(&mut buf, child);
                encode_operand(&mut buf, written_type);
            }
            CoreOp::InterfaceExtends(child, parent) => {
                encode_operand(&mut buf, child);
                encode_operand(&mut buf, parent);
            }
            CoreOp::Implements(cid, iid) => {
                encode_operand(&mut buf, cid);
                encode_operand(&mut buf, iid);
            }
            CoreOp::Injects(cid, deps) => {
                write_varint(&mut buf, (1 + deps.len()) as u64);
                encode_operand(&mut buf, cid);
                for d in deps {
                    encode_operand(&mut buf, d);
                }
            }
            CoreOp::Import(alias, module, named) => {
                encode_operand(&mut buf, alias);
                encode_operand(&mut buf, module);
                encode_operand(&mut buf, named);
            }
            CoreOp::TypeAlias(alias, original) => {
                encode_operand(&mut buf, alias);
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
            // v0x04: [mid_idx, text_idx, has_span_flag, start?, end?].
            // Spans are raw varints (not string-table entries) so repeated
            // offsets don't pollute the table. Span-less bodies write flag=0
            // and no offsets.
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
                encode_operand(&mut buf, effect_type.as_str());
            }
            CoreOp::ExecutionContext(mid, context_type) => {
                encode_operand(&mut buf, mid);
                encode_operand(&mut buf, context_type.as_str());
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
    super::observe(|w| w.output_bytes = buf.len());
    buf
}

pub(super) fn missing_operand(mode: Mode) {
    let table = StringTable::new();
    let map: std::collections::HashMap<&str, u64> = std::collections::HashMap::new();
    let value = match mode {
        Mode::Rebuild => map.get("absent").copied(),
        Mode::Reuse => table.index_of("absent").map(|i| i as u64),
    };
    let _ = value.expect("canonical binary operand missing from string table");
}
