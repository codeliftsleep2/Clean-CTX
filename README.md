# Clean-CTX - Token Waste Reducer & Structured Transport Protocol

> **Version 0.4.0** - A local-first, air-gapped MCP server that reduces LLM token waste through four independent mechanisms:

| Mechanism | Savings | What it affects |
|-----------|---------|-----------------|
| **CBM symbol filtering** | 30-50% fewer tokens | Drops low-importance symbols before compression |
| **Compression** (3 fidelities) | 75-97% vs raw source | Every `provide_code_context` / `compress_code_context` response |
| **Tool output filtering** (26 filters) | 70-90% on tool results | Proxy-collapsed build/lint/test output |
| **Prompt caching** (proxy) | ~90% API cost savings | Repeated turns via Anthropic/OpenAI/DeepSeek |

**Delta transport** is a CPU-savings layer (up to 53% faster re-compiles) - it does **not** reduce LLM tokens. The LLM receives the same full compressed output either way.

Supports **TypeScript, C#, Rust, Java** with **Angular, Spring Boot, .NET** meta-layers, IR-level delta protocol, SCHEMA v2 response notation, and a multi-platform proxy with auto-cache + tool filters + secret scrubbing.

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

### Language & Feature Selection

Clean-CTX uses Cargo **feature flags** to control which languages and meta-layers are compiled into the binary. This lets you build a minimal binary with only the languages you need, reducing compile time and binary size.

| Category | Feature | Implies | Includes | Build With | Default |
|----------|---------|---------|----------|------------|---------|
| **Language** | `typescript` | - | Base TypeScript/JavaScript grammar | `--features typescript` | Yes |
| **Language** | `csharp` | - | Base C# grammar | `--features csharp` | Yes |
| **Language** | `rust` | - | Base Rust grammar | `--features rust` | No |
| **Language** | `java` | - | Base Java grammar | `--features java` | No |
| **Meta-Layer** | `angular` | `typescript` | Components, Services, DI, Pipes, Directives, Modules, Input/Output, Template/Shape extraction, Style extraction, NgRx, RxJS, Signals, PrimeNG, Bundle graph | `--features angular` | Yes |
| **Meta-Layer** | `spring_boot` | `java` | RestController, Controller, Service, Repository, Configuration, RequestMapping, Autowired, Value, Bean, ConfigurationProperties, Cross-file graph | `--features spring_boot` | No |
| **Meta-Layer** | `dotnet` | `csharp` | ASP.NET Core (Controllers, Actions, Routes, Auth), EF Core (DbContext, DbSet, Entities), SignalR (Hubs, Clients, Streaming), AutoMapper (Profiles, Mappings), JSON Serialization, DI, Validation, Identity, Caching, Logging, Cross-file graph | `--features dotnet` | Yes |

**Build with only specific languages:**

```bash
# Default build: TypeScript + Angular + C# + .NET
cargo build --release

# TypeScript + Angular only (no C#)
cargo build --release --no-default-features --features typescript,angular

# .NET/C# only (no TypeScript, Angular)
cargo build --release --no-default-features --features csharp,dotnet

# Rust only (no TypeScript, C#, Java, meta-layers)
cargo build --release --no-default-features --features rust

# All languages + all meta-layers
cargo build --release --features rust,java,spring_boot,dotnet
```

Default features give you **TypeScript with Angular meta-layer, C#, and .NET enrichment** - the most common full-stack combination. Everything else is opt-in:

```bash
# Add Rust, Java, and Spring Boot
cargo build --release --features rust,java,spring_boot

# Add just Rust
cargo build --release --features rust
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

Restart your editor. The tools `provide_code_context`, `compress_code_context`, `diff_code_context`, `diff_commits`, `delta_code_context`, `apply_delta`, `apply_edit`, `restore_context`, `context_stats`, `context_history`, `save_context`, `list_sessions`, `replay_history`, and `purge_old_deltas` will be available.

---

## Key Features

### Zero-Touch Workflow

The **recommended entry point** is `provide_code_context` - a single tool that automatically handles compression, delta transport, Angular detection, fidelity selection, and CBM symbol filtering:

| Tool | Purpose |
|------|---------|
| `provide_code_context` | **Single entry point** - auto-detects file type, selects optimal fidelity, uses delta transport on subsequent calls, filters low-importance symbols via CBM |
| `restore_context` | Force full re-compression, clearing all baselines and DB entries |
| `context_history` | View compression history and delta savings for tracked files |
| `context_stats` | Dashboard: token savings, compression stats, session metrics |

The workflow automatically:
- Runs a **heuristics engine** to select the best fidelity and strategy based on file characteristics
- Detects **Angular/Spring Boot files** and enables the Meta-Layer with framework-specific markers
- Uses **delta transport** on subsequent calls to avoid re-compiling the full IR from scratch (reduces CPU/latency by up to 53%, while the LLM receives the same full compressed output)
- Records **session stats** for monitoring compression efficiency
- Persists contexts to **SQLite** for crash recovery and cross-session continuity

### CBM (Codebase Memory) Integration

Clean-CTX integrates with [codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp) (CBM) using a **filter-first architecture**: CBM symbol importance scores determine which symbols are excluded from compression entirely, reducing token output instead of adding post-compression metadata.

| Feature | Description |
|---------|-------------|
| **Symbol Importance Filtering** | Symbols with importance score < 0.4 are dropped before compression runs, reducing token output by 30-50% for noisy files |
| **Blast Radius** | Dependency graph tracing - knows which files are affected by a change |
| **Dead Code Detection** | Identifies orphaned classes, methods, and fields |
| **Architecture Awareness** | Understands layering, module boundaries, and dependency direction |

**How it works:**
1. `provide_code_context` queries CBM for symbol importance scores via `get_symbol_importance(project)`
2. `build_cbm_skip_set()` identifies symbols with score < 0.4 for the current file
3. The compression pipeline checks `should_skip_capture()` for each class/method/field - low-importance symbols are dropped entirely
4. The IR compiler applies the same skip check before emitting `DefClass`/`DefMethod`/`DefField`
5. Session stats record tokens removed under the `cbm_filter` domain

**Result:** CBM reduces token output instead of increasing it. The post-compression enrichment step is removed.

### Three-Fidelity Compression

| Fidelity | Description | Savings | Best For |
|----------|-------------|---------|----------|
| **Low** | Maximum compression with symbol opcodes | ~81-96% | Reading large codebases |
| **Medium** | Preserves async, exports, behavior markers | ~61-84% | Understanding code behavior |
| **High** | Preserves full keywords + indentation | ~61-83% | Code review / documentation |

### Core Tools

| Tool | Purpose |
|------|---------|
| `compress_code_context` | Source file -> compressed skeleton (text or IR with encoding selection) |
| `diff_code_context` | Source file -> AST-level change-set (`+` / `-` / `~` / `=`) |
| `diff_commits` | Git ref-range diff -> per-file AST-level change-sets in one call |
| `delta_code_context` | IR-level delta compression - instruction-level deltas between compiled IR states |
| `apply_delta` | Client-side state update - applies an IR delta to the in-session state machine |
| `apply_edit` | Structural edits on previously-seen files (replace_body / insert_after / ...) |

### Persistence Layer (Built-in)

Compression contexts persist automatically across sessions using SQLite (enabled by default, stored in `.clean-ctx/persistence.db`):

| Tool | Purpose |
|------|---------|
| `save_context` | Manual checkpoint to DB |
| `list_sessions` | List persisted contexts from the DB (per-file rows with fidelity, token counts, delta count, last update) |
| `replay_history` | Replay deltas from DB (crash recovery) |
| `purge_old_deltas` | Trim old delta history |

Persistence uses a **three-tier reliability stack**:
1. **Batched writes** - operations queue in memory and flush as single transactions
2. **Retry with exponential backoff** - transient DB failures retry up to 3 times
3. **JSON file fallback** - if all retries fail, data writes to `.clean-ctx/fallback/` and re-imports on next successful flush

Disable in `.clean-ctx.json` with: `"persistence": { "enabled": false }`

### Smart Caching

- **Content-hash cache** - identical files compress instantly on repeat calls
- **Baseline snapshots** - `diff_code_context` remembers the previous state, producing small deltas instead of full re-compressions. Note: this baseline is **local to `diff_code_context`** (keyed by canonical path + fidelity in the session cache) - it is NOT seeded by `provide_code_context`/`compress_code_context`, so the first call on a file legitimately reports "No baseline snapshot for this file yet" and stores one for subsequent calls
- **Raw-token count cache** - skip the BPE encode on cache hits (sub-millisecond responses)

### Path Aliases

Path aliases (`α1`, `α2`, ...) are session-global: `provide_code_context` and `compress_code_context` populate aliases that are visible to all subsequent tools, keeping the `§PATHMAP` footer stable across multiple calls. Aliases are pre-assigned deterministically to ensure numbering is stable across runs.

### Multi-Platform Proxy

Clean-CTX ships with an optional **local HTTP proxy** that sits between your LLM client and any AI API (Anthropic, OpenAI, DeepSeek, etc.), automatically injecting `cache_control` breakpoints to achieve ~90% API cost savings on cached turns:

```bash
AUTO_CACHE=1 TOOL_FILTERS=1 SCRUB_SECRETS=1 cargo run -p clean-ctx-proxy
```

Works with Cline, Cursor, Aider, Continue.dev, and GitHub Copilot (BYOK). See [`docs/PROXY.md`](docs/PROXY.md) for full documentation.

### Tool Output Filtering

The proxy includes **26 built-in TOML filters** that compress verbose tool output by 70-90%:

| Category | Filters |
|----------|---------|
| **Build** | cargo, make, mvn, node-build, dotnet-build, go |
| **Lint** | eslint, ruff, biome, mypy, pyright, golangci-lint, shellcheck, hadolint, yamllint |
| **Test** | pytest, dotnet-test, ng |
| **Package Mgr** | npm, pip, apt, brew |
| **DevOps** | docker, docker-logs, kubectl |
| **Git** | gh, git-diff, pre-commit |
| **System** | curl, ssh, systemctl, tsc |

Enable with `TOOL_FILTERS=1`. Filters auto-detect the command from tool input and apply program-specific compression (e.g., collapsing a successful `cargo build` to `"cargo: ok"`). Custom filters can be added as TOML files in `.clean-ctx/filters/`.

### Secret Scrubbing

The proxy detects and redacts secrets (AWS keys, GitHub tokens, JWTs, PEM keys, etc.) in tool results before they reach the LLM. Enable with `SCRUB_SECRETS=1`.

### Security

- **Zero network transport** - stdio-only via MCP, no HTTP/WS/RPC servers
- **No external runtimes** - single statically linked binary
- **No AI models** - fully deterministic, rule-based AST processing
- **Minimal unsafe code** - only in test utilities for environment variable manipulation (required by Rust's stdlib)

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

**Output (Phase 6 IR-first):**
```
// SCHEMA v2  @=meta X=extends I=implements F=field M=method $=import →=scope fl:=flags cl:=class-flags P=pattern T=type-alias
// ── SampleService ──
M doWork(payload:$s[]):$b
```

### AST-level diff (track changes over time)

```json
{
  "name": "diff_code_context",
  "arguments": {
    "filePath": "/path/to/MyService.ts"
  }
}
```

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

### IR-level delta (edit sessions)

```json
{
  "name": "delta_code_context",
  "arguments": {
    "filePath": "/path/to/MyService.ts"
  }
}
```

First call stores baseline IR; subsequent calls return only the structural delta:

```
IR delta: v1 -> v2
```

### View compression dashboard

```json
{
  "name": "context_stats",
  "arguments": {}
}
```

---

## Performance Benchmarks

### Token Compression

Clean-CTX delivers **75-97% token waste reduction** on real-world files. See [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) for the full per-file breakdown across all three fidelity levels (Low/Medium/High) and aggregated savings across all test files.

Key highlights:
- **Low fidelity**: Up to **97.5% savings** on large files (438 lines)
- **Medium fidelity**: Up to **86.3% savings** - balanced detail with behavior markers
- **High fidelity**: Up to **77.2% savings** with full type annotations preserved
- **Aggregate** (3 test files): **96.1% worst-case reduction** at Low fidelity

### Delta Transport (50-Edit Session, CPU Savings Only)

Delta transport does NOT reduce LLM token counts - it reduces local CPU/latency by avoiding full re-parsing on subsequent calls. Simulated 50 sequential edits on a ~440-line file:

| Fidelity | Full ReComp (cumulative) | Delta (cumulative) | Delta vs ReComp |
|----------|:-----------------------:|:------------------:|:----------------:|
| **Low** | 7,823 tokens | 8,490 tokens | +8.5% overhead* |
| **Medium** | 37,338 tokens | 18,287 tokens | **-51% cheaper** |
| **High** | 48,556 tokens | 22,955 tokens | **-53% cheaper** |

*\*At Low fidelity the compressed output is already tiny (~156 avg tokens), so delta's fixed envelope cost adds overhead. Delta is always within 0.3 percentage points of recompression at Low.*

- Delta transport **breaks even from Edit #1** at Medium/High fidelity

See [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) for per-edit breakdowns, caching analysis, microbenchmarks, and optimization checklist.

---

## Response Notation (SCHEMA v2 - primary)

Every `provide_code_context` / `compress_code_context` / `restore_context` response starts with this legend and uses the structural grammar below:

```
// SCHEMA v2  @=meta X=extends I=implements F=field M=method $=import →=scope fl:=flags cl:=class-flags P=pattern T=type-alias
```

| Symbol | Meaning |
|--------|---------|
| `// ── Name ──` | opens a class scope |
| `cl:` | class-level flags |
| `X <Parent>` | extends |
| `I <Iface...>` | implements |
| `F name:type` | field |
| `M name(+N)` | method (`+N` = overload by param count) |
| `→ p:name:type ...` / `→ type` | parameters / return type |
| `fl:` | method flags: `IF LOOP RET THROW ASYNC GEN EXPORT STATIC PRIVATE PROTECTED ABSTRACT UNSAFE` |
| `$ alias module [names]` | import |
| `T alias = Type` | type alias |
| `P NAME [args]` | structural pattern (CTOR, OBSERVABLE, GETTER, SETTER...) |

**High fidelity** adds `cf:` (control flow), `df:` (reads/writes), `se:` (side effect), `ec:` (execution context). **Edit fidelity appends each focused method's verbatim source body** - byte-exact. Types render exactly as captured.

The full SCHEMA v2 notation reference (structure letters, behavior flags, meta-layer markers) is in [`docs/COMPILER_IR.md`](docs/COMPILER_IR.md).

---

## IDE Configuration

All clients register the same MCP server; only the config file location differs.

| Client | Config file |
|--------|-------------|
| Cline / Roo Code | `~/.vscode/extensions/saoudrizwan.claude-dev/settings/cline_mcp_settings.json` |
| Cursor | `.cursor/mcp.json` (project root) |
| Claude Code | `~/.claude/settings.json` or VS Code `settings.json` |
| Continue.dev | `.continue/config.json` (uses a JSON array of servers) |
| Zed | Zed `settings.json` under `context_servers` |

Standard server block (Continue.dev adapts it to its array form):

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

---

## Project Status

| Metric | Status |
|--------|--------|
| Build | OK: `cargo check` clean |
| Linting | OK: `cargo clippy --all-targets -- -D warnings` - **0 warnings, 0 errors** |
| Tests | OK: **All tests passing** - includes live-CBM semantic probes and a self-contained multilingual fixture suite |
| Languages | OK: TypeScript, C#, Rust, Java with Angular/Spring Boot/.NET meta-layers |
| Transport | OK: Stateful IR delta transport - compile once, send deltas thereafter |
| Pass Architecture | OK: Composable IRPass pipeline (Core -> Language -> Meta -> Exec* -> Graph* -> Inference* -> Validation), IR validator (E001-E010), query engine, semantic delta intents (*optional passes) |
| CBM Integration | OK: Filter-first - symbol importance scores drop low-importance symbols before compression; typed-error graph queries |
| Persistence | OK: SQLite cross-session persistence with three-tier reliability |
| Proxy | OK: Multi-platform proxy (Anthropic/OpenAI/Generic) with auto-cache + tool filters |
| Filters | OK: 26 built-in TOML filters - cargo, npm, eslint, docker, go, and more |

Unsafe code is test-only (environment-variable manipulation).

---

## Documentation

| Document | Audience | Content |
|----------|----------|---------|
| [`README.md`](README.md) | **Users** | Installation, usage, opcode reference |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contributors | Overview, process, quick links to detailed docs |
| [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md) | Users | **Configuration source of truth** - `.clean-ctx.json` schema, env vars, precedence, resource limits, persistence, heuristics, meta-layers, cache, proxy lifecycle |
| [`docs/ARCHITECTURE_OVERVIEW.md`](docs/ARCHITECTURE_OVERVIEW.md) | Architects | System design, module structure, pipeline stages, design decisions |
| [`docs/DEVELOPER_DOCUMENTATION.md`](docs/DEVELOPER_DOCUMENTATION.md) | Contributors | Building, testing, adding languages/tools/opcodes, code quality gates |
| [`docs/COMPILER_IR.md`](docs/COMPILER_IR.md) | Architects | Compiler IR protocol, delta state transport, wire format, phase implementation |
| [`docs/ANGULAR_META_LAYER.md`](docs/ANGULAR_META_LAYER.md) | Developers | Angular Meta-Layer design, marker vocabulary, template extraction, graph |
| [`docs/ANGULAR_ECOSYSTEM_DEEPENING.md`](docs/ANGULAR_ECOSYSTEM_DEEPENING.md) | Developers | Angular Ecosystem Deepening - RxJS/NgRx/Signals/Routing meta-layers, cross-layer graph |
| [`docs/DOTNET_META_LAYER.md`](docs/DOTNET_META_LAYER.md) | Developers | .NET/C# Meta-Layer design, marker vocabulary, ASP.NET/EF Core/SignalR |
| [`docs/EDIT_TYPE.md`](docs/EDIT_TYPE.md) | Developers | Edit categorization vocabulary for delta transport annotation |
| [`docs/DIFF_COMMITS_GUIDE.md`](docs/DIFF_COMMITS_GUIDE.md) | Users | `diff_commits` tool usage, ref validation, security posture |
| [`docs/PROXY.md`](docs/PROXY.md) | Users | Multi-platform proxy, tool output filtering, secret scrubbing, IDE integration |
| [`docs/TROUBLESHOOTING.md`](docs/TROUBLESHOOTING.md) | Users | Common issues, error codes, diagnostic commands |
| [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) | Architects | Benchmarks, caching, memory profile, optimization checklist |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Administrators | Compliance checklist, hardening, SBOM, air-gap deployment |
| [`docs/CHANGELOG.md`](docs/CHANGELOG.md) | All | Version history with all additions, fixes, and deferrals |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | Contributors | Future plans, prioritized items, carry-over from audit |

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) - Dedicated to the public domain.
