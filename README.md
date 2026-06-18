# Clean-CTX — Token Waste Reducer & Context Compiler

> **🚀 Version 0.1.8** — Zero-touch workflow (`provide_code_context`), SQLite persistence layer, Angular/Spring Boot meta-layers, IR-level delta compression, text-level delta transport, cross-file dependency graph, modern Angular 17–21 syntax support, CBM (Codebase Memory) integration for graph-based symbol importance + blast radius, Rust and Java language support, multi-platform proxy (Anthropic/OpenAI/Generic), **26 built-in tool output filters**, secret scrubbing, and all 1,277 tests passing with **zero clippy warnings**.

A local-first, air-gapped context optimization engine that eliminates token waste in LLM interactions while maintaining zero network footprint. Built in Rust for restrictive firewall and DLP environments.

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

Restart your editor. The tools `provide_code_context`, `compress_code_context`, `decompress_code_context`, `compress_workspace`, `diff_code_context`, `delta_code_context`, `delta_text_context`, `apply_delta`, `context_stats`, `context_history`, `save_context`, `list_sessions`, `replay_history`, `purge_old_deltas`, and `restore_context` will be available.

---

## Key Features

### Zero-Touch Workflow

The **recommended entry point** is `provide_code_context` — a single tool that automatically handles compression, delta transport, Angular detection, fidelity selection, and CBM enrichment:

| Tool | Purpose |
|------|---------|
| `provide_code_context` | **Single entry point** — auto-detects file type, selects optimal fidelity, uses delta transport on subsequent calls, enriches with CBM symbol importance |
| `restore_context` | Force full re-compression, clearing all baselines and DB entries |
| `context_history` | View compression history and delta savings for tracked files |
| `context_stats` | Dashboard: token savings, compression stats, session metrics |

The workflow automatically:
- Runs a **heuristics engine** to select the best fidelity and strategy based on file characteristics
- Detects **Angular/Spring Boot files** and enables the Meta-Layer with framework-specific markers
- Uses **delta transport** on subsequent calls for minimal token usage (**51–53% cheaper** than full recompression at Medium/High fidelity)
- Enriches context with **CBM (Codebase Memory)** symbol importance and blast radius data
- Records **session stats** for monitoring compression efficiency
- Persists contexts to **SQLite** for crash recovery and cross-session continuity

### CBM (Codebase Memory) Integration

Clean-CTX integrates with [Codebase Memory MCP](https://github.com/codeliftsleep2/CodebaseMemory-MCP) to provide graph-based code intelligence:

| Feature | Description |
|---------|-------------|
| **Symbol Importance** | PageRank scores for every symbol in the codebase — high-importance symbols get higher fidelity |
| **Blast Radius** | Dependency graph tracing — knows which files are affected by a change |
| **Dead Code Detection** | Identifies orphaned classes, methods, and fields |
| **Architecture Awareness** | Understands layering, module boundaries, and dependency direction |

CBM responses are intercepted at the pipe level and compressed via JSON-aware compression (**~5,000 → ~1,100 tokens**, ~78% savings) before reaching the agent.

| Component | Description |
|-----------|-------------|
| `client.rs` | JSON-RPC 2.0 subprocess client with retry + exponential backoff |
| `bridge.rs` | DashMap-based TTL caching with `detect_changes()` cache invalidation |
| `proxy.rs` | Pipe-level response interception and pluggable tokenizer integration |
| `json_compress.rs` | JSON-aware compressor: key shortening, envelope stripping, null field removal |

### Spring Boot Meta-Layer

For Spring Boot Java projects, Clean-CTX automatically detects framework annotations and enriches compressed output with structured `Φ` markers:

| Marker | Meaning |
|--------|---------|
| `Φrest:` / `Φctrl:` | `@RestController` / `@Controller` with request mappings |
| `Φsvc:` | `@Service` component |
| `Φrepo:` | `@Repository` component |
| `Φconf:` | `@Configuration` component |
| `Φmap:` | `@RequestMapping` method mappings |
| `Φaut:` | `@Autowired` field injection |
| `Φval:` | `@Value` property injection |
| `Φbean:` | `@Bean` method-level injection |
| `Φprop:` | `@ConfigurationProperties` class |
| `Φpropf:` | Properties file structural shape |
| `Φgraph:` | Cross-file dependency graph (workspace mode) |

### Three-Fidelity Compression

| Fidelity | Description | Savings | Best For |
|----------|-------------|---------|----------|
| **Low** | Maximum compression with symbol opcodes | ~81-96% | Reading large codebases |
| **Medium** | Preserves async, exports, behavior markers | ~61-84% | Understanding code behavior |
| **High** | Preserves full keywords + indentation | ~61-83% | Code review / documentation |

### Core Tools

| Tool | Purpose |
|------|---------|
| `compress_code_context` | Source file → compressed skeleton (text or IR with encoding selection) |
| `decompress_code_context` | Compressed skeleton → human-readable format |
| `compress_workspace` | Entire directory → single compressed manifest |
| `diff_code_context` | Source file → AST-level change-set (`+` / `-` / `~` / `=`) |
| `delta_code_context` | IR-level delta compression — instruction-level deltas between compiled IR states |
| `delta_text_context` | Text-level delta compression — line-level deltas between compressed body snapshots |
| `apply_delta` | Client-side state update — applies IR delta to in-session state machine |

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

### Multi-Platform Proxy

Clean-CTX ships with an optional **local HTTP proxy** that sits between your LLM client and any AI API (Anthropic, OpenAI, DeepSeek, etc.), automatically injecting `cache_control` breakpoints to achieve ~90% API cost savings on cached turns:

```bash
AUTO_CACHE=1 TOOL_FILTERS=1 SCRUB_SECRETS=1 cargo run -p clean-ctx-proxy
```

Works with Cline, Cursor, Aider, Continue.dev, and GitHub Copilot (BYOK). See [`docs/PROXY.md`](docs/PROXY.md) for full documentation.

### Tool Output Filtering

The proxy includes **26 built-in TOML filters** that compress verbose tool output by 70–90%:

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

**Output (Phase 6 IR-first):**
```
// SCHEMA v2  @=meta X=extends I=implements F=field M=method $=import →=scope fl:=flags cl:=class-flags P=pattern T=type-alias
// ── SampleService ──
M doWork(payload:$s[]):$b
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
IR delta: v1 → v2
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

### Spring Boot Meta-Layer Markers (Φ)

| Marker | Meaning |
|--------|---------|
| `Φrest:` | `@RestController` with request mappings |
| `Φctrl:` | `@Controller` with request mappings |
| `Φsvc:` | `@Service` component |
| `Φrepo:` | `@Repository` component |
| `Φconf:` | `@Configuration` component |
| `Φmap:` | `@RequestMapping` / `@GetMapping` / etc. method mappings |
| `Φaut:` | `@Autowired` field injection |
| `Φval:` | `@Value` property injection |
| `Φbean:` | `@Bean` method definition |
| `Φprop:` | `@ConfigurationProperties` class |
| `Φpropf:` | Properties file structural shape |

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
- How to interpret Angular and Spring Boot Meta-Layer markers
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
| TypeScript | `.ts`, `.js` | ✅ Full support (Angular meta-layer) |
| C# | `.cs` | ✅ Full support |
| Rust | `.rs` | ✅ Full support (structs, enums, traits, impls, generics, derives, cfg, unsafe) |
| Java | `.java` | ✅ Full support (classes, interfaces, records, enums, Spring Boot meta-layer) |

Angular framework detection is automatic for TypeScript files containing `@Component`, `@Injectable`, `@NgModule`, `@Directive`, or `@Pipe` decorators. Spring Boot detection is automatic for Java files containing `@RestController`, `@Service`, `@Repository`, `@Configuration`, or `@RequestMapping` annotations.

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
| Linting | ✅ `cargo clippy --all-targets -- -D warnings` — **0 warnings, 0 errors** |
| Tests | ✅ **1,277 core tests** + 243 proxy tests = **1,520 total, all passing** |
| Audit | ✅ FAANG-level audit — all 41 findings resolved; CBM audit — all 6 findings resolved; Compiler-IR audit — all findings resolved |
| Languages | ✅ TypeScript, C#, Rust, Java with Angular/Spring Boot meta-layers |
| CBM Integration | ✅ Subprocess client with retry + caching, pipe-level JSON compression, session stats |
| Delta Transport | ✅ IR-level + text-level, field-patch encoding, compact delta format |
| Persistence | ✅ SQLite cross-session persistence with three-tier reliability |
| Proxy | ✅ Multi-platform proxy (Anthropic/OpenAI/Generic) with auto-cache + tool filters |
| Filters | ✅ 26 built-in TOML filters — cargo, npm, eslint, docker, go, and more |
| IR Subsystem | ✅ 8 phases A–H complete, 530+ IR-specific tests, 6 wire formats, full delta pipeline |
| Largest file | ~170 lines (down from 913) |
| Unsafe code | 0 blocks |

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