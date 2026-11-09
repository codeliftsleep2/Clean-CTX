# AF-PERF-002: approved SQLite discarded pretty-text ownership correction

Agent: Agent MaxHeadRoom
Status: CLOSED / FIX / PROTECTED for bounded discarded pretty-text ownership.
Original RED, exact semantic/public/work GREEN, focused costs/checks, real restoration
RED/restored GREEN and mandatory composed acceptance executed.

## Frozen scope and semantic questions

Keep the exact SELECT, updated_at DESC LIMIT1, eligibility, column indices and
conversion order. Keep Option NULL behavior and rusqlite String's existing as_str
SQL-type/UTF-8 validation. No schema/query/connection/transaction, stored payload,
cache or persistent index change. Existing encoder, overload, edit and other repairs
remain protected. Dense same-signature equality deferred. No publication authorized.

Production lifecycle remains registered context_history -> persistence SqliteStore
load_latest, with existing fallback ContextStore behavior. Durable semantic restore
calls the same metadata boundary before canonical binary/delta/semantic restoration.
The selected row owns metadata; reads do not create live authority or flush writes.
Candidate discovery/proof/cost limitations are recorded in AF_PERF_002_SQLITE_TEXT_DESIGN.md.

This packet adds only cfg(test) production observations before the actual column4
conversion and after successful original conversion. It counts conversion attempts,
successful metadata rows, non-NULL owned strings and logical text bytes. Empty
non-NULL String counts even with zero bytes; NULL does not. The original conversion
and error propagation remain unchanged. Necessary query/validation observations
remain separate from owned-copy counts; they do not measure allocator events, peak
memory, SQLite engine copying or public latency.

A tracked frozen original reader copies the complete current method, adapting only
receiver/import paths. It retains observations but never delegates to production.
Frozen semantic test compares complete metadata and downcast rusqlite Debug/Display
errors over21 text/earlier-later-fault combinations, absent rows, distinct-time
selection and delta-derived version. Invalid timestamp fallback is checked within
real clock bounds rather than demanding impossible equal now instants. Missing-column
prepare errors are compared after renaming the original column. No new tie ordering
or corrupt-schema acceptance policy is introduced.

Public test uses six real persisted Edit compilations: NULL, empty, Unicode, long
text, blob and invalid UTF-8 TEXT. Each runs live and restarted history/metadata/
durable restore, yielding12 complete identity rows. Successful restore is required
for valid text/NULL cases. Complete history and compilation responses, metadata or
errors, durable result, source/live tuples/hash/cache state and database identity
are retained. Fixed created/updated times are fixture preparation, not production
normalization. Only temporary root strings (including Debug escaping) and binary
file_id are normalized; complete normalized binary bytes remain retained. All
other response/identity fields remain present. Reads must leave database fingerprint
and live/source/cache identity unchanged. Corrupt text errors and public fallback
responses are frozen instead of being presumed successful.

These are bounded controls, not all possible corruption scenarios. Existing
checkpoint/history/durable tests remain required in focused acceptance after any
change. Full original reader/public/budget source bytes are frozen by the driver.
Runtime evidence and complete oracle arrays belong under target, not scratch tests.

## Genuine original RED command

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-sqlite-text-budget.ps1 -Phase Before
```

Expected two tracked semantic/public passes,12 complete public identity rows, then
one intended final zero-materialization budget failure with all12 identities still
GREEN. Budget independently sums actual production discarded String count and byte
count after complete registered history/metadata/durable operations. Compilation,
fixture, incomplete rows, cross-run oracle or earlier semantic failure is invalid
RED. Driver freezes actual original SqliteStore, original reader/tests, complete
public array and all source hashes. No production optimization precedes inspected
valid RED. Warning-fatal focused checks, costs, real restoration and composed gates
remain later obligations. This command does not launch a server or field pilot.

After genuine RED, only the approved private same-column validated-text conversion
may replace Option<String>. Retain query, validation, metadata observations and
zero-owned-copy budget; reconcile historical design's source identity explicitly,
without relabeling its test-only measurements as current production acceptance.
No new helper/public policy is approved beyond this local conversion.

Then require unchanged-budget/exact public/durable/error GREEN, finite full-reader/
validation/error-order protection, all12 integrated construction-inclusive cost
populations, focused SQLite/checkpoint/history/persistence/binary and warning-fatal
Clippy/format/size/encoding. Consequential integrated regressions return for owner
review before mutation/composed. Independently restore the real original conversion
in isolated production: budget/static intended RED with semantic/public GREEN,
then byte-exact restored GREEN. Mandatory composed current-byte identities close
only after every required acceptance passes. No prior protected repair is reopened.

All runtime gates UNRUN. Agent performed bounded reads, diff, PowerShell parser,
strict UTF-8 and line checks only. No tests/build/formatter/Clippy/server started,
no stage/commit/push. Census unchanged13/9763 scoped,9750 unresolved plus96 supplements.
## First Before attempt rejected at test compilation

sqlite-text-before-20261108-172212 stops before semantic execution: the new public
oracle attempted to serialize CompiledIR, which intentionally has no Serialize
implementation. This is a test-authoring error, not original optimization RED.
The test now exhaustively destructures all three IR fields and retains file_id,
version and the complete ordered instruction Debug representation. No field is
omitted and no production type/trait is changed. Existing normalized binary bytes
remain independently retained. Failed evidence remains separate and unchanged.
Rerun the same Before command; no runtime or acceptance is claimed from this attempt.
Agent performed bounded source/diff/strict UTF-8 review only, no test/build rerun.
## Second Before attempt rejected at temporary storage-key identity

sqlite-text-before-20261108-172341 passes both semantic prerequisites, then fails
the budget's frozen public comparison before emitting complete identity rows. This
is invalid optimization RED. Inspection of full assertion records establishes they
are identical except the content_hash storage-key field: production deliberately
stores source_hash plus a hash of the temporary canonical path. Root-string
normalization cannot normalize an already hashed path.

The fixture now independently asserts that the RAW stored key equals the existing
source_hash:path_hash derivation and that the complete RAW database fingerprint is
byte-identical before/after reads. Only for cross-run oracle transport, that verified
path suffix becomes TEMP_PATH_HASH; source hash, payload, metadata and every response
field remain retained. This is fixture identity normalization, not a production
change or relaxed materialization budget. Existing failed receipts/oracles stay
untouched. Per-case equality failures now produce a bounded diagnostic instead of
printing the entire public corpus. Rerun Before to freeze the corrected oracle;
neither failed attempt counts as genuine RED. No agent runtime rerun performed.
## Genuine original RED inspected; approved conversion applied

Agent MaxHeadRoom inspected sqlite-text-before-20261108-173446 before optimization:
frozen semantics passes0.21s, complete12 public identities pass18.18s. Budget repeats
all12 complete identities and fails only the final intended materialization assertion
at16.93s:18 owned strings/786504 logical bytes versus zero. All1382 source hashes
match at inspection. Original reader/public/budget sources and original production
snapshot remain frozen. inspection.json records valid RED; earlier failed attempts
are preserved separately and do not count as RED.

Only the approved local conversion now changes: column4 remains Row::get with
Option, but a private FromSql validation type uses the same value.as_str() type/
UTF-8 validation and retains no owned text. Query, ordering, metadata extraction,
NULL semantics, error mapping and connection ownership remain unchanged. Successful
validation bytes/rows and conversion attempts remain separate from eliminated
ownership counters. The frozen original still performs and observes its actual
String conversion. No SQL removal or validation bypass is introduced.

User-owned current-byte execution:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-sqlite-text-budget.ps1 -Phase After
```

Expected three tracked passes: unchanged frozen semantics,12 complete frozen public/
durable identities and unchanged zero-owned-string/byte budget with12 identities.
No test/oracle weakening or regeneration. After is UNRUN; focused/static/integrated
cost, real restoration and composed acceptance remain outstanding. No FIX / PROTECTED
claim. Only bounded source/diff/UTF-8 review performed; no agent runtime, stage,
commit or push. All prior repairs remain protected and dense equality deferred.
## After GREEN inspected; integrated focused packet authored

Agent MaxHeadRoom inspected sqlite-text-after-20261108-201436: frozen semantic
pass0.12s, complete12 public/durable identity pass19.26s and unchanged budget
pass18.11s, zero failed/ignored. All1383 source hashes match at inspection. Size
passes10 active files and strict encoding2252 text files. Original reader/budget/
public test sources and all frozen original identities remain unchanged. No earlier
design or encoder receipt substitutes for actual current-production acceptance.

New finite source protection compares the complete production SQLite source to
exact genuine Before bytes plus ONLY module registration/type/observation replacement.
The helper's complete validation path is protected, including as_str, successful
validation observations and Option/column handling. Self-mutations reject restored
String ownership, different validation, different column and changed probe boundary.
A separate exact copy of the frozen original reader supports actual integrated
cost measurement without changing the frozen oracle's visibility or bytes.

Twelve integrated populations match design axes exactly. Both actual production
and frozen actual reader use identical common query/row/conversion observation
boundaries. Observations are enabled separately to require identical query/row/
conversion/validated-byte work, zero new discarded strings/bytes and original
ownership even for empty text. Timed24 alternating samples after3 warmups disable
updates symmetrically; full prepare/query/conversion/metadata/result/statement
teardown remains inside timing. Fixture/connection setup is outside. Residual
probe checks remain; no allocator/public/release or universal speedup claim.

Historical design source identity now uses the exact Before source text fixture
rather than requiring active production to retain the old reader forever. Its
actual necessary validation-byte assertion replaces the now-eliminated owned-byte
assertion; historical test-only receipts remain unchanged. Frozen repair/public/
budget oracles are not edited. New integrated measurements are authoritative for
this production change, not design-copy medians.

User-owned command:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-sqlite-text-focused.ps1
```

Expected eleven tracked groups, including original semantic/public/budget, new
static/integrated cost, SQLite and durable-compatibility tests, checkpoint reload,
read-only history and binary controls. Require twelve complete cost rows and
warning-fatal all-target/all-feature Clippy, format/size/encoding/source receipts.
UNRUN; inspect costs before authoring/running real restoration mutation or composed.
Consequential new regressions return for owner review. No further production change
after After; only test registration/assets added. Agent ran bounded source/parser/
UTF-8/diff review only. No stage/commit/push; this slice is not FIX / PROTECTED.
## Integrated focused GREEN; real restoration packet authored

Agent MaxHeadRoom inspected sqlite-text-focused-20261108-203851:57 tracked passes
across11 groups, zero failed/ignored. All1386 recorded hashes match at inspection.
Frozen/public/budget/static and integrated cost pass; SQLite18, durable compatibility7,
binary24 and checkpoint/history/frozen encoder controls pass. Warning-fatal all-target/
all-feature Clippy46.76s, both format logs empty, size14 active files and strict
encoding2256 text files GREEN. Original oracle/budget identities remain preserved.

Actual original / actual production complete-query medians in microseconds:

| Storage / bytes / requests / deltas | Original | Production |
| --- | ---: | ---: |
| Memory / 0 / 1 / 0 | 58.8 | 58.5 |
| Memory / 96 / 1 / 0 | 58.0 | 57.8 |
| Memory / 6144 / 1 / 0 | 68.0 | 67.4 |
| Memory / 393216 / 1 / 0 | 1486.6 | 1308.0 |
| Memory / 6144 / 16 / 0 | 1082.7 | 1069.4 |
| Memory / 6144 / 1 / 64 | 74.6 | 73.9 |
| File / 0 / 1 / 0 | 70.3 | 69.4 |
| File / 96 / 1 / 0 | 71.4 | 71.0 |
| File / 6144 / 1 / 0 | 77.1 | 76.7 |
| File / 393216 / 1 / 0 | 1787.3 | 1474.9 |
| File / 6144 / 16 / 0 | 1250.4 | 1239.8 |
| File / 6144 / 1 / 64 | 85.1 | 84.5 |

All medians favor production. Long-text index6..18 ranges overlap: memory original
1377.4..1616.2us versus production1219.0..1384.4us; file original1606.8..1976.4us
versus production1410.4..1703.9us. These are descriptive24 alternating debug samples,
not statistical or release/public guarantees. No consequential new regression is
demonstrated; proceed to authorized restoration acceptance. Counts separately verify
one query/row/conversion and equal validated bytes per operation, original one owned
String (including empty) and zero production ownership. SQLite retrieval/validation
and all metadata work remain. No allocator/engine-copy/peak-memory savings claimed.

Current production SHA-256, unchanged by subsequent test-only additions:

- sqlite_store.rs: bf0ea1362c09225c1cbd0c0c33d46ee64599d711e22e42bdfa1e026d38e97180
- metadata_text.rs: bef3e5f481835cd2fd7fbe48be32274763dd526d41f0aef1c7816e7dbf53e96c

User-owned next command:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-sqlite-text-mutation.ps1
```

UNRUN: isolated copy restores the COMPLETE real original load_latest method directly
from genuine Before. Current helper registration remains for an independent tracked
actual-type conversion law, which checks SQL NULL/type/UTF-8 error equality and
validation observations. This also keeps the real helper compiled in the mutant;
no warning suppression or production decision changes. The law is a test-only
addition after focused acceptance, checked by warning-fatal Clippy and baseline/
restored law runs in this packet. Existing frozen tests and timed cost body unchanged.

Require baseline static/law GREEN, two intended original budget/static RED with
12 public cases, frozen semantic/public two GREEN, then byte-exact restored five
GREEN. Active production never mutated; full active/isolated inventory must agree.
Failed compile/fixture/warnings do not satisfy RED. No raw giant log tails printed.

Tracked complete public fixture now preserves the exact original12-row oracle;
mandatory composed includes five new SQLite protections, expected at least97 passes
across42 groups. Composed remains UNRUN and follows inspected mutation acceptance.
No agent runtime, stage, commit or push. Not FIX / PROTECTED until all gates complete.
All previous corrections stay protected; dense equality deferred.
## Real original-method restoration acceptance inspected

Agent MaxHeadRoom inspected sqlite-text-mutation-20261108-210528. Warning-fatal
all-target/all-feature baseline Clippy passes5m11s; baseline static and actual-type
law pass. Restoring the complete genuine original load_latest method produces
intended zero-materialization budget RED16.95s (18 strings/786504 logical bytes)
and finite static RED. Frozen metadata/error semantics GREEN0.15s and registered
public/durable identity GREEN16.12s remain exact. All12 mutant public identities
complete in both budget/public logs; compile/warning/fixture failures are not RED.

Byte-exact restoration passes five tracked tests: budget19.97s, static0.01s,
actual validation law0.00s, frozen equivalence0.10s and public identity15.94s,
zero failed/ignored. Both restored public logs contain all12 cases. All2265 inventory
hashes match active and isolated restored bytes. Active production was never mutated;
SqliteStore SHA-256 remains bf0ea1362c09225c1cbd0c0c33d46ee64599d711e22e42bdfa1e026d38e97180.
Helper identity remains the focused identity above. inspection.json records checked
counts and inventory status. No production/test changes follow this receipt.

Only mandatory composed acceptance remains. User-owned command:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

Expected at least97 tracked passes across42 groups, including five new SQLite
protections and retained encoder/overload/other protections with final source
identities. UNRUN. This slice is not CLOSED / FIX / PROTECTED until composed GREEN
is inspected. No agent runtime, stage, commit or push. Dense equality remains
deferred and all previous repairs stay closed.
## Final composed acceptance: CLOSED / FIX / PROTECTED

Agent MaxHeadRoom inspected af-perf-001-20261108-212233:97 tracked passes across
42 groups, zero failed/ignored. All324 recorded source hashes match current bytes,
including all five new SQLite protections and retained previous repairs. Final
production identities match focused and byte-exact restored mutation acceptance:

- src/mcp/sqlite_store.rs:
  bf0ea1362c09225c1cbd0c0c33d46ee64599d711e22e42bdfa1e026d38e97180
- src/mcp/sqlite_store/metadata_text.rs:
  bef3e5f481835cd2fd7fbe48be32274763dd526d41f0aef1c7816e7dbf53e96c

inspection.json preserves checked group/pass/hash totals and final identities.
Evidence chain remains distinct:

- Original RED sqlite-text-before-20261108-173446: two semantic/public passes,
  twelve complete identities, intended final ownership failure18 strings/786504
  logical bytes;1382 matching hashes. Earlier invalid attempts remain separate.
- Current-byte After sqlite-text-after-20261108-201436: three passes, twelve exact
  public/durable cases, zero discarded ownership;1383 matching hashes.
- Focused sqlite-text-focused-20261108-203851:57 passes/11 groups, twelve actual
  integrated construction-inclusive cost rows, warning-fatal Clippy and formatting/
  size/encoding GREEN;1386 matching hashes.
- Real restoration sqlite-text-mutation-20261108-210528: clean static/law baseline,
  two intended budget/static RED, two preserved semantic/public GREEN, five exact
  restored GREEN and warning-fatal Clippy;2265 active/isolated inventory matches.
- Final composed af-perf-001-20261108-212233:97 passes/42 groups,324 matching hashes.

The accepted change removes only discarded String ownership. SQL selection/order,
column4 Option/null/type/UTF-8 validation, conversion/error ordering, timestamp
fallback, metadata/version, registered history, durable restore and source/cache/
read-only ownership contracts remain protected by the bounded controls described
above. Existing retrieval, UTF-8 validation, metadata allocation and SQLite engine
work remain. No query/schema/transaction/cache/index or stored representation change.

Local debug full-query medians and overlapping sample ranges remain recorded;
logical avoided copy bytes are not allocator events, peak memory, SQLite internal
copies, CPU instructions or public/release latency. No universal nonregression or
speedup claim. Finite malformed-row controls are not exhaustive corrupt-database
certification. Exact tied-row policy remains unchanged, without a new tiebreaker.

No production/test changes follow completed acceptance; closure updates evidence/
documentation only. No agent runtime, stage, commit or push. Repository publication
remains separate. Both encoder corrections, overload retrieval and all previous
repairs remain CLOSED / FIX / PROTECTED. Dense same-signature equality stays deferred.
Next campaign step is reassessment of remaining default-operation candidates, not
automatic dense-equality tuning or an implicitly approved implementation. Census
remains13/9763 scoped,9750 unresolved plus96 supplements.