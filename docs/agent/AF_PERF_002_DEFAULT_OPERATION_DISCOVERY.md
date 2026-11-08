# AF-PERF-002: resumed broad default-operation discovery

Agent: Agent MaxHeadRoom
Status: public characterization GREEN; encoder vector clone subsequently CLOSED / FIX / PROTECTED.
Historical discovery below retains its original proposed scope and observations.
Final repair evidence: [binary clone repair](AF_PERF_002_BINARY_CLONE_REPAIR.md).
Overload retrieval and all previous protected repairs remain CLOSED / FIX / PROTECTED.
Dense same-signature equality is deferred; no new matcher investigation performed.

## Acquisition and operation selection

Fresh Clean-CTX lifecycle acquisition reports degraded admission with
`frontend_retired: timeout`. This session exposes neither search_codebase nor
read_files fallback tools. Bounded host searches/known source reads supply the
trace; provider failure is not evidence of absent callers. No server launched.
The working tree was clean before this packet. This pass prioritizes uncovered
serialization/persistence-metadata work rather than repeating closed diff gates.
It is a bounded sample of default operations, not exhaustive system coverage.

Registry `create_default_registry` routes compress_code_context, save_context and
context_history to the real core/persistence/context handlers. Compression's
normal named response still persists binary IR when persistence is configured and
auto-save is enabled or effective fidelity is Edit. Binary wire need not be the
requested response encoding. `core/provide_persistence::persist_read_baseline_with_authority`
normalizes durable identity then calls binary_wire::encode. Public save_context
also encodes through its captured-authority checkpoint path, before comparing
against the durable state. Edit and delta publication are other source-traced
encoder callers, not public operations executed by this packet.

## Source-grounded shortlist, provisional order

| Rank | Candidate / exposure | Avoidable work hypothesis / risk |
| --- | --- | --- |
| 1 | binary_wire::encode clones table.strings().to_vec after completing the table and interning file identity; reached by persisted compression and checkpoints | One extra owned unique-string vector and its complete string payload per encode. Table remains alive and immutable through writing and operand retrieval, so borrowing its ordered slice appears sufficient. Small local correction; exact binary bytes, first-appearance ordering and all opcode/error contracts require independent protection before repair. |
| 2 | same encoder rebuilds HashMap<&str,u64> over every unique string despite StringTable already owning an equivalent reverse map | U entry attempts and offered key bytes, map capacity and teardown in every encode. A narrow StringTable index accessor might reuse existing authority, but requires separate design/approval and careful missing-operand panic/index semantics. No API or representation change made. Keep separate from rank 1's simpler clone elimination. |
| 3 | SqliteStore::load_latest selects pretty_text and row.get::<Option<String>>(4) materializes it into unused _pretty; context_history consumes metadata/version only | Returned text payload grows with stored presentation bytes and repeated reads, independent of history output. Durable restore also uses this metadata method. SQL column conversion errors and malformed/NULL rows are observable; dropping the column is not presumed equivalent across corrupt stores. This has narrower public exposure and broader failure-contract risk than rank 1. |

These are source priorities, not measured benefit rankings. The packet will decide
whether measured growth supports this order. At most rank 1 is the provisional
next implementation slice: remove only the redundant encoder vector clone,
retaining the separate reverse map and all encoding policy. No repair authorized
or implemented here. A measured recommendation follows inspected results.

Already-durable save_context still constructs tuples/IR/hierarchy/presentation,
binary and identities before equality validation. This is visible surrounding work,
not a fourth proposed optimization: the bytes and authority validations can be
necessary to establish sameness. No early-return, hash-only checkpoint or lock/
guard/currentness change is inferred from a no-op save result.

## Actual-boundary observations

Only cfg(test) calls are added to binary_wire::encode and SqliteStore::load_latest.
All other new code is tracked under src/tests, registered from lib.rs. Production
algorithms, stores, SQL statements, ownership, error propagation and guards remain
unchanged. Observations never feed matching, encoding, query or authority state.

For the encoder, count actual cloned string population/length/capacity, reverse-map
entry calls, offered key lengths, operand lookups/offered lengths, final map capacity
and binary output bytes. This includes setup and query work at their actual sites.
It does not measure hash/equality probe counts, actual bytes inspected, allocator
events, requested allocation bytes, peak memory, or all earlier StringTable
interning/temporary tuple costs. Capacity sums across calls are not peak storage.
Two earlier owned copies per unique string in StringTable::intern remain outside
the named cloned-vector count. Payload copying must not be called total request cost.

Metadata counts actual query attempts, successful rows and discarded owned text
length. It does not claim SQLite page reads, internal engine copies or allocation
events. The existing COUNT(deltas) establishes version and remains necessary here.

Public timings include complete synchronous dispatch through response construction
and local teardown, but exclude fixture creation/oracles. Test-only observations
add overhead, including a string-length walk over the clone; these samples cannot
attribute cost to either candidate or establish a latency improvement. No paired
replacement timing exists because no replacement is implemented. Teardown has no
independent counter/allocator measurement. No CPU ranking against deferred dense
equality can be established from logical byte counts or unlike debug timings.

## Characterization and semantic controls

Two tracked tests dispatch the actual registered public tools, serialized through
the existing response-capture guard. Ten real TypeScript fixtures independently
vary class count, methods per class, Unicode body payload and unchanged save count.
Each reports four intervals: cold Edit compression, first explicit checkpoint,
repeated already-durable checkpoint and checkpoint history (40 rows). Actual class,
method and body instruction populations are prerequisites, not inferred captures.
Binary unique-string counts/bytes are independently recomputed outside measurement.
Complete response Values are retained in rows. This is characterization, not a
frozen independent response oracle for a future repair.

Decoded durable identity and binary output size are checked. Checkpoints must retain
exact cold binary bytes; repeated saves require saved=0/already_durable=true. Source
bytes, live tuples/source hash remain identical. Existing checkpoint reload and
read-only history regressions run separately, plus full IR binary roundtrip.

Six additional history cases seed normal committed ContextStore metadata rows,
independently varying pretty-text bytes and request count. This directly exercises
public context_history against stored payloads, not source compiler integration.
Complete expected history text is asserted, with no invented live alias or semantic
ownership; public metadata/history output stays independent of pretty payload.
The real compiled-checkpoint history rows separately establish producer reachability.
No proprietary fixtures or enterprise extrapolation is involved.

Existing binary roundtrip/v04 tests cover opcode/identity transport; checkpoint
contracts cover reload/currentness and history contracts cover read-only ownership.
Missing for any subsequent correction: frozen original byte/response oracle across
all opcode families and fidelity states, genuine unchanged zero-copy budget RED,
malformed/missing operand behavior, finite clone-site protection, real restoration
mutation and current-byte affected/composed acceptance. This characterization is
not retroactively optimization RED. No such repair gates are run for discovery.

## User-owned execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-default-operation-discovery.ps1
```

Expected five tracked passes: two new characterization tests and three existing
checkpoint/history/binary controls. Require 46 complete operation rows, warning-fatal
all-target/all-feature Clippy, formatting, size and strict encoding. The driver
formats only the five packet files and freezes source hashes plus full rows under
target/af-perf-002. Eight protected matcher/edit/oracle file identities are checked
unchanged across execution; this does not rerun their closure. No composed gate,
binary rebuild, field switch, stage, commit or push is requested.

All runtime gates UNRUN. Agent performed only bounded reads/diffs, PowerShell parser
and strict byte/line checks. Return after execution: inspect exact receipts, rank
demonstrated avoidable work against exposure/risk, and propose at most one bounded
slice for owner review. Dense equality remains deferred absent stronger comparable
evidence. Syntax census remains 13/9763 scoped, 9750 unresolved plus 96 supplements.

## Executed characterization and ranked return

Agent MaxHeadRoom inspected default-operation-discovery-20261108-005910: five
tracked passes, zero failed/ignored, all 46 operation rows. New public tests pass
in 51.89s; existing checkpoint reload, read-only history and full binary roundtrip
controls pass in 4.13s, 2.80s and 0.00s. Warning-fatal all-target/all-feature Clippy
passes (2m26s), both format logs are empty, size passes six active text files and
strict encoding passes 2215 text files. All 1361 recorded source hashes match at
inspection, before this documentation-only update. Eight protected source/oracle
identities also match. No agent runtime was started. inspection.json retains
parsed work rows and the checked identity summary; original receipt/logs untouched.

| Public encoder fixture | Cloned strings / logical bytes per encode | Reverse entries / operand lookups | Reverse capacity |
| --- | --- | --- | ---: |
| One class / one method | 16 / 149 | 16 / 14 | 28 |
| One class / eight methods | 51 / 449 | 51 / 77 | 56 |
| One class / 64 methods | 331 / 3110 | 331 / 581 | 448 |
| One class / 256 methods | 1291 / 12916 | 1291 / 2309 | 1792 |
| Four classes / 16 methods each | 241 / 1367 | 241 / 593 | 448 |
| 16 classes / one method each | 91 / 454 | 91 / 209 | 112 |
| One class / 16 methods / 64 Unicode repetitions | 91 / 6986 | 91 / 149 | 112 |
| One class / 16 methods / 4096 Unicode repetitions | 91 / 394116 | 91 / 149 | 112 |

Each cold compression and explicit checkpoint encodes once. Already-durable saves
also encode once per request despite returning saved=0: four repeats clone 364
strings/3252 bytes; sixteen clone 1456 strings/13008 bytes in the unpadded fixture.
These observations establish independent unique-string, body-payload and request
dimensions. Preserving binary construction for currentness comparison remains
necessary; only the redundant owned vector is proposed for elimination.

Checkpoint history loads/discards 431, 4256 and 16280 pretty-text bytes for the
one-, 64- and 256-method fixtures. Long-body checkpoint history loads/discards
394544 bytes without encoding. Independently seeded committed rows confirm exact
linear discarded payload: 0/96/6144/393216 bytes for single reads, then 24576 and
98304 bytes for four/sixteen reads of the 6144-byte row. All exact expected history
responses and read-only ownership assertions pass. Seeded rows are not claims of
compiler-produced payloads; checkpoint rows establish the real producer separately.

Final ranked shortlist for engineering priority, not measured CPU dominance:

1. **Encoder vector clone: FIX PROPOSED.** Directly observed complete payload copy
   on real persisted compression/checkpoint paths, including repeated saves.
   Borrow table.strings() instead of to_vec, keeping the existing separate reverse
   map, lookup/panic behavior and byte order unchanged. Smallest local correction;
   no representation, public API or authority change appears required.
2. **Encoder reverse-map reconstruction: active separate candidate.** Actual entry
   calls grow to 1291/map with capacity1792; offered key lengths reach394116 bytes
   in the long-body case. Existing StringTable already retains a reverse map, but
   lookup reuse requires its own accessor/missing-key design and cost comparison.
   Actual hash probes, comparison work and allocator costs are unmeasured. Do not
   bundle this with vector borrowing or call all operand lookup work unnecessary.
3. **Metadata unused pretty_text: active separate candidate.** Materialization is
   demonstrated on compiled checkpoint and seeded metadata paths. Public history
   output needs no text payload. Broader restore callers and conversion-error
   behavior increase semantic risk; investigate failure-contract equivalence before
   removing the column/read. No SQL or ContextStore contract change is proposed now.

Construction/map/lookup/capacity observations are retained separately from logical
copy lengths. Full public elapsed samples include parsing, authority, persistence,
formatting and diagnostic overhead; no candidate CPU attribution or paired benefit
is established. Allocator traffic, peak live storage and actual inspected hash bytes
remain unmeasured. Default-operation frequency in real workspaces is also unknown.
This shortlist therefore does not establish which operation consumes most runtime.
There is no comparable evidence warranting reopening dense same-signature equality;
it remains deferred independently of this actionable low-risk copy candidate.

Owner review requested for **one slice only: encoder vector borrowing**. Before
production edits, freeze complete binary/public/identity semantics, author a separate
zero-vector-clone budget, and execute genuine original RED with semantics GREEN.
Retain reverse-map accounting and all necessary encoding work. Then unchanged-budget
GREEN, byte-identical all-opcode/fidelity/Unicode/duplicate controls, real clone
restoration RED/restored GREEN, focused checks and composed acceptance are required.
This passing characterization is not optimization RED. No implementation, staging,
commit or push follows this discovery return. All protected repairs remain closed.
