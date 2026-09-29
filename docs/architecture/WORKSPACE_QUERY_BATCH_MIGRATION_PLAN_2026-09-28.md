# Workspace Query Batch Migration Plan

**Date:** 2026-09-28  
**Status:** Approved for incremental implementation on 2026-09-28. Phase 0 is
complete and Phase 1 RED contract work is in progress. No production behavior
has changed.  
**Scope:** Add an ordered, failure-isolated batch request form to the existing
`workspace_query` MCP tool while preserving the complete legacy single-query
contract.

---

## 1. Executive summary

`workspace_query` currently answers one semantic question per MCP tool call.
That boundary makes each answer easy to qualify, but it forces a caller that is
investigating several related entities or relationship directions to pay for
multiple tool calls and potentially repeat discovery work.

The proposed design adds a second, explicitly batched request form:

```json
{
  "workspaceRoot": "C:/repo",
  "withinPath": "src",
  "queries": [
    {
      "id": "service-callers",
      "type": "reverse_edges",
      "domain": "builtin",
      "entity_type": "Class",
      "name": "OrderService"
    },
    {
      "id": "dependencies",
      "type": "transitive_dependencies",
      "name": "OrderService",
      "depth": 2
    }
  ]
}
```

The existing single-query form remains supported without response-shape or
behavior changes. Batch execution validates one shared workspace scope,
prepares every valid item, deduplicates equivalent hydration work, then answers
index-backed items from one final `WorkspaceIndex` read snapshot. Results remain
separate, ordered, and correlated by a caller-supplied unique `id`.

A batch is explicitly **heterogeneous**: each item independently selects any of
the seven existing operations. A caller may combine `forward_edges`,
`reverse_edges`, `find_entities`, `transitive_dependencies`, `has_cycle`,
`entities_in_file`, and `calls_in_file` in one request. An item-local validation,
identity-resolution, discovery, compilation, or evaluation failure becomes that
item's error outcome and does not fail or suppress independent siblings.

This is an additive public-contract migration. It does not change semantic
identity, WorkspaceIndex authority, discovery authority, query completeness,
or any existing query-family behavior.

---

## 2. Motivation

### 2.1 Current caller cost

A caller asking about several symbols or several relationships for one symbol
must issue one tool call per question. For example, examining a service may
require separate calls for:

- incoming consumers;
- outgoing dependencies;
- a transitive dependency walk;
- related repository entities;
- precise calls in one selected source file.

Each call repeats MCP framing, argument parsing, scope construction, response
rendering, and potentially discovery checks. The discovery cache prevents some
provider work from repeating after successful completion, but it does not
remove the tool-call latency or guarantee that related answers observe the same
post-preparation index state.

### 2.2 Desired outcome

One `workspace_query` call can carry several independent questions while
retaining the truthfulness and debuggability of separate calls:

- every item retains its query type and identity;
- every item retains its own semantic result and completeness diagnostics;
- one failed item does not erase successful siblings;
- no result arrays from different questions are merged;
- no answer is silently truncated;
- equivalent hydration work is performed once per batch;
- index-backed answers observe the same final index snapshot.

---

## 3. Current production architecture

### 3.1 Public contract

`src/mcp/tools.rs` declares one object with one required `type`,
operation-specific fields from `tool_schemas::workspace_query_variants()`,
optional shared scope fields, and one output schema covering the seven result
families: entity discovery, forward/reverse edges, entities in a file,
transitive dependencies, cycle detection, and calls in a file.

### 3.2 Dispatch and response coupling

`handle_workspace_query` in `src/mcp/tool_handlers/query.rs` reads `type` and
dispatches directly to a family handler. Those handlers currently own both
query evaluation and response transmission. This is suitable for one request
but cannot safely be called repeatedly by a batch coordinator because each
item would attempt to send its own JSON-RPC response.

The first migration boundary must therefore separate:

```text
query evaluation -> typed outcome/error -> response serialization/transmission
```

The existing single-query entry point must then use that same internal
evaluator. A batch implementation must never duplicate the seven query-family
algorithms.

### 3.3 Scope authority

`query_scope` constructs the effective scope once per existing call:

```text
workspaceRoot + configured additional_roots
intersected with optional withinPath
```

`WorkspaceScope` remains the only admission authority. The batch form will put
`workspaceRoot` and `withinPath` at the batch top level and construct exactly
one shared scope. Individual items may not override or widen that scope.

### 3.4 Query preparation differences

The seven operations do not all obtain evidence the same way:

| Query family | Preparation/evidence boundary |
|---|---|
| `find_entities` | Registered hydration, then WorkspaceIndex |
| `forward_edges` | Registered hydration and identity resolution, then WorkspaceIndex |
| `reverse_edges` | Registered inbound-reference hydration and identity resolution, then WorkspaceIndex |
| `transitive_dependencies` | Registered hydration and identity resolution, then WorkspaceIndex traversal |
| `entities_in_file` | Source/fidelity-aware file projection preparation, then WorkspaceIndex |
| `has_cycle` | Existing WorkspaceIndex evidence only; never hydrates |
| `calls_in_file` | Fresh read-only canonical file candidate; never reads or publishes WorkspaceIndex facts |

Batch orchestration must preserve these differences. “Shared hydration” does
not authorize `has_cycle` or `calls_in_file` to begin hydrating, and it does not
authorize CBM-discovered relationships to become WorkspaceIndex facts.

### 3.5 Existing response authority

Single queries use the canonical MCP envelope: model-readable `content`,
machine-readable `structuredContent`, and a declared `outputSchema`.
`query/content.rs` emits `clean-ctx/workspace-query-answer` version 1; batching
requires a distinct envelope and must not reinterpret that format.

---

## 4. Approved invariants that remain unchanged

The implementation must preserve these existing architectural authorities:

- **MCP-001:** every success remains a canonical MCP result envelope;
- **WSC-001:** facts do not imply workspace-complete coverage;
- **WSC-002:** CBM/filesystem discovery supplies paths only; Clean-CTX alone
  determines WorkspaceIndex semantics;
- **WSC-003:** successful discovery completion remains session-state work and
  its existing generation/scoping rules remain authoritative;
- **WSC-004:** one declared workspace scope filters occurrence provenance and
  traversal during the walk;
- **WSC-005:** `calls_in_file` remains a fresh canonical file-local query and
  never becomes a WorkspaceIndex query;
- **WSC-006:** `entities_in_file` retains source-hash and semantic-fidelity
  coverage rules;
- **WSC-007:** `has_cycle` remains an index-evidence-only witnessed dependency
  query and never hydrates;
- **WSC-008:** incomplete identity resolves without guessing and ambiguity
  remains explicit.

The migration does not change Model C semantic identity, edge occurrence
identity, configured-root authorization, fidelity normalization, discovery
cache keys, source-cache ownership, or CBM cache ownership.

---

## 5. Proposed public contract

### 5.1 Two mutually exclusive request forms

`workspace_query.inputSchema` will accept exactly one of:

1. the existing single-query form, unchanged; or
2. the new batch form with shared scope plus `queries`.

The request must not contain both top-level `type` and `queries`. The batch
form is:

```json
{
  "workspaceRoot": "C:/repo",
  "withinPath": "src/orders",
  "queries": [
    { "id": "q1", "type": "find_entities", "name": "OrderService" },
    { "id": "q2", "type": "reverse_edges", "name": "OrderService" }
  ]
}
```

Rules:

- `queries` must be a non-empty array;
- every item must be an object;
- every item must carry a non-empty string `id`;
- IDs must be unique within the request;
- every item carries one existing query specification;
- items may select different query types within the same batch;
- item fields use the same validation and meaning as the corresponding legacy
  query;
- `workspaceRoot` and `withinPath` exist only at the batch top level;
- item-level scope fields are rejected rather than ignored;
- input order is semantically significant and preserved in output order.

### 5.2 Proposed batch-size policy

This plan recommends a maximum of **32 items** per batch. An oversized request
would be rejected as invalid parameters before discovery, compilation, or
index work. Results would never be silently truncated.

The value is a proposed operational guard, not an implementation detail. It is
a new externally observable limit and therefore requires explicit approval.
Alternatives are recorded in Section 9.

### 5.3 Structured response

The batch `structuredContent` shape is proposed as:

```json
{
  "batch": true,
  "results": [
    {
      "id": "q1",
      "type": "find_entities",
      "status": "ok",
      "result": {
        "entities": [],
        "count": 0
      }
    },
    {
      "id": "q2",
      "type": "reverse_edges",
      "status": "error",
      "error": {
        "code": -32602,
        "message": "..."
      }
    }
  ]
}
```

Contract rules:

- `results` has exactly one element per accepted input item;
- result order equals input order;
- `id` and `type` repeat the item correlation identity;
- `status` is exactly `ok` or `error`;
- an `ok` item carries `result` with the same structured payload the
  corresponding legacy call would return;
- an `error` item carries `error` and no fabricated empty semantic result;
- error codes/messages reuse existing query-family classifications;
- the batch itself succeeds as an MCP tool result when its envelope and shared
  scope are valid, even when one or more items fail;
- successful siblings are returned even when every other item fails;
- malformed batch structure, duplicate IDs, invalid shared scope, or an
  approved batch-size violation rejects the whole request with JSON-RPC
  `-32602` before query work begins.

### 5.4 Model-facing content

Batch `content` will use a distinct envelope:

```text
// WORKSPACE-QUERY-BATCH v1; structuredContent remains authoritative
{
  "schema": "clean-ctx/workspace-query-batch-answer",
  "schema_version": 1,
  "scope": { ... },
  "results": [ ... ]
}
```

Each successful item retains the existing single-query model result and
completeness information under its own result boundary. Each failed item is
rendered as a compact structured error. The renderer must not flatten or merge
item result families.

The existing `clean-ctx/workspace-query-answer` version 1 output remains
byte-for-byte governed by the legacy path and is not version-bumped merely
because batching exists.

### 5.5 Failure isolation and side effects

Batch execution is best-effort per item, not transactional. `workspace_query`
is read-only with respect to source files, but existing preparation may warm
WorkspaceIndex, semantic coverage, source caches, or discovery-completion
state. Those valid session-state effects are not rolled back when a sibling
item fails.

Failure rules:

- invalid shared scope fails the entire request before item execution;
- item-local validation errors produce item-local errors;
- discovery or compilation failure affects only items that depend on the
  failed preparation key when safe dependency attribution is possible;
- unrelated items continue;
- no failed item is represented as `count: 0` success;
- successful discovery/compilation state remains reusable under existing
  lifecycle rules.

---

## 6. Proposed internal architecture

### 6.1 Typed request boundary

Introduce internal typed representations rather than passing unvalidated
`serde_json::Value` throughout the coordinator. The exact names may change,
but the responsibilities should resemble:

```rust
struct WorkspaceQueryRequest {
    id: Option<String>,
    operation: WorkspaceQueryOperation,
}

enum WorkspaceQueryOperation {
    FindEntities { /* existing fields */ },
    ForwardEdges { /* existing fields */ },
    ReverseEdges { /* existing fields */ },
    EntitiesInFile { /* existing fields */ },
    TransitiveDependencies { /* existing fields */ },
    HasCycle { /* existing fields */ },
    CallsInFile { /* existing fields */ },
}
```

This is not a new generic query framework. It is a closed representation of
the seven already-public operations that lets validation, preparation,
evaluation, and serialization remain distinct.

### 6.2 Typed outcome boundary

Family handlers should return a structured internal outcome or typed query
error instead of transmitting JSON-RPC directly. The outer single or batch
adapter owns MCP serialization and `send_response`.

Required preservation:

- existing legacy success payloads remain unchanged;
- existing legacy JSON-RPC error codes and messages remain unchanged;
- existing ordering and occurrence behavior remain unchanged;
- no handler sends a response behind the coordinator's back.

### 6.3 Preparation plan

The batch coordinator builds an explicit preparation plan after validation:

1. construct the shared `WorkspaceScope` once;
2. classify item-local validation failures;
3. prepare required `entities_in_file` semantic projections using existing
   coverage rules;
4. prepare fresh canonical candidates required by `calls_in_file` without
   publishing them to WorkspaceIndex;
5. collect hydration keys for eligible index-backed items;
6. deduplicate hydration by the existing semantic key:
   `(discovery mode, exact query name, effective configured roots)`;
7. run every required unique hydration job through the existing discovery and
   compilation authorities;
8. retain each job's complete `HydrationReport` for the dependent item
   outcomes.

No new persistent cache is introduced. WSC-003's existing discovery cache
remains the cross-request completion authority; batch deduplication is
request-local orchestration only.

### 6.4 Final snapshot evaluation

After all successful preparation that can change WorkspaceIndex is complete,
the coordinator acquires one final WorkspaceIndex read guard and evaluates all
valid index-backed items against that guard in input order.

This yields one precise consistency claim:

> Every index-backed result in one batch is evaluated against the same
> post-preparation WorkspaceIndex snapshot.

It does not claim filesystem transactionality. `calls_in_file` remains a fresh
file-local candidate answer and explicitly reports its existing
`fresh_canonical_file_ir` authority. The response must not imply that a
file-local result and index-backed result came from one canonical storage
snapshot.

The implementation must not retain the WorkspaceIndex read guard while doing
filesystem IO, compilation, CBM calls, or other preparation work.

### 6.5 Identity resolution

WSC-008 identity resolution must become evaluator-friendly without changing
its semantics. Hydration occurs in the preparation stage; final identity
selection occurs against the shared snapshot. Exactly one identity proceeds,
zero remains explicit not-found, and multiple identities remain deterministic
ambiguity.

No batch-level identity inference or cross-item answer reuse is allowed. One
item may not use another item's result as an implicit selector.

### 6.6 Error representation

Use one internal error type capable of preserving the existing code, message,
and optional data. The single-query adapter maps it back to the existing
JSON-RPC error. The batch adapter nests it under the corresponding result item.

This avoids parsing already-serialized JSON-RPC errors and prevents error
classification from drifting between single and batch paths.

---

## 7. Phased implementation plan

Every phase is independently reviewable. Repository-wide gates are reserved
for finalization; focused user-run tests provide phase evidence.

### Phase 0 - Baseline inventory and contract lock

**Status:** Complete (2026-09-28). This phase changed documentation only; no
production code, public schema, or test behavior was modified.

Deliverables:

- inventory every workspace-query handler, schema variant, result field,
  response renderer, validation rule, hydration path, and test module;
- capture representative legacy success and error payloads for all seven
  operations through registered MCP dispatch;
- map which operations mutate only session discovery/index coverage during
  preparation;
- record current source-file line counts before activating modules;
- identify any files that require semantic decomposition to stay below the
  615-line active-file ceiling.

Checkpoint 0 exit criteria:

- all seven production lifecycles are documented;
- exact compatibility fixtures exist for legacy single-query behavior;
- no production code has changed;
- implementation module boundaries are agreed before file activation.

#### Phase 0 production inventory

The registered entry point is
`mcp::tool_handlers::query::handle_workspace_query`. It reads one `type` and
delegates to a family handler. Every family handler currently owns validation,
evaluation, content rendering, MCP envelope construction, and the final
`send_response`, which is the exact coupling Phase 2 must remove before a batch
coordinator can reuse it.

| Operation | Production owner | Preparation/authority | Success payload | Principal item-local failures |
|---|---|---|---|---|
| `find_entities` | `query/entities.rs::handle_find_entities` | One registered name-discovery hydration cycle; scoped WorkspaceIndex occurrence query | `entities`, `count`, optional `discovery` | missing/empty `name`; invalid scope; hydration/compilation failure |
| `forward_edges` | `query/edges.rs::handle_forward_edges` | Exact identity fast path or one hydrated WSC-008 resolution; scoped outgoing WorkspaceIndex occurrences | `edges`, `count`, `resolved_identity`, optional `discovery` | missing/empty `name`; invalid scope; not-found/ambiguous identity; hydration failure |
| `reverse_edges` | `query/edges.rs::handle_reverse_edges` | Exact identity fast path or one inbound-reference hydrated resolution; scoped incoming WorkspaceIndex occurrences | `edges`, `count`, `resolved_identity`, optional `discovery` | same classifications as `forward_edges` |
| `entities_in_file` | `query/entities.rs::handle_entities_in_file` | Trusted path plus source-hash/fidelity coverage; conditionally compiles and atomically replaces that file's semantic projection | `entities`, `count` | invalid fidelity; source/preflight/compile failure; missing or unanswerable path retains the legacy empty-success contract |
| `transitive_dependencies` | `query/graph.rs::handle_transitive_dependencies` | Exact or hydrated WSC-008 start identity; scope applied during WorkspaceIndex traversal | `dependencies`, `count`, `depth_used`, `resolved_identity`, optional `discovery` | missing/empty `name`; invalid scope; not-found/ambiguous identity; hydration failure |
| `has_cycle` | `query/graph.rs::handle_has_cycle` | One scoped WorkspaceIndex read; dependency evidence only; never hydrates | `has_cycle`, `cycle`, `coverage`, identity model/ambiguity fields | invalid `kind`; invalid scope |
| `calls_in_file` | `query/calls.rs::handle_calls_in_file` | Trusted path; fresh High-fidelity canonical candidate and hierarchy; never reads or publishes WorkspaceIndex facts | `file`, typed `owner`, `method`, `overload_count`, `overloads`, `count` | malformed owner/method selector; invalid scope/path; compilation/projection failure; ambiguous typed owner |

#### Phase 0 validation and failure lock

The migration must preserve these current single-query classifications while
representing the same failure as an item-local error inside a valid batch:

| Boundary | Current single-query behavior to preserve |
|---|---|
| Missing/unknown top-level `type` | JSON-RPC `-32602` with the supported-operation list |
| Missing operation field | JSON-RPC `-32602` with the operation-specific message |
| Unauthorized `withinPath` | JSON-RPC `-32602` before the index is consulted |
| Partial identity resolves to zero or several identities | JSON-RPC `-32602`; ambiguity/not-found message and deterministic candidates where applicable |
| Hydration failure | JSON-RPC `-32603`; never an empty semantic success |
| `entities_in_file` path is missing/outside accepted roots or narrowing | Legacy successful `{entities: [], count: 0}` rather than an error |
| Invalid `entities_in_file` fidelity | JSON-RPC `-32602` |
| Invalid `has_cycle.kind` | JSON-RPC `-32602` |
| Invalid `calls_in_file` selector/path | JSON-RPC `-32602` |
| Candidate compile/read/projection failure | Existing internal/server error mapping; never fabricated empty success |

Batch-wide rejection is reserved for the new batch structure and its shared
authority: malformed/empty `queries`, duplicate/missing IDs, mixed top-level
`type` plus `queries`, item scope overrides, invalid shared scope, and (if
approved) more than 32 items. Once those checks pass, the table above maps to
the affected item's `error` and independent siblings continue.

#### Phase 0 lifecycle and snapshot findings

The existing code mixes preparation and evaluation in three distinct ways:

1. `run_query_with_hydration` reads the index, hydrates, then rereads it for one
   closure;
2. WSC-008 resolution may hydrate and select identity before the edge/traversal
   handler performs its final index read;
3. `entities_in_file` may replace one semantic projection before reading it,
   while `calls_in_file` compiles an unpublished candidate.

Therefore heterogeneous batching cannot be implemented by calling current
handlers repeatedly. The locked orchestration order is:

```text
parse request -> validate shared scope -> validate/classify items
-> prepare file-local/index-coverage requirements
-> deduplicate and run eligible hydration jobs
-> acquire one final WorkspaceIndex read view
-> resolve identities and evaluate every index-backed item
-> combine those outcomes with prepared file-local outcomes
-> render/send one MCP response
```

The final read guard is never held across source IO, compilation, CBM calls, or
hydration. `calls_in_file` remains explicitly outside the shared-index snapshot
claim because its fresh canonical candidate is a different authority.

#### Phase 0 compatibility authorities

Existing tracked tests already lock every single-query family through the real
registered dispatch boundary:

| Contract | Existing authority |
|---|---|
| Supported operations, required fields, unknown/missing type | `src/tests/mcp/workspace_query.rs` |
| Success shapes for entity, edge, file, traversal, and cycle families | `src/tests/mcp/workspace_query_dispatch.rs` |
| Exact model-content envelope/version and occurrence projection | `src/tests/mcp/workspace_query_content.rs` |
| Canonical MCP envelope and structured output | shared `assert_valid_mcp_envelope` use across workspace-query suites |
| Public polymorphic input requirements | `src/tests/mcp/tools.rs` |
| Partial identity success/not-found/ambiguity | `src/tests/mcp/workspace_query_identity_resolution.rs` |
| Cycle witness, kind error, coverage, and side-effect boundary | `src/tests/mcp/workspace_query_cycle_witness.rs` |
| File-local fidelity/source coverage lifecycle | `src/tests/mcp/workspace_query_entities_auto_compile.rs` |
| Typed owner, overloads, occurrence order, fresh source, and schema | `src/tests/mcp/workspace_query_calls_in_file.rs` |
| Workspace/additional-root/`withinPath` authority | `workspace_query_scope*.rs` and `workspace_query_within_path*.rs` |
| Hydration provider, completeness, cache, and diagnostics | `workspace_query_2.rs` through `workspace_query_9.rs` plus diagnostics suites |

Phase 1 will add a dedicated `workspace_query_batch.rs` RED suite rather than
expanding any near-ceiling legacy test module. Phase 2 must run the existing
single-query authorities unchanged after extracting the evaluator; these tests
are the compatibility fixtures and may not be rewritten to accommodate the
refactor.

#### Phase 0 file-size and decomposition audit

Production line counts at the baseline:

| File | Lines | Migration treatment |
|---|---:|---|
| `query.rs` | 311 | Keep as dispatch/shared-scope root; declare focused child modules |
| `query/entities.rs` | 280 | Extract evaluation without adding batch orchestration |
| `query/edges.rs` | 207 | Extract evaluation without duplicating direction logic |
| `query/graph.rs` | 198 | Preserve traversal/cycle authority |
| `query/calls.rs` | 279 | Preserve canonical-candidate authority |
| `query/identity.rs` | 91 | Convert response-sending selection into typed result/error |
| `query/content.rs` | 120 | Keep legacy renderer; place batch rendering in a child file |
| `query/diagnostics.rs` | 241 | Reuse unchanged per-item diagnostic projection |
| `hydration.rs` | 581 | Do not grow with batch orchestration; compose its existing API externally |
| `tools.rs` | 438 | Keep tool registration; delegate batch schema construction to `tool_schemas.rs` children if needed |
| `tool_schemas.rs` | 71 | Own request/output schema fragments; decompose by tool if growth warrants it |

Planned semantic boundaries:

- `query/request.rs`: closed operation/request parsing and validation;
- `query/outcome.rs`: typed semantic outcome and preserved error information;
- `query/prepare.rs`: request-local preparation requirements and reports;
- `query/batch.rs`: ordered heterogeneous orchestration only;
- `query/content/batch.rs`: batch model-facing renderer, leaving the legacy
  renderer unchanged;
- `src/tests/mcp/workspace_query_batch.rs`: RED/GREEN public-contract and
  production-dispatch coverage;
- optional sibling batch test files if the first test module approaches 600
  lines, split by schema, orchestration, and authority boundaries.

No oversized production file must be created, and the design does not route
work into the 581-line hydration owner. Existing test files at 601 and 604
lines remain untouched; activating either would require semantic decomposition.

#### Phase 0 checkpoint evidence

- Current implementation and intended boundary are now recorded above.
- All seven production paths, result families, error classes, preparation
  effects, and test authorities are inventoried.
- The legacy behavior reference is the unchanged production implementation plus
  the existing tracked dispatch/content/schema regressions.
- The module plan keeps parsing, preparation, evaluation, orchestration, and
  rendering separate without introducing a framework or second query path.
- No Cargo command was run and no unrun gate is reported as passing.
- Phase 1 remains blocked on the explicit architectural approval in Section 10,
  especially the externally observable 32-item limit.

### Phase 1 - RED batch contract regressions

**Status:** Complete (2026-09-28). The maintainer approved continuation after
reviewing the completed Phase 0 checkpoint and the decisions in Section 10.
The operator reported all 11 tracked batch regressions compiled and failed
behaviorally against the legacy single-query-only production path.

Add tracked tests under `src/tests/mcp/` and register them through the existing
`#[path = "..."]` convention. The initial regressions must compile against the
legacy API but fail behaviorally because batching is not yet supported.

Required RED contracts:

1. two different query types succeed in one call and preserve input order;
2. two targets of one query type retain separate result boundaries;
3. IDs are echoed and duplicate IDs reject the whole batch;
4. one invalid item does not erase a valid sibling;
5. shared invalid `withinPath` rejects before any query work;
6. item-level scope overrides are rejected;
7. equivalent hydration requirements execute once within the batch;
8. distinct hydration names/modes both execute and retain their own reports;
9. every index-backed item observes the same final post-preparation snapshot;
10. `has_cycle` remains non-hydrating inside a mixed batch;
11. `calls_in_file` remains fresh, file-local, and unpublished inside a mixed
    batch;
12. `entities_in_file` retains fidelity/source coverage behavior;
13. batch content and structuredContent conform to MCP-001 and the declared
    output schema;
14. empty, malformed, oversized, and mixed single+batch requests receive the
    approved validation result;
15. legacy single-query responses remain unchanged.

The mixed-type regression must include at least one index-backed operation,
one operation with a different preparation authority, and one deliberately
failing item between successful siblings. This proves heterogeneous dispatch,
input-order preservation, and failure isolation through the real MCP response
rather than only through an internal evaluator.

RED procedure:

- the operator runs the focused test target and observes behavioral RED;
- compile errors do not count as RED;
- stash only the new tests and required test registration;
- implement Phases 2-5 without modifying the stashed tests;
- restore the exact tests and run the same command for GREEN.

Authored RED artifacts (2026-09-28):

| Artifact | Scope | SHA-256 before RED execution |
|---|---|---|
| `src/tests/mcp/workspace_query_batch.rs` | heterogeneous dispatch, ordering, item isolation, shared/structural validation, schema, MCP/content envelope | `c5a1307e10bbc6feb415a3c01b67bed6f883ae1b28cd1011f2db32be8de03500` |
| `src/tests/mcp/workspace_query_batch_preparation.rs` | hydration-key deduplication, distinct discovery work, final snapshot, unpublished file-local calls | `9e264ebd1ae2061fbd01addab58b7bbd40a7dab6b03bb7e05954ae18b4d85333` |

These post-format hashes are the restore authority for the unchanged
RED-to-GREEN procedure.

The first operator attempt exposed two fixture compile defects (`PathBuf`
passed to a string-key helper and a moved fixture path). That attempt was not
counted as RED. Both test-only defects were corrected before production work,
the hashes were replaced, and the RED procedure restarted. The next compiled
run correctly exposed the missing batch path in ten tests, but one scope test
passed against the unrelated legacy “missing type” error. Structural validation
could accept the same wrong classification. Those assertions now require their
intended batch-specific error reasons; that mixed-validity run is also not
counted as the final RED checkpoint. The hashes above identify the tightened
contract for the next clean RED run.

Final RED evidence (operator run, 2026-09-28):

- command: `cargo test --all-features mcp::tool_handlers::query::tests_batch -- --nocapture`;
- result: 11 batch tests failed, with no compilation error;
- heterogeneous dispatch, content, preparation, hydration, and snapshot tests
  failed because the legacy handler returned the top-level “Missing required
  argument: 'type'” response instead of a batch result;
- the schema test failed because no `queries` request branch exists;
- the shared-scope test rejected the unrelated legacy error because it did not
  classify `withinPath`;
- the structural-validation test rejected the unrelated legacy error because
  it did not report the required batch-specific reason.

Every final regression therefore demonstrated an approved missing contract.
The two files identified by the final hashes above, plus only their module
registration, are the test-only stash set for production implementation.

Checkpoint 1 exit criteria:

- failures demonstrate missing batch behavior rather than fixture defects;
- test hashes are recorded before stashing;
- production behavior remains unchanged.

### Phase 2 - Extract typed single-query evaluation

**Status:** Complete (2026-09-28). Production extraction is implemented, and
the maintainer reported the complete legacy workspace-query target GREEN.

Deliverables:

- introduce the closed typed operation representation;
- introduce the internal outcome/error boundary;
- refactor family handlers to return outcomes rather than sending responses;
- make the legacy dispatcher the sole single-query response adapter;
- preserve the existing content renderer and structured payloads;
- decompose `query.rs`, `tools.rs`, or activated child modules along semantic
  boundaries if required by the active-file ceiling.

This phase is a behavioral-preservation refactor. It must not expose batching
yet.

Checkpoint 2 exit criteria:

- every legacy query routes through the typed evaluator;
- no family evaluator calls `send_response`;
- representative success and error payloads match the Phase 0 fixtures;
- all existing focused workspace-query tests remain GREEN;
- no second implementation path exists.

Implementation checkpoint evidence (2026-09-28):

- `query/request.rs` owns the closed seven-variant operation identity and the
  single family-evaluation dispatch;
- `query/outcome.rs` owns typed semantic success, typed code/message/data
  failure, legacy content rendering, and the one family-result response sender;
- entity, edge, traversal, cycle, and file-local-call modules return
  `QueryResult` and no longer transmit responses;
- WSC-008 identity resolution returns `Result<IdentitySelection,
  QueryFailure>` and preserves deterministic candidate data;
- missing/unknown top-level operation remains at the registered outer handler,
  which delegates every recognized operation through the closed enum;
- the legacy content renderer and every structured payload construction remain
  unchanged in meaning and order;
- searches confirm no family/identity module calls `send_response`; only the
  registered outer validation boundary and `outcome::send_single` transmit;
- the active-file size guard passes with eight active production files; every
  modified/new Rust file remains below 600 lines;
- standalone Rust formatting and `git diff --check` pass;
- no Cargo command was agent-run and no unrun compatibility suite is reported
  as passing.

Operator checkpoint:

- command: `cargo test --all-features mcp::tool_handlers::query -- --nocapture`;
- result: GREEN;
- scope: the complete registered workspace-query module, including the legacy
  dispatch, scope/provenance, hydration, identity, cycle, file-coverage,
  file-local-call, diagnostics, and envelope regressions currently present in
  the production worktree;
- the Phase 1 batch RED tests remained absent in their test-only stash and were
  not weakened or used to claim this compatibility result.

### Phase 3 - Separate preparation from snapshot evaluation

**Status:** Complete (2026-09-28). The preparation/final-view boundary is
implemented, and the maintainer reported the complete legacy workspace-query
target GREEN.

Deliverables:

- express hydration requirements without executing a final query early;
- separate WSC-008 hydration from final identity selection;
- expose existing `entities_in_file` projection preparation as a reusable
  internal operation;
- expose `calls_in_file` candidate preparation without publishing index state;
- evaluate index-backed operations against a caller-provided WorkspaceIndex
  read view;
- preserve per-operation hydration diagnostics.

Checkpoint 3 exit criteria:

- single-query behavior remains equivalent;
- preparation completes before the final index read guard is acquired;
- no IO, compilation, or CBM call occurs while that guard is held;
- `has_cycle` and `calls_in_file` retain their existing authority boundaries;
- identity ambiguity/not-found behavior is unchanged.

Implementation checkpoint evidence (2026-09-28):

- `query/prepare.rs` owns `PreparedQuery`, which distinguishes a deferred
  WorkspaceIndex evaluator from an already-completed file-local success/failure;
- the legacy dispatcher now performs `operation.prepare(...)` before
  `evaluate_single(...)`; only the latter acquires the final index read view for
  an index-backed outcome;
- `find_entities`, exact edge queries, and exact transitive traversal preserve
  their established initial-read-before-hydration sequence, then defer their
  authoritative result until the final view;
- incomplete edge/traversal identity still hydrates once inside WSC-008
  resolution and preserves deterministic ambiguity/not-found failures;
- `entities_in_file` performs trusted-path, source-hash, fidelity, preflight,
  compilation, and atomic projection replacement during preparation, then
  defers only the final projection read;
- `has_cycle` prepares scope only, never hydrates, and evaluates its witness and
  ambiguity evidence from the final index view;
- `calls_in_file` compiles and projects its fresh canonical candidate during
  preparation and becomes a ready outcome; it never reads or publishes
  WorkspaceIndex facts;
- no final index guard is held across filesystem IO, source compilation, CBM
  discovery, or hydration;
- the former production hydration helper is now test-only because existing
  hydration suites use it to retain their focused lifecycle assertions; the
  production path has one preparation implementation, not two;
- standalone formatting, `git diff --check`, and the active-file size guard
  pass; no Cargo command was agent-run.

Operator checkpoint:

- command: `cargo test --all-features mcp::tool_handlers::query -- --nocapture`;
- result: GREEN;
- the full legacy workspace-query surface remained behaviorally compatible
  after separating preparation from final-view evaluation;
- the Phase 1 batch RED tests remained unchanged and isolated in `stash@{0}`.

### Phase 4 - Add request-local batch orchestration

**Status:** Implementation complete (2026-09-28); preserved Phase 1 batch
regressions remain isolated until the public schema phase is complete, so their
unchanged GREEN acceptance is still pending.

Deliverables:

- parse and validate the batch form;
- validate one shared scope before item work;
- preserve input order and unique IDs;
- build and deduplicate request-local preparation jobs;
- execute preparation with dependency-aware failure attribution;
- acquire one final index snapshot;
- evaluate valid index-backed items in order;
- evaluate/project prepared file-local items without changing their authority;
- create exactly one outcome per accepted input item.

Initial implementation should be sequential and deterministic. Parallel
hydration or compilation is explicitly out of scope; it would introduce new
concurrency, ordering, and resource-policy decisions without being necessary
to eliminate multiple MCP calls.

Checkpoint 4 exit criteria:

- equivalent hydration keys execute once per request;
- no semantic answer is shared across items merely because their names match;
- per-item errors do not suppress independent successes;
- all index-backed outcomes use one final read snapshot;
- no new cache, lock, worker, or background task exists.

Implementation checkpoint evidence (2026-09-28):

- `query/batch.rs` validates the complete batch structure before item work,
  preserves input order and correlation IDs, and produces exactly one outcome
  for every accepted item;
- one request-local `PreparationContext` owns the shared scope and memoizes
  hydration outcomes by exact name and discovery mode, including failed jobs;
- preparation remains sequential and independent item failures become typed
  item outcomes without suppressing sibling preparation or evaluation;
- every deferred index-backed evaluator runs under one final post-preparation
  WorkspaceIndex read guard;
- incomplete identity selection is deferred into that same final view, so a
  later preparation job cannot make an earlier item observe stale identity
  evidence;
- `calls_in_file` remains a ready, fresh canonical candidate result and never
  publishes WorkspaceIndex facts;
- the coordinator adds no persistent cache, synchronization primitive, worker,
  task, or background execution;
- standalone formatting, `git diff --check`, and the active-file size guard
  pass; no Cargo command was agent-run.

Operator checkpoint:

- command: `cargo test --all-features mcp::tool_handlers::query -- --nocapture`;
- result: GREEN after narrowing the obsolete hydration-eligibility helper and
  its import to test builds;
- scope: compilation plus the complete registered legacy workspace-query test
  surface currently in the production worktree;
- the Phase 1 batch RED tests remain byte-identical and isolated in
  `stash@{0}`; their GREEN run will occur after Phase 5 exposes the final public
  schema and rendering contract.

### Phase 5 - Public schema and batch rendering

**Status:** Complete (2026-09-28). The public catalog and distinct batch
envelope are implemented, and the unchanged Phase 1 batch regressions are
GREEN.

Deliverables:

- extend `workspace_query.inputSchema` with mutually exclusive single and batch
  forms;
- declare the batch result union in `outputSchema`;
- retain the legacy structured result schema;
- add `clean-ctx/workspace-query-batch-answer` version 1 rendering;
- keep item results visibly separated and errors compact;
- update tool descriptions so clients can select batching intentionally.

Checkpoint 5 exit criteria:

- `tools/list` accurately describes both request forms and all possible batch
  result items;
- both single and batch responses satisfy MCP-001;
- legacy content remains on `workspace-query-answer` version 1;
- no domain fields leak directly into the MCP result envelope;
- schema-validation regressions are GREEN.

Implementation checkpoint evidence (2026-09-28):

- `workspace_query.inputSchema` now declares seven legacy single-operation
  branches plus one mutually exclusive batch branch;
- batch items reuse the seven operation-specific validation branches, add a
  required non-empty correlation ID, exclude item-level scope fields, and
  declare the approved 1..=32 size bound;
- `outputSchema.results` declares ordered `ok` and `error` item variants while
  retaining every legacy top-level result property;
- successful model-facing items reuse the legacy query, completeness, and
  result projection, while failed items carry only compact typed error data;
- batch content uses `clean-ctx/workspace-query-batch-answer` version 1 and the
  legacy path remains on `clean-ctx/workspace-query-answer` version 1;
- tool guidance now explains heterogeneous batching, shared scope, hydration
  deduplication, failure isolation, and the unchanged `has_cycle` and
  `calls_in_file` authority boundaries;
- standalone formatting, `git diff --check`, and the active-file size guard
  pass; no Cargo command was agent-run.

Operator checkpoint:

- command: `cargo test --all-features mcp::tool_handlers::query::tests_batch -- --nocapture`;
- result: GREEN;
- scope: the restored batch contract, schema, failure-isolation, hydration
  deduplication, final-snapshot, and file-local authority regressions.

### Phase 6 - Restore unchanged RED tests and integration verification

**Status:** In progress. The Phase 1 tests are restored byte-identically at
Git's normalized blob boundary and the original focused batch target is GREEN;
the post-renderer legacy compatibility run remains pending.

Deliverables:

- restore the Phase 1 tests byte-identically;
- verify their recorded hashes;
- run the same focused target and observe GREEN;
- run the existing single-query suites most exposed to the refactor:
  scope/provenance, hydration, identity resolution, cycle witness,
  `entities_in_file`, `calls_in_file`, diagnostics, and envelope contracts;
- add a registered production-dispatch integration covering a mixed batch from
  MCP request through response serialization.

Checkpoint 6 exit criteria:

- unchanged RED regressions are GREEN;
- legacy single-query regressions are GREEN;
- mixed batch production dispatch is GREEN;
- test results are reported separately from any optional live harness.

Integration checkpoint evidence (2026-09-28):

- restored `workspace_query_batch.rs` blob:
  `30d139f35234873eff353e04ac72546daeed080c`;
- restored `workspace_query_batch_preparation.rs` blob:
  `217aa299043251e9c4c9e8eb661d805d039c5fd5`;
- both hashes exactly match their corresponding `stash@{0}^3` blobs; raw
  filesystem SHA-256 differs only because checkout applies the repository's
  CRLF worktree convention;
- the maintainer ran the original focused batch target and reported GREEN;
- the full registered workspace-query target must still be rerun because Phase
  5 factored the legacy renderer while preserving its envelope.

### Phase 7 - Live client acceptance

**Status:** Pending.

Where a relevant MCP client can consume the new schema, verify one real
workspace scenario containing:

- at least two index-backed query types;
- at least two distinct names;
- one query with non-empty results;
- one item-local ambiguity, not-found, or validation error;
- repeated execution demonstrating discovery completion reuse;
- comparison against equivalent legacy single calls.

The expected semantic payload for each successful item must match its legacy
single-call equivalent. The batch is allowed to reduce tool calls and provider
work; it is not allowed to change facts.

Checkpoint 7 exit criteria:

- the client renders or consumes the batch response correctly;
- successful item semantics match equivalent single calls;
- failure isolation is visible and actionable;
- any reproducible field discovery is recorded in
  `docs/agent/DISCOVERY_REGISTRY.md` and distilled into a tracked regression.

### Phase 8 - Documentation, audit, and final gate

**Status:** Pending.

Deliverables:

- add a narrowly scoped batch invariant to
  `docs/ARCHITECTURAL_INVARIANTS.md` if the completed implementation establishes
  the proposed snapshot/failure-isolation contract;
- update `docs/agent/tooling.md` with single-versus-batch selection guidance;
- correct stale comments and schema descriptions;
- perform the post-task architectural audit from
  `docs/agent/architecture.md`;
- run the complete user-owned Final Verification Gate from
  `docs/agent/verification.md`.

Checkpoint 8 exit criteria:

- production code, schemas, tests, and documentation describe the same
  contract;
- old direct-response paths and transitional adapters are removed;
- every active Rust file is at most 615 lines, preferably 600 or fewer;
- the complete final gate is reported GREEN by the operator;
- no unrun gate is described as passing;
- no critical or high-severity audit gap remains.

---

## 8. Verification matrix

| Contract | Primary tracked authority |
|---|---|
| Legacy single request compatibility | Existing workspace-query suites plus Phase 0 payload locks |
| Request-form exclusivity | New batch schema/dispatch regressions |
| Unique ordered correlation IDs | New batch contract regressions |
| Shared scope cannot be widened | Existing WSC-004 suites plus mixed-batch scope regressions |
| Per-item failure isolation | New mixed success/error dispatch regression |
| Hydration deduplication | Existing discovery instrumentation extended for batch |
| Same final index snapshot | New preparation-order/snapshot regression |
| CBM remains discovery-only | Existing WSC-002 regressions plus mixed batch control |
| Discovery cache lifecycle unchanged | Existing WSC-003 suites |
| `has_cycle` never hydrates | Existing WSC-007 suite plus mixed batch control |
| `calls_in_file` remains unpublished | Existing WSC-005 suite plus mixed batch control |
| `entities_in_file` coverage remains truthful | Existing WSC-006 suite plus mixed batch control |
| Identity never guessed | Existing WSC-008 suite plus per-item ambiguity regression |
| MCP envelope/schema conformance | Existing shared envelope helper plus batch output-schema tests |
| Live client reachability | Phase 7 operator scenario |

---

## 9. Decisions, alternatives, and tradeoffs

### 9.1 Batch size

**Recommended:** maximum 32 items; reject larger batches before work.

Tradeoff: this bounds one request's orchestration and response size while still
covering realistic multi-question investigations. It is externally observable
and requires approval.

Alternatives:

- **No fixed maximum:** simplest contract and fully exhaustive, but one local
  request can monopolize discovery/compilation and create an unbounded response.
- **Configuration-driven maximum:** flexible but introduces new global
  configuration, defaults, documentation, and compatibility policy before
  there is evidence that configurability is needed.
- **Silent truncation:** rejected because it violates completeness and makes
  omitted questions indistinguishable from answered questions.

### 9.2 Item identifiers

**Recommended:** require caller-supplied, unique, non-empty string IDs.

Alternative index-only correlation is smaller but fragile when callers reorder
or compose requests. Server-generated IDs do not help the caller correlate a
response with its own investigation plan.

### 9.3 Failure model

**Recommended:** reject structurally invalid batches/shared scopes globally;
represent valid-item failures locally.

Fail-fast execution would make one ambiguous symbol discard unrelated useful
answers. Treating every defect as item-local would be unsafe when the shared
scope itself is unauthorized or contradictory.

### 9.4 Snapshot claim

**Recommended:** one post-preparation WorkspaceIndex read snapshot for all
index-backed items, with file-local authority disclosed separately.

Sequentially running the existing handler seven times would be easier but
would not guarantee consistency and would repeat hydration orchestration.
Holding an index guard across IO would create unnecessary lock contention and
risk lock-order problems.

### 9.5 Parallelism

**Recommended:** deterministic sequential preparation in the first version.

Parallel execution is deferred. The primary objective is fewer external calls
and deduplicated work, not maximum internal concurrency. Parallelism would need
separate resource limits, cancellation behavior, deterministic diagnostics,
and lock analysis.

### 9.6 Separate tool versus additive form

**Recommended:** add a batch form to `workspace_query`.

A separate `workspace_query_batch` tool would duplicate tool discovery,
schemas, documentation, and client-selection surface. The semantics belong to
the same query capability, while mutually exclusive input forms keep parsing
unambiguous.

---

## 10. Architectural approval checkpoint

**Decision:** Approved by the maintainer on 2026-09-28. The approval includes
the additive heterogeneous batch form, shared top-level scope, unique item IDs,
ordered per-item outcomes and failure isolation, one post-preparation index
snapshot, request-local hydration deduplication, sequential execution, the
32-item reject-not-truncate maximum, and unchanged legacy single-query behavior.

Implementation must not begin until the maintainer explicitly approves these
cross-cutting decisions:

1. `workspace_query` gains an additive `queries` request form while preserving
   the legacy `type` form;
2. batch scope is declared once at the top level and cannot be overridden per
   item;
3. every item requires a unique caller-supplied string ID;
4. structurally valid batches use ordered per-item success/error outcomes;
   heterogeneous query types are supported and one item failure never aborts
   independent siblings;
5. shared-scope and batch-structure errors reject the whole request;
6. index-backed items use one final post-preparation WorkspaceIndex snapshot;
7. equivalent request-local hydration work is deduplicated without adding a
   new persistent cache;
8. the initial implementation is sequential and introduces no new concurrency;
9. the initial maximum is 32 items, rejected rather than truncated when
   exceeded;
10. existing single-query request and response behavior remains compatible.

Approval authorizes the phased migration described here, including the public
schema addition, internal evaluation refactor, shared preparation boundary,
tracked regressions, documentation, live acceptance, architectural audit, and
final verification. It does not authorize unrelated WorkspaceIndex identity,
query semantics, discovery-provider, fidelity, caching, concurrency, or source
mutation changes.

---

## 11. Explicit non-goals

This migration does not change query semantics, Model C or edge occurrence
identity, CBM's discovery-only authority, trusted scope, persistence, source
mutation, or existing single calls. It does not batch arbitrary MCP tools,
create item dependencies or a query language, introduce concurrency,
background work, cancellation, streaming, pagination, silent truncation, or a
generic orchestration framework.

---

## 12. Completion definition

The migration is complete only when:

- both public request forms are accurately declared and production-reachable;
- the legacy single path uses the same internal evaluator as batch execution;
- all seven operations work in batch without weakening their established
  authorities;
- batch preparation is deduplicated and index-backed answers share one final
  snapshot;
- per-item results/errors remain ordered, correlated, and unmerged;
- unchanged RED regressions are GREEN;
- equivalent legacy single-call payloads remain compatible;
- applicable live client acceptance is complete;
- durable invariant/tooling documentation reflects the implementation;
- the architectural audit has no critical or high-severity gap;
- the complete Final Verification Gate is GREEN.
