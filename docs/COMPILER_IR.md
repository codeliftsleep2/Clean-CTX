# Clean-CTX — Compiler IR: Structured State Protocol

> **Owner:** Compiler IR spec + delta transport · **Status:** Living reference
> **Version:** 1.0.0 (Implemented) · **Last updated:** 2026-06-18
> **Status:** All phases A–H implemented and deployed in production.
>
> **Test coverage:** see `docs/CHANGELOG.md` for the current workspace test count (this document does not duplicate it).
>
> **Living document.** This document describes the Compiler IR subsystem as implemented. It replaces the original proposal/spec with accurate details about the actual wire formats, module structure, MCP integration, and test coverage.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Motivation & Architecture](#2-motivation--architecture)
3. [The Pipeline](#3-the-pipeline)
4. [Module Structure](#4-module-structure)
5. [CoreOp Instruction Set](#5-coreop-instruction-set)
6. [Wire Formats](#6-wire-formats)
7. [Delta Transport](#7-delta-transport)
8. [State Replay](#8-state-replay)
9. [Layered Encoding](#9-layered-encoding)
10. [MCP Tool Integration](#10-mcp-tool-integration)
11. [Test Coverage](#11-test-coverage)

---

## 1. Overview

The Compiler IR subsystem translates source code (TypeScript, C#, Rust, Java) into a structured intermediate representation — a stream of `CoreOp` instructions. This IR serves as the canonical source of truth for all subsequent operations:

- **Rendering**: IR → human-readable compressed text (3 fidelity levels)
- **Delta transport**: IR → instruction-level diffs → state machine application
- **Wire encoding**: 6 formats (named, positional, tagged, string_table, hierarchical, binary)
- **Language layers**: Pluggable passes for TS, C#, Rust, Java
- **Meta layers**: Pluggable passes for Angular, Spring Boot

The IR subsystem replaces the earlier text-only compression pipeline with a structured, stateful approach. Instead of re-parsing the full source on every interaction, the IR enables incremental updates via delta transport.

### Key Design Decisions

- **IR-first output**: `provide_code_context` renders hierarchical IR text as primary output; text pipeline is fallback
- **Backward compatible**: `ir_to_text()` can produce byte-identical output to the legacy text pipeline
- **Render-time fidelity**: The same IR compiles once and renders at Low/Medium/High (no re-parse)
- **Deterministic**: BTreeMap-based indexing for delta comparison ensures stable ordering
- **Cross-file symbol tracking**: GlobalSymbolTable registers classes/methods across files with monotonic versioning

---

## 2. Motivation & Architecture

### 2.1 The Problem

The original Clean-CTX was a one-shot text compression pipeline:

```
Source → Tree-sitter AST → Fidelity Filter → Opcode Encode → Text Output
```

Every compression required a full re-parse. The diff system computed snapshots and emitted human-readable diff lines — not machine-replayable instruction streams. There was no concept of incremental state application.

### 2.2 4-Layer Architecture

The IR subsystem introduces a layered approach:

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 4: Pattern Recognition + Positional Encoding         │
│  Additive patterns (CTOR/OBSERVABLE/GETTER/SETTER)          │
│  Consumptive patterns (CompressingPatternRecognizer → PAT)  │
│                                                             │
│  ["PAT","CTOR","C1","M1"]  or  ["C1","M1","processData"]   │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Meta-Layer (Framework-Specific)                   │
│  Angular @ markers, Spring Boot Φ markers                   │
│                                                             │
│  ["@cmp","C1","AppComponent"]  ["Φrest:","C1","Controller"] │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Language Layer (TS, C#, Rust, Java)               │
│  Language-specific ops: async, static, export, public, etc. │
│                                                             │
│  ["ASYNC","M1"]  ["EXPORT","C1"]  ["STATIC","F1"]          │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Core IR (Language-Agnostic)                       │
│  Universal instruction set — every language compiles here   │
│                                                             │
│  DEF_C, DEF_M, DEF_F, SIG, RET, FLAGS, EXT, IMP, INJECTS    │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 Pipeline

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
┌───────────────────────┐
│   IRCompiler          │  Translates captures → Vec<CoreOp>
│   1. Core IR emission │  Runs all 4 layers
│   2. Language layers  │
│   3. Meta-layer pass  │
│   4. Pattern recog.   │
│   5. Forward alias    │
│      resolution       │
└─────────┬─────────────┘
          │ CompiledIR { file_id, instructions, version }
          │
     ┌────┴────────────────────────────┐
     ▼                                 ▼
┌──────────────┐              ┌──────────────────┐
│ IR → Wire    │              │ IR → LLM Text    │
│ (6 formats)  │              │ (SCHEMA v2)      │
│ (delta,      │              └────────┬─────────┘
│  transport,  │                       ▼
│  positional) │              ┌──────────────────┐
└──────┬───────┘              │ HierarchicalIR   │
       │                      │ → compact        │
       ▼                      │   LLM output     │
┌──────────────┐              └──────────────────┘
│ Delta        │
│ Transport    │
│ (IRDelta)    │
└──────┬───────┘
       │
       ▼
┌─────────────────────┐
│   State Replay      │  Apply delta ops to ContextState
│   (Apply + Render)  │
└─────────────────────┘
```

---

## 3. The Pipeline

### 3.1 Compilation (Source → IR)

The IR compiler (`IRCompiler::compile` in `src/ir/compiler.rs`) reuses the existing tree-sitter capture pipeline (`run_capture_pipeline`) but replaces text formatting with instruction emission:

```
1. Tree-sitter parse (existing — no change)
2. Capture walk (existing — no change)
3. Core IR emission (replaces build_output_lines)
4. Language layer translation (TypeScriptLayer, CSharpLayer, RustLayer, JavaLayer)
5. Meta-layer pass (AngularMetaLayer, SpringMetaLayer)
6. Additive pattern recognition (CodePatternRecognizer — CTOR/OBSERVABLE/GETTER/SETTER)
7. Consumptive pattern compression (CompressingPatternRecognizer — PAT ops)
8. Forward alias resolution (resolve_forward_aliases)
```

The compiler maintains per-compilation state:
- `id_counter: u64` — monotonic instruction ID generator (F-31: u64 to avoid overflow)
- `current_method: Option<String>` — O(1) method tracking (F-27)
- `current_method_flags: Vec<String>` — flag accumulation (F-28)
- `current_class: Option<String>` — class context tracking

Methods/fields outside a class are skipped (F-29). Standalone functions (`func.root`, `arrow.root`) create synthetic classes.

### 3.2 Delta Computation (IR → Delta)

The delta engine (`DeltaComputer` in `src/ir/delta.rs`) computes instruction-level deltas between two `CompiledIR` states:

```
1. Index both IRs by primary key (opcode + IDs) via BTreeMap
2. Diff the indices:
   - In current but not baseline → additions
   - In baseline but not current → deletions
   - In both but different tuples → modifications
3. Emit IRDelta envelope
```

### 3.3 State Replay (Delta → Updated State)

The state machine (`ContextState` in `src/ir/replay.rs`) applies delta ops:

```
1. Validate version chain (file.version must match delta.from)
2. Validate monotonic version (delta.to > delta.from)
3. Apply deletions (swap_remove + index update)
4. Apply modifications (full replacement or field-patch)
5. Apply additions (append + index update, reject duplicates)
6. Bump version
```

---

## 4. Module Structure

```
src/ir/
├── mod.rs                # Module declarations + public re-exports
├── opcodes.rs            # CoreOp enum (15 variants), flag constants, arity table
├── compiler.rs           # IRCompiler struct, compile() method, CompileError
├── compiler_methods.rs   # MethodSig, parse_method_sig, emit_method_ir, emit_import_ir,
│                         # resolve_forward_aliases
├── render.rs             # ir_to_text(), ir_to_text_ops() — fidelity-aware text rendering
├── render_llm.rs         # render_hierarchical_for_llm() — SCHEMA v2 LLM-optimized output
├── wire.rs               # op_to_tuple(), tuple_to_op(), ir_to_wire(), wire_to_ir()
├── delta.rs              # IRDelta, DeltaOps, ModOp, FieldPatch, DeltaComputer,
│                         # compact_encode, compact_decode, primary_key_from_tuple
├── replay.rs             # ContextState, FileState, DeltaError
├── symbol_table.rs       # GlobalSymbolTable, SymbolEntry, SymbolKind
├── string_table.rs       # StringTable, ir_to_string_table_wire (integer-indexed IR)
├── hierarchical.rs       # HierarchicalIR, ClassNode, MethodNode, FieldNode,
│                         # ir_to_hierarchical, hierarchical_to_ir
├── positional.rs         # PositionalConfig, encode/decode, ir_to_positional_wire
├── patterns.rs           # PatternOp, CompressingPatternRecognizer, CompressionStats
├── binary_wire.rs        # Binary encode/decode, BLOB format
└── layers/
    ├── mod.rs            # LanguageLayer, MetaLayer, PatternRecognizer traits,
    │                     # LayerContext struct
    ├── typescript.rs     # TypeScriptLayer — extends, implements, async, export, static
    ├── csharp.rs         # CSharpLayer — inheritance, interfaces, abstract/public
    ├── rust.rs           # RustLayer — derives, generics, cfg, self kind, impl relationships
    ├── java.rs           # JavaLayer — extends, implements, abstract/static/public,
    │                     # constructor detection, Jakarta/Spring annotation patterns
    ├── angular.rs        # AngularMetaLayer — wraps existing angular_meta logic
    ├── spring.rs         # SpringMetaLayer — Spring Boot annotation extraction
    └── patterns.rs       # CodePatternRecognizer — CTOR, OBSERVABLE, GETTER, SETTER
```

---

## 5. CoreOp Instruction Set

**File:** `src/ir/opcodes.rs`

15 variants (14 structural + 1 pattern):

```rust
pub enum CoreOp {
    // ── Structural Definitions ────────────
    DefClass(String, String),       // DEF_C  class_id, name
    DefMethod(String, String, String), // DEF_M  class_id, method_id, name
    DefField(String, String, String),  // DEF_F  class_id, field_id, name
    DefInterface(String, String),      // DEF_I  interface_id, name

    // ── Signatures & Types ───────────────
    Param(String, String, String, String), // SIG  method_id, param_id, type, name
    Return(String, String),                // RET  method_id, type
    FieldType(String, String),             // FIELD_T  field_id, type

    // ── Control Flow & Behavior ──────────
    Flags(String, Vec<String>),         // FLAGS target_id, flags...
    ClassFlags(String, Vec<String>),    // FLAGS_C class_id, flags...

    // ── Relationships ────────────────────
    Extends(String, String),            // EXT child_id, parent_id
    Implements(String, String),         // IMPL class_id, interface_id
    Injects(String, Vec<String>),       // INJECTS class_id, deps...

    // ── Imports ──────────────────────────
    Import(String, String, String),     // IMP alias, module, named_export

    // ── Type Aliases ─────────────────────
    TypeAlias(String, String),          // TYPE alias, original

    // ── Compressed Patterns (Phase H) ────
    Pattern(String, Vec<String>),      // PAT pattern_name, args...
}
```

**Flag constants:** `IF`, `LOOP`, `RET`, `THROW`, `ASYNC`, `GEN`, `EXPORT`, `STATIC`, `PRIVATE`, `PROTECTED`, `ABSTRACT`, `UNSAFE`

**Type opcodes:** `$s` (string), `$n` (number), `$b` (boolean), `$v` (void), `$T` (true), `$F` (false), `$nl` (null), `$ud` (undefined)

**Arity table** (`arity()`): Fixed arities (3–5) + variadic (-1) for FLAGS, FLAGS_C, INJECTS, PAT.

### Compiler Error Handling

```rust
pub enum CompileError {
    Capture(String),      // tree-sitter pipeline failure
    Layer(String),         // Language/Meta/Pattern layer error
    NoCaptures,            // source produced no captures (not fatal)
}
```

---

## 6. Wire Formats

The IR supports 6 wire formats, selected via the `encoding` parameter:

| Format | Encoding Key | Description | Savings |
|--------|-------------|-------------|---------|
| Named | `"named"` | JSON arrays with opcode strings | Baseline |
| Positional | `"positional"` | Stripped opcode, schema-aware | ~30% vs named |
| Tagged | `"tagged"` | Positional + opcode preserved | Debug/mixed |
| String Table | `"string_table"` | Integer-indexed string interning | ~40% vs named |
| Hierarchical | `"hierarchical"` | Class→method→param tree, no parent IDs | ~40-60% vs named |
| Binary | `"binary"` | Compact binary encoding with varints | ~50% vs named |

### Named Format (Default)

```json
{
  "file": "α1",
  "v": 1,
  "encoding": "named",
  "ir": [
    ["DEF_C", "C1", "UserService"],
    ["DEF_M", "C1", "M1", "processData"],
    ["SIG", "M1", "P1", "$s", "payload"],
    ["RET", "M1", "$b"]
  ]
}
```

### Hierarchical Format (Primary LLM Output)

The hierarchical format is the primary output of `provide_code_context`. It reorganizes the flat CoreOp stream into a class→method→param tree, eliminating all opcode strings and parent ID repetitions:

```json
{
  "encoding": "hierarchical",
  "file": "α1",
  "v": 1,
  "ir": {
    "c": [{
      "n": "C1", "nm": "UserService",
      "m": [{
        "n": "M1", "nm": "processData",
        "p": [["P1", "$s", "payload"]],
        "r": "$b"
      }],
      "f": [{"n": "F1", "nm": "userRepo", "tp": "UserRepository"}]
    }],
    "i": [["IM1", "./types", "User"]],
    "t": []
  }
}
```

### LLM-Optimized Text (SCHEMA v2)

The hierarchical IR is rendered to compact LLM-friendly text via `render_hierarchical_for_llm()`:

```
// SCHEMA v2  @=meta X=extends I=implements F=field M=method $=import →=scope fl:=flags cl:=class-flags P=pattern T=type-alias
// ── UserService ──
X BaseService
F userRepo:UserRepository
M processData(payload:$s):$b  fl:IF RET
```

---

## 7. Delta Transport

### IRDelta Envelope

```json
{
  "file": "α1",
  "from": 1,
  "to": 2,
  "ops": {
    "+": [["DEF_M", "C1", "M3", "newMethod"], ["RET", "M3", "$s"]],
    "~": [{"k": ["DEF_M","C1","M1"], "r": ["DEF_M","C1","M1","renamedMethod"]}],
    "-": [["DEF_M", "C1", "M2"]]
  }
}
```

### ModOp Formats

| Format | Description | Best For |
|--------|-------------|----------|
| Full replacement (`r`) | Complete instruction tuple | Major signature changes |
| Field patches (`d`) | Changed fields only (index:value pairs) | Minor edits (rename, type change) |

### Compact Delta Format

The `CompactDelta` struct provides an abbreviated wire format:
- `f` instead of `file`
- `"5→6"` version range instead of separate `from`/`to` fields
- Single-character opcode abbreviations (C=DEF_C, M=DEF_M, etc.)
- Field-patch encoding for all modifications

### Delta Application

`ContextState` in `src/ir/replay.rs` manages per-file IR state:
- `FileState`: ordered instruction tuples + primary-key index (HashMap)
- `ContextState`: per-file HashMap + global monotonic version
- `apply()`: validates → deletions → modifications → additions (order ensures correctness)
- `load_ir()`: bootstraps state from full CompiledIR
- `render_pretty()`: re-renders current state to text

### Error Handling

```rust
pub enum DeltaError {
    UnknownFile(String),
    VersionMismatch { expected: u64, got: u64 },
    SymbolNotFound(String),
    DuplicateSymbol(String),
    NonMonotonicVersion { from: u64, to: u64 },
}
```

---

## 8. State Replay

The state machine supports:

- **Multiple files**: Load IR for several files, apply deltas independently
- **Sequential deltas**: v1→v2→v3 applies correctly with monotonic version tracking
- **Error recovery**: Version mismatch, unknown file, and symbol-not-found errors are recoverable
- **Render after apply**: `render_pretty()` produces human-readable output from current state

### FileState Operations

| Operation | Description | O-complexity |
|-----------|-------------|--------------|
| `from_compiled` | Build from CompiledIR | O(n) |
| `remove_by_key` | swap_remove + index update | O(1) amortized |
| `replace_by_key` | In-place replacement + re-index | O(1) |
| `append` | Push + index insert (rejects dupes) | O(1) amortized |
| `contains_key` | Hash lookup | O(1) |

---

## 9. Layered Encoding

### Layer 2: Language Layers

| Language | File | Capabilities |
|----------|------|-------------|
| TypeScript | `layers/typescript.rs` | `extends`/`implements` extraction, `async`/`export`/`static` flags |
| C# | `layers/csharp.rs` | Colon-syntax inheritance, `abstract`/`public`/`static` class flags |
| Rust | `layers/rust.rs` | `derive` macros, generic params, `cfg` attributes, `impl` relationships, `self` kind, `unsafe` |
| Java | `layers/java.rs` | `extends`/`implements` with generics stripping, constructor detection, `abstract`/`static`/`private`/`protected` flags, Jakarta/Spring annotation patterns |

### Layer 3: Meta Layers

| Framework | File | Features |
|-----------|------|----------|
| Angular | `layers/angular.rs` | Wraps `angular_meta` module, emits `@cmp`/`@svc`/`@pipe` markers |
| Spring Boot | `layers/spring.rs` | Wraps `spring_meta` module, emits `Φrest:`/`Φsvc:` markers |

### Layer 4: Pattern Recognizers

**Additive** (`layers/patterns.rs` — `CodePatternRecognizer`):
- CTOR: constructor injection pattern
- OBSERVABLE: Observable return type
- GETTER: getter method pattern
- SETTER: setter method pattern

**Consumptive** (`patterns.rs` — `CompressingPatternRecognizer`):
- Collapses recognized patterns into single `PAT` ops (Phase H)
- Consumes source instructions to reduce wire size (~30% savings)

---

## 10. MCP Tool Integration

### Tools Exposed

| Tool | Handler | Description |
|------|---------|-------------|
| `compress_code_context` | `handle_compress_code_context` | IR-first compression with encoding selection |
| `delta_code_context` | `handle_delta_code_context` | IR-level delta computation |
| `delta_text_context` | `handle_delta_text_context` | Text-level delta (backward compat) |
| `apply_delta` | `handle_apply_delta` | Client-side state update |
| `provide_code_context` | `handle_provide_code_context` | Zero-touch entry point (auto-detect + delta) |
| `restore_context` | `handle_restore_context` | Force full re-compression |
| `context_history` | `handle_context_history` | Per-file delta history |
| `context_stats` | `handle_context_stats` | Session dashboard |

### Response Format (Phase 6 IR-first)

```json
{
  "content": [{ "type": "text", "text": "// SCHEMA v2 ..." }],
  "ir": { "encoding": "hierarchical", "file": "α1", "v": 1, "ir": { ... } },
  "pretty": { "encoding": "named", "file": "α1", "v": 1, "ir": [...] },
  "v": 1,
  "file": "α1",
  "_meta": {
    "fidelity": "low",
    "strategy": "full_compress",
    "angular_detected": false,
    "line_count": 32,
    "version": 1,
    "decision_summary": "fidelity=Low, strategy=full_compress, class=general, angular=none, lines=32"
  }
}
```

### Zero-Touch Workflow

`provide_code_context` orchestrates:
1. Heuristics engine → decide fidelity + strategy
2. IR compilation (primary) or text pipeline (fallback)
3. Session stats recording
4. Angular detection + Meta-Layer
5. CBM enrichment injection
6. Cache invalidation (CBM graph version check)
7. Persistence (SQLite, when configured)

### Helper Functions

`compile_file_ir()` in `src/mcp/tool_helpers.rs`:
- Detects language from file extension
- Instantiates appropriate language/meta layers
- Wires additive + consumptive pattern recognizers
- Sets monotonic version from previous ContextState
- Handles NF-01 (pattern ordering), NF-02 (version chain)

---

## 11. Test Coverage

### IR Unit Tests

| Module | File | Count | Scope |
|--------|------|-------|-------|
| opcodes | `tests/ir/opcodes.rs` | ~10 | arity table, Display, flag constants |
| compiler | `tests/ir/compiler.rs` | ~20 | DefClass/DefMethod emission, fidelity, determinism |
| wire | `tests/ir/wire.rs` | ~40 | op_to_tuple all variants, round-trip, decode errors |
| render | `tests/ir/render.rs` | ~20 | fidelity comparison, round-trip |
| render_llm | `tests/ir/render_llm.rs` | ~20 | SCHEMA v2, overloaded methods, fidelity layout |
| delta | `tests/ir/delta.rs` | ~30 | add/modify/remove, version chain, compact encode |
| replay | `tests/ir/replay.rs` | ~30 | apply, remove/replace/append, error cases, sequential |
| symbol_table | `tests/ir/symbol_table.rs` | ~30 | registration, lookup, versioning, unregister |
| string_table | `tests/ir/string_table.rs` | ~20 | encode/decode round-trip, savings |
| hierarchical | `tests/ir/hierarchical.rs` | ~25 | tree structure, round-trip, synthetic classes |
| positional | `tests/ir/positional.rs` | ~35 | encode/decode, config, savings |
| patterns | `tests/ir/patterns.rs` | ~30 | pattern matching, round-trip, stats |
| binary_wire | `tests/ir/binary_wire.rs` | ~20 | encode/decode, varint, all-op round-trip |

### Language Layer Tests

| Layer | Tests | Coverage |
|-------|-------|----------|
| TypeScript | ~10 | extends, implements, async, export, static flags |
| C# | ~5 | colon-syntax inheritance, abstract/public |
| Rust | ~30 | derives, generics, cfg, impl relationships, self kind |
| Java | ~25 | extends, implements, constructor, Jakarta/Spring patterns |

### Integration Tests

| Suite | File | Tests | Scope |
|-------|------|-------|-------|
| IR integration | `tests/ir/integration.rs` | 4 | Full cycle: compile → serialize → delta → replay |
| Layer integration | `tests/ir/layers_integration.rs` | ~10 | 4-layer pipeline via IRCompiler |
| Rust integration | `tests/ir/rust_integration.rs` | ~30 | Rust struct/enum/trait/impl IR |
| Rust stats | `tests/ir/rust_stats_integration.rs` | 3 | Rust token tracking + session stats |
| MCP regression | `tests/mcp/tool_handlers.rs` | ~40 | Handler smoke tests, relative paths, IR-first format |
| CBM integration | `tests/cbm/integration.rs` | 5 | CBM enrichment compression |

**Total: 530+ IR-specific tests. See `docs/CHANGELOG.md` for the current workspace test count.**
