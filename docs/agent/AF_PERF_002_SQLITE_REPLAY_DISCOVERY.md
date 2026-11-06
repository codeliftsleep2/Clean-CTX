# AF-PERF-002: SQLite canonical and enclosing durable replay work

Agent: Agent MaxHeadRoom
Status: first replay characterization executed GREEN; MONITOR overall.
No repair. Original authored-status sections remain chronological evidence.

## Real operation and authority boundaries

Persistence replay_history acquires a publication ticket, recovers pending edit,
locks the persistence store and calls load_durable_context. That loader reads latest
metadata, resolves current context ID, delegates to load_context_with_deltas, then
loads the matching semantic snapshot/config/producer identities and checks structural
alignment. Canonical replay separately queries baseline binary, decodes exact file
identity, queries current owner ID, selects ordered deltas, applies versioned replay,
clones reconstructed tuples and converts them back to CoreOps.

The handler then validates historical compatibility against actual source/config,
builds checked hierarchy, selects presentation and commits under publication/currentness
checks before returning. Untrusted durable bytes are not current authority; passing
store round trips cannot certify historical adoption or live publication. No SQLite
transaction/snapshot/locking, authority, compatibility or IO policy is changed.

Potential duplicate owner lookups are source-grounded: the outer and inner loader
each call current_context_id. Cost is unknown and separate statements can observe
different external database writes; no sharing/removal is authorized. Atomic loader
consistency is a distinct correctness design, not an incidental performance repair.

## Actual-boundary observations and fixtures

Test-only counters record baseline and current-owner query invocations, baseline
binary decode input bytes, successful delta row/payload lengths, and logical tuples
copied at the existing output clone. No global SQL tracer, cache or connection policy
is added. Query counts cover only named helpers, not all metadata/snapshot/statement
operations, query planner work, physical IO or lock duration. Bytes/counts are not
allocator measurements. Existing production loops and error propagation remain intact.

The first new test independently grows baseline method count and stored nonempty
sequence-delta count over1/2/4/8/16/64/256 plus16x16. Each delta renames the first
method while preserving stable ID and incrementing version. Target sequence0 and
latest each assert exact returned file/version/instruction stream against the known
baseline/final artifact. Fixture encoding/delta computation/inserts are outside the
observation interval. This is real canonical replay, not a public HTTP/MCP fixture.

The second test grows enclosing load_durable_context baseline size with paired
semantic storage, requiring exact canonical output, hash/fidelity and empty edge
snapshot. It uses the existing legacy test save helper and returns untrusted data;
no compatibility/currentness grant is inferred. Missing-file controls report named
query counts but still perform uninstrumented metadata SQL. SQLite is in-memory:
results are not disk/fsync/cold-cache or enterprise persistence benchmarks.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-sqlite-work.ps1
```

Expected 27 tracked executions: two new characterizations plus 18
existing SQLite and 7 compatibility-persistence controls. Includes malformed
sequence/refusal tests, not merely positive replay. Then focused main-package all-
feature/all-target warning-fatal lint, format, size and strict encoding; source hashes
and actual logs retained. Agent only performed bounded source/text/syntax checks,
not runtime/build/test/formatter/lint execution. No hypothetical repair budget.

## Adjacent presentation lead, not implementation authority

Replay economics renders a presentation candidate. If raw passthrough is not selected
with matching current source, replay invokes a compact closure rendering presentation
again. This may be repeated candidate work, but presentation also reads a dictionary
footer from shared state. Identical source/IR does not automatically prove identical
observation across those calls. Restore/delta/replay have distinct fallback/currentness
semantics. C1 successful-compression authorization does not extend to these callers.
Keep this separate until actual render counts and observation equivalence are known.

## Disposition and remaining work

SQLite replay STRUCTURAL RISK / MONITOR pending execution. No context-ID reuse,
replay representation, presentation shortcut or production correction is authorized.
Independent total instructions, delta payload size/shape, history failure position,
semantic snapshot population, repeated load invocations, physical persistence,
concurrent/external writers and public adoption remain open. Existing tests do not
become exhaustive lifecycle coverage by inclusion in this packet.

Spring stays MONITOR; protected previous repairs remain protected. Syntax accounting
unchanged9763 preserved/13 scoped/9750 pending plus96 callback supplements; no census
record disposition follows from tracing these named methods. Broader discovery and
independent final coverage review remain incomplete.


## Actual canonical/durable replay execution verified

sqlite-work-20261105-172853 passes27 tracked executions: two characterization tests
0.45s,18 existing SQLite tests0.12s, seven compatibility-persistence tests3.19s;
zero failed/ignored. Main-package all-feature/all-target warning-fatal Clippy54.49s,
empty format-check, size10 active files and strict encoding2115 files pass. All
1307 receipt source hashes match at inspection before documentation updates.
inspection.json retains42 parsed rows:28 canonical, seven durable, seven absent.
No agent runtime/build/lint/test was started.

### Baseline growth, one nonempty delta, latest canonical replay

| Methods | Output instructions | Baseline binary bytes | Delta rows | Payload bytes | Output tuple clones | us |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
|1|2|58|1|422|2|929|
|2|3|70|1|422|3|336|
|4|5|94|1|422|5|352|
|8|9|142|1|422|9|391|
|16|17|250|1|422|17|490|
|64|65|928|1|422|65|995|
|256|257|4313|1|422|257|2376|

Each canonical replay has one baseline query and one owner-ID query. Target sequence0
reads/decodes that same baseline and copies the same number of final tuples but
reads zero delta rows/payload bytes. Source/hash/version/instruction identities pass
exact oracles for both targets, not merely instruction counts.

### History growth, one method, latest canonical replay

| Stored/applied deltas | Payload bytes | Output tuple clones | us |
| ---: | ---: | ---: | ---: |
|1|422|2|929|
|2|854|2|322|
|4|1718|2|435|
|8|3446|2|728|
|16|6943|2|1581|
|64|27967|2|7253|
|256|113004|2|16031|

Binary bytes stay58; named baseline/owner queries stay one each. Joint16 methods/
16 deltas reads250 baseline bytes plus6943 delta bytes and clones17 output tuples,
1855us. Timings are single in-memory, instrumented full-replay observations. They
are not physical IO, fsync, planner attribution or enterprise latency measurements.

### Enclosing durable baseline and accounting limit

All seven durable loads execute one baseline helper query and two owner-ID helper
queries, decode58..4313 bytes and copy2..257 output tuples. Exact canonical/hash/
fidelity and empty semantic snapshot assertions pass. These helper counts do not
include metadata/snapshot SQL. Missing-file loads record zero named helper work,
but still execute the uninstrumented metadata lookup. Do not report zero total SQL.

Crucial delegated-cost finding: ir/replay/sequence.rs::apply_sequence clones the
current instruction vector, applies checked edits, validates the complete candidate
and rebuilds indexes before committing. The final-output tuple counter above does
not count those per-delta clones or validation/index work. Its constant value2 at
D256 therefore does not establish bounded total cloning. Separate I*D work is
source-grounded and needs actual counts plus failure-atomicity proof before repair.
A candidate copy can be required to preserve state if a later edit/refusal fails;
no in-place replay or weakened validation is authorized.

## Disposition after measurements

Named query invocation cardinality is bounded in these fixtures; ACCEPT that scoped
observation, not blanket loader safety. Two outer/inner owner reads remain MONITOR:
constant small count alone does not justify changing consistency/snapshot behavior.
Whole replay remains MONITOR pending delegated candidate-copy/validation work and
additional payload/failure/semantic population dimensions. No production correction
is proposed as justified from these counts alone.

Adjacent presentation duplication remains source-only, unmeasured and subject to
shared dictionary/footer observation semantics. Protected C1 does not authorize its
migration. Public historical compatibility/publication and disk/external-writer
boundaries remain separate open obligations. No new runtime command is required
just to record this result; source-wide discovery remains incomplete and syntax
accounting unchanged9763/13/9750 plus96 callback supplements.


Owner accepts the measured canonical/durable lookup behavior as bounded. No lookup
optimization is queued. Next authorized slice is intermediate per-delta cloning,
validation and failure atomicity; see AF_PERF_002_REPLAY_INTERMEDIATE_DISCOVERY.md.
Final-output tuple counts are not evidence about those intermediate operations.
