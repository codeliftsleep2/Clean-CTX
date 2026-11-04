# AF-PERF-002: A2 baseline reconciliation and #173 observation review

Agent: Agent MaxHeadRoom
Status: original-position prepass implemented; genuine budget RED then focused GREEN.
Focused semantic/static/currentness, isolated mutation proof and composed gate GREEN.
Disposition: FIX / CLOSED / PROTECTED for the approved collector slice only. Historical review retained below.

## A2: reconcile completed work without reopening it

Current commit a6a41b42 is `perf: bound C# exact-call selection with call-local
borrowed buckets`. This implemented repair is A3 in the tracked reports, not A2.
It covers project_with_authority ordinary and property-initializer selection within
one call, with borrowed duplicate-preserving direct/extension buckets. Its genuine
original RED, full-operation budget GREEN, exact semantic/lifecycle preservation,
finite AST and isolated mutation RED/restored GREEN, and composed38-test GREEN
remain the protected baseline. Latest named receipts include a3-gaps-20261104-105659,
a3-mutation-20261104-104227 and af-perf-001-20261104-110439. Exact-call production
and semantic-test hashes match the frozen gap receipt at this inspection.

A2 current six consumers and artifact orchestration still implement their original
owner predicates; only test observations and unchanged vocabulary relocation were
added. semantic.rs, known_types.rs, artifact.rs and density test bytes match
 a2-density-20261104-120150. A2 also has a2-ownership-20261104-112938 and
 a2-owner-class-20261104-101547 characterization evidence. These are executed
semantic/work observations, not optimization RED/GREEN or permanent A2 budgets.
The last explicit A2 disposition accepts characterization for deciding against
A2a, with A2 MONITOR/DESIGN REVIEW and broader coverage open.

Therefore: A2's accepted no-repair decision is CLOSED for that decision; do not
reinvestigate or reimplement it. A3's approved implemented correction is PROTECTED.
There is no discoverable later A2 repair/completion receipt here to certify as
CLOSED/PROTECTED production optimization. If a different completed repair is intended,
the exact gap is its path/commit and receipt identity, not a need to repeat A2
characterization. No new A2 engineering is queued. Remaining broad risks stay separate.
Exact inspected hashes are retained in target/af-perf-002/
a2-status-ngrx-observation-review/inspection.json.

## #173: facts and mutation paths traced

The current eager binding collection occurs after old refined-edge retraction and
synthetic entity cleanup, after candidate cloning, before authority passes.
object_member_value_bindings reads forward.values().flatten(), selects exact
ReferencesValue relation and typescript_object_members layer, then clones complete
endpoint names/files. It retains occurrence multiplicity and traversal order.

Consumers: MAP_LAYER loop in the second candidate pass; SWITCH_VALUE_LAYER filtering
in the third candidate pass. No other value_bindings uses were found in this module.
The authority passes call target/declaration/export helpers without mutating forward.
They may perform filesystem IO, which must not be moved/replayed by this optimization.

Writes: add_edges and remove_file mutate forward before invoking the synchronous
rebuild. Rebuild itself retracts old refined occurrences before the original collector
point. Third-pass insert_refined_edge mutates forward.entry(subject).or_default().push
and reverse/file structures, registers synthetic entities, and records refined keys.
All rewritten refined edges have ngrx layer; these do not add eligible binding facts.
Synthetic registration/removal modifies entity/name/publisher structures, not eligible
forward ReferencesValue storage. No callback or concurrent mutation can intervene
through exclusive &mut self during the synchronous rebuild; filesystem edits are
still separately observable through existing proof helpers.

Thus the eligible fact multiset appears stable after the eager collector point.
But multiset equality alone does not prove identical ordered Vec construction.

## Exact order-equivalence gap: forward HashMap mutation

forward is HashMap<EntityKey, Vec<StoredEdge>> (index.rs). Its values iteration
order is not insertion-order authority and may change on growth. Before the first
switch-value consumer in the third pass, earlier direct dispatch, registration,
map or other switch/refinement candidates may insert a new forward key. A later
lazy collector can traverse existing eligible buckets in a different order after
HashMap growth even though no eligible fact is inserted or removed.

MAP first consumption happens before third-pass insertion, so it would capture the
original order and share that Vec with switch-value. Switch-only demand has no such
guarantee. Switch refusal currently requires exactly one eligible action, which may
make binding order unobservable for that consumer's final fact; nevertheless the
owner explicitly requires preserving original collection occurrence order and
duplicates. Do not silently weaken that requirement based on today's cardinality
check. No sorting, new ordered index, lookup buckets or eager order-token structure
is authorized to repair this gap incidentally.

This is a source-grounded proof gap, not an executed ordering-regression counterexample.
No production edit or alleged failing semantic test is claimed.

## Original-position demand-prepass alternative for owner review

At the original eager position, inspect the already-owned candidate Vec for MAP_LAYER
or SWITCH_VALUE_LAYER. If neither exists, retain an empty Vec without collecting.
Otherwise collect once at the original position. This preserves exact observation
point, traversal order, duplicate records and all subsequent IO/branch predicates.
It introduces one short-circuit candidate-layer scan and one conditional, no new
persistent state, cache, registry, authority or protocol. It does not replay target
resolution or export IO. All reconciliation still executes immediately.

Costs: no-family case adds at most M cheap layer comparisons and removes E+B forward
visits and B four-string clones. Relevant-family case adds up to M layer checks but
retains the original one collection and all joins. May allocate no new structures
beyond the existing Vec; logical payload benefit in measured no-family B256 fixture
is34596 bytes. Empty candidate list needs no scan iterations. This is a construction-
inclusive analytical comparison, not an implemented or measured after result.

Tradeoff: candidates of a relevant layer that later fail target resolution still
trigger collection. Therefore this alternative cannot promise zero collection for
all unreachable-after-resolution cases. Its precise invariant is zero collection
when neither binding-consuming candidate family exists; at most one otherwise.
Testing actual first-consumption reachability beforehand would require moving or
repeating filesystem observations, or new staged candidate state. Neither is an
incidental substitute for the approved one-slot design.

Owner decision needed: approve this original-position conservative demand prepass
and its narrower zero-work invariant, or explicitly permit later reordered binding
enumeration after proving its order unobservable in every affected consumer. Recommend
the former: minimal code, exact old observation semantics, measured waste eliminated
without changing IO or requiring a new representation. No implementation until that
fallback disposition is supplied, consistent with the owner's proof-failure instruction.

## Verification sequence after fallback disposition

Author real tracked deterministic pre-repair RED for zero no-family collection/
visits/clones, plus demanding at-most-one and fresh-call controls. Preserve original
characterization separately; do not call its GREEN optimization RED. Then implement
only approved conditional collection, verify positive map/switch/mixed and refused/
duplicate/currentness cases, complete-operation counts/timings, nine af174 controls,
finite static known-sink gate and isolated production eager-restoration mutation.
No broad campaign, A2 repeat, join indexing or reconciliation deferral is proposed.


## Owner approves original-position prepass; baseline authored UNRUN

Owner explicitly supersedes first-consumption lazy collection. Approved correction
is a short-circuit check of the existing candidate Vec for MAP_LAYER or
SWITCH_VALUE_LAYER at the original collection position. Zero work means no such
family exists; a later refusing relevant-family candidate may still collect.
No predicate, IO ordering, collector order or reconciliation scheduling changes.

Before production modification, index_ngrx_binding_budget_af_perf_002.rs now adds:

- Actual empty add_edges operations with1/2/4/8/16/64/256 binding records, with
  no candidates or a supported unrelated direct-dispatch candidate. Require zero
  collector invocations/visits/records/string clones while reconstruction still
  runs and the supported dispatch survives. All rows print before final budget
  assertion so baseline counts/times are preserved even when intended RED occurs.
- Map-only, switch-only and mixed family presence controls, deliberately invalid
  target authority, requiring one collection per invocation. A new binding in a
  subsequent add_edges must appear in the freshly constructed collection. Presence
  does not grant semantic admission. These are not positive map/switch certification.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-ngrx173-binding-budget.ps1 -Phase Before
```

Expected one intended no-family budget failure and one demanding/fresh/mixed control
pass, zero ignored. Compile/fixture errors are not valid RED. Driver retains exact
source/test bytes, hashes, phase and results. No agent test/build/formatter/lint/
runtime execution occurred. Production collector remains unconditional; only tests,
registration and a test-only future demand-check counter were added. Positive-family,
duplicate/refusal/currentness/static/mutation acceptance remains required after
verified RED and the approved minimal conditional correction.


## Intended RED verified; approved prepass implemented, After UNRUN

ngrx173-budget-before-20261104-132405 records the intended one no-family budget
failure and one demanding/fresh/mixed control pass, zero ignored,0.40s. All receipt
source hashes matched before editing. Fourteen no-family rows show one collection
and1..256 binding visits; largest logical cloned payload33572 bytes without
candidates and32548 bytes with an independent supported direct dispatch. Payload
sizes vary with fixture paths; do not substitute earlier fixture bytes as identical.
No compilation/fixture failure is counted as RED.

Production change is confined to the original collection position: short-circuit
candidate-layer any check for MAP_LAYER/SWITCH_VALUE_LAYER, then original collector
or empty Vec. Test-only prepass comparison counter observes the actual predicate.
No downstream join, collector implementation, IO order or reconciliation call changed.
Baseline regression assertions are unchanged. New production code is UNRUN.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-ngrx173-binding-budget.ps1 -Phase After
```

Expected two tracked passes. Positive map/switch/mixed semantics, duplicate and
currentness controls, finite static gate, isolated mutation proof and refreshed
focused/composed gates remain pending. No agent runtime was started.


## First repaired budget GREEN; acceptance assets authored

Operator ngrx173-budget-after-20261104-132558 passes two unchanged budget tests,
zero failed/ignored,0.20s. No-family cases have zero collection/visits/records/string
clones at every1/2/4/8/16/64/256 point. Direct supported dispatches still perform
one export read/parse; empty-candidate cases perform zero prepass comparisons,
direct-candidate cases one. Map-only/switch-only/mixed demanding calls each collect
once and see newly added bindings on their next invocation. Presence with an invalid
target remains refused; collection in those cases is intentionally permitted.

Same bounded fixture dimensions, different temp roots, descriptive single samples:
B256/no candidates changes1 collection/256 visits/33572 logical bytes/190us to
0/0/0/10us. B256/direct changes1/256/32548/539us to0/0/0/543us, retaining export
IO. B1/no candidates8us before versus25us after; B1/direct381us versus595us.
Do not infer universal latency speedup or regression from these single observations.
No allocator-byte or enterprise benefit claim. Demanding cost remains the original
collection/joins plus the short-circuit layer comparison; broader reconstruction
characterization will provide its focused descriptive timing.

New tracked production-path positive fixture uses existing public publication helpers
to capture a reducer map plus switch-value chain, then actual empty add_edges. It
requires one shared collection and exact registration/handler identity, layer and
file attribution. A second asserting publisher duplicates a captured binding;
switch ambiguity must refuse, and removing that publisher restores the exact prior
singleton fact. Existing negative/refusal, direct/reexport retirement/repair and
nine membership controls remain separate authority.

Finite AST gate requires guarded known-sink use and original placement before
candidate authority passes, and flags new/unclassified collector calls. Mandatory
fitness composition includes four af173_binding tests. The original characterization
expectations are updated only for newly approved no-family behavior; frozen old
source and receipts remain preserved. No-family work budgets themselves are unchanged.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-ngrx173-focused.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-ngrx173-mutation.ps1
```

Run sequentially. First expects17 tracked passes: four reconstruction/currentness,
nine af174, four budget/positive/static; plus warning-fatal all-feature lint/format/
size/encoding. Second clones/overlays an isolated exact-source workspace, restores
unconditional collector in the real production function, requires exactly two
intended counter/static failures and two preservation passes, then restores exact
source and requires four GREEN. Active source is never mutated by that driver.
These assets are authored UNRUN; only bounded text, script syntax, UTF-8 and size
review was performed. No agent runtime was started. Full composed gate remains a
later final integration check; A2/A3 work is not repeated or reimplemented.


## Focused semantic/static and isolated mutation proof verified

ngrx173-focused-20261104-133246 passes17 tracked tests: four reconstruction/
currentness19.39s, nine af1740.22s, four collector budget/positive/static3.29s.
Zero failed/ignored. Warning-fatal all-target/all-feature Clippy27.77s, empty
format logs, size14 active files and strict encoding2073 text files pass.
Positive mixed map/switch fixture observes one collection, one prepass comparison,
three forward visits,232 logical cloned string bytes, two joins in each consumer
and one export read/parse. Exact map registration/handler layer/endpoints/files
pass, duplicate binding causes refusal, publisher removal restores the exact fact.
Direct and one-hop export mutation/retirement/byte repair remain GREEN.

ngrx173-mutation-20261104-133501 restores unconditional collection in isolated
production: exactly two intended failures (unused-work budget plus AST guard),
two preservation passes, zero ignored,3.85s. Exact source restoration gives four
passes, zero failures/ignored,3.58s. Active and restored isolated production SHA-256
match. Full mutation inventory hashes match at inspection, including new acceptance
assets; no production/test source changes follow these runs.

Receipt bookkeeping correction: the focused driver inherited an11-test category
label and a narrow legacy source list. Its logs prove17 passes. The independently
recorded mutation inventory binds the newly added assets omitted from that focused
list. Both receipts remain frozen; inspection.json explains this distinction.
The focused driver is corrected for future runs to label17 and hash all src/Cargo
plus its wrapper. No valid test is rerun solely to repair descriptive metadata.

### Demanding-path comparison (single bounded samples, us)

| Branch | B | M | Before | After | Join comparisons before/after |
| --- | ---: | ---: | ---: | ---: | ---: |
| map | 16 | 256 | 82702 | 82121 | 4096 /4096 |
| map | 256 | 16 | 8891 | 8328 | 4096 /4096 |
| switch-value | 16 | 256 | 40468 | 43182 | 4096 /4096 |
| switch-value | 256 | 16 | 2845 | 2986 | 4096 /4096 |

Construction remains once and demanding binding payloads remain2124/34596 logical
bytes in these fixtures. No-family B256 drops256 visits and34596 logical bytes to
zero; descriptive timings231/208us become8/10us. Samples show no consistent
large demanding regression, but are not controlled latency distributions. No
arbitrary timing threshold or universal speedup is claimed. New state is one local
boolean; the empty Vec allocates no binding payload. Allocator overhead is unmeasured.

Disposition: FIX implemented/protected for the no-family collector waste, pending
the final composed mandatory architectural gate. This does not close #173 scheduling,
M*B joins or whole-system discovery; A2 remains closed for its no-repair decision,
A3 protected, no rework queued. No authority/cache/order/IO/lifecycle change.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

Expected at least42 passes across the composed groups, including four collector
controls. Existing mandatory gate only; no historical broad campaign or binary/
Claude switch requested. No agent runtime was started.


## Collector slice final composed GREEN

Agent MaxHeadRoom inspected af-perf-001-20261104-144311:42 tracked passes,
zero failures/ignored across14 admission/AST/lifecycle,5 resolver P0,6 delegated,
4 A3,9 af174 and4 NgRx collector controls. All recorded source hashes match;
inspection.json records exact hash count and group results. No production/test
source changed after verification; this update is evidence prose only.

Approved no-family collector repair disposition: FIX / CLOSED / PROTECTED.
Original-position prepass removes proven unused collection while preserving demanding
construction, binding order/multiplicity, matching, export validation and immediate
reconciliation. Genuine original budget RED, unchanged GREEN, positive/negative/
duplicate/currentness controls, finite static enforcement, real isolated eager
mutation RED/restored GREEN and final composed fitness GREEN are all recorded.
No enterprise latency, allocator-byte or whole-system certification claim follows.

This closes only the approved collector slice. #173 M*B joins and reconstruction
scheduling remain unresolved; no deferral/index/cache change is authorized. A2's
no-repair decision remains closed and A3 protected. Source-wide discovery, A1/A4,
declaration membership and remaining inventory gaps are independent. No automatic
repeat campaign, binary rebuild or Claude field switch is needed for this result.
