# Query-boundary completeness and redundant-work proposal

**Status:** Phase E1 implemented and locally verified; `has_cycle` bounded
witness contract implemented; wider audit remains open
**Recorded:** 2026-09-26
**Production behavior:** `entities_in_file` now performs fidelity-aware,
freshness-checked semantic compilation on first touch
**Primary surfaces:** `workspace_query`, `graph_trace`/CBM `trace_path`,
`cbm_proxy`, and `provide_code_context.focusMethods`

## 1. Decision summary

`entities_in_file` now auto-compiles its explicit file when the
WorkspaceIndex has no sufficiently complete semantic projection for that file.
Its optional `fidelity` argument should mean **compile semantic facts at least
as completely as the requested fidelity requires**, including recompilation
when an existing index entry was produced at a lower semantic fidelity.

`has_cycle` did not adopt a superficially similar single-file shortcut. It is
a workspace-scoped graph property and currently has no target file. Compiling
one file before querying a partially populated graph can still return a clean
`false` while relevant files remain unindexed. Its completeness policy therefore
required a separate architectural decision. That process is implemented through
Phase 4 in `HAS_CYCLE_WITNESS_PROPOSAL.md`; this document remains the
authority for the fidelity-aware `entities_in_file` work.

Phase E1 and the separate `has_cycle` witness contract were explicitly
authorized and implemented through tracked RED/GREEN regressions. The wider
audit remains investigation work; neither completed item authorizes its other
contracts.

This document also records four adjacent query-boundary candidates discovered by
auditing for the same broader failure shape: a caller must perform a discovery
call that the queried operation could potentially perform itself, or a request
is accepted while its answer does not clearly disclose missing resolution or
ignored input. Current-source review closed the project-resolution candidate as
already fixed; the other three have not all been freshly reproduced through the
live MCP/CBM boundary and remain investigation items until the live gates in
section 11 confirm them.

## 2. Problem

Before Phase E1, `entities_in_file` resolved and authorized `file_path`, then
queried only the existing WorkspaceIndex. An uncompiled file was therefore
indistinguishable from a genuinely empty file:

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

The production handler in `src/mcp/tool_handlers/query/entities.rs` now:

1. validates `file_path` through `resolve_file_path_checked`;
2. enforces `workspaceRoot`, configured roots, and optional `withinPath`;
3. canonicalizes the resolved path; and
4. validates and normalizes the optional semantic fidelity;
5. hashes current source and checks WorkspaceIndex-owned coverage;
6. compiles a read-only candidate when coverage is absent, stale, or
   insufficient;
7. atomically replaces facts plus coverage, including empty projections; and
8. calls `WorkspaceIndex::entities_in_file`.

The operation uses no name-based discovery diagnostic. A successful empty
response now means the authorized current file was semantically compiled and
produced no entities at the requested semantic fidelity.

### 3.2 `has_cycle` — resolved through the dedicated witness process

The production handler in `src/mcp/tool_handlers/query/graph.rs` resolves an
effective workspace scope and calls the index-owned deterministic dependency
witness primitive. It accepts optional/default `kind=dependency`, admits only
`Injects` and `ImportsModule`, and performs no discovery, coverage hydration, or
source compilation. Its response preserves the compatible boolean while adding
the ordered witness, asserting-file provenance, semantic-identity ambiguity,
and explicit `indexed_evidence_only` / `source_complete=false` coverage.

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

## 6. `has_cycle` decision — resolved as bounded Option B

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

### Implemented decision

`entities_in_file` remains independent. `has_cycle` implements Option B: a
bounded deterministic query over scoped retained index evidence, with one
actionable witness and an explicit non-source-complete coverage statement.
Exhaustive compilation remains deferred as a separately named and approved
operation if a concrete host later requires it.

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

### Phase E1 — `entities_in_file` — complete locally

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

The initial one-call/schema/validation tests and the reuse/cross-publication
tests were each observed RED, stashed, restored unchanged, and observed GREEN.
The complete eight-test E1 module is green. Existing WSC-004 scope suites remain
the authority for path/root/`withinPath` security. Live no-prior-provide use is
the remaining field gate.

### Phase C1 — `has_cycle` — complete locally through dedicated phases 0–4

Option B was approved and implemented under
`HAS_CYCLE_WITNESS_PROPOSAL.md`. Tracked tests prove relation policy,
deterministic closed witnesses, provenance, identity-collision disclosure,
scope isolation, honest empty results, and absence of hydration/compilation
side effects. The remaining gates are the optional live stdio scenario and the
final repository audit/verification gate.

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

## 10. Wider query-boundary audit

The following candidates are ordered by potential correctness impact. Source
inspection establishes the current code shape, but it does not substitute for
live reproduction across the external tool boundary. No item in this section
is scheduled for implementation solely because it appears here.

### 10.1 `trace_path` / `graph_trace`: duplicate bare names silently select one identity

**Status:** Implemented with tracked RED/GREEN coverage and verified through
the live MCP stdio boundary on 2026-09-27.

The two public surfaces have different request shapes:

- CBM `trace_path` accepts `function_name`, `direction`, optional `depth`, and
  a project; and
- the structured Clean-CTX `graph_trace` wrapper accepts `from` and `to`.

The bridge forwards the caller's `from`/function name to CBM and filters
returned qualified endpoints against `to`. That is not itself a defect: CBM
accepts an unambiguous bare name, and Clean-CTX's endpoint matcher deliberately
matches a bare final segment against the qualified wire identity.

Tracked live tests in `src/tests/cbm/trace_wire.rs` already establish that:

- an unambiguous bare source and target return the correct outbound edge;
- the same bare pair is found through the inbound fallback when appropriate;
- a genuinely unrelated pair returns a successful empty result; and
- CBM's `function not found` soft-error envelope becomes an explicit tool
  error before edge parsing, never a plausible empty result.

The old claim that bare names generally require `search_graph` first is
therefore stale. Adding a compulsory pre-search would impose a hidden second
CBM operation on a path that already works and would duplicate CBM's own
resolution behavior.

The tracked investigation harness at
`verification/cbm/scripts/Investigate-DuplicateTraceNames.ps1` indexed two Rust
functions named `duplicate_probe`, one calling `alpha_leaf` and the other
calling `beta_leaf`. `graph_search` returned two distinct canonical identities.
The observed behavior was deterministic and silently selective:

- structured `graph_trace` with bare `duplicate_probe` found the Alpha edge;
- the same bare source to the Beta target returned zero edges and no error;
- each canonical source found its own edge;
- crossed canonical pairs correctly returned zero edges; and
- CBM-native `trace_path` through `cbm_proxy` likewise returned only
  `alpha_leaf` for the bare name, while both canonical calls returned their
  respective callees.

This proves the ambiguity originates at the installed CBM boundary and is
passed through by Clean-CTX. It is not a speculative endpoint-matching concern.
The recommended corrective contract is:

1. accept either a canonical identity or a bare name;
2. preserve canonical identities without an extra search;
3. resolve a bare name within the explicitly selected project;
4. trace automatically when exactly one candidate matches;
5. return a structured disambiguation error listing bounded candidate
   identities when multiple candidates match; and
6. preserve the existing explicit not-found failure when no candidate matches.

The implemented shared resolver preserves canonical inputs as a no-search fast
path. Bare inputs use exact-name matches from a project-scoped search, collapse
repeated occurrences of one canonical identity, and return deterministic
canonical candidates rather than selecting by result order. The proxy's
project-scoped search does not mutate the bridge's active project. The contract
is protected by `src/tests/cbm/trace_identity_resolution.rs`; the tracked CBM
field harness asserted the same behavior through a freshly built stdio server,
with evidence recorded at
`target/cbm-duplicate-trace-verification/20260927-195506.json`.

### 10.2 `cbm_proxy`: project-name resolution parity — resolved

**Status:** Already fixed on this branch by `4a86ee43` (`Fix CBM project
identity: resolve canonical path slugs for all graph calls`). No new production
work is proposed.

The current source does not support the premise that `cbm_proxy` still requires
an exact slug while wrappers accept a repository basename. Both paths reach
`GraphBridge::resolve_project_id`:

- `cbm_proxy::resolve_proxy_target_project` resolves a project supplied in
  native parameters, top-level arguments, or `workspaceRoot`; and
- structured wrappers call `GraphBridge::set_project`, whose lifecycle path
  resolves the supplied project identity.

`resolve_project_id` accepts a known slug, a path/root, or a configured root's
directory basename before falling back to the literal value for authoritative
CBM rejection. The reported stricter-proxy behavior describes the pre-fix
boundary, not the current source architecture.

Do not schedule a second resolver or retain this as an open verification
candidate. A future live parity failure would be a new regression against the
shared-resolution contract and should enter the normal RED/GREEN process with
fresh evidence.

### 10.3 `focusMethods` outside Edit fidelity — implemented

**Potential severity:** Medium — accepted input is silently ineffective and
can make the returned context look more specifically targeted than it is.

**Decision (approved 2026-09-27):** use a hybrid compatibility contract. A
non-empty focus implies Edit only when both `fidelity` and `intent` are omitted.
An explicit non-Edit fidelity or intent is contradictory and returns `-32602`
rather than being overridden. An empty focus retains its existing Edit-only
meaning and is rejected when Edit was not explicitly selected.

**Implementation status:** Complete under the tracked RED/GREEN contract in
`src/tests/mcp/focus_fidelity_contract.rs`. Validation occurs before file IO or
session mutation. The public tool description and operational guidance expose
the same precedence rules.

Current source confirms the boundary. `provide_code_context` parses
`focusMethods` at every fidelity, but passes it into body selection only when
the heuristically effective fidelity is `Edit`. The public schema describes
the field as optional with Edit; it does not reject the incompatible
combination. The operational guide currently warns callers that non-Edit use
is silently ignored.

This preserves explicit caller choices while removing the common redundant
retry: a caller that supplies an actual target without otherwise choosing a
mode receives focused Edit output in one call. Empty focus is not inferred
because its specialized “Edit structure with no bodies” meaning is not evident
without an explicit Edit request.

### 10.4 Name-only workspace edge and traversal queries — implemented

**Potential severity:** Medium — primarily redundant discovery work, with a
correctness risk if a caller guesses domain/type or mistakes a required-field
error for absence.

**Decision (approved 2026-09-27):** preserve the fully specified identity fast
path and allow either or both classification fields to be omitted. Supplied
fields are exact filters. After the existing scoped hydration, physical
occurrences are grouped by semantic identity `(domain, entity_type, name)`:
zero identities is an explicit not-found error, one runs the requested query,
and multiple identities return `-32602` with distinct candidates. Repeated
physical occurrences of the same identity are not ambiguity.

**Implementation status:** Complete under
`src/mcp/tool_handlers/query/identity.rs` and the unchanged tracked contract in
`src/tests/mcp/workspace_query_identity_resolution.rs`. The shared resolver
performs the existing hydration exactly once, applies workspace/`withinPath`
scope before identity selection, and returns the selected identity in
`resolved_identity`. Fully specified requests retain their existing direct
identity query plus hydration path.

**Live status:** Verified through a freshly built server over MCP stdio. The
field run established unique bare-name equivalence with fully qualified
`forward_edges`, unique `reverse_edges`, resolved transitive traversal,
non-ambiguous repeated physical occurrences, explicit multi-identity
disambiguation, partial-filter resolution, explicit not-found behavior, and
`withinPath` isolation. The live Angular fixture also confirmed why ambiguity
must remain explicit: one source declaration may legitimately project as
multiple semantic identities across the builtin and framework domains.

Current handlers for `forward_edges`, `reverse_edges`, and
`transitive_dependencies` require the complete Model-C identity
`(domain, entity_type, name)` before hydration begins. Missing `domain` or
`entity_type` already produces a precise `-32602` error. `find_entities`, by
contrast, can search by name across domains and types. The documented two-call
flow is therefore real at the code-contract level when only a name is known.

If live usage demonstrates that the two-call pattern is common enough to
justify a contract expansion, allow `domain` and `entity_type` to be omitted
together while retaining the exact-identity fast path:

1. perform the existing scoped name discovery/hydration;
2. collect matching identities within the effective workspace/`withinPath`;
3. run the requested query automatically for exactly one identity;
4. return a structured disambiguation error with bounded candidates for more
   than one identity; and
5. distinguish no match, incomplete discovery coverage, and a real empty edge
   or dependency result.

Supplying only one of `domain` or `entity_type` acts as an explicit filter and
must never be ignored. Resolution must reuse
`find_entities`/hydration semantics rather than introduce a second discovery
algorithm, and it must retain the existing scoped occurrence/provenance rules.

## 11. Live-verification gate for the wider audit

Before implementation planning, capture each case against a real indexed
repository and the actual MCP stdio path:

| Candidate | Required live comparison | Decision evidence |
| --- | --- | --- |
| Duplicate bare-name trace | **Complete:** two canonical Rust functions compared through wrapper and proxy | both bare surfaces silently selected Alpha; both canonical traces were correct |
| Non-Edit focus | omitted focus vs supplied focus at Low/Medium/High and Edit | effective fidelity, content kind, body selection, warning/error |
| Name-only workspace query | **Complete:** live stdio comparison of bare, exact, partial, ambiguous, missing, repeated-occurrence, and narrowed requests | unique result equality, explicit candidates, and scoped occurrence isolation verified |

The live record must distinguish a tool error, explicit incomplete coverage,
an empty but complete answer, and a successful-looking incomplete or wrongly
resolved answer. Only the last category establishes a silent-correctness defect.

Recommended scheduling after verification:

1. non-Edit `focusMethods` validation;
2. duplicate-name trace resolution only if silent guessing/merging reproduces;
3. name-only workspace-query resolution if real workflows show recurring
   two-call overhead.

## 12. RED/GREEN and approval boundaries

Every reproducible behavior defect must use the tracked RED/GREEN procedure.
Each surface needs its own unchanged RED test because a proxy fix does not prove
the structured wrapper, and a handler-unit test does not prove MCP schema or
stdio reachability.

These changes are externally observable tool-contract decisions. Live
verification authorizes investigation only. Before implementation, approve the
specific contract for each confirmed item, including ambiguity, not-found,
partial-coverage, and compatibility behavior. In particular:

- automatic trace-name resolution adds a hidden discovery operation;
- focus auto-promotion changes fidelity and is not presently recommended;
- name-only workspace queries alter which arguments are required; and
- exhaustive `has_cycle` compilation remains a separate workspace-completeness
  decision, not an instance of name resolution.
