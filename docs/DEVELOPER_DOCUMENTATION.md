# Clean-CTX — Developer Documentation

> **Owner:** How-to-extend (languages/tools/opcodes/Φ markers) + opcode/marker vocabulary + build/test gates · **Status:** Living reference
> **Version:** 0.4.0 · **Last updated:** 2026-08-24
>
> **Test count:** see `docs/CHANGELOG.md` for the current workspace test count.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Project Overview](#project-overview)
3. [Architecture Overview](#architecture-overview)
4. [Building & Testing](#building--testing)
5. [Codebase Organization](#codebase-organization)
6. [Zero-Touch Workflow](#zero-touch-workflow)
7. [Persistence Layer](#persistence-layer)
8. [Compiler IR Subsystem](#compiler-ir-subsystem)
9. [Adding a New Language](#adding-a-new-language)
10. [Adding a New Tool](#adding-a-new-tool)
11. [Adding a New Opcode](#adding-a-new-opcode)
12. [Adding a New Φ Marker](#adding-a-new--marker)
13. [Meta-Layer Architecture](#meta-layer-architecture)
14. [Configuration System](#configuration-system)
15. [Testing Conventions](#testing-conventions)
16. [Code Quality Gates](#code-quality-gates)

---

## Getting Started

```bash
# Clone the repository
git clone https://github.com/codeliftsleep2/Clean-CTX.git
cd Clean-CTX

# Build (debug) with all default features
cargo build

# Build (release) with default features
cargo build --release

# Build with only specific languages (smaller binary, faster compile)
cargo build --release --no-default-features --features typescript,angular

# Build with .NET meta-layer (in defaults)
cargo build --release

# Run tests with all features
cargo test --all-features

# Run linter (must pass before PR)
cargo clippy --all-targets -- -D warnings
```

**Feature flag system:**

The binary uses Cargo feature flags to select which languages and meta-layers to compile. See `Cargo.toml` `[features]` for the full dependency tree.

| Feature | Implies | Default | Includes |
|---------|---------|---------|----------|
| `typescript` | — | ✅ | Base TypeScript/JavaScript tree-sitter grammar |
| `csharp` | — | ✅ | Base C# tree-sitter grammar |
| `rust` | — | ❌ | Base Rust tree-sitter grammar |
| `java` | — | ❌ | Base Java tree-sitter grammar |
| `angular` | `typescript` | ✅ | Components, Services, DI, Pipes, Directives, Modules, Input/Output, Template/Shape extraction, Style extraction, NgRx, RxJS, Signals, PrimeNG, Bundle graph |
| `spring_boot` | `java` | ❌ | RestController, Controller, Service, Repository, Configuration, RequestMapping, Autowired, Value, Bean, ConfigurationProperties, Cross-file graph |
| `dotnet` | `csharp` | ✅ | ASP.NET Core (Controllers, Actions, Routes, Auth), EF Core (DbContext, DbSet, Entities), SignalR (Hubs, Clients, Streaming), AutoMapper (Profiles, Mappings), JSON Serialization, DI, Validation, Identity, Caching, Logging, Cross-file graph |

**Prerequisites:**
- Rust 1.85+ (edition 2024)
- No external runtimes or dependencies — everything is statically linked

---

## Project Overview

Clean-CTX is an MCP (Model Context Protocol) server that compresses TypeScript and C# source code into a token-efficient notation for LLM consumption. It runs as a single statically-linked binary, communicating over stdin/stdout via JSON-RPC 2.0.

The core workflow is:
1. **Parse** source code with tree-sitter into an AST
2. **Extract** structural nodes (classes, methods, fields, imports, control flow)
3. **Filter** by fidelity level (Low/Medium/High)
4. **Encode** repeated tokens as short opcodes (`$c` = `class`, `$s` = `string`, etc.)
5. **Report** token savings using the cl100k BPE estimator

The **recommended entry point** is `provide_code_context` — a single tool that automatically handles compression, delta transport, Angular detection, and fidelity selection via a heuristics engine.

---

## Architecture Overview

The system architecture is documented in detail in [`docs/ARCHITECTURE_OVERVIEW.md`](ARCHITECTURE_OVERVIEW.md). The key subsystems are:

- **MCP stdio Interface** — JSON-RPC 2.0 request/response loop over stdin/stdout
- **Heuristics Engine** — selects fidelity and strategy per file based on intent, size, language, and Angular detection
- **Compressor Engine** — AST extraction → fidelity filter → opcode encoding → text delta snapshots
- **IR Subsystem** — structured intermediate representation with delta-based state transport (see [Compiler IR Subsystem](#compiler-ir-subsystem))
- **Decompressor** — opcode → readable expansion (precomputed sorted opcodes for O(L×N) performance)
- **Meta-Layer** — framework/dialect-specific annotation layers that enrich compressed output (see [Meta-Layer Architecture](#meta-layer-architecture))
- **Persistence Layer** — built-in SQLite cross-session storage for baselines and deltas, with three-tier reliability (batched writes, exponential backoff retry, JSON file fallback) — see [Persistence Layer](#persistence-layer)

For the full system diagram, module dependency graph, and design decisions (tree-sitter, no network, HashMap over BTreeMap), see [`docs/ARCHITECTURE_OVERVIEW.md`](ARCHITECTURE_OVERVIEW.md).

---

## Building & Testing

```bash
# Debug build (fast iteration)
cargo build

# Release build (optimized, stripped)
cargo build --release

# Run the full test suite
cargo test

# Run a specific test
cargo test fidelity_is_hashable

# Run clippy (must pass with -D warnings)
cargo clippy --all-targets -- -D warnings

# Check for outdated dependencies
cargo outdated

# Audit for security vulnerabilities
cargo audit
```

The CI pipeline (see `.github/workflows/ci.yml`) runs `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo audit` on every push to `main`/`master` and every PR.

---

## Codebase Organization

```
src/
├── main.rs                 # Entry point (3 lines)
├── lib.rs                  # Module declarations
│
├── mcp/                    # MCP server layer
│   ├── mod.rs              # run() entry point + persistence init
│   ├── server.rs           # Stdin/stdout JSON-RPC loop
│   ├── router.rs           # Method dispatch
│   ├── handlers.rs         # Lifecycle + discovery handlers
│   ├── tools.rs            # Tool definitions + get_tool_definitions()
│   ├── tool_handlers.rs    # Tool handler implementations (handle_*)
│   ├── tool_helpers.rs     # Shared helper functions for tool handlers
│   ├── prompts.rs          # System prompts (cleanctx-notation + dashboard)
│   ├── workspace.rs        # Workspace compression
│   ├── workspace_util.rs   # Workspace utility functions
│   ├── state.rs            # McpState (shared session state + persistence)
│   ├── heuristics.rs       # Heuristics engine (fidelity + strategy selection)
│   ├── context_store.rs    # ContextStore trait + InMemoryContextStore
│   ├── sqlite_store.rs     # SqliteStore (SQLite-backed ContextStore)
│   ├── buffered_store.rs   # BufferedStore (three-tier persistence wrapper)
│   └── session_stats.rs    # SessionStats + dashboard rendering
│
├── compression/            # Core compression engine
│   ├── mod.rs              # Public API
│   ├── fidelity.rs         # Fidelity enum
│   ├── language.rs         # Language detection
│   ├── capture_pipeline.rs # Tree-sitter capture extraction
│   ├── markers.rs          # Behavior markers
│   ├── opcodes.rs          # Primitive opcode table
│   ├── symbol_compression.rs # Symbol encoding pass
│   ├── report.rs           # Output formatting
│   ├── pipeline.rs         # Non-streaming orchestrator
│   ├── streaming.rs        # Streaming orchestrator
│   ├── text_delta.rs       # Delta-aware text compression (line-level)
│   └── workspace_symbols.rs # Global symbol table for workspace
│
├── ir/                     # IR Subsystem (Compiler IR + Delta Transport)
│   ├── compiler.rs         # IRCompiler: source → CompiledIR
│   ├── compiler_methods.rs # Compiler method implementations
│   ├── opcodes.rs          # CoreOp enum
│   ├── wire.rs             # ir_to_wire: CompiledIR → tuple format
│   ├── string_table.rs     # Compact index format
│   ├── symbol_table.rs     # IR symbol table for cross-file resolution
│   ├── delta.rs            # DeltaComputer, IRDelta
│   ├── replay.rs           # ContextState: apply deltas to baseline
│   ├── hierarchical.rs     # Hierarchical IR (grouped by file/class/method)
│   ├── positional.rs       # Positional encoding for compact IR format
│   ├── render.rs           # IR rendering to text
│   ├── binary_wire.rs      # Binary wire format for IR transport
│   ├── patterns.rs         # CompressingPatternRecognizer
│   └── layers/
│       ├── mod.rs
│       ├── typescript.rs   # TypeScriptLayer for IR compilation
│       ├── csharp.rs       # C#Layer for IR compilation
│       ├── angular.rs      # AngularMetaLayer for IR compilation
│       ├── rust.rs         # RustLayer for IR compilation
│       └── patterns.rs     # CodePatternRecognizer for IR compilation
│
├── angular_meta/           # Angular Meta-Layer (Phases 1–3)
│   ├── mod.rs              # MetaBlock struct, run_meta_layer entry point
│   ├── detect.rs           # Angular detection heuristic
│   ├── decorators.rs       # @Component/@Injectable/@NgModule/etc. extractor
│   ├── markers.rs          # Φ marker construction & expansion
│   ├── bundler.rs          # File-triplet resolver (*.component.ts → .html + .scss)
│   ├── template.rs         # tree-sitter-html Angular-syntax template extractor
│   ├── style.rs            # CSS/SCSS class selector + variable extractor
│   ├── footer.rs           # §ΦMAP workspace footer formatter
│   ├── graph.rs            # AngularGraph — cross-file DI + selector graph
│   └── graph_state.rs      # AngularGraphHandle — McpState integration
├── diff/                   # AST-level diff engine
├── compaction/             # AST node compaction
├── decompression/          # Opcode → readable output
├── dictionary/             # Path + symbol registries
├── cache.rs                # Content hash + baseline cache
├── config.rs               # .clean-ctx.json config
├── queries.rs              # Tree-sitter query patterns
├── analytics.rs            # Token counting (cl100k BPE)
└── protocol.rs             # JSON-RPC types
```

**Key design principles:**
- **Single Responsibility Principle** — each module owns one concern
- **No unsafe code** — the entire codebase is safe Rust
- **No network dependencies** — stdio-only MCP transport
- **HashMap over BTreeMap** — no caller iterates in sorted order; HashMap is faster
- **Meta-Layer is purely additive** — non-Angular files produce byte-identical output with zero overhead
- **Non-fatal persistence** — DB writes are fire-and-forget; compression never fails due to DB issues

---

## Zero-Touch Workflow

The zero-touch workflow is the **recommended entry point** for any file-related coding task. It orchestrates all subsystems automatically via `provide_code_context`.

### How It Works

1. **Heuristics Engine** (`src/mcp/heuristics.rs`) decides the optimal fidelity and strategy based on:
   - Explicit intent parameter (`"edit"`, `"debug"`, `"overview"`, `"refactor"`, `"implement"`)
   - Explicit fidelity override
   - File characteristics (size, language, Angular detection)
   - Existing baselines (text delta state, IR delta state)
   - Project config overrides

2. **Strategy Dispatch**:
   - `FullCompress` — runs full compression pipeline + IR compilation + persistence save
   - `DeltaTransport` — computes text-level and IR-level deltas + persistence save + delta append

3. **Session Stats** (`src/mcp/session_stats.rs`) records compression metrics for the dashboard

### Tools

| Tool | Purpose |
|------|---------|
| `provide_code_context` | **Single entry point** — auto-detects, selects fidelity, uses delta transport on subsequent calls |
| `restore_context` | Force full re-compression, clearing all baselines and DB entries |
| `context_history` | View compression history and delta savings for tracked files |
| `context_stats` | Dashboard: token savings, compression stats, session metrics |

### Adding a New Strategy

To add a new strategy to the heuristics engine:

1. Add a variant to `ContextStrategy` in `src/mcp/heuristics.rs`
2. Add selection logic in `decide()` function
3. Add a dispatch arm in `handle_provide_code_context()` in `src/mcp/tools.rs`
4. Add tests in `src/tests/mcp/heuristics.rs`

---

## Persistence Layer

The persistence layer provides **cross-session persistence** for compression contexts via SQLite. It is **enabled by default** (stored in `.clean-ctx/persistence.db` relative to the project root) and can be disabled in `.clean-ctx.json` with `"persistence": { "enabled": false }`.

### Architecture

```
ContextStore trait (src/mcp/context_store.rs)
    ├── InMemoryContextStore  (session-scoped, in RAM)
    ├── SqliteStore           (cross-session, SQLite on disk)
    └── BufferedStore         (three-tier reliability wrapper)
```

The `BufferedStore` wraps `SqliteStore` with a **three-tier reliability stack**:
1. **Batched writes** — operations queue in memory and flush as a single SQLite transaction when the buffer reaches 5 pending ops (or on explicit flush)
2. **Retry with exponential backoff** — transient DB failures (file lock, WAL contention) are retried up to 3 times with [0ms, 50ms, 200ms] delays
3. **JSON file fallback** — if all retries fail, data writes as standalone JSON files in `.clean-ctx/fallback/`. On the next successful flush, fallback files are re-imported automatically

### ContextStore Trait

The `ContextStore` trait abstracts how compression baselines, deltas, and metadata are persisted:

```rust
pub trait ContextStore {
    fn save_context(&mut self, file_path: &str, fidelity: Fidelity, 
                    compressed_output: &str, ir_blobs: Option<&[u8]>, 
                    source_hash: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn load_latest(&self, file_path: &str) -> Result<Option<StoredContextMeta>, ...>;
    fn has_context(&self, file_path: &str) -> bool;
    fn append_delta(&mut self, context_id: &str, delta_payload: &[u8], 
                    edit_type: Option<&str>) -> Result<(), ...>;
    fn delta_count(&self, context_id: &str) -> usize;
    fn clear_file(&mut self, file_path: &str);
}
```

### SqliteStore

The `SqliteStore` implementation (`src/mcp/sqlite_store.rs`) provides:

- **WAL mode** for concurrent read/write safety
- **Schema versioning** via `_schema_version` table
- **Content-hash deterministic IDs** (`ctx-{sha256_hex}`) for idempotent saves
- **`binary_wire::encode/decode`** for IR BLOB serialization
- **`load_context_with_deltas()`** — replay baseline + deltas from DB
- **`rebuild_stats()`** — reconstruct SessionStats from persisted data
- **`purge_old_deltas(days)`** — trim old history

### Schema (v1)

- **`contexts`** — baselines (content-hash PK, IR BLOB, fidelity, pretty text)
- **`deltas`** — sequential delta payloads (FK → contexts, auto-increment edit_sequence)
- **`symbols`** — symbol table entries (FK → contexts, phi markers)
- **`sessions`** — workspace session tracking
- **`_schema_version`** — migration version tracking

### Persistence Hooks

Persistence hooks fire automatically in:
- `provide_code_context` → `FullCompress` path (baseline save)
- `provide_code_context` → `DeltaTransport` path (baseline + delta save)
- `restore_context` → DB clear on file reset

### Adding a New ContextStore Implementation

To add a new storage backend:

1. Create a new file `src/mcp/<backend>_store.rs`
2. Implement the `ContextStore` trait
3. Add the store variant to `McpState` in `src/mcp/state.rs`
4. Add initialization logic in `src/mcp/mod.rs`
5. Add tests in `src/tests/mcp/<backend>_store.rs`

---

## Compiler IR Subsystem

The Compiler IR subsystem transforms source code into a structured intermediate representation (IR) for efficient delta-based state transport. It is documented in full in [`docs/COMPILER_IR.md`](COMPILER_IR.md). Key capabilities:

- **Compilation** — `IRCompiler` transforms source → `CompiledIR` (a sequence of structured instructions with CoreOp opcodes)
- **Wire formats** — `ir_to_wire` serializes CompiledIR into compact tuple format; `binary_wire::encode/decode` provides a binary BLOB format for persistence
- **String tables** — reduces repeated identifiers to compact index references
- **Delta transport** — `DeltaComputer` computes instruction-level deltas between two IR states; deltas are represented as `IRDelta` envelopes with `+`/`-`/`~` operations
- **State replay** — `ContextState` maintains a baseline and applies deltas incrementally without re-compilation
- **Language-specific compilation layers** — `src/ir/layers/` contains per-language compilation rules

The IR subsystem powers the `delta_code_context` MCP tool and the persistence layer's delta append/replay features.

For the full protocol specification, state machine lifecycle, and all phase implementations (A through H), see [`docs/COMPILER_IR.md`](COMPILER_IR.md).

---

## Adding a New Language

Adding a new language to Clean-CTX requires changes in 5 locations. Here is the step-by-step guide.

### Step 1: Add the Cargo feature flag and tree-sitter grammar dependency

In `Cargo.toml`, add the new feature flag and grammar:

```toml
[features]
# ...existing features...
python = ["dep:tree-sitter-python"]  # NEW: language feature

[dependencies]
# SAFETY: Must match tree-sitter ABI via tree-sitter-sys.
tree-sitter-python = { version = "0.23", optional = true }  # NEW
```

The feature flag must be added to the `[features]` section so users can toggle it at build time:
```bash
cargo build --release --no-default-features --features python
```

### Step 2: Register the language in `src/layers/registry.rs`

In `src/layers/registry.rs`, add the new language layer behind its feature flag:

```rust
#[cfg(feature = "python")]
reg.languages.push(Box::new(crate::layers::language::PythonLayer::new()));
```

### Step 3: Add tree-sitter queries

In `src/queries.rs`, add the query patterns for the new language:

```rust
/// Tree-sitter query for Python.
pub static PY_QUERY: &str = concat!(
    // Classes
    "(class_definition body: (block) @class.body) @class.root",
    " (identifier) @class.name",
    // Methods (functions in class body)
    "(function_definition body: (block) @method.body) @method.root",
    " (identifier) @method.name",
    // ...
);
```

Queries use the standard tree-sitter query syntax. Each capture name follows the convention:
- `@<node>.root` — the outer container node
- `@<node>.name` — the name identifier
- `@<node>.body` — the body block

### Step 3: Register the language in `language.rs`

In `src/compression/language.rs`, add the new extension and grammar:

```rust
pub fn language_for_extension(extension: &str) -> Option<(Language, &'static str)> {
    match extension {
        "ts" | "js" => Some((tree_sitter_typescript::language_typescript(), queries::TS_QUERY)),
        "cs" => Some((tree_sitter_c_sharp::language(), queries::CS_QUERY)),
        "py" => Some((tree_sitter_python::language(), queries::PY_QUERY)),  // NEW
        _ => None,
    }
}
```

If the language has distinctive content signatures, also update `looks_like_csharp` (or create a more general heuristic) so the `diff_code_context` tool can auto-detect it:

```rust
pub fn looks_like_python(source: &str) -> bool {
    source.contains("def ")
        || source.contains("class ")
        || source.contains("import ")
        || source.contains("from ")
}
```

### Step 4: Register the language in `mcp/tools.rs`

Update the `compress_code_context` and `diff_code_context` tool descriptions to mention the new extension. The tools already iterate through `language_for_extension` — no code change needed, just documentation.

### Step 5: Add tests

Create test files for the new language in `src/test_files/` and add language detection tests:

```rust
// In src/compression/language.rs tests
#[test]
fn language_for_extension_handles_python() {
    assert!(language_for_extension("py").is_some());
    assert_eq!(
        language_for_extension("py").unwrap().1,
        queries::PY_QUERY
    );
}
```

---

## Adding a New Tool

Tools are defined in `src/mcp/tools.rs`. To add a new MCP tool:

### Step 1: Define the tool handler

Add a handler function in `src/mcp/tools.rs`:

```rust
pub(crate) fn my_new_tool_handler(
    args: &serde_json::Value,
    state: &mut McpState,
) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = args
        .get("filePath")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing 'filePath'"))?;

    // ... handler logic ...

    Ok(result)
}
```

### Step 2: Register in the tool list

Add the tool definition to the `get_tool_definitions()` function:

```rust
pub(crate) fn get_tool_definitions() -> Vec<serde_json::Value> {
    vec![
        // ... existing tools ...
        json!({
            "name": "my_new_tool",
            "description": "Does something useful",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": {
                        "type": "string",
                        "description": "Path to the file"
                    }
                },
                "required": ["filePath"]
            }
        }),
    ]
}
```

### Step 3: Wire up the dispatch

Add a match arm in `dispatch_tools_call`:

```rust
"my_new_tool" => {
    let result = my_new_tool_handler(args, state)?;
    send_response(&json!({
        "jsonrpc": "2.0",
        "id": req.id,
        "result": { "content": [{ "type": "text", "text": result }] }
    }));
}
```

### Step 4: Add tests

Add tests following the pattern in `src/tests/mcp/tools.rs`:

```rust
#[test]
fn my_new_tool_happy_path() {
    // Arrange
    let mut state = McpState::new(CleanCtxConfig::default());
    let args = json!({ "filePath": "C:/test/file.ts" });

    // Act
    let result = my_new_tool_handler(&args, &mut state);

    // Assert
    assert!(result.is_ok());
    assert!(result.unwrap().contains("expected content"));
}
```

---

## Adding a New Opcode

Opcodes are defined in `src/compression/opcodes.rs`. To add a new built-in primitive:

### Step 1: Add the opcode/token pair

```rust
pub(crate) const PRIMITIVE_OPCODES: &[(&str, &str)] = &[
    // ...existing opcodes...
    ("$m",  "Map"),       // NEW
];
```

### Step 2: Add it to the marker expansion (if applicable)

If the token has a decompression-time marker, add it to `src/compression/markers.rs`.

### Step 3: Update the opcode table assertion

Update the test in `src/compression/opcodes.rs`:

```rust
#[test]
fn table_has_35_entries() {
    // Updated from 34
    assert_eq!(PRIMITIVE_OPCODES.len(), 35);
}
```

---

## Adding a New Φ Marker

The Angular Meta-Layer uses `Φ`-prefixed markers (Φcmp, Φsvc, Φin, etc.) to encode framework annotations. The marker vocabulary is centrally defined in `src/angular_meta/markers.rs`.

### Step 1: Add a variant to `PhiLineKind`

The `PhiLineKind` enum is the **single source of truth** for the marker vocabulary:

```rust
// In src/angular_meta/markers.rs
pub enum PhiLineKind {
    Component,   // Φcmp:
    Service,     // Φsvc:
    // ...existing variants...
    MyNewMarker, // NEW
}
```

### Step 2: Add marker_prefix and expansion arms

```rust
impl PhiLineKind {
    pub fn marker_prefix(self) -> &'static str {
        match self {
            // ...existing arms...
            Self::MyNewMarker => "Φnew:",
        }
    }

    pub fn expansion(self) -> &'static str {
        match self {
            // ...existing arms...
            Self::MyNewMarker => "@MyNew",
        }
    }
}
```

### Step 3: Add to `all_in_expand_order`

Place the new variant at the correct position (longer prefixes before shorter ones):

```rust
pub fn all_in_expand_order() -> &'static [PhiLineKind] {
    &[
        // ...existing entries...
        Self::MyNewMarker, // insert at correct length position
    ]
}
```

### Step 4: Add to `from_token` and `token`

```rust
pub fn from_token(token: &str) -> Option<PhiLineKind> {
    match token {
        // ...existing arms...
        "Φnew" => Some(Self::MyNewMarker),
        _ => None,
    }
}

pub fn token(self) -> &'static str {
    match self {
        // ...existing arms...
        Self::MyNewMarker => "Φnew",
    }
}
```

### Step 5: Add a builder struct (if the marker has data)

```rust
pub struct MyNewLine<'a> {
    pub field_name: &'a str,
}

impl PhiLine for MyNewLine<'_> {
    fn kind(&self) -> PhiLineKind { PhiLineKind::MyNewMarker }
    fn render(&self) -> String { format!("Φnew:{}", self.field_name) }
}

pub fn build_my_new_line(field_name: &str) -> String {
    MyNewLine { field_name }.render()
}
```

### Step 6: Add extraction logic

Wire the new marker into `decorators.rs` (for class-level markers) or the appropriate extractor. Add tests in `src/tests/angular_meta/`.

**Key invariant:** The `expand_phi_in_line` function in the decompressor is **generic** over `PhiLineKind` — it needs no manual updates when you add a new marker. Only the enum and its arms need updating.

---

## Meta-Layer Architecture

Meta-Layers are framework/dialect-specific annotation layers that enrich compressed output with structured metadata. They are **purely additive** — they never modify or interfere with the base compression output and have zero overhead for files that don't match the target framework.

### Concept

A Meta-Layer is a pluggable module that:

1. **Detects** whether a file belongs to a particular framework or dialect
2. **Extracts** framework-specific metadata (decorators, templates, dependency graphs)
3. **Annotates** the compressed output with structured markers using a unique prefix
4. **Augments** workspace-level output with cross-file dependency information

Each Meta-Layer uses a unique Greek-letter prefix for its markers to avoid collisions:
- `Φ` (Phi) — Angular
- Future layers will use their own prefixes (e.g., `Ψ` for React, `Ω` for Vue)

### Angular Meta-Layer (Φ)

The Angular Meta-Layer is the **first reference implementation** of the Meta-Layer pattern. It is a fully built-out, three-tier system that enriches compressed output with Angular framework context. Angular is the only framework currently supported.

For complete details on the Angular Meta-Layer — including detection strategy, decorator extraction, template shape extraction, file-triplet bundling, cross-file dependency graphs, fidelity control, and the full marker vocabulary — see [`docs/ANGULAR_META_LAYER.md`](ANGULAR_META_LAYER.md).

### Edit Type System

The edit type system (`docs/EDIT_TYPE.md`) defines the vocabulary for categorizing edits as deltas are applied to compressed contexts. It provides a structured way to report what kind of changes occurred in a delta: small changes, method-level edits, structural refactors, cross-method changes, and more. This is used by the delta transport layer to annotate delta payloads.

### Extending to New Frameworks

The Meta-Layer pattern is designed to be extensible. To add a new framework Meta-Layer:

1. Create `src/<framework>_meta/` following the module structure in `src/angular_meta/`
2. Implement detection (`is_<framework>_file`)
3. Define a marker vocabulary using a unique Greek letter prefix
4. Implement extraction for the framework's specific annotations
5. Wire the new Meta-Layer into the compiler pipeline
6. Add tests in `src/tests/<framework>_meta/`

See [`docs/ANGULAR_META_LAYER.md`](ANGULAR_META_LAYER.md) for the reference implementation pattern.

---

## Configuration System

The `CleanCtxConfig` struct (in `src/config.rs`) is loaded from `.clean-ctx.json` at the project root. The **complete configuration reference** — including the full `.clean-ctx.json` schema, precedence rules, environment variables, resource limits, persistence, heuristics, meta-layers, intelligence layer, type aliases, cache, and proxy lifecycle — is documented in **[`docs/CONFIGURATION.md`](CONFIGURATION.md)**, which is the single source of truth for configuration.

Key facts for developers:
- Config is loaded once at server startup (cached in a `OnceLock`)
- Shared across all tools via `McpState.config`
- Immutable for the session — edits require a server restart
- Precedence: tool argument > environment variable > config file > default

### Adding a New Config Field

To add a new field to `.clean-ctx.json`:

1. Add the field to the `CleanCtxConfig` struct in `src/config.rs` with a sensible default
2. Add the serde `#[serde(default)]` attribute so old config files continue to work (backward compatible)
3. Wire the field into the relevant subsystem (heuristics, compression, proxy, etc.)
4. Document the field in `docs/CONFIGURATION.md` (the source of truth)
5. Add tests in `src/tests/config.rs`

---

## Testing Conventions

Tests follow these conventions:

### Location
- Each module's tests are in a separate file under `src/tests/`, referenced via `#[path]` attribute:
  ```rust
  #[cfg(test)]
  #[path = "../tests/compression/fidelity.rs"]
  mod tests;
  ```

### Naming
- Test functions use `snake_case` and describe the scenario:
  ```rust
  fn parse_typo_rejected()          // Tests that a typo is rejected
  fn compress_file_cache_hit_...()  // Tests a specific cache-hit scenario
  fn extract_class_name_...()       // Tests a specific extraction variant
  ```

### What to test
- **Happy paths** — normal inputs produce expected output
- **Edge cases** — empty input, unusual Unicode, boundary conditions
- **Error paths** — invalid inputs produce appropriate errors (not panics)
- **Round-trips** — compress → decompress produces the original structure

### Running tests
```bash
cargo test                          # All tests
cargo test fidelity                 # Tests matching "fidelity"
cargo test -- --ignored             # Integration tests (tagged with #[ignore])
cargo test cbm::tests               # Full CBM suite incl. live probes
cargo test --workspace --all-targets --all-features   # Full verification gate
```

### Live-CBM semantic tests

CBM-facing behavior is protected by two layers in `src/tests/cbm/`:

- **Deterministic regression tests** pin the live-captured CBM wire contract so an upstream CBM upgrade cannot silently break us: `in_degree` cells arrive as JSON strings through `query_graph`; tool failures arrive as `result.isError=true` envelopes inside successful JSON-RPC results; there is no `DATAFLOW` edge type in CBM 0.8.1 (a guard fails if one ever appears).
- **Live probes** (`src/tests/cbm/graph_intel.rs`, serial `cbm_live`) spawn a real CBM subprocess per probe and re-index from scratch first (fresh process, fresh index), then assert semantic truth: blast radius equals the ground-truth caller set, unknown projects return `Err` while valid zero-result queries return `Ok(empty)`, dead code covers both `Function` and `Method` labels, and project switches never poison the disk cache.

The self-contained multilingual fixture (`src/tests/cbm/e2e.rs`) builds a temp-dir polyglot project (Rust/C#/Java/TS/JS/HTML/CSS), indexes it, exercises language discovery, web-file nodes and C# cross-language resolution, switches back to the primary project, and asserts the primary stays healthy.

Treat this suite as the authority on upstream CBM behavior: if a live probe fails after a CBM upgrade, the upgrade changed the wire contract or semantics - update the pins deliberately, never silently.

---

## Running Examples

The `examples/` directory contains demonstration programs.

### Token Savings Demo (`token_savings_demo`)

Shows single-pass compression savings across named IR, string-table IR, and compact delta formats:

```bash
cargo run --example token_savings_demo
```

### 50-Edit Simulation (`fifty_edit_simulation`)

Simulates a realistic developer editing session on a ~440-line Angular service (`UserManagementService.ts`). Applies 50 sequential edits across 5 categories (small changes, method-level, structural, cross-method, refactors) and measures token costs across three pipelines:

- **Raw**: Uncompressed BPE token count of full source at each step
- **Clean-CTX full recompression**: Compress at each step via `compress_file`
- **Clean-CTX + delta transport**: Compress once, then send only text-level deltas

```bash
cargo run --example fifty_edit_simulation
```

Output includes a per-edit table (50 rows × 8 columns), final summary with per-pipeline totals and savings percentages, breakdown by edit category, and key insight callouts (break-even point, best/worst case delta savings). See [`docs/PERFORMANCE.md`](PERFORMANCE.md) for the full results.

---

## Observability (A-04)

Clean-CTX uses structured tracing and in-memory metrics for observability. The `observability` module at `src/observability/` provides the infrastructure.

### Initialization

Tracing is initialized once at MCP server startup via `crate::observability::init_tracing()` (called in `src/mcp/server.rs`). It configures the `tracing-subscriber` from environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `CLEAN_CTX_LOG` | `info` | Log level |
| `CLEAN_CTX_LOG_FORMAT` | `text` | Output format (`json` or `text`) |
| `CLEAN_CTX_LOG_FILTER` | — | Fine-grained filter (e.g. `warn,clean_ctx=debug`) |

Example usage:
```bash
# JSON structured logging
CLEAN_CTX_LOG_FORMAT=json clean-ctx

# Debug for clean_ctx module only
CLEAN_CTX_LOG_FILTER=warn,clean_ctx=debug clean-ctx
```

### Adding a New Metric

1. **Add a field** to `MetricsRegistry` in `src/observability/metrics.rs`:
   - Use `Histogram` for latency/size measurements (has `record()` and `record_duration()`)
   - Use `Counter` for event counts
   - Use `Gauge` for current values (workers, queue depth)

2. **Initialize** in `MetricsRegistry::new()`:
   - Histograms: `Histogram::latency_default()` (fixed buckets) or `Histogram::latency_exponential()` (powers of 2)
   - Counters/gauges: `Counter::new()` / `Gauge::new(initial)`

3. **Snapshot** in `MetricsRegistry::snapshot()`:
   - Add the field to `MetricsSnapshot` struct
   - Record it in the snapshot method body

4. **Record** at the point of measurement:
   - `registry.some_histogram.record(value)` or `.record_duration(duration)`
   - `registry.some_counter.increment(delta)`
   - `registry.some_gauge.set(value)` or `.add(delta)`

### Tracing Spans

Key spans are instrumented in hot paths:

| Span | Location | Attributes |
|------|----------|------------|
| `provide_code_context` | `src/mcp/tool_handlers/core.rs` | `file_path`, `fidelity`, `strategy`, `cbm_status`, phase timings |
| `compress_workspace` | `src/mcp/workspace.rs` | `dir_path`, `fidelity`, `file_count`, `total_ms` |
| `cbm_proxy_call` | `src/cbm/bridge.rs` | `tool_name`, `latency_ms`, `output_len`, `is_ok` |
| Dispatcher spans | `src/mcp/dispatcher.rs` | queue wait + execution time histograms |

### Structured Events

Use the `tracing::info!` macro with key-value pairs:
```rust
tracing::info!(
    heuristics_ms = heuristics_ms,
    compile_ms = compile_ms,
    raw_tokens = raw_tokens,
    savings_pct = savings_pct,
    "provide_code_context complete"
);
```

### Error Categories

Use `ErrorCategory` enum instead of free-form strings for metrics:
```rust
use crate::observability::metrics::ErrorCategory;
registry.record_error(ErrorCategory::CbmTimeout);
```

Available categories: `CompressionFail`, `DeltaApplyError`, `CbmTimeout`, `CbmQueryFail`, `IoError`, `ParseError`, `Internal`.

### Dashboard

Call `context_stats` with no arguments to view the metrics dashboard, which includes:
- Operations summary (compressions, deltas, CBM queries, workspace scans)
- Latency histograms (compression, delta, CBM)
- File size distribution
- Resource gauges (active workers, queue depth)
- Error counts by category

For a verbose metrics dump (JSON format), use the `MetricsSnapshot` which implements `Serialize`.

## Code Quality Gates

Every pull request must pass these checks:

1. **`cargo check`** — compiles without errors
2. **`cargo clippy --all-targets -- -D warnings`** — zero warnings (treated as errors)
3. **`cargo test --workspace --all-targets --all-features`** — all 2,507 workspace tests pass (2,167 core library)
4. **`cargo audit`** — no known security vulnerabilities
5. **No new `#![allow(...)]`** annotations without a `// SAFETY:` or `// Phase N:` comment
6. **No new `.unwrap()` calls** without a `// SAFETY:` comment explaining why it cannot fail
7. **`let _ = ...`** dead-code suppression is not accepted — remove the unused variable

### Pre-commit checklist

```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test && cargo audit --ignore RUSTSEC-2025-0009