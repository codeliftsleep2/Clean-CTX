# Clean-CTX — Enterprise Token Waste Reducer & Context Compiler

A local-first, air-gapped context optimization engine that eliminates token waste in LLM interactions while maintaining zero network footprint. Built in Rust for enterprise environments with restrictive firewalls and DLP systems.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│  MCP stdio Interface (JSON-RPC 2.0)                    │
│                                                         │
│  ┌───────────────┐  ┌──────────────┐  ┌─────────────┐  │
│  │ compress_     │  │ decompress_  │  │ compress_   │  │
│  │ code_context  │  │ code_context │  │ workspace   │  │
│  └───────┬───────┘  └──────┬───────┘  └──────┬──────┘  │
│          │                  │                  │         │
│          │           ┌─────▼─────┐  ┌─────────▼──────┐  │
│          │           │  diff_    │  │  Tree-sitter   │  │
│          │           │  code_    │  │  AST + baseline│  │
│          │           │  context  │  │  snapshots     │  │
│          │           └─────┬─────┘  └─────────┬──────┘  │
│  ┌───────▼──────────────────▼──────────────────▼──────┐ │
│  │              Compressor Engine                     │ │
│  │  AST Extraction → Fidelity Filter → Opcode Encode  │ │
│  └───────┬────────────────────────────────────────────┘ │
│          │                                              │
│  ┌───────▼──────────┐  ┌─────────────────────────────┐ │
│  │ SymbolDictionary │  │ Decompressor                │ │
│  │ PathDictionary   │  │ Opcode → Readable expansion │ │
│  └───────┬──────────┘  └─────────────────────────────┘ │
│          │                                              │
│  ┌───────▼──────────┐  ┌─────────────────────────────┐ │
│  │ Tree-sitter AST  │  │ LocalStateCache             │ │
│  │ Parser (TS + C#) │  │ Hash + baseline snapshots   │ │
│  └──────────────────┘  └─────────────────────────────┘ │
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

## Key Features

### 1. Three-Fidelity Compression

| Fidelity | Description | Savings | Best For |
|----------|-------------|---------|----------|
| **Low** | Maximum compression with symbol opcodes | ~81-96% | Reading large codebases |
| **Medium** | Preserves async, exports, behavior markers | ~61-84% | Understanding code behavior |
| **High** | Preserves full keywords + indentation | ~61-83% | Code review / documentation |

### 2. Symbol Opcode Dictionary

Automatically maps repeated tokens to ultra-short opcodes:

```
$c = class          $s = string          $b = boolean
$ctor = constructor  $P = Promise         $a = async
$r = return         $t = throw           $E = Error
$T = true           $F = false           $e = export
```

Custom opcodes (`$1`, `$2`, etc.) are auto-assigned for tokens appearing 2+ times in a session.

### 3. Path Alias Mapping (§MAP)

Long file paths are compressed to short aliases:

```
§MAP
  α1 = C:\project\src\core\auth\security\Provider.tsx
  α2 = C:\project\src\core\auth\security\TokenVerifier.tsx
```

### 4. Bidirectional Compression

- **compress_code_context** — Source file → compressed skeleton
- **decompress_code_context** — Compressed skeleton → human-readable format
- **diff_code_context** — Source file → AST-level change-set (`+` / `-` / `~` / `=`)


### 5. Fidelity-Aware Caching

Cache keys include both file path AND fidelity level, so different compression modes of the same file are cached independently.

### 6. Workspace Compression

The `compress_workspace` tool scans an entire directory tree and produces a single compressed manifest of all TypeScript/C# files with shared path aliases and opcode dictionaries.

### 7. MCP Prompts (System Instructions)

The `cleanctx-notation` prompt provides system-level instructions to the AI explaining how to read and write Clean-CTX compressed notation, ensuring correct interpretation of opcodes, markers, and path aliases.

## Project Structure

```
src/
├── main.rs              # MCP server (JSON-RPC stdio transport)
├── lib.rs               # Module declarations
├── compressor.rs        # AST extraction, fidelity levels, symbol encoding
├── decompressor.rs      # Opcode expansion, path alias resolution
├── diff.rs              # AST-level structural diff (+/-/~/=)
├── dictionary.rs        # PathDictionary + SymbolDictionary with opcode registry
├── queries.rs           # Tree-sitter query patterns (TypeScript + C#)
├── cache.rs             # SHA-256 hashing + baseline-snapshot registry
├── analytics.rs         # tiktoken-based token counting (cl100k model)
├── protocol.rs          # JSON-RPC message handling
├── config.rs            # Project-level .clean-ctx.json configuration
└── test_files/
    ├── sample_service.ts   # Small test file (32 lines)
    └── LargeService.ts     # Large test file (~400 lines)
```

## Token Savings Results

### Small File: sample_Service.ts (193 raw tokens)

| Fidelity | Retained | **Savings** |
|----------|----------|-------------|
| Low | 36 | **81.35%** |
| Medium | 75 | **61.14%** |
| High | 75 | **61.14%** |

### Large File: LargeService.ts (2,957 raw tokens)

| Fidelity | Retained | **Savings** |
|----------|----------|-------------|
| Low | 119 | **95.98%** |
| Medium | 476 | **83.90%** |
| High | 499 | **83.12%** |

**Key insight:** Larger files compress significantly better (~96%) because structural overhead is amortized across more methods and classes.

## MCP Tools

### compress_code_context

Reads a source file and returns a compressed structural skeleton.

**Parameters:**
- `filePath` (required): Absolute path to .ts or .cs file
- `fidelity` (optional): `"low"` | `"medium"` | `"high"` — defaults to `"low"`

### decompress_code_context

Expands a compressed skeleton back into human-readable format with full opcode resolution.

**Parameters:**
- `compressedText` (required): The compressed output to expand

### compress_workspace

Compresses all TypeScript/C# files in a directory tree. Outputs a manifest with shared path aliases and opcode dictionaries.

**Parameters:**
- `directoryPath` (required): Absolute path to project directory
- `fidelity` (optional): `"low"` | `"medium"` | `"high"` — defaults to `"low"`

### diff_code_context

AST-level diff compression. Returns a compact change-set describing only the structural deltas between a file's previous in-session snapshot and its current on-disk state. Uses `+` / `-` / `~` / `=` markers, each typically costing 1 token under cl100k. The baseline is stored per-file in `LocalStateCache` after the first call; subsequent calls diff against the most recently stored snapshot and then rotate it forward.

**Parameters:**
- `filePath` (required): Absolute path to .ts or .cs file
- `fidelity` (optional): `"low"` | `"medium"` | `"high"` — defaults to `"low"`

**First call (no baseline):** stores the current snapshot and returns a notice.
**Subsequent calls:** returns a change-set like:

```
// --- AST Diff: C:\src\UserController.ts ---
// +2 -0 ~1 =1 (classes/methods/fields/imports)

~ class UserController
  + method delete(id:string):void
  ~ method create(name:string,email:string):Promise<User>
        was: create(name:string):Promise<User>
  = method getById(id:string):Promise<User> (unchanged)

+ class AuditLogger (1 methods)
+ import AuditLogger
```

This is dramatically cheaper than re-emitting the full compressed skeleton when most of the file is unchanged.

## MCP Prompts

### cleanctx-notation

System instructions for reading and writing Clean-CTX compressed notation. When loaded, the AI learns:
- How to interpret all opcodes (`$c`, `$ctor`, `$s`, etc.)
- How to interpret behavior markers (`⊕guard`, `⊕loop`, `⊕!throw`, `⊕⇒`)
- To respond in compressed form when appropriate
- To never output raw opcode tables or metadata sections

## Compression Pipeline Stages

```
Source Code
    │
    ▼
┌─────────────────────┐
│ Tree-sitter AST Parse│  Extract class, method, field, control flow nodes
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

## Enterprise Deployment Requirements

### Firewall Stealth Compliance
- **Zero Network Transport**: stdio-only via MCP, no HTTP/WS/RPC servers
- **No External Runtimes**: Single statically linked binary, no dynamic dependencies
- **Fully Deterministic Compaction**: Rule-based AST processing, no AI models required

### Supported Languages
- **TypeScript/JavaScript**: Full method, class, function, control flow extraction
- **C# (.cs)**: Full method, class, interface, field, control flow extraction

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

The binary will be output as `clean-ctx.exe` (Windows) or `clean-ctx` (Linux/Mac).

## VS Code Configuration

The MCP server works with **any** MCP-compatible VS Code extension. Below are configuration examples for the most common ones:

### Cline / Roo Code

File: `~/.vscode/extensions/saoudrizwan.claude-dev/settings/cline_mcp_settings.json`

```json
{
  "mcpServers": {
    "clean-ctx": {
      "command": "C:\\path\\to\\clean-ctx.exe",
      "args": []
    }
  }
}
```

### Cursor

File: `.cursor/mcp.json` (in project root)

```json
{
  "mcpServers": {
    "clean-ctx": {
      "command": "C:\\path\\to\\clean-ctx.exe",
      "args": []
    }
  }
}
```

### Claude Code (Anthropic)

File: `~/.claude/settings.json` or VS Code `settings.json`

```json
{
  "mcpServers": {
    "clean-ctx": {
      "command": "C:\\path\\to\\clean-ctx.exe",
      "args": []
    }
  }
}
```

### Continue.dev

File: `.continue/config.json`

```json
{
  "mcpServers": [
    {
      "name": "clean-ctx",
      "command": "C:\\path\\to\\clean-ctx.exe",
      "args": []
    }
  ]
}
```

### Zed

File: `settings.json` (Zed settings)

```json
{
  "context_servers": {
    "clean-ctx": {
      "command": "C:\\path\\to\\clean-ctx.exe",
      "args": []
    }
  }
}
```

## Usage Example

### Compress a file (Low fidelity)
```json
{
  "name": "compress_code_context",
  "arguments": {
    "filePath": "/path/to/MyService.ts",
    "fidelity": "low"
  }
}
```

**Output:**
```
$c SampleService;$ctor();processComplexData(payload: $s[]): $b;healthCheck(): $s
```

### Decompress back to readable format
```json
{
  "name": "decompress_code_context",
  "arguments": {
    "compressedText": "$c SampleService;$ctor();processComplexData(payload: $s[]): $b;healthCheck(): $s"
  }
}
```

**Output:**
```
class SampleService;constructor();processComplexData(payload: string[]): boolean;healthCheck(): string
```

### Compress entire workspace
```json
{
  "name": "compress_workspace",
  "arguments": {
    "directoryPath": "C:\\path\\to\\project",
    "fidelity": "medium"
  }
}
```

### AST-level diff between file versions
```json
{
  "name": "diff_code_context",
  "arguments": {
    "filePath": "C:\\path\\to\\MyService.ts",
    "fidelity": "low"
  }
}
```

**First call** stores the current state as the baseline. **Subsequent calls** return a compact change-set like:
```
// --- AST Diff: C:\path\to\MyService.ts ---
// +1 ~1 =1 (classes/methods/fields/imports)

~ class MyService
  + method archive():void
  ~ method process(id:string):boolean
        was: process(id:number):boolean
  = method healthCheck():string (unchanged)
```

The baseline rotates forward on every call, so a series of `diff_code_context` invocations against a changing file emits a stream of small deltas instead of repeated full re-compressions.

## Opcode Reference

### Built-in Primitives (always available)

| Opcode | Token | Opcode | Token | Opcode | Token |
|--------|-------|--------|-------|--------|-------|
| `$c` | class | `$s` | string | `$b` | boolean |
| `$n` | number | `$v` | void | `$a` | async |
| `$e` | export | `$r` | return | `$t` | throw |
| `$T` | true | `$F` | false | `$P` | Promise |
| `$ctor` | constructor | `$fn` | function | `$E` | Error |
| `$nw` | new | `$i` | if | `$fr` | for |
| `$w` | while | `$h` | this | `$k` | const |
| `$l` | let | `$pu` | public | `$pv` | private |
| `$st` | static | `$x` | extends | `$m` | implements |
| `$if` | interface | `$ty` | type | `$nl` | null |
| `$ud` | undefined | `$fm` | from | `$im` | import |

### Behavior Markers

| Marker | Meaning |
|--------|---------|
| `⊕guard` | Conditional branch (if statement) |
| `⊕loop` | Iteration (for/while loop) |
| `⊕⇒` | Return value follows |
| `⊕!` | Throws error |
| `⊕export` | Module export |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tree-sitter` + `tree-sitter-typescript` + `tree-sitter-c-sharp` | AST parsing |
| `tiktoken-rs` | Exact token counting (cl100k model) |
| `serde` / `serde_json` | JSON-RPC serialization |
| `sha2` | File content hashing for cache |
| `clap` | CLI argument parsing |

## License

Internal use — Enterprise Token Reduction Engine.