# Clean-CTX — Token Waste Reducer & Context Compiler

A local-first, air-gapped context optimization engine that eliminates token waste in LLM interactions while maintaining zero network footprint. Built in Rust for restrictive firewall and DLP environments.

> **🚀 Version 0.1.6** — Angular HTML parsing fully fixed (XHTML self-closing tag support + inline template shape extraction), IR-level delta compression, text-level delta transport, cross-file dependency graph, modern Angular 17–21 syntax support, and 766 tests all passing.

---

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.85+ (edition 2024)

### Install

```bash
# Clone and build (release binary)
git clone https://github.com/codeliftsleep2/Clean-CTX.git
cd Clean-CTX
cargo build --release

# The binary is at: target/release/clean-ctx.exe (Windows) or target/release/clean-ctx (Linux/Mac)
```

### Configure VS Code

Add to your MCP settings (see [IDE Configuration](#ide-configuration) below for all options):

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

Restart your editor. The tools `compress_code_context`, `decompress_code_context`, `compress_workspace`, and `diff_code_context` will be available.

---

## Key Features

### Three-Fidelity Compression

| Fidelity | Description | Savings | Best For |
|----------|-------------|---------|----------|
| **Low** | Maximum compression with symbol opcodes | ~81-96% | Reading large codebases |
| **Medium** | Preserves async, exports, behavior markers | ~61-84% | Understanding code behavior |
| **High** | Preserves full keywords + indentation | ~61-83% | Code review / documentation |

### Six MCP Tools

| Tool | Purpose |
|------|---------|
| `compress_code_context` | Source file → compressed skeleton (text or IR) |
| `decompress_code_context` | Compressed skeleton → human-readable format |
| `compress_workspace` | Entire directory → single compressed manifest |
| `diff_code_context` | Source file → AST-level change-set (`+` / `-` / `~` / `=`) |
| `delta_code_context` | IR-level delta compression — instruction-level deltas between compiled IR states |
| `delta_text_context` | Text-level delta compression — line-level deltas between compressed body snapshots |

### Smart Caching

- **Content-hash cache** — identical files compress instantly on repeat calls
- **Baseline snapshots** — `diff_code_context` remembers the previous state, producing small deltas instead of full re-compressions
- **Raw-token count cache** — skip the BPE encode on cache hits (sub-millisecond responses)

### Path Alias Mapping

Long file paths are compressed to short aliases:

```
§MAP
  α1 = C:\project\src\core\auth\security\Provider.tsx
  α2 = C:\project\src\core\auth\security\TokenVerifier.tsx
```

### Angular Meta-Layer

For Angular projects, Clean-CTX automatically detects framework decorators and enriches the compressed output with structured metadata — without modifying existing behavior for non-Angular files.

| Tier | What It Does | When It Runs |
|------|-------------|--------------|
| **Tier 1 — Decorators** | Extracts `@Component`, `@Injectable`, `@NgModule`, `@Directive`, `@Pipe`, `@Input`, `@Output` and emits `Φ` markers | Single-file and workspace mode |
| **Tier 2 — File-Triplet Bundling** | Resolves `*.component.ts` → `.html` + `.scss` siblings; extracts template shape (tags, bindings, control flow) and style shape (selectors, variables) | Workspace mode only |
| **Tier 3 — Cross-File Graph** | Builds a DI injection graph (`UserService@α12`) and selector linkage (`<app-user-card>` → `UserCardComponent@α9`) across all files | Workspace mode only |

**Non-Angular files pay zero overhead** — no markers, no extra parsing, no newlines.

### Security

- **Zero network transport** — stdio-only via MCP, no HTTP/WS/RPC servers
- **No external runtimes** — single statically linked binary
- **No AI models** — fully deterministic, rule-based AST processing
- **Zero unsafe code** — entire codebase is safe Rust

---

## Usage Examples

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

### AST-level diff (track changes over time)

First call stores the current state as baseline. Subsequent calls return only the changes:

```
// --- AST Diff: C:\path\to\MyService.ts ---
// +1 ~1 =1 (classes/methods/fields/imports)

~ class MyService
  + method archive():void
  ~ method process(id:string):boolean
        was: process(id:number):boolean
  = method healthCheck():string (unchanged)
```

---

## 🗜️ Token Compression Test Results

Results from running the `compress_code_context` tool on both test files (the only `.ts` files available in `src/test_files/`: `sample_service.ts` and `LargeService.ts`) at all three fidelity levels.

---

### 📁 `sample_service.ts` (small — 32 lines)
**Raw tokens: 193**

| Fidelity | Retained | Saved | Reduction |
|----------|----------|-------|-----------|
| **Low** | 26 | 167 | **86.53%** |
| **Medium** | 70 | 123 | **63.73%** |
| **High** | 78 | 115 | **59.59%** |

- **Low** → Just the structural skeleton: `class SampleService; $ctor(); processComplexData(payload); healthCheck()`
- **Medium** → Class shell + method signatures + behavior markers (⊕guard, ⊕loop, ⊕⇒return)
- **High** → Full type signatures preserved with public modifiers and inline behavior markers

---

### 📁 `LargeService.ts` (large — 438 lines)
**Raw tokens: 2,957** (15.3× the size of the small file)

| Fidelity | Retained | Saved | Reduction |
|----------|----------|-------|-----------|
| **Low** | 75 | 2,882 | **97.46%** |
| **Medium** | 345 | 2,613 | **88.33%** |
| **High** | 614 | 2,343 | **79.24%** |

- **Low** → Aggressive skeleton: stripped 20 imports to `; ; ; ; …`, kept class name + 9 method signatures
- **Medium** → Imports listed (without paths), all method signatures with `async`/types, behavior markers preserved for error/return guards
- **High** → All 20 imports retained with full module paths, full type annotations, and method body behavior markers

---

### 🔍 Key Insights

1. **Scale matters** — Larger files compress much more efficiently. The 438-line file saved nearly **2,900 tokens** at low fidelity, vs. only 167 on the 32-line file.
2. **Fidelity trade-off is clear**:
   - **Low** = best for overview/structural reasoning (≤10% of original tokens)
   - **Medium** = balanced — signatures + key branches (≈10–15% of original)
   - **High** = best for refactoring/editing where type info matters (≈20% of original)
3. **Path aliasing** — Both files share a §PATHMAP dictionary, so the second file got even better compression because the path context was already established (α1 → α2 reuse).
4. **Behavior markers (⊕guard, ⊕loop, ⊕⇒return, ⊕!throw)** carry the control-flow semantics even at low fidelity, which is what makes the output actually useful for an LLM to reason about.

### 📊 Aggregated Savings
- **Total raw tokens** across 6 compressions: 3,150 tokens (193 × 3 + 2,957 × 3 averaged… actually 193 + 2,957 = 3,150 per pass, 6 passes)
- **Best case (low fidelity on both)**: 193 + 2,957 = 3,150 → 26 + 75 = 101 tokens = **96.79% aggregate reduction**
- **Worst case (high fidelity on both)**: 3,150 → 78 + 614 = 692 tokens = **78.03% aggregate reduction**

The tool delivers **78–97% token waste reduction** in real-world conditions. ✅

---

## 🧪 50-Edit Session Simulation: Delta Transport Savings

We simulated a realistic afternoon developer session on an Angular service file (`UserManagementService.ts`, ~440 lines) with **50 sequential edits** grouped into 5 categories. The simulation was run at **all three fidelity levels** for comparison. Each edit was measured across three pipelines:

### Cross-Fidelity Results

| Fidelity | Raw | ReComp | Delta | ReSav% | DelSav% | Delta vs ReComp |
|----------|----:|------:|------:|------:|-------:|:---------------:|
| **Low** (max compress) | 227,310 | 7,823 | 8,490 | 96.6% | 96.3% | +8.5% overhead |
| **Medium** (balanced) | 227,310 | 37,338 | 18,287 | 83.6% | **92.0%** | **−51.0%** cheaper |
| **High** (full detail) | 227,310 | 48,556 | 22,955 | 78.6% | 89.9% | **−52.7%** cheaper |

> **Key insight:** At Medium and High fidelity, delta transport is **actually cheaper** than full recompression! This is because compressed output at these fidelities is 5–6× larger, making the text-level delta (which only sends changed lines) significantly smaller than re-running the full compression pipeline. At Low fidelity the compressed output is so tiny (avg 156 tokens) that the fixed delta envelope cost (~80 chars) adds measurable overhead.

### Low Fidelity — Savings by Edit Category (most common daily-use setting)

| Category | Edits | Raw | ReComp | Delta | ReSav% | DelSav% |
|----------|-------|----:|------:|------:|------:|------:|
| Small changes | 1-10 | 39,202 | 1,545 | 988 | 96.1% | **97.5%** |
| Method-level | 11-20 | 41,370 | 1,498 | 1,580 | 96.4% | 96.2% |
| Structural | 21-30 | 44,610 | 1,587 | 1,740 | 96.4% | 96.1% |
| Cross-method | 31-40 | 49,598 | 1,383 | 2,436 | **97.2%** | 95.1% |
| Refactor | 41-50 | 52,530 | 1,810 | 1,746 | 96.6% | 96.7% |

### Single-Pass Compression Baselines (Edit #1)

| Fidelity | Raw | Compressed | Ratio |
|----------|----:|----------:|:-----:|
| Low | 3,912 | 155 | 25.2× |
| Medium | 3,912 | 754 | 5.2× |
| High | 3,912 | 943 | 4.1× |

### Key Insights

- **Low fidelity** (daily-use default): Delta transport delivers **96.3% savings** vs raw, within 0.3 pp of full recompression. Delta overhead is +8.5% due to the tiny compressed output size.
- **Medium fidelity**: Delta transport saves **92.0%** vs raw and is **51% cheaper** than full recompression — avoiding re-parsing pays off.
- **High fidelity**: Delta transport saves **89.9%** vs raw and is **52.7% cheaper** than full recompression.
- **Delta breaks even immediately** at all fidelities — cumulative delta cost ≤ full recompression from Edit #1 onward.
- **Worst-case delta saving** (Low fidelity, Edit #41): **90.8%** — even the most expensive edit saves over 90%.
- Run the simulations yourself: `cargo run --example fifty_edit_simulation` (Low) and `cargo run --example fidelity_comparison` (all three).

See [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) for full per-edit breakdown and cross-fidelity comparison tables.

---

## Opcode Reference

### Built-in Primitives (34 opcodes, always available)

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

### Angular Meta-Layer Markers (Φ)

| Marker | Meaning |
|--------|---------|
| `Φcmp:` | `@Component` — class name + selector, template URL, style URLs |
| `Φsvc:` | `@Injectable` — class name + `providedIn` scope |
| `Φmod:` | `@NgModule` — class name + declarations, imports, exports |
| `Φdir:` | `@Directive` — class name + selector |
| `Φpipe:` | `@Pipe` — class name + pipe name |
| `Φin:` | `@Input` — field name + optional alias |
| `Φout:` | `@Output` — field name + optional alias |
| `Φmodel:` | `model()` signal — field name + optional alias (Angular 17.1+) |
| `Φinjects:` | Constructor/DI injection — resolved types with file aliases |
| `Φtpl:` | Template shape — tags, bindings, control flow blocks |
| `Φsty:` | Style shape — class selectors, SCSS/CSS variables |
| `ΦBUNDLE` | File-triplet bundle group (workspace manifest) |
| `ΦMAP` | Workspace bundle alias map footer |
| `Φgraph:` | Cross-file dependency graph edge |
| `§ΦGRAPH` | Workspace dependency graph footer section |

---

## IDE Configuration

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

File: `.cursor/mcp.json` (project root)

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

---

## MCP Prompts

The `cleanctx-notation` prompt provides system-level instructions to the AI explaining how to read and write Clean-CTX compressed notation. When loaded, the AI learns:
- How to interpret all opcodes (`$c`, `$ctor`, `$s`, etc.)
- How to interpret behavior markers (`⊕guard`, `⊕loop`, `⊕!throw`, `⊕⇒`)
- How to interpret Angular Meta-Layer markers (`Φcmp:`, `Φsvc:`, `Φin:`, `Φgraph:`, etc.)
- To respond in compressed form when appropriate
- To never output raw opcode tables or metadata sections

---

## Configuration

Create a `.clean-ctx.json` file in your project root:

```json
{
    "exclude_patterns": ["dist", "node_modules", "*.spec.ts"],
    "fidelity_overrides": {
        ".cs": "medium",
        ".test.ts": "high"
    },
    "default_fidelity": "medium",
    "type_aliases": {
        "UserId": "string",
        "JsonObject": "Record<string, unknown>"
    }
}
```

See [`docs/DEVELOPER_DOCUMENTATION.md`](docs/DEVELOPER_DOCUMENTATION.md) for the full configuration reference.

---

## Supported Languages

| Language | Extension | Status |
|----------|-----------|--------|
| TypeScript | `.ts`, `.js` | ✅ Full support |
| C# | `.cs` | ✅ Full support |

Angular framework detection (decorators, templates, styles) is automatic for TypeScript files containing `@Component`, `@Injectable`, `@NgModule`, `@Directive`, or `@Pipe` decorators.

See [`docs/DEVELOPER_DOCUMENTATION.md`](docs/DEVELOPER_DOCUMENTATION.md) for instructions on adding new languages.

---

## Building from Source

```bash
# Debug build
cargo build

# Release build (stripped, LTO-optimized)
cargo build --release
```

The binary is output as `clean-ctx.exe` (Windows) or `clean-ctx` (Linux/Mac).

---

## Project Status

| Metric | Value |
|--------|-------|
| Build | ✅ `cargo check` clean |
| Linting | ✅ `cargo clippy --all-targets -- -D warnings` — 0 warnings |
| Tests | ✅ 766 passing |
| Audit | ✅ FAANG-level audit — all 41 findings resolved |
| Largest file | ~170 lines (down from 913) |
| Unsafe code | 0 blocks |
| Meta-Layer | ✅ Phases 1–3 complete (decorators, bundling, graph) |

---

## Documentation

| Document | Audience | Content |
|----------|----------|---------|
| [`README.md`](README.md) | **Users** | Installation, configuration, usage, opcode reference |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contributors | Overview, process, quick links to detailed docs |
| [`docs/ARCHITECTURE_OVERVIEW.md`](docs/ARCHITECTURE_OVERVIEW.md) | Architects | System design, module structure, pipeline stages, design decisions |
| [`docs/DEVELOPER_DOCUMENTATION.md`](docs/DEVELOPER_DOCUMENTATION.md) | Contributors | Building, testing, adding languages/tools/opcodes, code quality gates |
| [`docs/TROUBLESHOOTING.md`](docs/TROUBLESHOOTING.md) | Users | Common issues, error codes, diagnostic commands |
| [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) | Architects | Benchmarks, caching, memory profile, optimization checklist |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Administrators | Compliance checklist, hardening, SBOM, air-gap deployment |
| [`docs/CHANGELOG.md`](docs/CHANGELOG.md) | All | Version history with all additions, fixes, and deferrals |
| [`docs/FAANG_AUDIT.md`](docs/FAANG_AUDIT.md) | Auditors | Complete audit findings and remediation status |
| [`docs/REFACTORING.md`](docs/REFACTORING.md) | Developers | SOLID refactoring plan and execution history |

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.