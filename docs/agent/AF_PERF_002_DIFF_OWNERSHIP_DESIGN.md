# AF-PERF-002: registered diff source ownership design comparison

Agent: Agent MaxHeadRoom
Status: test-only comparison executed GREEN; minimal Arc retention FIX PROPOSED.
No production correction authorized or implemented.
All earlier corrections remain CLOSED / FIX / PROTECTED; all deferred/rejected
dispositions unchanged. Delta previous-authority copying remains independent.

## Exact source and lifetime trace

Registry create_default_registry binds diff_code_context to core::handle_diff_code_context
in core/delta.rs. Path/fidelity validation precede state.read_source(resolved_path).
The source cache resolves canonical key, checks mtime/size, and returns Arc<String>:
cache hits clone the Arc reference; misses read/store an immutable source snapshot.
Its lookup/update Mutex guards are released inside read_source before handler use.
The handler currently converts that acquired snapshot to String with
snapshot.as_str().to_string(), drops its acquired Arc, and lends &str to the
UNCHANGED diff_code_context_handler while holding only the existing diff-cache
write guard. This new owned String is dropped after helper/response handling.

The protected helper canonicalizes its path, constructs the same fidelity key and
hashes source. Matching stored hash plus baseline returns the existing no-change
text. Otherwise it builds/diffs the current snapshot, updates baseline/hash and
formats the existing response. Ownership replacement must not change any of that.

The test-only alternative transfers the acquired Arc into a local owner enum that
implements Deref<Target=str>. Its String field stays alive through the SAME helper
and response construction, independently of the source-cache guard or subsequent
cache eviction. No additional Arc::clone, cache, read, lock, API or producer policy
is introduced by the alternative. Transfer is counted as an ownership operation,
not a new atomic reference increment. Both alternatives release their acquired Arc
eventually; original releases it before helper after copying, alternative afterward.

Production builds retain the original owned String expression. Only cfg(test,
typescript) selects the comparison enum and hook. Default test mode selects original
ownership, preserving earlier discovery observations. Mode/drop guards restore
thread-local controls on unwind; no such thread-local state is production authority.
Protected helper, source cache, structural matching and baseline storage bytes are
untouched. This is test-only execution of actual registered handlers, not a proposed
test-mode production switch.

## Complete paired public and lifetime controls

Six independent structure/body/comment populations cover 0/1/8/64 methods, Unicode
body and comment repeats 0/64/4096. Each runs all five supported fidelities through
cold, warm, four repeated warm calls, external changed source, changed warm, explicit
cache eviction/replacement and fresh-state restart. Original and alternative use
the same canonical fixture path in successive fresh states. Complete responses,
baseline snapshot Debug representation, source hash, cache source, semantic owner,
alias and canonical tuples compare exactly: 210 paired rows, 420 work rows. No
response/path/error/hash/order normalization. Alias absence is preserved rather than
inventing ownership for diff. Necessary hash-input lengths and copy lengths are
checked independently against generated source bytes.

Additional persistence-enabled high-fidelity sequence produces seven paired rows
and fourteen work rows. It checks SQLite load_latest remains absent before/after
diff and restart; diff's rolling baseline is in-memory, not a durable checkpoint
producer. The existing positive delta restart/high-fidelity test runs separately.
This does not claim diff has a durable baseline contract or certify every persistent
failure. Database files belong only to the temporary fixture.

Missing file, invalid-UTF8 source and directory-as-source exercise real registered
errors in both modes, requiring exact complete error equality and no acquired
snapshot/helper invocation. No platform-specific chmod assumption is made; directory
and decoding failures provide deterministic unreadable-source boundaries.

Five positive concurrent-edit controls, one per fidelity, acquire old snapshot then
pause at a deterministic test hook before helper. A writer thread changes the file
and joins; the hook evicts/reloads source cache and drops its new reference. The
borrowed old source must remain byte-identical, helper baseline/hash must represent
old source, cached source must be new, and following public request must update
baseline/hash to new source. Both modes require full response/cache identity equality.
The hook's deliberate extra read belongs to the fault fixture, not either normal
ownership path. This tests a controlled interleaving, not exhaustive scheduling.

## Work and paired local cost boundaries

Actual handler observations record named read attempts, successful acquired snapshots,
owned copies/logical bytes, retained Arc transfers, helper calls/input lengths and
owner drop entries. They do not count source-cache-internal Arc increments, allocator
events, peak memory, CPU instructions or bytes inspected by hashing. The helper's
single source hash remains source-traced required work; helper input length is not
a measured CPU hash-byte count. Drop counters count owner teardown entries, not
allocator frees or Arc strong-count internals.

Ten independent structure/payload populations run local hash-hit and changed-hash
paths separately (20 paired cost rows). Same actual read_source/acquire/helper code
is used; state, fixture writing, baseline setup and source-cache warmup are outside
timing. A fixed three warmups then 24 alternating samples include metadata/cache
source retrieval, original copy or retained Arc, protected helper, output String
drop and ownership drop. Normal baseline cache/state destruction is outside timing,
matching operation-local reuse. Retained public JSON response construction is not
timed locally. These are not public request-latency measurements.

Diagnostic updates and earlier discovery counters are disabled for BOTH timed modes;
residual enum/mode/drop checks remain. Common output/hashing/parse/diff work is kept,
not attributed to ownership. Enabled counts come from separate registered semantic
runs. Source/helper layout, enum dispatch and debug diagnostics limit extrapolation
to a minimal integrated expression. No allocator-saving or public speedup claim is
authorized from this packet. No timing search or alternative matrix expansion.

## User-owned verification and recommendation boundary

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-diff-ownership-design.ps1
```

Expected seven tracked passes: four design tests, two earlier discovery controls,
one existing persisted delta restart control. Require all 210 paired public rows,
434 work rows, five lifetime cases, seven persistent rows and twenty cost rows;
warning-fatal all-target/all-feature Clippy, formatting, size, strict encoding and
exact source/protected identities. No mutation or composed closure gate.

All runtime UNRUN. Agent performed bounded source/diff/script parser/UTF-8 review
only; no Cargo/test/build/formatter/lint/runtime. After receipt inspection return
FIX/DEFER/REJECT with measured tradeoffs and at most the minimal ownership proposal.
Any later fix requires owner approval, separate genuine original copy-budget RED,
frozen complete public/cache/currentness semantics, real original source-copy
restoration RED/restored GREEN, integrated costs and composed acceptance. Design
GREEN cannot substitute for optimization RED. No production optimization, stage,
commit or push. Census unchanged; protected/deferred/rejected scopes stay closed.

## First attempt: incorrect test fidelity variant

diff-ownership-design-20261109-095331 stops during warning-fatal Clippy compilation:
the new fixture referenced nonexistent Fidelity::Full. The supported fifth mode is
Fidelity::Verbatim with public argument verbatim. Corrected the test mapping and
both five-mode matrices; coverage remains all supported fidelities. No production
or protected helper change, suppression or gate weakening. No tracked design test
ran in this attempt. Rerun the same design command; all runtime gates remain
user-owned. Agent MaxHeadRoom performed bounded source/diff/UTF-8 review only.

## Second attempt: cache observer used the wrong path spelling

diff-ownership-design-20261109-101136 passes Clippy and three design tests but fails
the first lifetime case when the observer reports cached_source=null. The observer
queried canonicalized path spelling, whereas the actual handler's read_source uses
the admitted resolved spelling. Source-cache absolute-key fast path preserves that
spelling on Windows. Updated only the test observer to call the same existing
resolve_file_path_checked and query its returned path verbatim. Baseline key remains
the canonical key used by the protected helper. The assertion requiring the new
cached source is retained; no response/hash/path normalization or production change.
All complete public, persistence and cost results from this incomplete attempt stay
separate and do not constitute full acceptance. Rerun the same design command;
Agent MaxHeadRoom ran no tests/build/formatter/lint/runtime.

## Executed paired comparison and recommendation

Agent MaxHeadRoom inspected diff-ownership-design-20261109-101927: seven tracked
passes, zero failed/ignored. Four new tests pass in 71.92s; two existing discovery
controls in 27.19s; persisted delta restart control in 4.10s. All 210 paired public,
434 work, five lifetime, seven persistent and twenty paired cost rows complete.
Warning-fatal all-target/all-feature Clippy passes in 29.13s; format logs both empty,
size passes six active text files, strict encoding passes 2286 text files. All 1399
recorded source hashes match before this documentation update. Protected identities
remain exact. inspection.json preserves checked totals, work totals and full costs;
original receipt/logs untouched. No agent runtime or production replacement.

Exact executed instrumented SHA-256 identities:

- core/delta.rs: 56774a64cad78bde641d488b6076b0377ef516ca4c7868d24d52b9e2a000965f
- diff_ownership_design_af_perf_002.rs: 203f1eff7631fd053281a8d6de98fda4bd20a824fa5763128f7c4eda38267701
- diff_ownership_cases_af_perf_002.rs: 861e1642e3dcc34465fe4f82ec2739f572d4169ddd43f0ec2115851138184787

### Semantics, ownership and work

All five supported fidelity modes retain exact complete registered responses and
baseline snapshot/hash, source-cache contents, canonical path/owner/alias/tuple
identity. Cold, hash-hit, repeated warm, real external size-changing edit, subsequent
warm, explicit source-cache eviction/reload and fresh-state restart compare equal.
All errors compare exactly for missing paths, invalid UTF-8 and directory access.
No normalization, dropped response fields, changed helper or baseline policy.

The five controlled writer-thread interleavings pass: cache replacement after
source acquisition does not change either operation's borrowed old snapshot. Helper
baseline/hash are old while cache source is new; the next request consumes new source
and updates the baseline/hash identically. Both owners survive the full helper use.
This establishes the tested observation boundary, not exhaustive filesystem races.

Persistence-enabled diff/restart cases preserve the same in-memory reset behavior
and absent durable checkpoint metadata. Existing positive persisted delta restart
coverage remains separate. No new durable diff semantics or persisted identity is
invented from an alias. Paired canonical identities and all complete responses stay
in the receipt; test observations never supply publication/currentness authority.

Across 217 public phase rows per mode (including persistent cases and four-request
warm rows), each mode performs 310 named acquisition attempts/successful snapshots
and 310 helper calls offering 11419240 source bytes. Original ownership performs
310 full String copies/11419240 logical bytes and 310 owner drops. Alternative
performs zero copies/zero copied bytes, 310 acquired-Arc transfers and 310 owner
drops. The shared source-cache reference operations inside read_source are common
and uncounted; transferring the returned Arc is not an extra Arc::clone. These are
aggregate fixture work totals, not workload frequency, allocator events or inspected
CPU/hash bytes. Lifetime/error controls are separate from these public work totals.

### Construction-inclusive local results

Microsecond medians below are original / alternative. Each cell retains 24 fixed
alternating samples, including acquisition, helper/output teardown and source-owner
drop, with common setup outside timing. Only High fidelity is timed; semantic
comparisons cover all five fidelities. Changed-hash local fixtures use a prior
comment difference, so they exercise the parse/snapshot/diff path without claiming
structural edit diversity; the public matrix separately changes method bodies.

| Methods / body repeats / comment repeats | Hash hit us | Changed hash us |
| --- | --- | --- |
| 0 / 0 / 0 | 334.2 / 334.0 | 20893.9 / 21297.5 |
| 1 / 0 / 0 | 354.6 / 322.5 | 20565.7 / 20638.5 |
| 8 / 0 / 0 | 314.0 / 319.8 | 21122.3 / 21178.3 |
| 64 / 0 / 0 | 482.1 / 467.6 | 26685.9 / 27579.1 |
| 256 / 0 / 0 | 1043.2 / 1038.4 | 48985.7 / 51286.1 |
| 8 / 64 / 0 | 676.8 / 658.5 | 23535.9 / 22051.7 |
| 8 / 4096 / 0 | 13739.6 / 14140.3 | 61140.5 / 61348.2 |
| 64 / 64 / 0 | 2124.3 / 2151.4 | 36679.0 / 35296.3 |
| 1 / 4096 / 4096 | 3671.5 / 3680.7 | 30148.7 / 30086.2 |
| 8 / 0 / 4096 | 1971.5 / 1976.4 | 25803.5 / 26099.3 |

No consistent speedup is demonstrated. Every original/alternative index-6..18 range
overlaps. Long-body hash-hit alternative median is about 2.9% higher: original
13592.6..14154.1us versus alternative 13576.8..14366.7us. The 256-method changed-hash
median is about 4.7% higher with broad overlap: 45767.9..56008.4us versus
45267.0..56326.4us. Preserve these increases, not just favorable cells. Identical
helper work, filesystem/debug logging, enum dispatch and scheduling remain substantial;
logical copy removal does not prove a causal timing change. No consequential stable
regression is demonstrated, but integrated costs still require their own review.
No public/release speedup, allocator saving or peak-memory benefit is claimed.

### Disposition: FIX PROPOSED, conditional owner review

Recommend exactly one minimal ownership correction: retain read_source's returned
Arc<String> in the diff handler and pass its borrowed str to the same helper. Keep
the read/path/fidelity/error order, helper/hash/baseline/matching implementation,
cache guard and response handling unchanged. The case for this proposal is removal
of directly observed redundant ownership with positive lifetime/currentness evidence,
not a demonstrated CPU/latency win. An owner requiring demonstrated speedup rather
than work elimination may defer on the present timing evidence; do not seek more
samples or add variants to manufacture a favorable result.

Before any production edit: owner approval, frozen original handler and complete
response/cache/currentness/error/lifetime oracles, an independent actual-site zero-
source-copy budget, then genuine original budget RED with all semantics GREEN.
This design GREEN is not optimization RED. After approval/RED require unchanged
budget and current-byte public/source/cache GREEN, actual integrated costs with
consequential regression review before closure, finite ownership/helper protection,
isolated restoration of the real original source copy producing budget/static RED
with semantic GREEN, byte-exact restored GREEN, focused warning-fatal checks and
mandatory composed acceptance with exact final identities. Keep historical test-only
comparison distinct from active production evidence. No implementation follows
this return. Delta previous-authority clone remains independent; other protected,
deferred and rejected scopes remain unchanged. No staging, commit or push.

## Owner approval and separate original RED packet

Owner approves only the minimal ownership substitution after genuine original RED.
The original-only baseline and real handler-site copy budget are tracked separately
in [diff source repair](AF_PERF_002_DIFF_SOURCE_REPAIR.md). All 225 original semantic/
public/error/cache/lifetime rows must pass before intended budget RED. Historical
design timings and source identities above remain their executed evidence, not
optimization RED or current production acceptance. No release correction yet.

## Separately accepted production correction

The authorized minimal copy removal is now CLOSED / FIX / PROTECTED. Its genuine
original RED, 225 frozen semantic/work controls, integrated costs, real conversion
restoration mutation and composed acceptance (106 passes/51 groups, 350 matching
hashes) are recorded in [diff source repair](AF_PERF_002_DIFF_SOURCE_REPAIR.md).
Historical design timings/identities above remain design evidence; they are not
substituted for final production receipts. No public speedup or allocator claim.
