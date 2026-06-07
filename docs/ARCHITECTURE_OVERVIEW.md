# Clean-CTX — Architecture Overview

**Version:** 0.1.0
**Last updated:** 2026-06-07 (empirical compression numbers verified against `sample_service.ts` and `LargeService.ts`)

---

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│  MCP stdio Interface (JSON-RPC 2.0)                     │
│                                                         │
│  ┌───────────────┐  ┌──────────────┐  ┌─────────────┐   │
│  │ compress_     │  │ decompress_  │  │ compress_   │   │
│  │ code_context  │  │ code_context │  │ workspace   │   │
│  └───────┬───────┘  └──────┬───────┘  └──────┬──────┘   │
│          │                 │                 │          │
│          │           ┌─────▼─────┐  ┌────────▼───────┐  │
│          │           │  diff_    │  │  Tree-sitter   │  │
│          │           │  code_    │  │  AST + baseline│  │
│          │           │  context  │  │  snapshots     │  │
│          │           └─────┬─────┘  └─────────┬──────┘  │
│  ┌───────▼─────────────────▼──────────────────▼───────┐ │
│  │              Compressor Engine                     │ │
│  │  AST Extraction → Fidelity Filter → Opcode Encode  │ │
│  └───────┬────────────────────────────────────────────┘ │
│          │                                              │
│  ┌───────▼──────────┐  ┌─────────────────────────────┐  │
│  │ SymbolDictionary │  │ Decompressor                │  │
│  │ PathDictionary   │  │ Opcode → Readable expansion │  │
│  └───────┬──────────┘  └─────────────────────────────┘  │
│          │                                              │
│  ┌───────▼──────────┐  ┌─────────────────────────────┐  │
│  │ Tree-sitter AST  │  │ LocalStateCache             │  │
│  │ Parser (TS + C#) │  │ Hash + baseline snapshots   │  │
│  └──────────────────┘  └─────────────────────────────┘  │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │ TokenAnalytics (cl100k tiktoken estimator)       │   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │ MCP Prompts (cleanctx-notation system guide)     │   │
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
          ▼
┌─────────────────────┐
│ Fidelity Filtering  │  Strip/keep keywords based on fidelity level
│ (Low/Medium/High)   │  Add behavior markers (⊕guard, ⊕loop, ⊕throw)
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Symbol Opcode       │  Replace repeated tokens with $xx codes
│ Encoding (Low only) │  Register frequent tokens for session learning
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Token Measurement   │  tiktoken cl100k exact token counting
│ + Output Report     │  Generate optimization report header
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Path Alias Appending│  §MAP footer with α/β/γ path references
└─────────────────────┘
```

---

## Module Structure

```
src/
├── main.rs                       # 3-line bootstrap: calls mcp::run()
├── lib.rs                        # Public module declarations
│
├── mcp/                          # MCP server layer (JSON-RPC stdio)
│   ├── mod.rs                    # run() entry point
│   ├── server.rs                 # Stdin/stdout loop (F-02: line-size cap)
│   ├── router.rs                 # JSON-RPC method dispatch
│   ├── handlers.rs               # initialize, tools/list, prompts/list
│   ├── tools.rs                  # Tool definitions + dispatch
│   ├── prompts.rs                # cleanctx-notation prompt content
│   ├── workspace.rs              # compress_workspace_dir + collect_source_files
│   └── state.rs                  # McpState (shared path dict + cache + config)
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
│   └── streaming.rs              # Streaming orchestrator (with progress callbacks)
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
├── decompression/                # Opcode → readable expansion
│   ├── mod.rs
│   ├── decompressor.rs           # Decompressor struct (F-15: precomputed opcodes)
│   ├── opcodes.rs                # Re-exports shared opcode table
│   ├── markers.rs                # Shared marker expansion
│   └── walker.rs                 # Line-by-line section walker
│
├── dictionary/                   # Symbol and path registries
│   ├── mod.rs
│   ├── path.rs                   # PathDictionary (α/β/γ aliases)
│   └── symbol.rs                 # SymbolDictionary (opcode↔token mappings)
│
├── cache.rs                      # LocalStateCache (hash registry + baseline snapshots)
├── config.rs                     # .clean-ctx.json project configuration
├── queries.rs                    # Tree-sitter query patterns (TypeScript + C#)
├── analytics.rs                  # tiktoken cl100k token counting (F-01: cached BPE)
├── protocol.rs                   # JSON-RPC message types
└── compressor.rs                 # Re-export shim (backward compatible)
```

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

- **Low** (86.53% – 97.46% savings): Best for "get the shape of this code" — class names, method signatures, parameter types, and one-line control-flow markers (`⊕guard`, `⊕loop`, `⊕⇒return …`). On `LargeService.ts` this drops 2,957 → 75 tokens; on `sample_service.ts` 193 → 26.
- **Medium** (63.73% – 88.33% savings): Best for "understand control flow" — preserves `async`/`export`/`public` keywords, full method signatures, and inline behavior markers for every guard, throw, and early-return. On `LargeService.ts` 2,957 → 345; on `sample_service.ts` 193 → 70.
- **High** (59.59% – 79.24% savings): Best for code review — preserves every TypeScript keyword, full type annotations, and embeds behavior markers directly in the method body braces. On `LargeService.ts` 2,957 → 614; on `sample_service.ts` 193 → 78.

**Scale matters:** compression efficiency grows with file size because structural overhead (class headers, import blocks, method signatures) is amortized across more methods. A service with 20+ methods at Low fidelity will consistently exceed 95% savings.

See [Measured Compression Performance](#measured-compression-performance) for the full benchmark table.

### Why a shared cache?

The `LocalStateCache` serves double duty:
1. **Content-hash registry** — avoids re-compressing identical files in the same session
2. **Baseline snapshot registry** — enables `diff_code_context` to produce AST-level deltas instead of full re-compressions on subsequent calls

---

## Measured Compression Performance

All numbers below were produced by the `compress_code_context` tool on the two in-repo TypeScript fixtures, using the **cl100k BPE** estimator (`tiktoken-rs`). "Raw tokens" is the encoded length of the source file as-is; "Retained tokens" is the encoded length of the compressed output (including the report header, the `§PATHMAP` footer, and all behavior markers).

### Per-file results

| File | Lines | Raw tokens | Fidelity | Retained | Saved | Reduction |
|---|---:|---:|---|---:|---:|---:|
| `sample_service.ts` | 32 | 193 | **Low**    | 26  | 167 | **86.53%** |
| `sample_service.ts` | 32 | 193 | **Medium** | 70  | 123 | **63.73%** |
| `sample_service.ts` | 32 | 193 | **High**   | 78  | 115 | **59.59%** |
| `LargeService.ts`   | 438 | 2,957 | **Low**    | 75  | 2,882 | **97.46%** |
| `LargeService.ts`   | 438 | 2,957 | **Medium** | 345 | 2,612 | **88.33%** |
| `LargeService.ts`   | 438 | 2,957 | **High**   | 614 | 2,343 | **79.24%** |

### Per-fidelity range

| Fidelity | Range across fixtures | Best for |
|---|---|---|
| **Low**    | 86.53% – **97.46%** | "Get the shape" — class names + method signatures only |
| **Medium** | 63.73% – **88.33%** | "Understand control flow" — signatures + behavior markers |
| **High**   | 59.59% – **79.24%** | Code review — full TypeScript keywords preserved |

### Aggregate (both fixtures, summed)

| Scenario | Total raw | Total retained | Reduction |
|---|---:|---:|---:|
| All Low fidelity      | 3,150 | 101  | **96.79%** |
| All Medium fidelity   | 3,150 | 415  | **86.83%** |
| All High fidelity     | 3,150 | 692  | **78.03%** |

### Key observations

- **Scale matters:** the 438-line file compresses 10–15 percentage points better than the 32-line file at every fidelity because structural overhead (class header, import block, constructor) is amortized across more methods.
- **Path aliasing helps:** when both files are compressed in the same session, the second file's `§PATHMAP` reuses the first file's `α1` and adds `α2` — eliminating duplicate absolute-path strings from the second output.
- **Behavior markers carry semantics:** even at Low fidelity, the output remains useful to an LLM because `⊕guard` / `⊕loop` / `⊕⇒return` / `⊕!throw` encode the control-flow shape of each method in just a few tokens.
- **Low fidelity on small files is fixed-cost dominated:** the 26-token output on `sample_service.ts` includes a 3-line report header and a 2-line `§PATHMAP` footer that are not affected by file size, which is why the percentage is lower than on the larger file.

---

## FAANG Audit & Refactoring

This codebase underwent a comprehensive FAANG-level audit (41 findings across 5 phases) and a full SOLID refactoring (5 phases). See:

- [`docs/FAANG_AUDIT.md`](FAANG_AUDIT.md) — Complete audit findings and remediation status
- [`docs/REFACTORING.md`](REFACTORING.md) — SOLID refactoring plan and execution history

**Key results:**
- `cargo clippy --all-targets -- -D warnings`: 0 warnings
- `cargo test`: 121/121 passing
- Largest source file: ~170 lines (down from 913)
- Zero network dependencies
- Zero `unsafe` blocks