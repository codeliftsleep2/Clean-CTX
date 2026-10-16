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
* **Semantic identity and provenance** - `(domain, entity_type, name)` is the WorkspaceIndex semantic tuple. Multiple declaration occurrences may share it; file/span provenance and language-specific binding establish exact declaration identity when a consumer needs it.
* **Semantic relationships** - typed, producer-owned edges such as `Injects`, `HasSelector`, `Dispatches`, `Selects`, `HandlesAction`, `RouteMapsTo`, `Autowired`, `ControllerAction`, and `HasEntity`, with source provenance. An edge is authoritative only within its producer's supported domain and current evidence generation.
* **Provenance tracking** - every semantic edge records which meta-layer discovered it.

### Workspace semantic indexing

A cross-file semantic graph that survives beyond single-file compression:

* **WorkspaceIndex** - framework-agnostic index of entities and semantic edges across an entire workspace.
* **Semantic-tuple lookup** - `(domain, entity_type, name)` groups semantic occurrences; it is not a universally unique declaration key. A separate convenience lookup finds occurrences by bare name across all domains and types.
* **Forward/reverse relationship traversal** - follow `Injects`, `Dispatches`, `Selects`, and other edges in either direction.
* **Selector resolution** - retain structured selector occurrences and refine to a component only when current exact selector authority yields one unique candidate; zero or multiple candidates remain unresolved.
* **Injection discovery** - resolve an injected type to all workspace entities referenced as `Injects`/`Autowired` targets (reference occurrences; ambiguity preserved).
* **Transitive dependencies** - BFS traversal through DI and routing relationships, with cycle detection.

### Context compilation

Task-relevant context rather than indiscriminate source dumping:

* **Compiles source into LLM-optimized representations** - Low/Medium/High are bounded presentations; Edit and Verbatim add separate exact-source guarantees. Canonical semantic authority and model-facing projection are distinct contracts.
* **Declared projection loss** - bounded presentations may omit supported semantic categories only when the response declares that loss machine-readably. Stronger internal acquisition does not silently upgrade the published fidelity.
* **Single-pass framework evaluation** - one compilation-scoped lexical/capture context is reused across applicability, markers, and semantic-edge extraction, eliminating redundant tree-sitter parses and repeated prefix scans.
* **Layered cache reuse** - unchanged files reuse canonical IR and tokenizer counts, while CBM graph entries and disk partitions remain isolated by canonical project identity.
* **Explicit IR-level delta transport** - `delta_code_context` computes an instruction-level transition and `apply_delta` acknowledges it through a code-side protocol. Ordinary context reads remain complete responses and never silently switch to deltas. Historical benchmark results are retained in [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md), not presented as a universal current guarantee.
* **Structural edits** - `apply_edit` performs byte-exact edits on previously-seen files using the semantic model.
  The exact physical source precondition is revalidated at commit, so an
  externally superseded file is never overwritten merely because replacement
  itself would be atomic.

### MCP integration

Exposes code intelligence and context capabilities to AI coding agents through the Model Context Protocol:

* **`provide_code_context`** - complete model-facing context for either one `filePath` or a failure-isolated `files` batch of 1–8 items; batch callers may choose safe mirrored, structured-only, or indexed content placement. The tool auto-detects file type, selects per-item fidelity, and filters low-importance symbols. Structured delta transport remains explicit through `delta_code_context` / `apply_delta`.
* **`workspace_query`** - single or heterogeneous batched semantic queries: entity lookup, forward/reverse edges, file entities/calls, transitive dependencies, and cycle detection.
* **`compress_code_context` / `restore_context`** - direct compression control and compatible durable reuse. Restore admits the path and validates current source plus scoped canonical/semantic compatibility before persisted state can become live authority.
* **`diff_code_context` / `diff_commits`** - AST-level change-sets, single-file and git ref-range.
* **`delta_code_context` / `apply_delta`** - explicit, code-side IR delta generation and acknowledgement. Clean-CTX exposes both server tools; the repository does not ship an automatic host consumer.
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
* **Deterministic, validated** - the IR validator (E001-E011) catches dangling references, orphaned methods/callers, and inconsistent annotations before output reaches the consumer.
* **Authority survives only compatible transitions** - semantic state is admitted or restored only when current source, canonical compatibility, semantic dependencies, and the owning lifecycle permit it.

### Token and context efficiency

A major benefit - but a *consequence* of better context compilation, not the whole product:

| Mechanism | What it affects |
|-----------|-----------------|
| **Semantic projection** (Low/Medium/High/Edit/Verbatim) | Trades presentation size for explicitly declared detail; results vary by source shape and requested fidelity |
| **Delta transport** | Avoids resending an unchanged full IR when a compatible consumer explicitly owns the delta lifecycle |
| **CBM integration** | Optionally narrows acquisition work; it does not decide Clean-CTX semantic truth or completeness |
| **Tool-output filtering** | Applies configured filters to supported CLI output |
| **Prompt caching** (proxy) | Adds provider cache hints where the configured provider path supports them |

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
   An authoritative source refresh retracts evidence owned by sources no longer
   present; CBM assists discovery but does not determine semantic truth.
5. **Serve** - MCP tools expose context compilation and semantic queries to AI agents.

For authoritative semantics, the pipeline is more precise: acquire evidence
broadly, validate it against one compatible source generation, freeze that
generation, interpret each domain using only its producer-owned authority, earn
any completeness certificate, and only then project at the requested fidelity.
Warm caches may reduce cost; they must not improve truth.

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

Every call returns complete current context. Incremental IR transport is an
explicit `delta_code_context` / `apply_delta` workflow and is never selected
automatically by `provide_code_context`.

### Query the workspace semantic graph

```json
{
  "name": "workspace_query",
  "arguments": {
    "type": "find_entities",
    "name": "UserCardComponent",
    "workspaceRoot": "/path/to/workspace"
  }
}
```

The tool accepts two mutually exclusive request forms: singular `filePath` for
one file, or top-level `files` for 1–8 files (normally two or more). A batch keeps
`workspaceRoot` and `tokenizer` at the top level; each item has a unique `id`,
its own `filePath`, and optional `intent`, `fidelity`, and `focusMethods`.
Inspect every item status. Batch-only `responseMode` controls where exact code
is placed: prefer `structured` for a verified structured-result consumer, omit
it for safe `mirrored` when host capability is unknown, or use `indexed` only
for a verified top-level content consumer. Never guess a compact mode;
`response_mode` reports the projection used.

```json
{
  "name": "provide_code_context",
  "arguments": {
    "workspaceRoot": "/path/to/repo",
    "responseMode": "structured",
    "files": [
      { "id": "overview", "filePath": "src/service.ts", "intent": "overview" },
      { "id": "target", "filePath": "src/controller.ts", "intent": "edit", "focusMethods": ["Controller.update"] }
    ]
  }
}
```

Several independent questions sharing one workspace scope can be sent together:

```json
{
  "name": "workspace_query",
  "arguments": {
    "workspaceRoot": "/path/to/workspace",
    "queries": [
      { "id": "entity", "type": "find_entities", "name": "UserCardComponent" },
      { "id": "callers", "type": "reverse_edges", "name": "UserCardComponent" },
      { "id": "cycles", "type": "has_cycle" }
    ]
  }
}
```

Batch results preserve input order and isolate failures per item. Other query types include `forward_edges`, `entities_in_file`, `transitive_dependencies`, and `calls_in_file`.

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

**Structural output (SCHEMA vNext):**
```
// SCHEMA vNext  @=meta C=class X=extends I=implements F=field M=method $=import p:=params →=return mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias
C SampleService
M doWork p:payload:$s[] → $b
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

## Response Notation (SCHEMA vNext)

Structural `provide_code_context`, `compress_code_context`, and
`restore_context` presentations start with this legend. Byte-exact raw
fallbacks, Angular-template output, and delta acknowledgements use their own
explicit content kinds instead.

```
// SCHEMA vNext  @=meta C=class X=extends I=implements F=field M=method $=import p:=params →=return mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias
```

| Symbol | Meaning |
|--------|---------|
| `C Name` | opens a class scope |
| `cmod:` | class modifiers |
| `cl:` | additional class metadata |
| `X <Parent>` | extends |
| `I <Iface...>` | implements |
| `F name:type` | field |
| `M name` | method; visible signatures distinguish overloads |
| `p:name:type ...` / `→ type` | parameters / return type |
| `mod:` | method modifiers such as `ASYNC`, `STATIC`, or visibility |
| `ctl:` / `pf:` | control summary and typed pattern facts |
| `fl:` | compatibility-only legacy flags when present |
| `$ alias module [names]` | import |
| `T alias = Type` | type alias |
| `P NAME [args]` | structural pattern (CTOR, OBSERVABLE, GETTER, SETTER...) |

**High fidelity** adds `cf:` (control flow), `df:` (reads/writes), `se:` (side
effect), and `ec:` (execution context). **Edit fidelity appends the selected
method bodies as byte-exact source.** Types render exactly as captured.

The full SCHEMA-vNext notation reference is in
[`docs/COMPILER_IR.md`](docs/COMPILER_IR.md).

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
| Release candidate | Package version `0.10.0`; verification evidence is recorded in the release/acceptance documents and CI, not asserted as a timeless README property |
| Languages | OK: TypeScript, C#, Rust, Java with Angular/Spring Boot/.NET meta-layers |
| Semantic intelligence | Typed `SemanticEdge`/`EntityRef`/`SemanticRelation` model; WorkspaceIndex occurrence storage with forward/reverse traversal and bounded domain refinement |
| Workspace queries | OK: single or ordered failure-isolated batch calls across `find_entities`, `forward_edges`, `reverse_edges`, `entities_in_file`, `transitive_dependencies`, `has_cycle`, and `calls_in_file` |
| Transport | OK: Explicit occurrence-aware `dv:2` IR generation/application with complete ordinary context reads |
| Pass Architecture | OK: Composable IRPass pipeline (Core → Language → Meta → Pattern* → Validation), IR validator (E001–E011), query engine, semantic delta intents |
| CBM Integration | Optional. Clean-CTX owns one spawned thin MCP frontend/session and its private pipes; CBM coordinates that frontend with a shared account-local daemon. Clean-CTX does not own the daemon or use CBM as core semantic authority. |
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
| [`docs/ARCHITECTURAL_INVARIANTS.md`](docs/ARCHITECTURAL_INVARIANTS.md) | Architects | Enforced current invariants and their authority |
| [`docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md`](docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md) | Architects | Current semantic identity, acquisition, completeness, dependency, refinement, CBM, and fidelity contracts |
| [`docs/README.md`](docs/README.md) | Everyone | Normative documentation index and historical classification |
| [`docs/ARCHITECTURE_OVERVIEW.md`](docs/ARCHITECTURE_OVERVIEW.md) | Architects | Broad subsystem overview; defer to the two current-authority documents above on semantic authority |
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
| [`docs/README.md`](docs/README.md) | All | Documentation map: current authority, guides, and historical records |
| [`docs/changelogs/CHANGELOG.md`](docs/changelogs/CHANGELOG.md) | All | Version history with all additions, fixes, and deferrals |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | Contributors | Future plans, prioritized items, carry-over from audit |

---

## License

CC0-1.0. See [`LICENSE`](LICENSE).
