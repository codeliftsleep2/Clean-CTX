# Tooling (`docs/agent/tooling.md`)

This is the detailed MCP / code-context tool usage guidance. It is NOT
always-loaded; the concise pointer lives in `.clinerules/engineering.md`.

## `provide_code_context` first

When you need to understand a TypeScript, JavaScript, or C# file, use the
`provide_code_context` MCP tool **FIRST** before using `read_file`. It gives a
compressed but semantically complete view of the file structure, classes,
methods, and types.

Use `provide_code_context` with `intent` based on the task:

- `"overview"` — understanding file structure; lowest token usage
- `"edit"` — planning an edit; balanced detail
- `"refactor"` — major restructuring; highest detail
- `"debug"` — finding issues; medium detail
- `"implement"` — adding new code; high detail

Parameters:

- `filePath` — relative or absolute path to the file.
- `fidelity` / `focusMethods` — fine-tune token cost vs. verbatim detail (this
  server's defaults are appropriate in most cases).

The server handles Angular detection, delta transport, fidelity selection, and
persistence automatically.

## `read_file` fallback

Only use `read_file` directly when:

1. `provide_code_context` fails, or
2. exact line-by-line content is required.

Examples of the latter: regex searches, exact textual replacement, byte-level
analysis, verifying precise surrounding code, investigating a specific
compiler error.

Prefer `provide_code_context` for understanding structure and architecture.

## `context_stats`

To view compression-savings statistics, call `context_stats` with no
arguments. It shows session-level metrics including raw vs. compressed tokens,
delta hit rate, and a per-file breakdown. Use this when useful for
understanding context efficiency.