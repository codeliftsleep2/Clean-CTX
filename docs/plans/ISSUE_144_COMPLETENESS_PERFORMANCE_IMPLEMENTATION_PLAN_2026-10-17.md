# Agent MaxHeadRoom — Issue #144 Completeness Performance Implementation Plan

**Date:** 2026-10-17

**Phase:** approved architecture translated to an incremental implementation plan

**Production implementation:** separately gated; not performed by this document

**Approved investigation:**
`docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_PERFORMANCE_INVESTIGATION_2026-10-17.md`

## 1. Approved objective

Replace redundant whole-workspace C# recompilation with a provider-neutral,
single-parse semantic artifact lifecycle that:

- compiles only owners whose required evidence is absent, stale, or inadequate;
- defers exact C# Calls reconciliation until an exact-call consumer requires it;
- resolves exact calls from retained source-bound artifacts without reparsing;
- replaces all-authority-file dependency lists with a capability-scoped
  authority fingerprint;
- parallelizes pure compilation/projection work through one bounded Rayon pool;
- commits semantic authority deterministically and atomically; and
- preserves Issue #144 exactness, fail-closed behavior, and every current exact
  C# call correctness invariant.

The fixed field criterion remains:

> Every single or batched reverse-edge request in the Issue #144 release-binary
> Live MCP gate completes in at most 10 seconds.

This is an acceptance criterion only. It is not a timeout, cancellation policy,
candidate cap, partial-result permission, or completeness downgrade.

## 2. Fixed architectural decisions

The approved implementation uses these boundaries:

1. **One parse, one artifact.** A current C# source snapshot is parsed once to
   produce canonical instructions, local declarations, invocation evidence,
   non-call semantic edges, hashes, fidelity, and producer generations.
2. **Provisional parallel work.** Worker threads create immutable artifacts;
   they do not mutate aliases, indexes, persistence, caches of authority, or
   publication clocks.
3. **Deterministic coordinator commit.** Results are validated and committed in
   canonical-owner order after generation checks.
4. **Capability-gated exact Calls.** C# publication dirties exact-call
   authority. The central exact-call preparation boundary reconciles before any
   exact-call consumer reads results.
5. **Capability-scoped dependency identity.** Exact-call currentness binds to
   one authority fingerprint. Generic TypeRef evidence does not inherit global
   exact-call dependencies.
6. **One bounded Rayon pool.** Crossbeam remains the MCP dispatcher and
   backpressure layer. No per-request pool and no one-thread-per-file model.
7. **Separate green baselines.** Every phase ends with its own focused GREEN
   result and baseline commit before the next phase begins.

## 3. Concurrency policy

### 3.1 Pool ownership

Add one `rayon::ThreadPool` owned by `McpState` through a small compiler-executor
component. It is constructed once with the session and shared by compilation
batches.

Default worker count:

```text
max(1, available_parallelism - 1)
```

The one-core reservation leaves capacity for MCP dispatch, source I/O, and
coordinator commit. Tests inject pool sizes 1, 2, and 4. Do not add a public
configuration option in this implementation unless measured evidence proves
the default inadequate and a separate decision approves the surface.

### 3.2 Nested work

Inventory reads already use Rayon. Avoid nesting compilation pool work beneath
another compilation-pool job. The executor exposes one batch-map entry point;
individual compilation functions remain synchronous and pure.

The dedicated pool limits compilation concurrency even when several Crossbeam
dispatcher workers submit concurrent requests. The pool performs work stealing
across all submitted jobs without multiplying OS threads per request.

### 3.3 Lock discipline

Workers may read immutable configuration and provided `Arc<String>` snapshots.
They may not acquire shared write locks. The coordinator performs:

```text
snapshot -> parallel compile -> ordered validation -> ordered publication
         -> optional artifact-backed projection -> one bounded index commit
```

No parsing occurs while holding `workspace_index`, `ir_context`, dictionary,
source-cache, persistence, derived-catalog, or publication-clock locks.

## 4. Proposed source layout

Several current files are already near the 615-line active ceiling:

- `src/mcp/tool_helpers.rs` — 607 lines;
- `src/mcp/state.rs` — 611 lines;
- `src/mcp/tool_handlers/query/edges.rs` — 608 lines; and
- `src/workspace/index.rs` — 610 lines.

Do not add material logic to those files. Keep them as registration/delegation
surfaces and place new behavior in bounded modules:

```text
src/mcp/semantic_compilation.rs
src/mcp/semantic_compilation/
  artifact.rs
  executor.rs
  compile.rs

src/mcp/csharp_calls.rs
src/mcp/csharp_calls/
  authority.rs
  lifecycle.rs
  projection.rs
  commit.rs

src/workspace/index/capability_coverage.rs

src/tests/mcp/
  semantic_compilation_artifact.rs
  semantic_compilation_parallel.rs
  csharp_call_authority_lifecycle.rs
  csharp_call_artifact_reconciliation.rs
  issue_144_performance_work_counts.rs
```

Exact splitting may combine small modules, but every new or materially modified
code file must remain at or below 615 lines, preferably below 600. Do not move
new behavior into another near-limit coordinator.

## 5. Core types and symbols

### 5.1 `semantic_compilation/artifact.rs`

Add a typed return boundary:

```rust
pub(crate) struct SemanticCompilationArtifact
pub(crate) struct CSharpSemanticArtifact
pub(crate) struct ArtifactIdentity
pub(crate) struct ProducerGenerationFingerprint
```

`SemanticCompilationArtifact` owns:

- canonical owner;
- exact source snapshot `Arc<String>` and content hash;
- fidelity and producer-generation fingerprint;
- compiled canonical instructions;
- semantic edges excluding globally resolved exact Calls;
- semantic dependencies unrelated to exact Calls; and
- optional language-specific artifact data.

`CSharpSemanticArtifact` owns:

- local declaration authority;
- `InvocationEvidenceSet` bound to the same source identity/hash; and
- whether structured invocation evidence exists.

No field contains mutable session authority or a provider identity.

Required functions:

```rust
compile_source_artifact(snapshot, request) -> Result<SemanticCompilationArtifact, CompileError>
artifact_is_compatible(artifact, current_generation, evidence_requirement) -> bool
artifact_fingerprint(artifact) -> ArtifactFingerprint
```

### 5.2 `semantic_compilation/executor.rs`

Add:

```rust
pub(crate) struct SemanticCompilerExecutor
pub(crate) struct CompilationJob
pub(crate) struct CompilationBatch
pub(crate) struct CompilationBatchResult
```

Required behavior:

- construct one bounded Rayon pool;
- accept a canonically sorted job vector;
- compile jobs through indexed parallel iteration;
- return results in input order regardless of completion order;
- catch no semantic errors as success;
- expose cfg(test) worker-count injection and structural counters; and
- avoid nested pool creation.

The executor owns scheduling only. It does not select candidates, publish
artifacts, or decide completeness.

### 5.3 `csharp_calls/authority.rs`

Add:

```rust
pub(crate) struct CSharpAuthorityManifest
pub(crate) struct CSharpAuthorityFingerprint
pub(crate) struct CSharpAuthorityGeneration
pub(crate) struct CSharpDeclarationCatalog
```

The fingerprint is derived deterministically from sorted tuples of canonical
owner, source hash, producer/schema generation, and scope/admission policy
fingerprint. Provider state and query target are absent.

Required functions:

```rust
authority_manifest(artifacts, scope_policy) -> CSharpAuthorityManifest
merge_declarations(artifacts) -> CSharpDeclarationCatalog
authority_fingerprint(manifest) -> CSharpAuthorityFingerprint
```

### 5.4 `csharp_calls/lifecycle.rs`

Add session-owned exact-call lifecycle state:

```rust
pub(crate) struct CSharpCallAuthorityState
pub(crate) enum CSharpCallAuthorityStatus { Current, Dirty, Reconciling }
pub(crate) struct CSharpCallPreparation
```

State binds:

- current source/admission generation;
- last committed authority fingerprint;
- artifact identities used for that fingerprint; and
- exact-call projection generation.

Required functions:

```rust
mark_csharp_authority_dirty(owner, generation)
prepare_exact_csharp_calls(scope, evidence_requirement)
is_exact_call_authority_current(scope, generation) -> bool
invalidate_csharp_artifact(owner)
invalidate_csharp_root(root)
```

Concurrent preparations for the same authority generation must coalesce or
serialize without duplicating compilation. A newer generation makes an older
preparation ineligible to commit.

### 5.5 `csharp_calls/projection.rs`

Move/adapt the current exact-call projection so it consumes artifacts:

```rust
project_exact_calls(artifact, declaration_catalog) -> ExactCallProjection
project_exact_calls_parallel(artifacts, catalog, executor) -> Vec<ExactCallProjection>
```

It uses the artifact's exact source, canonical instructions, and validated
invocation evidence. It does not invoke `IRCompiler`, reread a source file, or
mutate `WorkspaceIndex`.

### 5.6 `csharp_calls/commit.rs`

Add one generation-validated commit boundary:

```rust
commit_exact_call_generation(state, preparation, projections)
```

Commit requirements:

- source and authority generations still match;
- all required artifacts/projections succeeded;
- canonical owner order is deterministic;
- a newer publication cannot be overwritten;
- exact-call edges and capability coverage advance atomically; and
- failure leaves the previous complete generation intact but marked dirty for
  exact-call consumers, which must fail closed rather than read it as current.

## 6. Existing symbols to migrate

### `src/mcp/tool_helpers.rs`

- Extract compilation setup into `semantic_compilation/compile.rs`.
- Replace the tuple alias `CompiledSemantic` with the typed artifact boundary.
- Preserve compatibility wrappers temporarily for existing callers.
- Capture `IRCompiler::invocation_evidence` instead of dropping it.
- Separate pure source compilation from alias/version/session mutation.
- Remove wrappers only after every caller migrates.

### `src/mcp/tool_handlers/hydration/publication.rs`

- Keep eligibility and evidence-adequacy selection.
- Replace per-owner `compile_candidate_for` with job preparation plus ordered
  artifact publication.
- Return per-owner Current/Published/Failed outcomes and artifact identities.
- Mark C# call authority dirty after successful relevant publication.
- Do not invoke exact-call reconciliation here.

### `src/mcp/tool_handlers/hydration/execution.rs`

- Build the K selected compilation jobs once.
- Submit one batch to `SemanticCompilerExecutor`.
- Commit results deterministically.
- Remove `csharp_reconciliation_required` and eager
  `reconcile_csharp_exact_calls` from reverse TypeRef acquisition.
- Preserve proof commit only after all required publication succeeds.

### `src/mcp/csharp_call_reconciliation.rs`

- First delegate to the new lifecycle/preparation boundary.
- Preserve the current API while callers migrate.
- Remove the two compilation loops after artifact-backed projection is green.
- Retain only compatibility delegation or delete the file when no callers
  remain.

### `src/mcp/state.rs`

- Register the compiler executor, artifact store, and C# call authority state
  through extracted submodules.
- Keep state construction concise; do not add lifecycle logic inline.
- Ensure artifact/lifecycle state participates in edit, refresh, delete,
  restore, and reset invalidation.

### `src/mcp/tool_handlers/query/prepare.rs`

- Add a request-local capability preparation memo keyed by effective scope,
  C# authority generation, and required exact-call capability.
- Exact call preparation occurs once per compatible batch generation.

### `src/mcp/tool_handlers/query/edges.rs`

- Before evaluating an exact C# `Calls` request, require current exact-call
  authority through the preparation context.
- Reverse builtin TypeRef never requests that capability.
- Keep public exact/lower-bound semantics unchanged.

### `src/workspace/index/capability_coverage.rs`

- Track exact-call authority fingerprint/generation separately from generic
  per-file semantic coverage.
- Provide atomic replacement/removal operations for exact-call edges without
  overwriting unrelated current TypeRef/declaration evidence.
- A dirty exact-call generation is never reported current.

### `src/layers/meta/semantic.rs`

- Do not force authority fingerprints into path-based `SemanticDependency`.
- Keep file dependencies for genuinely file-local external inputs.
- Model capability generation at the index coverage boundary, avoiding an enum
  expansion that would make unrelated semantic layers understand C# authority.

## 7. Invalidation map

| Event | Artifact action | Exact-call authority action | TypeRef acquisition proof |
| --- | --- | --- | --- |
| C# content edit | Invalidate owner artifact | Dirty generation | Invalidate affected root/unit |
| C# create/delete/rename | Update owner inventory | Dirty generation | Invalidate affected root/unit |
| External refresh | Invalidate changed/root artifacts conservatively | Dirty generation | Existing root invalidation |
| Exclusion/root policy change | Invalidate scoped artifact manifest | Dirty/new fingerprint | Invalidate scope proof |
| Producer/schema change | Reject incompatible artifacts | Dirty/new fingerprint | Existing producer mismatch |
| Non-C# change | Language-specific only | No C# dirtiness unless admission mapping changes | Existing unit behavior |
| Failed compilation | No artifact commit | Remains dirty | No completeness commit |
| Superseded generation | Discard provisional result | Newer generation controls | No stale proof commit |

Deletion must retract exact-call projections owned by the deleted file and
dirty callers whose resolution may change. Restoration/rename can reestablish
authority only after the new generation is prepared.

## 8. Incremental RED/GREEN phases

### Phase 1 — structural work-count RED

Add `src/tests/mcp/issue_144_performance_work_counts.rs` and test-only counters
for full parses, artifact reuse, exact-call preparation, projection, and source
dependency validation.

RED assertions:

```text
reverse TypeRef full parses == K
reverse TypeRef exact-call reconciliations == 0
warm unchanged full parses == 0
dependency validations <= cN
```

The unfixed implementation must fail because it performs `K + 2N` parses and
eager reconciliation. Stash only the RED test/registration before production
work. End Phase 1 only after the exact unchanged regression is GREEN, then
commit its baseline.

Suggested user-run filter:

```powershell
cargo test --all-features issue_144_performance_work_counts -- --nocapture
```

### Phase 2 — typed artifact boundary

- Introduce artifacts and pure source compilation.
- Preserve sequential execution and eager reconciliation temporarily.
- Add artifact equivalence tests for every enabled language, with C# asserting
  declarations and invocation evidence survive.
- Prove failed compilation produces no reusable artifact.

Focused gate:

```powershell
cargo test --all-features semantic_compilation_artifact -- --nocapture
```

Commit a separate Phase 2 green baseline.

### Phase 3 — bounded parallel artifact compilation

- Add the one owned Rayon pool.
- Batch only K inadequate owners.
- Preserve deterministic ordered commit and current public behavior.
- Test pool sizes 1, 2, and 4; output and publication order must match.
- Inject one failed and one generation-superseded job; neither may partially
  commit or earn completeness.

Focused gate:

```powershell
cargo test --all-features semantic_compilation_parallel -- --nocapture
```

Commit a separate Phase 3 green baseline.

### Phase 4 — capability-gated exact-call lifecycle

- Add dirty/current generation state and request-local preparation memo.
- Route every exact C# Calls consumer through the central gate.
- Remove eager reconciliation from reverse TypeRef and unrelated acquisition.
- Keep the old reconciliation implementation behind the gate during this
  phase so authority timing changes without changing exact-call semantics.

Regression matrix:

- TypeRef publication performs no reconciliation;
- first Calls consumer reconciles once;
- second compatible consumer reuses current authority;
- batch Calls items reconcile once;
- concurrent same-generation consumers do not duplicate reconciliation;
- dirty/failed preparation returns no falsely current exact result.

Focused gate:

```powershell
cargo test --all-features csharp_call_authority_lifecycle -- --nocapture
```

Commit a separate Phase 4 green baseline.

### Phase 5 — artifact-backed exact-call projection

- Merge declaration authority from current artifacts.
- Project exact calls without `IRCompiler` or source rereads.
- Remove the two whole-workspace compilation loops.
- Preserve all existing file-order, ambiguity, deletion, signature-change,
  property-initializer, arity, spread, static/instance, and additional-root
  behavior.

Focused gate:

```powershell
cargo test --all-features csharp_call_artifact_reconciliation -- --nocapture
```

Commit a separate Phase 5 green baseline.

### Phase 6 — authority fingerprint and linear validation

- Add capability-scoped fingerprint coverage.
- Remove all-authority-file dependency lists from exact-call projections.
- Prove one manifest construction per generation and O(1) warm currentness.
- Confirm exact-call dirtiness never makes TypeRef evidence inadequate.

Focused gate:

```powershell
cargo test --all-features csharp_call_authority_fingerprint -- --nocapture
```

Commit a separate Phase 6 green baseline.

### Phase 7 — migration removal and integration audit

- Remove compatibility tuple/wrappers and old eager reconciliation authority.
- Trace producer → artifact → lifecycle owner → invalidation → exact-call gate
  → index commit → public query response.
- Update architectural invariants and Issue #144 documentation.
- Confirm no alternate exact-call consumer bypasses preparation.
- Run focused combined gates, then the authoritative repository gate.

Commit a separate Phase 7 green baseline.

### Phase 8 — release Live MCP acceptance

After the complete local gate, rebuild the all-feature release binary and run
the consolidated Issue #144 harness. Every reverse-edge request must be at or
below 10 seconds and every semantic case must pass. Store `timings.json`,
verdicts, requests, responses, stderr, and binary hashes under `target/**`.

This is operator/live evidence, not a replacement for tracked Rust tests.

## 9. Required correctness matrix

Tracked tests must cover:

1. K-only reverse TypeRef compilation across N admitted owners;
2. unchanged warm zero-compilation behavior;
3. deterministic results for pool sizes 1/2/4;
4. worker failure produces no partial authority;
5. source changes during compilation reject stale artifacts;
6. concurrent newer publication wins;
7. file-order-independent exact cross-file calls;
8. new ambiguity revokes an exact call;
9. ambiguity removal reestablishes the call;
10. deletion, rename, restore, and signature change invalidate correctly;
11. exact-zero Calls and TypeRef results remain truthful;
12. additional roots and exclusions participate in the fingerprint;
13. unsupported/missing artifacts fail closed;
14. batch order and single/batch parity;
15. property initializers and ordinary method calls retain exact evidence;
16. arity, spread, static/instance, duplicate-name, and overload controls;
17. dirty Calls authority cannot contaminate current TypeRef adequacy;
18. provider enabled/unavailable does not change semantic truth;
19. artifact and authority generations invalidate on producer/schema changes;
20. no old eager reconciliation path remains reachable.

## 10. Work-count acceptance

For N admitted owners, K inadequate owners, C call-bearing artifacts, and E
relevant declaration/call evidence:

```text
reverse TypeRef inventory owners examined <= cN
reverse TypeRef full semantic parses == K
reverse TypeRef exact-call projections == 0
unchanged warm full semantic parses == 0

dirty exact Calls artifact validation <= cN
dirty exact Calls missing/stale parses == K_missing_artifacts
dirty exact Calls projection work <= c(C + E)
unchanged exact Calls parses/projections == 0
```

Constants must be pinned by deterministic counters, not wall-clock tests.
Logging and metrics may observe these values but never participate in semantic
authority.

## 11. Failure and cancellation behavior

- One failed required compilation fails the batch's completeness/reconciliation
  commit; successful provisional artifacts may be cached only if their source
  identity remains independently valid, but they do not make the generation
  current.
- A source/admission generation change discards the stale batch at commit.
- Panics in worker work must be converted at the executor boundary into a
  failed batch without poisoning semantic authority.
- Cancellation support is not introduced by this plan. If a future task adds
  cancellation, incomplete work must remain non-authoritative.
- No deadline or ten-second runtime branch is added to production code.

## 12. Observability

Use test-only structural counters for regression authority. Production tracing
may record phase durations and counts only if existing tracing facilities can
carry them without a new public contract:

```text
inventory owners
selected K
artifacts reused/compiled/rejected
authority manifest owners/declarations
call-bearing artifacts projected
compile/project/commit elapsed time
```

Do not emit one log line per file in the common path. The live evidence showed
that verbose per-read diagnostics can itself distort debug observations.

## 13. Migration and removal map

| Current construct | Replacement | Removal condition |
| --- | --- | --- |
| `CompiledSemantic` tuple | `SemanticCompilationArtifact` | Every compiler caller uses typed output |
| Per-candidate sequential compilation loop | Executor batch + ordered commit | Parallel determinism/failure tests green |
| `csharp_reconciliation_required` in hydration | Dirty authority generation | Every Calls consumer uses preparation gate |
| Eager `reconcile_csharp_exact_calls` | Capability-gated preparation | Lifecycle matrix green |
| First full reconciliation compile pass | Artifact declaration merge | Artifact equivalence green |
| Second full reconciliation compile pass | Artifact exact-call projection | Existing C# reconciliation tests green |
| All-authority-file dependencies | Authority fingerprint | Linear currentness tests green |
| Full projection replacement for Calls | Capability-scoped exact-call commit | Non-call evidence preservation tests green |

Do not retain old and new reconciliation authority paths after migration.

## 14. Stop gates

Stop for explicit approval if implementation requires:

- a public configuration option for compiler concurrency;
- a different worker-count policy than `max(1, P - 1)`;
- durable persistence of compilation artifacts;
- changes to public edge identity or exact/lower-bound semantics;
- cancellation, timeouts, budgets, or caps;
- provider-specific semantic authority;
- exact-call behavior changes beyond preserving current semantics; or
- parallel mutation of shared semantic authority from worker threads.

## 15. Verification handoff

Agents do not run Cargo. For every phase, provide the exact focused command,
wait for the user's RED/GREEN result, and commit only the corresponding green
baseline. After Phase 7, hand off the single authoritative Final Verification
Gate from `docs/agent/verification.md` without abbreviating it.

Only after local completion, the user runs:

```powershell
cargo build --release --all-features
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-Issue144Acceptance.ps1 `
  -BinaryPath ./target/release/clean-ctx.exe `
  -ProviderPath "$env:LOCALAPPDATA/Programs/codebase-memory-mcp/codebase-memory-mcp.exe"
```

The release live run fails if any reverse-edge request exceeds 10 seconds even
when its result is semantically correct.

## 16. Files inspected and changed

### Inspected

- approved Issue #144 investigation, design, implementation, and performance
  artifacts;
- `Cargo.toml` / `Cargo.lock` concurrency dependencies;
- MCP dispatcher, state locks, publication clocks, hydration execution,
  publication, filesystem inventory, compiler helpers, C# exact-call
  reconciliation/projection, workspace coverage, and current regression files;
- Issue #144 saved live timing/response/stderr evidence; and
- architecture and incremental-migration policy.

### Changed

- Added only this implementation plan.
- No production, test, configuration, generated, or verification file changed.

## 17. Final recommendation

**APPROVE THIS FILE/SYMBOL/TEST-LEVEL PERFORMANCE PLAN FOR SEPARATE PRODUCTION
IMPLEMENTATION AUTHORIZATION.**

The first production phase must establish deterministic RED work-count evidence.
No implementation phase may trade semantic quality for latency, and no phase
may proceed without its own focused GREEN baseline commit.

