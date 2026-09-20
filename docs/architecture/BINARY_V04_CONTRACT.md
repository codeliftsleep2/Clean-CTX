# Corrected Binary Format Contract — Physical Version `0x04`

**Status:** Approved Phase 8 target. Phase 8A fixed the normative contract;
Phase 8B codec and tracked coverage are implemented, awaiting verification.

**Date:** 2026-09-20

**Companion documents:**

- [`CORE_OP_CONTRACT_MATRIX.md`](CORE_OP_CONTRACT_MATRIX.md)
- [`IR_ARCHITECTURE_LOCKDOWN_PLAN.md`](IR_ARCHITECTURE_LOCKDOWN_PLAN.md)
- [`IR_PROJECTION_RECONNAISSANCE_REPORT.md`](IR_PROJECTION_RECONNAISSANCE_REPORT.md)

## 1. Governing invariant

For every valid supported `CompiledIR` value:

```text
decode_v04(encode_v04(value)) == value
```

Equality includes:

- `file_id`;
- IR edit-sequence `version`;
- complete `CoreOp` variant and operands;
- instruction order;
- every repeated and exactly duplicated occurrence;
- body text and optional byte spans;
- typed semantic-family values;
- call argument count and spread qualification.

Compression never authorizes semantic loss. A value that cannot be represented
must fail structurally rather than decode with an empty, inferred, or synthetic
operand.

## 2. Scope

Phase 8 changes the canonical binary serialization only. It does not change:

- the positional, named, tagged, string-table, hierarchical, or LLM-facing
  representations;
- the Phase 7 delta protocol version `dv: 2`;
- `CoreOp` identity, cardinality, or producer contracts;
- projection behavior;
- source-language capture;
- byte-exact `apply_edit` body content or span meaning.

The physical binary version is `0x04`. Calling this format “binary V2” is
incorrect because it can be confused with physical version `0x02`.

## 3. Current `0x03` inventory

### 3.1 Preserved container data

Physical `0x03` preserves the IR edit-sequence version, instruction count, and
instruction order. Raw decoding replaces `file_id` with the literal `"bin"`.
The JSON wrapper carries `file` and `v`, but its decoder currently ignores both
fields rather than restoring or validating them.

### 3.2 Identity loss

The following `0x03` layouts omit canonical identity operands and the decoder
fabricates empty strings:

| Operation | Preserved by `0x03` | Lost by `0x03` |
|---|---|---|
| `DefClass(class, name)` | `name` | `class` |
| `DefMethod(class, method, name)` | `method`, `name` | owner `class` |
| `DefField(class, field, name)` | `field`, `name` | owner `class` |
| `DefInterface(interface, name)` | `name` | `interface` |
| `Extends(child, parent)` | `parent` | `child` |
| `Implements(class, interface)` | `interface` | `class` |
| `Import(alias, module, named)` | `module`, `named` | `alias` |
| `TypeAlias(alias, original)` | `original` | `alias` |

Every other current `CoreOp` variant carries its present canonical operands in
`0x03`. That local preservation does not make the complete `0x03` stream
semantic because container identity and the eight layouts above remain lossy.

### 3.3 Decoder strictness gaps

The current decoder also accepts non-canonical or ambiguous input in several
places:

- a `BODY` span flag other than `1` is treated as absent rather than rejected;
- trailing bytes after the declared instruction stream are ignored;
- raw call argument counts are cast to `usize` without an explicit overflow
  error;
- the public raw decoder always synthesizes `file_id = "bin"`;
- the JSON wrapper does not verify its `file` and `v` metadata against the
  payload.

These are corrected-format requirements, not authority for `0x04` behavior.

## 4. Compatibility evidence and policy

Source inspection found no tracked `.bin`, database, fixture, or other
persisted binary artifact. The production `compress_code_context` and Angular
provider persistence paths currently pass an empty IR blob to `BufferedStore`.
SQLite can store and decode a supplied blob, but no inspected production writer
supplies `binary_wire::encode` output.

The current decoder contains branches labelled `0x01`, `0x02`, and `0x03`.
That code is not evidence of a supported persisted input:

- the documented original V1 magic is `0xCC, 0x01`;
- the current decoder requires magic `0xCC, 0x02` for every accepted version;
- no tracked test fixture exercises a real historical artifact;
- existing tests construct current output or assert deliberately lossy
  structural comparisons.

Therefore the corrected decoder supports physical `0x04`. It does not preserve
a silent compatibility route that fabricates missing semantic
identity for `0x01`–`0x03`. Those versions return a structured unsupported-
version error. A future compatibility reader requires concrete artifact
evidence and separate architectural approval.

## 5. Physical `0x04` layout

The magic remains `0xCC, 0x02`. The next byte is physical version `0x04`.

```text
magic                  [0xCC, 0x02]
physical_version       u8 = 0x04
ir_version             unsigned varint
string_count           unsigned varint
strings                repeated (byte_length varint, strict UTF-8 bytes)
file_id_index           unsigned varint into string table
instruction_count      unsigned varint
instructions           exactly instruction_count encoded operations
EOF                     required immediately after final instruction
```

The string table contains `file_id` and every string operand used by every
instruction. First-occurrence interning and canonical instruction traversal
remain deterministic.

Integer values that are semantic integers remain raw unsigned varints:

- `Body.start_byte` and `Body.end_byte`;
- `Call.explicit_arg_count`.

## 6. Opcode and operand contract

Opcode indices remain `0` through `25`; a version bump changes layouts, not
semantic operation assignment.

| Opcode | `CoreOp` | `0x04` operands in order |
|---:|---|---|
| 0 | `DefClass` | `class`, `name` |
| 1 | `DefMethod` | `class`, `method`, `name` |
| 2 | `DefField` | `class`, `field`, `name` |
| 3 | `DefInterface` | `interface`, `name` |
| 4 | `Param` | `method`, `parameter`, `type`, `name` |
| 5 | `Return` | `method`, `type` |
| 6 | `FieldType` | `field`, `type` |
| 7 | `Flags` | count, `method`, ordered values |
| 8 | `ClassFlags` | count, `class`, ordered values |
| 9 | `Extends` | `child`, `parent` |
| 10 | `Implements` | `class`, `interface` |
| 11 | `Injects` | count, `class`, ordered dependencies |
| 12 | `Import` | `alias`, `module`, `named` |
| 13 | `TypeAlias` | `alias`, `original` |
| 14 | `Pattern` | count, `name`, ordered arguments |
| 15 | `DataFlow` | `method`, `direction`, `target` |
| 16 | `ControlFlow` | `method`, `kind`, `target` |
| 17 | `SideEffect` | `method`, typed effect value |
| 18 | `ExecutionContext` | `method`, typed context value |
| 19 | `Body` | `method`, text, span flag, optional start/end |
| 20 | exact `Call` | `caller`, `callee`, raw argument count |
| 21 | spread `Call` | `caller`, `callee`, raw argument count |
| 22 | `MethodModifiers` | count, `method`, ordered typed values |
| 23 | `ClassModifiers` | count, `class`, ordered typed values |
| 24 | `ControlSummary` | count, `method`, ordered typed values |
| 25 | `PatternFacts` | count, `method`, complete serialized facts |

`count` is the number of following string-table operands for that instruction.
It is not a deduplication boundary. Instruction occurrences and payload
duplicates remain byte-represented in their original order.

`Body` span flag is exactly `0` or `1`. Flag `0` carries no offsets. Flag `1`
carries both start and end offsets. Every other value fails decoding.

## 7. Structural failure contract

`0x04` decoding fails without returning partial IR for:

- bad magic or any physical version other than `0x04`;
- truncated or overflowing varints;
- invalid UTF-8;
- invalid string-table indexes;
- unknown opcodes;
- invalid fixed or variadic operand counts;
- invalid typed modifier, summary, pattern-fact, side-effect, or execution-
  context values;
- invalid `Body` span flags or incomplete span pairs;
- call argument counts that cannot fit `usize`;
- trailing bytes;
- JSON-wrapper `file` or `v` metadata that conflicts with the decoded payload.

The decoder is transactional: it returns one complete `CompiledIR` or one
structured error.

## 8. Phase 8B executable acceptance

The implementation slice must land its assertions with the repair that makes
them green. Required tracked coverage includes:

1. exact `CompiledIR` equality across all `CoreOp` variants;
2. exact class, method, field, interface, import, and type-alias identities;
3. repeated identical operations and repeated payload values;
4. instruction-order preservation;
5. `file_id` and IR-version preservation;
6. span-less and spanned bodies with byte-exact text;
7. exact and spread calls;
8. deterministic bytes;
9. JSON-wrapper semantic equality and metadata-conflict rejection;
10. structured rejection of legacy versions and every malformed case in
    Section 7;
11. a production-path persistence test once a real writer is wired to store
    non-empty `0x04` bytes.

Opcode-only, instruction-count-only, and selective-field comparisons are not
semantic round-trip evidence.

## 9. Production integration boundary

Phase 8B corrects the codec. It is not production-complete merely because its
codec tests pass. The real persistence lifecycle currently has a missing
producer: registered compression handlers queue empty IR blobs even though
SQLite owns an `ir_binary` column and `replay_history` attempts to decode it.

Wiring a non-empty `0x04` blob into that lifecycle must preserve the existing
LLM-facing representation and byte-exact edit behavior. It must be verified
through producer, queued persistence, SQLite ownership, reload, delta replay,
and MCP exposure before Phase 9 can certify production integration.
