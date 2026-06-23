# Clean-CTX — Architecture Overview

**Version:** 0.1.6
**Last updated:** 2026-06-10 (added zero-touch workflow, SQLite persistence, heuristics engine, session stats dashboard)

---

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│  MCP stdio Interface (JSON-RPC 2.0)                     │
│                                                         │
│  ┌──────────────────────┐  ┌──────────────────────────┐ │
│  │ Zero-Touch Workflow  │  │ Heuristics Engine        │ │
│  │ provide_code_context │  │  fidelity + strategy     │ │
│  │  restore_context     │  │  selection per file      │ │
│  │  context_history     │  └──────────┬───────────────┘ │
│  │  context_stats       │             │                 │
│  └──────────┬───────────┘             │                 │
│             │                         │                 │
│  ┌──────────▼─────────────────────────▼──────────────┐  │
│  │              Compressor Engine                    │  │
│  │  AST Extraction → Fidelity Filter → Opcode Encode │  │
│  │  + Text Delta Snapshots + IR Source Cache         │  │
│  └──────────┬────────────────────────────────────────┘  │
│             │                                           │
│  ┌──────────▼────────────┐  ┌────────────────────────┐  │
│  │ SymbolDictionary      │  │ Decompressor           │  │
│  │ PathDictionary        │  │ Opcode → Readable      │  │
│  └──────────┬────────────┘  └────────────────────────┘  │
│             │                                           │
│  ┌──────────▼────────────┐  ┌────────────────────────┐  │
│  │ Tree-sitter AST       │  │ LocalStateCache        │  │
│  │ Parser (TS + C#)      │  │ Hash + baseline snaps  │  │
│  └───────────────────────┘  └────────────────────────┘  │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │ TokenAnalytics (cl100k tiktoken estimator)       │   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │ IR Subsystem (Compiler IR + Delta Transport)     │   │
│  │  compile → wire → string_table → delta → replay  │   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │ ContextStore (ContextStore trait)                │   │
│  │InMemoryContextStore | BufferedStore → SqliteStore│   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │ Angular Meta-Layer (Φ markers + graph)           │   │
│  │   detect → decorators → markers → bundler → graph│   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │ MCP Prompts (cleanctx-notation + dashboard)      │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

---

## Compression Pipeline Stages

```
     Source Code
          │
          ▼
┌─────────────────────┐
│     Tree-sitter     |
|     AST Parse       │  Extract class, method, field, control flow nodes
└─────────┬───────────┘
          │
     ┌────┴────┐
     ▼         ▼
┌─────────┐ ┌──────────────┐
│ Text    │ │ IR           │  IR compilation (opcodes + structured tuples)
│ Pipeline│ │ Pipeline     │  → wire format → string table → delta
└────┬────┘ └──────┬───────┘
     │             │
     ▼             ▼
┌─────────┐ ┌──────────────┐
│ Fidelity│ │ Delta        │
│ Filter  │ │ Computer     │
│ + Symbol│ │ (instruction │
│ Opcodes │ │  level diff) │
└────┬────┘ └──────┬───────┘
     │             │
     ▼             ▼
┌─────────────────────────┐
│  Output Formatter       │
│  (report + path aliases)│
└─────────────────────────┘
```

The **IR Subsystem** provides an alternative transport path. Instead of sending compressed text, the source is compiled to an instruction-level Intermediate Representation (IR), and deltas are computed between successive IR states. This enables:

- **Named format**: JSON arrays with opcode strings (human-readable IR)
- **String table format**: Integer-indexed arrays for ~30% additional savings
- **Compact delta format**: Field-patch deltas for edit sessions (~70-90% vs full re-compression)

---

## Zero-Touch Workflow

The zero-touch workflow is the **recommended entry point** for any file-related coding task. It orchestrates all subsystems automatically:

```
     provide_code_context(file)
          │
          ▼
┌─────────────────────┐
│   Heuristics Engine │  Decide fidelity + strategy based on:
│   (heuristics.rs)   │  - file characteristics (size, language)
└─────────┬───────────┘  - explicit intent ("edit", "debug", etc.)
          │              - existing baselines (text delta, IR delta)
          ▼              - Angular detection
┌─────────────────────┐
│ Strategy Dispatch   │
│                     │
│  FullCompress ──────┤──→ Full compression + IR compilation
│                     │    + persistence save
│  DeltaTransport ────┤──→ Delta computation (text + IR)
│                     │    + persistence save + delta append
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Session Stats       │  Record compression metrics:
│ (session_stats.rs)  │  - raw/compressed tokens, strategy
└─────────┬───────────┘  - Angular detection, fidelity
          │
          ▼
┌─────────────────────┐
│ Response            │  JSON-RPC response with content +
│ (tools.rs)          │  _meta (fidelity, strategy, version)
└─────────────────────┘
```

### Tools

| Tool | Purpose |
|------|---------|
| `provide_code_context` | **Single entry point** — auto-detects, selects fidelity, uses delta transport on subsequent calls |
| `restore_context` | Force full re-compression, clearing all baselines and DB entries |
| `context_history` | View compression history and delta savings for tracked files |
| `context_stats` | Dashboard: token savings, compression stats, session metrics |

---

## Persistence Layer

The persistence layer provides **cross-session persistence** for compression contexts via SQLite. It is enabled by setting the `CLEANCTX_PERSISTENCE_DB` environment variable.

```
     provide_code_context(file)
          │
          ▼
┌─────────────────────┐
│ Compress + Compile  │  Full compression pipeline + IR compilation
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ SqliteStore         │  Non-fatal persistence (fire-and-forget)
│ (sqlite_store.rs)   │
│                     │
│  contexts table     │  Baseline IR BLOB + compressed text
│  deltas table       │  Sequential delta payloads
│  symbols table      │  Symbol table entries
│  sessions table     │  Workspace session tracking
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ WAL-mode SQLite     │  Concurrent read/write safety
│ (rusqlite crate)    │  Schema versioning via _schema_version
└─────────────────────┘
```

### Design Decisions

- **Non-fatal persistence**: All DB writes are fire-and-forget with `eprintln!` warnings — compression never fails due to DB issues.
- **Content-hash deterministic IDs**: `ctx-{sha256_hex}` ensures idempotent saves (same content → same ID → UPSERT).
- **No Mutex**: MCP server is single-threaded (stdin/stdout loop), so no concurrent access protection needed.
- **Lazy initialization**: DB only opens if `CLEANCTX_PERSISTENCE_DB` env var is set — zero overhead for users who don't need persistence.
- **`binary_wire::encode/decode`**: IR is serialized/deserialized as BLOBs; `file_id` and `version` are restored from DB columns on load.

### Tools

| Tool | Purpose |
|------|---------|
| `save_context` | Explicit manual checkpoint to DB |
| `list_sessions` | Show tracked sessions/files |
| `replay_history` | Replay deltas from DB up to target sequence |
| `purge_old_deltas` | Trim old delta history by age |

---

## Module Structure

```
src/
├── main.rs                       # 3-line bootstrap: calls mcp::run()
├── lib.rs                        # Public module declarations
│
├── mcp/                          # MCP server layer (JSON-RPC stdio)
│   ├── mod.rs                    # run() entry point + persistence init
│   ├── server.rs                 # Stdin/stdout loop (F-02: line-size cap)
│   ├── router.rs                 # JSON-RPC method dispatch
│   ├── handlers.rs               # initialize, tools/list, prompts/list
│   ├── tools.rs                  # Tool definitions + get_tool_definitions()
│   ├── tool_handlers.rs          # Tool handler implementations (handle_*)
│   ├── tool_helpers.rs           # Shared helper functions for tool handlers
│   ├── prompts.rs                # cleanctx-notation + dashboard prompt content
│   ├── workspace.rs              # compress_workspace_dir + collect_source_files
│   ├── workspace_util.rs         # Workspace utility functions
│   ├── state.rs                  # McpState (dict + cache + config + persistence)
│   ├── heuristics.rs             # Heuristics engine (fidelity + strategy selection)
│   ├── context_store.rs          # ContextStore trait + InMemoryContextStore
│   ├── sqlite_store.rs           # SqliteStore (SQLite-backed ContextStore)
│   ├── buffered_store.rs         # BufferedStore (three-tier persistence wrapper)
│   └── session_stats.rs          # SessionStats + dashboard rendering
│
├── compression/                  # Core compression engine
│   ├── mod.rs                    # Public API: compress_file, CompressionProgress
│   ├── fidelity.rs               # Fidelity enum (Low/Medium/High)
│   ├── language.rs               # Language detection (extension + heuristic)
│   ├── capture_pipeline.rs       # Shared tree-sitter extraction + capture walk
│   ├── markers.rs                # Behavior markers (⊕guard, ⊕loop, ⊕⇒, ⊕!)
│   ├── opcodes.rs                # Shared primitive opcode table (34 entries)
│   ├── symbol_compression.rs     # Low-fidelity symbol opcode pass
│   ├── report.rs                 # Output report formatting
│   ├── pipeline.rs               # Non-streaming orchestrator
│   ├── streaming.rs              # Streaming orchestrator (with progress callbacks)
│   ├── text_delta.rs             # Phase IV: delta-aware text compression (line-level)
│   ├── scope_defaults.rs         # Scope default application per fidelity
│   ├── micro_opcodes.rs          # Micro-opcode expansion for ultra-compact output
│   └── workspace_symbols.rs      # Global symbol table for workspace compression
│
├── ir/                           # IR Subsystem (Compiler IR + Delta Transport)
│   ├── mod.rs                    # Public module declarations
│   ├── compiler.rs               # IRCompiler: source → CompiledIR
│   ├── compiler_methods.rs       # Compiler method implementations
│   ├── opcodes.rs                # CoreOp enum (DefClass, DefMethod, etc.)
│   ├── wire.rs                   # ir_to_wire: CompiledIR → tuple format
│   ├── string_table.rs           # ir_to_string_table_wire: compact index format
│   ├── symbol_table.rs           # IR symbol table for cross-file resolution
│   ├── delta.rs                  # DeltaComputer, IRDelta, compact_encode
│   ├── replay.rs                 # DeltaReplay: apply deltas to baseline
│   ├── hierarchical.rs           # Hierarchical IR (grouped by file/class/method)
│   ├── positional.rs             # Positional encoding for compact IR format
│   ├── render.rs                 # IR rendering to text
│   ├── binary_wire.rs            # Binary wire format for IR transport
│   ├── patterns.rs               # CompressingPatternRecognizer
│   └── layers/
│       ├── mod.rs
│       ├── typescript.rs         # TypeScriptLayer for IR compilation
│       ├── csharp.rs             # C#Layer for IR compilation
│       ├── angular.rs            # AngularMetaLayer for IR compilation
│       ├── rust.rs               # RustLayer for IR compilation
│       └── patterns.rs           # CodePatternRecognizer for IR compilation
│
├── diff/                         # AST-level structural diff
│   ├── mod.rs                    # Public API
│   ├── snapshot.rs               # CapturedStructure, CapturedClass, CapturedMethod
│   ├── action.rs                 # DiffAction, DiffKind, DiffTarget
│   ├── builder.rs                # build_snapshot + try_build_with
│   ├── differ.rs                 # diff_snapshots + diff_class
│   ├── formatter.rs              # format_diff + diff_summary
│   └── keys.rs                   # method_key, field_key, group_by_key
│
├── compaction/                   # AST node compaction (was: helpers.rs)
│   ├── mod.rs
│   ├── modifiers.rs              # Shared modifier lists (public, private, static, etc.)
│   ├── class.rs                  # extract_class_name, format_class_entry
│   ├── method.rs                 # extract_method_sig + helpers
│   ├── field.rs                  # extract_field + helpers
│   ├── import.rs                 # compact_import, extract_import_names
│   └── expression.rs             # compact_expression, simple_compact
│
├── angular_meta/                 # Angular Meta-Layer (Phase 1 + 2 + 3)
│   ├── mod.rs                    # MetaBlock struct, run_meta_layer entry point
│   ├── detect.rs                 # Angular detection heuristic + sibling detection
│   ├── decorators.rs             # @Component / @Injectable / @NgModule / etc. extractor
│   ├── markers.rs                # Φ marker construction & expansion
│   ├── bundler.rs                # File-triplet resolver (*.component.ts → .html + .scss)
│   ├── template.rs               # tree-sitter-html Angular-syntax template extractor
│   ├── style.rs                  # CSS/SCSS class selector + variable extractor
│   ├── footer.rs                 # §ΦMAP workspace footer formatter
│   ├── graph.rs                  # AngularGraph — cross-file DI + selector graph (Phase 3)
│   └── graph_state.rs            # AngularGraphHandle — McpState integration wrapper
│
├── decompression/                # Opcode → readable expansion
│   ├── mod.rs
│   ├── decompressor.rs           # Decompressor struct (F-15: precomputed opcodes)
│   ├── opcodes.rs                # Re-exports shared opcode table
│   ├── markers.rs                # Shared marker expansion + Φ marker re-export
│   └── walker.rs                 # Line-by-line section walker
│
├── dictionary/                   # Symbol and path registries
│   ├── mod.rs
│   ├── path.rs                   # PathDictionary (α/β/γ aliases)
│   ├── symbol.rs                 # SymbolDictionary (opcode↔token mappings)
│   ├── huffman.rs                # HuffmanSymbolDictionary (frequency-based encoding)
│   └── workspace.rs              # GlobalSymbolTable (workspace-level symbol sharing)
│
├── cache.rs                      # LocalStateCache (hash registry + baseline snapshots)
├── config.rs                     # .clean-ctx.json project configuration
├── queries.rs                    # Tree-sitter query patterns (TypeScript + C#)
├── analytics.rs                  # tiktoken cl100k token counting (F-01: cached BPE)
├── protocol.rs                   # JSON-RPC message types
├── compressor.rs                 # Re-export shim (backward compatible)
└── src/test_files/               # Test fixtures including:
    ├── UserManagementService.ts  # ~440-line Angular service (added for edit simulation)
    └── ...                       # Other test files
```

---

## Delta Transport: LLM vs CPU Efficiency

Clean-CTX offers two delta transport mechanisms — text-level and IR-level. Both are designed to reduce **CPU load** and **local compute time** on subsequent calls, rather than reducing LLM token usage.

### How Delta Saves Resources (Not LLM Tokens)

| What delta **does** save | What delta does **not** save |
|--------------------------|------------------------------|
| ✅ CPU cycles — avoids re-parsing and re-compressing the full source file | ❌ LLM prompt tokens — the LLM receives the same compressed output either way |
| ✅ Latency — delta computation is faster than full re-compression (up to 53% faster at High fidelity) | ❌ API costs — delta payload is delivered to the LLM at the same token count |
| ✅ Session throughput — more edits per minute for the same CPU budget | ❌ Wire overhead — delta metadata adds a small fixed envelope per call |

**LLM token savings come exclusively from compression** (Low/Medium/High fidelity), not from delta transport. Delta transport is a CPU-savings layer on top of compression.

### Two Delta Pipelines

| Pipeline | Granularity | CPU Savings vs Full ReCompress | Best For |
|----------|-------------|:------------------------------:|----------|
| Text-level (`delta_text_context`) | Line-level diffs of compressed body | ~70-90% | Rapid edit sessions with small changes |
| IR-level (`delta_code_context`) | Instruction-level diffs of compiled IR | Field-patch encoding | Structured code analysis, workspace-aware refactoring |

### 50-Edit Session Results

Simulated 50 sequential edits on a ~440-line file across all three fidelity levels:

| Fidelity | Raw | ReComp | Delta | ReSav% | DelSav% | Delta vs ReComp |
|----------|----:|------:|------:|------:|-------:|:---------------:|
| **Low** | 227,310 | 7,823 | 8,490 | 96.6% | 96.3% | +8.5% overhead |
| **Medium** | 227,310 | 37,338 | 18,287 | 83.6% | 92.0% | **−51.0%** cheaper |
| **High** | 227,310 | 48,556 | 22,955 | 78.6% | 89.9% | **−52.7%** cheaper |

Key insight: at Medium/High fidelity, delta is **51–53% cheaper** than full recompression because larger compressed outputs make line-level deltas significantly smaller than re-parsing.

See [`docs/PERFORMANCE.md`](PERFORMANCE.md) for the full 50-edit breakdown and visualizations.

---

## IR Subsystem: Delta Transport

The IR subsystem provides the IR-level delta transport mentioned above. It has three key components:

### 1. IR Compilation

Source code is compiled into a `CompiledIR` — a sequence of `CoreOp` instructions:

```
CoreOp::DefClass(id, "UserService")
CoreOp::DefMethod(id, parent, "getUserById", ["string"], "Promise<ApiResponse<UserProfile>>")
CoreOp::DefMethod(id, parent, "createUser", ["Partial<UserProfile>"], "Promise<ApiResponse<UserProfile>>")
...
```

### 2. Wire Formats

| Format | Description | Overhead |
|--------|-------------|----------|
| Named | JSON arrays with opcode strings | ~1.5x raw |
| String table | Integer-indexed arrays (Phase I) | ~1.0x raw (30% savings vs named) |
| Binary | Compact binary encoding | ~0.5x raw |

### 3. Delta Transport

The `DeltaComputer` produces instruction-level deltas between two `CompiledIR` states:

```json
{
  "file": "α1",
  "from": 1,
  "to": 2,
  "ops": {
    "+": [ ["DefMethod", "3", "2", "getUserPermissions", ...] ],
    "~": [ { "k": ["DefMethod","1","0"], "d": [{"i":3,"v":"(userId, fields?)"}] } ],
    "-": [ ["DefMethod", "4", "2", "deprecatedMethod", ...] ]
  }
}
```

Deltas can optionally use **field-patch encoding** (Idea #3) where only the changed fields are transmitted rather than the full replacement instruction — this is the most compact form.

---

## Design Decisions

### Why stdio-only (no HTTP)?

Clean-CTX is designed for **air-gapped environments** where DLP (Data Loss Prevention) systems block all network traffic. By communicating exclusively over stdin/stdout via the Model Context Protocol (MCP), the binary:
- Has zero network footprint
- Requires no ports, no HTTP server, no TLS configuration
- Works behind the most restrictive firewalls out of the box

### Why tree-sitter instead of regex?

Regex-based extraction of code structure is fragile — it breaks on comments containing keywords, multi-line signatures, and language-specific syntax. Tree-sitter provides:
- Error-tolerant parsing (produces a valid AST even on incomplete code)
- Language-specific grammars that handle edge cases correctly
- A unified query language (`(class_declaration name: (identifier) @class.name)`) that works across languages

### Why three fidelity levels?

Different LLM tasks need different amounts of structural detail. Numbers below are **measured** on the in-repo fixtures (`sample_service.ts`, 32 lines / 193 raw tokens; `LargeService.ts`, 438 lines / 2,957 raw tokens) using the cl100k BPE estimator:

- **Low** (77.20% – 97.50% savings): Best for "get the shape of this code" — class names, method signatures, parameter types, and one-line control-flow markers (`⊕guard`, `⊕loop`, `⊕⇒return …`). On `LargeService.ts` this drops 2,957 → 74 tokens; on `sample_service.ts` 193 → 44.
- **Medium** (34.20% – 86.34% savings): Best for "understand control flow" — preserves `async`/`export`/`public` keywords, full method signatures, and inline behavior markers for every guard, throw, and early-return. On `LargeService.ts` 2,957 → 404; on `sample_service.ts` 193 → 127.
- **High** (30.05% – 77.24% savings): Best for code review — preserves every TypeScript keyword, full type annotations, and embeds behavior markers directly in the method body braces. On `LargeService.ts` 2,957 → 673; on `sample_service.ts` 193 → 135.

**Scale matters:** compression efficiency grows with file size because structural overhead (class headers, import blocks, method signatures) is amortized across more methods. A service with 20+ methods at Low fidelity will consistently exceed 95% savings.

See [Measured Compression Performance](#measured-compression-performance) for the full benchmark table.

### Why a shared cache?

The `LocalStateCache` serves double duty:
1. **Content-hash registry** — avoids re-compressing identical files in the same session
2. **Baseline snapshot registry** — enables `diff_code_context` to produce AST-level deltas instead of full re-compressions on subsequent calls

### Why path aliases are global across the session

Path aliases (`α1`, `α2`, …) are **session-global** — `compress_workspace` populates aliases that are immediately visible to subsequent `provide_code_context` calls, and vice versa. This means that if a workspace compression assigns `α1` to `src/user.service.ts`, a later `provide_code_context("src/user.service.ts")` will reuse the same `α1` alias, keeping the `§PATHMAP` footer stable across multiple tools. Aliases are never recycled within a session; they only reset on server restart.

### Why both text-level and IR-level delta transport?

Two delta pipelines serve different scenarios:

| Pipeline | Granularity | Best For | Overhead |
|----------|-------------|----------|----------|
| Text-level (`delta_text_context`) | Line-level diffs of compressed body | Rapid edit sessions with small changes | ~70-90% savings vs full recompression |
| IR-level (`delta_code_context`) | Instruction-level diffs of compiled IR | Structured code analysis, workspace-aware refactoring | Field-patch encoding for maximum compactness |

The text-level pipeline is faster and simpler for quick edits. The IR pipeline preserves structural semantics and enables workspace-level cross-file analysis.

### Why SQLite for persistence?

SQLite provides:
- **Zero configuration** — embedded database, no server process
- **WAL mode** — concurrent read/write safety without external locking
- **BLOB support** — compact IR binary storage
- **Schema versioning** — forward-compatible migrations via `_schema_version` table
- **Portability** — single file can be backed up or moved

### Why non-fatal persistence?

All DB writes are fire-and-forget with `eprintln!` warnings. This ensures:
- Compression **never fails** due to DB issues (disk full, permissions, etc.)
- The MCP server remains **always available** even if persistence is misconfigured
- Users can **opt-in** to persistence without breaking existing workflows

### Why string-based Meta-Layer extraction?

The Meta-Layer walks raw text of existing `class.root` tree-sitter captures rather than building a new tree-sitter query or re-parsing the AST. This design choice:
- Keeps the dependency footprint minimal (only `tree-sitter-html` for template parsing)
- Avoids duplication with the core compression pipeline's capture logic
- Runs at O(L) where L is the class body length — negligible compared to the parse step
- Makes the Meta-Layer a self-contained additive pass that can be independently tested

### Why word-boundary heuristics for modern Angular syntax?

Angular 17+ template control-flow (`@if`, `@for`, `@switch`, `@defer`) is not valid HTML, so tree-sitter-html parses these tokens as opaque `text` nodes. The extractor uses word-boundary heuristics instead of regex to detect `@`-prefixed keywords, avoiding the `regex` crate dependency entirely. This keeps the binary size small and the attack surface minimal.

---

## Angular Meta-Layer (Phase 1 + 2 + 3)

The Meta-Layer is **purely additive** — it never modifies the existing TS compaction output. It only appends a `Φ` block below the existing compacted class. Non-Angular files pay zero overhead (byte-identical output).

### Fidelity-Aware Output (F-ANG-23)

The Meta-Layer respects the compression fidelity level:

| Fidelity | Meta-Layer Output |
|----------|-------------------|
| **Low** | Class-level summaries only: `Φcmp:`, `Φsvc:`, `Φdir:`, `Φpipe:`, `Φmod:`. No field-level `@Input`/`@Output`, no `Φinjects:`. |
| **Medium** | Adds field-level `Φin:` / `Φout:` markers. Skips `Φinjects:` (class summary already shows the class). |
| **High** | Full output: adds `Φinjects:` with constructor DI types, `Φmodel:` for signal-based APIs, and `input()`/`output()` signal lines. |

### Phase 1 — Decorator Extraction (single-file mode)

```
     Source Code
          │
          ▼
┌─────────────────────┐
│   Angular Detect    │  is_angular_file() — strong + weak decorator signals
└─────────┬───────────┘
          │ (Angular file)
          ▼
┌─────────────────────┐
│ Decorator Extract   │  @Component, @Injectable, @NgModule, @Directive, @Pipe
│ + Field I/O Scan    │  @Input, @Output fields + constructor injection
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Φ Marker Emit       │  Φcmp:, Φsvc:, Φmod:, Φdir:, Φpipe:, Φin:, Φout:, Φinjects:
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Append to body      │  // --- Φ Angular Meta --- block after compacted class
└─────────────────────┘
```

### Phase 2 — File-Triplet Bundling (workspace mode only)

```
     compress_workspace
          │
          ▼
┌─────────────────────┐
│ Compress .ts/.js/.cs│  Standard Phase 1 + compression pipeline
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Triplet Resolver    │  *.component.ts → .html + .scss siblings (bundler.rs)
└─────────┬───────────┘
          │
     ┌────┴────┐
     ▼         ▼
┌──────────┐ ┌──────────┐
│ Template │ │  Style   │  tree-sitter-html parse  Byte-level scan
│ Extract  │ │ Extract  │  (tags, bindings, etc.)  (selectors, vars)
└────┬─────┘ └────┬─────┘
     │            │
     ▼            ▼
┌─────────────────────┐
│ ΦBUNDLE Groups      │  // ===== Φ1: user-card.component =====
│ + §ΦMAP Footer      │  Φtpl:...  Φsty:...
└─────────────────────┘
```

### Phase 2.5 — Modern Angular Syntax (Angular 17–21)

Supports Angular's evolving template and decorator syntax:

| Feature | Syntax | Detection |
|---------|--------|-----------|
| Control flow | `@if`, `@for`, `@switch`, `@else`, `@case`, `@empty` | Text-node word-boundary scanning |
| Defer blocks | `@defer (on viewport)`, `@placeholder`, `@loading`, `@error` | Text-node scanning with trigger extraction |
| `@let` declarations | `@let user = expr` | Text-node scanning |
| Signal inputs | `readonly x = input<T>()` | Class body scan (High fidelity) |
| Signal outputs | `readonly x = output<T>()` | Class body scan (High fidelity) |
| Model signals | `readonly x = model(false)` | Class body scan (High fidelity) |
| Signal inject | `private x = inject(Service)` | Class body scan (High fidelity) |
| Self-closing tags | `<app-avatar />` | tree-sitter `self_closing_tag` node |

These are embedded in the `Φtpl:` marker line (`@if`, `@for`) or emit their own `Φ` markers (`Φmodel:`, `Φin:...signal`).

### Phase 3 — Cross-File Dependency Graph (workspace mode only)

```  
     compress_workspace (post-bundling)
          │
          ▼
┌──────────────────────────┐
│ GraphCollector (per-file)│  Reads raw .ts → decorators::extract_graph_entries
│                          │  Collects (class_name, file_alias, kind, selector, injects)
└─────────┬────────────────┘
          │
          ▼
┌──────────────────────┐
│ AngularGraph Builder │  register_class → resolve_all() builds injected_by reverse edges
└─────────┬────────────┘
          │
     ┌────┴──────────────┐
     ▼                   ▼
┌──────────────┐  ┌──────────────┐
│ DI Resolution│  │ Selector     │  resolve_inject_type → `UserService@α12`
│              │  │ Linkage      │  resolve_selector  → `UserCardComponent@α9`
└──────┬───────┘  └──────┬───────┘
       │                 │
       └──────┬──────────┘
              ▼
┌──────────────────────────┐
│ Φgraph: + §ΦGRAPH Footer │  // Φgraph:UserCardComponent → injects=[UserService@α2] ← injected-by=[] 
│                          │  §ΦGRAPH
│                          │    cmp UserCardComponent@α1
│                          │      Φcmp:injects=[UserService@α2]
│                          │      selector="app-user-card"
└──────────────────────────┘
```

**Modules:** `src/angular_meta/` — `mod.rs`, `detect.rs`, `decorators.rs`, `markers.rs`, `bundler.rs`, `template.rs`, `style.rs`, `footer.rs`, `graph.rs`, `graph_state.rs`

**Added to McpState:** `angular_graph: AngularGraphHandle` — built once per `compress_workspace` call

**Marker vocabulary:** `Φcmp:`, `Φdir:`, `Φpipe:`, `Φsvc:`, `Φmod:`, `Φin:`, `Φout:`, `Φmodel:`, `Φinjects:`, `Φtpl:`, `Φsty:`, `ΦBUNDLE`, `ΦMAP`, `Φgraph:`, `§ΦGRAPH`

**Key design decisions:**
- **String-based extraction (no AST re-parse)** — the Meta-Layer walks raw text of existing `class.root` captures rather than building a new tree-sitter query. This keeps the dependency footprint minimal (only `tree-sitter-html` for templates) and avoids duplication with the core compression pipeline.
- **Typestate pattern (F-ANG-05)** — `AngularGraph` can only be created via `AngularGraphBuilder::build()`, making it a compile-time error to register classes after resolution.
- **Text-node scanning for modern syntax** — `@if`/`@for`/`@switch` are not valid HTML, so tree-sitter-html parses them as opaque `text` nodes. The extractor uses word-boundary heuristics instead of regex to avoid adding dependencies.

**Resolution rules:**
- Constructor `private` / `protected` params resolve to `Type@αN` where the type is a registered `@Injectable` class
- Custom-element tags (`<app-foo>`) resolve to the `@Component({selector: 'app-foo'})` class via `resolve_selector`
- Unknown types get a `?` suffix (e.g., `HttpClient?`) — no spurious errors
- Transitive dependencies are tracked (if A injects B which injects C, all edges are recorded)

**Config:** `meta_layers.angular.enabled` in `.clean-ctx.json` (default: on)

**Dependencies:** `tree-sitter-html = "=0.20.0"` (Phase 2+)

---

## Measured Compression Performance

All numbers below were produced by the `compress_code_context` tool on the in-repo TypeScript fixtures, using the **cl100k BPE** estimator (`tiktoken-rs`). "Raw tokens" is the encoded length of the source file as-is; "Retained tokens" is the encoded length of the compressed output (including the report header, the `§PATHMAP` footer, and all behavior markers).

### Per-file results (single pass)

| File | Lines | Raw tokens | Fidelity | Retained | Saved | Reduction |
| |---|---:|---:|---|---:|---:|---:|
| `sample_service.ts` | 32 | 193 | **Low**    | 44  | 149 | **77.20%** |
| `sample_service.ts` | 32 | 193 | **Medium** | 127 | 66  | **34.20%** |
| `sample_service.ts` | 32 | 193 | **High**   | 135 | 58  | **30.05%** |
| `LargeService.ts`   | 438 | 2,957 | **Low**    | 74  | 2,883 | **97.50%** |
| `LargeService.ts`   | 438 | 2,957 | **Medium** | 404 | 2,553 | **86.34%** |
| `LargeService.ts`   | 438 | 2,957 | **High**   | 673 | 2,284 | **77.24%** |
| `UserManagementService.ts` | 575 | 3,912 | **Low**    | 155 | 3,757 | **96.04%** |
| `UserManagementService.ts` | 575 | 3,912 | **Medium** | 754 | 3,158 | **80.73%** |
| `UserManagementService.ts` | 575 | 3,912 | **High**   | 943 | 2,969 | **75.89%** |

### Per-fidelity range

| Fidelity | Range across fixtures | Best for |
|---|---|---|
| **Low**    | 77.20% – **97.50%** | "Get the shape" — class names + method signatures only |
| **Medium** | 34.20% – **86.34%** | "Understand control flow" — signatures + behavior markers |
| **High**   | 30.05% – **77.24%** | Code review — full TypeScript keywords preserved |

### Aggregate (all three fixtures, summed)

| Scenario | Total raw | Total retained | Reduction |
|---|---:|---:|---:|
| All Low fidelity      | 7,062 | 273  | **96.13%** |
| All Medium fidelity   | 7,062 | 1,285 | **81.80%** |
| All High fidelity     | 7,062 | 1,751 | **75.21%** |

### 50-Edit Session Simulation (Delta Transport, All Fidelities)

Added in v0.1.0: a realistic simulation of a developer editing `UserManagementService.ts` (~440 lines) over an afternoon session — run at **all three fidelity levels**. See [`docs/PERFORMANCE.md`](PERFORMANCE.md) and `examples/fidelity_comparison.rs`.

| Fidelity | Raw | ReComp | Delta | ReSav% | DelSav% | Delta vs ReComp |
|----------|----:|------:|------:|------:|-------:|:---------------:|
| **Low** | 227,310 | 7,823 | 8,490 | 96.6% | 96.3% | +8.5% overhead |
| **Medium** | 227,310 | 37,338 | 18,287 | 83.6% | 92.0% | **−51.0%** cheaper |
| **High** | 227,310 | 48,556 | 22,955 | 78.6% | 89.9% | **−52.7%** cheaper |

**Key findings:**
- **Low fidelity**: Delta is within 0.3 pp of recompression. Fixed envelope cost adds +8.5% because compressed output is tiny (avg 156 tokens).
- **Medium/High fidelity**: Delta is **actually cheaper than full recompression** (−51% to −53%) because larger compressed outputs make line-level deltas significantly smaller than re-parsing.
- **Delta breaks even immediately** at all fidelities — cumulative delta cost ≤ full recompression from Edit #1.

See [`docs/PERFORMANCE.md`](PERFORMANCE.md) for full per-edit breakdown and the examples:
- `cargo run --example fifty_edit_simulation` (Low fidelity)
- `cargo run --example fidelity_comparison` (all three fidelities)

### Key observations

- **Scale matters:** the 438-line file compresses 10–15 percentage points better than the 32-line file at every fidelity because structural overhead (class header, import block, constructor) is amortized across more methods.
- **Path aliasing helps:** when both files are compressed in the same session, the second file's `§PATHMAP` reuses the first file's `α1` and adds `α2` — eliminating duplicate absolute-path strings from the second output.
- **Behavior markers carry semantics:** even at Low fidelity, the output remains useful to an LLM because `⊕guard` / `⊕loop` / `⊕⇒return` / `⊕!throw` encode the control-flow shape of each method in just a few tokens.
- **Low fidelity on small files is fixed-cost dominated:** the 26-token output on `sample_service.ts` includes a 3-line report header and a 2-line `§PATHMAP` footer that are not affected by file size, which is why the percentage is lower than on the larger file.
- **Delta transport breaks even immediately:** in the 50-edit simulation, cumulative delta cost was ≤ full recompression from Edit #1 onward, with worst-case single-edit savings of 90.8% vs raw.

---

**Key results from the FAANG audit and SOLID refactoring:**
- `cargo clippy --all-targets -- -D warnings`: 0 warnings
- Largest source file: ~170 lines (down from 913)
- Zero network dependencies
- Zero `unsafe` blocks
- 1,306 tests passing (unit + integration + E2E + proxy regression tests)

> **ℹ️ Cache System Separation:** Clean-CTX has **two independent cache systems** with different scopes and configuration paths:
> 1. **MCP Server `CacheConfig`** (in `.clean-ctx.json`) — Controls `_meta.cache_hints` annotations in JSON-RPC responses. These annotations tell the LLM which parts of compressed output are cacheable (stable vocabulary, tool definitions, persisted baselines). Configuration is via the `cache` key in `.clean-ctx.json`.
> 2. **Proxy Cache** (environment variables `AUTO_CACHE`, `TAIL_TTL`) — A 4-slot Anthropic API `cache_control` breakpoint injector for HTTP request bodies. This reduces API costs by activating Anthropic's prompt caching. Configuration is via environment variables only.
>
> These systems are architecturally separate: the MCP server operates over stdin/stdout JSON-RPC and never makes HTTP requests; the proxy operates over HTTP and never processes JSON-RPC. Enabling one has no effect on the other, and they share no code or state.
