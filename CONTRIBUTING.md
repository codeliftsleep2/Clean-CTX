# Contributing to Clean-CTX

Thank you for your interest in contributing! Clean-CTX is an open-source project, and we welcome contributions that improve the codebase, fix bugs, add features, or improve documentation.

---

## Quick Links

- [Developer Documentation](docs/DEVELOPER_DOCUMENTATION.md) — build, test, extend
- [Architecture Overview](docs/ARCHITECTURE_OVERVIEW.md) — system design and module structure
- [Compiler IR Specification](docs/COMPILER_IR.md) — IR protocol, delta transport, wire format
- [Angular Meta-Layer](docs/ANGULAR_META_LAYER.md) — framework annotation layer design
- [Performance Guide](docs/PERFORMANCE.md) — benchmarks and optimization
- [Security Guide](docs/SECURITY.md) — compliance and hardening
- [Troubleshooting Guide](docs/TROUBLESHOOTING.md) — common issues
- [Changelog](docs/CHANGELOG.md) — version history
- [Roadmap](docs/ROADMAP.md) — future plans and priorities

---

## Getting Started

```bash
# Clone the repository
git clone https://github.com/codeliftsleep2/Clean-CTX.git
cd Clean-CTX

# Build
cargo build

# Run tests
cargo test

# Run linter (must pass before PR)
cargo clippy --all-targets -- -D warnings
```

**Prerequisites:** Rust 1.85+ (edition 2024). No external runtimes or dependencies.

---

## Code Quality Gates

Every pull request **must** pass these checks. The full checklist is in [`PULL_REQUEST_TEMPLATE.md`](PULL_REQUEST_TEMPLATE.md) — GitHub will auto-load it when you open a new PR.

1. **`cargo check`** — compiles without errors
2. **`cargo clippy --all-targets -- -D warnings`** — zero warnings (treated as errors)
3. **`cargo test`** — all 945 tests pass
4. **`cargo audit`** — no known security vulnerabilities
5. **No new `.unwrap()` calls** without a `// SAFETY:` comment explaining why it cannot fail
6. **No `let _ = ...` dead-code suppression** — remove the unused variable instead
7. **No `#![allow(...)]`** annotations without a `// SAFETY:` or `// Phase N:` comment

---

## Pre-commit Checklist

```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test && cargo audit
```

---

## How to Contribute

### Reporting Bugs

1. Check the [Troubleshooting Guide](docs/TROUBLESHOOTING.md) to see if your issue is already addressed
2. Search existing issues to avoid duplicates
3. Include:
   - Binary version (build date or commit hash)
   - Operating system and Rust version (`rustc --version`)
   - Full error output
   - Steps to reproduce
   - Input data (redacted if necessary)

### Suggesting Enhancements

Open an issue with:
- A clear description of the proposed change
- Why it improves the project
- If applicable, a sketch of the implementation approach

### Adding a Language

See the step-by-step guide in [Developer Documentation → Adding a New Language](docs/DEVELOPER_DOCUMENTATION.md#adding-a-new-language).

**Summary:**
1. Add the tree-sitter grammar to `Cargo.toml`
2. Add tree-sitter queries to `src/queries.rs`
3. Register in `src/compression/language.rs`
4. Update documentation
5. Add tests

### Adding a Tool

See the step-by-step guide in [Developer Documentation → Adding a New Tool](docs/DEVELOPER_DOCUMENTATION.md#adding-a-new-tool).

**Summary:**
1. Add handler function in `src/mcp/tools.rs`
2. Register in `get_tool_definitions()`
3. Wire up dispatch in `dispatch_tools_call()`
4. Add tests in `src/tests/mcp/tools.rs`

---

## Project Structure

```
src/
├── main.rs                 # Entry point (3 lines)
├── lib.rs                  # Module declarations
│
├── mcp/                    # MCP server layer (JSON-RPC stdio)
├── compression/            # Core compression engine
├── diff/                   # AST-level structural diff
├── compaction/             # AST node compaction
├── decompression/          # Opcode → readable expansion
├── dictionary/             # Symbol and path registries
├── cache.rs                # Content hash + baseline cache
├── config.rs               # .clean-ctx.json configuration
├── queries.rs              # Tree-sitter query patterns
├── analytics.rs            # Token counting (cl100k BPE)
└── protocol.rs             # JSON-RPC types
```

---

## Design Principles

- **Single Responsibility Principle** — each module owns exactly one concern
- **No unsafe code** — the entire codebase is safe Rust (`#![forbid(unsafe_code)]` would pass)
- **No network dependencies** — stdio-only MCP transport
- **HashMap over BTreeMap** — no caller iterates in sorted order
- **Explicit errors, not panics** — all error paths return `Result`, never `.unwrap()`
- **Cached at startup** — config and BPE data are loaded once and reused

---

## Testing Conventions

Tests live in `src/tests/` and are referenced from their respective source modules via `#[path]`:

```rust
#[cfg(test)]
#[path = "../tests/compression/fidelity.rs"]
mod tests;
```

See [Developer Documentation → Testing Conventions](docs/DEVELOPER_DOCUMENTATION.md#testing-conventions) for naming, coverage expectations, and running tests.

---

## Documentation

| Document | Audience | Content |
|----------|----------|---------|
| `README.md` | Users | Installation, configuration, usage |
| `CONTRIBUTING.md` | Contributors | This file — overview and process |
| `docs/DEVELOPER_DOCUMENTATION.md` | Contributors | Detailed build/test/extend guide |
| `docs/ARCHITECTURE_OVERVIEW.md` | Architects | System design, module structure |
| `docs/COMPILER_IR.md` | Architects | Compiler IR protocol, delta state transport, wire format |
| `docs/ANGULAR_META_LAYER.md` | Developers | Angular Meta-Layer design, marker vocabulary, graph |
| `docs/EDIT_TYPE.md` | Developers | Edit categorization for delta transport |
| `docs/PERFORMANCE.md` | Architects | Benchmarks, caching, optimization |
| `docs/SECURITY.md` | Enterprise admins | Compliance, hardening, deployment |
| `docs/TROUBLESHOOTING.md` | Users | Common issues and resolutions |
| `docs/CHANGELOG.md` | All | Version history |
| `docs/INTELLIGENCE_LAYER_PLAN.md` | Architects | Intelligence Layer: PageRank scoring, blast radius, token budget packing |
| `docs/ROADMAP.md` | Contributors | Future plans and priorities |

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.
