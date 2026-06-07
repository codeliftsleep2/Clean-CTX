# Clean-CTX — Developer Documentation

**Version:** 0.1.0
**Last updated:** 2026-06-07

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Project Overview](#project-overview)
3. [Building & Testing](#building--testing)
4. [Codebase Organization](#codebase-organization)
5. [Adding a New Language](#adding-a-new-language)
6. [Adding a New Tool](#adding-a-new-tool)
7. [Adding a New Opcode](#adding-a-new-opcode)
8. [Configuration System](#configuration-system)
9. [Testing Conventions](#testing-conventions)
10. [Code Quality Gates](#code-quality-gates)

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
│   ├── mod.rs              # run() entry point
│   ├── server.rs           # Stdin/stdout JSON-RPC loop
│   ├── router.rs           # Method dispatch
│   ├── handlers.rs         # Lifecycle + discovery handlers
│   ├── tools.rs            # Tool implementations
│   ├── prompts.rs          # System prompts
│   ├── workspace.rs        # Workspace compression
│   └── state.rs            # McpState (shared session state)
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
│   └── streaming.rs        # Streaming orchestrator
│
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

## Code Quality Gates

Every pull request must pass these checks:

1. **`cargo check`** — compiles without errors
2. **`cargo clippy --all-targets -- -D warnings`** — zero warnings (treated as errors)
3. **`cargo test`** — all 121+ tests pass
4. **`cargo audit`** — no known security vulnerabilities
5. **No new `#![allow(...)]`** annotations without a `// SAFETY:` or `// Phase N:` comment
6. **No new `.unwrap()` calls** without a `// SAFETY:` comment explaining why it cannot fail
7. **`let _ = ...`** dead-code suppression is not accepted — remove the unused variable

### Pre-commit checklist

```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test && cargo audit