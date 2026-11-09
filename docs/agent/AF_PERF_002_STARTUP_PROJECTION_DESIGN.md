# AF-PERF-002: startup path projection bounded design investigation

Agent: Agent MaxHeadRoom
Status: executed design GREEN; preserving projection DEFER; count-removal REJECT.
All completed corrections remain CLOSED / FIX / PROTECTED. Dense equality deferred.

## Default startup trace and authority

McpState::new opens configured SQLite persistence, wraps it in BufferedStore,
initializes empty session accounting and a new PhysicalIdentityRegistry, then calls
sqlite.list_contexts(i64::MAX as usize). Only on complete Ok does it iterate every
summary and register_durable_owner(context.file_path). CBM lifecycle/state assembly
follows this block. Errors are discarded with if let Ok: no paths, including valid
rows preceding a failing row, are registered. Streamed registration would violate
this all-or-nothing contract. New session stats remain empty; persisted statistics
are a separate consumer. No constructor order, failure policy or authority change.

Shared list_contexts prepares one statement: SELECT path/fidelity/token counts/
updated_at plus correlated COUNT(*) from deltas WHERE context_id=c.id, ORDER BY
updated_at DESC LIMIT parameter. It converts fidelity first to i32, maps its label,
then constructs a summary in field order: owned path, owned fidelity label, raw i64,
compressed i64, owned timestamp, delta i64. Paths and fidelity/timestamp ownership,
summary vector and query/sort/count machinery coexist. Public list_sessions needs
those fields; startup uses only paths but still inherits every conversion and query
error. The SQL is extracted verbatim from current source for both comparison modes.

First durable spelling for a physical file is significant. PhysicalIdentityRegistry
uses actual filesystem identity, retains durable_path on first resolution and
returns that spelling thereafter. Multiple roots, hardlinks, duplicate paths and
SQL order cannot be normalized away. Tied timestamps have no extra tie breaker;
the candidate must retain the exact same SQL rather than invent deterministic order.

Fresh Clean-CTX admission remains degraded frontend_retired timeout. No server
started. Bounded known-source host reads supply this trace; failure is not absence.

## Delta-count removal is an error-contract question

A negative control removes COUNT only in a rejected scratch query. After DROP TABLE
deltas or renaming its context_id column, the actual complete reader must error
while SELECT file_path FROM contexts can still succeed. The tracked test requires
that difference and logs it explicitly. This is NOT a viable optimization candidate.
The original error prevents registration; succeeding would change authority even
if no caller sees a surfaced startup error. Do not hide that as unused validation.

Without separately approved failure-policy changes, retain count computation and
its schema/error boundary. No new metadata existence query or two-phase workaround
is selected. A preflight probe alone would not prove equivalence for all query/
engine failures. Real indexed count work therefore remains in the preservation
candidate. This packet cannot promise removal of the largest measured delta axis.

## Smallest comparison candidate

Two symmetric test-only query modes use the same SQL, complete conversion order,
row staging and count computation. Original mode constructs actual summary structs;
projection mode returns a Vec<String> of paths, validates discarded timestamp via
FromSql value.as_str() rather than owning it, and keeps the fidelity label borrowed.
Both still validate fidelity and token counts and convert delta_count. Original
summary results are compared in full against actual production; all ordered paths
and rusqlite Debug/Display errors are compared across actual/copy/candidate.
No production function delegates to these copies. No persistent structure, query
rewrite, index, cache, public reader replacement or initialization change.

This candidate still stages ALL paths before registration. It does not register
incrementally, deduplicate paths or sort again. It removes only unused summary and
two string ownerships per successful row; count/sort/validation costs remain.
The projection validates timestamp with the same as_str conversion as String; row
get retains SQL type/UTF-8 errors and their original column boundary.

## Controls and measured boundaries

Three tracked design tests cover:

- Empty/populated stores, two root spellings, duplicate paths and tied order;
  NULL/storage-class/malformed UTF-8 metadata, unknown fidelity fallback, token
  COALESCE and negative values, simultaneous conversion faults, missing contexts
  and missing/malformed deltas schema. Relaxed scratch metadata schema permits
  storage classes disallowed by normal NOT NULL columns, through the ACTUAL reader.
- Actual default startup with two declared roots and real hardlinks. Valid state
  retains the most-recent durable spelling. A malformed later row makes the full
  read fail; resolving the other spelling first proves no earlier row was partially
  registered. State exposes no live alias/semantic edges merely from startup.
  Candidate ordered paths/errors are independently compared, not integrated startup.
- Twelve existing context/delta populations in a normal file-backed temporary
  schema; actual complete production summaries and ordered paths agree. Complete
  test-copy query/prepare/validation/materialization/result/statement teardown is
  timed for24 alternating samples after3 warmups. Fixture/connection preparation,
  physical registration and full startup are outside these paired timings.

Enabled source-site observations count statement attempts, successful rows, six
successful conversions per row, paths/summary entries, owned strings/logical bytes,
validated timestamp lengths, count results/summed returned counts, retained vector
capacity and SQLite VM/fullscan counters. Partial failing-row conversion attempts
and SQLite internal indexed row visits are NOT counted. count_results are values
returned by correlated COUNT, not independently issued SQL statements. delta_total
is a result sum, not actual rows physically read. Existing covering index remains.
No actual hash/allocator events, peak live memory or physical I/O measured.

Timed modes disable updates symmetrically; both share the same mode branch and
observation sites, including residual if checks. Original-only ownership work is
real in its mode. Each includes an unused empty Vec header for the other mode,
which allocates nothing; this shared test apparatus limits extrapolation. Header
sizes, capacities and payload counts do not measure allocator savings. No public
startup latency, release improvement or statistical/universal guarantee follows.

Actual public list_sessions committed-only regression runs separately, unchanged.
No candidate integration, original optimization RED, mutation or composed gate.
Protected source identities checked before/after include startup, SQLite text
validation, encoder/table, matcher and edit repairs. No production edits here.

## User-owned execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-startup-projection-design.ps1
```

Expected four tracked passes (three design and one existing public control), twelve
paired cost rows and two rejected COUNT-removal error cases. Warning-fatal all-target/
all-feature Clippy, formatting, size, strict encoding and complete source receipts
required. Full rows/logs stay under target; short console summaries only. All runtime
UNRUN. Agent performed bounded source/parser/UTF-8/diff review only. No publication.

Return after execution: FIX/DEFER/REJECT with measured local results and exact error
limits. If projection gains are negligible beside retained COUNT work, prefer DEFER
rather than invent a more aggressive algorithm/policy. If favorable and equivalent,
propose at most a startup-only projection preserving the shared reader and SQL/error
contract, with owner approval, genuine original RED, frozen ordering/public/physical
identity oracle, actual integration/cost review, real restoration mutation and
composed acceptance required before closure. Do not implement it from this design
approval. Per-root inventories remain unmeasured, listing scaling monitored and
dense equality deferred. Census unchanged13/9763 scoped,9750 unresolved plus96
supplements. No stage, commit or push.
## Executed comparison: DEFER projection, REJECT count removal

Agent MaxHeadRoom inspected startup-projection-design-20261108-215733: four tracked
passes, zero failed/ignored. Three design tests80.23s and existing public committed-
only listing1.37s pass. All twelve paired populations and24 samples per mode/case
complete. Warning-fatal all-target/all-feature Clippy38.93s, both format logs empty,
size22 active files and strict encoding2265 text files GREEN. All1391 recorded
hashes match at inspection before this documentation-only update; all eleven
protected identities unchanged. inspection.json preserves complete work/sample rows
and disposition. No production correction, mutation or composed gate executed.

Microsecond medians from symmetric test-only complete query/projection/drop:

| Contexts / deltas per context | Summary | Validated path projection |
| --- | ---: | ---: |
| 0 / 0 | 65.5 | 71.4 |
| 1 / 0 | 70.2 | 70.4 |
| 16 / 0 | 99.3 | 94.3 |
| 128 / 0 | 370.7 | 325.8 |
| 256 / 0 | 747.6 | 659.2 |
| 1 / 16 | 71.5 | 71.7 |
| 16 / 16 | 131.6 | 126.6 |
| 128 / 16 | 603.4 | 558.6 |
| 256 / 16 | 1452.1 | 1240.8 |
| 1 / 128 | 82.1 | 82.3 |
| 16 / 128 | 259.6 | 257.3 |
| 128 / 128 | 1629.3 | 1569.9 |

128/0 central sample indices6..18 are365.9..386.3us versus320.6..332.0us;
256/0 are734.8..780.2us versus625.6..677.6us. Those show separated descriptive
ranges for45/88us median savings.256/16 has broad overlapping1251.2..2407.0us
versus1165.4..2163.2us, so its211us median difference is not a reliable uniform
saving.128/128 also overlaps1605.3..1665.0us versus1533.8..1654.1us. Empty
projection median is5.9us higher with overlapping ranges; singleton differences
are0.2us. No production regression or improvement follows from these debug copies.

At128 contexts, both issue one statement, return128 paths/counts, perform768
successful conversions, validate2432 timestamp bytes and retain128 vector slots.
Original constructs128 summaries and384 owned strings/5504 logical payload bytes;
projection constructs zero summaries and128 strings/2560 bytes. Thus256 discarded
string ownerships/2944 logical bytes disappear in the model. On the executed target
summary header is96 bytes versus path String24: logical element-header extent for
128 initialized entries is12288 versus3072 bytes, excluding allocator/control/
capacity overhead and common structures. These are layouts and retained slot
counts, not allocator events or measured peak memory.

SQLite VM work remains IDENTICAL:128/0 is4235 steps,128/16 is10378 and128/128 is
53386 in both modes. Count-result totals0/2048/16384 are identical. These totals
are returned count sums, not actual index row visits or CPU instructions. Preserving
errors retains the dominant measured delta-dependent work; no count removal benefit
is smuggled into the projection timings. Physical owner registration/full startup
are outside these paired samples. No total-startup, release/public latency, allocation
saving or statistical/universal nonregression claim.

### Error and ownership findings

Complete production/copy summaries, ordered projected paths and exact rusqlite
Debug/Display errors agree in executed valid/malformed/NULL/schema controls. Token
NULL values follow existing COALESCE; unknown fidelity follows low fallback. Tied
order and duplicate paths remain the query's original order, without deduplication.
Actual startup with two declared roots and real hardlinks retains the most recent
path spelling. A malformed later row causes the complete reader to fail; resolving
the other spelling first proves no valid earlier row was partially registered.
No alias/semantic edges are created by durable enumeration alone.

Both negative count-removal cases execute as intended: dropping deltas produces
SqliteFailure/no such table; renaming context_id produces SqlInputError/no such
column d.context_id with original SQL/offset. The path-only scratch query succeeds
in each case. That would change startup's all-or-nothing durable-owner registration,
so count removal is REJECTED under the current preservation contract. A mere schema
preflight is not certified equivalent for other engine failures. No policy exception
or extra query is proposed.

Bounded tests are not proof for every corrupt schema, SQLite resource failure or
filesystem identity condition. The shared SQL and same conversion ordering support
the source argument for the preserving projection; a future repair would still
need frozen independent actual startup/public/error and physical-identity oracles,
genuine original ownership-budget RED, actual integration/costs, isolated real
restoration mutation and composed acceptance. Passing design copies are not RED.

Protected production identities remain:

- state.rs: aa912912876e10206517f86580b9ba79dd3b0b75b4e918363e98c25d5dff5c67
- sqlite_store.rs: bf0ea1362c09225c1cbd0c0c33d46ee64599d711e22e42bdfa1e026d38e97180
- metadata_text.rs: bef3e5f481835cd2fd7fbe48be32274763dd526d41f0aef1c7816e7dbf53e96c

All comparison test/source identities are retained in receipt.json; no production
implementation delegates to them. Historical discovery samples remain distinct.

### Returned disposition

**DEFER the preserving projection.** It demonstrates avoidable owned summary work
with bounded semantic equivalence, but typical measured savings are tens of
microseconds and count-heavy retained work limits benefit. Real startup frequency/
populations and consequential total-operation benefit are not established. Adding
a separate reader and its conversion/order/authority protections is not currently
justified by this evidence. This is an engineering-priority judgment, not a new
global optimization threshold or claim that45us can never matter.

No implementation slice is recommended now. Keep this measured candidate for
reconsideration only with stronger exposure evidence. Count-elimination is rejected
under existing semantics; do not pursue more speculative SQL/preflight variants.
Next discovery priority is bounded characterization of per-root inventory filtering/
cloning, still unmeasured and not authorized for production correction. Public
listing scaling stays monitored; dense same-signature equality remains deferred.
No further timing requested solely to obtain favorable samples. All completed
repairs stay CLOSED / FIX / PROTECTED. No stage, commit or push; census unchanged.