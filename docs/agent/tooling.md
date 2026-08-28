# Tooling: MCP / Code-Context Tool Selection & Workflow

This document is the **authoritative operational guide** for choosing and
using MCP/code-context tools in this repository. It is NOT always-loaded;
the concise pointer lives in `.clinerules/engineering.md` (Routing table).

Three related files exist with different roles:

| File | Role |
|------|------|
| **`docs/agent/tooling.md`** (this file) | Authoritative detailed agent/tooling guidance |
| `docs/CLAUDE_INTEGRATION_RULES.md` | Portable projection for automated CI/runner workflows |
| `src/mcp/prompts.rs` | Runtime MCP system prompt (injected during initialization) |

Do not treat the more-compact files as contradictory — each serves its
audience. This document governs agent decision-making.

---

## 1. Tool Inventory

### 1.1 Standard Host Tools

| Tool | Description |
|------|-------------|
| `read_files` | Read text/image files from disk. Use for non-code files and exact line inspection. |
| `search_codebase` | Regex search across all files (asynchronous, multiple patterns in one call). |
| `fetch_web_content` | Fetch external URLs. Primarily for documentation/API references. |
| `ask_question` | Ask the user a clarifying question. Use when information is genuinely missing. |
| `run_commands` | Execute non-interactive shell commands. Use for builds, tests, git operations, and command-line verification. |

### 1.2 Clean-CTX Context/Read Tools

All accept `workspaceRoot` to anchor relative paths.

| Tool | Required | Optional | Semantics |
|------|----------|----------|-----------|
| `provide_code_context` | `filePath` | `intent`, `fidelity`, `focusMethods`, `workspaceRoot`, `tokenizer` | **Primary entry point.** Heuristics engine selects fidelity, classifies file, and auto-detects delta transport. Prefer over `compress_code_context`. |
| `compress_code_context` | `filePath` | `fidelity`, `encoding`, `tokenizer`, `workspaceRoot` | Direct AST compilation without heuristics. Lower-level tool; prefer `provide_code_context`. |
| `restore_context` | `filePath` | `fidelity`, `workspaceRoot` | Restore a previously persisted compressed context from the DB. |
| `decompress_code_context` | `compressedText` | — | Expand compressed IR back to human-readable format. |
| `context_stats` | — | `filePath`, `format` | Token-savings dashboard. Shows raw vs compressed tokens, delta hit rate, per-file breakdown. |

### 1.3 Diff/Delta Tools (Read-Only Comparisons)

| Tool | Required | Optional | Semantics |
|------|----------|----------|-----------|
| `diff_code_context` | `filePath` | `workspaceRoot`, `fidelity` | AST-level diff: compares in-session baseline against current on-disk state for a **single file**. |
| `delta_code_context` | `filePath` | `workspaceRoot`, `fidelity` | IR-level delta compression. Uses opcode-level differences between two compiled IRs. |
| `delta_text_context` | `filePath` | `workspaceRoot`, `fidelity` | Text-level line-oriented delta. Source-code files only (not markdown/json/yaml). |
| `diff_commits` | `fromRef` | `toRef`, `workspaceRoot`, `fidelity` | **Multi-file git-ref diff.** Compares an entire workspace between two Git refs and emits per-file AST-level change-sets. Most token-efficient way to understand PR/commit-level changes. |

### 1.4 Edit/Mutation Tools

| Tool | Required | Optional | Semantics |
|------|----------|----------|-----------|
| `apply_edit` | `filePath`, `operations` | `verify` | Tree-sitter-gated single-unit edit. Operations: `replace_body`, `delete`, `insert_after`, `insert_before`. **Requires prior `provide_code_context` at edit/verbatim fidelity in the same session.** After a successful edit, Clean-CTX automatically performs a synchronous CBM `fast` reindex when CBM is available — the agent does not need to call `index_repository` manually. |
| `apply_delta` | `delta`, `currentVersion` | — | Apply an IR delta envelope to the in-session state machine. Low-level; typically not called directly by agents. |
### 1.5 Admin/Persistence Tools

| Tool | Required | Optional | Semantics |
|------|----------|----------|-----------|
| `save_context` | `filePath` | — | Explicitly save in-memory compressed context to the persistence DB. |
| `list_sessions` | — | — | List all persisted file contexts with fidelity, token counts, and timestamps. |
| `replay_history` | `filePath` | `targetSequence`, `fidelity` | Replay delta history from the DB. |
| `purge_old_deltas` | — | `days`, `filePath` | Purge old delta entries. |
| `context_history` | `filePath` | — | View compression history for a specific file. |

### 1.6 CBM / Graph Tools (Architectural Intelligence)

| Tool | Required | Optional | Status |
|------|----------|----------|--------|
| `cbm_proxy` | — | `cbm_tool`, `parameters`, `query`, `project` | **MANDATORY** — the only permitted entry point for all CBM queries. Intercepts raw ~5000-token CBM response and compresses it to ~1100 tokens. |
| `get_cbm_status` | — | — | **ONLY direct CBM call permitted.** Returns `available`, `degraded`, or `unavailable`. |
| `list_projects` | — | — | **AVAILABLE** — list CBM-indexed projects. Project-independent, no project parameter required. |
| `index_repository` | `repo_path` | `mode` | **AVAILABLE** — trigger CBM to index/reindex a repository. `mode`: `"fast"` (normal refresh, default) or `"full"` (rebuild/recovery). |
| `graph_search` | `query` | `project` | **STRUCTURED** — returns typed results (cached). Prefer `cbm_proxy` for token efficiency. |
| `graph_query` | `query` | `project` | **STRUCTURED** — returns typed `{nodes, edges}` (cached). Prefer `cbm_proxy` for token efficiency. |
| `graph_trace` | `from`, `to` | `project` | **STRUCTURED** — returns typed `{edges}` (cached). Prefer `cbm_proxy` for token efficiency. |
| `get_architecture` | — | `project` | **STRUCTURED** — returns typed `{modules, dependencies}` (cached). Prefer `cbm_proxy` for token efficiency. |

### 1.7 Workspace Tools

| Tool | Required | Optional | Semantics |
|------|----------|----------|-----------|
## 2. Tool-Selection Hierarchy

### 2.1 For Code Understanding

| Situation | Preferred Tool | Why | Avoid |
|-----------|---------------|-----|-------|
| Understand a code file | `provide_code_context` | Compressed IR with signatures, fields, flags; delta transport on repeat calls; heuristics select appropriate fidelity | `read_files` (wasteful — full raw content), `compress_code_context` (no heuristics) |
| Understand a non-code file | `read_files` | `provide_code_context` only supports `.ts`/`.cs`/`.rs`/`.java` | `provide_code_context` (will fail or produce no useful output) |
| Exact line/byte inspection | `read_files` | Line-range reads, byte-level exactness | `provide_code_context` (IR is structural, not byte-exact at non-verbatim fidelities) |
| Search across files | `search_codebase` | Regex pattern search; supports multiple parallel searches | Reading every file manually |
| Quick token-savings check | `context_stats` | Dashboard compression metrics | Manual token counting |

### 2.2 For Bug Investigation

| Phase | Tool Sequence | Rationale |
|-------|--------------|-----------|
| 1. Locate relevant code | `search_codebase` with error/function/symptom patterns | Fastest way to find files containing suspects |
| 2. Understand suspects | `provide_code_context(intent="debug")` on located files | Compressed overview with balanced detail |
| 3. Deep dive (if needed) | `provide_code_context(intent="debug" or "refactor", fidelity="high")` | Higher detail when debug mode is insufficient |
| 4. Cross-file relationships | `cbm_proxy(cbm_tool="search_graph" or "trace_path")` | Architectural/relationship context — only when needed |
| 5. Recent changes (regression) | `diff_commits(fromRef="HEAD~5", toRef="HEAD")` | Understand what changed recently |
| 6. Exact source inspection | `read_files` with specific line ranges | When byte-level detail is required |
| 7. Verify root cause | `run_commands` to build/test/reproduce | Actual compilation/semantic verification |

### 2.3 For Architectural Investigation

| Phase | Tool Sequence | Rationale |
## 3. Intent Selection

`intent` is the **preferred** way to specify how much detail `provide_code_context`
returns. It triggers heuristics that select the appropriate fidelity. Only use
explicit `fidelity` when you need to override the heuristic choice.

| Intent | When to Use | Detail Level | Fidelity Mapping |
|--------|-------------|--------------|------------------|
| `overview` | Understanding file structure/purpose; first look at an unfamiliar file | Lowest token usage | Maps to `Low` (configurable) |
| `debug` | Investigating a defect or root cause | Balanced detail with behavior flags | Maps to `Medium` or `High` depending on config |
| `edit` | Preparing for a targeted edit | Verbatim method bodies for edit-safe replacement | Maps to `Edit` |
| `refactor` | Understanding broader structural changes | Highest structural detail including control-flow/data-flow metadata | Maps to `High` (configurable) |
| `implement` | Adding new code or extending existing functionality | Moderate-to-high detail preserving method bodies and type information | Maps to config default (typically `Edit`) |

---

## 4. Fidelity Selection

When you explicitly specify `fidelity` instead of `intent`, these are the values:

| Fidelity | What the Agent Sees | Method Bodies | Verbatim? | Typical Savings |
|----------|---------------------|:-------------:|:---------:|:---------------:|
| `low` | Structural skeleton (thin) | ❌ | ❌ | ~85% |
| `medium` | Structural skeleton (balanced) with async/export/behavior markers | ❌ | ❌ | ~70-80% |
| `high` | Structural skeleton (max detail) + control-flow/data-flow metadata | ❌ | ❌ | ~50-60% |
| `edit` | Structural skeleton + verbatim method bodies | ✅ (all or focused) | ✅ (bodies) | ~40-60% |
| `verbatim` | Full raw source, entire document | ✅ | ✅ (all) | 0% |

---

## 5. `focusMethods` Discipline

`focusMethods` is an optional array parameter on `provide_code_context` that
controls **which** method bodies receive verbatim content at Edit fidelity.

### When to supply `focusMethods`

- You are editing or deeply inspecting **only specific methods** in a file.
- You want verbatim body text only for the methods you intend to change.
- Target names use qualified notation: `"ClassName.methodName"`, or an
  unambiguous bare method name when no overload ambiguity exists.

### When to omit `focusMethods`

- You need verbatim bodies for **every** method in the file.
- You are reading the file for understanding (overview/debug/refactor), not
  editing.

### When to use an empty array `[]`

- Rarely. An empty `focusMethods` array at Edit fidelity means **no** method
  bodies receive verbatim content — the response will be skeleton-only. This
  is useful when you want Edit fidelity's structural detail but don't need
  any bodies.

---

## 6. `apply_edit` Safety

### Expected Sequence

```
1. provide_code_context(filePath, intent="edit", focusMethods=[target])
   → Response includes content_kind indicating which bodies are byte-exact
   → You now have the verbatim body as expectedOldText for apply_edit

2. apply_edit(filePath, operations: [{
       type: "replace_body",              // or delete, insert_after, insert_before
       target: "ClassName.methodName",    // qualified or unambiguous bare name
       expectedOldText: "{...current verbatim body...}",
       newText: "{...replacement body...}"
    }], verify: true)
   → Tree-sitter parses the spliced result BEFORE writing
   → If parse fails: NOTHING is written; error returned
   → If parse succeeds: response includes syntaxGated: true

3. Run compiler/tests
   → run_commands(cargo check / tsc / dotnet build)
   → Syntax gating ≠ semantic correctness
```

### Operation Types

| Operation | Parameters | Use For |
|-----------|-----------|---------|
| `replace_body` | `target`, `expectedOldText`, `newText` | Replacing a method/function body |
| `delete` | `target`, `expectedOldText` | Deleting an entire method/function |
| `insert_after` | `anchor`, `unitText` | Inserting a new method after an existing one |
| `insert_before` | `anchor`, `unitText` | Inserting a new method before an existing one |

### Prerequisites

- The file must have been read via `provide_code_context` at `fidelity="edit"`
  or `fidelity="verbatim"` **in the current session**.
- The `expectedOldText` must byte-match the current on-disk body (as
  delivered by `provide_code_context`).
- Multi-unit batches targeting different units are supported within a single
  `apply_edit` call.

### When NOT to use `apply_edit`

- **New files** — no prior tracked state exists (v1 policy).
- **Cross-file edits** — `apply_edit` operates on one file per call.
---

## 7. CBM/Graph Rules

### Mandatory Entry Point

`cbm_proxy` is the **sole permitted entry point** for all CBM architectural
intelligence queries. It:

1. Forwards the query to CBM via stdin pipe.
2. Intercepts the raw ~5000-token structural response at the pipe level.
3. Compresses it down to ~1100 tokens using a JSON-aware compressor.
4. Returns the compressed result.
5. On compression failure, applies minimum compression — NEVER returns raw
   CBM output.

### Permitted `cbm_tool` Values

Passed as `cbm_tool` parameter inside `cbm_proxy`:

| `cbm_tool` | Purpose | Parameters Object |
|-----------|---------|------------------|
| `search_graph` | Search symbols by name/pattern | `{ name_pattern: string, project?: string }` |
| `query_graph` | Execute Cypher-like query | `{ query: string, project?: string }` |
| `trace_path` | Trace call/dependency path | `{ function_name: string, direction: "inbound"|"outbound"|"both", depth?: int, project?: string }` |
| `get_architecture` | Get module/component overview | `{ project?: string }` |
| `list_projects` | List all CBM-indexed projects | `{}` (no parameters required) |
| `index_repository` | Trigger CBM indexing/reindexing | `{ repo_path: string (required), mode?: "fast"|"full" }` |

### Index Repository Modes

| Mode | Use Case |
|------|----------|
| `fast` | Normal post-edit refresh — default for `apply_edit`'s automatic reindex |
| `full` | Explicit rebuild/recovery when a complete reindex is needed |

### Automatic Reindex after `apply_edit`

`apply_edit` now automatically performs a synchronous CBM `fast` reindex when CBM is available:

```text
apply_edit
    ↓
filesystem mutation succeeds
    ↓
automatic synchronous CBM fast reindex
    ↓
apply_edit returns
    ↓
graph query can be performed
```

Therefore, an agent normally does **not** need to manually call `index_repository` after a successful Clean-CTX `apply_edit`.

However:

```text
external edit
(host write tool / shell / editor / git operation)
    ↓
Clean-CTX cannot observe the mutation
    ↓
explicit index_repository may be required
    ↓
graph query
```

Edits performed outside Clean-CTX are not automatically observed; use `index_repository` when graph freshness is required after an external edit.

### Direct Call Comparison

`cbm_proxy` is the preferred tool for token-efficient CBM access. The following tools exist for
cases where structured/typed responses are preferred over compressed text:

- `graph_search` — typed `{nodes, count}` (cached, uncompressed)
- `graph_query` — typed `{nodes, edges, count}` (cached, uncompressed)
- `graph_trace` — typed `{edges, count}` (cached, uncompressed)
- `get_architecture` — typed `{modules, dependencies}` (cached, uncompressed)
- `list_projects` — routes through `cbm_proxy` internally
- `index_repository` — routes through `cbm_proxy` internally

The structured tools apply Clean-CTX-specific transformations (query wrapping, path resolution)
and return cached results. Responses are NOT compressed — prefer `cbm_proxy` when token
efficiency matters.

**Project state:** `graph_search`, `graph_query`, `graph_trace`, and `get_architecture`
change the bridge's active project when a `project` argument is supplied. A subsequent
wrapper call without an explicit `project` uses the last-set active project. `cbm_proxy`
does **not** mutate the bridge's active project — its project resolution is scoped to the
individual proxy call.

**Freshness:** The structured tools return TTL-cached results from the bridge (the
cache TTL is configurable). `cbm_proxy` bypasses the bridge cache and fetches fresh data
from CBM before compression. The wrapper and proxy paths therefore have intentionally
different freshness semantics — prefer wrappers for repeated queries where staleness
is acceptable, and the proxy when fresh data is required.

### Other Prohibited Values

Do **not** pass `get_symbol_importance` or `get_dead_code` as `cbm_tool`.
These are not CBM proxy tool names — they are implemented internally via
`query_graph` Cypher queries.

### The Only Allowed Direct Call

`get_cbm_status` is the **only** CBM tool that may be called directly. Its
response is a tiny status object that does not benefit from compression.

### CBM Unavailable Fallback

When `get_cbm_status` returns `unavailable` or `degraded`, do NOT attempt
to bypass the proxy by calling raw CBM tools directly. Instead:

1. Use `search_codebase` for symbol/pattern discovery.
2. Use `provide_code_context` on discovered files for structural understanding.
3. Use `read_files` for exact source inspection when needed.

---

## 8. `diff_commits` Guidance

### When to Use
`diff_commits` answers the question **"What changed between two Git refs?"**
Use it when:

- Understanding a PR or commit before reviewing individual files.
- Checking what changed in a specific commit range.
- Investigating whether a regression was introduced by recent changes.
- Getting a workspace-level summary without reading every file.

### Comparison: `diff_commits` vs `compress_workspace`

| Tool | Scope | Output | Best For |
|------|-------|--------|----------|
| `diff_commits` | Git ref comparison | Per-file AST change-set (additions, deletions, modifications) | Understanding what changed |
| `compress_workspace` | Directory tree | Full structural skeletons of all files (legacy manifest) | Broad structural overview of the entire codebase |

**Do not use `compress_workspace` as a substitute for Git diff analysis.**
If you need to understand changes, use `diff_commits`. If you need the full
structure of all files regardless of change status, use `compress_workspace`.

### Workflow

```
1. diff_commits(fromRef="HEAD~3", toRef="HEAD")
   → Identifies which files changed and how (methods added/removed/modified)

2. For each relevant file:
   → provide_code_context(filePath, intent="overview"|"debug"|"edit")
   → Understand the file's full structure

3. Dig deeper into specific changes as needed
```

### Output Format

The response is a compact manifest:

```
§GITDIFF <from>..<to> (N files)
┌ FILE α1: <path> (+A -D ~M)   ← A=added, D=deleted, M=modified structural units

<change-set body>
- FILE α3: <path> (deleted)
~ FILE α4: <old> → <new> (+A -D ~M)
```

The per-file change-set shows **what** changed (methods, fields, classes)
not **how** the raw lines differ. This is significantly more token-efficient
than reading every file.

---

## 9. Language Support

Clean-CTX supports these languages (feature-gated at build time):

| Language | Cargo Feature | Default? | Clean-CTX Preferred? |
|----------|--------------|:--------:|:--------------------:|
| TypeScript / JavaScript | `typescript` | ✅ Yes | ✅ Yes |
| C# | `csharp` | ✅ Yes | ✅ Yes |
| Rust | `rust` | ❌ Opt-in | ✅ Yes |
| Java | `java` | ❌ Opt-in | ✅ Yes |

The `supportedLanguages` field in every tool schema lists which languages
the current binary supports (computed from enabled Cargo features).

**For all supported languages:** Use `provide_code_context` for code
understanding and `apply_edit` for single-unit edits. This applies to both
investigation and editing workflows.

**Verification still belongs to language-specific tools:**
- Rust → `run_commands` with `cargo check`, `cargo test`, `cargo clippy`
- TypeScript → `run_commands` with `tsc --noEmit`, `jest`
- C# → `run_commands` with `dotnet build`, `dotnet test`
- Java → `run_commands` with `mvn compile`, `gradle build`

Clean-CTX structural/syntax gating (`apply_edit`'s `syntaxGated: true`) is
**not** a substitute for compilation or test verification.

---

## 10. Verification Workflow

| Phase | Tool | What It Confirms |
|-------|------|------------------|
| Edit-time | `apply_edit` response `syntaxGated: true` | Tree-sitter parsed the result without syntax errors |
| Fast syntax | `run_commands(cargo check / tsc --noEmit / dotnet build)` | Compilation succeeds |
| Tests | `run_commands(cargo test / jest / dotnet test)` | Behavioral correctness |
| Full gate | See `docs/agent/verification.md` | Formatting, Clippy, full test suite, encoding guards |

### The Final Verification Gate

The authoritative final verification procedure is documented in
`docs/agent/verification.md`. This document does not replace it. After
any change affecting source code, run the single authoritative gate from
that document.

For Rust projects (this repository):

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace --all-targets --all-features
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
cargo test encoding
```

For non-Rust projects, use the equivalent language-specific tools.

---

## 11. Antipatterns — Do Not

### ❌ Do Not default to `read_files` for source code

`provide_code_context` is always preferred for `.ts`, `.cs`, `.rs`, `.java`
files. `read_files` wastes tokens by returning full raw content. Only use
`read_files` when exact byte-level or line-range inspection is specifically
required, or when `provide_code_context` cannot handle the file.

### ❌ Do Not use `compress_code_context` as a first resort

`provide_code_context` provides heuristics, content classification, and
auto-delta transport. `compress_code_context` is a lower-level mechanism
without these benefits.

### ❌ Do Not pass `focusMethods` without Edit fidelity

At `low`/`medium`/`high` / non-edit fidelities, `focusMethods` is silently
ignored. You will receive skeleton-only output but no error.

### ❌ Prefer `cbm_proxy` over structured wrappers for token efficiency

`graph_search`, `graph_query`, `graph_trace`, and `get_architecture` return
structured/typed Clean-CTX responses rather than compressed text. Prefer `cbm_proxy`
when minimizing token usage is important, and use the structured wrappers when
programmatic access to typed data (nodes, edges, architecture overview) is needed.

### ❌ Do Not use `apply_edit` for changes it cannot safely represent

New files, signature changes, cross-file edits, class-level structural
changes. Use the host write tool for those.

### ❌ Do Not assume syntax gating means tests will pass

`apply_edit`'s `syntaxGated: true` confirms the result parses as valid
syntax. It does **not** confirm type correctness or behavioral correctness.

### ❌ Do Not manually inspect an entire workspace when `diff_commits` suffices

`diff_commits` provides a token-efficient AST-level summary of what
changed between Git refs. Manually reading every file is wasteful.

### ❌ Do Not use `compress_workspace` as a Git diff substitute

Use `diff_commits` for change analysis, `compress_workspace` for broad
codebase structural overview.

### ❌ Do Not bypass Clean-CTX for supported source languages without a reason

For TypeScript, C#, Rust, and Java files, use Clean-CTX tools
(`provide_code_context`, `apply_edit`) rather than raw host tools.
Document any concrete reason for bypassing.

### ❌ Do Not pass `get_symbol_importance` or `get_dead_code` as `cbm_tool`

These are not CBM proxy tool names. They are implemented internally via
`query_graph` Cypher. The proxy will not forward them correctly.