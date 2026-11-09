# AF-PERF-002: next broad default-operation discovery, rolling reads

Agent: Agent MaxHeadRoom
Status: characterization executed GREEN; bounded diff-source ownership DESIGN recommended.
All completed corrections stay CLOSED / FIX / PROTECTED. Startup projection deferred,
delta-count removal rejected, dense same-signature equality deferred. No publication.

## Acquisition and actual production trace

Fresh Clean-CTX admission is degraded: frontend_retired timeout. No search_codebase
or read_files fallback tools are exposed; bounded host reads establish this trace.
No server launched. This pass selects uncovered registered diff/delta read handlers,
not more measurements of protected snapshot matching or manifest construction.

create_default_registry in src/mcp/tool_handlers/registry.rs registers
diff_code_context and delta_code_context. Tool dispatch invokes core/delta.rs directly.
No nondefault feature or custom pipeline is required beyond supported TypeScript.
The discovery fixture disables persistence/provider lifecycle to isolate these normal
read paths; persisted delta behavior has separate existing positive controls.

## Source hypotheses and consumer boundaries, not measured ranks

| Candidate | Exact work and consumer | Preliminary boundary |
| --- | --- | --- |
| Diff source ownership | handle_diff_code_context calls read_source, then as_str().to_string() before diff_code_context_handler takes &str | Full source clone occurs even when protected hash shortcut returns no changes. Arc<String> already owns an immutable snapshot independently of the cache guard; retaining that Arc may suffice. Do not alter the protected helper/hash/snapshot policy. |
| Delta previous IR on unchanged return | handle_delta_code_context clones get_ir into previous_authority before unchanged-source admission, then separately clones cached_ir and returns | First clone has no consumer on that return. Changed-source code needs the captured tuples/hash for publication equality and delta generation. A potential local delayed capture must retain the same lock, original guard/error ordering and changed-path authority, not remove the snapshot globally. |
| Delta second source read | initial source snapshot is followed by another read_source under the canonical IR lock before comparing hash | A repeated metadata/cache access is source-visible, but can observe a concurrent file change. Reusing the first read is not presumed equivalent. Currentness/error/TOCTOU analysis precedes any proposal; no change is authorized. |

Other inspected persistence/replay paths construct data for compatibility, publication
or presentation consumers; no new candidate is promoted merely because construction
is visible. This is a bounded scan of uncovered operations, not exhaustive coverage.
Real workspace invocation frequencies and CPU dominance remain unknown.

## Actual-site characterization and semantic controls

Only cfg(test,typescript) observation calls are added to core/delta.rs. Existing
source clone, tuple clone, lock scope, hash/check/compile/publication and return order
remain. New tracked observation/tests live under src/tests/mcp and never feed a
production decision. The original tuple expression is named locally to observe its
actual resulting owned vector; its cloning and fallback behavior remain identical.
Protected source implementations, including all three final manifest identities,
are checked unchanged by the operator driver.

Twelve independently controlled TypeScript populations vary 0/1/8/64/256 methods,
0/64/4096 repetitions of six-byte Unicode text in bodies, independent comment-only
source padding, and 1/4/16 repeated unchanged requests. Each dispatches BOTH actual
registered operations in a fresh state: cold, unchanged warm, external comment edit,
and changed-source warm. These are 96 full rows, not 96 syntax dispositions.
Source size changes force normal mtime/size invalidation without sleeps. Compiled
tuple population is checked rather than assumed from source method count.

Rows retain complete responses, canonical aliases/versions/tuples/hashes before and
after, absence of persistence, actual owned source/tuple clone counts/logical string
lengths/capacities, cached-return discard count and second-read count/offered length.
Diff cold/no-change responses and stored baseline hash must agree with actual source.
Unchanged delta must return cached=true with identical canonical identity; every
delta phase's stored source hash must match current source. Changed-path snapshots
are retained separately; previous copies there are not called unused. Missing-file
operations must return errors before any observed clone. Existing restart/fidelity
and canonical-epoch delta controls run separately, preserving persistent/public
lifecycle coverage without editing protected persistence or encoder code.

Capacities are cumulative slots/logical string capacities, not peak memory or
allocator events. Recursive tuple length observation adds overhead. Complete dispatch
elapsed samples include response construction and local intermediate teardown, but
retained responses survive outside timing. Fixture/oracle/identity capture is outside
timing. There is no paired replacement or test-only alternative, so no speedup,
CPU attribution, allocator traffic, public latency improvement or complete teardown
measurement can be inferred. Second-read lengths are offered source lengths, not
physical file-read bytes. Source bodies shared by Arc are not copied by Arc cloning.

This characterization is not a future optimization RED or frozen independent oracle.
A subsequent design/repair needs separate complete response/currentness/error,
concurrency and real restoration controls. No production correction is implemented.

## User-owned verification and ranked return

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-rolling-read-discovery.ps1
```

Expected four tracked passes: two new public characterization/error tests plus two
existing delta persistence/epoch controls; all 96 rows; warning-fatal all-target/
all-feature Clippy, formatting, active size, strict encoding and exact source receipts.
No optimization RED, mutation or composed gate is run for discovery.

All runtime gates UNRUN. Agent performed bounded source/diff, script parser and
strict UTF-8/line checks only; no tests/builds/lints/formatter/runtime. After receipt
inspection, return INVESTIGATE/DEFER/REJECT/UNMEASURED ranking based on observed work
and realistic semantic exposure. Recommend at most one bounded design investigation,
not implementation. Do not manufacture a ranking from unexecuted observations.
No stage/commit/push. Census remains 13/9763 scoped, 9750 unresolved plus96 supplements.

## Executed characterization and ranked owner return

Agent MaxHeadRoom inspected rolling-read-discovery-20261109-093147: four tracked
passes, zero failed/ignored, all 96 complete public rows. New characterization/error
tests pass in 27.66s; existing restart/high-fidelity and canonical-epoch acceptance
pass in 4.08s and 3.57s. Warning-fatal all-target/all-feature Clippy passes in 1m15s;
both format logs are empty. Size passes three active text files; strict encoding
passes 2282 text files. All 1397 recorded hashes match at inspection before this
documentation update. Protected closure identities agree, including all three
manifest identities. inspection.json retains checked totals, executed packet
identities and compact warm-row observations. Original logs/receipt stay untouched.

Exact executed packet SHA-256 identities:

- core/delta.rs: 07fcdd4cd88b831428af3890faedf211021da5d534ba7007847d55ebc8def7d6
- rolling_read_discovery_af_perf_002.rs: a775f9148ce17d35a6d797ffd8a8f7346910cd2968175428694ac33ec16a6b00

Actual unchanged public requests demonstrate these independent dimensions:

| Methods / body repeats / comment repeats / requests | Diff source copies / logical bytes | Delta discarded tuples / strings / logical bytes |
| --- | --- | --- |
| 0 / 0 / 0 / 1 | 1 / 22 | 1 / 3 / 14 |
| 1 / 0 / 0 / 1 | 1 / 52 | 4 / 13 / 49 |
| 8 / 0 / 0 / 1 | 1 / 262 | 25 / 83 / 294 |
| 64 / 0 / 0 / 1 | 1 / 2050 | 193 / 643 / 2476 |
| 256 / 0 / 0 / 1 | 1 / 8506 | 769 / 2563 / 10594 |
| 8 / 64 / 0 / 1 | 1 / 3334 | 25 / 83 / 294 |
| 8 / 4096 / 0 / 1 | 1 / 196870 | 25 / 83 / 294 |
| 8 / 0 / 4096 / 1 | 1 / 24838 | 25 / 83 / 294 |
| 8 / 0 / 0 / 4 | 4 / 1048 | 100 / 332 / 1176 |
| 8 / 0 / 0 / 16 | 16 / 4192 | 400 / 1328 / 4704 |
| 64 / 64 / 0 / 4 | 4 / 106504 | 772 / 2572 / 9904 |
| 1 / 4096 / 4096 / 1 | 1 / 49204 | 4 / 13 / 49 |

Zero methods still includes a class, not empty IR. Body repeats are six-byte Unicode
text repetitions, not bytes. High fidelity in these actual cases does not retain
body payload in canonical tuples: the long-body delta copy remains 294 logical
bytes. This prevents projecting source-size growth onto tuple-copy growth. At 256
methods the discarded previous copy retains 769 outer capacity slots, 2563 inner
slots and 10594 logical String capacity bytes; these are not allocator-event counts
or a peak-live-byte measurement. The separately needed cached-return copy remains
769 tuples. Each warm request discards exactly one observed previous-authority copy.

Diff clones source on cold, warm, externally changed and changed-warm calls. Its
unchanged helper returns the protected no-change response with the matching baseline
hash; it does not need a new structural parse. Delta cold calls have no previous
copy; unchanged warm and changed-warm return cached=true with identical canonical
identity. Comment-only external edits execute the existing source-change path and
refresh source hash; their previous snapshots are required by changed-path admission,
so they are not classified as discarded warm copies. Full responses and before/after
tuples/hashes remain in the receipt. Missing-source errors observe no copies.

The second delta read occurs once per request with an established baseline, including
unchanged requests: 16 repeated calls offer 4192 source bytes at that site. This is
not 16 disk reads; read_source can return a cached Arc after checking filesystem
metadata. Its possible observation of a concurrent edit is part of the currentness
boundary. No attempt was made to bypass it or model it as useless work.

### Ranked shortlist

1. **INVESTIGATE — diff source ownership.** Measured full-source copying on every
   explicit diff request, including hash-hit/no-change calls. Triggering exposure is
   repeated client diff requests over large source files; production frequency is
   unknown. The downstream helper consumes only &str and the existing Arc owns an
   immutable snapshot after cache-lock release. Retaining that Arc appears to remove
   only an extra copy without a new index/cache or changed matching/currentness.
   Lowest apparent implementation complexity and semantic risk among measured
   candidates. Recommended next step is a bounded design comparison, not a repair.
2. **INVESTIGATE — delta previous-authority copy on unchanged return.** Measured
   growth with actual tuple count and request repetition; one copy per cached return
   has no downstream consumer. A local delayed capture may be possible inside the
   same existing IR lock after the unchanged branch. This has greater semantic risk:
   failed checks, concurrent publication and changed-source authority equality need
   preserved capture/lock/error ordering. Retain as independent candidate, not bundled
   with diff ownership. No direct allocator or time dominance over rank one is proved.
3. **DEFER — second delta source read.** Repetition is demonstrated, but the earlier
   source snapshot can become stale. Benefit is unmeasured separately from necessary
   hashing, and currentness/error equivalence is unresolved. Reusing the first snapshot
   is not currently a defensible implementation recommendation.
4. **REJECT — removing cached-return IR construction wholesale.** The second cached
   tuple copy feeds compiled_from_tuples, hierarchy, economics and the actual public
   IR/presentation. Its consumer is real; disappearance of the first unused copy does
   not authorize elimination of this necessary output path.

No additional UNMEASURED hypothesis is promoted to this shortlist. This is engineering
priority, not measured CPU ranking. Single complete debug dispatch observations include
source reads/hashing, presentation/economics, JSON responses, existing diagnostics and
new length walks; they cannot attribute cost to either copy. Necessary output and
retained response teardown are separate limits. No public latency, machine-instruction,
allocator-saving, peak-memory or enterprise-frequency claim follows.

### One proposed next design slice, not implementation

Compare ONLY the existing diff handler's owned source conversion with retaining the
read_source Arc and lending its str to the SAME protected helper. Require complete
registered cold/warm/changed/error responses, exact baseline snapshot/hash, fidelity
and path identity, source-cache invalidation, source-read failure and concurrent-edit
snapshot-lifetime controls. Include all supported fidelities and independent source
payload versus structural dimensions. Compare symmetric construction-inclusive local
costs including copy/drop or Arc ownership/drop, while retaining actual public controls;
do not call test-copy timings public speedup or remove protected helper work.

Only if owner authorizes that design should it be executed. Any later implementation
needs separate genuine original copy-budget RED with semantics GREEN, frozen binary/
public/cache/currentness controls as applicable, actual integrated costs, finite
protection, real source-copy restoration mutation and mandatory composed acceptance.
This discovery GREEN is not optimization RED. No production optimization, further
design execution, mutation/composed run, stage, commit or push follows this return.
All prior scopes remain protected and deferred/rejected dispositions stay unchanged.
