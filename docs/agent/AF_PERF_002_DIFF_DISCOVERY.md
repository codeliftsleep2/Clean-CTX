# AF-PERF-002: default snapshot diff discovery

Agent: Agent MaxHeadRoom
Status: public characterization executed GREEN; shortcut clone CLOSED / FIX / PROTECTED.
The bounded clone repair is recorded in [repair evidence](AF_PERF_002_DIFF_SHORTCUT_REPAIR.md).
Overload matching remains a separate active design investigation; no matcher edit.
Structural-edit exact-end joins remain CLOSED / FIX / PROTECTED. Their completed
verification is not reopened by this independent operation.

## Default reachability and ownership

The registry exposes diff_code_context through core/delta.rs. The handler resolves
the permitted file, reads source through the source cache, owns a source String,
and calls tool_helpers::diff_code_context_handler under the existing cache write
guard. The helper owns rolling structural baselines and source hashes. It returns
formatted text; the public handler wraps it and injects tail-breakpoint metadata.
This is not compilation into the live semantic IR or durable baseline publication.
No lock, read, hash observation, cache mutation or response boundary is moved.

diff_commits separately reaches the same snapshot comparator for modified/renamed
compressible files through gitdiff/engine.rs. Its Git acquisition and fallback
branches are separate: this packet does not execute that public operation or claim
Git subprocess integration coverage.

Clean-CTX acquisition returned degraded frontend_retired timeout during this pass.
Bounded host source searches/reads supplied the trace. Provider failure does not
establish caller absence.

## Two independent copy boundaries

On byte-identical source, the helper checks the cached hash and then calls
get_baseline(...).cloned(). It reads only the resulting classes.len() before
returning the no-change string. CapturedStructure owns class/method/field/import
vectors and strings, including method bodies and declaration evidence. The getter
already returns a shared reference. This is a source-proven unused deep-copy
candidate on a reachable shortcut, not an executed allocation measurement or FIX.

Changed source builds a fresh snapshot and separately clones the old baseline
before comparing and replacing it. That clone has its own lifetime/ownership
boundary; shortcut evidence does not automatically authorize removing it. The
handler's owned source String is another copy outside the named snapshot counts.
No claim of complete request-copy accounting is made.

## Reconcile existing overload inventory

The membership source inventory already lists same-name method comparison as R3,
STRUCTURAL RISK / MONITOR. It lacks executed independent group-width dimensions.
The comparator first searches each same-name current group for an unused exact
method, then searches for an unused matching signature. Remaining pairing/refusal
behavior is distinct. Exact-first matching protects overload insertion/reordering;
a name-only overwrite would discard that contract and duplicate occurrences.

Candidate visits and actual equality evaluations are counted separately: used
prefix entries still incur visits while short-circuiting equality. Method equality
can stop before reaching body bytes. Neither counter measures string comparison
CPU bytes. Comment-only source changes and reordered declarations exercise the
exact pass; body changes exercise signature matching. No matcher rewrite proposed.

## Authored actual-boundary characterization

All observations are cfg(test). Snapshot build calls, offered source bytes and
parser attempts are separate from comparator calls, method record populations,
exact/signature candidate visits and equality evaluations. Parser attempts reflect
the existing language/fallback path, not parser CPU or filesystem reads.

Clone sites are [byte-identical shortcut, changed-source comparison]. Object
subtotals are [classes, methods, fields, imports]. Owned string lengths cover
class names/meta, fields/imports, method signatures/markers/bodies and declaration
return type/attributes/modifiers, including orphan records. These are executed
logical payload counts only after execution: not allocator events, capacities,
RSS, memcpy traffic or complete request cost. Counting itself walks the snapshot
and contributes observation overhead to public timings.

Two tracked tests dispatch the registered public tool. Ten TypeScript fixtures
independently vary class count, method count, body payload and unchanged request
count. Each records cold initialization, changed source and byte-identical repeat:
30 rows. Exact diff text is checked against snapshot/comparator/formatter output;
the shortcut text, actual cached hash and baseline class count are checked too.
The text oracle shares the current algorithm; it is a public-wiring consistency
control, not a frozen independent semantic proof for a future matcher replacement.

Ten C# fixtures vary total methods and same-name group width independently, with
all-body, one-body, reversed-order and comment-only changes: ten more rows.
Actual captured method/group populations are asserted. Written parameter type
names are parsed by the snapshot path; semantic type compilation is not claimed.
Byte changes explicitly invalidate only the source cache after writing, avoiding
timestamp-granularity assumptions while retaining the rolling diff baseline.
Public response capture is serialized; fixture/oracle setup is outside measurement.

## User-owned execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-diff-work.ps1
```

Expected six tracked passes: two new characterization tests, three existing exact
overload/declaration controls and one canonical public-envelope control. Forty
operation rows must be present. Wrapper runs warning-fatal all-feature/all-target
Clippy, formatting, size and strict encoding, then freezes source hashes/logs under
target/af-perf-002. This packet is UNRUN. No agent build/test/formatter/lint/runtime
was started; only bounded source, script syntax and text checks were performed.

Return after execution: measured per-site copy payload and group-width work,
exact public consistency, source identities and smallest defensible disposition.
Characterization assertions describe current work and are not optimization RED.
Any subsequent repair must first freeze semantic evidence and execute a genuine
unchanged production-path work-budget RED. Borrowing only the shortcut class
count is a narrow candidate; changed-source ownership and overload matching remain
separate questions. No authority, cache policy, locking or response change follows
from this source observation.

A4, replay, statistics and protected structural-edit work stay closed. Census
remains 13 scoped of 9763 original records, 9750 unresolved plus 96 callback
supplements. No stage, commit or push performed.

## Executed characterization and disposition

Agent MaxHeadRoom inspected diff-work-20261107-103929: six tracked passes,
zero failed/ignored, all 40 operation rows present. New public tests pass in
4.77s; existing overload insertion, body-change identity, declaration-modifier
and public-envelope controls pass separately. Warning-fatal all-target/all-feature
Clippy passes (2m11s), both formatting logs are empty, size passes seven active
files and strict encoding passes 2165 text files. All 1329 recorded source hashes
match at inspection, before this documentation-only update. No agent runtime ran.

| Shortcut fixture | Cloned classes / methods / fields | Owned string bytes |
| --- | --- | ---: |
| One class, one method | 1 / 1 / 1 | 71 |
| One class, 64 methods | 1 / 64 / 1 | 3338 |
| One class, 256 methods | 1 / 256 / 1 | 13478 |
| Four classes, 16 methods each | 4 / 64 / 4 | 3368 |
| One class, 16 methods, 64-byte body padding | 1 / 16 / 1 | 1866 |
| One class, 16 methods, 4096-byte body padding | 1 / 16 / 1 | 66378 |
| One class, 16 methods, four unchanged requests | 4 / 64 / 4 | 3368 |
| One class, 16 methods, sixteen unchanged requests | 16 / 256 / 16 | 13472 |

Every unchanged interval has zero snapshot builds, parser attempts and comparator
calls, yet one shortcut clone per request. Changed intervals each build once,
compare once and clone the baseline at the separate comparison site once. Cold
requests build once and clone neither site. All exact expected text/cache controls
pass. Payload density and request population amplify copying independently of
class count; these are logical payload observations, not allocation events.

| Methods / group width / mode | Exact visits / equalities | Signature visits / equalities |
| --- | --- | --- |
| 8 / 1 / all-body | 8 / 8 | 8 / 8 |
| 8 / 8 / all-body | 64 / 64 | 36 / 8 |
| 32 / 8 / all-body | 256 / 256 | 144 / 32 |
| 32 / 32 / all-body | 1024 / 1024 | 528 / 32 |
| 128 / 128 / all-body | 16384 / 16384 | 8256 / 128 |
| 32 / 32 / reordered | 528 / 528 | 0 / 0 |
| 32 / 32 / comment-only | 528 / 32 | 0 / 0 |
| 8 / 8 / one-body | 43 / 22 | 1 / 1 |

These fixtures establish group-width amplification and the distinction between
used-prefix visits and equality evaluations. They do not prove a semantics-safe
matcher replacement. Matching is a distinct measured work unit, pending its own
equivalence design; no name-only lookup or shared cache is proposed.

Smallest defensible correction: FIX PROPOSED for the byte-identical shortcut only.
Remove its whole-snapshot clone and borrow the existing cached snapshot to read
classes.len(), under the same guard and hash predicate. The owned formatted output
does not retain the borrow. Keep the changed-source clone, all cache writes,
source acquisition, locking and response formatting unchanged. The shortcut
returns before baseline mutation, and its getter has no required side effects.
No representation, persistent state or ownership redesign is required.

Before implementing, author a separate zero-shortcut-clone public budget, execute
genuine original RED after exact semantic controls, then require unchanged-budget
GREEN and frozen complete public responses/cache identity. Preserve cold/changed
behavior and the independent changed-source clone. Add finite site protection,
isolated real clone-restoration RED/restored GREEN and affected/composed gates.
The passing characterization above is not retroactively optimization RED.
No repair is implemented or classified PROTECTED by this packet.
