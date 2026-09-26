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

The Compiler IR subsystem translates source code (TypeScript, C#, Rust, Java)
into a structured intermediate representation—a stream of `CoreOp`
instructions. `CompiledIR` together with the extracted semantic-edge state is
the canonical semantic source for subsequent operations:

- **Rendering**: checked hierarchy → SCHEMA-v5 model presentation
- **Delta transport**: IR → occurrence-aware `dv:2` sequence edits → explicit application
- **Persistence**: binary `0x04` IR + aligned semantic-edge snapshot + checked delta history
- **Auxiliary wire views**: reduced hierarchy and legacy named/positional debug representations
- **Language layers**: Pluggable passes for TS, C#, Rust, Java
- **Meta layers**: Pluggable passes for Angular, Spring Boot

The IR subsystem replaces the earlier text-only compression pipeline with a structured, stateful approach. Instead of re-parsing the full source on every interaction, the IR enables incremental updates via delta transport.

### Key Design Decisions

- **IR-first output**: structural responses render checked SCHEMA-v5; raw source is an explicit economics fallback, not a legacy text fallback
- **Fidelity-aware compilation**: each canonical baseline records the fidelity used to compile it
- **Deterministic deltas**: positional sequence edits preserve semantic identity, occurrence multiplicity, and order
- **Cross-file semantics**: complete semantic edges are owned separately and published through `WorkspaceIndex`

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
┌──────────────────┐          ┌──────────────────┐
│ Canonical        │          │ Checked hierarchy│
│ code-side forms  │          │ → SCHEMA-v5      │
│ binary v04       │          │ model output     │
│ dv:2 delta       │          └──────────────────┘
└────────┬─────────┘
         │ explicit acknowledgement
         ▼
┌─────────────────────┐
│ State replay        │  Transactional sequence apply plus aligned edges
│ and durable commit  │
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
5. Meta-layer pass (AngularMetaLayer, SpringBootMetaLayer, DotNetMetaLayer)
   — Derives class_captures directly from PassContext.captures per C-22,
     NOT from DefClass.name. Each type-root capture (class/interface/enum/
     record/struct/trait) produces its canonical source span via
     class_source_from_capture(), ensuring decorators/annotations/attributes
     are included for all three framework meta-layers.
   — Multi-class-per-file invariant: each class capture receives ONLY its
     own source span; the } class-boundary guard prevents backward scans
     from crossing into preceding classes.
6. Additive pattern recognition (CodePatternRecognizer — CTOR/OBSERVABLE/GETTER/SETTER)
7. Consumptive pattern classification (CompressingPatternRecognizer — PAT ops).
   The classification is ADDITIVE with respect to method identity:
   `DefMethod` / `Param*` / `Return` are retained and the `PAT` op follows them
   (F2 — see `docs/ARCHITECTURAL_INVARIANTS.md`).
8. Forward alias resolution (resolve_forward_aliases)
```

The compiler maintains per-compilation state:
- `id_counter: u64` — monotonic instruction ID generator (F-31: u64 to avoid overflow)
- `current_method: Option<String>` — O(1) method tracking (F-27)
- `current_method_flags: Vec<String>` — flag accumulation (F-28)
- `current_class: Option<String>` — class context tracking

Methods/fields outside a class are skipped (F-29). Standalone functions (`func.root`, `arrow.root`) create synthetic classes.

### 3.2 Delta Computation (IR → Delta)

Production uses `SequenceDeltaComputer` to compute occurrence-aware positional
edits between two ordered `CompiledIR` streams:

```
1. Derive typed semantic identity and a stable occurrence ordinal per tuple.
2. Compare the ordered baseline and target streams.
3. Emit validated insert/remove/replace edits with exact positions and expected tuples.
4. Attach the target source hash; the server separately retains the complete target edge snapshot.
```

Legacy key-based `IRDelta` remains decode-only compatibility input.

### 3.3 State Replay (Delta → Updated State)

The state machine applies the sequence transactionally:

```
1. Validate version chain (file.version must match delta.from)
2. Validate monotonic version (delta.to > delta.from)
3. Validate every position, semantic identity, occurrence ordinal, and expected tuple
4. Apply edits to a candidate state
5. Commit the candidate only after the complete stream succeeds
6. Persist/install aligned semantic edges at the acknowledgement boundary
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
├── render_llm.rs         # render_hierarchical_for_llm() — SCHEMA-v5 model presentation
├── wire.rs               # op_to_tuple(), tuple_to_op(), ir_to_wire(), wire_to_ir()
├── delta.rs              # SequenceDelta + legacy IRDelta compatibility
├── delta/sequence.rs     # occurrence-aware dv:2 computation and compact transport
├── replay.rs             # ContextState and transactional replay
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
}
```

---

## 6. Representation Formats

These formats do not have equal authority. Binary `0x04` is the physical
durable representation. The reduced hierarchical `result.ir` is
non-reversible auxiliary output. The `encoding` argument selects only the
legacy `pretty` response field where that field is still exposed pending R-46.

| Format | Current surface | Description | Authority |
|--------|-----------------|-------------|-----------|
| Named | default legacy `pretty` | JSON arrays with opcode strings | auxiliary/debug |
| Positional | `encoding: "positional"` legacy `pretty` | Schema-aware positional tuples | auxiliary/debug |
| Tagged | `encoding: "tagged"` legacy `pretty` | Positional tuples with opcode tags | auxiliary/debug |
| String Table | internal/research helper | Integer-indexed string interning | none |
| Reduced Hierarchical | fixed `result.ir` | Class→method→parameter auxiliary tree | non-reversible auxiliary |
| Binary v04 | internal persistence | Lossless canonical `CompiledIR` encoding with varints | physical durable authority |

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

### Reduced Hierarchical Format (Auxiliary Output)

The hierarchy reorganizes the flat CoreOp stream into a
class→method→parameter tree. A deliberately reduced form is exposed as
`result.ir`; SCHEMA-v5 text rendered from the checked full hierarchy is the
model-visible structural presentation.

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

### LLM-Optimized Text (SCHEMA v5)

The hierarchical IR is rendered to compact LLM-friendly text via `render_hierarchical_for_llm()`:

```
// SCHEMA v5  @=meta X=extends I=implements F=field M=method $=import →=scope mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias
// ── UserService ──
X BaseService
F userRepo:UserRepository
M processData(payload:$s):$b  ctl:IF,RET
```

---

## 7. Delta Transport

### Corrected Sequence Delta Envelope (`dv:2`)

```json
{
  "dv": 2,
  "file": "α1",
  "from": 1,
  "to": 2,
  "target_hash": "…",
  "edits": [
    {
      "op": "insert",
      "at": 2,
      "value": {
        "identity": {"opcode": "DEF_METHOD", "key": ["DEF_M", "C1", "M3"]},
        "occurrence": 0
      },
      "instruction": ["DEF_M", "C1", "M3", "newMethod"]
    }
  ],
  "intent": null
}
```

Legacy `+` / `~` / `-` envelopes are decode-only compatibility input.
Production generation emits only occurrence-aware `dv:2` sequence edits.

### Compact Delta Format

`CompactSequenceDelta` abbreviates only the outer `dv:2` envelope (`d`, `f`,
`v`, `e`, `h`, `i`). The occurrence-aware edits themselves retain their typed
identity and validation evidence. Durable MCP history stores normalized
sequence deltas with the physical file identity.

### Delta Application

`ContextState` in `src/ir/replay.rs` manages per-file IR state:
- `FileState`: ordered instruction tuples + primary-key index (HashMap)
- `ContextState`: per-file HashMap + global monotonic version
- `apply_sequence()`: validates and applies occurrence-aware edits transactionally
- `apply()`: legacy compatibility application
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
| Angular | `layers/angular.rs` | Wraps `angular_meta` module, emits `Φcmp:`/`Φsvc:`/`Φpipe:`/`Φin:`/`Φout:`/`Φmodel:`/`Φtpl:`/`Φsty:` markers + ecosystem sub-layers: RxJS (`Φobs:`/`Φsubject:`/`ΦpipeRx:`), NgRx (`Φngrx:`/`Φaction:`/`Φreducer:`/`Φeffect:`/`Φselector:`/`Φentity:`), Signals (`Φsignal:`/`Φcomputed:`/`Φsig-effect:`), Routing (`Φroute:`/`Φguard:`/`Φresolver:`) |
| Spring Boot | `layers/spring.rs` | Wraps `spring_meta` module, emits `Φrest:`/`Φctrl:`/`Φsvc:`/`Φrepo:`/`Φconf:`/`Φmap:`/`Φaut:`/`Φval:`/`Φbean:`/`Φprop:`/`Φpropf:` markers |
| .NET | `layers/dotnet.rs` | Wraps `dotnet_meta` module, emits `Φctrl:`/`Φapi:`/`Φaction:`/`Φhub:`/`Φef:`/`Φdbset:`/`Φmap:`/`Φsvc:`/`Φdi:`/`Φjson:`/`Φauth:`/`Φmodel:` markers for ASP.NET Core, EF Core, SignalR, AutoMapper |

### Layer 4: Pattern Recognizers

**Additive** (`layers/patterns.rs` — `CodePatternRecognizer`):
- CTOR: constructor injection pattern
- OBSERVABLE: Observable return type
- GETTER: getter method pattern
- SETTER: setter method pattern

**Consumptive** (`patterns.rs` — `CompressingPatternRecognizer`):
- Classifies recognized patterns as `PAT` ops (Phase H)
- **Preserves method identity (F2):** `DefMethod`, its `Param*`, and its
  `Return` are re-emitted unchanged immediately before the `PAT` op, so the
  method still reaches every downstream consumer (hierarchical `MethodNode`,
  rendered `M` line, `UnitTable`, semantic registration, `Calls` subject).
  Only genuinely redundant, non-identity ops are summarized into the pattern
  (`Injects` for CTOR, `Flags(ASYNC)` for OBSERVABLE, `Flags(OVERRIDE)` for
  OVERRIDE, and the additive/trailing `Flags(M)` runs). PROMISE, EMPTY_CTOR,
  GETTER and SETTER match nothing but identity, so those classifications are
  purely additive.

---

## 10. MCP Tool Integration

### Tools Exposed

| Tool | Handler | Description |
|------|---------|-------------|
| `compress_code_context` | `handle_compress_code_context` | IR-first compression with encoding selection |
| `delta_code_context` | `handle_delta_code_context` | IR-level delta computation |
| `apply_delta` | `handle_apply_delta` | Explicit code-side acknowledgement of an exact pending delta |
| `provide_code_context` | `handle_provide_code_context` | Heuristic model-facing entry point; always returns complete current context |
| `restore_context` | `handle_restore_context` | Restore checked binary-v04 IR, delta-v2 history, and aligned edges without recompiling source |
| `context_history` | `handle_context_history` | Per-file delta history |
| `context_stats` | `handle_context_stats` | Session dashboard |

### Current Legacy Result-Level Response Shape

Until the separately versioned R-46 migration, context tools retain legacy
result-level fields. `content` is the model-visible SCHEMA-v5 presentation (or
an explicitly classified alternative), while `ir` is a reduced,
non-reversible auxiliary hierarchy. It is not the persistence authority.

```json
{
  "content": [{ "type": "text", "text": "// SCHEMA v5 ..." }],
  "ir": { "encoding": "hierarchical", "file": "α1", "v": 1, "ir": { ... } },
  "pretty": { "encoding": "named", "file": "α1", "v": 1, "ir": [...] },
  "v": 1,
  "file": "α1",
  "content_kind": "skeleton",
  "byte_exact": []
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
| render_llm | `tests/ir/render_llm.rs` | ~20 | SCHEMA v5, overloaded methods, fidelity layout |
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
