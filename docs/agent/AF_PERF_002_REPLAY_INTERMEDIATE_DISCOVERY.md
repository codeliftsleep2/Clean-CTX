# AF-PERF-002: intermediate sequence replay work and atomicity

Agent: Agent MaxHeadRoom
Status: characterization packet CLOSED / NO REPAIR by owner disposition.
Executed measurements and seven refusal controls retained. Transactional isolation
ACCEPT; validation/prefix risks MONITOR. No production optimization queued.
Measured SQLite/canonical owner lookups ACCEPT; do not optimize them.

## Actual ownership and commit boundaries

ContextState::apply_sequence checks protocol, file ownership and version monotonicity
before cloning file.instructions. Edits mutate only that local vector. Expected tuple,
occurrence identity and incoming tuple checks can return Err after earlier candidate
edits have succeeded. Complete tuple validation follows. Only then does the method
replace live instructions/version and rebuild both legacy and occurrence indexes,
then advance global version. No live source hash is changed by this method.

The candidate clone currently supplies rollback isolation for normal Result failures.
That establishes a correctness purpose, not permission for mutation-in-place or proof
that every other traversal is minimal. Index rebuilding occurs after returned-error
checks; allocation panic/OOM semantics are not certified by these Result controls.

SQLite load_context_with_deltas owns a fresh temporary ContextState. Earlier accepted
deltas commit only inside that temporary owner; a later replay error returns Err and
no partially reconstructed CompiledIR is returned to the caller. load_durable_context
still verifies semantic/hash/version alignment. replay_history validates historical
compatibility and stages publication before committing IR/index/cache state.

Public delta_apply has another boundary: it clones the current ContextState to a
candidate under the IR-context lock, calls apply_sequence there, reconstructs target
IR/hierarchy and persists the scoped delta/semantic transition before assigning the
candidate back to ir_ctx. This outer owner-wide clone is not measured by inner
candidate counters and must not be silently merged/eliminated. Independent retained
file count and publication failure behavior remain separate observations.

## Test-only actual work observations

Counters distinguish sequence attempts; intermediate candidate clone count/logical
tuples/String payload lengths; complete validation passes and actual tuple visits;
all validate_tuple calls (includes per-edit checks); index rebuilds/actual tuple visits;
occurrence-key preceding-tuple predicate visits; successful sequence commits.
Final-output SQLite tuple clones retain their independent field and meaning.

Payload bytes sum String lengths at the actual candidate clone boundary, not capacity,
allocator events or all FileState/ContextState memory. That test-only sum adds work to
instrumented timings. validate_tuple calls do not imply identical CPU cost per tuple.
Occurrence counters also observe delta generation if enabled, but fixture generation
occurs before reset; only real replay application is in the measured interval.

Direct fixtures vary instruction-set size (one class plus1/2/4/8/16/64/256 methods)
and delta count independently over the same dimensions plus16x16. Delta generation
is outside observations; each accepted delta changes one method name, preserving ID
and instruction count. First/last method positions expose occurrence-prefix work;
for one method these positions coincide and are repeated controls, not independent
positions. Exact final tuple order/content and file/global versions are asserted.

Existing SQLite characterization now additionally prints these intermediate counters,
including baseline loading's initial index build separately from per-delta behavior.
Target sequence0 exercises zero delta applications. Existing final-output count is
never substituted for intermediate cloning. No hypothetical repair budget is imposed.

## Late-failure semantic controls

A generated two-replacement delta changes the first candidate tuple successfully,
then fails the last edit via expected-content conflict, occurrence conflict or malformed
replacement. Each must leave all live files' instructions, legacy/occurrence indexes,
versions and source hashes exactly unchanged, with zero commit/index rebuild. Peer
ownership/global monotonic version is checked. The same valid delta must succeed after
each refusal, proving the failed candidate did not poison subsequent application.

Unsupported protocol, unknown file, version mismatch and nonmonotonic target exercise
pre-clone refusals. Existing sequence/proptest, replay and historical publication/
compatibility tests remain independent controls. This is not evidence of every public
failure race or catastrophic allocation failure; no broader contract is invented.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-replay-intermediate-work.ps1
```

Expected63 tracked executions: two new intermediate/atomicity tests, two enclosing
SQLite characterizations,10 existing sequence tests,44 replay controls and five
historical replay compatibility controls. Then all-feature/main-package all-target
warning-fatal lint, format, size and strict encoding, preserving logs/hashes. Agent
performed bounded source/text/script checks only, no runtime/test/build/formatter/lint.

## Disposition boundary

No reuse, incremental validation, mutation-in-place, cache, lookup optimization or
publication change is authorized. After measured work, classify rollback-isolation
work ACCEPT where the invariant requires it. Evaluate complete candidate validation
separately: whether a valid inherited baseline plus each validated edit already
establishes the same tuple invariant needs a full constructor/mutator proof; timings
or a successful case cannot establish redundancy. An incremental validation design
requires separate owner approval even if its proof later succeeds.

Any small demonstrably redundant operation must be proposed precisely with unchanged
semantic/failure/publication proof. Until then do not call the entire I*D cost a defect
or claim all complete-set work is necessary. Broader replay/physical IO/owner-count
coverage and source inventory remain incomplete. Syntax accounting unchanged9763/
13/9750 plus96 callback supplements. Protected repairs remain untouched.


## Executed intermediate work and atomicity return

replay-intermediate-work-20261105-174640 passes63 tracked executions: two new
intermediate/atomicity tests0.11s, two enclosing SQLite controls0.24s,10 existing
sequence tests0.21s,44 replay tests0.01s, five historical replay compatibility tests
20.81s. Zero failed/ignored. Main all-target/all-feature warning-fatal Clippy28.33s,
empty format-check, size15 active files and strict encoding2118 files pass. All
1308 receipt source hashes match before documentation changes. inspection.json
preserves28 direct rows and seven refusal observations. No agent runtime was started.

### Independent instruction growth, one delta

N is actual instruction count, including the class. Each successful delta performs
one candidate clone, one complete validation pass, one index rebuild and one commit.

| Methods | N | Intermediate tuples cloned | Full validation tuple visits | Index tuple visits | Prefix visits first / last |
| ---: | ---: | ---: | ---: | ---: | ---: |
|1|2|2|2|2|2 /2|
|2|3|3|3|3|2 /4|
|4|5|5|5|5|2 /8|
|8|9|9|9|9|2 /16|
|16|17|17|17|17|2 /32|
|64|65|65|65|65|2 /128|
|256|257|257|257|257|2 /512|

### Independent delta growth, two instructions

| Deltas D | Intermediate clones | Tuples cloned | Full validation visits | Index tuple visits | Prefix visits |
| ---: | ---: | ---: | ---: | ---: | ---: |
|1|1|2|2|2|2|
|2|2|4|4|4|4|
|4|4|8|8|8|8|
|8|8|16|16|16|16|
|16|16|32|32|32|32|
|64|64|128|128|128|128|
|256|256|512|512|512|512|

All28 rows satisfy candidate tuples = full-validation visits = rebuilt-index visits
= D*N. With one replacement per delta, total tuple-validator calls are D*(N+1):
new replacement validation plus the complete pass. Occurrence-key prefix predicates
are2*D*P at changed position P, distinct from clone/validation work. Joint N17/D16
copies272 tuples, validates272 full-pass tuples plus16 replacements, rebuilds272
index entries and visits32 or512 prefix tuples depending on changed position.

Direct final-output SQLite clone counter remains0. In real SQLite replay at N2/D256,
intermediate copies512 tuples while final output copies2; index visits514 include
the initial2 baseline entries plus512 committed-delta entries. Joint N17/D16 has272
intermediate tuples,17 final-output tuples and289 index visits including baseline.
Sequence0 has no intermediate clones/validation/commits, but initial index construction
and final output copy remain. These distinctions are executed, not inferred.

String payload at direct N257/D1 is3887 logical bytes; N2/D256 sums7563 bytes across
candidate copies. These are lengths, not allocated or peak bytes. Full tuple validation
also materializes CoreOps through tuple_to_op before discarding them; that allocation
volume is not included in the candidate String counter. No allocation-free validator
or incremental replacement is proposed as already approved.

Descriptive samples: N257/D1 first/last positions2041/8269us; N17/D16 first/last
1721/2600us; N2/D256 repeated coincident positions7497/6474us. Counter payload summing,
occurrence identity construction, validation conversions and index rebuilding all
contribute. Do not attribute position latency entirely to cloning or use these single
samples as a controlled crossover or enterprise claim.

### Actual refusal/ownership evidence

Late expected-content, occurrence and malformed-tuple failures each clone the five-
tuple candidate, permit an earlier valid edit, then return Err with zero commit/index
rebuild. Full live instruction streams, both index maps, file/global versions, peer
state and source hashes match pre-call snapshots exactly. The valid delta succeeds
after each refusal. Unsupported protocol, unknown owner, version mismatch and
nonmonotonic target return before candidate cloning or tuple validation. All seven
modes pass, with a peer at global version7 preserving monotonic global ownership.

Historical compatibility tests execute separately and pass; source tracing establishes
that database temporary state and public delta candidate are not published on these
errors. New direct refusal controls do not certify every public publication race or
allocation panic. The outer full-ContextState clone in delta_apply remains outside
these metrics and cannot be justified/eliminated from inner counts alone.

## Per-work-unit disposition and restraint

| Work unit | Disposition | Evidence and limit |
| --- | --- | --- |
| Measured baseline/owner lookups | ACCEPT | Owner accepted bounded behavior; unchanged and no optimization queued. |
| Private candidate isolation for a normal failed delta | ACCEPT with current representation | Executed late failures leave all live ownership/index/version/hash state unchanged. In-place mutation would need a new rollback design; none authorized. |
| Maintaining committed indexes | ACCEPT with current representation | Each accepted candidate rebuilds once; failed candidates never rebuild. This accepts the maintenance obligation, not a claim that whole rebuild is globally minimal. |
| Whole-candidate tuple revalidation | MONITOR / proof gap | D*N work measured alongside per-edit checks; no constructor/mutator proof yet establishes it redundant. Preserving current trust boundary is required; incremental validation needs separate approval. |
| Occurrence-prefix reconstruction | MONITOR |2*D*P measured; duplicate/ordinal correctness requires exact semantics. No lookup reuse or new representation proposed. |
| Outer candidate/publication ownership | OPEN independent obligation | Source traced; retained-owner growth, physical persistence and race costs not measured here. |

No demonstrably redundant small correction is established by this packet. Do not
remove copies or validation merely because D*N is visible. If future proof shows
exact repeated work can be removed without changing trust/rollback semantics, return
that finite proposal; structural reuse, incremental validation and mutation-in-place
remain separately controlled. No new runtime command is needed to record this result.

This closes the current characterization packet, not complete replay architecture
coverage. Source accounting stays9763 preserved/13 scoped/9750 pending plus96 callback
supplements; behavioral rows do not substitute for census dispositions. Protected
repairs remain intact; broader audit discovery is incomplete.


## Owner closure: no repair and no operation-count elimination mandate

Owner closes this characterization packet without implementation. The executed
D*N clone/validation/index measurements,2*D*P occurrence observations and seven
refusal controls remain evidence. They are not a requirement or performance budget
to eliminate every D*N traversal. The current no-repair decision is complete for
this packet; broader replay coverage remains separate.

Future monitored questions are deferred investigations, not implementation tasks:

- Determine whether unchanged candidate regions have an already-established tuple
  validity guarantee at every supported constructor and mutator boundary. Identify
  the exact check repeated, its inputs and failure responsibility before calling it
  redundant. No validation result may be reused across a boundary merely because
  its typed shape appears unchanged.
- Determine whether occurrence-prefix checks reconstruct identical identity/ordinal
  information within one valid operation observation. Positional edits can change
  those prefixes between checks. An existing occurrence index does not automatically
  describe the in-flight candidate after earlier edits.

Return a proposal only for demonstrably redundant work removable through a small
local semantics-preserving correction. Any incremental validation state, copy-on-write,
new transactional model, mutation-in-place or cross-operation reuse requires a
separate architectural justification and owner approval. No such work is queued.

Resume breadth-first production discovery elsewhere. Named outstanding scope includes
observability/startup diagnostics, edit/diff tooling, binary serialization, remaining
compaction paths and untraced delegated consumers. Prioritize consequential production
work over more replay variations or incidental syntax volume. No new runtime gate
is necessary for this documentation-only closure; source evidence stays frozen.

```architecture-graph
node: DOC-187
kind: RESEARCH
status: INVESTIGATING
title: Historical intermediate replay atomicity and no-repair decision
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_REPLAY_INTERMEDIATE_DISCOVERY.md
note: Source-attributed exact historical subject and limits retained; no current executable claim, independent certification, reclosure or banking.
```
