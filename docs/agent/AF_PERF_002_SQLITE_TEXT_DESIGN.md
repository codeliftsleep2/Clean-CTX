# AF-PERF-002: SQLite unused text bounded design investigation

Agent: Agent MaxHeadRoom
Status: test-only comparison executed GREEN; validated text borrowing FIX PROPOSED.
Both encoder corrections and all previous repairs remain CLOSED / FIX / PROTECTED.
Dense same-signature equality remains deferred. No stage, commit or push.

## Production boundary and canonical questions

Fresh Clean-CTX admission reports degraded frontend_retired timeout. No server
started. Known-path host reads establish the trace; acquisition failure is not
absence of callers. Context history's registered handler calls SqliteStore::load_latest
through the persistence store, then may fall back to ContextStore. BufferedStore
also delegates metadata reads. SqliteStore::load_durable_context in semantic_state.rs
calls the same trait method before baseline/delta and semantic-state restoration.
The latest row is selected by updated_at DESC LIMIT 1, without an additional tie
breaker. Its id, path, hash, fidelity, pretty_text, timestamp, token counts and delta
count are converted in that exact order. Version remains COUNT(deltas)+1.

The unused Option<String> conversion is observable: NULL succeeds; invalid SQL
types or invalid UTF-8 can fail before later metadata conversion. Public history
swallows a metadata error and may use fallback state; durable restore maps it to a
physical failure. Removing the selected column is therefore not assumed equivalent.
Checkpoint_matches separately needs actual presentation text; it is outside scope.

Question matrix: caller/reachability is source-traced above and existing registered
history/checkpoint tests remain independent controls. Ownership is the committed
SQLite row; no pending write is published by reading. Currentness/ordering is the
existing SQL, unchanged in both test copies. Completeness is exactly the selected
row plus existing delta count, not an inferred database-wide truth. Refusal/error
order is compared for malformed text and simultaneous earlier/later faults. Full
future public integration, restore error mapping and original optimization RED are
not established by a passing test-only comparison.

## Finite comparison and error contract

No production source is edited. Two test-only modes share a copied complete
load_latest method, except imports, receiver and observation removal. A tracked
source identity control verifies that exact relation after whitespace normalization.
The sole branch selects original Option<String> or Option<ValidatedText> at column4.
ValidatedText implements FromSql through value.as_str().map(|_| Self). Locally read
rusqlite0.32.1 implements String through value.as_str().map(ToString::to_string).
Thus both perform the same SQL-type/UTF-8 validation through Row::get at the same
column; the candidate omits only the final owned copy. Option retains NULL behavior.
This is a proposed local conversion form, not permission to implement it.

Tests compare complete metadata Debug representations with fixed valid timestamps,
and downcast rusqlite error Debug plus Display for NULL, empty/Unicode, numeric
values under real TEXT affinity, blob and invalid-UTF8 TEXT. Each combines no fault,
a later token-column fault or an earlier fidelity fault. Affinity can convert numeric
values to text; these cases do not falsely assert they must fail. Missing rows and
two rows with distinct update times protect selection; no new tied-row policy.
Malformed timestamps retain the existing now fallback in both source copies but
are excluded from exact-time comparison. Additional corrupt-schema, missing-column,
invalid-date clock bounds and full restore/public failure controls would be needed
before an eventual repair. No assertion of exhaustive corrupt-database coverage.

## Cost boundary and actual production controls

Twelve populations vary in-memory versus temporary file-backed SQLite, text payload
0/96/6144/393216 bytes, one versus16 requests, and0 versus64 delta rows. Temporary
file cases use the normal SqliteStore schema/WAL configuration, not cold-disk or
physical-I/O claims. Fixtures/connections are prepared outside timing. Each complete
sample includes statement preparation, binding, query/row stepping, conversions,
metadata construction and row/statement/result teardown. Both modes have the same
branch and no production diagnostic updates; current production is an untimed
semantic/work control.24 alternating paired samples follow3 warmups. Full vectors
and medians are retained; no repeat-until-favorable procedure or composed gate.

Actual production metadata observations verify one query/row and the discarded
owned text length. Test-copy logical owned-text bytes follow the fixed source branch
and known fixture population; they are not allocator-event measurements. Borrowed
validation still traverses UTF-8 and SQLite still selects/returns the payload. No
claim of eliminated engine copies, database page reads, CPU instructions, peak
memory or public latency. Query plans/indexes/count work are unchanged. Whole-query
measurements may be dominated by fixed SQLite or COUNT work; report that result.

Three existing actual-production controls run separately: six seeded registered
history payload cases, exact checkpoint reload and read-only history ownership.
They do not integrate the candidate into public dispatch. No test-only result can
serve as original RED or authorize closure of a future production substitution.

## User-owned execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-sqlite-text-design.ps1
```

Expected five tracked passes: two design tests and three existing public/restore
controls; twelve complete cost rows. Driver requires warning-fatal all-target/
all-feature Clippy, formatting, size, strict encoding and frozen production hashes,
including SQLite, encoder/table and protected matcher/edit sources. Full logs stay
under target/af-perf-002; console prints short results only. No server/binary rebuild,
mutation or expensive composed acceptance is requested. Agent performed bounded
source/script/UTF-8 checks only; runtime UNRUN.

Return after execution: exact source identities, error/metadata controls, full-query
costs and one disposition. If favorable, propose only same-column validated borrowed
text conversion for separate owner approval and genuine original RED before edits.
Do not recommend dropping the column without equivalent failure behavior. No cache,
new SQL ordering, index, authority or persistent representation change. Encoder
corrections stay protected. Census remains13/9763 scoped,9750 unresolved plus96
supplements. This investigation neither reopens dense equality nor implements SQLite.
## Executed comparison and returned recommendation

Agent MaxHeadRoom inspected sqlite-text-design-20261108-170937: five tracked passes,
zero failed/ignored; both design tests19.99s, actual seeded public history9.57s,
checkpoint reload4.33s and read-only history3.21s. All twelve paired cost rows and
24 samples per mode/case complete. Warning-fatal all-target/all-feature Clippy
32.76s, empty format logs, size4 active files and strict encoding2247 text files
GREEN. All1381 recorded source hashes match at inspection, before this documentation
update. All nine protected source identities match. No agent runtime was started.
inspection.json preserves complete cost vectors and the checked summary.

Microsecond medians, rebuilt original owned-text conversion versus validated
borrowed-text test copy, including prepare/query/conversion/result/statement teardown:

| Storage / text bytes / requests / deltas | Owned | Validate without copy |
| --- | ---: | ---: |
| Memory / 0 / 1 / 0 | 57.6 | 57.4 |
| Memory / 96 / 1 / 0 | 94.0 | 88.3 |
| Memory / 6144 / 1 / 0 | 68.0 | 67.2 |
| Memory / 393216 / 1 / 0 | 1620.4 | 1415.8 |
| Memory / 6144 / 16 / 0 | 1078.2 | 1068.1 |
| Memory / 6144 / 1 / 64 | 73.5 | 72.7 |
| File / 0 / 1 / 0 | 67.7 | 67.6 |
| File / 96 / 1 / 0 | 71.0 | 70.8 |
| File / 6144 / 1 / 0 | 80.8 | 80.3 |
| File / 393216 / 1 / 0 | 1551.5 | 1343.1 |
| File / 6144 / 16 / 0 | 1683.8 | 1577.6 |
| File / 6144 / 1 / 64 | 85.4 | 84.8 |

The long-text medians improve descriptively12.6%/13.4%. Retained index6..18 samples
are memory owned1569.0..1686.6us versus borrowed1362.0..1475.4us, and file
owned1471.7..1702.7us versus borrowed1249.3..1406.6us. These separated central
ranges support the bounded local comparison; they do not establish statistical
significance, public/release latency or actual allocator savings. Small cases are
mostly close. File-backed16-request ranges overlap broadly: owned1317.2..1967.7us
versus borrowed1329.8..2107.1us. No universal nonregression or stable percentage
is claimed, and no extra threshold/fallback is justified by these measurements.

Production observations confirm one actual metadata query/row and discarded text
length matching each fixture. The long population copies393216 logical text bytes
per read; sixteen6144-byte reads copy98304 logical bytes. Candidate source has no
owned pretty-text result, while retaining UTF-8 validation, all metadata ownership,
SQL payload selection and COUNT work. These counts do not measure SQLite internal
copies, page reads, allocator traffic, peak memory or CPU instructions. No new
engine-I/O savings are proposed.

Executed contract controls compare original production, owned copy and candidate
for21 combinations of text values and earlier/later conversion faults, missing rows
and distinct-update-time selection. Metadata and downcast rusqlite Debug/Display
errors agree. NULL/type/UTF-8 behavior remains checked through the same Row::get
column boundary. Existing registered history/checkpoint controls pass separately;
they exercise unchanged production, not candidate public integration. Malformed
timestamps, corrupt schemas and full public/restore error mapping remain explicit
future repair obligations rather than certified by this design run.

Unchanged production SqliteStore SHA-256:
80d666fe35078bdb5c89a007d9a670136e15e8217da31554a1ac027913bdb477.
Exact test-copy identities are retained in receipt.json; the source relation guard
passed. Encoder/table and all other protected identities remain unchanged.

**Recommendation: FIX PROPOSED for owner approval.** Retain the SELECT, ordering,
column positions and conversion order. Replace only discarded Option<String> with
a private non-owning validation result implementing FromSql via value.as_str(),
using Option and Row::get at the original column. This retains type/UTF-8 errors
without constructing an unused String. Do not drop the column or skip validation.
No public API, schema, query policy, persistent index or synchronization change.

Before implementation, separately freeze actual production binary/public/durable
metadata and malformed-row failure behavior, add an actual-site zero-owned-text-copy
budget, and execute genuine original RED with semantics GREEN. This passing design
comparison is not RED. Any approved repair must retain necessary query/validation
observations, finite source/error-order protection, isolated real original conversion
restoration budget/static RED with semantics GREEN, exact restored GREEN, integrated
construction-inclusive costs and focused/composed checks on current bytes. Public
history fallback and durable restore physical-error mapping require independent
coverage; timestamp fallback must preserve clock behavior rather than fake equality.

No production optimization or further timing follows this return. Owner approval
is required for this one bounded correction. All previous repairs remain protected;
dense equality deferred. No staging, commit or push. Census unchanged.