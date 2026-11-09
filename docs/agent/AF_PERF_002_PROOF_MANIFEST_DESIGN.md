# AF-PERF-002: proof-aware per-root manifest test-only design comparison

Agent: Agent MaxHeadRoom
Status: executed comparison GREEN; explicit-demand manifest construction FIX PROPOSED.
All protected corrections remain closed. Startup projection deferred/count removal
rejected; dense same-signature equality deferred. No stage/commit/push.

## Exact demand and consumer trace

Registered workspace_query find_entities -> entities::try_prepare_find_entities.
Ordinary occurrence mode calls PreparationContext::hydrate -> CleanAuthoritative.
With declarations_only=true and workspaceRoot, hydrate_declarations calls proof-
backed acquisition for AcquisitionTarget::declarations. HydrationAuthority::proof_target
returns None only for the explicit ordinary authority variant; it returns Some for
the proof-backed variant, including failed, incomplete and cached proof operations.
Presence is available at discover_candidate_paths entry BEFORE filesystem scan.
Absence of a returned proof is never used as the skip predicate.

Filesystem-only and provider-fallback discovery use scan -> enumerate/build combined
inventory -> name candidate selection -> root_inventories. The combined inventory,
source reads/hashes/normalization and attempted/completed flags remain necessary.
Per-root construction filters/clones entries, constructs membership vectors and
sorts/deduplicates/hashes them. execution.rs consults these vectors only with a proof
target: required reconstruction after ordinary discovery completion and provisional
proof preparation. Proof-backed cached operations retain the original construction
path too; no additional cached-proof shortcut is introduced.

Incomplete discovery never becomes completion authority. Root completion decisions
use scan.completed from combined acquisition, not the per-root vector. Retries,
publication failure/currentness and provider versus Clean proof authority remain
unchanged. The extra proof scan in execution.rs remains untouched. Other callers
without a demand observation fail closed to original manifest construction.

## Test-only alternative, no proposed global state

The comparison runs the ACTUAL registered pipeline under a cfg(test,rust)-only
switch. An existing test-only candidate hook is wrapped to observe the actual typed
caller's proof_target().is_some() boolean before discovery. It preserves exactly
the original test candidate-injection return. The new wrapper and switch are never
compiled into release artifacts. No protected production implementation is replaced.

At the existing test observation boundary, root_inventories may return an empty
unused vector only when the test switch is enabled AND captured demand is explicitly
Some(false). Some(true) and unknown demand retain the exact original constructor.
A per-thread RAII guard resets mode, captured demand and observation state even on
unwind. This is TEST APPARATUS, not a new production cache/index/authority mechanism.
A future implementation would thread a private operation-local demand argument,
not ship this switch or introduce thread-local production policy.

The internal unused per-root vector intentionally differs for ordinary operations;
complete public/cache/source semantics must remain equal. Proof operations require
complete vector/digest/entry/flag/order/proof equality. Combined acquisition and all
errors/retry decisions remain on the original code. No source-body ownership change.
InventoryEntry continues to share Arc<String>; avoided bytes are metadata only.

## Positive public and failure controls

Twelve independently controlled discovery populations retain1/4/8 roots, disjoint/
nested membership,0/1/32 files,0/4096 Unicode repetitions, duplicate configured roots,
missing roots and malformed UTF-8 source. Each uses two fresh states over identical
source fixtures: original versus test-only projection. Five actual public operations
per state yield60 paired identity rows:

1. Ordinary cold query.
2. Identical successful warm query or incomplete-root retry.
3. Different-name ordinary query.
4. Proof-backed declarations_only query AFTER ordinary discovery.
5. Identical proof-warm or incomplete-root proof retry.

Complete response Values, ordered per-root digest/owner/hash/kind/flags, full provisional
proof Debug contents (including provenance), barriers, owners/currentness snapshots,
live source tuples/hashes/text-cache state and edge populations are compared. The
actual captured authority must match the public mode. Valid populated proof queries
must expose source_complete=true and reach manifest/group/proof creation; absence
of positive proof is test failure. Missing-root proof replies must remain incomplete.
Proof-backed work receipts must match exactly, including warm reconstruction counts.
Ordinary projection requires zero predicates/clones/manifests; missing-root ordinary
warm requests must still execute scan and the test-only skip boundary again.

The main population matrix disables persistence as in preceding discovery and checks
the absent store. An additional positive test enables persistence and actual Edit
compression before proof-backed public queries. Original/candidate use the identical
configuration/database path with fresh temporary database files between modes.
Complete compression/proof responses, actual provisional proof/work and complete
loaded durable context Debug identity before/after query are compared. Durable context
includes authority, canonical IR, semantic edges/dependencies, source hash, fidelity
and compatibility identities; no durable identity is invented from a live alias.
This bounded positive case is not universal persistence/restart certification.

Existing raw root equivalence retains malformed/relative owners, duplicate entries,
empty/missing/overlap normalization and Arc identity controls. Existing incomplete-
authorized-root proof regression runs separately. No optimization RED or closure gate.

## Symmetric local costs

For each population, benchmark ordinary and proof-required LOCAL pipelines separately:
actual filesystem scan (enumeration/read/hash/candidates), conditional per-root
construction, and for proof mode actual prepare_clean_acquisition. Full scan/result/
provisional proof teardown is inside timing. State/config creation and root resolution
are outside, as are compilation/publication/final response. These are24 cost rows,
not actual startup/public latency measurements.3 warmups then24 alternating samples
per mode/case are fixed; no repeat-until-favorable timing.

All Work/receipt updates are disabled symmetrically in timed samples, while the same
residual diagnostic checks remain at sites that execute. No observer JSON/Debug proof
receipt is built during timing. Enabled counters/complete receipt comparisons come
from public semantic runs. Ordinary successful warm and incomplete retries are
reported separately as actual work; local timing intentionally bypasses discovery
reuse to isolate complete construction/drop. Do not substitute local fresh scans
for actual cache-hit request cost. Proof-required timing retains original per-root/
proof work in both alternatives and serves as a code-layout/overhead control.

Counters distinguish root predicates/rejections, metadata String clone bytes, shared
Arc body lengths, entries/capacity, required group clones and manifest/proof receipts.
They do not count allocator events, peak memory, machine instructions or hash CPU
bytes. No shared source body bytes are called copied. No synchronization/ownership,
completeness policy or proof-cache change is proposed.

## User-owned execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-proof-manifest-design.ps1
```

Expected five tracked passes: paired actual-public/local-cost comparison, unknown-
demand fail-closed control, enabled-persistence positive proof control, raw root
equivalence and incomplete-root protection.
Require60 complete public/proof identity rows and24 paired local cost rows, warning-
fatal all-target/all-feature Clippy, format/size/encoding and source receipts.
Protected state/SQLite/encoder/matcher/edit bytes must match before/after; test-only
instrumented source identities are frozen after formatting and verified unchanged.
Full results remain under target; console prints a short outcome. All runtime UNRUN.
Agent performed bounded source/parser/UTF-8/diff review only. No server/binary launch.

Return FIX/DEFER/REJECT based on complete executed semantics, retained proof/retry
work and paired local costs. If FIX is justified, propose one private explicit-demand
slice, requiring owner approval, genuine original unused-manifest budget RED,
frozen ordinary/proof/public/durable/currentness oracles, actual integrated costs,
independent real constructor restoration mutation and composed acceptance. No release
optimization is authorized now. If costs/semantics fail or remaining coverage is
material, return the limitation rather than weaken authority. Prior protected scopes
stay closed; census unchanged13/9763 scoped,9750 unresolved plus96 supplements.
## Executed comparison and returned recommendation

Agent MaxHeadRoom inspected proof-manifest-design-20261108-222948: five tracked
passes, zero failed/ignored. Three comparison tests55.43s, raw root contract0.01s
and incomplete-authorized-root protection0.01s GREEN. All60 paired public rows,
24 local cost rows and one enabled-persistence paired proof control complete.
Warning-fatal all-target/all-feature Clippy34.18s, both format logs empty, size30
active files and strict encoding2271 text files GREEN. All1392 recorded hashes
match at inspection before this documentation-only update. All nine protected
implementation identities match; test-only instrumented bytes stayed frozen during
execution. inspection.json retains checked totals and costs. No release optimization,
original optimization RED, mutation or composed gate executed.

### Positive proof, public, currentness and retry results

All original/candidate complete responses and live/source/cache identities agree.
Proof-backed request per-root entries/order/digests/flags, grouping, full provisional
proof/provenance/barrier/currentness receipts and named work agree exactly. Populated
complete proof cases require and achieve source_complete=true. Required manifests
are reconstructed after an earlier ordinary discovery even when its discovery
completion was reused. Successful proof-warm calls reuse compatible completion
identically; no absence-of-returned-proof shortcut is used.

At eight overlapping roots/32 entries:

| Actual public phase | Original predicates / clones | Candidate predicates / clones | Candidate skipped calls | Group clones original / candidate |
| --- | ---: | ---: | ---: | ---: |
| Ordinary cold | 256 / 144 | 0 / 0 | 1 | 0 / 0 |
| Identical ordinary warm | 0 / 0 | 0 / 0 | 0 | 0 / 0 |
| Different-name ordinary | 256 / 144 | 0 / 0 | 1 | 0 / 0 |
| Proof after ordinary | 256 / 144 | 256 / 144 | 0 | 144 / 144 |
| Identical proof warm | 0 / 0 | 0 / 0 | 0 | 0 / 0 |

The skipped ordinary pass eliminates144 metadata/Arc clones,22560 logical String
bytes and eight per-root manifest constructions with144 entries/172 capacity slots
at this fixture. Shared source body bytes remain shared, not copied or newly saved.
Combined source traversal/read/hash/name-candidate construction remains in both paths.
Proof-required passes retain all original metadata/Arc and per-kind manifest work.

Missing plus overlapping roots exercise160 predicates/80 clones on original ordinary
cold, identical retry and different-name requests. Candidate still executes the scan
and skip boundary EACH time but has zero per-root predicates/clones. Both proof
requests (first and repeated incomplete retry) retain160 predicates/80 clones and80
group clones. Both remain source-incomplete; no retry/completion shortcut appears.
Missing/malformed source flags, partial responses and currentness snapshots agree.
Unknown demand independently retains original construction.

Enabled persistence control performs actual Edit compression then a proof-backed
registered query in both modes under identical configuration/fresh database paths.
Full compression/query/proof/work results and loaded durable context identities
before/after query agree. These include retained authority, canonical instructions,
semantic edges/dependencies, source hashes, fidelity and compatibility identities.
This is a bounded actual positive durable case, not whole-database/restart coverage.

### Paired construction-inclusive local costs

Milliseconds below include actual enumeration/read/hash/candidates, root manifest
work and (proof mode) provisional proof preparation, with result/provisional teardown.
They exclude state/config/root resolution, compilation/publication, final response
and actual warm-cache request behavior. Every mode retains24 alternating samples;
updates/JSON receipt creation are disabled symmetrically, residual probes remain.

| Roots / files each / Unicode repeats / overlap / missing | Ordinary original / candidate ms | Proof original / candidate ms |
| --- | ---: | ---: |
| 1 / 1 / 0 / no / no | 2.287 / 2.316 | 2.753 / 2.541 |
| 1 / 32 / 0 / no / no | 13.109 / 12.645 | 16.365 / 15.952 |
| 4 / 8 / 0 / no / no | 18.472 / 17.790 | 19.436 / 19.244 |
| 8 / 4 / 0 / no / no | 26.717 / 25.105 | 26.964 / 26.989 |
| 1 / 32 / 4096 / no / no | 21.409 / 20.427 | 21.636 / 21.490 |
| 4 / 8 / 4096 / no / no | 27.620 / 26.879 | 29.018 / 29.267 |
| 4 / 8 / 0 / yes / no | 27.104 / 25.228 | 27.878 / 28.486 |
| 8 / 4 / 0 / yes / no | 51.034 / 47.321 | 56.391 / 54.673 |
| 4 / 8 / 4096 / yes / no | 50.973 / 48.100 | 47.704 / 50.245 |
| 4 / 0 / 0 / no / no | 11.701 / 10.970 | 8.146 / 8.419 |
| 4 / 8 / 0 / no / yes | 19.054 / 17.725 | 19.605 / 19.953 |
| 4 / 8 / 0 / yes / yes | 34.095 / 29.312 | 28.833 / 29.065 |

Eight-overlap ordinary index6..18 ranges are50.081..51.563ms versus46.503..49.353ms,
with a3.713ms descriptive median reduction (~7.3%). Its unchanged-work proof control
ranges overlap54.671..57.660ms versus53.458..56.840ms. Long overlapping ordinary
ranges overlap44.608..52.619ms versus44.332..52.144ms, and its unchanged proof mode
also shifts42.987..52.729ms versus41.642..57.786ms. Missing+overlap ordinary ranges
are broad/overlapping27.027..38.048ms versus26.007..37.675ms. Singleton ordinary
median is0.029ms higher. Identical proof algorithms vary in both directions, limiting
causal/code-layout interpretation and exact percentage claims.

Eleven of twelve ordinary local medians favor omission, with separated central
ranges in the specified eight-overlap case. This supports investigating the minimal
source substitution alongside the deterministic removal of an unused complete work
unit. It does not establish statistical significance, universal nonregression,
public/release latency or allocator/peak-memory savings. Successful warm operations
already avoid all named work; local fresh-scan medians must not be applied to them.

### Source identities and limits

Exact executed instrumented identities:

- hydration.rs: 3252deae98d070f6a1e0354ee187592d3fc5f6e812aa25d2a9d9547fb347379d
- hydration/filesystem.rs: 65c8745624b0a737a3686f94a5e41e0882e6376c82f5158d83d678e44f641108
- hydration/acquisition.rs: ad1f85c792929cd40fded5173cc59117d9590a12dab1a1a2b046cd410d487690
- proof_manifest_design_af_perf_002.rs: 21d9cbfd311ff693c749e4a1ad8010758e765ffdafc2eb6fdb5a6e8704dec370

All other source/test identities are retained in receipt.json. These include cfg(test)
authority capture/switch/proof observations, not a release implementation. The proposed
production predicate is the EXISTING typed authority's absence of proof target at
request admission; it must be passed explicitly to private scan code. Do not ship
the thread-local test apparatus as production authority. Callers without explicit
demand and all proof-backed kinds must retain construction.

Positive public populations use Rust declaration proof, not every language/capability
or provider lifecycle. Inbound TypeRef proof, provider partial/fallback and source-
change/invalidation cases need independent frozen controls before any final repair
acceptance. Bounded malformed roots/source tests are not universal filesystem fault
certification. No semantic differences were observed within executed controls.

### Disposition: FIX PROPOSED for owner review

Recommend one minimal private explicit-demand implementation slice: avoid ONLY
per-root manifest construction on ordinary discovery known to have no proof consumer;
retain complete combined acquisition/candidates/flags and exact required construction
for proof-backed/unknown demand, including late reconstruction and incomplete retry.
Keep shared Arc ownership, normal root selection, existing caches/proof contracts,
source/currentness authority and public behavior. No general borrowed-manifest,
global index/cache, provider policy or matching change is justified.

This is an owner-review proposal, not implementation authorization. Before any release
edit: freeze actual original public/proof/durable/currentness oracles across ordinary,
proof-warm/reconstruction and failures, add independent actual-site zero-unused-
manifest budget, and execute genuine original intended RED with semantics GREEN.
Passing test-only comparison is not optimization RED. Then require current-byte
semantic/work/integrated-cost GREEN, finite demand/unknown/proof/retry protection,
independent isolated REAL constructor restoration budget/static RED with semantics
GREEN, exact restored GREEN, focused warning-fatal checks and mandatory composed
acceptance with final identities. Preserve test-only historical evidence separately.
If private signature propagation activates file size ceiling, perform only necessary
semantic-preserving local decomposition; do not weaken authority to avoid that work.

No release optimization, staging, commit or push follows this return. Prior repairs
remain protected; startup projection deferred, count removal rejected and dense
equality deferred. Census unchanged. No additional timing merely to seek favorable
samples; owner may approve this bounded slice or defer it based on disclosed limits.
## Owner disposition and original-baseline requirement

Owner approves only the minimal explicit private demand correction, conditional on
genuine original RED. Inbound TypeRef proof, provider partial/fallback and source
invalidation must be incorporated before release edits. New original-only packet
and acceptance sequence are tracked in AF_PERF_002_MANIFEST_DEMAND_REPAIR.md. Historical
design rows/source identities above remain their executed evidence; new cfg(test)
observation and baseline registration do not retroactively change that receipt.
No implementation follows this approval until the extended baseline's valid RED is
inspected. No publication authorized.

## Separately accepted production correction

The explicit-demand correction is now CLOSED / FIX / PROTECTED. Its independent
original RED, frozen extended semantics, integrated costs, real restoration mutation
and composed acceptance (103 passes/48 groups, 343 matching hashes) are recorded in
[manifest demand repair](AF_PERF_002_MANIFEST_DEMAND_REPAIR.md). Historical design
results above remain design evidence; they do not substitute for production receipts.
