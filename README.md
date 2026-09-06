# Clean-CTX - Local-first context compiler for AI coding agents

> Compiles large codebases into compact, semantically rich, task-relevant context for LLMs - with language-aware IR, workspace semantic indexing, and MCP integration.

Clean-CTX is **not** a source-code minifier or token compressor. It builds a semantic model of a workspace and compiles the information an AI agent needs into a compact, structured context representation.

**The goal is not merely fewer tokens. The goal is better context per token.**

```text
Understand the codebase
        ↓
Extract semantic structure
        ↓
Index relationships and provenance
        ↓
Resolve and filter relevant context
        ↓
Compile it into a compact representation
        ↓
Give the AI less noise and more signal
```

Currently supports **TypeScript, C#, Rust, Java** language layers with **Angular, Spring Boot, .NET** (see below for full current meta-layer support) semantic intelligence, IR-level delta transport, workspace semantic indexing, and a multi-platform MCP server.

---

## Features

### Semantic code intelligence

Clean-CTX understands code as more than text:

* **Language-aware parsing** - tree-sitter grammars extract structure, not just tokens, for TypeScript, C#, Rust, and Java.
* **Semantic Intermediate Representation (SIR)** - source compiles into a typed compiler IR (`DefClass`, `DefMethod`, `DefField`, patterns, scopes), from which meta-layers extract a semantic model of entities with identity, provenance, and relationships.
* **Entity identity** - every class, service, component, action, and selector is identified by `(domain, entity_type, name)` with file provenance, so the same entity referenced from multiple files compares equal.
* **Semantic relationships** - typed edges between entities: `Injects`, `HasSelector`, `Dispatches`, `Selects`, `HandlesAction`, `RouteMapsTo`, `Autowired`, `ControllerAction`, `HasEntity`, and 20+ more, each with confidence 1.0 and layer provenance.
* **Provenance tracking** - every semantic edge records which meta-layer discovered it.

### Workspace semantic indexing

A cross-file semantic graph that survives beyond single-file compression:

* **WorkspaceIndex** - framework-agnostic index of entities and semantic edges across an entire workspace.
* **Identity-based lookup** - `(domain, entity_type, name)` is the canonical identity; a separate convenience lookup finds entity occurrences by bare name across all domains and types.
* **Forward/reverse relationship traversal** - follow `Injects`, `Dispatches`, `Selects`, and other edges in either direction.
* **Selector resolution** - resolve a CSS selector (`app-widget`, `[app-widget]`, `.app-widget`) to the component that exposes it.
* **Injection discovery** - resolve an injected type to all workspace entities referenced as `Injects`/`Autowired` targets (reference occurrences; ambiguity preserved).
* **Transitive dependencies** - BFS traversal through DI and routing relationships, with cycle detection.

### Context compilation

Task-relevant context rather than indiscriminate source dumping:

* **Compiles source into LLM-optimized representations** - three fidelity levels (Low/Medium/High) plus Edit and Verbatim, each preserving the semantics appropriate to the task.
* **Semantic compression** - reduces representation size while preserving the relationships downstream consumers depend on.
* **IR-level delta transport** - compile once, send instruction-level deltas thereafter (up to 53% CPU/latency savings on repeat calls; the LLM receives the same full output).
* **Structural edits** - `apply_edit` performs byte-exact edits on previously-seen files using the semantic model.

### MCP integration

Exposes code intelligence and context capabilities to AI coding agents through the Model Context Protocol:

* **`provide_code_context`** - single entry point: auto-detects file type, selects fidelity, applies delta transport, filters low-importance symbols.
* **`workspace_query`** - cross-file semantic queries: entity lookup, forward/reverse edges, selector resolution, injection targets, transitive dependencies, cycle detection.
* **`compress_code_context` / `restore_context`** - direct compression control with history and stats.
* **`diff_code_context` / `diff_commits`** - AST-level change-sets, single-file and git ref-range.
* **`delta_code_context` / `apply_delta`** - IR-level delta compression and client-side state updates.
* **`apply_edit`** - structural edits on previously-seen files.
* **Structured responses** - canonical `CallToolResult` envelope (`content` + `structuredContent` + `_meta`) with declared `outputSchema`.

### Language / framework awareness

Meaningful semantic intelligence, not generic text processing:

| Language | Framework intelligence |
|----------|----------------------|
| **TypeScript** | **Angular** - Components, Services, DI, Pipes, Directives, Modules, `@Input`/`@Output`, NgRx, RxJS, Signals, Routing, PrimeNG, bundle graph |
| **C#** | **.NET** - Controllers, Actions, Routes, EF Core, SignalR, AutoMapper, DI, validation |
| **Java** | **Spring Boot** - Controllers, RequestMappings, Services, Repositories, `@Autowired`, `@ConfigurationProperties`, beans |
| **Rust** | Semantic extraction for structs, enums, traits, impls |

Plain files (no framework) are indexed through the `BuiltinMetaLayer` fallback.

### Semantics-preserving compression

This is the key differentiator. Clean-CTX is **not** doing textual minification:

* **Architectural invariant (IRPAT-001)** - consumptive IR transformations must never consume a `DefMethod` while leaving surviving operations that reference it orphaned. The compiler validates every transformation so compression does not silently destroy relationships.
* **Deterministic, validated** - the IR validator (E001-E010) catches dangling references, orphaned methods, and inconsistent annotations before output reaches the consumer.
* **Identity survives transformation** - entity identity and semantic edges persist through compression, delta transport, and edits.

### Token and context efficiency

A major benefit - but a *consequence* of better context compilation, not the whole product:

| Mechanism | What it affects |
|-----------|-----------------|
| **Semantic compression** (Low/Medium/High/Edit/Verbatim) | 61-97% smaller representations vs raw source |
| **Delta transport** | Up to 53% CPU/latency savings on repeat calls |
| **CBM symbol filtering** | Queries the separate CBM (codebase-memory-mcp) server for symbol-importance scores and filters its output before it reaches the LLM |
| **Tool output filtering** (26 filters) | Compresses build/lint/test CLI output |
| **Prompt caching** (proxy) | ~90% API cost savings on repeated turns |

---

## How it works

```text
Source file
    ↓
CoreIRPass → LanguageLayerPass → MetaLayerPass → PatternRecognitionPass → ValidationPass
    ↓                    ↓                ↓
Compact IR      Semantic edges      Φ markers
    ↓                    ↓
    └──── WorkspaceIndex ────┘
              ↓
    ┌─────────────────────────┐
    │  workspace_query (MCP)  │
    │  provide_code_context   │
    │  compress_code_context  │
    └─────────────────────────┘
```

1. **Parse** - tree-sitter grammars produce a concrete syntax tree.
2. **Compile to IR** - `CoreIRPass` emits a typed instruction stream; `LanguageLayerPass` adds behavioral annotations; `MetaLayerPass` extracts framework-specific semantic edges.
3. **Compress** - `PatternRecognitionPass` collapses recognized patterns into compact opcodes; `ValidationPass` enforces IRPAT-001 and structural validity.
4. **Index** - semantic edges flow into `WorkspaceIndex` for cross-file queries.
5. **Serve** - MCP tools expose context compilation and semantic queries to AI agents.

---

## Quick Start

### Prerequisites

* [Rust](https://rustup.rs/) 1.85+ (edition 2024)

### Install

```bash
# Clone and build (release binary)
git clone https://github.com/codeliftsleep2/Clean-CTX.git
cd Clean-CTX
cargo build --release

# The binary is at: target/release/clean-ctx.exe (Windows) or target/release/clean-ctx (Linux/Mac)
```

### Language & Feature Selection

Clean-CTX uses Cargo **feature flags** to control which languages and meta-layers are compiled into the binary.

| Category | Feature | Implies | Includes | Build With | Default |
|----------|---------|---------|----------|------------|---------|
| **Language** | `typescript` | - | Base TypeScript/JavaScript grammar | `--features typescript` | Yes |
| **Language** | `csharp` | - | Base C# grammar | `--features csharp` | Yes |
| **Language** | `rust` | - | Base Rust grammar | `--features rust` | No |
| **Language** | `java` | - | Base Java grammar | `--features java` | No |
| **Meta-Layer** | `angular` | `typescript` | Components, Services, DI, Pipes, Directives, Modules, `@Input`/`@Output`, NgRx, RxJS, Signals, Routing, PrimeNG, template/style extraction, bundle graph | `--features angular` | Yes |
| **Meta-Layer** | `spring_boot` | `java` | Controllers, Services, Repositories, `@Autowired`, `@ConfigurationProperties`, beans | `--features spring_boot` | No |
| **Meta-Layer** | `dotnet` | `csharp` | ASP.NET Core, EF Core, SignalR, AutoMapper, DI, validation | `--features dotnet` | Yes |

Default features give you **TypeScript with Angular meta-layer, C#, and .NET enrichment** - the most common full-stack combination. Everything else is opt-in:

```bash
# Default build: TypeScript + Angular + C# + .NET
cargo build --release

# TypeScript + Angular only (no C#)
cargo build --release --no-default-features --features typescript,angular

# .NET/C# only (no TypeScript, Angular)
cargo build --release --no-default-features --features csharp,dotnet

# All languages + all meta-layers
cargo build --release --features rust,java,spring_boot
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

Restart your editor. The tools `provide_code_context`, `compress_code_context`, `diff_code_context`, `diff_commits`, `delta_code_context`, `apply_delta`, `apply_edit`, `restore_context`, `context_stats`, `context_history`, `save_context`, `list_sessions`, `replay_history`, `purge_old_deltas`, and `workspace_query` will be available.

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

### Query the workspace semantic graph

```json
{
  "name": "workspace_query",
  "arguments": {
    "type": "resolve_selector",
    "name": "app-user-card"
  }
}
```

Returns the component entity that exposes the `app-user-card` selector. Other query types: `find_entities`, `forward_edges`, `reverse_edges`, `entities_in_file`, `transitive_dependencies`, `has_cycle`.

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

**Output (SCHEMA v2):**
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

### View compression dashboard

```json
{
  "name": "context_stats",
  "arguments": {}
}
```

---

## Response Notation (SCHEMA v2)

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

**High fidelity** adds `cf:` (control flow), `df:` (reads/writes), `se:` (side effect), `ec:` (execution context). **Edit fidelity appends each focused method verbatim source body** - byte-exact. Types render exactly as captured.

The full SCHEMA v2 notation reference is in [`docs/COMPILER_IR.md`](docs/COMPILER_IR.md).

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
| Semantic intelligence | OK: Typed `SemanticEdge`/`EntityRef`/`SemanticRelation` model; 30+ relation types; WorkspaceIndex with forward/reverse traversal, selector/injection resolution, transitive deps, cycle detection |
| Workspace queries | OK: `workspace_query` MCP tool - `find_entities`, `forward_edges`, `reverse_edges`, `entities_in_file`, `transitive_dependencies`, `has_cycle` |
| Transport | OK: Stateful IR delta transport - compile once, send deltas thereafter |
| Pass Architecture | OK: Composable IRPass pipeline (Core → Language → Meta → Pattern* → Validation), IR validator (E001–E010), query engine, semantic delta intents |
| CBM Integration | OK: CBM (codebase-memory-mcp) runs as a separate local server; Clean-CTX launches it as a subprocess, indexes the repo + additional roots, captures its graph output, and filters/compresses it (filter-first) before it reaches the LLM |
| Persistence | OK: SQLite cross-session persistence with three-tier reliability |
| Proxy | OK: Multi-platform proxy (Anthropic/OpenAI/Generic) with auto-cache + tool filters |
| Filters | OK: 26 built-in TOML filters - cargo, npm, eslint, docker, go, and more |

Unsafe code is test-only (environment-variable manipulation).

---

## Documentation

| Document | Audience | Content |
|----------|----------|---------|
| [`README.md`](README.md) | **Users** | What Clean-CTX is, features, quick start, usage |
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

CC0-1.0. See [`LICENSE`](LICENSE).
