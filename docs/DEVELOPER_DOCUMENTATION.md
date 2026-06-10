# Clean-CTX — Developer Documentation

**Version:** 0.1.6
**Last updated:** 2026-06-10

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Project Overview](#project-overview)
3. [Building & Testing](#building--testing)
4. [Codebase Organization](#codebase-organization)
5. [Zero-Touch Workflow](#zero-touch-workflow)
6. [Persistence Layer](#persistence-layer)
7. [Adding a New Language](#adding-a-new-language)
8. [Adding a New Tool](#adding-a-new-tool)
9. [Adding a New Opcode](#adding-a-new-opcode)
10. [Adding a New Φ Marker](#adding-a-new--marker)
11. [Angular Meta-Layer Architecture](#angular-meta-layer-architecture)
12. [Configuration System](#configuration-system)
13. [Testing Conventions](#testing-conventions)
14. [Code Quality Gates](#code-quality-gates)

---

## Getting Started

```bash
# Clone the repository
git clone https://github.com/codeliftsleep2/Clean-CTX.git
cd Clean-CTX

# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run tests
cargo test

# Run linter (must pass before PR)
cargo clippy --all-targets -- -D warnings
```

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
│   ├── tools.rs            # Tool implementations + persistence hooks
│   ├── prompts.rs          # System prompts (cleanctx-notation + dashboard)
│   ├── workspace.rs        # Workspace compression
│   ├── state.rs            # McpState (shared session state + persistence)
│   ├── heuristics.rs       # Heuristics engine (fidelity + strategy selection)
│   ├── context_store.rs    # ContextStore trait + InMemoryContextStore
│   ├── sqlite_store.rs     # SqliteStore (SQLite-backed ContextStore)
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
│   ├── opcodes.rs          # CoreOp enum
│   ├── wire.rs             # ir_to_wire: CompiledIR → tuple format
│   ├── string_table.rs     # Compact index format
│   ├── delta.rs            # DeltaComputer, IRDelta
│   ├── replay.rs           # ContextState: apply deltas to baseline
│   ├── binary_wire.rs      # Binary wire format for IR transport
│   └── layers/             # Language-specific compilation layers
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

The persistence layer provides **cross-session persistence** for compression contexts via SQLite. It is enabled by setting the `CLEANCTX_PERSISTENCE_DB` environment variable.

### Architecture

```
ContextStore trait (src/mcp/context_store.rs)
    ├── InMemoryContextStore  (session-scoped, in RAM)
    └── SqliteStore           (cross-session, SQLite on disk)
```

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

## Adding a New Language

Adding a new language to Clean-CTX requires changes in 4 locations. Here is the step-by-step guide.

### Step 1: Add the tree-sitter grammar dependency

In `Cargo.toml`, add the new grammar:

```toml
# SAFETY: Must match tree-sitter 0.20.x ABI.
tree-sitter-python = "=0.20.0"
```

### Step 2: Add tree-sitter queries

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

## Angular Meta-Layer Architecture

The Meta-Layer is a three-tier system that enriches compressed output with Angular framework context. It is **purely additive** — it never modifies existing TS compaction output.

### Module Overview

| Module | Tier | Purpose |
|--------|------|---------|
| `detect.rs` | 0 | Heuristic: is this file Angular? |
| `decorators.rs` | 1 | Extract `@Component`/`@Injectable`/etc. from class captures |
| `markers.rs` | 1 | Φ marker construction, rendering, and expansion |
| `bundler.rs` | 2 | Resolve `*.component.ts` → `.html` + `.scss` file triplets |
| `template.rs` | 2 | tree-sitter-html Angular-syntax template shape extraction |
| `style.rs` | 2 | CSS/SCSS class selector and variable extraction |
| `footer.rs` | 2 | `§ΦMAP` workspace footer formatting |
| `graph.rs` | 3 | Cross-file DI injection + selector linkage graph |
| `graph_state.rs` | 3 | `McpState` integration wrapper |

### Detection Strategy (`detect.rs`)

A file is "Angular" if it meets either condition:
1. Contains a **strong** decorator signal: `@Component(`, `@Injectable(`, `@NgModule(`, `@Directive(`, `@Pipe(`, etc.
2. Imports from `@angular/core` AND has at least one `@Input`/`@Output` (weak signal paired with import).

Plain `@Input`/`@Output` alone (no strong signal, no `@angular/core` import) returns `false` — these decorators are also used by MobX/Vue.

### Extraction Strategy (`decorators.rs`)

The extractor walks raw text of `class.root` tree-sitter captures (no AST re-parse):
1. Find the text before `class <Name>` keyword (the "head")
2. Collect all `@...(...)` decorator calls from the head
3. Classify each decorator and emit the corresponding Φ marker line
4. Optionally scan the class body for field-level `@Input`/`@Output` and signal-based APIs

### Fidelity Control (F-ANG-23)

The `fidelity` parameter controls Meta-Layer verbosity:

| Fidelity | Class-level | Field-level | DI/Signals |
|----------|------------|-------------|------------|
| Low | Φcmp, Φsvc, Φmod, Φdir, Φpipe | — | — |
| Medium | All above | Φin, Φout | — |
| High | All above | Φin, Φout | Φinjects, Φmodel, input()/output() signals |

### Template Extraction (`template.rs`)

Uses `tree-sitter-html` to parse Angular templates and extract structural shape:
- **Tags** and **custom elements** (tags containing a hyphen)
- **Property bindings** (`[prop]="expr"`) and **event bindings** (`(event)="handler"`)
- **Two-way bindings** (`[(ngModel)]="value"`)
- **Structural directives** (`*ngIf`, `*ngFor`)
- **Modern control flow** (`@if`, `@for`, `@switch`) — detected via text-node word-boundary scanning since these are not valid HTML
- **Defer blocks** with trigger extraction (`@defer (on viewport)`)
- **`@let` declarations** (Angular 18+)
- **Self-closing tags** (`<app-avatar />`)

Raw HTML content is **never** included — only the structural summary.

### Cross-File Graph (`graph.rs`)

Built once per `compress_workspace` call using the typestate pattern:

1. **Collection phase** — each file's `extract_graph_entries` produces `(class_name, file_alias, kind, selector, injects, pipe_name)` tuples
2. **Registration** — `AngularGraphBuilder::register_class` adds entries to the mutable builder
3. **Resolution** — `builder.build()` consumes the builder, builds `injected_by` reverse edges, and returns the immutable `AngularGraph`
4. **Querying** — `resolve_inject_type("UserService")` → `"UserService@α12"`, `resolve_selector("app-user-card")` → `"UserCardComponent@α9"`

The graph is purely in-memory and discarded after the workspace manifest is emitted.

### Adding a New Framework Meta-Layer

The Meta-Layer pattern is designed to be extensible to other frameworks (React, Vue, Svelte). To add a new framework:

1. Create `src/<framework>_meta/` with the same module structure
2. Implement detection (`is_<framework>_file`)
3. Define a marker vocabulary in `markers.rs`
4. Implement extraction in `decorators.rs` (or equivalent)
5. The `Φ` prefix is Angular-specific; use a different Greek letter for new frameworks (e.g., `Ψ` for React, `Ω` for Vue)

---

## Configuration System

The `CleanCtxConfig` struct (in `src/config.rs`) is loaded from `.clean-ctx.json` at the project root:

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
    },
    "custom_markers": {
        "$custom": "Custom marker description"
    },
    "diff_compression": true,
    "workspace_type_detection": true
}
```

The config is:
- Loaded once at server startup (cached in a `OnceLock`)
- Shared across all tools via `McpState.config`
- Immutable for the session — edits require a server restart

### Exclusion globs

Exclusion patterns use a simple two-tier matcher:

| Pattern Type | Example | Matching Behavior |
|-------------|---------|-------------------|
| Plain name | `"dist"` | Matches any path segment literally named `dist` (NOT `distribute`) |
| Dot pattern | `".test."` | Substring match against the file name, so `".test."` matches `file.test.ts` |
| Glob pattern | `"*.spec.ts"` | Standard `*`/`?` glob match against the file name |

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
```

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

## Code Quality Gates

Every pull request must pass these checks:

1. **`cargo check`** — compiles without errors
2. **`cargo clippy --all-targets -- -D warnings`** — zero warnings (treated as errors)
3. **`cargo test`** — all 798+ tests pass
4. **`cargo audit`** — no known security vulnerabilities
5. **No new `#![allow(...)]`** annotations without a `// SAFETY:` or `// Phase N:` comment
6. **No new `.unwrap()` calls** without a `// SAFETY:` comment explaining why it cannot fail
7. **`let _ = ...`** dead-code suppression is not accepted — remove the unused variable

### Pre-commit checklist

```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test && cargo audit