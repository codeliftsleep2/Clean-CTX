# AF-PERF-002: hierarchy owner-placement discovery

Agent: Agent MaxHeadRoom
Status: approved two-site placement repair FIX / CLOSED / PROTECTED.
Original RED, semantic/work GREEN, static/mutation and composed GREEN verified.
Historical discovery and authored-status entries below remain chronological evidence.

The successful compress_code_context path calls checked_hierarchy_or_respond in
core/common.rs, which invokes try_ir_to_hierarchical before rendering, economics
and publication. provide/evaluate.rs also invokes checked hierarchy construction.
This is an active production conversion, distinct from the downstream renderer.

In ir/hierarchical/encode.rs, DefMethod and DefField call find_class_by_id over
all currently materialized classes. The enclosing operation already populates
class_locations for later metadata attachments. This juxtaposition is a concrete
review lead, not proof that replacing the scan is semantically equivalent: forward
owner definitions use pending queues, validation precedes placement, and declaration
ordering must remain exact. No map substitution or eligibility change is approved.

With C owner definitions and K members of each kind per owner, source inspection
predicts K*C*(C+1) successful class-ID comparisons when each owner is found at its
ordered position. This is an analytical prediction, not executed measurement.
Definition-before-use and all-definitions-first layouts are independently exercised.
Missing/duplicate identities remain governed by checked identity validation.

The new tracked test varies C and K over1/2/4/8/16/64/256 independently plus16x16.
It calls the actual checked conversion, counts real helper predicate visits through
cfg(test) observations, and asserts ordered class/method/field IDs and names. It is
not a public-query fixture: parsing, repeated conversions, response construction,
allocation and persistence costs remain outside its timing boundary. Existing
identity tests run separately; no hypothetical optimization budget is introduced.

Renderer tracing also finds one owner-local method-name count followed by emission;
that count preserves low-fidelity overload signatures. It is justified by its stated
consumer, pending complete rendering cost coverage; it is not grouped with owner
placement scans as an independently proven defect.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-hierarchy-work.ps1
```

The wrapper runs one new characterization and existing checked identity controls,
then focused all-feature warning-fatal lint, format, size and encoding gates, retaining
logs and source hashes. No agent runtime/build/formatter/test was started. CBM is
degraded (frontend_retired timeout); bounded known-source reads supplied this trace.

Disposition: STRUCTURAL RISK / MONITOR pending measurements and enclosing-boundary
coverage. C2 remains MONITOR; C1, NgRx and A3 remain protected. Syntax accounting
unchanged9763 preserved/13 scoped/9750 pending plus96 callback supplements. Other
hierarchy phases, rendering, compaction and source-wide discovery remain incomplete.


Initial operator hierarchy-work-20261104-163053 stopped at compilation: the new
test-only counter called nonexistent record instead of the existing update API.
Corrected that call only. No characterization executed and no performance RED
is claimed. Corrected packet remains UNRUN.


## Executed characterization and bounded design return

Operator hierarchy-work-20261104-163440: one characterization passes0.03s,
twelve existing identity controls pass0.01s, zero failed/ignored. Warning-fatal
all-feature/all-target Clippy25.27s, formatting and size gates pass; strict encoding
passes2090 text files. All 1300 receipt source hashes match at inspection;
format-check output is empty. No source edits follow this inspection. This is
characterization GREEN, not an optimization budget or permanent scaling protection.

| Classes C | Members per kind K | Actual owner comparisons, each ordering |
| ---: | ---: | ---: |
|1|1|2|
|2|1|6|
|4|1|20|
|8|1|72|
|16|1|272|
|64|1|4160|
|256|1|65792|
|1|2/4/8/16/64/256|4/8/16/32/128/512|
|16|16|4352|

All28 layout/dimension cases match K*C*(C+1) exactly. Ordered class/member
identity and names pass. The two layouts are class-then-its-members and all-class-
definitions-first; neither is member-before-owner. Existing checked identity tests
supply additional legal-reordering and rejection controls, not a measured growth
axis for every refusal. Public repeated-conversion multiplicity remains unmeasured.

C256/K1 checked conversion samples are4845/4835us across the two layouts; C1/K256
samples1792/1814us; C16/K16 samples2097/1988us. Counter overhead, validation,
cloning and complete hierarchy construction are included. These timings do not
isolate lookup latency or establish enterprise impact. No allocation measurements
or corrected implementation comparison exists yet.

### Smallest correction candidate, not implemented

In only DefMethod and DefField placement, consult the existing class_locations
map using the class_id already created by that arm, copying the stored usize.
Retain the exact existing pending-queue branch when the owner is not materialized.
Do not use class_location's error-returning helper: valid forward definitions must
continue to queue. No new map, key clone, API, cache, ordering or authority protocol
is needed. Interface placement already follows this local map pattern.

The current map inserts every validated class immediately after pushing its node;
indices remain stable as the Vec grows. Duplicate class identities are rejected by
validation before construction. Exact typed ID equality must remain byte equality;
no normalization or fallback is proposed. Reconcile all find_class_by_id callers
before removing any helper, and keep legacy pattern behavior outside this slice.

Four criteria: repeated irrelevant class-ID checks are demonstrated; an already-
constructed map offers bounded lookup without new construction/state; pending
ownership and ordered insertion can be retained; the production diff can remain two
lookup substitutions. Hash costs and complete-operation timing still need before/
after evidence, especially C1. Classification: measured scaling defect / DESIGN
REVIEW, narrow FIX candidate awaiting owner approval. No production optimization
has been made under characterization authorization.

If approved, first author and execute deterministic construction-inclusive work
budget RED against these scans; preserve this characterization separately. Require
unchanged GREEN, forward-owner/mixed member ordering, duplicate/missing rejection,
full hierarchy and rendered response equivalence, small/growth timings, finite
caller/sink protection and isolated real scan mutation RED/restored GREEN. Enclosing
public-path multiplicity remains an explicit evidence requirement, not inferred from
this direct checked-conversion test. No broader hierarchy rewrite follows.


## Conditional approval: lookup equivalence and intended baseline

Owner conditionally approves only the two placement substitutions. Source proof:

- try_ir_to_hierarchical calls validate_identity_graph before constructing classes.
  contracts::collect_definitions_and_singular_facts and cardinality::insert_definition
  reject repeated ClassId even when both occurrences have identical names. Thus no
  successful placement scan can choose among duplicate class identities. Repeated
  class names with distinct IDs remain legitimate and distinct.
- ClassId derives exact String Eq/Hash; from_serialized only owns the same bytes.
  The original scan compares ClassNode.id to the same raw ID. No normalization.
- Each DefClass pushes one node then inserts its exact current index. The Vec only
  appends; reallocations do not change indices. Map iteration is never used for
  output order. Class and member insertion order stays governed by the same code.
- A valid owner not yet materialized is absent from both classes and class_locations.
  Both therefore take the original pending queue. When the owner arrives, the same
  queued members drain before later members. A truly missing owner fails global
  validation before either placement implementation runs; do not replace pending
  behavior with the error-returning metadata helper.

The new semantic control covers mixed queued/later method and field definitions,
class order B then A, duplicate class names with distinct IDs, identical/different
names with duplicate IDs, and unresolved method/field owners. Existing twelve
identity controls remain separate GREEN evidence, not claimed execution of these
new assertions.

Precise comparison arithmetic: a successful scan for class at zero-based position
j visits j+1 entries, not necessarily the complete collection. K methods plus K
fields contribute2*K*(j+1). Summing j=0..C-1 gives K*C*(C+1). Proposed map use removes
only these two scan predicates, replacing them with2*K*C logical map probes.
String hashing/equality still costs work; HashMap lookup is not a worst-case constant
CPU guarantee. Validation, class-map construction, pending drains, payload cloning,
metadata attachment, rendering and other hierarchy work remain. No claim that all
quadratic hierarchy work disappears is justified.

Tracked hierarchy_placement_bounded_work records full checked-conversion timings
and requires scan comparisons plus placement-map probes no greater than one per
method/field request across the same28 cases. Its baseline assertions are authored
before changing either production placement site. The existing characterization is
unchanged. A future map-probe counter is test-only and currently zero.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-hierarchy-placement-budget.ps1 -Phase Before
```

Expected one ownership semantic pass and one genuine bounded-work failure. The
wrapper selects each fully qualified test with --exact, validates the failure
message and summary, preserves source bytes and hashes. Compilation/fixture failures
are not valid RED. No runtime was started by the agent. Production remains the
original two scans; no implementation until actual intended RED is verified.
After that evidence, conditional approval authorizes the two existing-map lookups
without another design review. Static/mutation protection and final gates remain
required. Public multiplicity characterization remains useful independent work,
but owner explicitly does not require public speedup before this bounded repair.


## Genuine baseline RED verified; approved placement substitution authored

hierarchy-budget-before-20261104-164550 verifies ownership/refusal control GREEN
(0.29s) and intended work-budget RED (0.04s), zero ignored. All 1301 recorded
source hashes match before editing. C256/K1 observes65792 comparisons against512
allowed placement lookups; joint C16/K16 observes4352 against512. These are actual
tracked budget failures, separate from earlier characterization GREEN.

Production edit: only the DefMethod/DefField lookup expressions now use
class_locations.get(&class_id).copied(), with test-only counters at those exact
probes. The already-owned class_id is borrowed; no new key clone or map allocation.
All insertion, pending queues and validation remain unchanged. The unused private
scan helper is removed; its comparison counter remains for regression mutation
measurement. Neither production lookup nor validation has executed after this edit.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-hierarchy-placement-budget.ps1 -Phase After
```

Expected one semantic and one unchanged budget pass. Wrapper formatting list now
includes the edited encoder; test assertions are unchanged. No agent runtime was
started. Full semantic/rendering gates, static scope enforcement, isolated real
scan restoration mutation and final GREEN receipts remain pending; this is not
FIX/PROTECTED acceptance yet. No unrelated hierarchy or public caller changed.


## First repaired budget GREEN; permanent protection authored UNRUN

hierarchy-budget-after-20261104-164848 passes the unchanged bounded-work test0.04s
and ownership/refusal control0.02s. All1301 receipt source hashes match before new
acceptance assets. Every measured placement scan comparison is now zero; map probes
are exactly2*C*K. C256/K1 changes65792 comparisons to512 map probes; C16/K16 changes
4352 to512. These are logical operations, not equivalent CPU units. No new map,
capacity, string or declaration payload is allocated by the substituted lookups;
existing ClassId creation and map construction remain unchanged.

The new finite AST test classifies every selection call inside exactly the
DefMethod/DefField arms, requires the existing map/key and flags loops/new delegated
calls for review. It does not prohibit scans throughout hierarchy construction.
Caller scope remains checked projection reached by core/common and provide/evaluate;
no caller or operation boundary changed. The mandatory fitness composition now
includes both placement tests and this static test; old receipts remain frozen.

An isolated mutation driver overlays current tracked/untracked source into a local
clone, restores the original DefMethod scan from frozen Before source (inline with
actual comparison instrumentation), and requires independent budget and static RED.
The DefField map path remains, demonstrating sensitivity to one regressed site.
It restores exact encoder bytes and requires both tests GREEN, verifying active
source unchanged. This is authored enforcement, not an executed sensitivity claim.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-hierarchy-focused.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-hierarchy-mutation.ps1
```

Run sequentially. Focused selects all hierarchical and LLM renderer tests, the
static test and all-feature warning-fatal lint/format/size/encoding; mutation then
requires two intended failures and two restored passes. Both are UNRUN. Agent only
performed bounded source/text/UTF-8/size and PowerShell syntax inspection. A final
composed fitness receipt remains pending afterward. FIX/PROTECTED is not yet claimed.


Descriptive complete checked-conversion samples (interleaved layout, microseconds):

| C | K | Before | After |
| ---: | ---: | ---: | ---: |
|1|1|406|416|
|256|1|4831|3173|
|1|256|1740|1901|
|16|16|2512|1909|

These single instrumented observations are not latency distributions; they show
mixed small/one-owner results and reduced large-owner work. No universal speedup
or attributable public-query latency claim follows. The deterministic scaling
correction, rather than a timing threshold, is the approved justification.


Focused hierarchy-focused-20261104-165729 stopped at the new static test: its
finite call classification omitted the existing FieldLocation::Class enum
constructor. Classified that constructor alongside MethodLocation::Class; no
selection scan or delegated lookup was permitted. Production unchanged. Focused
acceptance remains pending rerun; this harness omission is not mutation evidence.


Mutation hierarchy-mutation-20261104-165823 used the earlier static-test bytes:
its isolated classifier lacks FieldLocation::Class, while current active source
includes it. Restored encoder bytes match active encoder exactly, but restored
static test fails the same unrelated constructor omission. Therefore this run is
not accepted mutation RED/GREEN evidence. Logs remain frozen. The focused failure
reported again references the same165729 run; no newer focused directory exists
at inspection. Both packets must run against the corrected current classifier.

Mutation wrapper now requires clean isolated static GREEN before inserting the
scan, preventing an unrelated baseline finding from being attributed to mutation.
No production or semantic assertion changed. That wrapper enhancement is UNRUN.


## Focused and isolated regression evidence verified

hierarchy-focused-20261104-171027 passes145 tracked tests:92 hierarchical (2.54s),
52 LLM renderer (0.02s), one static placement test (0.02s). Zero failed/ignored.
All-target/all-feature warning-fatal Clippy34.79s, empty format-check output, size
18 active files and strict encoding2095 files pass. All 1302 receipt source
hashes match at inspection before this documentation update.

hierarchy-mutation-20261104-171239 first establishes clean isolated static GREEN.
Restoring one actual production method-placement scan causes exactly two intended
failures: budget (C256/K1 total33152 vs512 allowed) and static selection-sink rule.
Field placement remains indexed. Exact encoder restoration gives budget and static
GREEN (two tests), zero failed/ignored. Active and isolated restored encoder bytes
match; all 2102 inventory hashes match at inspection. inspection.json retains
these comparisons. The earlier165823 failure remains superseded evidence, not an
accepted mutation proof. No production/test code changes follow verified runs.

Disposition: narrow FIX implemented and focused/protected evidence GREEN; final
composed architectural gate pending. This closes neither other hierarchy work nor
source-wide discovery. No universal constant CPU bound, public speedup or whole-
workspace certification is claimed. Pending queues, duplicate rejection and ordered
identity remain covered by the executed affected suite. Public-operation multiplicity
is not a prerequisite to this owner-approved scaling correction.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

Expected at least49 tracked passes including the three new placement protections.
This is the existing mandatory composition with the approved repair added, not a
historical campaign or live binary/Claude gate. Agent did not start runtime work.


## Final approved-scope acceptance

Composed mandatory gate af-perf-001-20261104-172427 passes 49 tracked executions
across 11 selected groups, zero failed/ignored. All 212 receipt source hashes
match at inspection; inspection.json retains per-group counts and identity checks.
This final receipt follows the145-test focused semantic/rendering/static GREEN and
isolated real production scan RED/exact restored GREEN. No new Rust/test edits
follow this verification. Agent performed evidence inspection only, no runtime.

Final disposition: hierarchy DefMethod/DefField placement FIX / CLOSED / PROTECTED.
The production change reuses the already-built class_locations map twice and removes
the now-unused scan helper. At C256/K1,65792 scans become512 logical map probes;
forward queues, exact duplicate rejection, class/member order and identity remain.
No map construction, payload cloning, public API, authority or lifecycle change.

This acceptance covers the approved two placement sites only. Remaining hierarchy
validation/assembly work, public invocation multiplicity, rendering and broad
AF-PERF-002 discovery stay independently open. It does not establish whole-system
architectural fitness. C2 stays MONITOR; no protected earlier work is reopened.
