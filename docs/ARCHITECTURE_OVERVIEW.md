# Clean-CTX — Architecture Overview

**Version:** 0.1.0
**Last updated:** 2026-06-07

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

Different LLM tasks need different amounts of structural detail:
- **Low** (~81-96% savings): Best for "get the shape of this code" — method names, parameter types, class hierarchy
- **Medium** (~61-84% savings): Best for "understand control flow" — preserves async, exports, if/for/while markers
- **High** (~61-83% savings): Best for code review — preserves full keywords, indentation, almost all syntax

### Why a shared cache?

The `LocalStateCache` serves double duty:
1. **Content-hash registry** — avoids re-compressing identical files in the same session
2. **Baseline snapshot registry** — enables `diff_code_context` to produce AST-level deltas instead of full re-compressions on subsequent calls

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