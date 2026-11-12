# AF-PERF-002: approved encoder reverse-map reuse correction

Agent: Agent MaxHeadRoom
Status: CLOSED / FIX / PROTECTED for bounded encoder reverse-map reuse.
Original RED, current-byte semantic/public/work/cost GREEN, real restoration mutation
RED/restored GREEN and mandatory composed acceptance executed.
Vector-borrow repair stays protected.

## Approved boundary and baseline

Owner authorizes only crate-private read-only StringTable::index_of(&str) returning
Option<usize> from the existing indices map, and encoder retrieval through that
accessor with the same as-u64 conversion and exact missing-key expect. No new map,
cache, container, threshold, persistent state, public API, ordering, dictionary,
opcode, varint, body flag, producer or currentness-policy change is authorized.
SQLite unused text stays separate; dense same-signature equality stays deferred.

Design evidence binary-reverse-design-20261108-154747 is preserved:2 test-only tests,
12 populations, protected production identities, key-law/39 public-derived payload
equivalence and paired construction/query/write/drop observations. It is NOT
optimization RED or production acceptance. Its prototype timing is not substituted
for actual integrated cost review. Long duplicates were effectively flat.

## Pre-repair observations, no release algorithm change

Existing actual encoder observations already count redundant map entries, offered
key lengths, final capacity, necessary operand queries and output bytes. Add only
cfg(test) observations inside current StringTable: builder entry, actual tuple
population/bytes, intern calls/offered key lengths, successful new strings/bytes
and maximum ordered-vector/owned-index capacities. These maxima are not cumulative
capacity or allocator totals. The existing two owned String copies per unique table
entry remain; neither is silently attributed to redundant borrowed-map setup.

Observation updates have a test-only disable scope, with unwind-safe restoration,
to support symmetric construction-inclusive integrated timing later. Diagnostic
updates do not feed production lookup or authority. No index_of exists yet; the
existing from_instructions/intern release logic and encoder are unchanged.

Before original RED, the discovery helper's old requirement that reverse entries
equal unique strings is replaced by independent necessary-table population/build
assertions. Its exact binary/public/source assertions remain intact. This avoids
freezing an implementation shape the owner explicitly authorized removing. New
independent budget requires BOTH redundant entry and capacity sums zero after all
public semantics pass. Original clone budget continues to require no vector clone.

## Frozen original and semantic controls

original_binary_reverse_af_perf_002.rs copies the COMPLETE current protected
encoder/wrappers, with import/module paths adapted and test registrations omitted.
Its decoder is a new byte-exact tracked copy so module-duplication lint is avoided.
Its original map observation sites remain available for later real restoration and
symmetric timing; production never delegates to this oracle. Common StringTable
construction still uses actual production code, whose pre-repair bytes are frozen.
New read-only accessor must leave common construction untouched.

New frozen-equivalence test compares actual complete bytes/decoded identity and
wrappers against this exact pre-reuse encoder for all-opcode, empty/max-version and
the same12 cost populations. The existing42 binary controls additionally retain
first-appearance/duplicate/empty/Unicode/long payload, body flags, malformed decoder
and exact faulted missing-key panic distinctions. Decoder identity and original
data remain preserved; no new error policy follows.

Existing actual registered public identity covers40 intervals across all relevant
fidelities, persisted compression, first checkpoint, repeated already-durable saves
and Verbatim no-encoding behavior. Full source/cache/live tuples/presentation/edges,
response metadata and durable binary identities match the already tracked frozen
records. New public budget independently repeats these same real operations and
emits all40 complete records before its final zero-reconstruction assertion.

Driver freezes encoder and StringTable source bytes separately, all oracle/budget/
helper/dependency sources and complete public records/work. Original and After must
retain exact tracked test/fixture identities. Shared observation/helper adjustments
are completed BEFORE RED, not edits to make an observed budget failure disappear.
The completed vector-borrow static guard still recognizes current encoder bytes;
its future reconciliation is limited to the separately approved retrieval change.

## User-owned genuine original execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-binary-reverse-budget.ps1 -Phase Before
```

Expected THREE prerequisite tracked passes: existing42 frozen binary cases, new
current-encoder frozen equivalence, and complete registered public identity. Then
one intended public budget FAIL after all40 cases, specifically
`Binary encoder redundant reverse-map budget`. Compilation, fixture, source/currentness
or earlier semantic failures cannot count as RED. Zero ignored required. Console
prints short status only; full data remains under target/af-perf-002.

Before packet formats its active sources and checks size only; it does not certify
Clippy/final acceptance. All execution is UNRUN. Agent performed bounded reads,
diff/strict UTF-8/line and PowerShell parser checks, no tests/build/formatter/runtime.
No production optimization or accessor before inspected genuine original RED.

## After RED and acceptance obligations

After valid RED, apply exact accessor and retrieval substitution without another
owner approval. Preserve all necessary common-table and operand observations;
zero obsolete entry/capacity counters must correspond to actual absence of map
construction. Reconcile finite no-vector-clone protection without weakening it.
Require frozen exact binary/public and unchanged budget GREEN on current bytes.

Then direct accessor first-position/None/duplicate/empty/Unicode/from_json/clone
controls and finite complete writer/lookup/panic protections. Retain all12 fixture
populations for integrated full setup/query/write/drop paired timing with symmetric
observation overhead and separately visible common/redundant/replacement work.
A consequential integrated regression returns to owner BEFORE mutation/composed.
No test-copy result or zero old counter alone grants cost acceptance.

After cost acceptance, isolate REAL original map/lookup restoration to require
intended budget/static RED while exact binary/public remain GREEN; require byte-exact
restored GREEN. Focused binary/persistence/checkpoint/history/edit/delta, warning-fatal
all-target/all-feature Clippy, format/size/encoding and mandatory composed fitness
with final encoder/table identities remain required. No stage, commit or push.
No FIX / PROTECTED before completed acceptance. Prior repairs stay closed; source
census unchanged13/9763 scoped,9750 unresolved plus96 callback supplements.

## Genuine original reconstruction RED verified; minimal reuse applied

Agent MaxHeadRoom inspected binary-reverse-before-20261108-160709 BEFORE production
edits: three prerequisite tracked passes (existing42 binary cases5.25s, protected
pre-reuse frozen equivalence1.73s, complete public40 identity65.12s). Independent
budget completes all40 exact identities then fails only the intended final reverse-
map assertion57.71s:9027 redundant entries/12264 capacity slots versus zero.
Zero ignored. All1376 recorded source hashes match, including exact pre-reuse
encoder and instrumented StringTable snapshots. inspection.json records the check.
The earlier test-only design GREEN is not substituted for this genuine RED.

Production now adds only index_of backed by self.indices.get(s).copied(), removes
the encoder's redundant map construction, and uses table.index_of(s).map(|i|i as u64)
before the EXACT original expect. Output observation reports zero redundant map
capacity because no such map exists; actual common owned-index capacities remain
separately visible. Interning, tuple construction, string-vector borrow/emission,
operand observer/order, conversions, panic, opcode writing and final bytes otherwise
retain the original algorithm. No new container, cache, threshold or public API.

The older vector-clone finite guard is reconciled only to this separately authorized
retrieval expression, retaining complete writer comparison, explicit no-vector-clone
check and ownership-restoration sensitivity. The historical missing-key probe remains
unchanged; an additional actual-accessor empty-table probe is appended to the same
test-support file. Thus the frozen42 control can still require exact active closure
source identity while retaining both old and new exact panic paths. A new tracked
accessor law test covers first positions, duplicate/empty/Unicode, read-only None,
clone/from_json and the actual new faulted panic. No fixture or semantic oracle
assertion is weakened or regenerated. Frozen reverse budget/oracle/helper sources
remain byte-identical; these finite static/probe adaptations are separately recorded.

Next user-owned current-byte execution:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-binary-reverse-budget.ps1 -Phase After
```

Expected four tracked passes: existing42 binary controls, new protected pre-reuse
equivalence, complete40 public identities and unchanged zero-reconstruction budget.
Current-byte work/cost, accessor/static protection, isolated original map restoration,
focused checks and mandatory composed acceptance remain required afterward. Stop
for owner review if integrated construction costs demonstrate a consequential new
regression before mutation/composed. No agent runtime, stage, commit or push.

## Current-byte After GREEN; integrated cost/focused packet authored

Agent MaxHeadRoom inspected binary-reverse-after-20261108-161508: four tracked
passes, zero failed/ignored. Existing42 exact binary controls5.33s; protected
pre-reuse frozen equivalence1.75s; complete40 public identity69.28s; unchanged
zero-reconstruction budget40 cases72.19s. All1376 recorded source hashes match at
inspection. Redundant entry/capacity sums are zero; common construction and operand
work remain separately observed. inspection.json preserves the parsed work. No
test-copy or earlier vector-clone receipt is substituted for this current-byte run.

New integrated cost test compares ACTUAL production encode with the frozen actual
pre-reuse encoder over the original12 fixture populations. Both use actual common
StringTable construction and identical common observation sites. All updates are
disabled symmetrically for24 alternating samples after3 warmups, with full table/
map/query/write/output teardown inside timing. Residual diagnostic calls still
exist; debug evidence cannot establish release/public latency or allocator savings.
Enabled work separately asserts common tuple/intern/new-string/capacity/operand/
output equality, retained no-vector-clone behavior and zero new redundant setup.

Finite new protection checks the complete encoder against the frozen pre-reuse
body after ONLY authorized retrieval substitution, and the complete StringTable
against a tracked exact Before text fixture plus ONLY the approved read-only
accessor. It independently rejects map restoration, changed lookup authority,
conversion and panic text. The older clone guard's independent ownership mutation
and exact binary/public controls remain mandatory. No fixture data is discarded.

User-owned integrated review:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-binary-reverse-focused.ps1
```

Sixteen groups include integrated cost/static/accessor law, frozen binary/new
pre-reuse equivalence, complete public identity/new reconstruction budget, old
clone budget/static protection, both binary modules, StringTable tests and existing
checkpoint/history/edit/delta controls. Exact totals are recorded rather than
inferred from module names. Require12 full cost rows and warning-fatal all-target/
all-feature Clippy, formatting, size, strict encoding and unchanged frozen source
identities. Full logs stay on disk; console prints short summaries only.

This packet is authored UNRUN. Inspect integrated cost evidence before authoring/
running real restoration mutation and composed closure. A consequential new
regression returns for owner review, without adding thresholds or alternative
paths. No additional production algorithm changes follow After; only tracked
tests/data and module registration are added. No agent runtime or repository
publication. All previously protected repairs stay closed.

## Integrated focused acceptance inspected; real restoration gate authored

Agent MaxHeadRoom inspected binary-reverse-focused-20261108-162253: 63 tracked
passes across 16 groups, zero failed/ignored, all twelve integrated cost rows.
All 1378 recorded hashes match. Warning-fatal all-target/all-feature Clippy passes
51.85s; formatting log empty, size passes 32 active files and strict UTF-8 passes
2242 text files. Existing binary, checkpoint, history, edit/delta, public identity,
clone prohibition and new accessor/static/work controls pass on current bytes.

Actual frozen pre-reuse / actual production medians in microseconds:

| Methods / Unicode repetitions / duplicate identity | Original | Reuse |
| --- | ---: | ---: |
| 0 / 0 / no | 21.0 | 15.2 |
| 1 / 0 / no | 20.9 | 15.9 |
| 8 / 0 / no | 81.3 | 63.6 |
| 64 / 0 / no | 490.9 | 408.9 |
| 256 / 0 / no | 2025.2 | 1671.9 |
| 1 / 4096 / no | 451.3 | 358.7 |
| 16 / 64 / no | 248.7 | 189.0 |
| 16 / 4096 / no | 10225.5 | 6694.7 |
| 64 / 4096 / no | 45454.1 | 31868.8 |
| 8 / 0 / yes | 130.0 | 112.9 |
| 256 / 0 / yes | 1064.1 | 956.5 |
| 64 / 4096 / yes | 12531.9 | 11872.4 |

All medians favor reuse; duplicate sample ranges overlap. No consequential new
regression is demonstrated. These are 24 alternating debug complete-operation
samples, with symmetric disabled observation updates and residual diagnostic calls.
They do not establish universal nonregression, release/public latency or allocator
savings. Common tuple/intern/string/capacity/query/output observations agree;
redundant map entries/capacity and owned vector clones are zero on current encode.
Full sample vectors and work observations remain in the focused logs.

Current production SHA-256:

- binary_wire.rs: 8d9148ef19d270e7e5f8ad2853fc6e1e23d1c886d5552170c2157028ba8a1018
- string_table.rs: 0cbd68f927dc41c9f6a73765912aaba2755f59d01fc18b057dab1562529c9966

Next user-owned command:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-binary-reverse-mutation.ps1
```

Authored UNRUN: an isolated copy restores the genuine Before encoder, retaining the
current StringTable accessor and protected borrowed vector. Require independent
budget/static intended RED, three preserved binary/equivalence/public GREEN, then
byte-exact restoration and five GREEN plus matching active/isolated inventory.
Compile, warning or fixture failures cannot satisfy intended RED. Active source is
never mutated. Console emits short summaries; complete public rows stay in logs.

Composed fitness now includes four new reverse-map protections, expected at least
92 tracked passes across 37 groups. It remains UNRUN and follows inspected mutation
acceptance. No agent runtime, stage, commit or push. This slice is not yet CLOSED /
FIX / PROTECTED; previous repairs remain closed and dense equality deferred.
## Real original-map restoration acceptance inspected

Agent MaxHeadRoom inspected binary-reverse-mutation-20261108-163357. Clean static
baseline passes. Restoring the complete genuine Before encoder in an isolated
production copy produces intended unchanged budget RED (63.79s) and static RED.
Frozen binary controls (4.87s), pre-reuse equivalence (1.68s) and complete public
identity (80.72s) remain GREEN. Both public mutant logs contain all 40 cases;
compile, fixture or warning failures are not counted as intended RED.

Byte-exact restoration passes five tracked tests: budget (61.65s), static (0.01s),
frozen binary (4.68s), pre-reuse equivalence (1.42s) and public identity (60.93s),
zero failed/ignored. Both restored public logs contain all 40 cases. All 2250
inventory hashes match active and isolated restored files. Active production was
never mutated; encoder and StringTable identities remain exactly those recorded
in focused acceptance above. inspection.json retains the checked outcome summary.

Only mandatory composed acceptance remains. User-owned command:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

Expected at least 92 tracked passes across 37 groups, including four new reverse-map
protections and retained vector-clone protection, with matching source identities.
UNRUN; no closure classification until this receipt is inspected. No production or
test changes follow focused/mutation acceptance. No agent runtime, stage, commit
or push. Previous protected repairs remain closed; SQLite unused text is separate
and dense same-signature equality remains deferred.
## Final composed acceptance: CLOSED / FIX / PROTECTED

Agent MaxHeadRoom inspected af-perf-001-20261108-164840: 92 tracked passes across
37 groups, zero failed/ignored. All 309 recorded source hashes match current bytes,
including the four new reverse-map protections and retained vector-clone controls.
Final encoder and StringTable identities match focused and restored mutation:

- src/ir/binary_wire.rs: 8d9148ef19d270e7e5f8ad2853fc6e1e23d1c886d5552170c2157028ba8a1018
- src/ir/string_table.rs: 0cbd68f927dc41c9f6a73765912aaba2755f59d01fc18b057dab1562529c9966

inspection.json records the checked pass/group/hash totals. This is current actual
production acceptance, not substitution of the earlier test-only design results.
The evidence chain remains distinct: genuine original RED160709 (9027 redundant
entries/12264 capacity slots with complete binary/public semantics GREEN), After
161508 (four passes), focused162253 (63 passes/16 groups, twelve integrated cost
rows and warning-fatal checks), real mutation163357 (two intended RED, three
preserved GREEN, five exact restored GREEN plus clean baseline), and composed164840.
All generated receipts remain under target; tracked tests remain the contracts.

The correction removes only the second reverse map. Existing StringTable ownership,
interning, first-appearance indices, duplicates, exact strings, dictionary/opcode/
varint writing, operand conversion/panic boundary and binary/public/durable/source/
cache identities remain protected. Necessary tuple construction, hashing, operand
queries and output writing still occur. The protected string-vector borrow remains.

Integrated complete-operation debug costs and full samples remain recorded above.
Logical eliminated entry/key/capacity work does not measure allocator events, peak
memory, CPU instructions or public latency. No universal speedup/nonregression or
release/public latency benefit is claimed. No further tuning or verification is
required merely to establish this bounded closure.

No production/test changes follow completed acceptance; this update records evidence
and closure only. No agent runtime, stage, commit or push. Publication remains
separately unauthorized. SQLite unused pretty_text materialization is the next
independent candidate, without implementation approval. Dense same-signature equality
remains deferred. All earlier protected corrections stay closed; census unchanged.
```architecture-graph
node: DOC-148
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Historical encoder reverse-map reuse correction
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_BINARY_REVERSE_REPAIR.md
preserves:
  - AUTH.BINARY_V04
note: Source-attributed exact historical subject and limits retained; no current executable claim, independent certification, reclosure or banking.
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
```
