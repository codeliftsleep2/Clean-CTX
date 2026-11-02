# Issue #162: larger-fixture config transition timeout

Agent: Agent MaxHeadRoom
Status: owner DEFERRED further performance investigation on 2026-11-02; preserved timeout remains OPEN; bounded correctness acceptance unchanged.

## Owner sequencing: performance deferred

The owner directs finishing current #162 correctness work before investigating
hydration performance further. Duplicate processes, repeated work and poor scaling
are hypotheses, not confirmed causes. Preserve all existing evidence and the open
finding; do not schedule additional performance diagnostics, instrumentation,
threshold/cache changes or long lifecycle reruns during this correctness sequence.
Historical next-performance steps below are deferred rather than authorization to
resume them. No existing bounded acceptance is revoked or expanded by this decision.

## Focused diagnostic inspected: transition alone does not reproduce timeout

Inspected config-transition-diagnostic-2026-11-02T14-12-51-543Z-9800. Both fresh
fixture cases COMPLETED with all three queries ANSWERED; each retained nine exact
Module occurrence-to-target pairs and truthful lower-bound coverage. No diagnostic
error/shutdown error is recorded; same generation-75 binary hash is preserved.

| Fixture | Cold | Warm | After baseUrl edit |
| --- | --- | --- | --- |
| Bounded, 58 TS files | 44.76 s | 1.08 s | 61.01 s |
| Large, 85 TS files | 57.54 s | 1.32 s | 79.90 s |

Both consume identical before/after config hashes. Exact report/RPC/stderr hashes:
 target/issue-162/config-transition-diagnostic-receipt.json.
The shorter cold/warm/config sequence did not reproduce the historical timeout.
This falsifies fixture size plus this transition alone as a sufficient reproducer
under this run's conditions. It does not prove which omitted factor is necessary:
prior dependency mutations, session/persistence history, process state or ambient
load remain possible differences. Do not call a production performance issue
repaired or the larger-fixture finding resolved from this diagnostic.

The post-config query is materially slower than the warm query in both cases;
these response durations are whole-query observations, not a causal breakdown of
compilation/resolution/currentness/index work. The next localization axis is prior
dependency-mutation history versus a fresh process, retaining exact source/config
bytes and the existing timeout. No production optimization or global timeout/cache
policy change is justified yet. No full acceptance rerun is requested by this
inspection; the bounded generation-75 acceptance remains valid independently.

## Observed question and evidence

Why does the first registered Route/same query after adding baseUrl dot fail to
answer within the existing client request deadline in the larger fixture? Is the
cold/warm/config transition alone sufficient, or is prior mutation/session history
required? This is a performance/currentness investigation, not new relation semantics.

The preserved larger Module fixture contains 85 TypeScript files. Two attempts
pass 364 recorded assertions and finish the first two sessions, then time out in
session three on the first query after tsconfig mutation. Removing redundant same
queries changed the request number (85 to 64), not the transition boundary. No
incorrect semantic answer has been observed for the timed-out request: no response
was available to classify. The previous successful queries were pre-mutation.

The narrowed Module fixture contains 58 TypeScript files and passes all 405
checks, including the same config mutation/repair under the unchanged deadline.
RouteConfig and affected preservation also pass the bounded checkpoint. File-count
correlation alone does not establish the cause or complexity of the timeout.
Receipt comparison: target/issue-162/config-transition-offline-comparison.json.
Large failure: target/issue-162/load-children75-config-transition-timeout/receipt.json.
Bounded success: target/issue-162/load-children75-modules-binary-receipt.json.

## Current production trace and its limits

hydration.rs owns configured-root discovery and publication of acquired candidates.
semantic_compilation/compile.rs captures TypeScript authority with the existing
Angular declaration-field and bootstrap demands, then invokes the shared compiler
and artifact boundary. This places config-dependent TypeScript capture on the
actual publication path. It does not localize the recorded delay to resolution,
recompilation, scanning, witness checks, indexing, locking or publication. Tail
stderr shows source reads continuing but lacks causal per-stage timings sufficient
to distinguish those explanations. No production optimization is justified yet.

## Focused next diagnostic

run-config-transition-diagnostic.ps1 reuses the exact hash-verified generation-75
binary and materializes fresh local workspaces from both preserved synthetic report
fixtures. It rewrites only the persistence database path for workspace isolation,
retains the recorded config/source inputs, verifies recorded Rust/build hashes,
and runs bounded first, then large. Each case initializes its own stdio client and
requires the actual workspace_query output schema.

Run only three registered queries per fixture: cold Route/same, warm Route/same,
then Route/same after the exact baseUrl edit. Every completed answer must retain
nine exact Module occurrence-to-target file pairs across all three caller forms,
with lower-bound coverage. Save request timestamps/durations, RPC/stderr, config
hashes, source-report/fixture/binary hashes and exact failure stage under target.
Keep the existing McpClient 120-second deadline and shutdown behavior unchanged.
If the bounded diagnostic itself fails, stop before the large case.

If bounded completes and large times out only after config edit, the small query
sequence reproduces the boundary and supports deeper localized timing next. If
both complete, prior mutation/history or another difference remains necessary;
do not call the historical finding resolved. If cold/warm fails, report that exact
stage separately. A diagnostic timeout is not a failed semantic oracle, a Rust
regression result, or binary acceptance. The script intentionally exits nonzero
for timeout/error so incomplete work cannot be read as PASS.

## Scope and return

No test/build/server/Node/runtime was started by the agent. PowerShell syntax and
UTF-8/static file-length inspection only. No production/test edits, retry, timeout,
cache, discovery, semantic identity/generation, durable format or public-contract
change. No new performance threshold or global policy is proposed. Operator owns
execution. Return evidence before selecting a repair or introducing instrumentation.
A reproducible correctness invariant, if found, needs a tracked src/tests regression;
this diagnostic does not satisfy that requirement. Broader gates remain open.
