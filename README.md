# Clean-CTX — Token Waste Reducer & Context Compiler

A local-first, air-gapped context optimization engine that eliminates token waste in LLM interactions while maintaining zero network footprint. Built in Rust for restrictive firewall and DLP environments.

> **🚀 Version 0.1.6** — Zero-touch workflow (`provide_code_context`), SQLite persistence layer, Angular HTML parsing (XHTML self-closing + inline template), IR-level delta compression, text-level delta transport, cross-file dependency graph, modern Angular 17–21 syntax support, and 945 tests all passing.

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

Restart your editor. The tools `provide_code_context`, `compress_code_context`, `decompress_code_context`, `compress_workspace`, `diff_code_context`, `delta_code_context`, `delta_text_context`, `context_stats`, `context_history`, and `restore_context` will be available.

---

## Key Features

### Zero-Touch Workflow

The **recommended entry point** is `provide_code_context` — a single tool that automatically handles compression, delta transport, Angular detection, and fidelity selection:

| Tool | Purpose |
|------|---------|
| `provide_code_context` | **Single entry point** — auto-detects file type, selects optimal fidelity, uses delta transport on subsequent calls |
| `restore_context` | Force full re-compression, clearing all baselines and DB entries |
| `context_history` | View compression history and delta savings for tracked files |
| `context_stats` | Dashboard: token savings, compression stats, session metrics |

The workflow automatically:
- Runs a **heuristics engine** to select the best fidelity and strategy based on file characteristics
- Detects **Angular files** and enables the Meta-Layer with Φ markers
- Uses **delta transport** on subsequent calls for minimal token usage
- Records **session stats** for monitoring compression efficiency

### Three-Fidelity Compression

| Fidelity | Description | Savings | Best For |
|----------|-------------|---------|----------|
| **Low** | Maximum compression with symbol opcodes | ~81-96% | Reading large codebases |
| **Medium** | Preserves async, exports, behavior markers | ~61-84% | Understanding code behavior |
| **High** | Preserves full keywords + indentation | ~61-83% | Code review / documentation |

### Core Tools

| Tool | Purpose |
|------|---------|
| `compress_code_context` | Source file → compressed skeleton (text or IR) |
| `decompress_code_context` | Compressed skeleton → human-readable format |
| `compress_workspace` | Entire directory → single compressed manifest |
| `diff_code_context` | Source file → AST-level change-set (`+` / `-` / `~` / `=`) |
| `delta_code_context` | IR-level delta compression — instruction-level deltas between compiled IR states |
| `delta_text_context` | Text-level delta compression — line-level deltas between compressed body snapshots |

### Persistence Layer (Built-in)

Compression contexts persist automatically across sessions using SQLite (enabled by default, stored in `.clean-ctx/persistence.db`):

| Tool | Purpose |
|------|---------|
| `save_context` | Manual checkpoint to DB |
| `list_sessions` | Show tracked files/sessions |
| `replay_history` | Replay deltas from DB (crash recovery) |
| `purge_old_deltas` | Trim old delta history |

Persistence uses a **three-tier reliability stack**:
1. **Batched writes** — operations queue in memory and flush as single transactions
2. **Retry with exponential backoff** — transient DB failures retry up to 3 times
3. **JSON file fallback** — if all retries fail, data writes to `.clean-ctx/fallback/` and re-imports on next successful flush

Disable in `.clean-ctx.json` with: `"persistence": { "enabled": false }`

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

### Quick context (recommended)

```json
{
  "name": "provide_code_context",
  "arguments": {
    "filePath": "/path/to/MyService.ts"
  }
}
```

First call performs full compression; subsequent calls automatically use delta transport.

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

### View compression dashboard

```json
{
  "name": "context_stats",
  "arguments": {}
}
```

---

## 📊 Performance Benchmarks

### Token Compression

Clean-CTX delivers **75–97% token waste reduction** on real-world files. See [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) for the full per-file breakdown across all three fidelity levels (Low/Medium/High) and aggregated savings across all test files.

Key highlights:
- **Low fidelity**: Up to **97.5% savings** on large files (438 lines)
- **Medium fidelity**: Up to **86.3% savings** — balanced detail with behavior markers
- **High fidelity**: Up to **77.2% savings** with full type annotations preserved
- **Aggregate** (3 test files): **96.1% worst-case reduction** at Low fidelity

### Delta Transport (50-Edit Session)

Simulated 50 sequential edits on a ~440-line file across all three fidelity levels:

| Fidelity | Savings vs Raw | Delta vs ReComp |
|----------|:--------------:|:----------------:|
| **Low** | 96.3% | +8.5% overhead |
| **Medium** | **92.0%** | **−51% cheaper** |
| **High** | 89.9% | **−53% cheaper** |

- Delta transport **breaks even from Edit #1** at all fidelities
- At Medium/High, delta is **51–53% cheaper** than full recompression
- Run the simulations: `cargo run --example fifty_edit_simulation` (Low), `cargo run --example fidelity_comparison` (all three)

See [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) for per-edit breakdowns, caching analysis, microbenchmarks, and optimization checklist.

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
| Tests | ✅ 945 passing (870 unit + 70 integration + 5 E2E workflow) |
| Audit | ✅ FAANG-level audit — all 41 findings resolved |
| Largest file | ~170 lines (down from 913) |
| Unsafe code | 0 blocks |
| Meta-Layer | ✅ Phases 1–3 complete (decorators, bundling, graph) |
| Workflow | ✅ Zero-touch workflow with heuristics engine |
| Persistence | ✅ SQLite cross-session persistence (built-in, three-tier reliability) |

---

## Documentation

| Document | Audience | Content |
|----------|----------|---------|
| [`README.md`](README.md) | **Users** | Installation, configuration, usage, opcode reference |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contributors | Overview, process, quick links to detailed docs |
| [`docs/ARCHITECTURE_OVERVIEW.md`](docs/ARCHITECTURE_OVERVIEW.md) | Architects | System design, module structure, pipeline stages, design decisions |
| [`docs/DEVELOPER_DOCUMENTATION.md`](docs/DEVELOPER_DOCUMENTATION.md) | Contributors | Building, testing, adding languages/tools/opcodes, code quality gates |
| [`docs/COMPILER_IR.md`](docs/COMPILER_IR.md) | Architects | Compiler IR protocol, delta state transport, wire format, phase implementation |
| [`docs/ANGULAR_META_LAYER.md`](docs/ANGULAR_META_LAYER.md) | Developers | Angular Meta-Layer design, marker vocabulary, template extraction, graph |
| [`docs/EDIT_TYPE.md`](docs/EDIT_TYPE.md) | Developers | Edit categorization vocabulary for delta transport annotation |
| [`docs/TROUBLESHOOTING.md`](docs/TROUBLESHOOTING.md) | Users | Common issues, error codes, diagnostic commands |
| [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) | Architects | Benchmarks, caching, memory profile, optimization checklist |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Administrators | Compliance checklist, hardening, SBOM, air-gap deployment |
| [`docs/CHANGELOG.md`](docs/CHANGELOG.md) | All | Version history with all additions, fixes, and deferrals |
| [`docs/INTELLIGENCE_LAYER_PLAN.md`](docs/INTELLIGENCE_LAYER_PLAN.md) | Architects | Intelligence Layer: PageRank scoring, blast radius, token budget packing |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | Contributors | Future plans, prioritized items, carry-over from audit |

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.