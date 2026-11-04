# AF-PERF-002 C1: equivalence proof and approved regression baseline

Agent: Agent MaxHeadRoom
Status: expressly authorized Before driver executed; genuine RED/equivalence GREEN.
Approved local wiring implemented; focused semantics/static and mutation proof GREEN.
Final composed gate GREEN. C1: FIX / CLOSED / PROTECTED for approved scope;
C2 remains separate characterization.
C2 remains characterization-only. Protected earlier repairs are not reopened.

## Equivalence of the existing and proposed raw-count paths

The handler owns source_arc, borrows source_text from it, creates one tokenizer_box
with create_tokenizer(kind).ok(), then borrows tokenizer_ref. The first raw count
and the economics call use that same immutable source slice, tokenizer object and
copied TokenizerKind. Between them the handler clones IR, obtains an alias and
renders presentation; none replaces source_arc/tokenizer_box/kind or mutates their
contents. Config is shared immutably; no policy read is introduced by reusing count.

count_tokens_with_tokenizer matches Some -> tokenizer.count_tokens(raw), None ->
estimate_tokens(raw). select_with_local_tokenizer uses tokenizer.map_or_else with
the exact same two implementations and then delegates to the already-counted
selector. Both therefore produce the same raw count for production factory outputs.

Production tokenizer factory returns BpeTokenizer over immutable static CoreBPE
and fixed ratio. count_tokens encodes and rounds with that immutable ratio; it
returns usize, not Result. Factory errors already become None at the single handler
creation site before either count. None uses the same estimator and economics
still returns fail-safe byte-exact raw. Llama3 likewise retains its existing raw
fallback decision; Claude retains the calibrated comparison margin. Exact local
and adjusted counts do not gain a new policy or confidence classification.

There is no recoverable count-error channel to alter. A panic during the initial
raw count happens before either old/new selection, so removing a later duplicate
does not catch or convert it. Arbitrary stateful externally supplied Tokenizer
implementations could violate repeated-count equality, but this public handler
only constructs the immutable factory implementations; no injection API is added.
No artificial factory-failure switch is introduced for testing. Explicit None
helper controls exercise the reachable error-to-None behavior after construction.

Candidate rendering and counting remain identical. The proposed local wiring uses
presentation_document then select_with_local_tokenizer_and_raw_tokens. EconomicContent
raw/candidate token fields, selected_tokens/savings, exact ties, approximation margin,
raw byte content and fail-safe selection must match old output. Publication, durable
metadata and response assembly continue to use the same original raw_tokens.

Source correction to earlier investigation wording: failed IR compilation does not
invoke a legacy compressor; compress.rs documents that fallback as retired and
returns ir_unavailable. That error branch, restore/delta/replay, Edit/verbatim and
all other callers remain unchanged. This proposal is only successful IR selection.

## Baseline authored before production editing

New c1_budget_public_raw_count_once uses actual compress_code_context over seven
source-byte dimensions. Requires exactly one handler raw count and zero economics
recounts. It records normalized complete result JSON for later public response
comparison and all counts/timings before its final aggregate budget assertion.
Original characterization is retained unchanged and is not retroactive RED.

New c1_equivalence_counted_and_recounted_paths compares complete EconomicContent
records using all four real factory tokenizer kinds, explicit None and Unicode/CRLF,
small/large/tied candidate forms. ByteTokenizer pins exact tie and Claude calibrated
97/95-of100 boundaries. This extends rather than replaces the six existing tests.
These helper equivalence cases are not reported as public missing-factory execution.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-c1-raw-count-budget.ps1 -Phase Before
```

Expected one intended public recount budget failure, one equivalence pass, zero
ignored. Compile/factory-prerequisite/fixture failures are not intended RED. Driver
preserves exact source/test bytes, hashes and phase logs. No agent tests/builds/
formatter/lint/runtime started. Only bounded script syntax and text checks run.

After verified RED: apply only approved local call wiring, run unchanged budget,
then preserve complete public response identity for normal and fail-safe selection,
six economics controls, finite caller/sink gate, isolated real recount mutation and
restored GREEN, focused/composed gates. No implementation before actual RED.
C2 alias dimensions/caller characterization remains an independent follow-up.


## Authorized Before executed and approved correction applied

Owner explicitly authorized the existing Before command for this run. Agent executed:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-c1-raw-count-budget.ps1 -Phase Before
```

c1-budget-before-20261104-150941 verifies one exact equivalence test GREEN and
one exact public budget test intended RED (6.64s), zero ignored. All seven byte
fixtures observe handler1/economics1. Wrapper exits successfully only after validating
that failure. Earlier operator150357 log also contains the genuine budget RED,
but its broad c1_ selector picked seven tests (six passes/one fail), contradicting
its expected two-test summary. That was a harness selection defect, not missing RED.
The corrected driver selects both fully qualified tests using --exact and records
independent equivalence/budget logs/exits. No assertion was weakened or renamed.

Production edit only in successful IR branch of core/compress.rs: render with
presentation_document, call existing select_with_local_tokenizer_and_raw_tokens
with source_text, rendered candidate, same tokenizer kind/ref and original raw_tokens.
No new helper, signature, state, policy or other caller changed. Snapshot source
and old normalized complete response rows remain in the frozen Before evidence.
Post-edit source drift is the handler and wrapper formatting list only; budget/
equivalence assertions remain identical. No After test/build/lint was started.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-c1-raw-count-budget.ps1 -Phase After
```

Expected one exact equivalence pass plus one exact public budget pass. Only the
Before run was expressly agent-authorized; this After command is operator-owned.
Exact response equivalence, remaining economics controls, static guard, isolated
recount mutation RED/restored GREEN and focused/final receipts remain required.

Response comparison note: cache breakpoint breaker hashes include model-visible
text containing a temporary path footer. Before/after fixture roots differ, so
compare normalized root paths and independently validate each actual breaker against
its actual unnormalized content; do not call different path-derived breakers a semantic
regression or simply drop the field without validating it. Fail-safe raw output,
metadata and all other response fields still require exact oracles.


## First After GREEN; exact public oracle and enforcement authored

Operator c1-budget-after-20261104-151135: equivalence1.77s and public budget6.76s
pass, zero failed/ignored. Every fixture records helper1/economics0; largest
count-boundary bytes11362->5681. No total latency improvement is claimed.

After logs OEM-decoded Unicode: all seven aliases and two structural response texts
contain mojibake. Original logs remain untouched. Explicit cp437->UTF8 inverse
inspection recovers complete responses; each recovered actual text matches its
original SHA-256 cache breaker. All seven normalized full results then equal the
original Before results. inspection.json records the recovery, not a clean-log claim.
New tracked Rust oracle compares Value objects before shell encoding, validates
actual breaker hashes, then normalizes only temp-root paths and derived hashes.
C1 wrappers now explicitly select UTF-8 native-output decoding.

Tracked fixtures/c1_public_responses.json freezes original normalized results.
New public test requires seven complete normal results plus a Llama3 fail-safe raw
case: exact content, content kind, document byte-exact declaration, IR/pretty and
all metadata. Six original economics controls remain plus one equivalence test.
None/initialization-failure mapping is helper coverage without fault injection.

Finite static protection prohibits known recount selectors in the named public
handler and requires its original raw_tokens argument. Other callers are unchanged.
Mandatory composition adds two exact-name selections and two unique afperf_c1 tests;
log filenames sanitize Windows-invalid characters.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-c1-focused.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-c1-mutation.ps1
```

Run sequentially. Focused expects11 passes: seven economics, public characterization,
unchanged budget, full response and static controls; then focused lint/format/size/
encoding. Isolated mutation restores the real original recount call: budget/static
RED, response GREEN; exact restored source then three GREEN. Active checkout is
untouched. All new acceptance assets UNRUN; no agent runtime beyond the expressly
authorized Before command. C2 characterization remains independent, no matcher change.


## Focused semantics/static and isolated production mutation verified

Operator c1-focused-20261104-151811:11 tracked passes, zero failed/ignored:
source growth9.42s, seven economics0.88s, unchanged budget8.51s, public response/
static pair7.11s. Complete original public result fixtures and fail-safe Llama3
raw response match in Rust before log encoding; actual cache breakers validate.
Exact/tie/approximate/None economic fields remain identical. All-target/all-feature
warning-fatal Clippy23.50s, empty format logs, size27 active files and strict
encoding2083 files pass. Source hashes match at inspection. No production/test
source changes follow these runs.

Operator c1-mutation-20261104-152013 restores the original recounting call in an
isolated source copy. Public budget intentionally fails7.04s; static intentionally
fails while exact response comparison still passes7.78s. Restored source yields
budget1 pass7.09s and semantic/static2 passes7.87s, zero failed/ignored. Active
production source and restored isolated source SHA-256 match. The complete frozen
mutation source inventory matches current files at inspection. No active source
was mutated by this proof; no agent runtime beyond the earlier authorized Before.

C1 narrow correction disposition: FIX, protection implemented and focused GREEN;
final composed mandatory gate remains the integration checkpoint. Duplicate raw
count requests disappear without extra state/API, rendering or economics policy
changes. No enterprise/time/allocation claim or change to other handler branches.
C2 is not certified by this result and retains its own authorized characterization.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

Expected at least46 tracked passes across the existing composed groups, including
four C1 executions. Exact selectors and Windows-safe log filenames are used.
This is existing gate composition, not a superseded historical broad campaign.
No binary rebuild or Claude field switch is requested. Earlier protected repairs
are exercised by the gate, not reopened or reimplemented.


## C1 final composed GREEN and bounded closure

Agent MaxHeadRoom inspected af-perf-001-20261104-153416:46 tracked passes,
zero failed/ignored. Groups:14 admission/AST/lifecycle,5 resolver P0,6 delegated,
4 A3,9 af174,4 NgRx collector and4 C1. All recorded source hashes match;
inspection.json retains exact count and outcome. No production/test edits follow
this verification; this update records evidence only.

C1: FIX / CLOSED / PROTECTED for successful public IR-branch raw-count reuse.
Original production-path RED, equivalence/semantic GREEN, exact public result/
fail-safe controls, before/after work counts, focused zero-warning/format/size/
encoding gates, static enforcement, isolated real recount mutation RED/restored
GREEN and composed architectural gate are established. No new helper/cache/state/
API/authority/lifecycle contract. Other compression branches/callers unchanged.
No universal latency or allocator-byte improvement claim follows.

C2 alias matching stays separate characterization-only. Remaining compression/
compaction/rendering and source-wide inventory are incomplete. A2 decision remains
closed; A3 and NgRx collector protected. No automatic broad campaign, binary rebuild
or Claude field switch is needed to record this bounded result.
