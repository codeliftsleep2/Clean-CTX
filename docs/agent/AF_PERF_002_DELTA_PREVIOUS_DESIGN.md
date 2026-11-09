# AF-PERF-002: delta previous-authority delayed-capture design investigation

Agent: Agent MaxHeadRoom
Status: executed paired semantics GREEN; delayed capture FIX PROPOSED for owner review.
No production correction authorized or implemented.
All completed corrections remain CLOSED / FIX / PROTECTED. Delta's second source
read, startup projection and dense equality remain deferred; startup count removal
rejected. Necessary cached-return construction remains required. No publication.

## Exact registered lock and authority trace

create_default_registry routes delta_code_context to core/delta.rs. The handler
resolves path, begins a publication ticket, preflights authority, validates fidelity
and acquires the first source Arc. Tokenizer/economics inputs precede canonical IR
lock acquisition. Under that same guard it reads version, then clones prior tuples
and source hash into previous_authority whenever a baseline exists.

The second read_source and hash/currentness comparison occur while that original
IR guard remains held. is_source_unchanged admits the cached branch. This is the
authoritative unchanged decision, not the mere presence of cache or previous tuples.
The required cached_ir copy happens under the guard before dropping it. That copy
feeds compiled_from_tuples, hierarchy, economics, public IR and presentation, then
the handler returns. The previous_authority copy is not consumed on this path.

On fallthrough the original guard releases before compilation. Publication checks
the compiled source hash/current authority, then reacquires canonical IR and compares
version, entire previous tuple sequence and previous source hash before delta creation.
That snapshot must remain exact on every consuming path. Persisted-baseline work,
pending transitions and semantic projection publication remain unchanged. A source
read/check failure does not authorize dropping the changed-path snapshot.

## The only test-only alternative

Default and release behavior capture before admission. A cfg(test,typescript) flag
can skip that early clone and capture the SAME tuples/hash after the unchanged branch
but BEFORE dropping the SAME original IR guard. No new lock/cache/retry/API. The
candidate still reads original version before admission and retains full equality
for changed-path publication. Required cached_ir copy and the second source read
are untouched. Failed cached conversion/presentation returns still preserve their
original error; the candidate omits only the unconsumed snapshot on that return.

All observations/switches are test-only, RAII-restored on unwind, with default delay
false. Release capture_before is always true; release matching/publication behavior
is unchanged. No optimizer implementation is applied. The whole core/delta.rs file
has a new instrumented identity, but its protected diff handler text is preserved
exactly and checked by the existing complete-handler protection. All other protected
implementation files retain their bytes. New packet identities freeze after formatting.

Moving clone allocation relative to the read/check is the experiment; allocation
failure/abort behavior is not claimed equivalent under resource exhaustion. No new
normal validation, canonical mutation or error-return boundary is introduced. Any
observed authority/lock/publication difference returns for owner review, not a new
mechanism to compensate for it.

## Paired actual public controls

Twelve independently controlled method/body/comment/repeat populations retain the
preceding discovery dimensions: 0/1/8/64/256 methods, 0/64/4096 six-byte Unicode body
repetitions, independent comment padding and 1/4/16 repeated requests. High and Edit
fidelity are compared separately. All calls use actual registered operations.

Each mode records ten phases: cold, unchanged warm, repeated warm, external comment
edit, comment warm, structural edit, repeated pending structural request, explicit
registered compression publication, changed-source warm and source-cache invalidation.
The structural repeat is not assumed cached: its pending delta can still require
the previous authority. Both modes retain exact complete responses, canonical version/
tuples/hash, path/owner/alias, current semantic provenance/edges/fidelity and cached
source: 240 paired rows. Required changed capture tuples/hash compare fully; necessary
cached copies/read/helper inputs and publication-check outcomes must match.

Persistent Edit sequence adds eleven rows, including actual state restart followed
by registered restore_context and delta. Complete restore response, loaded durable
context Debug identity and canonical/provenance identities compare. Exact named
temporary DB files reset between modes, with the same canonical fixture path.
Existing positive persisted restart and canonical-epoch tests run separately.

Missing, invalid-UTF8 and directory-source errors must compare complete responses
with no prior clone. Two controlled writer-thread interleavings are paired: source
edit between initial and second reads; and source edit plus registered compression
publication after compile but before outer publication. The latter must reject the
superseded operation, preserving the newer public authority. Nested response is
retained, not normalized away. Existing real superseded-baseline epoch race runs
separately too. These are bounded interleavings, not exhaustive concurrency proof.

## Work, ordering and timing boundaries

Actual clone-site observations distinguish previous copies, tuple/string populations,
logical lengths and outer/inner/String capacities from REQUIRED cached copies and
changed-path captures. Initial/second read successes and offered lengths, capture/
release-stage events and publication equality outcomes are separate. Capture events
move by design; release/currentness/publication semantics must not. A tracked source
control verifies early/original, cached-copy, delayed and original-release ordering.
It is not an allocator or mutex-event profiler. Lock cost is not separately measured.

Eight controlled populations compare unchanged and comment-changed local requests
separately (16 rows). Three warmups then 24 alternating paired samples are fixed.
State construction, baseline establishment, fixture edits and initial cache setup are
outside timing. Actual registered dispatch, source acquisition/check/hash, all clones,
compile/publication/output, owner teardown and captured response drop are inside.
Diagnostic updates, recursive counting and receipt construction are disabled in BOTH
modes, including earlier discovery counters. Residual test flag/hooks and existing
debug output remain, limiting extrapolation. Public frequency, release latency and
causal CPU benefit are unknown. Logical bytes/capacities are not allocator traffic,
peak memory, inspected hash bytes or CPU instructions. No timing matrix expansion.

## User-owned verification and owner return

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-delta-previous-design.ps1
```

Expected nine tracked passes: five new design/work/order/public/cost controls plus
existing persisted restart, canonical epoch, superseded-publication and protected
diff-handler static controls. Require 240 paired public rows, eleven persistent
rows, two interleaving rows and sixteen cost rows; warning-fatal all-target/all-feature
Clippy, format/size/strict encoding and exact source identities. No optimization RED,
restoration mutation or mandatory composed gate for this design-only packet.

All runtime gates UNRUN. Agent performed bounded source/diff/parser/UTF-8 checks only;
no tests/builds/lints/formatter/runtime. Return FIX/DEFER/REJECT only after complete
executed evidence, with differences/regressions disclosed. At most recommend moving
this same capture within the existing guard. Any later repair requires owner approval,
independent actual-site original unused-clone budget RED with complete semantics
GREEN, current-byte GREEN, real production restoration mutation, exact restored GREEN
and composed fitness. No implementation, staging, commit or push under this directive.

## First attempt: two original-path fixture assumptions

delta-previous-design-20261109-115843 passes Clippy and three tests (including all
sixteen fixed cost rows), but fails original public/persistent assertions. Explicit
cache eviction legitimately produces cached=null before an unchanged operation and
a repopulated cache afterward. Canonical authority must stay unchanged, not every
cache field. The test now compares canonical/provenance/edge/fidelity authority and
requires exact current source in the repopulated cache; full before/after cache
values remain paired. The observer also uses the handler's admitted resolved path
verbatim, respecting Windows source-cache spelling as in the earlier diff controls.

The persistent structural request after a comment-only edit legitimately rejects
the old durable baseline with code -32603 and exact message Invalid session canonical
IR: Durable baseline does not match session canonical state. This is the original
ensure_persisted_baseline contract, not a candidate discrepancy. Both modes must
retain that exact rejection for structural/repeated structural phases; all other
phases still require success. No production/error-policy repair or normalized output.

The failed packet's completed cost samples and inspected pre-edit source bytes are
preserved in cost-inspection.json/inspected-sources. That snapshot was taken during
failure inspection, not represented as a run-time source receipt. Resume verifies
exact unchanged production/observation source, support code before identity capture,
and the entire local/timing code suffix. Only non-timed identity/assertion code changed.
It reuses one completed cost test and reruns eight semantic/protection tests, keeping
counts/source categories explicit. No repeated timing merely to seek better results.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-delta-previous-design.ps1 -ResumeCostDirectory ./target/af-perf-002/delta-previous-design-20261109-115843
```

Full acceptance remains UNRUN; prior three passes are not whole-packet GREEN.
Agent MaxHeadRoom performed bounded source/parser/UTF-8 review only, no runtime.

## Executed comparison and owner return

Agent MaxHeadRoom inspected delta-previous-design-20261109-124338: eight new tracked
passes, zero failed/ignored, plus the one preserved completed cost-test pass from
delta-previous-design-20261109-115843 after the exact unchanged timed-code checks.
These categories remain separate; the failed first packet is not whole-packet GREEN.
All 240 paired public, eleven persistent/restart, two controlled interleaving and
sixteen fixed cost rows complete. Public controls pass in 310.22s, persistent in
17.90s, errors/interleavings in 9.64s; existing restart/epoch/superseded-publication
and protected diff static controls pass. Warning-fatal all-target/all-feature Clippy
41.89s, both format logs empty, size five active text files and strict encoding 2297
text files GREEN. All 1404 recorded source hashes match before this documentation
update. inspection.json retains checked totals, all costs and exact identities.

Executed instrumented SHA-256 identities:

- core/delta.rs: fdb6744c3826d6311cac9441b91d29669c4ffcfa78c75ef8c93d7b9340eefc18
- delta_previous_design_af_perf_002.rs: f434d997e36bf4911d2f756a8de9a129f572b92c1e141148aff306d9e4fe0b2d
- delta_previous_cases_af_perf_002.rs: 8bb51e23dfe2a023bb6d8164d15156bdf4181266687a165fe25a957296dabfe8
- rolling_read_discovery_af_perf_002.rs: 04c48286599e391293ca2681cc26d44c072b18af126f1a4a65223d871a9ff1c1

The existing complete-handler guard verifies the protected diff source ownership
implementation despite shared-file test instrumentation. Other protected file bytes
match. No release optimization, mutation or composed gate executed.

### Exact semantic, currentness and required-work results

Original/candidate complete responses, errors, canonical version/tuples/hash, alias/
owner, current provenance/fidelity/edges, source-cache states and durable contexts
agree in every paired case. Explicit eviction still repopulates cache. Both modes
retain the existing persistent structural rejection after comment-only editing,
then recover through the same explicit publication/restart/restore path. That error
boundary is not optimized away or treated as a new correction.

Both controlled interleavings preserve exact results: a between-read source change
still forces changed processing; a newer actual registered publication before the
older commit still rejects the superseded operation. Required previous snapshots
remain identical, and publication checks/outcomes agree. No new lock, retry, read or
authority policy. Allocation-failure/abort ordering under resource exhaustion remains
outside this bounded equivalence claim, as disclosed above.

Selected unchanged High-fidelity work:

| Methods / repeated requests | Original discarded tuples / Strings / bytes | Candidate discarded copies | Required cached tuples, both |
| --- | --- | --- | --- |
| 1 / 1 | 5 / 16 / 62 | 0 | 5 |
| 8 / 1 | 26 / 86 / 307 | 0 | 26 |
| 64 / 1 | 194 / 646 / 2489 | 0 | 194 |
| 256 / 1 | 770 / 2566 / 10607 | 0 | 770 |
| 8 / 16 | 416 / 1376 / 4912 | 0 | 416 |

These fixtures export the class, adding one MOD_C tuple versus earlier discovery;
their exact counts must not be relabeled as the old 769-tuple population. Every warm
request removes one previous-authority copy, while both initial and second reads and
the REQUIRED cached copy remain. High-fidelity body padding does not enlarge retained
body tuples here. Edit fidelity does: eight long-body methods discard 34 tuples,
126 Strings and 197159 logical bytes in original, versus zero previous copies in
candidate; both still perform the required 34-tuple cached copy. At 256 Edit methods,
1026 tuples/3846 Strings/18512 logical bytes are avoided while the same required copy
remains. Changed/comment/structural/pending/rejected paths retain one full previous
capture per consuming request, with exact tuples/hash and matching publication checks.

Counts/capacities are logical copied work, not allocator events, peak live storage,
machine instructions or inspected hash bytes. Guard placement is source-controlled;
no mutex-event profiler or independent lock-cost measurement is claimed.

### Fixed construction-inclusive local costs

Milliseconds are original/candidate medians, retained from the first fixed packet
without retiming after non-timed assertion/observer corrections. Each mode has 24
alternating samples. Only High fidelity is timed; Edit cost benefit is not measured.

| Methods/body repeats/comment repeats | Unchanged ms | Comment-changed ms |
| --- | --- | --- |
| 1/0/0 | 2.14 / 2.17 | 808.26 / 812.56 |
| 8/0/0 | 2.76 / 2.57 | 829.83 / 816.38 |
| 64/0/0 | 6.54 / 6.33 | 865.99 / 867.35 |
| 256/0/0 | 20.25 / 19.66 | 1055.79 / 1056.55 |
| 8/64/0 | 5.34 / 5.28 | 822.05 / 817.87 |
| 8/4096/0 | 199.47 / 200.42 | 1371.29 / 1372.07 |
| 8/0/4096 | 27.32 / 27.82 | 883.43 / 881.97 |
| 64/64/0 | 27.96 / 27.73 | 1073.90 / 1055.13 |

Unchanged 256-method central ranges overlap: 20.12..20.88ms versus 19.52..20.31ms.
Fifteen of sixteen central ranges overlap. The eight-method unchanged case has
separated favorable ranges, 2.669..2.8712ms versus 2.5337..2.6249ms; that descriptive
local result does not establish statistical significance or a public speedup.
Long-body unchanged is slightly unfavorable with broad overlap: 198.88..204.26ms
versus 199.11..209.43ms. Changed-path work remains required in both modes and costs
are much larger than clone-only work. Original/candidate helper/presentation/compile/
publication/debug work remains in timing. No universal speedup, stable consequential
regression, public/release latency or allocator saving is established. No additional
samples or matrix expansion is recommended.

### Recommendation: FIX PROPOSED, one minimal private change

Move only the existing previous_authority capture below the unchanged-return branch,
before release of the same existing canonical guard. Preserve original predicate,
tuple/hash capture, previous version, changed-path equality and publication consumers.
Keep required cached construction and second read exactly where they are. The case
is deterministic elimination of unconsumed copying with positive authority/concurrency
controls, not a demonstrated public speedup. No implementation under this directive.

After owner approval, establish a separate genuine original unused-previous-clone
budget RED at the real production site, with frozen complete responses/canonical/
currentness/cache/persistent errors/lifetime/interleaving semantics GREEN before any
production edit. Require zero unused previous clones only on cached returns, while
separately requiring changed-path and cached-output work. Extend fidelity protection
as necessary before freezing; current paired matrix covers High/Edit, not all five.
Then minimal integration, unchanged-budget/current-byte GREEN, integrated cost review,
finite lock/capture/required-work protection, independent real early-capture restoration
RED with semantics GREEN, byte-exact restored GREEN and mandatory composed acceptance.
The design comparison is not original optimization RED. Preserve historical cost
evidence separately. All protected/deferred/rejected boundaries remain unchanged;
no production optimization, staging, commit or push follows this return.

## Owner approval and new original-only gate

Owner approves only the existing capture's minimal move within the same guard,
conditional on inspected genuine original RED. The all-five-fidelity original
baseline, independent actual cached-return discarded-snapshot budget and subsequent
acceptance sequence are tracked in AF_PERF_002_DELTA_PREVIOUS_REPAIR.md. Historical
design results above remain unchanged. No release behavior change precedes valid
original RED; no publication is authorized.
