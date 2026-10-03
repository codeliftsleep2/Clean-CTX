# Claude rules for using Clean-CTX

**Status:** Portable Claude-facing projection of the current tool workflow
**Detailed authority:** [`agent/tooling.md`](agent/tooling.md)

Use Clean-CTX as the primary code-intelligence layer. These rules distinguish
three jobs that must not be conflated:

1. CBM discovers symbols and graph candidates when available.
2. Clean-CTX compiles source into canonical semantic context.
3. Native file tools handle non-code files and exact source inspection when a
   compressed/structured answer is insufficient.

Always pass `workspaceRoot` explicitly. A returned repository-relative `file`
path is authoritative only when paired with that root. Never derive a
filesystem path from a namespace, module name, filename convention, or CBM
project slug.

---

## 1. Start with the useful graph operation

For repository discovery, call the useful graph operation directly. The graph
handlers consult CBM's live status themselves, so a separate `get_cbm_status`
preflight adds no authority. Reserve `get_cbm_status` for setup diagnostics,
recovery checks, or explicit indexing-progress inspection.

### CBM is `available`

Use the structured Clean-CTX graph wrappers when their typed result matches the
job. `graph_search` is the normal symbol/file discovery entry point:

- `graph_search` — locate symbols and authoritative repository-relative files;
- `graph_query` — obtain typed nodes and edges;
- `graph_trace` — trace between symbol identities; and
- `get_architecture` — inspect modules and dependencies.

Use `cbm_proxy` when a compact or explicitly fresh rendering of a raw CBM
operation is more useful than typed wrapper output. Both paths are registered
Clean-CTX tools; do not bypass Clean-CTX to invoke the underlying CBM server
directly.

### CBM is `degraded` or `unavailable`

The failed graph call reports the direct fallback. Do not add a status probe,
retry equivalent CBM calls in a loop, or treat provider failure as an
authoritative empty graph. Fall back to:

1. Claude's native `Grep`/`Glob` tools for text or file discovery;
2. `workspace_query(type="find_entities")` when an exact semantic name is
   known and filesystem-backed semantic discovery is preferable;
3. `provide_code_context` for supported source files; and
4. native `Read` only for exact source ranges or unsupported/non-code files.

`search_codebase` is a Cline host-tool name, not a registered Clean-CTX MCP
tool and not a Claude Code tool. Never attempt to call it from Claude merely
because older fallback text names it.

`workspace_query` may also use its registered filesystem discovery path for
eligible name-bearing semantic queries. Its sparse `discovery` metadata reports
only exceptional coverage; absence of that field means the expected discovery
path completed, not that CBM facts became semantic authority.

---

## 2. Read source through `provide_code_context`

For supported code (`.ts`, `.cs`, `.rs`, `.java` when compiled into the
running binary), call `provide_code_context` before native `Read`. The tool
explicitly accepts either one singular `filePath` request or one top-level
`files` batch; the forms are mutually exclusive.

Single-file form:

```text
provide_code_context(
  filePath: "src/services/UserService.ts",
  workspaceRoot: "C:/work/my-repo",
  intent: "debug"
)
```

Choose the intent that matches the task:

- `overview` — compact structural orientation;
- `debug` — balanced diagnostic context;
- `refactor` — high structural detail;
- `implement` — implementation-oriented detail; or
- `edit` — byte-exact bodies needed for safe structural editing.

Every successful `provide_code_context` response is complete current context,
not an automatic delta. `delta_code_context` and `apply_delta` are an explicit
code-side protocol; use them only when a real host/consumer intentionally owns
the prior version and acknowledgment lifecycle.

### Batched multi-file form

Use singular `filePath` for one file. When two or more independently useful
source contexts are needed, prefer one `files` batch of at most eight items:

```json
{
  "workspaceRoot": "C:/work/my-repo",
  "tokenizer": "o200k",
  "responseMode": "structured",
  "files": [
    { "id": "service", "filePath": "src/services/UserService.ts", "intent": "overview" },
    { "id": "target", "filePath": "src/controllers/UserController.ts", "intent": "edit", "focusMethods": ["UserController.update"] }
  ]
}
```

Claude is a verified `structuredContent` consumer, so `structured` is the
preferred batch mode: request `responseMode: "structured"`. Exact code then
appears once in each successful structured item without the near-doubling of
the compatibility mirror. If integration behavior changes or is unknown, omit
the field and use safe `mirrored`. Do not select `indexed` unless that Claude
host has been explicitly verified to preserve and consume top-level MCP content
blocks.

Each item requires a unique non-empty `id` and `filePath` and may choose its
own `intent`, `fidelity`, and `focusMethods`. `workspaceRoot` and `tokenizer`
belong only at the top level. Inspect every ordered result: `status="ok"`
carries exact `content` and semantic `meta` in structured mode.
`status="error"` carries an item-local error without suppressing successful
siblings. The response's `response_mode` confirms the projection actually
used. If every item fails, the text block only directs the caller to the
structured errors.

Do not repeat the same canonical file in a batch; combine its desired selectors
into one `focusMethods` array. Request Edit or Verbatim only for files that need
exact text. Batch reads retain normal successful session/cache effects but do
not create cross-file transactional edit authority.

If the response is a skeleton and statement-level source is required, use
native `Read` for the known file/range. Native `Read` is also appropriate for
Markdown, JSON, TOML, configuration, unsupported languages, or after the
Clean-CTX read fails. Do not invent an alternate path when a returned path
fails—report or resolve the actual boundary.

### `focusMethods`

- A non-empty `focusMethods` with neither explicit `fidelity` nor `intent`
  implies Edit.
- An explicit non-Edit fidelity or intent conflicts with non-empty focus and
  returns `-32602`; focus is never silently ignored.
- An empty array is valid only with explicit Edit and intentionally selects no
  bodies.
- Selectors are owner-aware. Use `Owner.method` when a bare method name could be
  ambiguous; one same-owner overload family selects all matching overloads.

---

## 3. Use `workspace_query` for Clean-CTX semantic facts

Use `workspace_query` when the desired answer is an entity, relationship,
dependency traversal, cycle witness, or owner-qualified call list rather than
rendered file context.

### Name-bearing workspace queries

`find_entities`, `forward_edges`, `reverse_edges`, and
`transitive_dependencies` use Clean-CTX semantic facts. For edge/traversal
queries, `domain` and `entity_type` are optional identity filters:

- one matching semantic identity resolves automatically;
- repeated physical occurrences of the same semantic identity are not treated
  as identity ambiguity;
- multiple distinct identities return an explicit candidate list; and
- no match returns an explicit not-found error rather than a plausible empty
  answer.

CBM/filesystem discovery supplies candidate files only. Clean-CTX alone compiles
those files and determines `WorkspaceIndex` relationships.

Constructor-consumer queries are language-specific. Angular DI uses `Injects`.
For C#, "who consumes `IFooService` through a constructor?" uses
`reverse_edges` with `domain: "builtin"`, `entity_type: "TypeRef"`, and the
written type name; the returned relation is `HasConstructorParameterType`.
This records a source signature only. It does not identify a .NET container
registration, prove runtime injection, or answer which implementation .NET DI
will supply. It also does not cover service-locator calls such as
`GetRequiredService<T>()`, because those are not constructor parameters.

Query the written type identity, not the declaration identity:
`reverse_edges(builtin/TypeRef/IFooService)`. A reverse query on
`builtin/Interface/IFooService` does not own this capability and returns an
`alternative_query` pointing to the `TypeRef` form. Exact edge-query coverage
with `source_complete: false` also reports `result_semantics: "lower_bound"`
and `omitted_possible: true`; its `count` is a minimum, not a complete consumer
count. When discovery found more candidates than it compiled in that cycle,
`discovered_not_compiled_this_cycle` reports the observed gap; it can include
already-indexed or deduplicated candidates and is not itself an omitted-file
count.

### `entities_in_file`

Pass `file_path`, `workspaceRoot`, and optional `fidelity`. It compiles an
untracked trusted file on first touch, reuses fresh sufficient projections,
upgrades lower-fidelity projections when required, and replaces stale facts
even when recompilation yields an empty projection. Edit/Verbatim normalize to
High semantic compilation because this query returns no source bodies.

### `has_cycle`

`has_cycle` is intentionally index-only. It never discovers or compiles a
workspace. The default/only current `kind` is `dependency`; it returns one
deterministic closed witness using the approved dependency relations and states
`indexed_evidence_only` coverage. A false result is not proof that every source
file in the workspace has been compiled.

### `calls_in_file`

Use `type: "calls_in_file"` with `filePath`, `workspaceRoot`, typed `owner`,
and `method`. It compiles a read-only High-fidelity candidate and preserves
overloads, source order, duplicate calls, written argument count, and spread
evidence. Written callees are not claimed to be resolved cross-file identities.

### Scope

`withinPath` only narrows `workspaceRoot` plus configured `additional_roots`.
It never authorizes a new root and is rejected when no `workspaceRoot` is
present or when it lies outside the authorized root set.

### Batch several workspace questions

Use the legacy single form when only one semantic question is needed. When two
or more independent questions share the same workspace scope, prefer one batch
instead of issuing repeated `workspace_query` tool calls. A batch may mix any
of the seven operation types:

```json
{
  "workspaceRoot": "C:/work/my-repo",
  "withinPath": "src/orders",
  "queries": [
    { "id": "service", "type": "find_entities", "name": "OrderService" },
    { "id": "callers", "type": "reverse_edges", "name": "OrderService" },
    { "id": "cycles", "type": "has_cycle" }
  ]
}
```

Each item requires a unique non-empty string `id`. Put `workspaceRoot` and
optional `withinPath` only at the batch top level; item-level scope overrides
are invalid. Input order is preserved. Inspect every returned item:
`status="ok"` carries that operation's normal structured payload under
`result`, while `status="error"` carries an item-local error without suppressing
independent successes. A malformed batch, duplicate ID, invalid shared scope,
or more than 32 items rejects the whole call.

`name` is always one non-empty string. Do not pass an array to `name`, and do
not invent a plural `names` field. To ask the same operation about several
names, create one independently identified item per name:

```json
{
  "workspaceRoot": "C:/work/my-repo",
  "queries": [
    { "id": "method-a", "type": "reverse_edges", "name": "MethodA" },
    { "id": "method-b", "type": "reverse_edges", "name": "MethodB" },
    { "id": "method-c", "type": "reverse_edges", "name": "MethodC" }
  ]
}
```

Batching shares preparation work and one final WorkspaceIndex view; it does not
merge answers, infer identity across items, or change the special authority of
`has_cycle` and `calls_in_file`. Do not split a request merely because it mixes
forward edges, reverse edges, entity lookup, traversal, cycle, or file-local
operations.

---

## 4. CBM identity and project rules

A CBM project slug and a filesystem path are different namespaces. Only
`index_repository` establishes their relationship.

- Structured wrappers accept an optional project and may update the bridge's
  active project for later structured-wrapper calls.
- `cbm_proxy` project resolution is scoped to that one call and does not change
  the active project.
- An exact configured-root basename may be used as a project alias; Clean-CTX
  rewrites it to the canonical slug before dispatch. Do not abbreviate or
  partially copy a slug—unknown names fail explicitly and return no partial
  candidate or caller data.
- A bare trace source is accepted only when it resolves to one canonical
  identity. Ambiguity returns all canonical candidates; never select the first
  match silently.
- Use `list_projects` when the authoritative registered identities are needed.
- Use `index_repository(repo_path, mode: "fast")` after external edits when
  graph freshness is required. `full` is for explicit rebuild/recovery.

After a successful Clean-CTX `apply_edit`, the affected project is marked stale
and the next graph operation performs the supported lazy fast refresh. A manual
reindex is normally unnecessary for Clean-CTX-owned edits.

---

## 5. Prefer `apply_edit` for supported single-file structural edits

After Edit/Verbatim context has supplied the exact tracked unit, use
`apply_edit` for operations it represents:

- `replace_body`;
- `delete`;
- `insert_after`; and
- `insert_before`.

Pass `workspaceRoot` and byte-exact `expectedOldText` where required. A stale
rejection means the source changed: re-read, reassess, and retry with current
evidence—never retry blindly.

Use the host write tool for new files, signatures/renames, cross-file changes,
or broader edits that do not fit those structural operations.

---

## 6. Compact decision table

| Need | Preferred action |
|------|------------------|
| Locate a symbol with CBM available | `graph_search` |
| Typed graph nodes/edges | `graph_query` / `graph_trace` |
| Compact raw CBM operation | `cbm_proxy` |
| Locate text/files without CBM | Claude `Grep` / `Glob` |
| Locate an exact semantic name without CBM | `workspace_query(type="find_entities")` |
| Understand a supported source file | `provide_code_context` |
| Understand several supported source files | `provide_code_context(files=[...])` |
| Exact known source lines/body | Native `Read` after context, or when context fails |
| One semantic entity/edge/dependency question | Single-form `workspace_query` |
| Several semantic questions in one scope | Batched `workspace_query(queries=[...])` |
| File-local call occurrences | `workspace_query(type: "calls_in_file")` |
| Dependency-cycle witness | `workspace_query(type: "has_cycle")` |
| Supported tracked structural edit | `apply_edit` |
| New/cross-file/signature edit | Host write tool |
| Savings/session dashboard | `context_stats` |

Before finishing a task, confirm that paths came from an authoritative result,
`workspaceRoot` was explicit, CBM absence was not reported as semantic absence,
and any completeness claim matches the response's actual coverage.
