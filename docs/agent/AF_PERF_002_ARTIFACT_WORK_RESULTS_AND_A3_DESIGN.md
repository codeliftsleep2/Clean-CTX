# AF-PERF-002: artifact work measurements and bounded A3 proposal

Agent: Agent MaxHeadRoom
Status: A3 implementation, focused preservation, budgets/static and mutation proof GREEN.
Bounded acceptance return below; whole-system discovery and broader gates remain open. Generation 78 semantics remain baseline.

## Exact authorized execution and evidence

Owner explicitly authorized this run, overriding the normal operator handoff for
this specific command only:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-artifact-work.ps1
```

Agent executed exactly that packet. artifact-work-20261104-093758 passes:

- New public-query characterization: one tracked test, fourteen dimension cases,
  each with dirty and warm phases; 12.39s.
- retained_artifacts_project_cross_file_calls_without_reparsing: one pass, 0.88s.
- missing_artifacts_are_compiled_once_then_reused: one pass, 1.53s.

Zero failed/ignored. Main-package all-target/all-feature warning-fatal Clippy
finishes cleanly (0.55s); scoped format and format-check logs empty; active sizes
pass; strict encoding passes 2045 text files. This is not a whole-workspace/proxy,
binary or final field certification. All **1283 recorded file hashes match** at
inspection. No Rust source/test changes follow that verification in this return.

Evidence: target/af-perf-002/artifact-work-20261104-093758/{receipt,inspection}.json
and three test logs, clippy.log, fmt.log, sizes.log, encoding.log. inspection.json
contains 28 parsed dirty/warm rows; original logs are retained. Times are descriptive
single small-fixture observations, not enterprise benchmarks or attribution.

## Independent invocation growth: actual catalog D=3

C is the requested and observed matched resolution count. Extra declarations=1
means one unrelated Target.Other method; actual catalog also contains Run and Save.
Every dirty row has two artifact generation projections, one reconciliation,
zero full semantic parses, and zero extension predicate visits.

| C | Actual D | Direct predicate visits | Logical IR instructions cloned | Dirty query us |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 3 | 3 | 13 | 87830 |
| 2 | 3 | 6 | 14 | 87016 |
| 4 | 3 | 12 | 16 | 97187 |
| 8 | 3 | 24 | 20 | 92559 |
| 16 | 3 | 48 | 28 | 107869 |
| 64 | 3 | 192 | 76 | 109174 |
| 256 | 3 | 768 | 268 | 231863 |

## Independent unrelated-declaration growth: C=1

The baseline row above is shared, not counted as a second fixture. Actual admitted
catalog size is printed independently of the source generator's requested count.

| Requested unrelated methods | Actual D | Direct predicate visits | Logical IR instructions cloned | Dirty query us |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 3 | 3 | 13 | 87830 |
| 2 | 4 | 4 | 16 | 136037 |
| 4 | 6 | 6 | 22 | 92724 |
| 8 | 10 | 10 | 34 | 93793 |
| 16 | 18 | 18 | 58 | 138646 |
| 64 | 66 | 66 | 202 | 90755 |
| 256 | 258 | 258 | 778 | 179755 |

Joint fixture: requested C=16 and unrelated methods=16, actual D=18. Observed
16 resolutions, **288 direct visits**, two owner projections, one reconciliation,
73 logical IR instructions cloned, zero full parses/extensions; 107554us.

Across all fourteen dirty fixtures, direct visits equal **C*D** exactly. One Save
declaration is eligible in the constructed direct bucket; Other methods and Run
cannot select the requested target. The irrelevant-key conclusion follows from
the known fixture/catalog predicates; no separate matching-item counter was added.
The measurements establish amplified candidate work, not its share of query time.

## Warm controls and exact semantic authority

All fourteen warm repeats observe zero complete artifact projections, resolutions,
direct/extension visits, logical IR instruction clones, full parses and reconciliations.
Warm times range 1332..2128us; dirty times range 87016..231863us. These are different
lifecycle boundaries: the test explicitly dirties the first generation, then leaves
the second unchanged. No claim that ordinary warm queries reconcile is supported.

Both phases in every fixture pass the public builtin query response control and
an independent published-index oracle requiring:

- csharp-exact-call layer and Calls relation;
- exact csharp Method identity Caller::Run(Target)->void to Target::Save()->void;
- asserting caller file provenance, zero written arity and no spread.

Generic Run/Save output alone cannot satisfy the exact oracle. This proves exact
selected publication after the actual public trigger, not a new public exact-query
contract, callable certification or target-declaration-file attribution policy.

## A1-A4 classifications after measurement

| Unit | Actual outcome | Revised classification / disposition |
| --- | --- | --- |
| A1 publication accessor cloning | accessor calls/edge/dependency items are [0,0,0] in every measured phase; initial hydration is outside the interval | UNRESOLVED / MONITOR. Zero accessor calls do not mean zero cloning. Real publication ownership/copy volume still needs its own boundary. |
| A2 enclosing generation projection | dirty: two retained owners project once each; one reconciliation; cloned instruction items grow with fixture IR. Warm: all instrumented work zero | STRUCTURAL RISK / MONITOR beyond characterized two-owner behavior. No measured owner-count multiplier or per-role class-visit budget. No cross-owner sharing proposal follows. |
| A3 per-invocation declaration filtering | C*D direct predicate visits, unchanged exact semantic result; no extension or ambiguous case exercised | MEASURED DEFECT at candidate-work level / FIX PROPOSED. Repeated irrelevant-key visits are demonstrated; consequential wall-time fraction and allocation crossover remain unmeasured. |
| A4 dependency footprints | invoked in enclosing reconciliation, but owner/peer/term/source-byte work not separately counted here | STRUCTURAL RISK / MONITOR. Existing once-built source-symbol index must be preserved; no freshness/stamp change. |

Candidate-work defect is not a claim that the proposed data structure is faster
for every small input. One invocation still requires learning the supplied catalog;
hashing/allocation can outweigh saved comparisons. Before/after timing and memory
must evaluate this tradeoff without arbitrary global thresholds or input caps.

Clone counters count logical items at their instrumented boundaries, not bytes,
allocator events or all CompiledIR data. Reconciliation additionally clones projected
edges/dependencies/hashes during index replacement; that is not measured by the
published_semantics accessor counter. Thread-local observations cover synchronous
dispatch/reconciliation, not un-attributed compilation workers.

## Proposed smallest A3 correction: lazy borrowed buckets per projection

This is a design return under characterization-only authorization; **do not implement
until owner disposition**. Preserve the exact immutable declaration slice as authority.
Use private concrete call-local lookup state within one project_with_authority call:

1. Direct bucket key: exact borrowed (owner, name, parameter count).
2. Extension bucket key: exact borrowed (name, parameter count), containing only
   currently eligible static declarations. Keep the existing first-parameter versus
   receiver normalize_type comparison at selection; no new normalization policy.
3. Retain every declaration occurrence in original supplied order as borrowed
   references. Duplicate records must remain competing authority, not deduplicate.
4. Build once, lazily after a call has valid caller/occurrence/receiver and no spread.
   Zero lookup construction for projections with no eligible resolution demand.
   Reuse for ordinary and property-initializer resolution only within that call.
5. One direct candidate succeeds; multiple direct candidates refuse immediately.
   Extension fallback runs only if the direct bucket is empty. Preserve all existing
   owner/name/arity/static/normalized receiver rules and unsupported/refusal paths.

No state survives the project invocation. No generation-wide/owner-wide sharing,
authority cache, witness context, registry, discovery, identity/equality, lifecycle,
public API or durable format change is proposed. This narrows candidate selection;
it does not grant currentness or replace source/evidence checks.

Expected work is one D-element build plus actual demanded lookup/bucket work,
including key hashing/string lengths and necessary extension ambiguity evaluation.
Do not claim universal O(C): dense eligible extension buckets still require their
existing checks. Direct cardinality can be read from a duplicate-preserving bucket;
it must never select first/last among competitors. Across F demanding owners, local
builds can still cost F*D. That outer dimension is excluded from this finite repair.

Memory: maps hold borrowed string keys and declaration references, not cloned
declaration/string payloads; map/bucket headers and O(D) references still allocate.
Report capacity/payload and small-C tradeoff, not unmeasured allocator bytes.

## Required acceptance if the design is approved

- First author intended deterministic budget RED against current full scans,
  preserving original characterization as GREEN observation, not retroactive RED.
- Count complete lookup builds, declaration build visits, lookup probes and actual
  bucket predicate visits. Enclosing public-query totals must include construction,
  not only cheaper post-index lookups. Reuse the current independent axes/joint case.
- Zero builds for unreachable/unsupported/no-call branches; at most one per selected
  project invocation; no state reused in a separate call or changed authority.
- Retain duplicate/direct ambiguity refusal, direct-over-extension precedence,
  direct-negative/absent cases, extension normalization/static/arity constraints,
  property-initializer forms, exact identity/provenance, spread/unsupported refusal.
  Current positive measurements do not certify these cases.
- Extend finite AST caller/sink review; blocking new/unclassified selection paths;
  isolated real full-scan regression must fail work budget and relevant static rule,
  then exact restored source must pass. No blanket scan prohibition or registry.
- Compare fixed fixture before/after counts, time and memory, including C=1 and dense
  competing buckets; run affected existing exact-call and lifecycle tests/lint.

Agent Popper independently supports the measured local interpretation and the
conditional bucket proof equivalence above. Popper specifically requires retained
duplicates, direct precedence, no new extension owner condition, unchanged receiver
normalization, and counting build visits. This is not production implementation
approval, warm-cache authority proof or whole-inventory acceptance.

## Independent coverage indicators and remaining work

Syntax: 9763 frozen original records reconciled; 13 scoped syntax outcomes and
9750 macro/review-required records, plus 96 explicit all-callback supplements.
The accepted parser syntax limitation remains closed; original flags are historical
evidence. Instrumentation source drift stays explicit, not erased by a new run.

Production: A2's two-owner dirty/warm boundary and A3's independent invocation/catalog
axes are now measured, including exact publication. A1 and A4 are not characterized
by zero counts or total elapsed time. Macro/test-scope/source ownership grouping,
other significant call chains and final independent coverage review remain open.

If A2 work merits prioritization, next characterize retained-owner count and resolved
class population independently, with per-role visits and unchanged warm controls.
Do not infer consequential F multiplication from this fixed-two-owner result.
No owner-count run or further production repair was started. The declaration
membership design and this A3 design remain separately awaiting owner disposition.


## A3 baseline RED and independent A2 execution return

Agent MaxHeadRoom inspected operator receipts after both commands completed.
A3 baseline `a3-budget-before-20261104-101302`: one populated-catalog/no-call
control passes; two intended budget tests fail, zero ignored, 4.23s. Original
C8/D3 work24 exceeds budget22; C16/D3 work48 exceeds budget38. This is genuine
tracked pre-repair RED, separate from the earlier characterization GREEN.

A2 `a2-owner-class-20261104-101547`: new characterization passes (11.52s),
plus both existing artifact preservation tests. Warning-fatal all-feature/all-target
Clippy passes (27.31s), format and size gates pass, encoding passes2050 files.
Receipt comparison after A3 editing differs only in src/ir/csharp_exact_calls.rs;
these are frozen pre-repair results, not current post-repair lint authority.

| Owners | Classes | First-stage visits per consumer | Dirty query us |
| ---: | ---: | ---: | ---: |
| 2 | 32 | 64 | 93992 |
| 4 | 32 | 128 | 164698 |
| 8 | 32 | 256 | 331458 |
| 16 | 32 | 512 | 614363 |
| 2 | 2 | 4 | 87642 |
| 2 | 4 | 8 | 85355 |
| 2 | 8 | 16 | 90874 |
| 2 | 16 | 32 | 86865 |
| 2 | 64 | 128 | 87738 |
| 2 | 256 | 512 | 100748 |

All six first stages observe F*C visits; across consumers this fixture totals
6*F*C. Each dirty phase has one reconciliation and zero full semantic parses.
Every warm repeat has zero role entries/class visits/projections/reconciliations
and full parses (1342..1549us). Exact public-trigger and published C# identity
oracles pass. Ordinary classes exercise negative role scans; positive framework
roles, partial joins and duplicated ownership remain uncharacterized. Timings do
not isolate class traversal latency. A2 remains DESIGN REVIEW for bounded local
traversal opportunities, not approval for cross-owner sharing or a repair.

After preserving baseline RED, A3 production correction is authored: explicit lazy
borrowed direct/extension maps owned by one project_with_authority invocation;
ordinary/property forms share that local slot. Construction occurs after receiver
and spread validation. Duplicate order/cardinality, direct precedence and original
extension normalization remain intact. Construction/probe/candidate counters and
logical capacity payload accounting cover the new work. No string/declaration
payload ownership is cloned into buckets. This is UNRUN implementation, not A3
acceptance: semantic/refusal extensions, static/mutation enforcement, timing/memory
comparison and refreshed affected lint/lifecycle evidence remain required.

Next narrow operator command:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-csharp-a3-budget.ps1 -Phase After
```

Expected three tracked budget passes; compilation failure is not valid fitness
proof. No agent Cargo/formatter/lint/runtime execution occurred in this return.
Syntax accounting remains unchanged; measured A2 production coverage advances
independently. A1/A4 and broader inventory remain open.


## First post-repair budget GREEN; semantic packet authored

Operator run a3-budget-after-20261104-102137 passes three tracked tests,
zero failed/ignored, 11.34s. Every demanding projection builds once; populated
no-demand control passes. D3/C256: build visits3 + construction probes3 +
lookup probes256 + candidates256 =518 versus original768 predicate visits.
D18/C16:18+18+16+16=68 versus original288. D258/C16 total548;
logical capacity payload36928 bytes. D3 payload288 bytes; D18 payload2368.
No cloned declaration/string payload is retained; these are capacity calculations,
not allocator measurements.

Small C1/D3 now costs8 counted units versus original3 predicate visits, so a
comparison reduction is not universal. Observed93924us versus original87830us
is a single uncontrolled sample, not evidence of a consequential latency regression.
Joint fixture85697us versus107554us and C256/D3 208125us versus231863us
likewise do not establish causal latency improvements. Full C1/catalog timing
and dense competing controls remain part of acceptance.

Three new UNRUN semantic tests cover direct precedence, extension success/ambiguity,
direct ambiguity refusing extension fallback, and duplicate-preserving borrowed
bucket order with fresh authority. Existing four exact-call tests remain unchanged.
The source receipt now differs only in that newly edited semantic test file;
post-repair budget source/production hashes match. No new runtime was started.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-csharp-a3-preservation.ps1
```

This narrow packet runs seven exact-call controls, the fourteen-case public-query
characterization and two retained-artifact controls, then affected lint/format/size/
encoding. Static coverage extension and isolated production mutation proof remain
pending; this budget GREEN does not close A3. A2 is unchanged characterization-only.


## Preservation GREEN and enforcement handoff

Run a3-preservation-20261104-103037 passes seven exact-call tests (5.36s),
public-query characterization (10.74s), retained cross-file no-reparse (0.83s)
and missing-artifact reuse (1.36s): ten tracked passes, zero failed/ignored.
Warning-fatal all-target/all-feature Clippy passes35.76s, format logs empty,
size21 active files pass, strict encoding2051 files passes. The earlier
102621 attempt failed at Windows log-path creation before tests; it is not
semantic RED. The corrected wrapper preserves the exact Rust filter and uses
a Windows-safe log basename.

New UNRUN finite AST enforcement inspects the exact-call module: full-catalog
selection loops/iterators in resolve fail; new declaration-slice consumers,
authority aliases and delegated selection calls require classification. The
DeclarationLookup construction sweep remains legitimate and measured. This is
bounded source-aware enforcement, not type-resolved whole-program proof or a
universal scan ban. Mandatory fitness composition now includes its four A3 tests.
Existing source accounting and broader macro/alias uncertainty remain explicit.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-csharp-a3-fitness.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-csharp-a3-mutation.ps1
```

Run sequentially. First requires four static/budget passes and refreshed focused
gates. Second clones/overlays exact tracked and untracked nonignored sources into
an isolated target workspace, restores the original RED full-scan resolve body,
and requires both deterministic and static failures. It restores exact source
bytes and requires four clean passes; the active checkout is never mutated.
The frozen original baseline source is required, not silently regenerated.
No agent test/build/lint/runtime was run. Production source remains unchanged
since the post-repair budget/preservation passes. Static tests, wrapper and gate
composition are newly authored; their GREEN and mutation proof are pending.
Dense extension performance and wider unsupported/normalization adversarial
controls remain acceptance gaps; these passes alone do not close A3.


## Static/deterministic sensitivity verified; final narrow gaps authored

Fitness a3-fitness-20261104-104050: four tracked passes, zero failed/ignored,
11.42s; warning-fatal all-feature/all-target Clippy23.70s, empty format logs,
26 active files size-pass, strict encoding2054 files passes.
Mutation a3-mutation-20261104-104227: isolated original full-scan restoration
causes exactly two intended budget failures and one intended static failure,
with the no-demand test passing (4.55s). Exact source restoration then gives
four passes, zero failed/ignored (11.74s). Receipt records active source unchanged
and isolated source restored. This proves sensitivity for this finite A3 invariant,
not every C# authority path or whole-inventory architectural acceptance.

Production code remains unchanged. Newly authored gap tests extend C1/catalog
public-query counts/timing/memory and dense static extension competition through
1/2/4/8/16/64/256, plus static/arity/receiver negative controls. Dense timings
cover production compilation, not the public-query boundary; labels retain this
distinction. These tests are UNRUN and source drift is limited to the two edited
test files. No new optimization or adaptive threshold is introduced.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-csharp-a3-gaps.ps1
```

Expected three new gap tests and four existing fitness passes, then focused
lint/format/size/encoding. Existing exact identity/precedence/property/ambiguity
and lifecycle passes remain frozen preservation evidence while production stays
unchanged. No full historical campaign is requested. A3 remains open until these
small-workload/dense controls and any remaining semantic obligations are reviewed.
A1/A2/A4 and systematic discovery remain separate and open.


## Consolidated A3 measured return after gap packet

Agent MaxHeadRoom inspected a3-gaps-20261104-105659: three gap tests13.79s
and four unchanged deterministic/static tests13.93s, seven tracked passes,
zero failed/ignored. All recorded source hashes match at inspection. Clippy
all-target/all-feature -D warnings26.90s, empty format logs, size27 active files
and strict encoding2055 files pass. Production selection has not changed since
the earlier preservation and mutation receipts. No agent runtime was started.

| C | D | Original direct visits | Corrected total units | Logical payload bytes | Latest corrected public-query us |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 3 | 3 | 8 | 288 | 90407 |
| 1 | 18 | 18 | 38 | 2368 | 87100 |
| 1 | 258 | 258 | 518 | 36928 | 163963 |
| 16 | 18 | 288 | 68 | 2368 | 86596 |
| 256 | 3 | 768 | 518 | 288 | 253647 |

Total units include catalog construction visits, hash-map construction probes,
lookup probes and candidate inspections; units are logical operations, not equal
CPU costs. C1 pays construction overhead:8 vs3 at D3 and518 vs258 at D258.
Original observed C1/D3 query87830us versus90407us latest and C1/D258
179755us versus163963us provide no consistent demonstrated consequential small-
workload regression. C256/D3 latest253647us exceeds original231863us despite
fewer candidate-work units: this reinforces that candidate reduction is not a
universal latency claim. No adaptive threshold or broader caching is introduced.
Repeated controlled latency/allocator benchmarking was not performed.

Dense extension competitors n1/2/4/8/16/64/256 preserve exactly one result only
at n1; every competing case refuses. Each builds once, visits n+1 declarations,
performs2n+1 construction probes, two lookups and n necessary normalized receiver
candidate checks. Largest logical payload39088 bytes; observed production-
compilation time1071816us. These timings include compilation and cannot be
compared with dirty public-query timings or claimed as before/after extension
speedups. Static/arity/receiver refusal tests pass. Duplicate order, direct
precedence/ambiguity, property caller identity and retained lifecycle controls
remain separately verified by the earlier ten-test preservation packet.

Production changes are confined to src/ir/csharp_exact_calls.rs: one lazy local
lookup slot, passed into resolve, and explicit borrowed duplicate-preserving maps.
No currentness, source evidence, public contract, lifecycle or generation change.
The immutable slice remains authority. Candidate work changes from repeated C*D
irrelevant inspection to one D build plus requested buckets. Dense extension
ambiguity legitimately remains proportional to eligible bucket size; F demanding
owners can still build F catalogs. A2 measured six F*C traversals independently;
A1/A4 remain unmeasured obligations, not repaired by A3.

The finite static guard plus real isolated full-scan mutation prove this known
selection regression is detectable. Arbitrary aliases, changed parameter types,
macros and cross-module consumers remain limitations; no whole-program or complete
coverage-drift certification follows. The new A3 group is wired into mandatory
fitness composition, but the complete composed AF-PERF gate has not been rerun
since that extension. Exact spread/unsupported and normalization-specific adversarial
controls beyond existing compiler behavior are not newly certified here.

Next bounded repository gate, retaining all existing AF-PERF-001 authorities:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

This is the existing architectural gate, not the superseded historical semantic
campaign. Binary/field certification remains distinct and is not requested now.
A3's measured correction and focused evidence are ready for review with these
limits; no claim closes AF-PERF-002, broader #174 or system discovery.


## Composed architectural gate verified

Agent MaxHeadRoom inspected af-perf-001-20261104-110439:38 tracked passes,
zero failed/ignored across14 admission/AST/lifecycle,5 resolver P0,6 delegated,
9 index174 and4 C# A3 tests. Group times27.94/4.38/2.88/0.20/16.80s.
All receipt source hashes match at inspection; inspection.json records the count
and group outcomes. This gate runs tracked tests, not a binary/field certification,
new lint execution or complete whole-system discovery review.

A3's approved finite correction now has original intended RED, construction-inclusive
public-operation GREEN, exact semantic/lifecycle preservation, small/dense controls,
finite static enforcement, isolated production mutation RED/restored GREEN and the
composed mandatory fitness GREEN. No source changes follow this verification other
than evidence prose. Logical work improves for repeated irrelevant selection;
small-C construction overhead and nonuniform latency remain explicit. No universal
speedup, allocator-byte measurement or cross-owner bound is claimed.

Broader AF-PERF-002 remains active: A2's measured six F*C scans warrant a separate
proof-equivalent design; A1 cloning/A4 dependency work, declaration membership
representation disposition, NgRx reconstruction and witness/lifecycle/lock costs
remain separate. Syntax accounting remains9763 preserved/13 scoped/9750 pending
plus96 callback supplements. Neither composed GREEN nor A3 repair completes
production coverage, source-wide inventory or final adversarial review. No further
historical campaign or Claude environment switch is requested for this result.

```architecture-graph
node: DOC-143
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Historical A3 lazy candidate buckets with construction tradeoffs
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_ARTIFACT_WORK_RESULTS_AND_A3_DESIGN.md
note: Source-attributed exact historical subject and limits retained; no current executable claim, independent certification, reclosure or banking.
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
```
