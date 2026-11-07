# AF-PERF-002: default structural-edit relocation discovery

Agent: Agent MaxHeadRoom
Status: bounded public characterization GREEN; exact-end join correction owner APPROVED.
Approved correction is CLOSED / FIX / PROTECTED after original RED, exact semantic/
work GREEN, focused/mutation/cost and final75-pass/242-hash composed verification;
see the [repair report](AF_PERF_002_EDIT_EXACT_END_REPAIR.md).
Historical characterization stages below are retained. No correctness-contract change. A4 retrieval remains
CLOSED / FIX / PROTECTED; no A4 tests requested to reconfirm that repair.

## Default reachability and prior-scope reconciliation

registry.rs registers apply_edit -> tool_handlers/edit.rs::handle_apply_edit.
This is a shipped public write operation, independent of replay's already-closed
per-delta characterization. No replay/incremental-validation work is reopened.
The coverage ledger previously listed edit/diff paths as uncharacterized; prior
method-diff matching/source inspection does not measure structural edit relocation.

Path: parse operations and enforce existing cap -> path/exclusion/resource/hard-link
and tracked-owner gates -> begin publication and recover pending intent -> read
exact source bytes -> compare actual/live/durable source identity -> compile current
Edit-fidelity candidate -> UnitTable::from_instructions_with_declarations -> resolve
all operations, verify expected text, disjointness and splice in memory -> new-source
syntax gate -> compile target candidate and hierarchy/hash checks -> commit boundary.

UnitTable has an actual default caller here; its public library helper usage is not
substituted for request reachability. Initial provide_code_context establishes the
owned baseline before the measured apply_edit call. No fabricated IR admission.

## Ownership, currentness and failure boundaries

The pre-edit compilation is over the just-read source that matched existing owner
identity. The separate declaration parse establishes syntax-owned whole-declaration
boundaries and leading metadata/comment ownership. These are not declared redundant
merely because another compilation parsed those bytes. Existing body-only spans do
not automatically authorize deletion/insertion of whole declarations.

Planning and syntax/target-candidate checks precede commit_lock. Publication commit
rechecks disk bytes and prepared authority under existing boundaries. It stages exact
target bytes, records a recoverable intent with prior/target source and durable IR/
semantic material, atomically replaces source, commits durable semantics, then
publishes live IR/snapshots/index and cache. Failed durable commit attempts exact
source recovery and preserves recovery intent when needed. Cache/discovery retirement
and external response follow. No global transaction across every lock is inferred.

The new packet does not change compilation, parsing, locking, intent copies,
physical-owner normalization, hash/authority checks, persistence or publication.
Those copies may serve crash recovery/ownership. A1's handler clone repair does not
authorize their removal. Existing stale-source, candidate-failure and real durable-
failure controls remain separate semantic evidence selected by this packet.

## Concrete repeated retrieval candidates

In edit/locate.rs, declaration attachment iterates every unit and filters the
complete captured declaration Vec, then chooses min_by_key(end-start). The predicate
requires start <= unit.start_byte and exact end == unit.end_byte. Original capture
duplicates, shortest-containing selection and tie behavior matter; they are not
equivalent to arbitrary name or span-key overwrites.

A second loop visits every deduplicated syntax declaration and calls units.any
with matching end and contained body start. This determines whether the declaration
becomes an UneditableRecord. Its results feed ambiguity and refusal: a faster join
must preserve cases where editable and noneditable declarations share a selector.

Source-proven growth: U body-backed units and D declaration captures cause U*D
attachment predicates. S syntax declarations cause at most S*U editability tests,
with actual prefix length depending on declaration/unit order and misses. This is
not executed scaling evidence yet. A potential end-key grouping would need explicit
construction/duplicates/ties/ordering proof before production change; no new map is
implemented or justified by asymptotics alone.

Resolution already uses method-ID, signature and qualified-name indexes in that
order. Bare fallback scans units, and qualified/bare selection also scans residual
noneditable records to preserve ambiguity. These are distinct from table construction.
Batch operation population Q can multiply resolution without rebuilding the table.
Existing disjoint-range validation sorts planned edits and checks adjacent windows;
do not misreport that implementation as all-pairs overlap work.

## Actual-boundary observations and fixture design

All instrumentation is cfg(test). EditWork records current/target compilation
attempts separately (requests, not compiler-internal parse counts); table builds,
actual instruction visits, declaration parse requests/source lengths, editable units,
captured declarations, deduplicated syntax declarations, exact attachment predicates,
short-circuit editability predicates and target-resolution/bare/noneditable predicates.
No allocator events/RSS, true text-search CPU bytes or full parser work is measured.
Rendering, persistence and response capture contribute to public elapsed time.

New tracked edit_relocation_work_af_perf_002.rs dispatches actual registered
provide_code_context then apply_edit with a real SQLite store. Baseline setup is
outside the measured edit interval. Twelve success fixtures independently vary:

- Method population1/2/8/32/128/256 with one qualified operation.
- Noneditable field population16/128 at16 methods.
- Batch size8/16 at32 methods, within the existing64-operation cap.
- Unique bare lookup versus qualified lookup.
- Source-byte padding4096 independently of declaration population.

Fixtures retain Unicode prefix bytes and CRLF. Equal-length method replacements
keep exact offsets independently predictable. Tests require exact final disk bytes,
operation response spans/replacement bytes/deltas, one current and one target
compilation attempt, one relocation parse/table, expected method population and
actual U*D attachment work. They check physical-owner durable IR/version/hash,
live version/hash and every persisted Body byte span against final source. This
is finite TypeScript production-boundary evidence, not all-language certification.

Three independent refusal fixtures exercise ambiguous bare method, noneditable
field and mismatched expected body. They require unchanged exact disk/durable binary/
live tuples/version and only the current-source compilation attempt. Existing
comment ownership and location tests plus source/candidate/durable failure controls
are selected separately. Characterization assertions describe present work; none
are a future optimization budget or retroactive RED.

## User-owned first execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-edit-relocation-work.ps1
```

Expected two new tracked characterization tests with15 printed operation rows,
existing locator group, one whole-declaration comment refusal and three public
stale/candidate/durable failure controls; all must pass, none ignored. Wrapper also
runs warning-fatal all-target/all-feature Clippy, format/size/encoding and freezes
source hashes. No process/server/binary field switch is needed for this first packet.

All tests/gates UNRUN. Agent performed bounded source/text/PowerShell syntax review
only, no Cargo/formatter/runtime. Production behavior unchanged: only test observers
at actual predicates/attempt boundaries. Existing protected receipts stay frozen;
their hashes predate this instrumentation and are not claimed current verification.

Return after execution: actual independent dimensions, necessary versus unnecessary
work, exact semantic bounds, construction requirements and smallest defensible
FIX proposal/ACCEPT/MONITOR. Stop for architectural review if correction requires
different authority, transaction, ownership, normalization or public behavior.
Do not conflate table retrieval with replay D*N or reopen A4 to explain this result.

Coverage advances only for these named production operations. Compaction, rendering,
diff and broader edit paths remain separate; syntax census unchanged13/9763 scoped,
9750 unresolved plus96 callback supplements. No stage/commit/push performed.

## Executed characterization and independent growth

Agent MaxHeadRoom inspected edit-relocation-work-20261106-154640:16 tracked passes,
zero failed/ignored. New public growth/refusal2tests62.94s; existing location10tests
0.00s; comment-ownership refusal1test0.66s; stale-source1test5.74s; real durable
failure1test4.57s; candidate failure1test3.72s. Warning-fatal all-target/all-feature
Clippy1m06s, empty formatting logs, size6 active files and strict encoding2153 text
files pass. All1323 recorded source hashes match at inspection. inspection.json
preserves15 parsed actual-operation rows and grouped results. No agent runtime.
HEAD at inspection7afaf72c records the earlier A4 repair; no A4 work reopened.

| Methods U | Fields | Operations Q | Captures D | Syntax S | Attachment predicates | Editability predicates | Public us |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0 | 1 | 1 | 2 | 1 | 2 | 1796793 |
| 2 | 0 | 1 | 2 | 3 | 4 | 5 | 2348938 |
| 8 | 0 | 1 | 8 | 9 | 64 | 44 | 1814171 |
| 32 | 0 | 1 | 32 | 33 | 1024 | 560 | 1825983 |
| 128 | 0 | 1 | 128 | 129 | 16384 | 8384 | 1960395 |
| 256 | 0 | 1 | 256 | 257 | 65536 | 33152 | 2205570 |
| 16 | 16 | 1 | 16 | 33 | 256 | 408 | 1846685 |
| 16 | 128 | 1 | 16 | 145 | 256 | 2200 | 1906784 |
| 32 | 0 | 8 | 32 | 33 | 1024 | 560 | 1767702 |
| 32 | 0 | 16 | 32 | 33 | 1024 | 560 | 1718623 |

In these ordinary-method fixtures D=U, S=U+fields+1. Attachment is U*D and
editability is U*(U+1)/2 + U*(fields+1), because methods match progressively while
fields/class have no body-end match. These formulas are fixture-specific, not
universal language/capture bounds. Q8/Q16 retains one table construction while
residual uneditable resolution tests grow8/16. Unique bare selection at U16 adds
16 unit predicates; qualified selection adds none. Padding4096 retains U16/D16/
S17 and256/152 join predicates. Byte population, unrelated declarations and request
operation population are independent costs; neither join is multiplied by Q here.

All three new refusals see U9/D9/S19 and81/135 join predicates before refusing.
Bare ambiguity/noneditable field also visit9 bare units and10 noneditable records;
mismatched expected text has no bare scan and10 noneditable visits. Each makes
one current-source compilation attempt and zero target attempts, preserving exact
disk bytes, live tuples/version and durable binary. Successful requests make one
current and one target attempt, one table and declaration parse, exact response
replacement bytes/spans/deltas and final disk/durable/live identity checks pass.
Existing durable/candidate/stale failure controls remain independently GREEN.

Complete public timing includes compilation, syntax, persistence, response/capture
and observation overhead. These single samples do not attribute a latency fraction
to joins, establish enterprise performance or measure allocator traffic. Original
characterization is GREEN, not an optimization budget RED.

## Smallest correction: one operation-local exact-end lookup

Disposition FIX PROPOSED for the two declaration/unit joins only. No optimization
implemented. Existing UnitTable identity indexes do not index end spans; using them
by name would not preserve capture authority or overlapping declaration semantics.

Proposed local representation, scoped to from_instructions_with_declarations:
one BTreeMap<u64, EndBucket>, with each bucket containing the original ordered
declaration tuples and Option<u64> maximum body start. No persistent field/cache,
new synchronization, authority or public contract. The existing declaration Vec
can be consumed into these buckets after capture/observation, moving tuples rather
than cloning them. Visit units once to accumulate maximum start for each exact end.
Do not add a separate name/span index to UnitTable or change resolution.

Attachment looks up unit.end_byte, then retains the original start predicate and
min_by_key(end-start) over that bucket's declaration Vec. Relative capture order,
duplicate occurrences, insertion/comment marker and first minimum tie are unchanged.
It must not overwrite a bucket with one declaration or sort/deduplicate its Vec.

For syntax declaration(start,end), original existential condition is exactly:
there exists a unit with that end whose start >= start. A present maximum start
at that end satisfies the same inequality iff such a unit exists. Missing maximum
means false; no zero/default sentinel. This is a boolean summary of the existing
predicate, not new authority or a replacement span. Table attachment changes only
declaration_span/insert_before/blocker, not body start/end, so a summary built before
that loop remains invariant. Syntax declaration traversal/dedup and uneditable
record emission order remain exactly as today.

Thus output UnitRecord metadata and UneditableRecord membership/order are unchanged,
which preserves selectors, ambiguity candidates, refusal, byte boundaries and
subsequent plan/commit behavior. The current/target compilations, declaration parse,
source/authority checks, locks and transaction stages remain in their old positions.

Construction adds D tuple moves and U maximum updates plus map lookups; let K be
distinct ends across both populations. Costs are O((D+U) log K) key operations,
U attachment lookups, S boolean lookups and visits to only same-end declaration
candidates. Retain K map nodes/buckets and D declaration occurrences with per-bucket
Vec capacity; replace the original D-tuple Vec rather than duplicate its payload.
Per-bucket Vec capacity/node overhead is real additional cost to measure. Small
fixtures can incur map overhead. Same-end density can retain repeated candidate
work; do not claim elimination of all quadratic edit work. For the measured distinct
method ends, expected attachment candidate visits are U rather than U*D, and
editability uses S lookups rather than the recorded prefix scans.

No larger ownership/transaction change is required by this proposal. A sorted-end
vector alternative is possible but would still need a separate maximum summary
or inverse search; the one local bucket map serves both queries directly. Do not
introduce thresholds, cross-call reuse or parser/IR reuse incidentally.

## Pre-implementation and acceptance obligations

Before any production correction, author deterministic original work-budget RED
with setup/insertions/key comparisons, retained tuples/Vec capacities/buckets,
matched same-end candidate visits and both lookup populations separately measured.
Retain this passing characterization as original evidence. A compile/oracle failure
cannot satisfy RED. No such optimization RED is claimed in this return.

Freeze original join algorithms and compare complete declaration metadata and ordered
uneditable records for missing ends, duplicate captures, identical end/different
start, equal minimum ties with different comment/insertion authority, nested units,
reversed capture/unit order, duplicate body ends, zero/empty inputs, fields/classes,
overloads and ambiguous selectors. Retain exact boundary predicates and tie policy.
Use real language captures for leading attributes/comments and nested declarations;
helper tuples alone cannot certify parsing or callback ownership.

Then unchanged public budget GREEN, exact complete UnitTable/resolve/refusal and
public byte/span/durable/live equivalence; construction-inclusive small/large/
same-end-density cost comparison; finite static known-join coverage; isolated real
restoration of each old join budget/static RED with semantics preserved and exact
restored GREEN. Existing source/currentness/candidate/durable failure controls,
warning-fatal lint/format/size/encoding and composed acceptance remain required.
No further execution is requested merely to reconfirm this characterization.
Next implementation step is original budget/oracle authoring; no production edit
precedes valid RED. A consequential construction regression or required change to
authority/transaction/boundary semantics must return for owner review.

Other edit resolution, source parsing, splice construction and durability copies
remain distinct obligations. A4/replay/stats stay closed. The syntax census is still
13 scoped/9763 original,9750 unresolved plus96 supplements. No commit/stage/push.
