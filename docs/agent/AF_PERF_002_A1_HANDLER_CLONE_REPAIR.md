# AF-PERF-002 A1: approved handler-only redundant clone correction

Agent: Agent MaxHeadRoom
Status: CLOSED / FIX / PROTECTED for handler-only redundant candidate clone.
Genuine original RED, exact semantic/work GREEN, focused gates, real mutation
RED/restored GREEN and final composed verification executed.
No helper/normalizer/ownership change is authorized or implemented.

## Exact authorized change and proof boundary

After genuine original RED, remove only the handler's durable_candidate IR clone
and its test-only observation, and pass existing &ir at the SAME unconditional
persist_read_baseline_with_authority call. Preserve the handler live-IR clone,
all edge/dependency snapshots and the persistence-normalization clone. No cache,
new helper, API, currentness, lock, failure or publication-order change follows.
The owner's approval supersedes the prior FIX-PROPOSED status for this slice only.
All other A1 ACCEPT/MONITOR and protected/closed dispositions remain intact.

The existing source proof remains: CompiledIR has file_id/instructions/version.
Only file_id differs between the old durable candidate and the borrowed live
candidate at that call. The SAME persistence helper checks authority/checkpoint/
store, then the SAME normalizer clones and replaces file_id with the physical owner
BEFORE encoding; later metadata reads compiled.version. Rendering/economics borrow
immutably. No incoming file_id semantic consumer before normalization was found. The existing
A1 cfg(test) payload counter does record its string length before the clone; that
diagnostic subtotal is expected to differ after borrowing the alias-bearing IR.
It is never used for authority, persistence metadata, encoding or branch decisions.
Diagnostic work rows are not forced equal to semantic-oracle rows. No check,
IO, early return or call is moved or bypassed. If execution disproves that proof,
stop rather than editing the helper or broadening ownership semantics.

## Tracked pre-repair budget and semantic evidence

New src/tests/mcp/a1_clone_budget_af_perf_002.rs is registered through the existing
artifact-operation test module. Original characterization tests and receipts are
retained separately and are not relabeled intended RED.

- af002_a1_normalized_input_identity: explicit distinct pre-alias/alias inputs and
  physical owner, three versions; exact normalized IR and binary equality with
  unmodified input. This is a helper-level proof control, not public integration.
- af002_a1_public_redundant_candidate_budget: actual compress_code_context through
  public dispatch. Require zero redundant candidate clone calls at the measured
  handler site. Preserve the existing normalization clone when a checkpoint is
  required and a real store exists. All case rows print before the aggregate work
  assertion, so genuine RED retains work counts and complete response/durable rows.

Eleven cases cover persistence disabled/enabled, auto-save disabled/enabled,
High/Edit, member populations 1/16/256 and repeated requests 1/4. Every request
requires presentation alias != physical owner. Successful live IR must carry the
alias. Required persisted binary is compared BYTE-FOR-BYTE to encoding the exact
live instructions/version with physical owner identity, then decoded and compared
to the exact expected CompiledIR. Durable loading must agree on IR/source hash and
semantic edges. Auto-save off High requires no manufactured checkpoint. Edit with
a store requires normalization/persistence even with auto-save off. No store still
preserves the unconditional persistence call and its existing early-return policy.

Complete public JSON-RPC responses and durable observations are emitted for frozen
Before/After comparison. Only temporary root strings are normalized. Every actual
cache breaker is checked against actual model-visible bytes BEFORE root adjustment,
then recomputed on normalized content for comparison. Actual persisted binary is
validated against the actual physical owner first; a second binary with the fixed
<PHYSICAL_OWNER> label permits byte-exact cross-run comparison without treating
different temporary paths as semantic changes. No instruction/version/source-hash/
metadata field is dropped or normalized away.

Public timings measure each real request, excluding subsequent oracle reads and
manual expected-value construction. Work rows retain actual selected clone/sink
counts. Logical item counts are not allocator-byte or latency improvement claims.
These tests do not silently certify all languages, races or authority producers.

## Canonical questions and acceptance map

| Question | Required evidence |
| --- | --- |
| Is the correction reachable on the default public path? | Actual public dispatch with successful IR, live publication and durable row inspection. |
| Are physical identity/version/bytes preserved despite a different alias? | Exact in-run binary/decoded IR oracle and complete frozen Before/After rows; incoming identity normalization control. |
| Are failure/currentness/publication boundaries preserved? | Same unconditional helper call and source/authority ordering; existing failed durable-save owner/peer rollback and currentness controls in focused acceptance. |
| Is the measured unnecessary work actually removed? | Unchanged zero-candidate-clone budget original RED -> GREEN; required normalization clone retained; construction-inclusive site totals. |
| Is economics/presentation unchanged? | Full response comparison and existing six economics/C1 controls, without changing their policy or production implementation. |
| Can a real regression evade enforcement? | Finite actual handler clone-site protection plus isolated production clone restoration RED and exact restored GREEN. |

No production edit precedes inspected original RED. Compile/fixture/oracle failures
are NOT valid performance RED. In particular, any durable/public mismatch is a
stop condition, not permission to weaken expected values.

## User-owned first execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-a1-handler-clone-budget.ps1 -Phase Before
```

Expected one exact normalized-input control PASS and one exact public clone-budget
FAIL, zero ignored. The driver validates the intended assertion and all eleven
semantic rows before calling RED genuine. It freezes production, budget and
normalizer source bytes plus the complete source inventory/hash receipt.
The original handler clone and persistence call remain unchanged at this handoff.
Agent ran only bounded script parsing, UTF-8/no-BOM/line-limit and source review;
no Cargo/test/formatter/lint/runtime process was started.

After inspected genuine RED: apply the approved local wiring, run unchanged After
budget and exact response/durable comparison. After rejects changed budget bytes
or a changed persistence normalizer. Then add finite static coverage and isolated
real restored-clone mutation, run affected failure/currentness/economics controls,
lint/format/size/encoding and mandatory composed fitness. FIX / PROTECTED remains
pending all executed acceptance. No further design approval is needed inside the
owner-approved scope; stop conditions above still apply.

No other A1 clone elimination, replay/stats/dictionary rework, transaction redesign,
commit or push is authorized by this packet. Syntax accounting remains unchanged.


## Before attempt rejected at compilation; test API corrected

Operator a1-handler-clone-before-20261106-091226 never executed the identity
control or budget: compilation failed because the new test assumed get_ir returned
CompiledIR. The actual production replay state exposes Vec<Vec<String>> plus
file_version under its alias key. This is a test-authoring error, not semantic
failure or genuine optimization RED. Production/helper bytes remain unchanged.

The test now reconstructs exact expected durable CompiledIR from all alias-owned
live tuples and the stored version, assigning the physical owner for the durable
comparison. Each tuple must decode with expect; none are silently discarded.
No budget, binary equality, version, alias distinction or public response assertion
is weakened. There is no production API/ownership change. Frozen failed logs remain.

Rerun the same Before command. It still must yield one identity pass and one
intended final clone-budget failure with eleven complete semantic rows. Any earlier
compilation/oracle failure remains invalid RED. Only bounded text/UTF-8/line review
performed by the agent; no Cargo/test execution.


## Second Before attempt: identity GREEN; invalid breakpoint expectation

Operator a1-handler-clone-before-20261106-092258 passes the normalized-input
identity control (one pass). The public test completes nine full one-request
semantic/work rows, including off/on, auto-save off, Edit and 256-member durable
checks, then fails on the second request of the first repeated-request case.
This is NOT intended optimization RED: the test incorrectly unwraps missing cache
breakpoint metadata on an already-emitted unchanged-content breakpoint.

Source diagnosis: inject_baseline_breakpoint -> inject_cache_breakpoints returns
without re-emitting metadata when region/breaker was already emitted, preserving
its existing cache-hit behavior. Production is unchanged; no discrepancy in
persistence identity/bytes was observed before the test-only unwrap failure.
This partial observation is not acceptance or a substitute for all eleven rows.

Corrected test explicitly enables the existing cache policy and independently
tracks emitted actual content breakers per case. It now REQUIRES first emission,
validates the actual breaker/region, and REQUIRES metadata omission for repeated
unchanged content. It preserves absent metadata in complete Before/After responses;
it neither creates an empty field nor drops/ignores a present field. Present hint
arrays must have the expected type/cardinality. Only emitted breakers are recomputed
after temp-root normalization. No cache implementation, contract, budget, case,
durable binary/version assertion or production behavior changes.

Rerun the same Before command. One pass/one failure is insufficient by itself:
the driver requires the intended final clone-budget assertion and all eleven
complete semantic rows. No agent test/build/formatter/lint/runtime was started.


## Genuine original RED verified; approved local wiring applied

Agent MaxHeadRoom inspected **a1-handler-clone-before-20261106-093043**: one exact
normalized-input identity pass; the exact public test completes all eleven semantic
and work rows before failing only its aggregate redundant-candidate budget, 44.84s,
zero ignored. All **1314 receipt hashes match** before editing. Original clone work
is 9 instructions for High/M1, 11 for Edit/M1, 69 for High/M16, 1029 for High/M256
and 276 for four High/M16 requests, in each reached handler candidate site. This is
genuine executed tracked RED, separate from both invalid attempts and the earlier
characterization GREEN. inspection.json freezes parsed oracle/work rows and hashes.

Every baseline case already checks exact actual persisted binary/decoded IR,
physical owner/version/source hash and retained semantic edges where a checkpoint
exists; cache emission/dedup and complete public responses are retained for After.
These original semantic controls pass BEFORE the intended final work failure.

Only successful core/compress.rs changed: removed the redundant durable_candidate
clone and its test-only observation, then supplied existing &ir to the SAME
unconditional persist_read_baseline_with_authority invocation. Exact source equality
against the frozen Before file verifies those two local substitutions and no other
change. The normalization-source.rs bytes still match src/mcp/persistence_ir.rs
exactly. Other persistence helper, currentness, C1 economics, lock and publication
code is unchanged. No budget assertion/source change follows verified RED.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-a1-handler-clone-budget.ps1 -Phase After
```

Expected one identity pass plus one public budget pass, and eleven complete semantic
rows identical to the frozen original evidence under the declared temp-root/physical
binary normalization. The driver requires unchanged budget and normalization source
bytes. A new observation difference in incoming normalization file-ID length is
expected and is not semantic output; the actual durable binary must still match
exactly within each run. Any durable/public/version mismatch remains a stop condition.

After is UNRUN; no agent Cargo/test/formatter/lint/runtime started. Bounded UTF-8/no
BOM/line and exact-diff checks only. Static/mutation, focused economics/failure and
mandatory composed gates remain required before FIX / PROTECTED. No other A1 copy
elimination, helper/API/ownership change, commit or push is implemented.


## Repaired budget and exact semantic comparison verified

Agent MaxHeadRoom inspected **a1-handler-clone-after-20261106-093922**: one
normalized-input identity pass and one public budget pass (54.36s), zero failed/
ignored. All eleven complete semantic cases exactly match frozen original
Before rows under the declared path/binary normalization. **All 1314 hashes match**
at inspection. Budget source and existing normalization source are unchanged.
inspection.json retains parsed current rows; original receipts remain frozen.

| Persistence | Auto-save | Fidelity | Members / requests | Measured IR instruction copies before / after | Public us before / after |
| --- | --- | --- | --- | --- | --- |
| False | False | high | 1 / 1 | 18 / 9 | 1799081 / 2084647 |
| True | True | high | 1 / 1 | 27 / 18 | 908761 / 993067 |
| True | False | high | 1 / 1 | 18 / 9 | 862652 / 960650 |
| True | False | edit | 1 / 1 | 33 / 22 | 878425 / 992134 |
| False | False | edit | 1 / 1 | 22 / 11 | 839718 / 864225 |
| False | False | high | 16 / 1 | 138 / 69 | 1122490 / 1176282 |
| True | True | high | 16 / 1 | 207 / 138 | 1209101 / 1359223 |
| False | False | high | 256 / 1 | 2058 / 1029 | 6583391 / 6101568 |
| True | True | high | 256 / 1 | 3087 / 2058 | 7152474 / 7752407 |
| False | False | high | 16 / 4 | 552 / 276 | 4360140 / 5187911 |
| True | True | high | 16 / 4 | 828 / 552 | 5019042 / 5736547 |

The redundant handler site is absent in every After case. The existing persistence
normalization clone still executes exactly once per demanding checkpoint request;
High/auto-save-off and store-disabled variants preserve their prior early returns
WITHOUT skipping the helper call. Public field/version, exact physical-owner binary,
decoded durable IR/source hash/semantic edges and cache emission/dedup all pass.
Incoming normalization file-ID length legitimately changes to the alias length;
actual physical durable bytes remain exact. This is not a durable identity change.

Timings are single instrumented complete public-operation samples, not isolated
clone CPU or a controlled latency distribution. Some After samples are slower;
no overall latency improvement is claimed. Object-copy reductions are established
at the actual selected sites; no allocator/RSS total is inferred.

## Permanent protection and final gate assets authored, UNRUN

New finite AST test classifies whole-IR clone consumers in this handler. It requires
only the existing live-IR clone, AFTER one persistence call borrowing &ir. New direct
IR aliases, unknown whole-IR consumers/storage calls and an added conditional around
the persistence call require review. It also retains the existing normalization
clone/physical-owner replacement. This is a finite handler scope, not a global
clone prohibition. Its self-mutations reject an extra clone and delegated candidate.

All eleven original semantic rows are now tracked in
fixtures/a1_clone_semantic_responses.json. A separate frozen semantic test executes
seven bounded cases: all five persistence/auto-save/High/Edit modes plus both
four-request sequences. Each rechecks exact actual binary/IR before comparing
full durable/public rows to original evidence. The unchanged public budget still
runs all eleven cases, and the verified Before/After driver compared all eleven.
No claim that only counts establish semantic equivalence is made.

The focused driver selects sixteen tracked tests: two budget/identity, two new
static/frozen semantic, two existing real publication-failure controls, all seven
economics tests (six originals plus C1 equivalence), two protected C1 static/public
controls and C1 public count budget. It then runs warning-fatal all-feature/all-target
Clippy, format, size and strict encoding. Original high-cost A1 characterization is
not rerun merely because acceptance assets were added.

The isolated mutation driver clones/overlays exact current source into target,
first establishes clean static baseline, then restores the real redundant clone,
its observation and the original persistence argument in the production handler.
It requires exactly one intended budget failure and one intended static failure,
while the seven-case frozen semantic test stays GREEN. Finally it restores exact
production bytes and requires four tracked passes (budget/identity/static/frozen).
Active source is never mutated. Restoration in finally protects the isolated source
on failure; a compile failure cannot satisfy either intended RED check.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-a1-handler-clone-focused.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-a1-handler-clone-mutation.ps1
```

Run sequentially, proceeding to mutation only after focused completes successfully.
These assets remain UNRUN. Agent performed bounded UTF-8/no-BOM/line-limit and
PowerShell parser review only; no formatter/Cargo/lint/runtime started. The existing
mandatory composed fitness gate now includes the four new A1 protection tests
(expected at least 65 total). Final composed execution follows successful focused/
mutation inspection. No production/helper or budget-source edits follow After;
only test registrations, new acceptance assets and gate/docs were added.

FIX / PROTECTED remains pending executed focused/static/mutation/composed authority.
All other A1 and prior protected/closed dispositions, and syntax accounting, remain
unchanged. No commit/push performed.


## Focused and isolated mutation acceptance executed GREEN

Agent MaxHeadRoom verified **a1-handler-clone-focused-20261106-095253**: sixteen
tracked passes, zero failed/ignored. Budget/identity two tests 43.41s; static/frozen
semantic two 20.82s; failed durable-save control 4.50s; acquisition failure control
0.63s; seven economics controls 1.29s; protected C1 static/public two 7.84s; C1
public count budget 6.76s. All-target/all-feature warning-fatal Clippy passes
27.20s, empty format logs, size passes 43 active files and strict encoding passes
2139 text files. **All 1317 receipt hashes match** at inspection.

**a1-handler-clone-mutation-20261106-095605**: clean static baseline passes 0.06s.
Restoring the actual redundant clone/observation and original persistence argument
in isolated production causes exactly two intended failures: unchanged public
budget 46.73s, static clone-site rule 0.02s. The seven-case frozen durable/public
semantic test still passes 22.46s while the redundant clone is present, proving
the work mutation does not simply break the fixture or semantics. Exact restoration
passes budget/identity two tests 50.71s and static/frozen two 22.19s, zero failed/
ignored. **All 2146 recorded inventory hashes match** at inspection. Active handler,
frozen-before and restored isolated handler bytes are identical. Failed compiler
or fixture execution is not counted as mutation RED.

Budget source still matches the genuine original RED source exactly, and the
normalizer still matches its frozen original bytes. No production/test changes
follow these runs; this return updates documentation only. Both inspection.json
files retain parsed test groups and source-identity checks.

The failed-save control retains the original atomicity assertions for prior live
and durable owner/peer state; acquisition failure stays incomplete. Protected C1
count/response/economics tests pass without reworking C1. This is finite executed
coverage, not a proof of every publication race or whole-system authority behavior.

Next existing mandatory composed gate (user-owned; UNRUN in this return):

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

Expected at least 65 tracked passes, including the four new A1 protections. Final
FIX / PROTECTED remains pending composed GREEN/source identity. No agent test/build/
lint/runtime, commit or push started. All other A1 dispositions, closed/protected
repairs and source syntax accounting remain unchanged.


## Final composed verification: CLOSED / FIX / PROTECTED

Agent MaxHeadRoom verified **af-perf-001-20261106-102529**: **65 tracked passes in
17 groups, zero failures/ignored**, with **all 230 recorded hashes matching** at
inspection. inspection.json retains parsed group results and exact handler SHA-256:

`7c652990579af8c70acfe12a52fb538dcc13034bc17f08c4658160d3e938791c`

Final source comparison against the genuine Before snapshot proves the complete
handler diff is ONLY removal of durable_candidate and its test observation, and
replacement of &durable_candidate with &ir at the existing persistence call.
Original budget source and persistence-normalization source remain byte-identical
to the genuine RED snapshots. The persistence helper/ownership model was not edited.
No production/test changes follow final verification; this return edits docs only.

Acceptance evidence remains separated:

- Original tracked RED: a1-handler-clone-before-20261106-093043; all eleven semantic
  cases complete before final intended work-budget failure. Earlier compile and
  fixture failures remain invalid RED evidence and are explicitly preserved.
- Unchanged budget and complete durable/public comparison GREEN:
  a1-handler-clone-after-20261106-093922; all eleven rows identical.
- Focused sixteen-test/lint/format/size/encoding GREEN:
  a1-handler-clone-focused-20261106-095253, 1317 matching hashes at inspection.
- Isolated real redundant-clone restoration: intended budget/static RED, frozen
  semantics GREEN, exact restored four-test GREEN:
  a1-handler-clone-mutation-20261106-095605, 2146 matching inventory hashes.
- Final mandatory composed GREEN: af-perf-001-20261106-102529, 65 passes/230 hashes.

Final disposition **FIX / PROTECTED**, limited to the successful compression
handler's redundant durable candidate clone. Measured selected-site copies at
M256 fall 2058->1029 instructions with persistence off and 3087->2058 with it on.
Physical-owner normalization, exact durable IR/binary/version/source hash, public
responses, Edit checkpoint, auto-save-off behavior, failure atomicity, protected
C1 economics and durable-before-live ordering retain their tested behavior.
No overall latency, allocator-byte or whole-system correctness claim is made.

The other A1 ownership copies retain their ACCEPT/MONITOR classifications. Deep
proof/operand byte accounting, broader dependency density, publication races and
A4 remain independent gaps. Syntax accounting remains 13 scoped dispositions out
of 9763 original records, with 9750 unresolved plus 96 callback supplements.
No additional runtime or field/binary switch is required merely to close this
bounded internal correction. No commit, stage or push was performed.
