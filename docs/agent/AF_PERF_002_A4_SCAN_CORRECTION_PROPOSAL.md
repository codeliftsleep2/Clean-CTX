# AF-PERF-002 A4: original-input lookup correction proposal

Agent: Agent MaxHeadRoom
Status: CLOSED / FIX / PROTECTED for the three approved original-input retrievals.
Original RED, semantic/work GREEN, focused gates, isolated production mutation
RED/restored GREEN and final composed verification executed. Historical stages below.
The revised owner directive supersedes passive MONITOR for these three scans.
No optimization budget RED or post-repair result is claimed yet.

## Existing index first: why direct reuse is insufficient

Source authority: src/ir/csharp_symbols.rs and
src/ir/csharp_resolution/compilation/dependencies.rs.
CSharpSourceSymbolIndex already supplies exact composition and resolver lookup.
Keep its single construction per obligation operation/partition unchanged.
Its resolved symbols retain cloned CSharpDeclarationPart records, not ClassInput
references. A part has key/kind/partial/file/span/base references, but no Scope,
class attributes or method attributes. Composition sorts parts by file/span.
Ambiguous composition retains a conflict classification rather than original inputs.
Neither adding a getter nor iterating resolved_symbols recovers the missing data
or original traversal order. File/span is not an established unique occurrence key.
Searching original inputs for every returned part would recreate the scan problem.

Adding original positions to this shared index would require changing its build
contract or maintaining another association. Borrowing private compilation inputs
through the general symbol index also broadens its ownership/lifetime boundary.
Neither is needed. Reuse the existing index for authority; use a private,
call-local borrowed lookup solely for original input retrieval.

## Exact scans and replacement

| Site | Information reconstructed today | Proposed retrieval |
| --- | --- | --- |
| collect_obligations seed filter | Sorted unique exact keys whose input.part.file equals seed | File lookup containing a BTreeSet of borrowed exact keys |
| visit family filter | Every original ClassInput with input.part.key equal to resolved key, in flattened units/classes order | Exact-key lookup containing Vec of borrowed ClassInput references in that same order |
| visit base find | First original matching-key input whose base_references contains the resolver's written base, using exact existing equality | Unchanged find predicate over only that key's ordered input vector |

Proposed private representation inside the compilation dependency module:

```rust
BTreeMap<&CSharpSymbolKey, Vec<&ClassInput>>
BTreeMap<&str, BTreeSet<&CSharpSymbolKey>>
```

Populate both during the original flattened input traversal already feeding
CSharpSourceSymbolIndex::build. Yield the same input.part.clone() to that builder;
do not build or clone the resolver index again. Both ancestry_obligations and
ancestry_obligations_for_files own their lookups until return; recursive visit
borrows them. No reference escapes, persistent state or interior mutability exists.
Group by full key, never name/file/span approximations. Do not iterate the family
map to determine output ordering. Seed iteration remains BTree key order; each
family vector remains original input order. Keep existing owned output construction.

The single-owner helper can borrow its sorted seed keys rather than cloning them
into a new set. The batch helper reuses the same file/key and family lookups for
every requested owner. Do not memoize complete ancestry results in this slice.

## Source-level equivalence proof

Let U be the unchanged flattened input sequence. For each key k, appending references
while traversing U constructs exactly filter(U, input.key == k), including duplicate
occurrences. Thus family iteration gives the same inputs and order as the old scan.
Applying the original base predicate to this subsequence selects exactly the same
first occurrence as find over U with the conjunction of key and base predicates.
Do not trim/normalize the base predicate: composition trims its chosen written
base, while the existing occurrence test uses exact contains; preserve even an
existing missing-match refusal rather than silently correcting it.

For each file f, inserting borrowed keys into a BTreeSet constructs exactly the
old sorted unique owned-key set by value. Borrowed Ord/Eq compare the same full key
values. Equal keys in distinct files are recorded for both files. Missing seeds
produce an empty iteration, not a fallback. Duplicated requested file entries keep
the existing outer traversal/BTreeMap result behavior.

The resolver index still gates visit before family/base consumption. Non-partial
duplicates, conflicting bases and kind ambiguity therefore take the same refusal
path. Valid partial declarations retain every original class/method attribute,
attribute ordering and original alias/import scope. Resolution still uses the same
index/external inputs. First base-bearing input selection does not adopt the
index's sorted-part order. No requirement record, reason or hint is deduplicated
beyond its existing BTreeSet behavior.

Cycle detection and recursive calls remain unchanged. Active-set state depends on
the path and repeated ancestry can intentionally reproduce ordered requirements.
ACCEPT path-sensitive traversal for this correction; that does not accept scanning
unrelated inputs to recover each visited key. No ancestry-result memoization is
proposed without an independent proof of requirement multiplicity and cycle behavior.

Only immutable local computation changes. Peer matching, conservative dependencies,
source hashes, stamp checks, acquisition passes, publication and locks remain at
their existing positions. The lookups borrow the same immutable units used to build
the original index. They do not certify currentness or reuse anything across calls.
Allocation/resource exhaustion is not claimed identical; new lookup allocation is
the explicit cost below. Existing semantic errors/returns remain unchanged.

## Construction-inclusive costs and alternatives

Executed baseline a4-dependency-work-20261106-131054 remains frozen: five tests
GREEN,57 rows,1318 matching hashes at inspection. At F2/C256, batch family tests
are65536 and seed tests512. No-ancestry family tests follow C*C; seed tests F*C.
This proposal removes unrelated class comparisons at those named sites, not all
quadratic dependency work. Peer F*F visits, text searches, resolver composition,
recursive multiplicity and matching family/base processing remain.

Construction adds C family appends and C seed-set insertion attempts, during the
existing C input traversal. Let K be distinct keys, P distinct files and S distinct
(file,key) pairs: retain C input-reference slots and S key-reference set entries,
K family vectors and map entries, and P file-map/set owners. Keys, paths, Scope,
attributes and ClassInput payloads are borrowed, not cloned. On a64-bit target the
reference payload alone is8*(C+S) bytes plus borrowed map keys; vector capacity,
BTree node occupancy and allocator overhead remain additional/unmeasured. Worst
construction comparisons are O(C log K + C log P + C log C), with full key/path
comparison cost retained. No claim of zero allocation or universal speedup follows.

For distinct no-ancestry declarations, family consumption becomes C matching-input
visits plus those C setup appends, instead of C*C predicate tests. Seed consumption
is selected keys plus map lookups/setup, instead of F*C predicate tests. Base lookup
still scans the selected family prefix, which is necessary to preserve first-match
semantics without a third maintained lookup. Partial density and ancestry depth
can keep repeated matching-input work; this is intentional bounded scope.

Direct existing-index access is rejected because data/order are missing. Extending
the shared index is larger scope. A flattened Vec alone still requires repeated
full scans. Two private borrowed maps are the recommended smallest complete
correction for family and seed selection. Small populations may cost more due to
map construction; measure demanding and small cases before acceptance. Do not add
a threshold or adaptive path without returning its evidence for review.

## Required execution after authorization

Before production edits, author tracked work budgets and execute genuine original
RED. Require no unrelated full-input seed/family/base predicates; count replacement
setup entries, lookup calls, matched-family/base-prefix visits and key comparisons
separately. Keep original characterization counters/evidence rather than hiding
cost under zero old counters. Baseline semantic/fixture failure is not valid RED.

Add a frozen old-algorithm oracle in tracked tests and compare complete obligation
terms/owners/reasons and ordered requirement identity_material records. Exercise
reversed input order, same-file repeated occurrences, cross-file partials,
non-partial/kind/base conflicts, first base-bearing occurrence with different alias
scopes, whitespace/no-match base, class/method attributes, alias shadowing, generics,
nested/namespace identities, chains, cycles, missing seeds and empty inputs. Public
tests must retain exact dependency records/multiplicity/current hashes and exact
C# call provenance, not merely resolver success. Do not relabel generic symbol or
alias unit tests as full obligation equivalence coverage.

Then implement only the approved private retrieval change, require unchanged
budget GREEN and exact old/new semantic equality. Compare complete operation work
including setup for independent F/C, partial density, depth and small/empty cases;
report descriptive timings and retained-reference/container costs honestly. Existing
warm reuse, retirement, ambiguity, failure/currentness and authority controls remain.
Add finite known-scan static protection; isolated real production restoration of
each old retrieval must fail budget/static while semantic controls remain GREEN,
then require exact restored GREEN. Run focused all-feature tests/lint/format/size/
encoding and mandatory composed gate through user-owned wrappers.

If lookup construction produces a meaningful measured regression, or equivalence
requires changing resolver/ownership/currentness behavior, stop for TRADEOFF REVIEW.
No tests/builds/lints/runtime were started for this design return. No production
change or pre-repair RED is claimed. No commit/stage/push performed.

## Canonical review and disposition

Public reachability: existing dirty-query batch and successful C# compression.
Authority: unchanged source-symbol composition/resolution and conservative peers.
Currentness: no cross-call reuse; source/hash/stamp checks unchanged.
Ownership: private references into immutable operation-owned units; outputs owned.
Completeness: preserve all matching occurrences; no truncation/caps/name shortcuts.
Adversarial/lifecycle acceptance: tests listed above are REQUIRED, not executed.
CBM acquisition remains degraded (frontend_retired timeout); bounded host source
reads supplied the trace. Provider failure is not absence evidence.

Disposition: FIX PROPOSED for operation-local original-input retrieval, ready for
owner authorization. Earlier passive MONITOR is superseded for these scans.
Other A4 risks and all closed/protected scopes stay separate. Census unchanged:
13/9763 scoped,9750 unresolved plus96 callback supplements.

## Approved baseline authored before production correction

Owner authorizes only the two private borrowed lookup structures and three retrieval
sites. Family Vec occurrence order and duplicates are mandatory; no ancestry
memoization or cycle change. No new approval is needed to implement that scope
after genuine executed RED, but runtime commands remain user-owned under AGENTS.md.

Tracked src/tests/ir/a4_original_obligations_af_perf_002.rs freezes the original
algorithm, including composition, seed ordering, full-family and exact base predicate,
recursive state and complete obligation identity. Its counters are reset before
measuring the production algorithm; oracle execution is outside measured intervals.
The original file is independent of the upcoming lookup implementation.

Tracked a4_lookup_budget_af_perf_002.rs compares full terms/owners/reasons and ordered
requirement identity strings for batch and single-owner paths. Thirteen semantic
families include duplicates/partials, reversed cross-file order, nonpartial/kind/base
conflicts, differing alias scopes, class/method attributes, shadowing, whitespace
base refusal, inheritance/cycles and generic/nested/namespace identities. Each also
runs with split compilation units, missing seeds and repeated requested files.
Ten construction-budget rows cover empty/single,2/8/64/256 classes, sparse/dense
partials and ancestry. Each compares frozen semantics before its measured interval,
prints all work rows, then aggregates only the intended full-input retrieval budget.

New observation fields reserve separate setup visits, three insertion attempts,
lookup calls/key comparisons, matching-family/base-prefix visits, retained references,
family vectors/file sets and family Vec capacity. They are zero before a lookup
exists. Repaired nonempty operations must expose construction, not just zero the old
scan counters. BTree node/allocator bytes remain distinct from retained payload.

The public budget reuses the actual19-fixture/57-row characterization and all its
dependency/hash/provenance/warm assertions, then requires zero old retrieval visits
in the final demanding public interval. This final-interval budget does not certify
all earlier public intervals as zero; their work rows remain separately available.
Construction budgets independently protect the three operation-local retrieval sites.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-a4-lookup-budget.ps1 -Phase Before
```

Expected one frozen-equivalence PASS, one construction-budget intended FAIL and one
public-budget intended FAIL, zero ignored. Driver checks exact failure markers,
one-test summaries and complete10/57 rows. Compile errors, oracle failures or missing
rows are invalid RED. It preserves source/test snapshots and hashes before execution.
No production lookup edit occurred; only test registration and observations were
added. Agent performed bounded parser/UTF-8/no-BOM/line review only; no test/build/
formatter/lint/runtime started. After verified RED, apply the approved retrieval
correction; unchanged budget GREEN, wider public/static/mutation and final composed
acceptance remain required. No FIX / PROTECTED claim yet.

### First Before attempt: compilation failure, not RED

a4-lookup-before-20261106-134716 stops before semantic execution: the frozen
algorithm retained a module-relative CSharpTypeIdentityOrigin path that no longer
resolves under its test-module location (E0433). Agent test-authoring error.
Corrected only that enum qualification to its absolute crate path; the enum and
algorithm are identical. No budget, semantic assertion or production retrieval
changed. Failed logs and source snapshots remain preserved. Size passed50 active
files, but no semantic/budget result or genuine RED exists from this attempt.
Rerun the same Before command. No agent Cargo/test/runtime was started.

### Genuine original RED verified; approved lookup correction applied

Agent MaxHeadRoom inspected a4-lookup-before-20261106-135919 before editing:
frozen equivalence passes0.23s; construction budget fails only its final intended
assertion after10 complete rows0.24s; public budget fails only its final intended
assertion after57 rows130.17s. Zero ignored. All1320 receipt source hashes match
at that inspection. inspection.json retains parsed construction/public rows and
test exits. The compilation-failed attempt stays separate, invalid RED evidence.

At C256/F16/distinct classes, original batch work records65536 family predicates,
4096 seed predicates, zero base predicates and7958us. At C256/F16/partial density4/
depth16 it records75776 family predicates,4096 seeds,360 base predicates and9013us.
These are baseline complete obligation-operation samples, not latency improvement.

Production changes only compilation/dependencies.rs and its new private
compilation/dependency_inputs.rs. Both original index-build streams now insert
borrowed inputs during the SAME flattened class traversal that yields the same
cloned declaration parts. OriginalInputs owns ordered family vectors and sorted
per-file seed sets; collect/visit retrieve those instead of full-input predicates.
The base search retains exact base_references.contains(written) over the ordered
selected family. All resolver composition, attribute/alias processing, active
cycle state, recursion, output construction and peer/currentness/publication code
remain unchanged. No CSharpSourceSymbolIndex changes or ancestry memoization.

Private Key is a reference-only Ord wrapper delegating directly to original key/
str comparison. Its const site tag carries no runtime data; cfg(test) counts actual
BTree comparisons separately for family/file/seed keys. Counts include setup AND
query comparisons; they are not inferred logarithmic bounds. Setup records each
input and all three insertion attempts; storage observation records actual retained
input/seed references, family vectors/file sets and Vec capacities. Test bookkeeping
cost contributes to measured elapsed time. Allocator/node overhead remains unmeasured.

Original construction-budget, frozen-oracle and public-test sources remain exactly
the executed Before bytes. After driver checks these three sources against the
frozen successful Before receipt before executing. It also formats the two changed
production files. No After test/build/lint/runtime was started by the agent.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-a4-lookup-budget.ps1 -Phase After
```

Expected three tracked passes: complete frozen equivalence, unchanged construction
budget and unchanged public budget. Compare full10 setup-inclusive rows and57 public
rows to original; a semantic mismatch or consequential cost remains a stop condition.
Permanent static/isolated mutation protection, existing lifecycle/failure controls,
focused lint/format/size/encoding and mandatory composed gates remain required
before final FIX / PROTECTED. No commit/stage/push performed.

### First After attempt: private borrow signatures corrected

a4-lookup-after-20261106-141549 stops at compilation with E0700 and a returned
family-reference lifetime mismatch. No semantic or work-budget test executed.
The reference-key BTree query reborrows stored values to the shorter query/receiver
lifetime; the new method signatures had incorrectly implied a longer returned
borrow. Corrected seeds to name the shared receiver/query lifetime and explicitly
capture the stored-input lifetime in its opaque iterator; family now returns a
slice of references reborrowed to that same shorter lifetime. No allocation,
stored-reference lifetime, output ownership, lookup predicate or lock changes.
This is an implementation signature error, not evidence of semantic mismatch.
Original frozen oracle/budgets remain byte-identical; failed logs retained.
Rerun the same After command; all post-repair gates remain unverified.

## Repaired budget and semantic equivalence verified

Agent MaxHeadRoom inspected a4-lookup-after-20261106-142530: three tracked passes,
zero failed/ignored. Frozen equivalence0.03s, construction budget0.35s, public
budget135.97s. All1321 recorded source hashes match at inspection; original two
budget sources and frozen oracle are unchanged. inspection.json retains ten paired
construction rows and57 new public operation rows. Size passes51 active files;
Clippy/complete formatting/encoding are not yet claimed from this budget packet.

| Classes / owners / partial density / depth | Old seed / family / base predicates | New input setup / key comparisons / matching family / base prefix | Public-helper us before / after |
| --- | --- | --- | --- |
| 0 /1 /1 /0 | 0 /0 /0 | 0 /0 /0 /0 | 6 /14 |
| 1 /1 /1 /0 | 1 /1 /0 | 1 /2 /1 /0 | 46 /45 |
| 2 /2 /1 /0 | 4 /4 /0 | 2 /8 /2 /0 | 67 /69 |
| 8 /2 /1 /0 | 16 /64 /0 | 8 /89 /8 /0 | 231 /228 |
| 64 /8 /1 /0 | 512 /4096 /0 | 64 /1610 /64 /0 | 1191 /1927 |
| 256 /16 /1 /0 | 4096 /65536 /0 | 256 /8006 /256 /0 | 7958 /4144 |
| 64 /8 /8 /0 | 512 /4096 /0 | 64 /1108 /512 /0 | 1298 /680 |
| 64 /8 /64 /0 | 512 /512 /0 | 64 /443 /512 /0 | 456 /399 |
| 64 /8 /1 /8 | 512 /6400 /204 | 64 /2286 /100 /36 | 2049 /1765 |
| 256 /16 /4 /16 | 4096 /75776 /360 | 256 /7453 /1184 /40 | 9013 /3710 |

These are complete obligation-helper timings, not public latency, and include test
comparison-counter overhead. The table's old counts and new counts denote different
work units; do not sum them as equal CPU-cost operations. Each new input makes three
insertion attempts. At C256/distinct, storage is256 input references,256 seed
references,256 family Vecs,16 file sets and1024 family pointer-capacity slots.
At C256/density4 it is256/256 references,64 Vecs,16 sets and256 pointer slots.
Empty creates no map entries. These are actual logical capacities/container counts,
not allocator bytes. Public sums across acquisition helpers are cumulative, not
simultaneous peak memory. Exact existing public dependency/hash and call-provenance
oracles pass all57 intervals; this does not newly assert dependency ordering.

The slower C64 distinct sample warrants further comparison before acceptance.
New paired cost test alternates frozen-original/repaired execution,24 retained
samples after warmup per8 small/large/dense/ancestry cases. A scoped test-only
observer pause disables update bookkeeping and restores it on panic/return; the
production algorithm and all allocations still execute. Both outputs are compared
for exact complete identity outside timing. This is debug in-process helper
evidence, not enterprise/public latency or allocator attribution. No timing threshold,
adaptive production path or latency benefit is claimed before execution.

## Permanent acceptance assets authored, UNRUN

New tracked a4_lookup_acceptance_af_perf_002.rs adds finite AST/call classification
for these two compilation files and paired cost evidence. Static gate requires
two original index builds, original-input insertion at both, known lookup consumers,
exact base predicate and ordered borrowed family Vec insertion/retrieval. It rejects
restored units traversal in collect/visit and self-mutates missing lookup/occurrence
contracts. It is a finite known-sink gate, not a global ban on loops or indexes.

Isolated mutation driver overlays exact Git-inventoried source into target and
independently restores seed, family and base retrieval. Each requires original
budget RED plus static RED, while frozen semantics stays GREEN. Only mutated
isolated lookup storage exposes original units; active production never does.
Seed mutation retains the actual lookup call as unused work then restores full scan
selection, so bookkeeping remains honest and the final full-scan budget is the
intended failure. Each variant restores exact bytes before the next. finally also
restores on error. Final restoration requires five passes including public budget;
compiler/fixture failures cannot satisfy mutation RED. Active source hashes checked.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-a4-lookup-focused.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-a4-lookup-mutation.ps1
```

Run sequentially; proceed to mutation only after focused succeeds. Focused expects
nine tracked passes including static/cost, all57 public intervals, existing reuse/
removal/ambiguity/failure controls, then warning-fatal all-target/all-feature Clippy,
format/size/encoding. Mutation expects six intended failures, three semantic
preservation passes and exact restored five GREEN. Both assets UNRUN. Agent performed
only bounded PowerShell parser/text/UTF-8/line checks; no Cargo/runtime started.

Mandatory composed fitness now includes the five A4 protections, expected at least
70 total passes. That final user-owned command follows inspected focused/mutation
GREEN. Existing protected scopes and source census remain unchanged. No additional
production retrieval change after repaired GREEN; test diagnostics/registration,
acceptance scripts, composed selector and documentation were added in this return.

### First focused attempt: static call classification corrected

a4-lookup-focused-20261106-143527 passes unchanged unit budgets/equivalence2tests
0.47s and public budget1test126.75s. Paired cost test passes, but the static test
fails: AST call traversal also observes inputs.family(key).iter, its copied call
and its find call. Those exact existing ordered-base-search chains were omitted
from the expected call inventory. This is a static-test authoring error, not a
production semantic/budget failure. Added only those three exact call classifications;
unknown chains still fail. A reversed-family self-mutation now explicitly checks
that rejection. No production algorithm, frozen oracle or budget change.

All eight paired cost fixtures pass24 alternating samples after warmup with
observation updates disabled. Old/new median ns: empty15900/13400; single30100/28700;
C8 distinct134400/128200; C64 distinct1125200/962500; C256 distinct11807600/5837500;
C64/density8 1065100/742800; C64/density64 708300/431300; C256/density4/depth16
9694300/3845200. This bounded debug helper result does not show a consequential
construction regression in these fixtures; it does not establish public speedup.
Original single slower observed-counter sample remains retained, not discarded.

Focused halted before the remaining lifecycle controls, Clippy, final formatting
and encoding; no successful receipt. The simultaneously started isolated attempt
a4-lookup-mutation-20261106-143850 had no completed baseline log/receipt at inspection
and contains the old static-test snapshot. No mutation result is claimed. Do not
treat it as acceptance; rerun mutation only after corrected focused succeeds.
Focused now selects the cheap static/cost group first to expose guard failures
before the expensive public packet. Gates remain UNRUN after this correction.

### Second focused attempt: all tests GREEN, oracle module sharing corrected

a4-lookup-focused-20261106-144408 passes all nine tracked tests: static/cost2
0.88s, unit2 0.31s, public1 155.57s, retained reuse0.98s, removal4.01s,
ambiguity4.08s and preparation failure0.71s. Clippy then rejects loading the same
frozen oracle file as two separate modules. No suppression added. Acceptance now
imports the existing oracle from the budget test module, whose declaration alone
changes from mod original to pub(super) mod original. Exact comparison to original
Before verifies this is the sole budget-source change; every fixture, observation,
assertion and original oracle algorithm remains unchanged. After wrapper permits
only this exact visibility change and retains strict checks for the other sources.
No production algorithm change. This routing correction cannot manufacture RED.

a4-lookup-mutation-20261106-145105 passes clean static baseline, but its first
isolated seed mutation fails compilation E0451: parent code attempted struct-update
initialization of private lookup fields. This is invalid mutation RED. Mutation-only
initialization now lives inside the isolated helper's impl, where those fields are
accessible. Active production gains neither the original-units field nor constructor.
Failed isolated source was restored by finally; no successful mutation receipt.

Paired cost test again favors repair across all eight bounded cases; C256 distinct
old/new medians7575300/4048700ns, density4/depth16 8434500/3738700ns. Do not merge
this run with previous samples as one controlled distribution. Focused did not
reach final format/encoding and has no complete receipt. Rerun focused, then mutation
only after success. Clippy now precedes tests to catch compile/lint errors before
the expensive public run. No agent tests/builds/lints/runtime executed.

## Focused and isolated mutation acceptance verified

Agent MaxHeadRoom inspected a4-lookup-focused-20261106-150155: nine tracked passes,
zero failed/ignored. Unit frozen/work2 0.36s, public1 133.99s, static/paired-cost2
0.94s, retained reuse0.78s, removal4.07s, ambiguity4.42s and preparation failure
0.70s. Warning-fatal all-target/all-feature Clippy38.43s, empty format logs,
size54 active files and strict encoding2150 text files pass. All1324 receipt
source hashes match at inspection. inspection.json retains paired cost rows.

Final focused paired medians (old/new ns;24 retained alternating samples each):

| Classes / partial density / depth | Original | Corrected |
| --- | ---: | ---: |
| 0 /1 /0 | 6900 | 5900 |
| 1 /1 /0 | 37200 | 35400 |
| 8 /1 /0 | 135200 | 124700 |
| 64 /1 /0 | 1091600 | 926100 |
| 256 /1 /0 | 8023300 | 4103000 |
| 64 /8 /0 | 892300 | 642300 |
| 64 /64 /0 | 463600 | 413000 |
| 256 /4 /16 | 10170100 | 4222500 |

This paired construction-inclusive helper evidence finds no consequential regression
in these bounded cases. It preserves the full operation, allocations and outputs,
with observation updates disabled. It does not establish public-operation speedup,
allocator bytes, release-mode performance or arbitrary project bounds. The earlier
slower instrumented C64 sample remains retained and qualified separately.

a4-lookup-mutation-20261106-150830 passes clean static baseline0.10s. Independent
seed/family/base production restorations each fail only the intended construction
budget and static protection, with frozen semantics passing0.03/0.04/0.03s.
Budget RED durations0.56/0.44/1.20s; static RED0.01s each. All six exact failure
markers and one-test failure summaries verified; compile errors are not counted.
Exact restoration passes unit2 0.41s, static/cost2 0.81s and public1 127.91s,
zero failed/ignored. All2157 inventory hashes match at inspection. Active and
restored isolated production files match byte-for-byte. No failed-attempt log is
reclassified as valid mutation evidence. inspection.json records these identities.

Production SHA-256 at this acceptance:

- compilation/dependencies.rs:
  5815dfe122e02664777e4d378191d2fc3fc35d140ce2d271589ffca7c2f91cd6
- compilation/dependency_inputs.rs:
  ee17ac3f4166eb3715185d94b86909a5af001119c1ed618caba084b6991a7a52

No production/test edits follow these runs; documentation only. Original oracle
algorithm and both budget assertions remain intact; the previously recorded single
module-visibility routing change is the only difference from original budget bytes.
Identity/refusal/order/duplicate/alias/attribute/cycle controls and public dependency/
hash/provenance/lifecycle checks pass. Peer matching, currentness, index construction
frequency and all protected scopes remain unchanged.

Final mandatory user-owned composed gate, UNRUN in this return:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

Expected at least70 tracked passes including five A4 protections. Final bounded
FIX / PROTECTED disposition awaits composed GREEN and source identity. No additional
field/binary/Claude gate is proposed for this internal retrieval correction.
No agent test/build/lint/runtime, stage, commit or push performed.

## Final composed verification: CLOSED / FIX / PROTECTED

Agent MaxHeadRoom inspected af-perf-001-20261106-152751:70 tracked passes across
20 groups, zero failed/ignored. All236 recorded source hashes match current bytes.
The five A4 protections pass alongside all existing mandatory protected groups.
inspection.json records group results and exact production identities, unchanged
from focused acceptance and exact restored isolated source:

- dependencies.rs:5815dfe122e02664777e4d378191d2fc3fc35d140ce2d271589ffca7c2f91cd6
- dependency_inputs.rs:ee17ac3f4166eb3715185d94b86909a5af001119c1ed618caba084b6991a7a52

Acceptance chain remains separated: original a4-lookup-before-20261106-135919
genuine two-budget RED with semantic GREEN; repaired a4-lookup-after-20261106-142530
three-test GREEN; focused a4-lookup-focused-20261106-150155 nine-test and warning-
fatal lint/format/size/encoding GREEN; isolated a4-lookup-mutation-20261106-150830
six intended failures, three preserved semantic passes and exact restored five
GREEN; final composed70/236 GREEN. Failed compile/static/Clippy/harness attempts
remain documented and are not substituted for valid acceptance.

Final disposition FIX / PROTECTED is limited to file seed discovery, exact-key
original family retrieval and base-bearing search over that ordered family.
At C256/F16 distinct classes, full-input seed/family predicates4096/65536 become
zero, with256 setup visits,768 insertion attempts,8006 measured key comparisons
and256 necessary matching-input visits explicitly retained. Added operation-local
borrowed storage and its actual logical capacities are recorded above; no hidden
zero-cost index claim. Dense partial/ancestor matching work, recursion/path-sensitive
requirements, existing index construction, peer comparisons/text discovery,
conservative source-hash/currentness obligations and publication remain intact.

No meaningful construction tradeoff was demonstrated in the bounded paired helper
fixtures; no public/enterprise latency or allocator-total claim is made. This does
not eliminate all quadratic C# work or certify every authority/concurrency scenario.
No further A4 repair is implied for other work units. Protected/closed scopes stay
closed. Census remains13/9763 scoped,9750 unresolved plus96 callback supplements.
No code/test changes after acceptance; this closure updates documentation only.
No agent runtime, stage, commit or push performed. Repository publication remains
separate from completed verification.

```architecture-graph
node: DOC-142
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Accepted bounded CSharp original-input retrieval correction
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_A4_SCAN_CORRECTION_PROPOSAL.md
preserves:
  - SEMDEP-001
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
```
