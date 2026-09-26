# Workspace-query fidelity-aware auto-compilation proposal

**Status:** Proposed and deferred
**Recorded:** 2026-09-26
**Production behavior:** Unchanged
**Primary surfaces:** `workspace_query.entities_in_file`, `workspace_query.has_cycle`

## 1. Decision summary

`entities_in_file` should eventually auto-compile its explicit file when the
WorkspaceIndex has no sufficiently complete semantic projection for that file.
Its optional `fidelity` argument should mean **compile semantic facts at least
as completely as the requested fidelity requires**, including recompilation
when an existing index entry was produced at a lower semantic fidelity.

`has_cycle` must not adopt a superficially similar single-file shortcut. It is
a workspace-scoped graph property and currently has no target file. Compiling
one file before querying a partially populated graph can still return a clean
`false` while relevant files remain unindexed. Its completeness policy therefore
requires a separate architectural decision before implementation.

No production change is authorized by this document. It records the verified
boundary and the recommended future work.

## 2. Problem

`entities_in_file` currently resolves and authorizes `file_path`, then queries
only the existing WorkspaceIndex. When the file has not previously contributed
semantic facts, the response is indistinguishable from a genuinely empty file:

```json
{ "entities": [], "count": 0 }
```

The caller must infer that compilation may be missing, call
`provide_code_context`, and retry. That adds a tool round-trip and an avoidable
error-recovery reasoning step.

`has_cycle` also queries only the currently populated WorkspaceIndex. A `false`
answer says no cycle exists in the indexed evidence; it does not prove that the
effective workspace scope has complete indexed coverage.

The correctness risk is fidelity-sensitive. Some semantic facts are absent at
Low fidelity. For example, .NET `ControllerAction` edges are extracted only
when fidelity is not Low. Reusing a stale Low-fidelity index entry for a later
Medium or High request could silently omit facts.

## 3. Code-authority findings

### 3.1 `entities_in_file`

The production handler in `src/mcp/tool_handlers/query/entities.rs`:

1. validates `file_path` through `resolve_file_path_checked`;
2. enforces `workspaceRoot`, configured roots, and optional `withinPath`;
3. canonicalizes the resolved path; and
4. calls `WorkspaceIndex::entities_in_file` without compiling or checking
   semantic-fidelity coverage.

The existing empty response intentionally carries no discovery diagnostic.
Consequently, an uncompiled file and a compiled file with no registered
entities share the same response.

### 3.2 `has_cycle`

The production handler in `src/mcp/tool_handlers/query/graph.rs` resolves an
effective workspace scope and calls `WorkspaceIndex::has_cycle` or
`has_cycle_in_scope`. It accepts no target file and performs no coverage or
hydration step.

This is materially different from `entities_in_file`: the answer depends on
all relevant edge occurrences in the effective scope, not one explicit file.

### 3.3 `calls_in_file` is not a publication path

`calls_in_file` uses `compile_file_ir_candidate` at High fidelity and inspects
the resulting canonical IR directly. By design it publishes none of the
following:

- session IR baseline;
- semantic-edge snapshot;
- WorkspaceIndex entities or edges;
- persistence state; or
- compression statistics.

Future work may reuse its read-only candidate-compilation boundary, but cannot
reuse the operation wholesale for index hydration.

### 3.4 Existing name-based hydration is not a direct fit

Name-bearing queries discover candidate files through CBM or deterministic
filesystem fallback, compile candidates, and publish their semantic edges.
`entities_in_file` already has an authoritative path and needs no name-based
discovery. `has_cycle` has neither a target name nor a single authoritative
file, so exhaustive scope coverage is a different operation.

### 3.5 Fidelity is not globally monotonic today

`Fidelity` has the ordered-looking variants Low, Medium, High, Edit, and
Verbatim, but existing extractors do not uniformly treat later variants as
semantic supersets. Some use conditions such as:

```rust
if fidelity == Fidelity::High { /* additional semantic facts */ }
```

Therefore a generic enum ranking such as `Edit > High` is not a safe semantic
completeness rule. `provide_code_context` also bypasses structural compilation
for Verbatim responses.

## 4. Proposed `entities_in_file` contract

Add an optional `fidelity` argument using the existing public spelling:

```text
low | medium | high | edit | verbatim
```

Omission uses the configured default fidelity. Invalid values return JSON-RPC
`-32602`, consistent with other MCP fidelity-bearing tools.

For WorkspaceIndex semantic compilation, normalize the request to a semantic
fidelity:

| Requested fidelity | Semantic compilation fidelity |
|---|---|
| Low | Low |
| Medium | Medium |
| High | High |
| Edit | High |
| Verbatim | High |

Edit bodies and verbatim source bytes are irrelevant to WorkspaceIndex graph
facts. Mapping them to High avoids the current extractor equality trap and
requests the most complete semantic projection without publishing rendered
content.

The handler should then:

1. resolve and authorize the explicit file exactly as today;
2. determine the requested semantic fidelity;
3. inspect the WorkspaceIndex-owned fidelity for that canonical file;
4. compile when the file is absent or indexed below the requested level;
5. atomically replace the file's prior entities and edges with the newly
   compiled semantic edges, even when the new edge set is empty;
6. record the new WorkspaceIndex semantic fidelity; and
7. run `entities_in_file` against the refreshed index.

The operation should not render SCHEMA-v5, record compression statistics,
persist a context, or imply that the caller consumed file content.

### 4.1 “At least this fidelity” behavior

A file indexed at High satisfies later Low, Medium, High, Edit, and Verbatim
semantic requests under the normalization table. A file indexed at Low does
not satisfy Medium or High. Recompilation must replace rather than merge its
old contribution so facts absent from the newer source cannot survive.

Source freshness also matters: fidelity sufficiency must not permit reuse when
the indexed projection belongs to stale source bytes. The production design
must either record the source hash with the index entry or prove freshness
through an existing authoritative state boundary before reusing it.

## 5. State ownership and lifecycle

WorkspaceIndex, not the session context map, should own semantic coverage
metadata for each canonical file. `McpState::context_fidelity` describes the
session IR/rendering baseline and can diverge from index state because:

- `calls_in_file` compiles without publishing either state;
- name-based hydration publishes WorkspaceIndex facts without loading session
  IR;
- restore, delta, edit, delete, and persistence have distinct lifecycles; and
- a future query-only compile should not pretend a rendered context exists.

The preferred boundary is an atomic WorkspaceIndex file replacement carrying:

```text
canonical file identity
semantic fidelity
source hash or equivalent freshness identity
entity/edge occurrences
```

Removal, deletion, reset, edit publication, restore, and recompilation must
clear or replace the metadata together with the file's semantic occurrences.
An independent map is acceptable only if the implementation proves that it
cannot drift across those lifecycle operations.

## 6. `has_cycle` decision still required

### Option A — exhaustive scoped compilation

Before answering, enumerate every supported source file admitted by the
effective `workspaceRoot` plus optional `withinPath`, compile every missing,
stale, or lower-fidelity file, then run cycle detection.

Advantages:

- one-call definitive answer for the declared scope;
- removes the ambiguous incomplete-index `false`; and
- fidelity has a coherent workspace-wide meaning.

Costs and risks:

- potentially expensive on large workspaces;
- requires explicit failure and partial-coverage semantics;
- interacts with resource limits, exclusions, additional roots, symlinks, and
  unsupported files; and
- must not silently truncate candidate processing.

### Option B — explicit coverage diagnostics

Keep `has_cycle` index-only, but return actionable coverage information that
distinguishes a complete `false` from an incomplete-index `false`.

Advantages:

- bounded query cost;
- no surprise workspace-wide compilation; and
- makes current uncertainty explicit.

Costs:

- callers may still need a separate compilation operation; and
- does not achieve the proposed one-call workflow.

### Rejected — compile one seed file

Adding a file path, compiling only that file, and then querying the existing
workspace graph is insufficient. Other files may contain the missing edge that
closes a cycle. This preserves the same false-completeness failure under a more
confident-looking API.

### Recommendation

Implement `entities_in_file` independently. For `has_cycle`, choose Option A
only if definitive one-call workspace cycle detection justifies exhaustive
scope compilation. Otherwise choose Option B. Do not represent a partially
indexed graph as a complete negative answer.

## 7. Compatibility and response behavior

For `entities_in_file`, existing callers may omit `fidelity`; the configured
default applies. The only intended behavioral difference is that an authorized
existing source file can compile on first touch rather than returning an
ambiguous empty result.

Invalid, missing, unauthorized, or out-of-`withinPath` files must retain the
existing path-security and minimal-response contracts unless a separate public
error-policy change is explicitly approved.

Any response metadata should describe only actionable deviations. Do not add a
constant “auto-compile attempted” flag to every successful response.

## 8. RED/GREEN implementation plan

This is a reproducible behavioral gap, so implementation must follow the
tracked RED/GREEN procedure in `docs/agent/architecture.md`.

### Phase E1 — `entities_in_file`

Add focused tracked regressions proving:

1. a never-before-compiled file returns its real entities in one call;
2. an already indexed Low-fidelity .NET controller recompiles for Medium and
   exposes `ControllerAction` facts;
3. a later lower request reuses a sufficient higher-fidelity projection;
4. source changes invalidate or replace stale index facts;
5. recompilation to an empty edge set removes old facts;
6. source-written path/root/`withinPath` security remains unchanged;
7. invalid fidelity returns `-32602`;
8. query-only compilation does not create a rendered-context or persistence
   claim; and
9. the MCP schema advertises the optional fidelity contract accurately.

Observe RED against the current implementation, stash the exact regression,
implement the production boundary, restore the unchanged test, and observe
GREEN with the same focused command.

### Phase C1 — `has_cycle`

Do not write implementation tests until Option A or B is approved. The chosen
contract must include multi-file cases where the closing cycle edge is in a
previously unindexed file; a one-file fixture cannot prove workspace
completeness.

## 9. Documentation updates required on implementation

When implemented, update together:

- `src/mcp/tools.rs` public schema and descriptions;
- `docs/agent/tooling.md` workspace-query contract;
- `docs/ARCHITECTURAL_INVARIANTS.md` with the semantic-fidelity and index
  ownership invariant;
- `docs/agent/DISCOVERY_REGISTRY.md` if this originated from a live field
  discovery; and
- production-path tests under `src/tests/mcp/**`.

The implementation is complete only after tracing:

```text
workspace_query request
  -> path/scope/fidelity validation
  -> freshness and semantic-fidelity decision
  -> candidate compilation
  -> atomic WorkspaceIndex replacement
  -> query rerun
  -> MCP structured/text response
  -> live no-prior-provide verification
```
