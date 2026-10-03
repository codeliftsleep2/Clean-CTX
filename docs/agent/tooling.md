# Tooling: MCP / Code-Context Tool Selection & Workflow

This document is the **authoritative operational guide** for choosing and
using MCP/code-context tools in this repository. It is NOT always-loaded;
the concise pointer lives in `.clinerules/engineering.md` (Routing table).

Three related files exist with different roles:

| File | Role |
|------|------|
| **`docs/agent/tooling.md`** (this file) | Authoritative detailed agent/tooling guidance |
| `docs/CLAUDE_INTEGRATION_RULES.md` | Compact Claude-facing projection of the current with-CBM/without-CBM workflow |
| `src/mcp/prompts.rs` | Runtime MCP system prompt (injected during initialization) |

Do not treat the more-compact files as contradictory — each serves its
audience. This document governs agent decision-making.

---

## 1. Tool Inventory

The registered schemas in `src/mcp/tools.rs` and `src/cbm/tools.rs` are the
machine-readable authority. This inventory covers every currently registered
public tool; internal handler helpers are intentionally absent.

### 1.1 Standard Host Tools

These are host capabilities, not registered Clean-CTX MCP tools. Their names
vary by client: Cline exposes `read_files`/`search_codebase`, Claude Code uses
`Read`/`Grep`/`Glob`, and terminal-capable agents may use `rg`. Never instruct a
client to call a host-tool name that its active tool catalog does not expose.

| Tool | Description |
|------|-------------|
| `read_files` | Read text/image files from disk. Use for non-code files and exact line inspection. |
| Host text/file search (`search_codebase`, `Grep`/`Glob`, or `rg`) | Search file names and content using the active host's available capability. |
| `fetch_web_content` | Fetch external URLs. Primarily for documentation/API references. |
| `ask_question` | Ask the user a clarifying question. Use when information is genuinely missing. |
| `run_commands` | Execute non-interactive shell commands. Use for builds, tests, git operations, and command-line verification. |

### 1.2 Clean-CTX Context/Read Tools

All accept `workspaceRoot` to anchor relative paths. **Always pass `workspaceRoot`
explicitly** — auto-detection may be wrong in hosted environments (e.g. VS Code).
The `file` field from `graph_search` is a relative path; pair it with `workspaceRoot`
as `filePath`.

| Tool | Required | Optional | Semantics |
|------|----------|----------|-----------|
| `provide_code_context` | `filePath` (single) or `files` (batch) | Per-file `intent`, `fidelity`, `focusMethods`; shared `workspaceRoot`, `tokenizer`; batch-only `responseMode` | **Primary model-facing entry point.** Reads one file or up to 8 independently useful files in one ordered, failure-isolated batch. Every success returns complete current context. Prefer over `compress_code_context`. |
| `compress_code_context` | `filePath` | `fidelity`, `encoding`, `tokenizer`, `workspaceRoot` | Direct AST compilation without heuristics. Lower-level tool; prefer `provide_code_context`. |
| `restore_context` | `filePath` | `workspaceRoot` | Transactionally restore physical `0x04`, checked `dv:2` history, and the aligned semantic-edge snapshot. Never recompiles source as fallback. |
| `context_stats` | — | `filePath`, `format` | Token-savings dashboard. Shows raw vs compressed tokens, delta hit rate, per-file breakdown. |

#### Singular and batch request forms

`provide_code_context` explicitly accepts either one singular `filePath`
request or one top-level `files` batch. These forms are mutually exclusive.
Use the singular `filePath` form for one source file. For two or more
independently useful source contexts, use top-level `files`; each item requires
a unique non-empty `id` and `filePath` and may choose its own `intent`,
`fidelity`, and `focusMethods`. Keep `workspaceRoot` and `tokenizer` at the top
level. Batch results preserve request order and declare the resolved
`response_mode`. Inspect every item: a success has `status="ok"` and semantic
`meta`; an error remains item-local and never owns content.

`responseMode` is valid only on the batch form. Prefer `structured` for a host
verified to consume `structuredContent`; exact code then appears once in each
successful item and the top-level text is only a routing notice. On an unknown
host, omit the field to receive safe `mirrored`: exact code appears both in
each successful item's `content` and in its `content_index`-identified block.
Use `indexed` only for a verified content-channel host; exact code then appears
once in top-level blocks and successful items carry `content_index`. Never
select a compact mode by guessing host behavior.

Batching is best-effort context acquisition, not a cross-file transaction. One
item failure does not suppress successful siblings, and successful reads retain
their normal session/cache effects. A later item resolving to the same
canonical file is rejected; combine selectors for that file into one
`focusMethods` array. Use per-item fidelity so only actual edit targets request
Edit or Verbatim content. If every item fails, the sole text block directs the
caller to the authoritative structured item errors.

### 1.3 Diff/Delta Tools (Read-Only Comparisons)

| Tool | Required | Optional | Semantics |
|------|----------|----------|-----------|
| `diff_code_context` | `filePath` | `workspaceRoot`, `fidelity` | AST-level diff: compares in-session baseline against current on-disk state for a **single file**. |
| `delta_code_context` | `filePath` | `workspaceRoot`, `fidelity` | Generate an IR-level delta from opcode differences and retain its server-owned pending semantic transition. The structured payload is code-side only; generation does not apply it. |
| `diff_commits` | `fromRef` | `toRef`, `workspaceRoot`, `fidelity` | **Multi-file git-ref diff.** Compares an entire workspace between two Git refs and emits per-file AST-level change-sets. Most token-efficient way to understand PR/commit-level changes. |

### 1.4 Edit/Mutation Tools

| Tool | Required | Optional | Semantics |
|------|----------|----------|-----------|
| `apply_edit` | `filePath`, `operations` | `verify`, `workspaceRoot` | Byte-exact structural edit (`replace_body`, `delete`, `insert_after`, `insert_before`) over tracked units. Requires matching disk/live/durable source identity and commits source plus semantic state through the staged durable transaction. Full-body fidelity remains available when safe editing requires it. |
| `apply_delta` | `delta`, `currentVersion` | — | Explicitly acknowledge and apply an exact pending IR delta, committing durable and live semantic state before consuming the pending transition. This is a code-side protocol, not an LLM workflow. Clean-CTX does not ship an automatic host consumer, so repository-local use is manual or verification-driven. |
### 1.5 Admin/Persistence Tools

| Tool | Required | Optional | Semantics |
|------|----------|----------|-----------|
| `save_context` | `filePath` | — | Explicitly save in-memory compressed context to the persistence DB. |
| `delete_context` | `filePath` | — | Transactionally delete one file's persisted and session semantic context without modifying its source file. |
| `list_sessions` | — | — | List all persisted file contexts with fidelity, token counts, and timestamps. |
| `replay_history` | `filePath` | `targetSequence`, `fidelity` | Replay delta history from the DB. |
| `purge_old_deltas` | — | `days`, `filePath` | Purge old delta entries. |
| `context_history` | — | `filePath` | View compression history, optionally narrowed to a specific file. |
| `inspect_legacy_fallbacks` | — | — | Read-only inspection of quarantined legacy fallback artifacts. Reports why they are incomplete; never imports, repairs, deletes, or mutates semantic state. |

### 1.6 CBM / Graph Tools (Architectural Intelligence)

| Tool | Required | Optional | Status |
|------|----------|----------|--------|
| `cbm_proxy` | — | `cbm_tool`, `parameters`, `query`, `project` | **Preferred compact path** for raw CBM operations. Project resolution is call-scoped and does not mutate the wrapper bridge's active project. |
| `get_cbm_status` | — | — | Availability probe. Returns `available`, `degraded`, or `unavailable`. |
| `list_projects` | — | — | **AVAILABLE** — list CBM-indexed projects. Project-independent, no project parameter required. |
| `index_repository` | `repo_path` | `mode` | **AVAILABLE** — trigger CBM to index/reindex a repository. `mode` is optional: `"fast"` (normal refresh, default) or `"full"` (rebuild/recovery). |
| `graph_search` | `query` | `name_pattern`, `project` | **STRUCTURED** — returns typed results (cached). Prefer when the typed identity/file result is useful. |
| `graph_query` | `query` | `project` | **STRUCTURED** — returns typed `{nodes, edges}` (cached). |
| `graph_trace` | `from`, `to` | `project` | **STRUCTURED** — resolves bare identities uniquely or rejects ambiguity, then returns typed `{edges}`. |
| `get_architecture` | — | `project` | **STRUCTURED** — returns typed `{modules, dependencies}` (cached). |

### 1.7 Workspace Tools

| Tool | Required | Optional | Semantics |
|------|----------|----------|-----------|
| `workspace_query` | `type` (single) or `queries` (batch) | Operation fields; shared `workspaceRoot`, `withinPath` | **READ-ONLY** — Run one legacy operation or up to 32 heterogeneous operations in one ordered batch. A batch requires unique item IDs, shares one top-level scope, deduplicates equivalent hydration, evaluates index-backed items against one final post-preparation view, and returns independent per-item success/error outcomes. The seven operation semantics and authority boundaries are unchanged. |

The seventh operation, `calls_in_file`, requires `filePath`, `workspaceRoot`,
`owner: {kind: "class"|"interface", name}`, and `method: {name}`. Optional
`method.parameters` and `method.return_type` are exact visible-signature
selectors; omitting them returns the complete overload family. `withinPath` may
narrow the already authorized root set. The operation compiles a High-fidelity,
read-only canonical candidate and returns declaration-ordered overloads plus
ordered call occurrences (`callee_written`, `explicit_argument_count`, and
`has_spread`). It does not run hydration, publish session/WorkspaceIndex state,
expose canonical IDs, or claim that a written callee is resolved.

Direct constructor-consumer lookup is relation- and language-specific:
Angular dependency injection uses `Injects`, while a C# constructor parameter
projects the source-true `HasConstructorParameterType` relation from
`builtin / Class / <consumer>` to `builtin / TypeRef / <written type>`. Query
the C# form with `reverse_edges` on that exact `TypeRef` identity. It does not
claim container registration, runtime .NET DI, or a resolved interface/class
declaration. Consequently, a successful empty reverse-edge lookup for an
identity whose relation is not projected is not verified absence of consumers.
For `builtin / Interface / <name>`, reverse coverage returns an
`alternative_query` naming the same-name `builtin / TypeRef` lookup used by C#
constructor consumption. Exact edge results remain index-backed lower bounds:
when `source_complete` is false, `result_semantics: "lower_bound"` and
`omitted_possible: true` mean `count` may omit uncompiled source files;
`discovered_not_compiled_this_cycle`, when present, is the per-cycle
discovered-minus-compiled gap and can include already-indexed or deduplicated
candidates; it is not an exact omitted-file count.
This relation does not cover service-locator calls such as
`GetRequiredService<T>()` because no constructor parameter declares that fact.

Use the single form for one question or when a client does not consume batch
results. Use `queries` when two or more independent questions share the same
workspace scope, especially mixed `find_entities`, edge, traversal, and cycle
requests. Put `workspaceRoot` and optional `withinPath` only at the top level;
each item carries a unique non-empty `id`, its own `type`, and that operation's
normal fields. Do not split a batch merely because its operations differ.

A structurally invalid batch or invalid shared scope rejects the whole call.
Once the batch is accepted, inspect every ordered item: `status="ok"` carries
the corresponding legacy structured payload under `result`, while
`status="error"` carries that item's code/message without suppressing sibling
successes. Batch execution is best-effort rather than transactional; valid
session hydration/index warming is retained even when another item fails.

### 1.8 Standard Tool Annotations

Every public tool declares explicit MCP `readOnlyHint`, `destructiveHint`,
`idempotentHint`, and `openWorldHint` values. These are client hints, not
authorization. Internal caches and session projections do not make a source or
query tool externally mutating; source edits, durable semantic deletion/purge,
delta application, graph reindexing, and the generic proxy retain conservative
mutation classifications. All Clean-CTX tools operate inside the configured
local workspace/provider boundary and declare `openWorldHint: false`.

## 2. Tool-Selection Hierarchy

### 2.1 For Code Understanding

| Situation | Preferred Tool | Why | Avoid |
|-----------|---------------|-----|-------|
| Understand a code file | `provide_code_context` | Complete current SCHEMA-vNext context with signatures, fields, and flags; heuristics select appropriate fidelity | `read_files` (wasteful — full raw content), `compress_code_context` (no heuristics) |
| Understand a non-code file | `read_files` | `provide_code_context` only supports `.ts`/`.cs`/`.rs`/`.java` | `provide_code_context` (will fail or produce no useful output) |
| Exact line/byte inspection | `read_files` | Line-range reads, byte-level exactness | `provide_code_context` (IR is structural, not byte-exact at non-verbatim fidelities) |
| Discover a symbol, file, class, method, or concept | `graph_search` or `cbm_proxy(cbm_tool: "search_graph")` | Graph-aware semantic search across CBM-indexed symbols and files | Host text search alone (misses graph relationships) |
| Text/regex search across file content | Active host search (`search_codebase`, `Grep`, or `rg`) | Uses the discovery capability actually exposed by the client | Reading every file manually |
| Quick token-savings check | `context_stats` | Dashboard compression metrics | Manual token counting |

### 2.2 For Bug Investigation

| Phase | Tool Sequence | Rationale |
|-------|--------------|-----------|
| 1. Locate relevant code | `graph_search` or `cbm_proxy(cbm_tool: "search_graph")` with symbol/function/error patterns | Clean-CTX graph-aware discovery. When CBM is unavailable, use `workspace_query(find_entities)` for an exact semantic name or the active host's text/file search (`search_codebase`, Claude `Grep`/`Glob`, or `rg`). |
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
| `debug` | Investigating a defect or root cause | Balanced detail with behavior flags | Maps to `Medium` by default (configurable) |
| `edit` | Preparing for a targeted edit | Verbatim method bodies for edit-safe replacement | Maps to `Edit` |
| `refactor` | Understanding broader structural changes | Highest structural detail including control-flow/data-flow metadata | Maps to `High` (configurable) |
| `implement` | Adding new code or extending existing functionality | Moderate structural detail and type information | Maps to `Medium` by default (configurable) |

---

## 4. Fidelity Selection

When you explicitly specify `fidelity` instead of `intent`, these are the values:

| Fidelity | What the Agent Sees | Method Bodies | Verbatim? |
|----------|---------------------|:-------------:|:---------:|
| `low` | SCHEMA-vNext structural presentation of the compiled Low hierarchy | ❌ | ❌ |
| `medium` | SCHEMA-vNext structural presentation with Medium semantic detail | ❌ | ❌ |
| `high` | SCHEMA-vNext reasoning presentation with control/data-flow metadata | ❌ | ❌ |
| `edit` | SCHEMA-vNext structure plus exact bodies for all or the resolved focus | ✅ (all or resolved focus) | ✅ (bodies) |
| `verbatim` | Full raw source, entire document | ✅ | ✅ (all) |

---

## 5. `focusMethods` Discipline

`focusMethods` is an optional array parameter on `provide_code_context` that
controls **which** method bodies receive verbatim content at Edit fidelity.

### When to supply `focusMethods`

- You are editing or deeply inspecting **only specific methods** in a file.
- You want verbatim body text only for the methods you intend to change.
- Target names use qualified notation: `"ClassName.methodName"`, or an
  unambiguous bare method name when exactly one typed owner defines it.
- Selection is resolved to canonical method IDs before rendering. A bare name
  shared by owners, or a qualified owner name that is itself duplicated, is an
  invalid request; Clean-CTX returns `-32602` rather than guessing. A qualified
  same-owner overload family selects every overload because the selector grammar
  has no signature discriminator.

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
   → run_commands(cargo check --all-features / tsc / dotnet build)
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

### Compact and structured entry points

Use `graph_search` first for typed symbol/file discovery. Use the structured
wrappers when typed nodes, edges, or modules matter; use `cbm_proxy` when a
compact rendering of a raw CBM operation is more economical. `cbm_proxy`:

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
| `fast` | Normal post-edit refresh — used by the lazy freshness gate before graph queries |
| `full` | Explicit rebuild/recovery when a complete reindex is needed |

### Lazy CBM Graph Freshness after `apply_edit`

`apply_edit` now uses **per-project lazy freshness** instead of synchronous reindexing:

```text
apply_edit
    ↓
filesystem mutation succeeds
    ↓
session IR refreshed
    ↓
affected CBM project marked DIRTY (no CBM call)
    ↓
apply_edit returns immediately

next graph query (graph_search / graph_query / graph_trace / get_architecture / cbm_proxy)
    ↓
detects project is DIRTY
    ↓
synchronous CBM fast reindex
    ↓
project marked FRESH
    ↓
graph query executes against up-to-date graph
```

Therefore, an agent normally does **not** need to manually call `index_repository` after a successful Clean-CTX `apply_edit`. The next graph operation automatically refreshes the affected project before executing.

Key properties:

* **Lazy:** The reindex only happens when a graph query actually executes, not on every edit.
* **Per-project:** Each CBM project tracks its own freshness independently. Editing project A does not invalidate project B's graph.
* **Coalescing:** Multiple rapid edits to the same project trigger only one reindex (on the next graph query).
* **Transparent:** The agent sees fresh graph results without any explicit indexing step.
* **Non-blocking for edits:** `apply_edit` returns immediately after marking the project dirty — no CBM call on the write path.

However for external edits:

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

Choose the CBM surface by the result shape the task needs. Use the structured
wrappers for typed identities, nodes, edges, paths, or modules. Use `cbm_proxy`
when compact or explicitly fresh raw CBM output is preferable:

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

**Trace identity:** A canonical CBM source identity is traced directly. A bare
source is first resolved by exact name inside the selected project. Exactly one
canonical identity proceeds automatically; no match is an explicit not-found
error, and multiple identities return `-32602` with deterministic canonical
candidates. `graph_trace` and `cbm_proxy(trace_path)` enforce the same rule.

**Freshness:** The structured tools return TTL-cached results from the bridge (the
cache TTL is configurable). `cbm_proxy` bypasses the bridge cache and fetches fresh data
from CBM before compression. The wrapper and proxy paths therefore have intentionally
different freshness semantics — prefer wrappers for repeated queries where staleness
is acceptable, and the proxy when fresh data is required.

Every structured cache entry is owned by one canonical CBM project. Identical
query text in two projects produces distinct entries; switching the active
project preserves those safely isolated entries. After `apply_edit`, the next
structured graph operation refreshes the dirty project and invalidates all and
only that project's memory and disk results. Explicit-project disk access uses
the registered project-to-root mapping and never the unrelated active root.

### Clean-CTX-First Repository Discovery

Use Clean-CTX as the primary repository and code-intelligence layer. Do not
guess repository paths, namespaces, or structural conventions when Clean-CTX
can answer the question.

#### Tool selection

| Need | Preferred operation |
|------|--------------------|
| Find a symbol, file, class, method, or concept | `graph_search` or `cbm_proxy(cbm_tool: "search_graph")` |
| Explore structural / graph relationships | `graph_query` or `cbm_proxy(cbm_tool: "query_graph")` |
| Trace a dependency or call path | `graph_trace` or `cbm_proxy(cbm_tool: "trace_path")` |
| Understand repository / module architecture | `get_architecture` or `cbm_proxy(cbm_tool: "get_architecture")` |
| Discover available CBM projects | `cbm_proxy(cbm_tool: "list_projects")` |
| Trigger repository reindexing | `cbm_proxy(cbm_tool: "index_repository")` |
| Read a code file for context | `provide_code_context` |
| Filesystem read / write / edit | Host tool (`run_commands`, native editor, etc.) |

#### Path discipline

- **Never invent a path.** Do not guess a file's location from its namespace,
  module name, or symbol name. Use `graph_search` or `cbm_proxy(cbm_tool:
  "search_graph")` to discover the actual location.
- **Do not turn a CBM project name into a filesystem path.** The mapping from
  repository path to CBM project slug is derived and is not reversible from
  the slug alone.
- **Do not turn a filesystem path into a CBM project name** unless the tool
  explicitly handles that resolution (e.g. `index_repository` accepts
  `repo_path`).
- **When Clean-CTX returns a path, use it.** Do not replace or reconstruct
  the returned path.
- **When uncertain, discover.** Use Clean-CTX to discover paths and symbols
  rather than guessing.

**Bad — guessing paths:**
```text
> The class is probably under src/services/UserService.ts, so I will open that.
```

**Good — discovering with Clean-CTX:**
```text
> mcp__clean-ctx__graph_search(query: "UserService")
> → found at src/services/UserService.ts
> Now use provide_code_context on the discovered path.
```

#### Project semantics

The structured wrapper tools (`graph_search`, `graph_query`, `graph_trace`,
`get_architecture`) maintain a **sticky active project** on the bridge:

- When you supply a `project` parameter, that project becomes the active
  project for subsequent structured-wrapper calls that omit a project.
- If you work across multiple projects, explicitly supply `project` on each
  wrapper call rather than relying on inherited state.

The `cbm_proxy` tool does **not** change the active project:

- A project supplied to `cbm_proxy` is resolved only for that proxy
  invocation.
- An exact configured-root basename is accepted as an alias and rewritten to
  the canonical CBM slug before dispatch. Partial or invented slug names are
  not fuzzy-matched; CBM rejects them with one explicit `isError` result.
- A rejected project never returns candidate, caller, or other result-shaped
  data as partial success.
- The next `graph_search()` or `graph_query()` call will still target
  whatever project was active before the proxy call.

These semantics are documented in each tool's MCP schema (`project` parameter
description).

#### Recommended workflow

1. **Discover** — Determine which repository or project the task concerns.
   Use `cbm_proxy(cbm_tool: "list_projects")` or `get_architecture()` to
   understand the workspace.
2. **Locate** — Use `graph_search` to find symbols, files, or classes
   rather than guessing paths.
3. **Analyze** — Use `graph_query`, `graph_trace`, or `get_architecture`
   for structural and relationship discovery.
4. **Read** — Use `provide_code_context` on the discovered file path.
5. **Edit** — Use the appropriate edit tool: `apply_edit` for single-unit
   edits; the host write tool for broader changes.
6. **Verify** — Compile, test, and confirm correctness.

Filesystem operations are still appropriate when the task genuinely requires
them (create, delete, list, move files) or when Clean-CTX cannot answer the
question.

### Other Prohibited Values

Do **not** pass `get_symbol_importance` or `get_dead_code` as `cbm_tool`.
These are not CBM proxy tool names — they are implemented internally via
`query_graph` Cypher queries.

### Raw-service bypass boundary

Do not bypass Clean-CTX to invoke the underlying CBM server directly.
`get_cbm_status`, the structured wrappers, and `cbm_proxy` are all registered
Clean-CTX tools. Graph handlers consult live CBM health directly; reserve
`get_cbm_status` for setup diagnostics, recovery checks, or explicit
indexing-progress inspection rather than routine task preflight.

### CBM Unavailable Fallback

When a graph operation reports CBM as unavailable or degraded, do NOT add a
status probe or attempt to bypass the proxy by calling raw CBM tools directly.
Instead:

1. Use the active host's text/file search for symbol/pattern discovery
   (`search_codebase` in Cline, `Grep`/`Glob` in Claude Code, or `rg` in a
   terminal-capable host). For an exact semantic name, prefer
   `workspace_query(type="find_entities")` so Clean-CTX can use its filesystem
   discovery path.
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

### Comparison: `diff_commits` vs workspace-level tools

`compress_workspace` is an internal implementation detail — it is **not** an agent-invocable MCP tool. For broad structural overviews, use `cbm_proxy(cbm_tool: "search_graph")` for symbol discovery or `provide_code_context` on specific files for structural skeletons.

| Tool | Scope | Output | Best For |
|------|-------|--------|----------|
| `diff_commits` | Git ref comparison | Per-file AST change-set (additions, deletions, modifications) | Understanding what changed |
| `cbm_proxy(cbm_tool: "search_graph")` | Whole workspace | Graph-aware search results (symbols, files) | Broad symbol/structural discovery |

**Do not use `cbm_proxy` as a substitute for Git diff analysis.**
If you need to understand changes, use `diff_commits`. If you need broad structural discovery, use `cbm_proxy(cbm_tool: "search_graph")` or `provide_code_context` on specific files.

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

See [`tooling-language-support.md`](tooling-language-support.md) for the
feature-gated language matrix, preferred Clean-CTX tools, and language-specific
verification guidance.

---

## 10. Verification Workflow

| Phase | Tool | What It Confirms |
|-------|------|------------------|
| Edit-time | `apply_edit` response `syntaxGated: true` | Tree-sitter parsed the result without syntax errors |
| Fast syntax | `run_commands(cargo check --all-features / tsc --noEmit / dotnet build)` | Compilation succeeds |
| Tests | `run_commands(cargo test --all-features / jest / dotnet test)` | Behavioral correctness |
| Full gate | See `docs/agent/verification.md` | Formatting, Clippy, full test suite, encoding guards |

### The Final Verification Gate

The authoritative final verification procedure is documented in
`docs/agent/verification.md`. This document does not replace it. After
any change affecting source code, run the single authoritative gate from
that document.

For Rust projects (this repository):

```
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
cargo test --all-features encoding
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

`provide_code_context` provides intent/fidelity heuristics, content
classification, and complete current context. `compress_code_context` is a
lower-level mechanism without these benefits.

### ❌ Do Not combine `focusMethods` with an explicit non-Edit mode

A non-empty `focusMethods` array implies Edit when both `fidelity` and `intent`
are omitted. Explicit non-Edit fidelity or intent conflicts with focus and
returns `-32602`; it is never silently ignored or overridden. An empty array
retains its specialized Edit-only meaning and therefore requires explicit
`fidelity: edit` or `intent: edit`.

### ❌ Do Not choose a CBM surface without matching the required result shape

Use `graph_search` as the normal symbol/file discovery entry point. Use
`graph_query`, `graph_trace`, or `get_architecture` when their typed result is
required. Use `cbm_proxy` when compact or explicitly fresh raw CBM output is
the better result shape; it is not a universal replacement for the structured
wrappers.

### ❌ Do Not use `apply_edit` for changes it cannot safely represent

New files, signature changes, cross-file edits, class-level structural
changes. Use the host write tool for those.

### ❌ Do Not assume syntax gating means tests will pass

`apply_edit`'s `syntaxGated: true` confirms the result parses as valid
syntax. It does **not** confirm type correctness or behavioral correctness.

### ❌ Do Not manually inspect an entire workspace when `diff_commits` suffices

`diff_commits` provides a token-efficient AST-level summary of what
changed between Git refs. Manually reading every file is wasteful.

### ❌ Do Not attempt to invoke internal-only tools

`compress_workspace` is an internal implementation — it is **not** an
agent-invocable MCP tool and is not registered in `tools/list`.
Do not attempt to call it. Use `diff_commits` for change analysis,
`cbm_proxy(cbm_tool: "search_graph")` for broad symbol discovery, or
`provide_code_context` for structural file overviews.

### ❌ Do Not bypass Clean-CTX for supported source languages without a reason

For TypeScript, C#, Rust, and Java files, use Clean-CTX tools
(`provide_code_context`, `apply_edit`) rather than raw host tools.
Document any concrete reason for bypassing.

### ❌ Do Not guess or invent repository paths or project slugs

Never assume a file's location from its namespace or module name. Never turn
a CBM project slug into a filesystem path. Never turn a filesystem path into
a CBM project slug without explicit tool support (e.g. `index_repository`).
If you need a path, use `graph_search` or `cbm_proxy(cbm_tool:
"search_graph")` to discover the actual location. Use the path Clean-CTX
returns rather than reconstructing it.

### ❌ Do Not pass `get_symbol_importance` or `get_dead_code` as `cbm_tool`

These are not CBM proxy tool names. They are implemented internally via
`query_graph` Cypher. The proxy will not forward them correctly.
