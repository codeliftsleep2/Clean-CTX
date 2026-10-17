# Agent MaxHeadRoom — Issue #144 Completeness Performance Investigation

**Date:** 2026-10-17

**Phase:** static causal and algorithm design investigation

**Production implementation:** not authorized by this document

**Predecessor:**
`docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_IMPLEMENTATION_PLAN_2026-10-17.md`

## 1. Executive verdict

The observed 79–88 second cold/reacquisition latency is not an acceptable
implementation of the approved completeness design. Release optimization may
reduce the number, but it cannot repair the structural inefficiency.

The primary defect is **redundant whole-workspace C# compilation coupled to
eager exact-call reconciliation**:

1. hydration compiles each selected candidate sequentially;
2. any published C# candidate triggers `reconcile_csharp_exact_calls` once;
3. reconciliation compiles every admitted indexed C# owner at High fidelity to
   reconstruct declaration authority;
4. reconciliation compiles every owner again with that authority to regenerate
   semantic projections; and
5. exact-call dependency validation can read/hash every authority file for
   each caller, creating an O(N²) currentness surface.

For K missing/stale candidates among N admitted C# owners, the current cold work
is approximately:

```text
K full compilations + N full compilations + N full compilations
+ up to O(N²) authority-dependency validation
```

The Issue #144 fixture had roughly 48 compiled candidates. Saved field evidence
showed cold and reacquisition calls between 79 and 88 seconds, an unchanged warm
call at 13 ms, and 4,369 `read_source` trace entries across the complete live
run. The live run used a debug binary, so its wall time is not a production
benchmark; the redundant work is nevertheless established directly by code.

**Recommendation: replace eager recompile-based reconciliation with a
single-parse, artifact-backed, capability-gated reconciliation pipeline, then
parallelize its pure stages with one bounded Rayon pool.**

Crossbeam is usable but is not the right primitive for the inner compilation
batch. It already owns MCP request dispatch and backpressure. A second
Crossbeam worker system would duplicate scheduling. Rayon is already a direct
dependency and already performs parallel inventory reads.

## 2. Fixed performance acceptance

The implementation must satisfy both structural and field criteria:

> The common proof path is O(N) or better over admitted source-owner metadata,
> and expensive semantic compilation is proportional to owners whose required
> evidence is absent, stale, or inadequate.

Additionally:

> Every single or batched reverse-edge request in the Issue #144 production
> field gate completes in at most 10 seconds.

The ten-second requirement is an acceptance gate, not a timeout. Exceeding it
fails acceptance. It does not authorize cancellation, truncation, caps,
best-effort exactness, or lower-bound substitution.

## 3. Current production path and costs

`hydrate_workspace_index_for_target` currently:

```text
discover + build admitted inventory
  -> select candidates lacking adequate current evidence
  -> for each selected owner, sequentially:
       compile_file_ir_focused
       publish full semantic projection
  -> if any owner was C#:
       reconcile_csharp_exact_calls
```

`reconcile_csharp_exact_calls` currently:

```text
collect every admitted C# file from WorkspaceIndex
  -> pass 1, sequential: compile every file at High fidelity
       -> extract declaration authority from compiled instructions
  -> pass 2, sequential: compile every file again with full authority
       -> regenerate every semantic projection
       -> attach all authority files as dependencies when Calls exists
  -> sequentially reread/hash every file
  -> replace all projections under one WorkspaceIndex write lock
```

This implementation was introduced to make cross-file exact calls independent
of file publication order. The semantic requirement is valid. Recompiling the
workspace twice is not intrinsic to that requirement.

## 4. Why naive parallelism is insufficient

Parallelizing the existing three compilation passes would reduce elapsed time
on a many-core machine, but it would preserve:

- up to `K + 2N` full parses;
- the O(N²) dependency-currentness surface;
- unnecessary exact-call work on a reverse TypeRef request;
- large transient allocations;
- duplicated parsing of identical source generations; and
- contention around source cache, alias dictionary, publication clocks, and
  index commit.

It would also make latency hardware-dependent and risk oversubscription because
the dispatcher already owns approximately one worker per available processor.
Parallelism is a second-stage accelerator, not the causal repair.

## 5. Recommended algorithm

### 5.1 Immutable C# semantic compilation artifact

One C# parse should produce an immutable provisional artifact containing:

```text
canonical owner
source Arc<String> and source hash
producer/schema/fidelity generation
compiled canonical instructions
non-exact-call semantic edges and dependencies
local declaration authority
structured invocation evidence
```

The compiler already possesses all of these values. `CompiledSemantic` drops
the compiler's `InvocationEvidenceSet` at the helper boundary, forcing later
reconstruction. Extend that boundary with a typed artifact instead of reparsing.

The artifact is provisional: creating it changes no shared authority.

### 5.2 Compile only inadequate owners

For the acquisition requirement, compute the exact K inadequate owners first.
Read stable source snapshots and create publication tickets in deterministic
canonical-owner order. Compile those K snapshots in parallel outside all
shared write locks.

Compilation workers must not:

- allocate session aliases;
- mutate `WorkspaceIndex`;
- update persistence;
- hold publication clocks;
- mutate request-local catalogs; or
- reconcile exact calls.

The coordinator validates each artifact's source hash and commits successful
artifacts deterministically. Failed or superseded artifacts prevent earned
completeness exactly as they do today.

### 5.3 Make exact-call reconciliation capability-gated

Publishing C# evidence advances a C# declaration-authority generation and
marks exact-call resolution dirty. It does not eagerly reconcile exact calls
for a reverse TypeRef query.

Before a consumer requests the exact C# `Calls` capability, one centralized
preparation gate ensures that exact-call resolution is current. No query may
read exact-call edges without passing that gate. Reverse TypeRef, declaration,
and unrelated relationship consumers do not pay exact-call reconstruction.

This is deferred computation, not weakened correctness. The semantic
capability becomes authoritative before consumption rather than after every
unrelated C# publication.

### 5.4 Reconcile from artifacts without reparsing

When the exact-call capability is required:

1. snapshot the admitted C# owner set and authority generation;
2. reuse current artifacts for unchanged owners;
3. compile only missing/stale artifacts, in parallel;
4. deterministically merge local declarations into one authority catalog;
5. project exact calls directly from each artifact's instructions, source, and
   invocation evidence, in parallel;
6. validate the source/authority generation has not advanced; and
7. atomically commit exact-call projections in canonical-owner order.

No source is parsed a second time merely to resolve calls.

### 5.5 Replace all-file dependencies with one authority fingerprint

The current caller dependency list includes every authority file. That makes
one owner's currentness O(N) and all owners O(N²), even when nothing changed.

Create a deterministic C# authority fingerprint from sorted tuples of:

```text
canonical owner
source hash
producer/schema generation
admission/scope policy fingerprint
```

An exact-call projection binds to this fingerprint. Currentness becomes an
O(1) generation/fingerprint comparison after the manifest is constructed once
per source generation. File-specific source hashes still protect each owner.

This dependency is capability-scoped. A stale exact-call authority generation
must not make an otherwise current TypeRef projection inadequate. The index
must therefore track exact-call resolution state separately from generic
per-file semantic evidence rather than placing global call dependencies on the
entire file projection.

## 6. Parallel execution recommendation

### 6.1 Use Rayon, not another Crossbeam pool

Recommended primitive: a single bounded Rayon compilation pool owned by the
session or shared compiler service.

Reasons:

- Rayon is already a direct dependency and used in hydration inventory reads.
- Compilation jobs are CPU-bound, independent, finite map operations.
- Indexed parallel iterators preserve deterministic collection ordering when
  fed a sorted owner vector.
- Work stealing handles uneven source-file sizes better than static channels.
- Crossbeam remains responsible for request dispatch and queue backpressure.

### 6.2 Bound total compilation concurrency

Do not create a pool per request. Do not spawn one OS thread per file. Do not
let each dispatcher worker independently saturate another unbounded pool.

Use one pool with a fixed process/session concurrency bound derived once from
available parallelism. The exact default is an externally observable global
concurrency decision and must be approved during implementation planning.
Recommended starting policy:

```text
compiler workers = max(1, available_parallelism - 1)
```

with one core retained for dispatch, I/O, and commit. Tests must inject a small
fixed pool size rather than depend on host CPU count. No configuration surface
is needed in the first slice unless measurements prove it necessary.

### 6.3 Locking model

```text
coordinator: snapshot source identities/generation
workers: read immutable source Arcs; compile pure artifacts
coordinator: validate hashes/generation
coordinator: acquire ordered per-owner publication authority
coordinator: one short WorkspaceIndex commit
```

Never hold `workspace_index`, alias dictionary, source-cache, persistence, or
publication-clock locks across parsing. Never perform a nested parallel batch
while already on the compiler pool. Commit ordering is canonical-owner order,
not worker completion order.

## 7. Complexity target

Let:

- N = admitted C# owners;
- K = owners with absent/stale/inadequate required semantic evidence;
- C = owners containing invocation evidence requiring exact-call projection;
- E = total relevant declarations/calls.

Then:

```text
reverse TypeRef acquisition:
  metadata/inventory O(N)
  parsing O(K)
  exact-call reconciliation O(0)

exact Calls preparation after authority change:
  artifact validation O(N)
  parsing O(K_missing_artifacts)
  authority merge O(N + declarations)
  call projection O(C + E)
  commit O(C)

unchanged warm request:
  O(1) proof/authority-generation checks plus result lookup
```

No supported path may perform O(N²) file hashing or `2N` reparsing.

## 8. Correctness invariants

The optimized pipeline must preserve:

1. file-order-independent cross-file exact calls;
2. ambiguity introduction revokes formerly exact calls;
3. deletion and signature changes revoke affected calls;
4. removal of ambiguity can reestablish exact calls;
5. source changes during compilation cannot commit stale evidence;
6. failed artifact compilation cannot earn completeness;
7. publication order cannot make older work overwrite newer work;
8. exact-call consumers never observe a dirty authority generation as current;
9. TypeRef completeness remains independent of exact-call reconciliation; and
10. cold, warm, provider-enabled, and provider-unavailable semantic truth stays
    identical for equivalent source state.

## 9. Required deterministic regressions

Before production work, add tracked RED tests under `src/tests/**` that count
work rather than time:

1. reverse TypeRef cold hydration with N C# owners and K inadequate owners:
   semantic parses equal K, not `K + 2N`;
2. reverse TypeRef publication does not execute exact-call reconciliation;
3. first exact Calls query after publication reconciles once;
4. unchanged second Calls query performs zero parses and zero reconciliation;
5. reconciliation reuses artifacts and parses only stale/missing owners;
6. declaration merge and call projection results are deterministic across pool
   sizes 1, 2, and 4;
7. compilation failure/cancellation commits no partial authority;
8. generation change during parallel compilation discards stale artifacts;
9. ambiguity, removal, deletion, and signature-change tests retain current
   semantic outcomes; and
10. dependency validation is linear and does not read every authority source
    once per caller.

Wall-clock assertions do not belong in Rust tests. The tracked work-count tests
protect algorithmic complexity; the release live harness enforces the ten-second
production boundary.

## 10. Incremental implementation sequence

### Phase 1 — RED structural work counts

Add compilation/artifact/reconciliation counters under cfg(test), plus the
production-path reverse TypeRef fixture. Establish current RED evidence.

### Phase 2 — typed provisional artifacts

Return source-bound compilation artifacts without changing publication
behavior. Preserve all existing semantics and establish artifact equivalence.

### Phase 3 — one-parse batch compilation

Compile K inadequate owners through the bounded Rayon pool; validate and commit
deterministically. Keep eager reconciliation temporarily so this phase changes
cost, not authority timing.

### Phase 4 — capability-gated exact-call lifecycle

Add the dirty/current authority generation and central exact-call preparation
gate. Remove eager reconciliation from unrelated acquisition.

### Phase 5 — artifact-backed exact-call projection

Merge declaration authority and project exact calls from retained artifacts.
Remove the two whole-workspace recompilation passes.

### Phase 6 — authority fingerprint dependencies

Replace all-authority-file dependencies with the capability-scoped fingerprint.
Prove linear currentness work and lifecycle invalidation.

### Phase 7 — final verification

Run focused gates per phase, the complete repository gate once, then the
release Issue #144 live harness with the immutable ten-second ceiling.

Each phase receives its own green baseline commit.

## 11. Alternatives considered

### Parallelize current reconciliation only

Rejected as the target. It accelerates redundant `K + 2N` parsing and leaves
O(N²) validation.

### Crossbeam compilation workers

Not recommended. Crossbeam is appropriate for dispatcher queues and
backpressure already present in the process. Rayon better matches a bounded
CPU map/reduce workload and avoids a second hand-built scheduling system.

### Skip all C# reconciliation for TypeRef permanently

Rejected. Unrelated acquisition need not reconcile immediately, but an exact
Calls consumer must always establish current authority first.

### Persist every artifact immediately

Deferred. Session-local artifacts are sufficient to establish lifecycle and
performance. Durable artifact schema/versioning is a separate decision.

### Add a timeout or candidate cap

Rejected. It would trade latency for incorrect or lower-bound results and
violate the approved completeness architecture.

## 12. Architectural decisions requiring approval

Implementation requires approval for these cross-cutting decisions:

1. exact C# Calls authority becomes capability-gated and lazily reconciled at
   its consumption boundary rather than eagerly after unrelated publication;
2. C# compilation artifacts become session-owned semantic preparation state;
3. exact-call dependency currentness binds to an authority fingerprint rather
   than a list of every authority source file; and
4. one bounded Rayon compiler pool becomes the process/session CPU-concurrency
   authority for semantic compilation.

These affect C# publication, query preparation, semantic dependency validity,
shared concurrency, and Calls consumers. They do not change public result
semantics: exact consumers still receive exact current evidence or fail closed.

## 13. Final recommendation

**APPROVE SINGLE-PARSE ARTIFACT-BACKED RECONCILIATION WITH CAPABILITY-GATED
EXACT CALLS AND BOUNDED RAYON PARALLELISM FOR AN INCREMENTAL IMPLEMENTATION
PLAN.**

This design removes unnecessary work before adding threads, uses Rust's
available parallel execution safely, preserves exact-call quality, retains
deterministic commits, keeps Issue #144 completeness intact, and makes the
ten-second release gate a realistic consequence of the algorithm rather than
a timeout workaround.

