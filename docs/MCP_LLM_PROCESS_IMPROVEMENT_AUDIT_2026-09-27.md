# MCP-to-LLM process improvement audit — 2026-09-27

**Status:** Active improvement record; Phase 0 complete with focused
RED→GREEN verification
**Scope:** MCP tool discovery, LLM-facing instructions, tool-selection workflow,
request schemas, response projection, CBM project state, and live release-candidate
validation
**Operational authority:** `docs/agent/tooling.md`
**Machine-readable authority:** `src/mcp/tools.rs` and `src/cbm/tools.rs`
**Architectural authority:** `docs/ARCHITECTURAL_INVARIANTS.md`

## Why this audit exists

Clean-CTX has already removed or isolated many historically wasteful paths:

- `provide_code_context` is the primary model-facing file-context operation;
- graph discovery and semantic compilation have distinct authority boundaries;
- workspace facts are retrieved on demand instead of repeated in file context;
- focused Edit fidelity limits byte-exact source to the requested method bodies;
- successful `apply_edit` calls use lazy per-project graph freshness rather than
  synchronous reindexing;
- explicit delta transport is separate from ordinary complete-context reads;
- expected workspace-discovery diagnostics are omitted from normal responses;
- canonical `content`, `structuredContent`, and `_meta` roles are documented and
  tested; and
- the dispatcher permits independent non-conflicting work without an outer
  global state lock.

The remaining opportunities are primarily at the MCP-to-LLM boundary. The
server has strong internal contracts, but some of the instructions and schemas
present overlapping, contradictory, or overly broad choices to the model. That
can produce unnecessary probes, malformed first attempts, corrective calls, or
avoidable prompt and response tokens even when the underlying production path
is efficient.

This document records those opportunities before live release-candidate
testing. It does not itself change public contracts or approve the architectural
decisions identified below.

## Executive assessment

No additional duplicate production compilation pipeline or obviously wasteful
mandatory process was found that should block live release-candidate testing.
The strongest remaining improvements are:

1. deliver concise global tool-use instructions automatically;
2. reconcile schema and documentation contradictions that can cause failed
   calls;
3. stop requiring a routine CBM status probe before a useful graph call;
4. add standard MCP tool annotations;
5. strengthen polymorphic request schemas;
6. reduce irrelevant repeated tool-catalog metadata; and
7. use live host evidence to decide whether catalog tiering, call-scoped CBM
   project selection, or response de-duplication should become architectural
   changes.

Items 1–6 can be designed as compatible release-candidate hardening. Item 7
contains externally observable decisions and remains behind the repository's
Architectural Approval Gate.

## Implementation phases

### Phase 0 — Contract reconciliation

**Status:** Complete; focused RED→GREEN verification confirmed

Phase 0 is the compatibility-safe RC-2 cleanup. It aligns registered schema and
operational wording without changing runtime behavior, accepted inputs, response
shapes, defaults, or state ownership:

- Edit guidance names `apply_edit`, not `replace_in_file`;
- path-bearing schemas strongly recommend explicit `workspaceRoot` while
  preserving the CWD fallback for backward compatibility;
- unsupported `.js` input is removed from the `provide_code_context` guide;
- `provide_code_context` is described as returning complete current context,
  not automatic delta transport; and
- CBM selection consistently prefers typed wrappers for typed results and
  `cbm_proxy` for compact or explicitly fresh raw CBM output.

The tracked regression is
`schema_guidance_uses_current_model_workflow_terms` in
`src/tests/mcp/tools.rs`. The unchanged regression failed against the old schema
guidance and passed after the Phase 0 implementation.

### Phase 1 — Automatic workflow instructions

**Status:** Complete; focused RED→GREEN verification confirmed

Phase 1 is RC-1: add concise server instructions to initialization, retain the
detailed optional prompts, and add initialization contract coverage. This is
separate from Phase 0 because it changes what compliant hosts may inject into
the model context even though it does not change tool execution semantics.

The implementation keeps one compact workflow authority in
`src/mcp/prompts.rs`, returns it as `initialize.result.instructions`, and leaves
the detailed SCHEMA-vNext notation in the optional prompt surfaces. The tracked
regression is `initialize_includes_compact_workflow_instructions` in
`src/tests/mcp/prompts.rs`. The unchanged regression failed when initialization
omitted server instructions and passed after the Phase 1 implementation.

### Phase 2 — Optimistic graph-first discovery

**Status:** Complete; focused RED→GREEN verification confirmed

Phase 2 is RC-3: graph operations consult live CBM health directly, so ordinary
discovery starts with the useful graph call rather than a separate
`get_cbm_status` preflight. An unavailable first call prescribes the direct
`search_codebase` then `provide_code_context` fallback. Indexing responses state
that the graph is temporarily unavailable rather than empty, permit one bounded
retry, and then prescribe fallback instead of status polling.

`get_cbm_status` remains available for setup diagnostics, recovery checks, and
explicit indexing-progress inspection. No health gate, indexing state, graph
authority, or empty-result semantic changed. The tracked regression is
`graph_search_returns_is_error_when_cbm_unavailable` in
`src/tests/cbm/handler_unavailable.rs`. The unchanged regression failed when
the first unavailable graph call gave setup advice without a direct workflow
fallback and passed after the Phase 2 implementation.

### Phase 3 — Standard MCP tool annotations

**Status:** Complete; focused RED→GREEN verification confirmed

Phase 3 is RC-4: every public tool now receives an explicit standard MCP
classification for read-only, destructive, idempotent, and open-world effects.
The classification is centralized and exhaustive in
`src/mcp/tool_annotations.rs`; adding a public tool without an effect decision
fails instead of silently accepting protocol defaults.

The hints describe externally relevant effects. Internal caches, baselines,
and session projections do not make source/query operations externally
mutating. Exact source/delta mutations, semantic deletion/purge, graph
reindexing, and the generic proxy retain conservative classifications. All
tools remain inside the configured local workspace/provider boundary and use
`openWorldHint: false`. Annotations are client guidance only; existing
authorization, trusted-root, ownership, and transaction checks remain the
enforcement boundary.

The tracked regression is
`every_registered_tool_has_conservative_standard_annotations` in
`src/tests/mcp/tools.rs`. The unchanged regression failed on the first
unannotated public tool and passed after the exhaustive Phase 3 classification.

### Phase 4 — Polymorphic request schemas

**Status:** Complete; focused RED→GREEN verification confirmed

Phase 4 is RC-5: the consolidated `apply_edit` and `workspace_query` surfaces
now expose discriminated JSON Schema `oneOf` branches instead of relying on
descriptions to convey operation-specific requirements. `apply_edit` declares
the four supported structural operations and their exact required fields.
`workspace_query` declares all seven variants, including the compatibility-safe
root fallback for `entities_in_file` and the existing explicit-root requirement
for `calls_in_file`.

The public tool names, handler validation, shared optional properties, trusted
path boundaries, and runtime semantics are unchanged. Schema construction lives
in `src/mcp/tool_schemas.rs` so the catalog remains readable without introducing
a new public abstraction.

The tracked regression is
`polymorphic_tool_schemas_encode_operation_specific_requirements` in
`src/tests/mcp/tools.rs`. The unchanged regression failed because
`apply_edit.operations.items` had no `oneOf` and passed after the discriminated
operation schemas were installed.

### Phase 5 — Relevant language metadata only

**Status:** Complete; focused RED→GREEN verification confirmed

Phase 5 is RC-6: `supportedLanguages` is now attached only to the seven public
tools whose correct use depends on enabled source-language parsers. Persistence,
history, administration, statistics, IR-delta application, and CBM tools no
longer repeat irrelevant parser metadata.

This changes catalog metadata only. Tool names, schemas, handlers, parser
feature detection, and runtime behavior are unchanged. The tracked regression
is `supported_languages_is_limited_to_source_processing_tools` in
`src/tests/mcp/tools.rs`; it failed first on `apply_delta`, which consumes an
already-formed IR delta rather than source input, and passed after language
metadata was restricted to the source-processing boundary.

As part of finalizing this phase, `src/mcp/tools.rs` was decomposed at its
catalog/dispatch boundary: schemas and catalog assembly remain in `tools.rs`,
while argument parsing, registry ownership, and `tools/call` routing now live
in `src/mcp/tool_dispatch.rs`. Compatibility re-exports preserve established
crate paths; no public MCP surface or dispatch semantics changed. Focused
verification passed for both the tool-list/registry correspondence and the
source-processing language-metadata boundary.

AD-1 through AD-3 remain
separate approval-gated architectural decisions rather than implementation
phases.

## Current strengths to preserve

### One primary model-facing file read

`provide_code_context` owns automatic intent/fidelity selection and returns
complete current context. `compress_code_context` remains a lower-level direct
compiler surface, while `delta_code_context` and `apply_delta` form an explicit
code-side acknowledgement protocol. Any improvement must preserve that
separation and must not reintroduce automatic delta ambiguity into normal reads.

### Discovery is not semantic authority

CBM locates candidate symbols and files. Clean-CTX compilation and
`WorkspaceIndex` own semantic facts. Filesystem fallback may locate candidates
when CBM is degraded or unavailable, but it does not convert CBM absence into an
authoritative empty semantic result.

### Lazy graph freshness after owned edits

`apply_edit` marks the affected CBM project dirty and returns without invoking
CBM. The next graph operation coalesces pending edits into one synchronous fast
refresh. The model normally does not need to call `index_repository` after a
Clean-CTX-owned edit.

### Sparse exceptional diagnostics

Normal successful workspace discovery omits diagnostic metadata. Only deviation
from the expected provider, coverage, readiness, or candidate path is projected.
This is the correct response-economics direction and should remain the pattern
for new diagnostics.

### Explicit response authorities

Human/model-readable data belongs in `content`, structured machine data belongs
in `structuredContent`, and application-facing lifecycle or telemetry data
belongs in `_meta`. Improvements must preserve the MCP-001 response-envelope
invariant.

## Release-candidate hardening opportunities

### RC-1 — Deliver concise server instructions during initialization

#### Finding

The notation and exact-source guide exists as an MCP prompt in
`src/mcp/prompts.rs`, but prompts are optional client/user-selected templates.
The current `initialize` result in `src/mcp/handlers.rs` returns protocol
version, capabilities, and server information without server instructions.

Consequently, an LLM may receive the tool catalog without the compact workflow
rules that prevent redundant probes and incorrect tool selection. A host is not
required to fetch `cleanctx-notation` or `clean-ctx-vocabulary` before exposing
the tools.

#### Recommended compatible change

Add a concise `instructions` string to the `2025-11-25` initialization result.
It should teach decisions, not repeat the complete SCHEMA-vNext vocabulary:

- use `provide_code_context` as the default supported-source read;
- always pass `workspaceRoot` when the caller knows it;
- use `graph_search` for typed discovery;
- use `cbm_proxy` when compact or explicitly fresh raw CBM output is preferable;
- reserve delta and persistence tools for workflows that intentionally own
  their lifecycle;
- do not manually reindex after `apply_edit`;
- treat indexing responses as temporary rather than empty results; and
- use native reads for unsupported/non-code files or exact known ranges.

Essential safety and prerequisite information must remain in each relevant tool
schema because host support for server-instruction injection is not universal.
The existing prompts should remain available for detailed notation help and
interactive dashboard workflows.

#### Expected effect

- fewer wrong-tool first calls;
- fewer raw-source reads when structured context is sufficient;
- fewer manual reindex calls;
- less reliance on repository-specific host prompt configuration; and
- no handler or data-contract change.

### RC-2 — Reconcile LLM-facing contract drift

Several current statements can independently lead to failed or corrective
calls.

#### `workspaceRoot`

`docs/agent/tooling.md` and `docs/CLAUDE_INTEGRATION_RULES.md` say to always pass
`workspaceRoot`, while most path-bearing MCP schemas describe it as optional and
defaulted to the current working directory.

Recommended resolution:

- for this release candidate, describe it consistently as strongly recommended
  for reliable path and multi-workspace resolution;
- retain current fallback behavior for compatibility; and
- consider making it required only as an explicitly approved future contract
  change.

#### JavaScript support

`docs/CLAUDE_INTEGRATION_RULES.md` lists `.js` among supported
`provide_code_context` inputs, while the production language registry rejects
`.js` and `src/tests/compression/language.rs` explicitly pins that rejection.

Recommended resolution: remove `.js` from the supported-source list. Continue
to use CBM or native file tools for JavaScript until a real language layer is
implemented and enabled.

#### Automatic delta wording

The tooling antipattern section says `provide_code_context` supplies
"auto-delta transport," while the current contract says every successful call
returns complete current context and delta transport is explicit.

Recommended resolution: replace the stale wording with "intent/fidelity
heuristics and complete current context."

#### Editing tool terminology

The `compress_code_context` fidelity and `provide_code_context` intent schema
descriptions refer to safe `replace_in_file` use. The registered Clean-CTX
mutation surface is `apply_edit`.

Recommended resolution: name `apply_edit` directly and describe Edit fidelity
as supplying exact tracked method bodies for supported structural operations.
Host-native editing remains appropriate for unsupported edit shapes.

#### CBM entry-point priority

Current guidance alternately says:

- use `graph_search` first;
- `cbm_proxy` is preferred for token efficiency; and
- `cbm_proxy` is the primary CBM integration point.

Recommended single rule:

> Use a structured wrapper when typed identity, nodes, edges, paths, or modules
> matter. Use `cbm_proxy` when compact or explicitly fresh raw CBM output is the
> better result shape. Use `graph_search` as the normal symbol/file discovery
> entry point.

This reconciles selection without removing either surface or changing their
different caching and project-state semantics.

### RC-3 — Make CBM status diagnostic rather than a routine preflight

#### Finding

Current Claude-facing guidance requires `get_cbm_status` once per task before
repository discovery. The graph handlers already consult the bridge's live
status and refuse unavailable/degraded execution rather than returning a false
empty result.

The preflight therefore adds one tool call to every healthy task without
eliminating an otherwise necessary production check.

#### Recommended compatible workflow change

- optimistically call the appropriate graph operation first;
- if it reports CBM unavailable or degraded, immediately use the documented
  fallback;
- reserve `get_cbm_status` for setup diagnostics, recovery checks, and indexing
  progress; and
- make graph failure content name the fallback action clearly enough that the
  model does not enter a status/retry loop.

#### Expected call economics

| CBM state | Current workflow | Proposed workflow |
|-----------|------------------|-------------------|
| Available | status + graph query | graph query |
| Unavailable/degraded | status + fallback | graph attempt + fallback |
| Indexing | status/query polling is possible | one temporary response, then bounded retry or fallback |

The healthy path loses one call. The unavailable path does not add a call. Live
testing must confirm that target hosts expose graph failure content clearly
enough for deterministic fallback.

### RC-4 — Add standard MCP tool annotations

#### Finding

The public tool definitions do not currently declare standard MCP annotations
such as `readOnlyHint`, `destructiveHint`, `idempotentHint`, or
`openWorldHint`. Clients must therefore use conservative defaults even for
local, read-only queries.

#### Recommended compatible change

Add annotations from a reviewed per-tool classification table. At minimum:

- graph queries, context reads, diffs, history, statistics, and
  `workspace_query` should advertise read-only behavior;
- closed local-file/session/database operations should advertise
  `openWorldHint: false` where accurate;
- `apply_edit`, `apply_delta`, `save_context`, `delete_context`,
  `purge_old_deltas`, and `index_repository` must receive explicit mutation and
  idempotence classifications; and
- destructive hints must reflect durable semantic-state deletion separately
  from source-file mutation.

Annotations are client hints, not security enforcement. Existing trusted-root,
ownership, structural edit, and transaction checks remain authoritative.

#### Expected effect

- fewer unnecessary confirmation boundaries in clients that honor annotations;
- clearer safe-retry behavior;
- better separation of query and mutation surfaces; and
- no response-contract change.

### RC-5 — Strengthen polymorphic input schemas

#### `apply_edit.operations`

The array item schema currently accepts an arbitrary object. The actual handler
supports four discriminated operation shapes:

- `replace_body`;
- `delete`;
- `insert_after`; and
- `insert_before`.

Represent these as a JSON Schema `oneOf` keyed by the operation `type`, with
operation-specific required fields. This moves malformed-call rejection into
tool construction rather than spending a handler round trip on correction.

#### `workspace_query`

The consolidated tool has seven operations but only `type` is universally
required in the input schema. Each operation has different required and
permitted fields.

Keep the consolidated public tool—it reduces catalog breadth—but describe its
operations as conditional `oneOf` branches keyed by `type`. Preserve the common
scope fields where applicable and encode the special requirements for
`calls_in_file`, `entities_in_file`, and name-bearing relationship queries.

Before implementation, verify the target clients' JSON Schema support. If a
target host handles `oneOf` poorly, prefer tighter descriptions plus a simpler
operation-specific schema representation rather than splitting the tool solely
for schema expressiveness.

### RC-6 — Stop repeating irrelevant language metadata

`inject_supported_languages()` adds the same `supportedLanguages` field to all
25 public tool definitions, including project listing, persistence maintenance,
statistics, and legacy inspection tools that do not compile source languages.

Recommended resolution:

- attach language support only to source-processing tools; or
- expose it once as server-level instructions/metadata while keeping concise
  extension information on tools whose correct use depends on it.

This is a small prompt-catalog saving, but it is deterministic and does not
require a handler change.

## Architectural decision points

The following opportunities have global, externally observable, or
cross-cutting effects. They require explicit approval before implementation.

### AD-1 — Default model-facing catalog versus advanced/admin catalog

#### Decision required

Whether `tools/list` should expose the same 25-tool catalog to every model, or
whether Clean-CTX should support a smaller default model-facing profile plus an
advanced/admin profile.

#### Why the decision matters

The current surface combines:

- primary context and edit tools;
- low-level compression and explicit delta protocol tools;
- persistence and history administration;
- diagnostics and legacy inspection;
- typed CBM wrappers; and
- the generic compact CBM proxy.

This breadth increases catalog tokens and selection ambiguity. Hiding or
profiling tools, however, changes public discovery behavior and can affect
existing clients and automation.

#### Reasonable alternatives

1. **Keep one catalog.** Improve instructions, descriptions, schemas, and
   annotations only. Lowest compatibility risk; retains maximum catalog cost.
2. **Add an opt-in compact profile.** Preserve the current default and let
   operators request a reduced model-facing catalog. Compatible but does not
   improve default behavior.
3. **Make the compact profile the future default.** Expose primary context,
   edit, diff, workspace, and selected graph tools by default; retain all tools
   in an advanced/admin profile. Best selection economics, but a versioned
   public-contract change.

#### Recommendation

Implement RC-1 through RC-6 first, measure live selection behavior, then decide
whether an opt-in compact profile provides material additional value. Do not
remove or hide tools before that evidence exists.

### AD-2 — Sticky versus call-scoped structured-wrapper project state

#### Decision required

Whether `graph_search`, `graph_query`, `graph_trace`, and `get_architecture`
should continue changing the bridge's active project when a `project` or
`workspaceRoot` argument is supplied.

#### Current boundary

- structured wrappers mutate shared active-project state;
- subsequent wrapper calls that omit a project inherit it;
- the graph bridge mutex serializes those mutations and queries; and
- `cbm_proxy` resolves its project for one call without changing the bridge's
  active project.

This is race-safe under the current mutex, but it is order-dependent and creates
a semantic difference between two otherwise overlapping CBM surfaces.

#### Reasonable alternatives

1. **Preserve sticky state.** Require explicit project selection in
   multi-project workflows and retain compatibility.
2. **Make all public queries call-scoped.** Resolve the target per request and
   leave no cross-call project side effect. Easier for parallel or independent
   LLM calls, but changes documented behavior and cache/state ownership.
3. **Add explicit session/project handles.** Makes state visible but introduces
   a new protocol and abstraction that is not currently justified.

#### Recommendation

Prefer call-scoped public query semantics if live multi-repository testing finds
project inheritance surprising or error-prone. Otherwise preserve the current
contract for this release candidate and require explicit project/root arguments
in multi-project use.

### AD-3 — Portable `content` completeness versus response de-duplication

#### Decision required

Whether model-readable `content` should remain a complete portable projection
when the same semantic payload is also present in `structuredContent`.

#### Current boundary

`workspace_query` renders a pretty-printed answer envelope in `content` while
also returning authoritative `structuredContent`. `graph_search` similarly
lists each matching node in `content` and returns the nodes structurally.

This protects clients that forward only `content`. A host that exposes both
representations to the model may pay for substantially duplicated answer data.

#### Reasonable alternatives

1. **Keep complete portable content.** Maximum client compatibility; possible
   token duplication.
2. **Make content a compact summary.** Let `structuredContent` carry detail.
   Best economy when the host exposes structured data; incomplete for clients
   that expose only content.
3. **Host/profile-sensitive rendering.** Potentially optimal but introduces
   capability negotiation and divergent result semantics.

#### Recommendation

Do not change this boundary before the target-client live test identifies which
fields actually enter the LLM context. Preserve correctness and portability
until duplication is measured rather than inferred from wire presence.

## Live release-candidate validation matrix

Live harness output is operator evidence, not a tracked test and not part of
the repository verification gate. Any reproducible defect discovered here must
receive a tracked regression under `src/tests/**` before its fix is complete.

| Scenario | Expected efficient behavior | Evidence to capture |
|----------|-----------------------------|---------------------|
| Locate and understand one known symbol | One graph discovery call, then one `provide_code_context` call with explicit root | Tool sequence, arguments, returned path, context representation |
| Locate symbol with healthy CBM after RC-3 | No routine `get_cbm_status` preflight | Full tool transcript and call count |
| CBM unavailable | One failed graph attempt followed by documented fallback; no repeated status loop | Failure content, fallback choice, retries |
| CBM indexing | Temporary state is not treated as empty; retry is bounded | Elapsed/indexing metadata and subsequent action |
| Focused method edit | Discovery as needed, focused Edit context, one `apply_edit` batch | Whether raw whole-file reading or manual reindexing occurs |
| External edit followed by graph use | Explicit fast `index_repository` only when freshness is required | Reason reindex was selected and next query result |
| Clean-CTX-owned edit followed by graph use | No manual reindex; next graph call performs lazy refresh | Tool sequence and freshness evidence |
| Large `workspace_query` result | Correct answer with measured content/structured duplication | Exact host-visible fields and token contribution |
| Large `graph_search` result | Typed paths remain usable without double-paying for every node | Exact host-visible fields and token contribution |
| Two repositories in one conversation | Explicit project/root targeting; no unexpected inheritance | Ordered calls and selected project for each result |
| Prompt/instruction visibility | Model can state the primary tool-selection rules without user selecting a prompt | Host transcript after initialization |
| Malformed edit attempt | Schema prevents or clearly corrects the malformed operation once | Tool argument generated, error/correction count |
| Malformed workspace operation | Conditional requirements prevent or clearly correct missing fields once | Tool argument generated, error/correction count |

## Success criteria

The MCP-to-LLM workflow is ready for release-candidate acceptance when live
evidence shows:

- ordinary supported-source understanding defaults to
  `provide_code_context`;
- healthy CBM discovery does not require a separate status call;
- unavailable/degraded CBM produces one deterministic fallback transition;
- the model does not call `index_repository` after `apply_edit`;
- paths are authoritative and paired with explicit workspace roots;
- delta and persistence tools are not selected accidentally for ordinary file
  understanding;
- malformed polymorphic calls do not create repeated correction loops;
- multi-project results come from the intended project;
- target-host handling of `content` and `structuredContent` is known; and
- any token duplication retained is an explicit portability tradeoff.

## Suggested implementation sequence

1. Reconcile the documentation and schema wording in RC-2.
2. Add concise initialization instructions and contract tests for RC-1.
3. Update the CBM workflow and failure guidance for RC-3.
4. Add a reviewed annotation table and schema contract tests for RC-4.
5. Strengthen operation schemas with client-compatibility coverage for RC-5.
6. Narrow repeated language metadata for RC-6.
7. Run the live validation matrix.
8. Record reproducible discoveries in `docs/agent/DISCOVERY_REGISTRY.md` and add
   tracked regressions where applicable.
9. Bring AD-1, AD-2, or AD-3 forward for explicit approval only when live
   evidence shows material remaining cost or ambiguity.
10. Run the authoritative Final Verification Gate from
    `docs/agent/verification.md` after implementation; do not duplicate its
    command list here.

## Validation boundary for this document

This audit was produced through targeted, read-only inspection of the registered
tool schemas, initialization and prompt handlers, dispatch boundaries, CBM
project selection, response rendering, operational guidance, tests, and
architectural invariants.

Creating this document changes no runtime behavior, tool schema, public
contract, configuration default, or architectural invariant. No Cargo build,
test, check, Clippy, formatter, server, or live harness was run as part of the
documentation-only write-up. Repository-wide verification remains governed by
`docs/agent/verification.md`.
