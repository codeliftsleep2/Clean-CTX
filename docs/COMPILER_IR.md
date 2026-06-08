# Clean-CTX — Compiler IR: Structured State Protocol

**Version:** 0.1.0 (Proposed)
**Last updated:** 2026-06-08 (Phase H marked complete)
**Status:** Phase A Complete - Phase B Complete - Phase C Complete - Phase D Complete - Phase E Complete - Phase F Complete - Phase G Complete - **Phase H Complete**

> **Living document.** This spec defines the evolution from text-based compression to a structured intermediate representation (IR) with delta-based state transport. It serves as the implementation guideline for the Compiler IR subsystem.

---

## Table of Contents

1. [Motivation](#1-motivation)
2. [Architecture Overview](#2-architecture-overview)
3. [The Pipeline](#3-the-pipeline)
4. [Phase A: IR Core](#4-phase-a-ir-core)
5. [Phase B: Global Symbol Table](#5-phase-b-global-symbol-table)
6. [Phase C: Delta Transport](#6-phase-c-delta-transport)
7. [Phase D: State Replay](#7-phase-d-state-replay)
8. [Phase E: IR / Pretty Separation](#8-phase-e-ir--pretty-separation)
9. [Phase F: Layered Encoding](#9-phase-f-layered-encoding)
10. [Phase G: Integration & MCP Tools](#10-phase-g-integration--mcp-tools)
11. [Phase H: Positional Encoding & Advanced Compression](#11-phase-h-positional-encoding--advanced-compression)
12. [Migration Map](#12-migration-map)
13. [Wire Protocol Reference](#13-wire-protocol-reference)
14. [Static Schema Definition](#14-static-schema-definition)
15. [Phase Dependencies & Timeline](#15-phase-dependencies--timeline)

---

## 1. Motivation

The current Clean-CTX system operates as a **one-shot text compression pipeline**:

```
Source → Tree-sitter AST → Fidelity Filter → Opcode Encode → Text Output
```

Every compression is a full re-parse. The diff system computes deltas between structural snapshots but emits them as human-readable diff lines, not machine-replayable instruction streams. The client (LLM) must re-read the full compressed output on every interaction, even for a single method change.

**The core limitation:** all state is serialized as text strings. There is no concept of incremental state application.

### What Changes

| Dimension | Current | Compiler IR |
|-----------|---------|-------------|
| Output format | Compressed text | Structured instruction stream |
| Diff output | Human-readable text diff | Machine-applicable delta ops |
| State model | Stateless (re-parse each time) | Stateful (apply deltas to state) |
| Transport | Full output per call | Full first, deltas after |
| Symbol tracking | Per-file dictionaries | Cross-stage global symbol table |
| Layer separation | Monolithic pipeline | 4-layer encoding architecture |
| Fidelity | Compile-time filter | Render-time filter (same IR) |
| Wire size | Verbose opcode names | Key-stripped positional + pattern compression |

### Expected Token Savings

| Scenario | Current (Low) | IR Full | IR Delta | IR Full + Phase H |
|---|---|---|---|---|
| First compression (32-line file) | 26 tokens | ~20 tokens | N/A | ~14 tokens |
| First compression (438-line file) | 75 tokens | ~55 tokens | N/A | ~38 tokens |
| Subsequent edit (1 method changed) | 75 tokens (full re-compress) | 75 tokens | ~8-12 tokens | ~8-12 tokens |
| Subsequent edit (1 line changed) | 75 tokens | 75 tokens | ~5-8 tokens | ~5-8 tokens |
| 50-edit session (cumulative) | 3,750 tokens | 3,750 tokens | ~200 tokens | ~200 tokens |

Phase H reduces the *first compression* size by another ~30% on top of the named IR by stripping the redundant opcode strings and merging repeated patterns.

---

## 2. Architecture Overview

### 4-Layer Encoding Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Layer 4: Application Patterns + Positional Encoding    │
│  Pattern recognition, key stripping, positional tuples  │
│                                                         │
│  ["DEF_C","C1"] ["DEF_M","C1","M1"] ["SIG","M1","P1",   │
│   "$s","payload"]   ← or  ["C1","M1","processComplex…"] │
│                       ← or  ["PAT","CTOR","C1","M1",…]  │
├─────────────────────────────────────────────────────────┤
│  Layer 3: Meta-Layer (Framework-Specific)               │
│  Angular Φ markers, React patterns, NgRx patterns       │
│                                                         │
│  ["NG_COMPONENT","C1",{selector:"app-root",...}]        │
├─────────────────────────────────────────────────────────┤
│  Layer 2: Language Layer (TS, C#, etc.)                 │
│  Language-specific ops that map to Core IR              │
│                                                         │
│  ["TS_ASYNC","M1"] ["TS_GENERICS","C1",["T"]]           │
├─────────────────────────────────────────────────────────┤
│  Layer 1: Core IR (Language-Agnostic)                   │
│  Universal instruction set — every language compiles    │
│  down to these operations                               │
│                                                         │
│  DEF_C, DEF_M, DEF_F, SIG, RET, FLAGS, EXT, IMP, IMP    │
└─────────────────────────────────────────────────────────┘
```

### The Pipeline

```
     Source Code
          │
          ▼
┌─────────────────────┐
│   Tree-sitter AST   │  Existing capture pipeline (unchanged)
│   Parse + Extract   │
└─────────┬───────────┘
          │ CapEntry captures
          ▼
┌─────────────────────┐
│   IR Compiler       │  NEW: translates captures → Vec<CoreOp>
│   (Core + Lang +    │  Runs all 4 layers
│    Meta + Pattern)  │
└─────────┬───────────┘
          │ CompiledIR { instructions, version }
          │
     ┌────┴────────────────────┐
     ▼                         ▼
┌─────────────┐     ┌──────────────────┐
│ IR → Wire   │     │ IR → Pretty Text │
│ (delta,     │     │ (backward-compat │
│  transport, │     │  output)         │
│  positional)│     └────────┬─────────┘
└──────┬──────┘              │
       │                     ▼
       ▼              ┌──────────────────┐
┌─────────────┐        │ Human-Readable   │
│ Delta       │        │ Compressed       │
│ Transport   │        │ Output           │
│ Protocol    │        └──────────────────┘
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│   State Replay      │  Apply delta ops to client state
│   (Apply + Render)  │
└─────────────────────┘
```

---

## 3. The Pipeline

### 3.1 Compilation (Source → IR)

The IR compiler reuses the existing tree-sitter capture pipeline (`run_capture_pipeline`) but replaces the text-formatting orchestration (`build_output_lines`) with instruction emission:

```
1. Tree-sitter parse (existing — no change)
2. Capture walk (existing — no change)
3. Core IR emission (NEW — replaces build_output_lines)
4. Language layer translation (NEW — TS/C# specific ops)
5. Meta-layer pass (REFACTORED — angular_meta implements MetaLayer trait)
6. Pattern recognition (NEW — Layer 4 pattern compression)
7. Register in global symbol table (NEW — cross-stage tracking)
```

### 3.2 Delta Computation (IR → Delta)

Instead of computing text diffs between `CapturedStructure` snapshots, the delta engine computes instruction-level deltas between `CompiledIR` states:

```
1. Index both IRs by symbol (opcode + primary key)
2. Diff the indices:
   - Symbols in current but not baseline → additions
   - Symbols in baseline but not current → deletions
   - Symbols in both but different instructions → modifications
3. Emit DeltaOps envelope
```

### 3.3 State Replay (Delta → Updated State)

The client applies delta ops to its local state machine:

```
1. Validate version chain (from_version must match)
2. Apply deletions (remove instructions + unregister symbols)
3. Apply modifications (in-place replacement)
4. Apply additions (append + register symbols)
5. Bump version
6. Render if needed (IR → pretty text at requested fidelity)
```

---

## 4. Phase A: IR Core

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Define the IR instruction types and build the compiler that translates tree-sitter captures into structured instructions.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/mod.rs` | Module root, public API exports |
| `src/ir/opcodes.rs` | `CoreOp` enum, opcode constants, arity table |
| `src/ir/compiler.rs` | `IRCompiler` struct, `compile()` method |
| `src/ir/render.rs` | `ir_to_text()` — pretty output from IR |
| `src/ir/wire.rs` | `ir_to_wire()` — JSON serialization |
| `src/ir/mod.rs` (tests) | Unit tests for compiler + render |

### Core Instruction Types

```rust
// src/ir/opcodes.rs

/// Core IR opcodes — the universal instruction set.
/// Every language compiles down to these operations.
/// Serialized as positional JSON arrays: [opcode, ...operands]
pub enum CoreOp {
    /// ["DEF_C", class_id, original_name]
    DefClass(String, String),
    DefMethod(String, String, String),
    DefField(String, String, String),
    DefInterface(String, String),
    Param(String, String, String, String),
    Return(String, String),
    FieldType(String, String),
    Flags(String, Vec<String>),
    ClassFlags(String, Vec<String>),
    Extends(String, String),
    Implements(String, String),
    Injects(String, Vec<String>),
    Import(String, String, String),
    TypeAlias(String, String),
}
```

### Completion Criteria

- [x] `src/ir/mod.rs` created with module declarations
- [x] `CoreOp` enum defined with all 14 instruction types
- [x] `op_to_tuple()` serializes every `CoreOp` variant to positional tuple
- [x] `IRCompiler::compile()` processes captures and emits `Vec<CoreOp>`
- [x] `CompiledIR` struct defined (file_id, instructions, version)
- [x] `ir_to_wire()` serializes `CompiledIR` to JSON value
- [x] Unit tests: compile a sample TypeScript file → verify IR instruction count and types
- [x] Unit tests: `op_to_tuple()` round-trip for every `CoreOp` variant
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 5. Phase B: Global Symbol Table

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Unified symbol registry that subsumes `SymbolDictionary` and `PathDictionary` into a cross-stage, version-tracked registry.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/symbol_table.rs` | `GlobalSymbolTable`, `SymbolEntry`, `SymbolKind` |
| `src/tests/ir/symbol_table.rs` | 30 tests: registration, lookup, versioning, unregister |

### Completion Criteria

- [x] `GlobalSymbolTable` struct implemented with all methods
- [x] `SymbolEntry` and `SymbolKind` defined
- [x] `next_alias()` generates correct prefixed aliases (C1, M1, F1, etc.)
- [x] `register()` / `unregister()` / `touch()` maintain all indexes
- [x] `get()` / `get_by_original()` / `get_file_symbols()` / `get_changed_since()` work correctly
- [x] Version bumping works (each `bump_version()` increments monotonically)
- [x] Unit tests: register 10 symbols across 3 files → verify lookup by alias, original, and file
- [x] Unit tests: unregister removes from all indexes
- [x] Unit tests: `get_changed_since()` returns correct subset
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 6. Phase C: Delta Transport

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Instruction-level diffing between two `CompiledIR` states, producing a structured delta envelope for transport.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/delta.rs` | `IRDelta`, `DeltaOps`, `ModOp`, `DeltaComputer` |
| `src/tests/ir/delta.rs` | 26 tests: add/modify/remove detection, version chain, JSON round-trip, edge cases |

### Delta Wire Format

```rust
// src/ir/delta.rs

/// A structured delta between two IR states.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IRDelta {
    pub file: String,
    pub from_version: u64,
    pub to_version: u64,
    pub ops: DeltaOps,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DeltaOps {
    #[serde(rename = "+")]
    pub adds: Vec<Vec<String>>,
    #[serde(rename = "~")]
    pub mods: Vec<ModOp>,
    #[serde(rename = "-")]
    pub dels: Vec<Vec<String>>,
}
```

### Completion Criteria

- [x] `IRDelta` struct with `DeltaOps` (adds, mods, dels) defined and serializable
- [x] `ModOp` struct with key/replace fields
- [x] `DeltaComputer::compute()` correctly identifies additions, removals, modifications
- [x] `primary_key()` generates unique keys for every instruction type
- [x] `key_tuple()` extracts the match key for modifications
- [x] Delta is `None` when IRs are identical (no unnecessary deltas)
- [x] Version chain: `from_version` = baseline.version, `to_version` = current.version
- [x] Unit tests: add a method → delta has 1 add op
- [x] Unit tests: remove a method → delta has 1 del op (and its SIG/RET)
- [x] Unit tests: modify a method signature → delta has 1 mod op
- [x] Unit tests: unchanged IR → delta is None
- [x] JSON serialization produces correct `+`/`~`/`-` keys
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 7. Phase D: State Replay

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Client-side state machine that applies delta ops to reconstruct IR state, with version-based catch-up support.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/replay.rs` | `ContextState`, `FileState`, `DeltaError` |
| `src/tests/ir/replay.rs` | 39 tests: FileState ops, ContextState apply, version validation, error cases, sequential deltas, render, multi-file, full replay cycle |

### Completion Criteria

- [x] `FileState` with instructions, index, and version tracking
- [x] `ContextState` with per-file management
- [x] `apply()` validates version chain before applying
- [x] Apply order: deletions → modifications → additions
- [x] `remove_by_key()` correctly removes and rebuilds index
- [x] `replace_by_key()` correctly replaces and updates index
- [x] `append()` adds instruction and updates index
- [x] `render_pretty()` delegates to `ir_to_text()`
- [x] `load_ir()` bootstraps state from full CompiledIR
- [x] Error cases: unknown file, version mismatch, missing symbol
- [x] Unit tests: apply add delta → new instruction present
- [x] Unit tests: apply remove delta → instruction absent
- [x] Unit tests: apply modify delta → instruction updated
- [x] Unit tests: version mismatch → DeltaError::VersionMismatch
- [x] Unit tests: unknown file → DeltaError::UnknownFile
- [x] Unit tests: sequential deltas (v1→v2→v3) apply correctly
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 8. Phase E: IR / Pretty Separation

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Two completely independent render paths from the same canonical IR. The IR is the source of truth; pretty output is derived.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/render.rs` | `ir_to_text()` — fidelity-aware rendering |
| `src/tests/ir/render.rs` | 17 tests: round-trip tests, fidelity comparison |

### Key Insight: Backward Compatibility

The `ir_to_text()` function can produce **byte-identical output** to the current `compress_code_context` tool. This means:

1. **Existing tools keep working** — `compress_code_context` just calls `ir_to_text(compiler.compile(...))`
2. **New tools expose IR** — `compress_code_context_ir` calls `ir_to_wire(compiler.compile(...))`
3. **Same compilation, two outputs** — no duplicate parsing logic

### Completion Criteria

- [x] `ir_to_text()` renders IR to human-readable text at all 3 fidelity levels
- [x] Low fidelity: compact output with opcodes and ⊕ markers
- [x] Medium fidelity: full signatures with keywords
- [x] High fidelity: indented with behavior markers in braces
- [x] `flags_to_markers()` converts IR flags back to ⊕ notation
- [x] Import rendering at all fidelity levels
- [x] Class/method/field rendering at all fidelity levels
- [x] Round-trip test: compile sample → render → matches expected output format
- [x] Fidelity comparison: same IR at Low vs Medium vs High shows progressive detail
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 9. Phase F: Layered Encoding

**t y p e   :   C a n n o t   f i n d   p a t h   ' C : \ U s e r s \ M N a s t y \ D e s k t o p \ R u s t C o n t e x t L a y e r A I \ n u l '   b e c a u s e   i t   d o e s   n o t   e x i s t .  
 A t   l i n e : 1   c h a r : 1  
 +   t y p e   n u l   > >   d o c s \ C O M P I L E R _ I R . m d   2 > & 1  
 +   ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~  
         +   C a t e g o r y I n f o                     :   O b j e c t N o t F o u n d :   ( C : \ U s e r s \ M N a s t y . . . t e x t L a y e r A I \ n u l : S t r i n g )   [ G e t - C o n t e n t ] ,   I t e m N o t F o u n d E x c e p t i o n  
         +   F u l l y Q u a l i f i e d E r r o r I d   :   P a t h N o t F o u n d , M i c r o s o f t . P o w e r S h e l l . C o m m a n d s . G e t C o n t e n t C o m m a n d  
    
 # Clean-CTX — Compiler IR: Structured State Protocol

**Version:** 0.1.0 (Proposed)
**Last updated:** 2026-06-08 (Phase H marked complete)
**Status:** Phase A Complete - Phase B Complete - Phase C Complete - Phase D Complete - Phase E Complete - Phase F Complete - Phase G Complete - **Phase H Complete**

> **Living document.** This spec defines the evolution from text-based compression to a structured intermediate representation (IR) with delta-based state transport. It serves as the implementation guideline for the Compiler IR subsystem.

---

## Table of Contents

1. [Motivation](#1-motivation)
2. [Architecture Overview](#2-architecture-overview)
3. [The Pipeline](#3-the-pipeline)
4. [Phase A: IR Core](#4-phase-a-ir-core)
5. [Phase B: Global Symbol Table](#5-phase-b-global-symbol-table)
6. [Phase C: Delta Transport](#6-phase-c-delta-transport)
7. [Phase D: State Replay](#7-phase-d-state-replay)
8. [Phase E: IR / Pretty Separation](#8-phase-e-ir--pretty-separation)
9. [Phase F: Layered Encoding](#9-phase-f-layered-encoding)
10. [Phase G: Integration & MCP Tools](#10-phase-g-integration--mcp-tools)
11. [Phase H: Positional Encoding & Advanced Compression](#11-phase-h-positional-encoding--advanced-compression)
12. [Migration Map](#12-migration-map)
13. [Wire Protocol Reference](#13-wire-protocol-reference)
14. [Static Schema Definition](#14-static-schema-definition)
15. [Phase Dependencies & Timeline](#15-phase-dependencies--timeline)

---

## 1. Motivation

The current Clean-CTX system operates as a **one-shot text compression pipeline**:

```
Source → Tree-sitter AST → Fidelity Filter → Opcode Encode → Text Output
```

Every compression is a full re-parse. The diff system computes deltas between structural snapshots but emits them as human-readable diff lines, not machine-replayable instruction streams. The client (LLM) must re-read the full compressed output on every interaction, even for a single method change.

**The core limitation:** all state is serialized as text strings. There is no concept of incremental state application.

### What Changes

| Dimension | Current | Compiler IR |
|-----------|---------|-------------|
| Output format | Compressed text | Structured instruction stream |
| Diff output | Human-readable text diff | Machine-applicable delta ops |
| State model | Stateless (re-parse each time) | Stateful (apply deltas to state) |
| Transport | Full output per call | Full first, deltas after |
| Symbol tracking | Per-file dictionaries | Cross-stage global symbol table |
| Layer separation | Monolithic pipeline | 4-layer encoding architecture |
| Fidelity | Compile-time filter | Render-time filter (same IR) |
| Wire size | Verbose opcode names | Key-stripped positional + pattern compression |

### Expected Token Savings

| Scenario | Current (Low) | IR Full | IR Delta | IR Full + Phase H |
|---|---|---|---|---|
| First compression (32-line file) | 26 tokens | ~20 tokens | N/A | ~14 tokens |
| First compression (438-line file) | 75 tokens | ~55 tokens | N/A | ~38 tokens |
| Subsequent edit (1 method changed) | 75 tokens (full re-compress) | 75 tokens | ~8-12 tokens | ~8-12 tokens |
| Subsequent edit (1 line changed) | 75 tokens | 75 tokens | ~5-8 tokens | ~5-8 tokens |
| 50-edit session (cumulative) | 3,750 tokens | 3,750 tokens | ~200 tokens | ~200 tokens |

Phase H reduces the *first compression* size by another ~30% on top of the named IR by stripping the redundant opcode strings and merging repeated patterns.

---

## 2. Architecture Overview

### 4-Layer Encoding Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Layer 4: Application Patterns + Positional Encoding    │
│  Pattern recognition, key stripping, positional tuples  │
│                                                         │
│  ["DEF_C","C1"] ["DEF_M","C1","M1"] ["SIG","M1","P1",   │
│   "$s","payload"]   ← or  ["C1","M1","processComplex…"] │
│                       ← or  ["PAT","CTOR","C1","M1",…]  │
├─────────────────────────────────────────────────────────┤
│  Layer 3: Meta-Layer (Framework-Specific)               │
│  Angular Φ markers, React patterns, NgRx patterns       │
│                                                         │
│  ["NG_COMPONENT","C1",{selector:"app-root",...}]        │
├─────────────────────────────────────────────────────────┤
│  Layer 2: Language Layer (TS, C#, etc.)                 │
│  Language-specific ops that map to Core IR              │
│                                                         │
│  ["TS_ASYNC","M1"] ["TS_GENERICS","C1",["T"]]           │
├─────────────────────────────────────────────────────────┤
│  Layer 1: Core IR (Language-Agnostic)                   │
│  Universal instruction set — every language compiles    │
│  down to these operations                               │
│                                                         │
│  DEF_C, DEF_M, DEF_F, SIG, RET, FLAGS, EXT, IMP, IMP    │
└─────────────────────────────────────────────────────────┘
```

### The Pipeline

```
     Source Code
          │
          ▼
┌─────────────────────┐
│   Tree-sitter AST   │  Existing capture pipeline (unchanged)
│   Parse + Extract   │
└─────────┬───────────┘
          │ CapEntry captures
          ▼
┌─────────────────────┐
│   IR Compiler       │  NEW: translates captures → Vec<CoreOp>
│   (Core + Lang +    │  Runs all 4 layers
│    Meta + Pattern)  │
└─────────┬───────────┘
          │ CompiledIR { instructions, version }
          │
     ┌────┴────────────────────┐
     ▼                         ▼
┌─────────────┐     ┌──────────────────┐
│ IR → Wire   │     │ IR → Pretty Text │
│ (delta,     │     │ (backward-compat │
│  transport, │     │  output)         │
│  positional)│     └────────┬─────────┘
└──────┬──────┘              │
       │                     ▼
       ▼              ┌──────────────────┐
┌─────────────┐        │ Human-Readable   │
│ Delta       │        │ Compressed       │
│ Transport   │        │ Output           │
│ Protocol    │        └──────────────────┘
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│   State Replay      │  Apply delta ops to client state
│   (Apply + Render)  │
└─────────────────────┘
```

---

## 3. The Pipeline

### 3.1 Compilation (Source → IR)

The IR compiler reuses the existing tree-sitter capture pipeline (`run_capture_pipeline`) but replaces the text-formatting orchestration (`build_output_lines`) with instruction emission:

```
1. Tree-sitter parse (existing — no change)
2. Capture walk (existing — no change)
3. Core IR emission (NEW — replaces build_output_lines)
4. Language layer translation (NEW — TS/C# specific ops)
5. Meta-layer pass (REFACTORED — angular_meta implements MetaLayer trait)
6. Pattern recognition (NEW — Layer 4 pattern compression)
7. Register in global symbol table (NEW — cross-stage tracking)
```

### 3.2 Delta Computation (IR → Delta)

Instead of computing text diffs between `CapturedStructure` snapshots, the delta engine computes instruction-level deltas between `CompiledIR` states:

```
1. Index both IRs by symbol (opcode + primary key)
2. Diff the indices:
   - Symbols in current but not baseline → additions
   - Symbols in baseline but not current → deletions
   - Symbols in both but different instructions → modifications
3. Emit DeltaOps envelope
```

### 3.3 State Replay (Delta → Updated State)

The client applies delta ops to its local state machine:

```
1. Validate version chain (from_version must match)
2. Apply deletions (remove instructions + unregister symbols)
3. Apply modifications (in-place replacement)
4. Apply additions (append + register symbols)
5. Bump version
6. Render if needed (IR → pretty text at requested fidelity)
```

---

## 4. Phase A: IR Core

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Define the IR instruction types and build the compiler that translates tree-sitter captures into structured instructions.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/mod.rs` | Module root, public API exports |
| `src/ir/opcodes.rs` | `CoreOp` enum, opcode constants, arity table |
| `src/ir/compiler.rs` | `IRCompiler` struct, `compile()` method |
| `src/ir/render.rs` | `ir_to_text()` — pretty output from IR |
| `src/ir/wire.rs` | `ir_to_wire()` — JSON serialization |
| `src/ir/mod.rs` (tests) | Unit tests for compiler + render |

### Core Instruction Types

```rust
// src/ir/opcodes.rs

/// Core IR opcodes — the universal instruction set.
/// Every language compiles down to these operations.
/// Serialized as positional JSON arrays: [opcode, ...operands]
pub enum CoreOp {
    /// ["DEF_C", class_id, original_name]
    DefClass(String, String),
    DefMethod(String, String, String),
    DefField(String, String, String),
    DefInterface(String, String),
    Param(String, String, String, String),
    Return(String, String),
    FieldType(String, String),
    Flags(String, Vec<String>),
    ClassFlags(String, Vec<String>),
    Extends(String, String),
    Implements(String, String),
    Injects(String, Vec<String>),
    Import(String, String, String),
    TypeAlias(String, String),
}
```

### Completion Criteria

- [x] `src/ir/mod.rs` created with module declarations
- [x] `CoreOp` enum defined with all 14 instruction types
- [x] `op_to_tuple()` serializes every `CoreOp` variant to positional tuple
- [x] `IRCompiler::compile()` processes captures and emits `Vec<CoreOp>`
- [x] `CompiledIR` struct defined (file_id, instructions, version)
- [x] `ir_to_wire()` serializes `CompiledIR` to JSON value
- [x] Unit tests: compile a sample TypeScript file → verify IR instruction count and types
- [x] Unit tests: `op_to_tuple()` round-trip for every `CoreOp` variant
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 5. Phase B: Global Symbol Table

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Unified symbol registry that subsumes `SymbolDictionary` and `PathDictionary` into a cross-stage, version-tracked registry.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/symbol_table.rs` | `GlobalSymbolTable`, `SymbolEntry`, `SymbolKind` |
| `src/tests/ir/symbol_table.rs` | 30 tests: registration, lookup, versioning, unregister |

### Completion Criteria

- [x] `GlobalSymbolTable` struct implemented with all methods
- [x] `SymbolEntry` and `SymbolKind` defined
- [x] `next_alias()` generates correct prefixed aliases (C1, M1, F1, etc.)
- [x] `register()` / `unregister()` / `touch()` maintain all indexes
- [x] `get()` / `get_by_original()` / `get_file_symbols()` / `get_changed_since()` work correctly
- [x] Version bumping works (each `bump_version()` increments monotonically)
- [x] Unit tests: register 10 symbols across 3 files → verify lookup by alias, original, and file
- [x] Unit tests: unregister removes from all indexes
- [x] Unit tests: `get_changed_since()` returns correct subset
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 6. Phase C: Delta Transport

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Instruction-level diffing between two `CompiledIR` states, producing a structured delta envelope for transport.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/delta.rs` | `IRDelta`, `DeltaOps`, `ModOp`, `DeltaComputer` |
| `src/tests/ir/delta.rs` | 26 tests: add/modify/remove detection, version chain, JSON round-trip, edge cases |

### Delta Wire Format

```rust
// src/ir/delta.rs

/// A structured delta between two IR states.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IRDelta {
    pub file: String,
    pub from_version: u64,
    pub to_version: u64,
    pub ops: DeltaOps,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DeltaOps {
    #[serde(rename = "+")]
    pub adds: Vec<Vec<String>>,
    #[serde(rename = "~")]
    pub mods: Vec<ModOp>,
    #[serde(rename = "-")]
    pub dels: Vec<Vec<String>>,
}
```

### Completion Criteria

- [x] `IRDelta` struct with `DeltaOps` (adds, mods, dels) defined and serializable
- [x] `ModOp` struct with key/replace fields
- [x] `DeltaComputer::compute()` correctly identifies additions, removals, modifications
- [x] `primary_key()` generates unique keys for every instruction type
- [x] `key_tuple()` extracts the match key for modifications
- [x] Delta is `None` when IRs are identical (no unnecessary deltas)
- [x] Version chain: `from_version` = baseline.version, `to_version` = current.version
- [x] Unit tests: add a method → delta has 1 add op
- [x] Unit tests: remove a method → delta has 1 del op (and its SIG/RET)
- [x] Unit tests: modify a method signature → delta has 1 mod op
- [x] Unit tests: unchanged IR → delta is None
- [x] JSON serialization produces correct `+`/`~`/`-` keys
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 7. Phase D: State Replay

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Client-side state machine that applies delta ops to reconstruct IR state, with version-based catch-up support.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/replay.rs` | `ContextState`, `FileState`, `DeltaError` |
| `src/tests/ir/replay.rs` | 39 tests: FileState ops, ContextState apply, version validation, error cases, sequential deltas, render, multi-file, full replay cycle |

### Completion Criteria

- [x] `FileState` with instructions, index, and version tracking
- [x] `ContextState` with per-file management
- [x] `apply()` validates version chain before applying
- [x] Apply order: deletions → modifications → additions
- [x] `remove_by_key()` correctly removes and rebuilds index
- [x] `replace_by_key()` correctly replaces and updates index
- [x] `append()` adds instruction and updates index
- [x] `render_pretty()` delegates to `ir_to_text()`
- [x] `load_ir()` bootstraps state from full CompiledIR
- [x] Error cases: unknown file, version mismatch, missing symbol
- [x] Unit tests: apply add delta → new instruction present
- [x] Unit tests: apply remove delta → instruction absent
- [x] Unit tests: apply modify delta → instruction updated
- [x] Unit tests: version mismatch → DeltaError::VersionMismatch
- [x] Unit tests: unknown file → DeltaError::UnknownFile
- [x] Unit tests: sequential deltas (v1→v2→v3) apply correctly
- [x] `cargo clippy --all-targets -- -D warnings` passes

---

## 8. Phase E: IR / Pretty Separation

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Two completely independent render paths from the same canonical IR. The IR is the source of truth; pretty output is derived.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/render.rs` | `ir_to_text()` — fidelity-aware rendering |
| `src/tests/ir/render.rs` | 17 tests: round-trip tests, fidelity comparison |

### Key Insight: Backward Compatibility

The `ir_to_text()` function can produce **byte-identical output** to the current `compress_code_context` tool. This means:

1. **Existing tools keep working** — `compress_code_context` just calls `ir_to_text(compiler.compile(...))`
2. **New tools expose IR** — `compress_code_context_ir` calls `ir_to_wire(compiler.compile(...))`
3. **Same compilation, two outputs** — no duplicate parsing logic

### Completion Criteria

- [x] `ir_to_text()` renders IR to human-readable text at all 3 fidelity levels
- [x] Low fidelity: compact output with opcodes and ⊕ markers
- [x] Medium fidelity: full signatures with keywords
- [x] High fidelity: indented with behavior markers in braces
- [x] `flags_to_markers()` converts IR flags back to ⊕ notation
- [x] Import rendering at all fidelity levels
- [x] Class/method/field rendering at all fidelity levels
- [x] Round-trip test: compile sample → render → matches expected output format
- [x] Fidelity comparison: same IR at Low vs Medium vs High shows progressive detail
- [x] `cargo clippy --all-targets -- -D warnings` passes

---


Status: ✅ Complete — implemented 2026-06-08

### Goal

Separate language-specific and framework-specific logic into pluggable layers that emit additional ops on top of the Core IR.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/layers/mod.rs` | `LanguageLayer`, `MetaLayer` traits |
| `src/ir/layers/typescript.rs` | `TypeScriptLayer` implementation |
| `src/ir/layers/csharp.rs` | `CSharpLayer` implementation |
| `src/ir/layers/angular.rs` | `AngularMetaLayer` (refactored from `angular_meta/`) |
| `src/ir/layers/patterns.rs` | `PatternRecognizer` — Layer 4 (additive) |

### Completion Criteria

- [x] `LanguageLayer` trait defined with `process_capture()` and `finalize()`
- [x] `MetaLayer` trait defined with `extract()`
- [x] `PatternRecognizer` trait defined with `recognize()`
- [x] `LayerContext` struct with current_class, current_method, symbol_table, source, fidelity
- [x] `TypeScriptLayer` implements `LanguageLayer` — extracts extends/implements/async/static/export
- [x] `CSharpLayer` implements `LanguageLayer` — extracts inheritance, interfaces, flags
- [x] `AngularMetaLayer` implements `MetaLayer` — wraps existing angular_meta logic, emits CoreOps
- [x] `CodePatternRecognizer` implements `PatternRecognizer` — detects CTOR, OBSERVABLE, getter/setter
- [x] `IRCompiler` module declarations updated to expose `layers` submodule
- [x] Unit tests: TypeScript extends → `EXT` op in output
- [x] Unit tests: TypeScript implements → `IMPL` ops in output
- [x] Unit tests: TS async/static flags emitted correctly
- [x] Unit tests: C# inheritance with colon syntax → `EXT` + `IMPL` ops
- [x] Unit tests: C# abstract/public class flags emitted
- [x] Unit tests: Constructor injection pattern → `CTOR` flag
- [x] Unit tests: Observable pattern → `OBSERVABLE` flag
- [x] Unit tests: Getter/Setter pattern → `GETTER`/`SETTER` flags
- [x] Unit tests: Unrecognized patterns pass through unchanged
- [x] Unit tests: Angular meta-layer returns empty for non-Angular source
- [x] `cargo clippy --all-targets -- -D warnings` passes
- [x] All 504 existing tests pass with 0 failures

---

## 10. Phase G: Integration & MCP Tools

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Wire the IR system into the MCP tool interface, maintaining backward compatibility.

### Deliverables

| File | Description |
|------|-------------|
| `src/mcp/tools.rs` (modified) | New tool definitions for IR operations |
| `src/ir/mod.rs` (updated) | Public API for MCP integration |

### New MCP Tools

#### `compress_code_context` (upgraded, backward compatible)

```json
// Output: IR + pretty (new shape, but includes pretty for compat)
{
  "pretty": "// compressed text output (existing format)...",
  "ir": [
    ["DEF_C", "C1", "SampleService"],
    ["DEF_M", "C1", "M1", "processComplexData"],
    ["SIG", "M1", "P1", "$s", "payload"],
    ["RET", "M1", "$b"],
    ["FLAGS", "M1", "IF"]
  ],
  "v": 1,
  "file": "α1"
}
```

#### `delta_code_context` (new — replaces text-based diff)

```json
// Output: delta envelope
{
  "file": "α1",
  "from": 5,
  "to": 6,
  "ops": {
    "+": [["DEF_M", "C1", "M3", "newMethod"], "..."],
    "~": [{"k": ["DEF_M", "C1", "M1"], "r": ["DEF_M", "C1", "M1", "renamedMethod"]}],
    "-": [["DEF_M", "C1", "M4"]]
  }
}
```

#### `apply_delta` (new — client-side state update)

```json
// Output: confirmation + updated pretty
{
  "ok": true,
  "newVersion": 6,
  "pretty": "// re-rendered after delta application..."
}
```

### Completion Criteria

- [x] `compress_code_context` tool output includes `ir` field alongside `pretty`
- [x] `delta_code_context` tool computes and returns `IRDelta` envelope
- [x] `apply_delta` tool applies delta and returns updated state
- [x] Backward compatibility: existing `compress_code_context` `pretty` field is byte-identical to current output
- [x] MCP tool schemas updated for new tools
- [x] Integration test: compress → edit → delta → apply → render matches expected
- [x] `cargo clippy --all-targets -- -D warnings` passes

### Implementation Summary

**Files modified:**

| File | Change |
|------|--------|
| `src/mcp/tools.rs` | Added `delta_code_context` and `apply_delta` tool definitions + handlers; upgraded `compress_code_context` to also emit `ir` field; added `compile_file_ir()` helper. |
| `src/mcp/state.rs` | Added `ir_context: ContextState` field to `McpState` for in-session IR delta tracking. |

**Files added:**

| File | Purpose |
|------|---------|
| `src/tests/ir/integration.rs` | Phase G end-to-end tests. |
| `src/tests/ir/mod.rs` | Test module wiring. |

---

## 11. Phase H: Positional Encoding & Advanced Compression

**Status: ✅ Complete** — implemented 2026-06-08

### Goal

Maximum wire-size compression via two complementary mechanisms:

1. **Positional encoding** — strip the redundant opcode string from every tuple (the client consults the static schema in §14 to know which opcode a given tuple belongs to).
2. **Pattern compression** — *consume* recognised multi-op patterns (constructor + params + INJECTS, async + Promise, getter, setter, override, …) and emit a single compact `PAT_*` op instead.

### Deliverables

| File | Description |
|------|-------------|
| `src/ir/positional.rs` | `PositionalConfig`, encode/decode, `ir_to_positional_wire`, `estimate_savings`, `verify_round_trip` |
| `src/ir/patterns.rs` | `PatternOp`, `CompressingPatternRecognizer`, `CompressionStats`, `CompressedItem` |
| `src/tests/ir/positional.rs` | 35 tests: encode/decode round-trip, config, arity, wire, savings |
| `src/tests/ir/patterns.rs` | 30 tests: round-trip, pattern matching, compression stats, multi-pattern, passthrough |

### 11.1 Positional Encoding

Positional encoding is **not a separate wire format** — it is the same JSON tuple shape produced by `wire::op_to_tuple`, with the opcode string **stripped** from position 0. The client knows the static schema (see §14) and uses the index position to determine meaning.

```
// Before (Layer 1 named IR — opcode included):
["DEF_M", "C1", "M1", "processComplexData"]
//   5 chars  2     2      19

// After (Layer 4 positional — opcode stripped, schema-aware):
["C1", "M1", "processComplexData"]
//   2     2      19
```

Two configurations:

| `PositionalConfig` | Output | Use case |
|---|---|---|
| `stripped()` (default) | Operands only, no opcode | Maximum compression; client uses schema |
| `tagged()`            | `[opcode, ...operands]` | Debug / mixed streams / first-time schema bootstrap |

#### API Surface

```rust
// src/ir/positional.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PositionalConfig {
    pub tagged: bool,
}

impl PositionalConfig {
    pub fn stripped() -> Self { Self { tagged: false } }
    pub fn tagged() -> Self { Self { tagged: true } }
}

pub fn encode_op(op: &CoreOp, config: PositionalConfig) -> Vec<String>;
pub fn decode_op(opcode: &str, operands: &[String]) -> Option<CoreOp>;
pub fn encode_stream(ops: &[CoreOp], config: PositionalConfig) -> Vec<Vec<String>>;
pub fn ir_to_positional_wire(
    file_id: &str, version: u64, ops: &[CoreOp], config: PositionalConfig
) -> Value;
pub fn estimate_savings(ops: &[CoreOp]) -> (usize, usize); // (named, positional) chars
pub fn positional_char_count(ops: &[CoreOp], config: PositionalConfig) -> usize;
pub fn verify_round_trip(ops: &[CoreOp], tagged: &[Vec<String>]) -> Option<usize>;
```

#### Wire Format

```json
{
  "file": "α1",
  "v": 3,
  "encoding": "positional",
  "ir": [
    ["C1", "SampleService"],
    ["C1", "M1", "processComplexData"],
    ["M1", "P1", "$s", "payload"],
    ["M1", "$b"]
  ]
}
```

### 11.2 Pattern Compression

The Phase F `layers::patterns::CodePatternRecognizer` is **additive** — it emits a `FLAGS` op (CTOR / OBSERVABLE / GETTER / SETTER) alongside the original instructions. The Phase H `CompressingPatternRecognizer` is **consumptive**: when it recognises a pattern, it *replaces* the N source `CoreOp`s with a single compact `PatternOp`.

This is the actual wire-size-reducing pass called for in the spec.

#### Recognised Patterns

| Pattern name | Source ops consumed | Emitted wire form |
|---|---|---|
| `PAT_CTOR` | `DEF_M(constructor) + Param* + RET + INJECTS` | `["PAT", "CTOR", class_id, method_id, dep1, ...]` |
| `PAT_EMPTY_CTOR` | `DEF_M(constructor) + RET` (no params, no injects) | `["PAT", "EMPTY_CTOR", class_id, method_id]` |
| `PAT_OBSERVABLE` | `DEF_M + RET($P\|Observable) + FLAGS(ASYNC)` | `["PAT", "OBSERVABLE", class_id, method_id, return_type]` |
| `PAT_PROMISE` | `DEF_M + RET($P)` (no ASYNC) | `["PAT", "PROMISE", class_id, method_id, return_type]` |
| `PAT_GETTER` | `DEF_M(get X) [+ RET]` | `["PAT", "GETTER", class_id, method_id, property]` |
| `PAT_SETTER` | `DEF_M(set X) [+ Param] [+ RET]` | `["PAT", "SETTER", class_id, method_id, property]` |
| `PAT_OVERRIDE` | `DEF_M + FLAGS(OVERRIDE)` | `["PAT", "OVERRIDE", class_id, method_id]` |

#### API Surface

```rust
// src/ir/patterns.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternOp {
    Constructor { class_id: String, method_id: String, deps: Vec<String> },
    EmptyConstructor { class_id: String, method_id: String },
    Observable { class_id: String, method_id: String, return_type: String },
    Promise { class_id: String, method_id: String, return_type: String },
    Getter { class_id: String, method_id: String, property: String },
    Setter { class_id: String, method_id: String, property: String },
    Override { class_id: String, method_id: String },
}

impl PatternOp {
    pub fn to_tuple(&self) -> Vec<String>;
    pub fn from_tuple(tuple: &[String]) -> Option<Self>;
    pub fn name(&self) -> &'static str;
    pub fn consumed(&self) -> usize;
}

#[derive(Debug, Clone, Default)]
pub struct CompressingPatternRecognizer;

impl CompressingPatternRecognizer {
    pub fn new() -> Self;
    /// Compress and return only the recognised patterns + stats.
    pub fn compress(&self, instructions: &[CoreOp]) -> (Vec<PatternOp>, CompressionStats);
    /// Compress and merge: passthroughs are preserved at their original
    /// relative positions; matched spans are replaced by `PatternOp`s.
    pub fn compress_merged(&self, instructions: &[CoreOp]) -> Vec<CompressedItem>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompressionStats {
    pub source_ops: usize,
    pub output_ops: usize,
    pub ratio: f64,        // source / output; >1 means compression
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompressedItem {
    Passthrough(CoreOp),
    Pattern(PatternOp),
}
```

### 11.3 Design Decisions

1. **Phase F recognizer stays additive** — `layers::patterns::CodePatternRecognizer` remains in place and is the right tool when callers want to *annotate* the IR (e.g., for human-readable output). Phase H adds a *new* recognizer that actually *consumes* instructions to save bytes.

2. **Positional encoding is schema-driven** — The static schema in §14 (already sent once per session in the Layer 1 protocol) is reused. The client decodes by position, not by string match. The `tagged` config is provided for heterogeneous streams (e.g., logs) where the consumer does not know the schema a priori.

3. **Pattern ops start with `"PAT"`** — Even when positional encoding is in effect, pattern ops are *always* tagged (the very first operand is the pattern name). This is necessary because the client cannot infer the pattern from position alone — patterns can appear at any index in the stream. The arity varies per pattern; the client consults §14 for per-pattern arity rules.

4. **No regression on unknowns** — Patterns that don't match pass through unchanged. Both `compress()` (returns only matches + stats) and `compress_merged()` (returns `CompressedItem::Passthrough` for unmatched spans) preserve every original instruction.

5. **Opcode arity is consulted at decode** — `decode_op` rejects tuples whose length doesn't match the schema's arity. Variadic opcodes (`FLAGS`, `FLAGS_C`, `INJECTS`) accept any length ≥ 2.

### 11.4 Example: Before and After

```rust
// Source (Layer 1 — named):
let ops = vec![
    DefClass("C1".into(), "Service".into()),
    DefMethod("C1".into(), "M1".into(), "constructor".into()),
    Param("M1".into(), "P1".into(), "$s".into(), "dep".into()),
    Return("M1".into(), "$v".into()),
    Injects("C1".into(), vec!["S1".into()]),
];

// Phase H positional + pattern compression:
//   PatternOp::Constructor { class_id: "C1", method_id: "M1", deps: ["S1"] }
//   + passthrough: DefClass("C1", "Service")

// Wire output (named, untagged):
//   ["DEF_C","C1","Service"], ["PAT","CTOR","C1","M1","S1"]

// Wire output (stripped positional):
//   ["C1","Service"], ["PAT","CTOR","C1","M1","S1"]
```

### Completion Criteria

- [x] Positional encoding can serialize/deserialize IR without keys
- [x] Client can decode positional format using static schema
- [x] `PositionalConfig` with `stripped()` and `tagged()` variants
- [x] `encode_op` / `decode_op` / `encode_stream` / `ir_to_positional_wire` round-trip
- [x] `estimate_savings` returns character counts; positional is shorter than named
- [x] `verify_round_trip` detects any divergence
- [x] Arity validation on decode (fixed arity: exact match; variadic: ≥2)
- [x] Pattern recognizer detects constructor injection pattern (`PAT_CTOR`)
- [x] Pattern recognizer detects empty constructor pattern (`PAT_EMPTY_CTOR`)
- [x] Pattern recognizer detects observable stream pattern (`PAT_OBSERVABLE`)
- [x] Pattern recognizer detects promise pattern (`PAT_PROMISE`)
- [x] Pattern recognizer detects getter / setter accessor patterns
- [x] Pattern recognizer detects override pattern (`PAT_OVERRIDE`)
- [x] Pattern compression reduces instruction count for recognized patterns
- [x] Unrecognized patterns pass through unchanged (zero regression)
- [x] Unit tests: inject constructor → single `PAT_CTOR` op
- [x] Unit tests: unrecognized patterns pass through unchanged
- [x] 78 new tests added (35 positional + 30 pattern + 13 new module-wiring)
- [x] `cargo clippy --all-targets -- -D warnings` passes
- [x] All 582 tests pass (up from 504 at start of Phase H)

### Implementation Summary

**Files added:**

| File | Purpose |
|------|---------|
| `src/ir/positional.rs` | Positional encode/decode, wire, savings, verify. |
| `src/ir/patterns.rs`   | `PatternOp` enum + `CompressingPatternRecognizer`. |
| `src/tests/ir/positional.rs` | 35 tests for the positional module. |
| `src/tests/ir/patterns.rs`   | 30 tests for the pattern recognizer. |

**Files modified:**

| File | Change |
|------|--------|
| `src/ir/mod.rs` | Added `pub mod positional;` and `pub mod patterns;`, re-exported their public types. |
| `docs/COMPILER_IR.md` | Updated status banner and expanded §11 with implementation summary, API surface, and design decisions, plus the Phase G and F sections. |
| `docs/COMPILER_IR.md` (this file) | Status banner updated; §11 expanded. |

**Test counts at completion of Phase H:**

| Module | Tests | Status |
|---|---:|---|
| `ir::opcodes` | 14 | ✅ |
| `ir::compiler` | 12 | ✅ |
| `ir::render` | 17 | ✅ |
| `ir::wire` | 26 | ✅ |
| `ir::symbol_table` | 30 | ✅ |
| `ir::delta` | 26 | ✅ |
| `ir::replay` | 39 | ✅ |
| `ir::layers` | 19 | ✅ |
| `ir::tests::integration` | 4 | ✅ |
| `ir::positional` (new) | 35 | ✅ |
| `ir::patterns` (new) | 30 | ✅ |
| `ir::layers::patterns` | 7 | ✅ |
| **Phase H delta** | **+78** | ✅ |
| **Project total** | **582** | ✅ all green |

---

## 12. Migration Map

How each existing module maps to the new IR system:

| Existing Component | Role in IR System | Migration Path |
|---|---|---|
| `compression/opcodes.rs` (32 primitives) | Core IR type constants | Becomes `ir/opcodes.rs` — primitives become `CoreOp` type constants |
| `compression/markers.rs` (⊕guard etc.) | Replaced by `FLAGS` opcodes | `FLAGS("M1", ["IF","LOOP","RET"])` replaces `⊕guard ⊕loop ⊕⇒` |
| `compaction/*` | IR emitters | Each function emits `Vec<CoreOp>` instead of formatted strings |
| `diff/snapshot.rs` (CapturedStructure) | Replaced by `CompiledIR` | `CapturedClass` → `Vec<CoreOp>` instructions |
| `diff/differ.rs` (diff_snapshots) | Replaced by `DeltaComputer` | Text diff → instruction-level delta |
| `diff/action.rs` (DiffAction) | Replaced by `IRDelta` ops | `DiffKind::Added` → `DeltaOp::Add(CoreOp)` |
| `dictionary/symbol.rs` | Subsumed by `GlobalSymbolTable` | Runtime opcodes → typed aliases |
| `dictionary/path.rs` | Subsumed by `GlobalSymbolTable` | File aliases → file membership tracking |
| `angular_meta/*` | Layer 3 Meta-Layer | Implements `MetaLayer` trait |
| `decompression/*` | Replaced by `ir_to_text()` | Text expansion → IR rendering |
| `compression/pipeline.rs` | Wrapped by `IRCompiler` | One-shot compression → compiled state |
| `cache.rs` (LocalStateCache) | Enhanced by version tracking | Hash check → version check |

---

## 13. Wire Protocol Reference

### Instruction Tuple Format

Every instruction is a JSON array where position determines meaning:

```
[op0, op1, op2, ...]
 ^     ^     ^
 |     |     └── operand (type depends on opcode)
 |     └── target/class/method id
 └── opcode (always first element when tagged)
```

### Complete Opcode Reference

| Opcode | Arity | Operands | Example |
|--------|------:|----------|---------|
| `DEF_C` | 3 | id, name | `["DEF_C","C1","SampleService"]` |
| `DEF_M` | 4 | class_id, id, name | `["DEF_M","C1","M1","process"]` |
| `DEF_F` | 4 | class_id, id, name | `["DEF_F","C1","f1","payload"]` |
| `DEF_I` | 3 | id, name | `["DEF_I","I1","IMyService"]` |
| `SIG` | 5 | method_id, param_id, type, name | `["SIG","M1","P1","$s","payload"]` |
| `RET` | 3 | method_id, type | `["RET","M1","$b"]` |
| `FIELD_T` | 3 | field_id, type | `["FIELD_T","f1","$n"]` |
| `FLAGS` | 3+ | target_id, flag1, ... | `["FLAGS","M1","IF","LOOP"]` |
| `FLAGS_C` | 3+ | class_id, flag1, ... | `["FLAGS_C","C1","EXPORT"]` |
| `EXT` | 3 | child_id, parent_id | `["EXT","C1","C2"]` |
| `IMPL` | 3 | class_id, iface_id | `["IMPL","C1","I1"]` |
| `INJECTS` | 3+ | class_id, dep1, ... | `["INJECTS","C1","S1","S2"]` |
| `IMP` | 4 | alias, module, named | `["IMP","$im","rxjs","map"]` |
| `TYPE` | 3 | alias, original | `["TYPE","$uid","UserId"]` |
| `PAT` | 3+ | pattern_name, class_id, method_id, ... | `["PAT","CTOR","C1","M1","S1"]` |

### Delta Wire Format

```json
{
  "file": "<path_alias>",
  "from": <baseline_version>,
  "to": <current_version>,
  "ops": {
    "+": [ [<instruction>, ...], ... ],
    "~": [ {"k": [<key_tuple>], "r": [<replacement>]}, ... ],
    "-": [ [<instruction>, ...], ... ]
  }
}
```

### Type Opcode Reference

| Opcode | Type | Inherited From |
|--------|------|----------------|
| `$s` | string | `PRIMITIVE_OPCODES["$s"]` |
| `$n` | number | `PRIMITIVE_OPCODES["$n"]` |
| `$b` | boolean | `PRIMITIVE_OPCODES["$b"]` |
| `$v` | void | `PRIMITIVE_OPCODES["$v"]` |
| `$T` | true | `PRIMITIVE_OPCODES["$T"]` |
| `$F` | false | `PRIMITIVE_OPCODES["$F"]` |
| `$nl` | null | `PRIMITIVE_OPCODES["$nl"]` |
| `$ud` | undefined | `PRIMITIVE_OPCODES["$ud"]` |

### Flag Reference

| Flag | Meaning | Replaces |
|------|---------|----------|
| `IF` | Conditional branch (if/switch) | `⊕guard` |
| `LOOP` | Loop construct (for/while/do) | `⊕loop` |
| `RET` | Return statement | `⊕⇒` |
| `THROW` | Throw/exception | `⊕!` |
| `ASYNC` | Async function | — (keyword preserved) |
| `GEN` | Generator function | — |
| `EXPORT` | Exported symbol | `$e` (existing opcode) |
| `STATIC` | Static member | `$st` (existing opcode) |
| `PRIVATE` | Private visibility | `$pv` (existing opcode) |
| `PROTECTED` | Protected visibility | `$pd` (existing opcode) |
| `ABSTRACT` | Abstract class/method | — |

### Pattern Reference (Phase H)

| Pattern | Source | Wire form |
|---|---|---|
| `CTOR` | `DEF_M(constructor) + Param* + RET + INJECTS` | `["PAT", "CTOR", class_id, method_id, dep1, ...]` |
| `EMPTY_CTOR` | `DEF_M(constructor) + RET` | `["PAT", "EMPTY_CTOR", class_id, method_id]` |
| `OBSERVABLE` | `DEF_M + RET($P) + FLAGS(ASYNC)` | `["PAT", "OBSERVABLE", class_id, method_id, return_type]` |
| `PROMISE` | `DEF_M + RET($P)` | `["PAT", "PROMISE", class_id, method_id, return_type]` |
| `GETTER` | `DEF_M(get X)` | `["PAT", "GETTER", class_id, method_id, property]` |
| `SETTER` | `DEF_M(set X)` | `["PAT", "SETTER", class_id, method_id, property]` |
| `OVERRIDE` | `DEF_M + FLAGS(OVERRIDE)` | `["PAT", "OVERRIDE", class_id, method_id]` |

---

## 14. Static Schema Definition

Sent once per session, defines the opcode vocabulary for all subsequent communication:

```json
{
  "schema_version": 1,
  "opcodes": {
    "DEF_C":    { "arity": 3, "args": ["id", "name"] },
    "DEF_M":    { "arity": 4, "args": ["class_id", "id", "name"] },
    "DEF_F":    { "arity": 4, "args": ["class_id", "id", "name"] },
    "DEF_I":    { "arity": 3, "args": ["id", "name"] },
    "SIG":      { "arity": 5, "args": ["method_id", "param_id", "type", "name"] },
    "RET":      { "arity": 3, "args": ["method_id", "type"] },
    "FIELD_T":  { "arity": 3, "args": ["field_id", "type"] },
    "FLAGS":    { "arity": -1, "args": ["target_id", "flags..."] },
    "FLAGS_C":  { "arity": -1, "args": ["class_id", "flags..."] },
    "EXT":      { "arity": 3, "args": ["child_id", "parent_id"] },
    "IMPL":     { "arity": 3, "args": ["class_id", "iface_id"] },
    "INJECTS":  { "arity": -1, "args": ["class_id", "deps..."] },
    "IMP":      { "arity": 4, "args": ["alias", "module", "named"] },
    "TYPE":     { "arity": 3, "args": ["alias", "original"] },
    "PAT":      { "arity": -1, "args": ["pattern_name", "class_id", "method_id", "extra..."] }
  },
  "patterns": {
    "CTOR":        { "arity": -1, "args": ["class_id", "method_id", "dep1..."] },
    "EMPTY_CTOR":  { "arity": 3,  "args": ["class_id", "method_id"] },
    "OBSERVABLE":  { "arity": 4,  "args": ["class_id", "method_id", "return_type"] },
    "PROMISE":     { "arity": 4,  "args": ["class_id", "method_id", "return_type"] },
    "GETTER":      { "arity": 4,  "args": ["class_id", "method_id", "property"] },
    "SETTER":      { "arity": 4,  "args": ["class_id", "method_id", "property"] },
    "OVERRIDE":    { "arity": 3,  "args": ["class_id", "method_id"] }
  },
  "flags": [
    "IF", "LOOP", "RET", "THROW", "ASYNC", "GEN",
    "EXPORT", "STATIC", "PRIVATE", "PROTECTED", "ABSTRACT"
  ],
  "types": ["$s", "$n", "$b", "$v", "$T", "$F", "$nl", "$ud"],
  "primitives": {
    "$e": "export", "$c": "class", "$a": "async", "$P": "Promise",
    "$b": "boolean", "$s": "string", "$n": "number", "$v": "void",
    "$T": "true", "$F": "false", "$pu": "public", "$pv": "private",
    "$pd": "protected", "$st": "static", "$nw": "new", "$r": "return",
    "$t": "throw", "$E": "Error", "$k": "const", "$l": "let",
    "$i": "if", "$fr": "for", "$w": "while", "$h": "this",
    "$x": "extends", "$m": "implements", "$im": "import", "$fm": "from",
    "$if": "interface", "$ty": "type", "$fn": "function", "$ctor": "constructor",
    "$ud": "undefined", "$nl": "null"
  }
}
```

`arity: -1` indicates variadic instruction length. The new `patterns` and `PAT` opcode sections were added in Phase H.

---

## 15. Phase Dependencies & Timeline

```
Phase A: IR Core ─────────────────────────────────────┐
  (instruction types, compiler, render)                │
                                                       ├──→ Phase E: IR/Pretty Separation
Phase B: Global Symbol Table ──────────────────────────┤     (two render paths)
  (unified registry)                                   │
                                                       ├──→ Phase F: Layered Encoding
Phase C: Delta Transport ──────────────────────────────┤     (language + meta layers)
  (diff engine, delta envelope)                        │
                                                       │
Phase D: State Replay ─────────────────────────────────┘
  (apply, version chain, catch-up)                     │
                                                       │
Phase G: Integration & MCP Tools ──────────────────────┤
  (tool definitions, backward compat)                  │
                                                       │
Phase H: Positional Encoding & Patterns ───────────────┘
  (maximum compression, pattern recognition) ─── ✅ DONE
```

### Estimated vs Actual Effort

| Phase | Estimated | Actual | Status |
|-------|----------:|-------:|--------|
| **A: IR Core** | 3-4 days | ~1 day | ✅ |
| **B: Global Symbol Table** | 2-3 days | ~0.5 day | ✅ |
| **C: Delta Transport** | 2-3 days | ~0.5 day | ✅ |
| **D: State Replay** | 2-3 days | ~0.5 day | ✅ |
| **E: IR/Pretty Separation** | 2-3 days | ~0.25 day | ✅ |
| **F: Layered Encoding** | 3-4 days | ~1 day | ✅ |
| **G: Integration & MCP Tools** | 2-3 days | ~0.5 day | ✅ |
| **H: Positional Encoding** | 2-3 days | ~0.5 day | ✅ |
| **Total** | **18-26 days** | **~4.75 days** | ✅ |

### Minimum Viable Protocol

Phases **A → C → D → G** form the minimum viable path to a working delta transport protocol. Phases E, F, H are enhancements that have now all been delivered.

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.
