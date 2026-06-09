# Clean-CTX — Ultra-Compact IR & Ultra-Compressed Text: Plan

**Version:** 0.2.0 (Phase I Complete)
**Created:** 2026-06-09
**Updated:** 2026-06-09
**Status:** Phase I Complete — Phase II Next

> **Living document.** This plan defines the next evolution of both the Compiler IR and text compression subsystems, targeting maximum token savings without any loss of correctness. Every proposal preserves 100% information fidelity — the AI receives identical semantic information through a more compact encoding.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Current State Analysis](#2-current-state-analysis)
3. [Bottleneck Analysis](#3-bottleneck-analysis)
4. [Phase I: Ultra-Compact IR — Quick Wins](#4-phase-i-ultra-compact-ir--quick-wins)
5. [Phase II: Ultra-Compact IR — Structural](#5-phase-ii-ultra-compact-ir--structural)
6. [Phase III: Ultra-Compressed Text](#6-phase-iii-ultra-compressed-text)
7. [Phase IV: Text Delta Support](#7-phase-iv-text-delta-support)
8. [Expected Savings Summary](#8-expected-savings-summary)
9. [Correctness Guarantees](#9-correctness-guarantees)
10. [Implementation Timeline](#10-implementation-timeline)

---

## 1. Executive Summary

The existing Clean-CTX compression systems — both the text pipeline and the Compiler IR — are well-designed and already deliver significant token savings. This plan identifies **12 specific improvement ideas** that can be stacked on top of the current system to achieve additional savings of:

- **Full IR compression**: 40-60% smaller than current positional encoding
- **Delta transport**: 30-50% smaller deltas via field-level diffing
- **Text compression**: 20-35% smaller output via structural deduplication
- **Text edit sessions**: 70-90% savings via delta support (matching IR delta performance)

The improvements are organized into 4 phases, ordered by impact × feasibility, with zero correctness regressions.

---

## 2. Current State Analysis

### 2.1 Text Compression Pipeline

```
Source → Tree-sitter AST → Fidelity Filter → Opcode Encode → Symbol Dict → Text Output
```

**Key characteristics:**
- 32 primitive opcodes ($e=$export, $c=$class, $a=$async, etc.)
- Symbol dictionary replaces frequent tokens with $1, $2, ... at Low fidelity
- Three fidelity levels: Low (compact), Medium (signatures), High (indented)
- Stateless: full re-compress on every invocation
- Path dictionary for file path aliases (α1, α2, ...)

### 2.2 Compiler IR System

```
Source → Tree-sitter AST → IR Compiler (4 layers) → CoreOp instructions → Wire Format
```

**Key characteristics:**
- 15 CoreOp types: DefClass, DefMethod, DefField, DefInterface, Param, Return, FieldType, Flags, ClassFlags, Extends, Implements, Injects, Import, TypeAlias, Pattern
- Wire format: JSON positional arrays
- Three wire encodings: named (opcode included), positional (opcode stripped), tagged (debug)
- Delta transport: instruction-level diffing with adds/mods/dels
- Pattern compression: CTOR, OBSERVABLE, PROMISE, GETTER, SETTER, OVERRIDE patterns consumed into single PAT ops
- State replay with version tracking

### 2.3 Current Token Savings

| Scenario | Text Low | IR Full | IR Delta | IR + Phase H |
|---|---|---|---|---|
| First compression (32-line file) | 26 tok | ~20 tok | N/A | ~14 tok |
| First compression (438-line file) | 75 tok | ~55 tok | N/A | ~38 tok |
| Single method edit | 75 tok (full) | 75 tok (full) | ~8-12 tok | ~8-12 tok |
| Single line edit | 75 tok | 75 tok | ~5-8 tok | ~5-8 tok |
| 50-edit session (cumulative) | 3,750 tok | 3,750 tok | ~200 tok | ~200 tok |

---

## 3. Bottleneck Analysis

### 3.1 IR System Bottlenecks

| Bottleneck | Waste Source | Impact |
|---|---|---|
| **JSON syntax overhead** | Quotes, commas, brackets in every array: `["DEF_M","C1","M1","name"]` | ~40% of wire bytes |
| **String repetition** | "C1" appears 10+ times per class; "M1" appears 3-4 times per method | ~20% of wire bytes |
| **Delta ModOp redundancy** | Key is prefix of replacement: `{"k":["DEF_M","C1","M1"],"r":["DEF_M","C1","M1","new"]}` | ~30% of ModOp bytes |
| **Delta envelope overhead** | `{"file":"α1","from":5,"to":6,"ops":{...}}` is ~60-80 chars even for 1-op deltas | ~50% of small deltas |
| **Flat instruction array** | No structural hierarchy; parent IDs repeated in every child instruction | ~15% of wire bytes |

### 3.2 Text Compression Bottlenecks

| Bottleneck | Waste Source | Impact |
|---|---|---|
| **Symbol dictionary footer** | `// $1=Service; $2=Observable; ...` grows with unique token count | ~10-15 tokens/file |
| **Report header** | `// --- Token Optimization Report --- \n// Raw Tokens: X...` | ~30 tokens/file |
| **No delta support** | Full re-compress on every change | 100% waste on edits |
| **No structural dedup** | Repeated type annotations, parameter patterns not consolidated | ~15-20% |
| **Import verbosity** | Module paths retained even at Low fidelity | ~5-10% |

---

## 4. Phase I: Ultra-Compact IR — Quick Wins

**Goal:** Maximum savings with minimal schema changes. These improvements modify the wire serialization without changing the CoreOp instruction set.

### 4.1 Idea #2: String Table + Relative Referencing

**Wire format change:** Build a per-file string table, then all instructions reference strings by table index.

**Current wire:**
```json
{
  "file": "α1", "v": 1, "encoding": "positional",
  "ir": [
    ["C1", "SampleService"],
    ["C1", "M1", "processComplexData"],
    ["M1", "P1", "$s", "payload"],
    ["M1", "$b"],
    ["C1", "IF"]
  ]
}
```

**Proposed wire:**
```json
{
  "file": "α1", "v": 1, "encoding": "string_table",
  "t": ["C1", "SampleService", "M1", "processComplexData", "P1", "$s", "payload", "$b", "IF"],
  "ir": [
    [0, 1],
    [0, 2, 3],
    [2, 4, 5, 6],
    [2, 7],
    [0, 8]
  ]
}
```

**How it works:**
1. Collect all unique strings from all instructions
2. Build an ordered table (most frequent first for potential BPE benefit)
3. Replace each string with its table index (integer)
4. Send table once, then integer arrays for instructions

**Estimated savings:** 25-40% reduction on top of positional encoding.

**Files to create/modify:**
- `src/ir/string_table.rs` — `StringTable` builder and lookup
- `src/ir/wire.rs` — add `ir_to_string_table_wire()` encoder
- `src/ir/replay.rs` — add `wire_to_ir_string_table()` decoder
- `src/tests/ir/string_table.rs` — round-trip tests

### 4.2 Idea #3: Delta Field-Level Diffing

**Wire format change:** ModOps encode only the changed field, not the full replacement.

**Current ModOp:**
```json
{"k": ["DEF_M", "C1", "M1"], "r": ["DEF_M", "C1", "M1", "renamedMethod"]}
```
= 85 characters

**Proposed ModOp:**
```json
{"k": ["DEF_M", "C1", "M1"], "d": [[3, "renamedMethod"]]}
```
= 55 characters (35% smaller)

**How it works:**
1. `k` = key tuple (unchanged — identifies the instruction)
2. `d` = array of `[field_index, new_value]` pairs
3. Field index is the positional index in the tuple where the change occurred
4. For multi-field changes, multiple `[idx, val]` pairs are included
5. Client applies patches: `tuple[field_index] = new_value`

**Estimated savings:** 30-50% reduction in delta ModOp size.

**Files to create/modify:**
- `src/ir/delta.rs` — add `FieldPatch` struct, modify `ModOp` to support both formats
- `src/ir/delta.rs` — add `compute_field_patches()` function
- `src/ir/replay.rs` — add field-patch application logic
- `src/tests/ir/delta.rs` — field-patch round-trip tests

### 4.3 Idea #6: Contextual Delta Encoding

**Wire format change:** Compact delta format with abbreviated keys and positional args.

**Current delta envelope:**
```json
{
  "file": "α1",
  "from": 5,
  "to": 6,
  "ops": {
    "+": [["DEF_M", "C1", "M3", "newMethod"]],
    "~": [{"k": ["DEF_M", "C1", "M1"], "r": ["DEF_M", "C1", "M1", "renamed"]}],
    "-": [["DEF_M", "C1", "M4"]]
  }
}
```

**Proposed compact delta:**
```json
{
  "f": "α1", "5→6",
  "+": [["M", "C1", "M3", "newMethod"]],
  "~": [["C1:M1", 3, "renamed"]],
  "-": [["M", "C1", "M4"]]
}
```

**How it works:**
1. Top-level keys abbreviated: `file`→`f`, `from`/`to`→`5→6`
2. Instruction tuples use abbreviated opcode where unambiguous
3. ModOps use field-patch format from Idea #3
4. Version encoded as range string `"5→6"` (saves the `from`/`to` key overhead)

**Estimated savings:** 40-60% reduction in delta envelope size.

**Files to create/modify:**
- `src/ir/delta.rs` — add `compact_encode()` / `compact_decode()` methods
- `src/ir/delta.rs` — add `CompactDelta` struct
- `src/tests/ir/delta.rs` — compact delta round-trip tests

---

## 5. Phase II: Ultra-Compact IR — Structural

**Goal:** Maximum savings via structural wire format changes. These require new encoder/decoder pairs but preserve the same CoreOp semantics.

### 5.1 Idea #4: Scoped Hierarchical IR

**Wire format change:** Replace flat instruction array with class→method→param tree.

**Current flat:**
```json
{
  "ir": [
    ["C1", "SampleService"],
    ["C1", "M1", "processComplexData"],
    ["M1", "P1", "$s", "payload"],
    ["M1", "$b"],
    ["C1", "IF"],
    ["C1", "M2", "doWork"],
    ["M2", "$v"]
  ]
}
```

**Proposed hierarchical:**
```json
{
  "ir": [{
    "c": "C1",
    "n": "SampleService",
    "m": [{
      "n": "M1",
      "nm": "processComplexData",
      "p": [["_", "P1", "$s", "payload"]],
      "r": "$b",
      "f": ["IF"]
    }, {
      "n": "M2",
      "nm": "doWork",
      "r": "$v"
    }]
  }]
}
```

**How it works:**
1. Class is the top-level container — class ID and name appear once
2. Methods are nested under their class — method ID and class ID are implicit
3. Params are nested under their method — method ID is implicit
4. Return type, flags, extends, implements are scoped to their parent
5. Import statements are a separate top-level array
6. Type aliases are a separate top-level array

**Key mappings from flat CoreOp to hierarchical fields:**

| CoreOp | Hierarchical Location |
|---|---|
| DefClass(id, name) | `{c: id, n: name}` (top-level array element) |
| DefMethod(cid, mid, name) | `{n: mid, nm: name}` (inside class) |
| DefField(cid, fid, name) | `{f: fid, n: name}` (inside class) |
| Param(mid, pid, type, name) | `[pid, type, name]` (inside method.p) |
| Return(mid, type) | `{r: type}` (inside method) |
| FieldType(fid, type) | `{ft: type}` (inside field) |
| Flags(tid, flags) | `{fl: flags}` (inside method) |
| ClassFlags(cid, flags) | `{cl: flags}` (inside class) |
| Extends(child, parent) | `{x: parent}` (inside class) |
| Implements(cid, iid) | `{i: iid}` (inside class) |
| Injects(cid, deps) | `{ij: deps}` (inside class) |
| Import(alias, module, named) | `[alias, module, named]` (top-level imports array) |
| TypeAlias(alias, original) | `[alias, original]` (top-level types array) |
| Pattern(name, args) | `{p: name, a: args}` (inside method or class) |

**Estimated savings:** 40-60% reduction. Eliminates all opcode strings AND parent ID repetitions.

**Files to create/modify:**
- `src/ir/hierarchical.rs` — `HierarchicalIR`, `ClassNode`, `MethodNode`, `FieldNode` structs
- `src/ir/hierarchical.rs` — `ir_to_hierarchical()` / `hierarchical_to_ir()` converters
- `src/ir/wire.rs` — add `ir_to_hierarchical_wire()` encoder
- `src/ir/replay.rs` — add hierarchical wire decoder
- `src/tests/ir/hierarchical.rs` — round-trip tests

### 5.2 Idea #1: Binary Wire Format

**Wire format change:** Replace JSON with a compact binary encoding.

**Current JSON positional:**
```
["C1","M1","processComplexData"]  = 35 bytes (with quotes/commas)
```

**Proposed binary:**
```
[0x01][varint:C1_idx][varint:M1_idx][varint:name_len][name_bytes]  = ~15 bytes
```

**Binary encoding spec:**

```
┌─────────────────────────────────────────────────────────┐
│ Header: magic(2) + version(1) + string_table_len(varint)│
│ String Table: [count(varint), (len(varint), bytes)*]    │
│ Instructions: [count(varint), instruction*]              │
│                                                          │
│ Instruction:                                             │
│   opcode_idx: u8 (0-14 for 15 core opcodes + patterns)  │
│   operands: [varint]* (string table indices)             │
│   For variadic ops: operand_count as varint prefix       │
└─────────────────────────────────────────────────────────┘
```

**Estimated savings:** 60-70% reduction in wire bytes.

**Trade-off:** Not human-readable. Best used as an optional transport encoding. The JSON wire format remains the default for debugging and mixed streams.

**Files to create/modify:**
- `src/ir/binary_wire.rs` — binary encoder/decoder
- `src/ir/wire.rs` — add encoding detection in `wire_to_ir()`
- `src/tests/ir/binary_wire.rs` — round-trip tests

---

## 6. Phase III: Ultra-Compressed Text

**Goal:** Reduce the text compression pipeline output without changing the structural information it conveys.

### 6.1 Idea #8: Progressive Header Elision

**Current header (~30 tokens):**
```
// --- Token Optimization Report ---
// Raw Tokens: 245 | Retained Tokens: 67 | Waste Reduced: 72.65%
// Fidelity: Low
// Structures: 3, 12, 5 | 245/67 tokens
// α1
```

**Proposed compact header (~5 tokens):**
```
§245:67:72.6|L|3:12:5|α1§
```

**Format spec:** `§raw_tokens:compressed_tokens:savings_pct|fidelity_code|classes:methods:imports|file_alias§`

**Fidelity codes:** `L` = Low, `M` = Medium, `H` = High, `C` = Cache hit

**Estimated savings:** ~25 tokens per compression.

**Files to create/modify:**
- `src/compression/report.rs` — add `format_compact_header()` function
- `src/decompression/` — add compact header parser (if needed for analytics)
- `src/tests/compression/` — header format tests

### 6.2 Idea #7: Structural Deduplication with Scope Defaults

**Current Low fidelity:**
```
$ctor C1 M1 $s payload;$r M1 $b;FLAGS M1 IF;$ctor C1 M2 $s data;$r M2 $b;FLAGS M2 IF
```

**Proposed with scope defaults:**
```
$dft$r=$b fl=IF;$ctor C1 M1 $s payload;$ctor C1 M2 $s data
```

**How it works:**
1. Before emitting method lines, scan for common patterns within a class
2. If ≥2 methods share the same return type, emit it as a scope default
3. If ≥2 methods share the same flags, emit them as scope defaults
4. Methods that match the defaults omit those elements
5. Methods that differ from defaults include them explicitly

**Scope default syntax:**
- `$dft` = scope default marker
- `r=$b` = return type default
- `fl=IF` = flags default
- Methods omit defaults: `$ctor C1 M1 $s payload` (no return type or flags — they're defaulted)

**Estimated savings:** 20-35% for classes with repetitive method patterns.

**Files to create/modify:**
- `src/compression/pipeline.rs` — add scope-default detection in `build_output_lines()`
- `src/compression/markers.rs` — add default marker encoding
- `src/decompression/` — add default expansion logic
- `src/tests/compression/` — scope default tests

### 6.3 Idea #9: Cross-File Symbol Deduplication

**Current per-file symbol dictionaries:**
```
File A: $1=Service $2=Observable $3=HttpClient
File B: $1=Service $4=BehaviorSubject $5=HttpClient
```

**Proposed workspace-level symbol dictionary:**
```
Global: $1=Service $2=Observable $3=HttpClient $4=BehaviorSubject
File A: refs($1,$2,$3)
File B: refs($1,$4,$3)
```

**How it works:**
1. When compressing a workspace, build a global frequency table across all files
2. Assign symbol opcodes by global frequency (most frequent = shortest code)
3. Per-file output references the global dictionary
4. Each file only needs to declare which global symbols it uses (a small bitmask or list)

**Estimated savings:** 15-30% for workspace-level compression.

**Files to create/modify:**
- `src/compression/workspace.rs` — add global symbol table builder
- `src/compression/symbol_compression.rs` — add workspace-aware compression
- `src/dictionary/symbol.rs` — add `GlobalSymbolTable` type
- `src/tests/compression/workspace.rs` — workspace symbol tests

### 6.4 Idea #10: Huffman-Coded Symbol Dictionary

**Current sequential assignment:**
```
$1=service (used 50 times)
$2=x (used 2 times)
$3=Observable (used 30 times)
```

**Proposed frequency-weighted:**
```
$a=service (1 char × 50 uses = 50 chars saved)
$b=Observable (1 char × 30 uses = 30 chars saved)
$z=x (1 char × 2 uses = 2 chars saved)
```

**How it works:**
1. Count token frequencies in the compressed body
2. Build a Huffman tree from frequencies
3. Assign shortest codes to most frequent tokens
4. The symbol footer includes the Huffman tree structure for decompression

**Estimated savings:** 10-20% on symbol-compressed output.

**Files to create/modify:**
- `src/dictionary/symbol.rs` — add `HuffmanSymbolDictionary` type
- `src/compression/symbol_compression.rs` — add Huffman-aware compression path
- `src/tests/dictionary/` — Huffman round-trip tests

### 6.5 Idea #11: Micro-Opcode Table for Text

**Current:** Structural markers use multi-character patterns

**Proposed:** Add single-character micro-opcodes for common structures

| Micro-opcode | Meaning | Current equivalent |
|---|---|---|
| `§C` | Class definition start | `$c` + class entry |
| `§M` | Method definition | method signature |
| `§F` | Field definition | field entry |
| `§I` | Import block | import entries |
| `§D` | Decorator/meta | `Φ` block |
| `§P` | Pattern (ctor/getter/etc.) | pattern marker |

**Estimated savings:** 15-25% by reducing structural repetition.

**Files to create/modify:**
- `src/compression/opcodes.rs` — add micro-opcode table
- `src/decompression/opcodes.rs` — add micro-opcode expansion
- `src/compression/pipeline.rs` — emit micro-opcodes at Low fidelity
- `src/tests/compression/` — micro-opcode round-trip tests

---

## 7. Phase IV: Text Delta Support

**Goal:** Make the text compression pipeline stateful, enabling delta-based transport like the IR system.

### 7.1 Idea #12: Delta-Aware Text Compression

**First compression (full output):**
```
§245:67:72.6|L|3:12:5|α1§
$1=Service;$2=Observable;$3=HttpClient;$ctor C1 M1 $s payload;$r M1 $b;FLAGS M1 IF
```

**Subsequent edit (delta):**
```
§Δα1:3:4§
+$ctor C1 M5 $s data;$r M5 $b
-FLAGS M2 IF
```

**Delta format spec:**
```
§Δfile_alias:from_version:to_version§
+<added lines>
-<removed lines>
~<modified lines>
```

**How it works:**
1. First compression produces full output + stores structural snapshot
2. On re-compression, compute text-level structural diff
3. Emit compact delta instead of full output
4. Client maintains last-seen output and applies delta patches

**Estimated savings:** 70-90% for edit sessions.

**Trade-off:** Requires client-side state management (similar to IR replay). The text delta is a parallel system to the IR delta — clients can use either.

**Files to create/modify:**
- `src/compression/text_delta.rs` — `TextDelta`, `TextDeltaComputer`
- `src/compression/pipeline.rs` — add delta mode to `compress_file()`
- `src/mcp/state.rs` — add text delta state tracking
- `src/mcp/tools.rs` — add `delta_text_context` tool
- `src/tests/compression/text_delta.rs` — delta round-trip tests

---

## 8. Expected Savings Summary

### 8.1 Full Compression (First Time)

| Pipeline | Current | After Phase I | After Phase II | Combined |
|---|---|---|---|---|
| IR (32-line file) | ~20 tok | ~12 tok | ~8 tok | **~8 tok** (60%↓) |
| IR (438-line file) | ~55 tok | ~33 tok | ~22 tok | **~22 tok** (60%↓) |
| Text (32-line file) | 26 tok | 26 tok | 26 tok | **~18 tok** (31%↓) |
| Text (438-line file) | 75 tok | 75 tok | 75 tok | **~50 tok** (33%↓) |

### 8.2 Delta Transport (Edit Session)

| Pipeline | Current | After Phase I | After Phase IV | Combined |
|---|---|---|---|---|
| IR delta (1 method) | ~8-12 tok | ~5-7 tok | N/A | **~5-7 tok** (40%↓) |
| IR delta (1 line) | ~5-8 tok | ~3-5 tok | N/A | **~3-5 tok** (40%↓) |
| Text delta (1 method) | 75 tok (full) | 75 tok (full) | ~8-12 tok | **~8-12 tok** (85%↓) |
| Text delta (1 line) | 75 tok (full) | 75 tok (full) | ~5-8 tok | **~5-8 tok** (90%↓) |

### 8.3 Cumulative 50-Edit Session

| Pipeline | Current | After All Phases |
|---|---|---|
| IR | ~200 tok | **~120 tok** (40%↓) |
| Text | 3,750 tok (full re-compress) | **~300 tok** (92%↓) |

---

## 9. Correctness Guarantees

Every proposal in this plan preserves **100% information fidelity**:

### 9.1 IR Changes (Phases I & II)
- **String table** (Idea #2): Lossless index mapping. Client rebuilds identical CoreOp tree.
- **Field-level diffing** (Idea #3): Client applies patches to existing instruction — identical result.
- **Contextual delta** (Idea #6): Same delta semantics, abbreviated encoding.
- **Hierarchical IR** (Idea #4): Same CoreOp instructions, tree structure instead of flat list.
- **Binary wire** (Idea #1): Lossless binary encoding. Optional transport format.

### 9.2 Text Changes (Phase III)
- **Header elision** (Idea #8): Same metadata, different format. Parseable by AI.
- **Scope defaults** (Idea #8): Defaults expanded on decompression — identical structural output.
- **Cross-file symbols** (Idea #9): Same token replacement, shared dictionary.
- **Huffman coding** (Idea #10): Optimal prefix-free codes — lossless.
- **Micro-opcodes** (Idea #11): Expanded to original markers — identical output.

### 9.3 Delta Changes (Phase IV)
- **Text delta** (Idea #12): Same structural diff semantics as IR delta.

### 9.4 Testing Strategy
- All new encoders MUST have round-trip tests: encode → decode → compare with original
- All delta formats MUST have apply-and-verify tests: delta applied to state → matches expected
- Existing test suite (504+ tests) MUST continue passing with zero regressions
- Token savings demo (`examples/token_savings_demo.rs`) MUST be updated to show new formats

---

## 10. Implementation Timeline

### Phase I: Ultra-Compact IR — Quick Wins (✅ Complete)
1. [x] Implement string table per-file (Idea #2)
   - [x] `src/ir/string_table.rs` — StringTable struct, encode/decode, wire encoder/decoder
   - [x] `src/ir/wire.rs` — `wire_to_ir_detect()` for encoding-aware decoding
   - [x] `src/ir/replay.rs` — decoding works via `wire_to_ir_detect()` chain
   - [x] `src/tests/ir/string_table.rs` — 22 round-trip tests
2. [x] Implement delta field-level diffing (Idea #3)
   - [x] `src/ir/delta.rs` — `FieldPatch`, `compute_field_patches()`, `ModOp` dual format
   - [x] `src/ir/replay.rs` — field-patch application in `ContextState::apply()`
   - [x] `src/tests/ir/delta.rs` — existing tests updated for new ModOp format
3. [x] Implement contextual delta encoding (Idea #6)
   - [x] `src/ir/delta.rs` — `CompactDelta`, `CompactOps`, `compact_encode()`, `compact_decode()`
   - [x] `src/tests/ir/delta.rs` — compact delta via demo/examples
4. [x] Update token savings demo with new formats
5. [x] Run full test suite — verify 0 regressions (`cargo check` clean)

### Phase II: Ultra-Compact IR — Structural (Week 3-4)
6. [ ] Implement scoped hierarchical IR (Idea #4)
   - [ ] `src/ir/hierarchical.rs` — tree structs + converters
   - [ ] `src/ir/wire.rs` — hierarchical wire encoder
   - [ ] `src/ir/replay.rs` — hierarchical wire decoder
   - [ ] `src/tests/ir/hierarchical.rs` — round-trip tests
7. [ ] Implement binary wire format (Idea #1)
   - [ ] `src/ir/binary_wire.rs` — binary encoder/decoder
   - [ ] `src/ir/wire.rs` — encoding detection
   - [ ] `src/tests/ir/binary_wire.rs` — round-trip tests
8. [ ] Run full test suite — verify 0 regressions

### Phase III: Ultra-Compressed Text (Week 5-6)
9. [ ] Implement progressive header elision (Idea #8)
   - [ ] `src/compression/report.rs` — format_compact_header()
   - [ ] `src/tests/compression/` — header format tests
10. [ ] Implement structural deduplication (Idea #7)
    - [ ] `src/compression/pipeline.rs` — scope-default detection
    - [ ] `src/tests/compression/` — scope default tests
11. [ ] Implement cross-file symbol deduplication (Idea #9)
    - [ ] `src/compression/workspace.rs` — global symbol table
    - [ ] `src/tests/compression/workspace.rs` — workspace tests
12. [ ] Implement Huffman-coded symbols (Idea #10)
    - [ ] `src/dictionary/symbol.rs` — HuffmanSymbolDictionary
    - [ ] `src/tests/dictionary/` — Huffman tests
13. [ ] Run full test suite — verify 0 regressions

### Phase IV: Text Delta Support (Week 7-8)
14. [ ] Implement delta-aware text compression (Idea #12)
    - [ ] `src/compression/text_delta.rs` — TextDelta, TextDeltaComputer
    - [ ] `src/compression/pipeline.rs` — delta mode
    - [ ] `src/mcp/state.rs` — text delta state tracking
    - [ ] `src/mcp/tools.rs` — delta_text_context tool
    - [ ] `src/tests/compression/text_delta.rs` — delta tests
15. [ ] Run full test suite — verify 0 regressions
16. [ ] Update documentation (COMPILER_IR.md, ROADMAP.md)

---

## Appendix A: Idea Reference Table

| # | Idea | System | Savings | Feasibility | Correctness Risk | Phase |
|---|---|---|---|---|---|---|
| 1 | Binary Wire Format | IR | 60-70% | Medium | None | II |
| 2 | String Table + Indexing | IR | 25-40% | High | None | I |
| 3 | Delta Field-Level Diffing | IR Delta | 30-50% | High | None | I |
| 4 | Scoped Hierarchical IR | IR | 40-60% | Medium | Low | II |
| 5 | RLE Delta Batching | IR Delta | 15-25% | Medium | None | Deferred |
| 6 | Contextual Delta Encoding | IR Delta | 40-60% | High | None | I |
| 7 | Structural Deduplication | Text | 20-35% | Medium | None | III |
| 8 | Progressive Header Elision | Text | ~25 tok | Very High | None | III |
| 9 | Cross-File Symbols | Text | 15-30% | Medium | None | III |
| 10 | Huffman-Coded Symbols | Text | 10-20% | High | None | III |
| 11 | Micro-Opcodes | Text | 15-25% | Medium | None | Deferred |
| 12 | Delta Text Compression | Text | 70-90% | High | Medium | IV |

**Deferred:** Ideas #5 (RLE Delta Batching) and #11 (Micro-Opcodes) are lower priority and can be added in a future phase if needed.

---

## Appendix B: Wire Format Compatibility

All new wire formats are identified by the `"encoding"` field in the JSON envelope:

| Encoding Value | Format | Decoder |
|---|---|---|
| `"named"` | Original named IR (Phase A) | `wire::wire_to_ir()` |
| `"positional"` | Opcode-stripped (Phase H) | `positional::decode_op()` |
| `"tagged"` | Debug/mixed (Phase H) | Same as named |
| `"string_table"` | **NEW** — string table indexed (Phase I) | `string_table::wire_to_ir()` |
| `"compact_delta"` | **NEW** — field-patch deltas (Phase I) | `delta::compact_decode()` |
| `"hierarchical"` | **NEW** — tree structure (Phase II) | `hierarchical::wire_to_ir()` |
| `"binary"` | **NEW** — binary wire (Phase II) | `binary_wire::decode()` |

Existing decoders continue to work unchanged. New decoders are additive.