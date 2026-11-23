# OPS-R1: residual currentness cost and next architectural decision

Agent MaxHeadRoom, 2026-11-23. Engineering evidence and proposed decision only.
Governing mission: OPS-001. No new freshness contract or production cache is adopted.

## Completed work within the existing boundary

The continued engineering work retains two small corrections beyond the integrated
source `9c4b40d2da333eb832e03153bcdd9e1bc49994a603a3341088d9d9777bcaacb1`.

1. Sealing and qualification now check each exact immutable witness constraint once
   within one call. Each subsequent call starts with an empty temporary set.
   Attribution traversal and distinct source-role checks remain intact. Witness
   values, identities, ordering, persistence and returned receipt nodes are retained.
2. Host/publication/retained-currentness predicates use the same per-call rule.
   The temporary set contains exact witness values by reference; it retains no
   validation result after the predicate returns. There is no cross-owner reuse.

Validated root source:
`628e47c3696e9520b48870b4ee72c4ebc2b083cadd8ee7c546d079bddc040206`.
Root test executable:
`80b64ace12bd468c445ca2753d9605c8111d40f2f404215e7dc346c4feb83b08`.
Fifteen affected tracked controls passed on that source. They cover the new work
assertions, later-call freshness, conflicting same-path hashes, caller-supplied owner
identity, source/config changes, failure/refusal, independent source authority,
persistence and a positive public resolver case. Builds succeeded with no reported
compiler warnings. Scoped linting and full gates are deferred until engineering
is complete, as directed by the owner; no current Clippy or CI pass is inferred.

The M2/M3 campaigns and the previous growth campaign were not repeated. Each new
measurement was a single bounded baseline/candidate workload, justified by a changed
implementation or contradictory measurement evidence.

## Exact RED/GREEN and fixture correction

The tracked controls live in `src/tests/ir/ops_r1_shared_acquisition_cost.rs`.

| Obligation | Intended RED | Verified GREEN |
| --- | --- | --- |
| Once-per-call sealing of an exact constraint | 2 checks versus 1 | Repeated calls each check once; a later disk change refuses sealing |
| Once-per-call qualification | 57 checks for 14 distinct constraints | Each recipient checks 14; attributed duplicate nodes remain; changed disk authority is refused |
| Once-per-call retained-currentness validation | 104 checks for 9 constraints | Each call checks 9; config mutation and conflicting hashes are refused |

The first retained-currentness fixture incorrectly edited the supplied owner source
while continuing to supply the old owner hash to the low-level index predicate.
That predicate compares the caller's owner hash and checks retained disk witnesses;
the real public query separately validates its owner footprint. The failure did not
justify changing that contract. It is preserved as failed, with fourteen controls
unexecuted in that stopped bundle.

The corrected fixture selects an actual consumed config Content witness, establishes
its physical identity, changes that disk-backed dependency, and requires refusal.
It separately proves that a changed owner hash supplied by the caller is refused.
The corrected source was executed against both the original and repaired consumer.
The exact test file hash is identical in those executions:
`ca04cdad687360fc50391f1efa00a531f4ad7a6e0c69e298e73d417f0a74bec9`.
Corrected RED source:
`f69eceae5f0a3d13c883802f3e0b4bcb98f1db2c46ba7113654731326892c3e8`.
Corrected GREEN source is the root source above. The earlier RED and failed GREEN
retain their original source identities; neither was relabeled.

## Measurements and retention decisions

| Candidate | Evidence | Decision |
| --- | --- | --- |
| Per-call sealing/qualification | Same 64-owner workload: cold checks 198,927 to 147,732; qualification checks 75,285 to 24,885; observed cold query 12.35 to 11.10 s; warm work unchanged | FIX: removes demonstrated duplicate filesystem work with two temporary sets and unchanged authority semantics |
| Per-call retained currentness, legacy-instrumented pair | Warm checks 61,226 to 51,068, but observed warm latency 5.15 to 5.46 s | Initial DESIGN REVIEW: fewer checks alone did not establish benefit |
| Per-call retained currentness, warm observer-isolated pair | Warm checks remain 61,226 versus 51,068; observed warm latency 4.394 versus 4.148 s; sampled process peak 43.72 versus 44.36 MiB | FIX: modest observed benefit supports one small helper and four caller delegations; no memory benefit or timing guarantee is claimed |

The diagnostic correction is confined to the isolated measurement workspace.
The existing `unobserved` mode previously still serialized witnesses before checking
whether observation was enabled. Moving that serialization inside the observer's
enabled update and wrapping the warm dispatch in `unobserved` removes caller-thread
bookkeeping from that timing. Lightweight OPS counters remain. Cold worker-thread
bookkeeping remains, so this is not a release-binary or fully uninstrumented cold
measurement. No unrelated production or global test-infrastructure change was made.

The observer-isolated sources are
`327c30d8521414d993101e2e46f95ea66ad0ea491a0f62f9bf90db0bfa8c49a1`
and `80df547b58a36198648098bb03835ab7ff8d32cd7fe126e7e11c3393f9187b6e`.
They are diagnostic variants, not executions of the root source. Both completed
with verified identities, correct lower-bound empty responses, no safety stop,
one complete-project acquisition, 64 compilations and 64 publications.

These are single attributable captures, not statistical speedup estimates.
The whole-process difference includes untimed setup/teardown and is not attributed
solely to the correction. Sampled working-set differences do not establish a leak,
its absence, or a retained-state memory correction.

## Residual work and architectural scope

The warm query still takes about 4.15 s and performs 51,068 witness checks for
64 owners despite zero acquisition, compilation and publication work. Distinct
owners retain related complete-project evidence and are checked independently.
Qualification still traverses 83,475 input nodes in the 64-owner cold workload.

Currentness is checked during preparation and again during final query consumption.
Those are separate production freshness boundaries. Per-call deduplication preserves
them. A request-wide cache shared across both would not preserve the existing design.

| Risk/mechanism | Disposition and limit |
| --- | --- |
| Complete-project acquisition repeated per unrelated owner | Corrected and preserved; legitimate per-owner compilation/publication remains |
| Duplicate exact checks within one validation call | Corrected above; no result survives that call |
| Repeated related project evidence across owner validations | Confirmed residual work; next material architectural decision below |
| Retained evidence size | `TypeScriptAuthority` owns a `Vec<Witness>`; capture moves the host vector into it, and artifacts/index coverage retain authority per owner. This is a plausible amplification risk, not proof of unbounded retention or the historical leak |
| Transient all-owner compilation batch | Existing accepted P0 ordering/failure behavior; not reopened without regression evidence |
| Other TypeScript witness consumers | Class-role authority, module metadata, arrow selection, return-member and resolve-mapping paths have additional currentness calls. They were identified but not broadly optimized or certified by these captures |
| C#, EF and unrelated lifecycle families | Prior exposure matrix still applies; no extrapolation of these TypeScript results |
| Original healthy-hydration, absent-route field hang/memory growth | OPEN. Owner-reported behavior remains accepted; the exact historical blocking stack and allocation/lifetime mechanism remain unestablished |

Relevant source boundaries: `shared_acquisition.rs::ProjectProof::qualify`,
`host.rs::Witness::all_current`, `coverage.rs::has_current_semantic_projection`,
hydration `publication.rs::prepare_candidate`, and the indexed closure in
`query/resolver_registrations.rs`. `capture.rs` retains the witness vector;
`semantic_compilation/artifact.rs` and index coverage carry the resulting authority.

## Decision required before a larger production change

AGENTS.md's Architectural Approval Gate requires explicit owner authorization for
changes to shared caching, authority/currentness boundaries or externally observable
behavior. The next potential improvement is larger than the completed per-call fix.

1. **Exact decision:** whether multiple owners may use a common validation receipt
   within one explicitly delimited query phase, instead of each owner invoking a
   fresh independent currentness predicate for overlapping constraints.
2. **Why:** repeated cross-owner evidence remains the dominant measured warm work.
   Further reductions of that relationship require either sharing validation across
   owners or changing which owners/evidence are required. Neither follows merely
   from the performance measurements or from a lower-bound response classification.
3. **Affected systems:** preparation, publication qualification if included, final
   consumption, per-owner source hashes and dependency proofs, and refusal/retry
   behavior under edits during validation. Different phases must never share a receipt.
4. **Alternatives:**
   - Retain independent owner checks and monitor the remaining cost. This preserves
     today's boundary and accepts the measured residual latency.
   - Design and validate a phase-scoped batch receipt, keeping source-role binding
     per owner and testing forced source/config changes between owner evaluations.
     The handling of edits during the phase must be explicit before production use.
   - Change source-discovery/completeness or retained evidence ownership. Those
     are separate, larger architectural changes and are not proposed as local fixes.
5. **Recommendation:** review a phase-scoped batch-validation design next. Require
   a concrete edit/refusal contract and deterministic counterexamples before enabling
   it in production. Do not implement a general cache, reuse across phases, change
   per-owner source authority, or suppress validation failures to make it faster.

The bounded preparation work can describe that design, but no new shared receipt
or changed freshness boundary is implemented by this packet. No additional runtime
experiment or full gate is required merely to obtain the owner's architectural
direction. Full CI remains deferred until the engineering change set is complete.

### Concrete, unimplemented phase-batch proposal

The proposed contract would make a named validation phase the freshness unit for
shared filesystem constraints. This differs from independent per-owner calls and
therefore is not silently adopted from the observed performance benefit.

1. Keep the existing nominated owners, expected source hashes, registration scope,
   fidelity, compatibility and source-role qualifications. No discovery filter or
   partial-result shortcut is introduced.
2. Collect the owners' exact typed witness constraints into a temporary plan inside
   the existing phase. Different hashes, paths, kinds, physical targets and directory
   entries remain different constraints. Supplied owner bytes do not become another
   owner's disk authority.
3. Freshly validate the distinct constraints at an explicit phase validation pass.
   Associate each owner with its actual required facts; preserve owner hash validation
   separately. A receipt is only a local result of those checks, not retained authority.
4. Recheck the phase's required facts before exposing its accepted results. A detected
   source/config change must produce the applicable existing refusal/reacquisition
   result for affected owners, never a stale-current result or an inferred deletion.
   Keep existing public query and all-before-publication failure scopes.
5. Drop the plan/receipt when the phase ends. Preparation, publication where applicable,
   and final consumption must have distinct plans; requests never share them. Do not
   put the receipt in the workspace, persisted payload or a global cache.
6. Preserve direct single-owner predicates as independently fresh calls. The proposed
   batch path is explicit and private to applicable production loops; it is not a
   caller-transparent memo around existing methods.
7. Establish deterministic edit interleavings and owner-role negative controls before
   enabling the production batch path. Preserve the current absence of an atomic
   filesystem snapshot guarantee; do not claim continuous freshness between reads.

Required proof cases are equal versus different typed facts; supplied seed versus
disk-backed dependency; an edit between owner evaluations; an edit between phases;
independent sibling validity; incompatible config/root/demand scopes; unchanged
standalone behavior; and exact public output/refusal behavior. The edit case must
exercise a real boundary, rather than rely on a sleep or different path spelling.
The initial implementation, if authorized, would use existing prepared footprints
and returned per-owner validity outcomes, not a new registry or general cache API.

## Evidence custody and governance

### Authority reconciliation following the reissued Mission Directive 24

The earlier question treated a possible contract change as an established blocker.
Further source review found an existing boundary contract in
`docs/agent/ISSUE_162_ROUTE_RESOLVER_ACCEPTANCE.md`: filesystem currentness is observed
at boundaries, without continuous freshness or an atomic multi-file snapshot.
Individual owner loops are implementation structure, not a separate documented
freshness guarantee. This qualifies the earlier escalation: operation-local work
sharing inside the already existing final-consumption closure may be an ordinary
implementation correction if it preserves the boundary and refusal behavior.

The next bounded proof therefore targets consumption only. Preparation and publication
remain unchanged, direct single-owner calls remain fresh, and no result survives
the consumption closure. A public warm-query work witness will establish actual
shared-fact work; a separate prepared-query witness changes a real config after
preparation and requires the existing exact consumption refusal. No production batch
path is implemented before its baseline witness is captured. If equivalence cannot
be established under the controlling contract, the genuine conflict will be returned
to the owner; this reconciliation does not authorize a new freshness policy.

- `verification/ops-r1/proposals/validation-27/results.json` and its archives retain
  the exact per-call producer RED/GREEN and matched enclosing-operation captures.
- `verification/ops-r1/proposals/currentness-28/historical-red.zip`,
  `failed-green-original-fixture.zip`, and `corrected-historical-red.zip` preserve
  the original, failed and corrected witness histories separately.
- `currentness-28/results.json` preserves the initially mixed performance outcome.
  `observer-isolated-results.json` records the subsequent qualified retention decision;
  `observer-isolated-evidence.zip` retains exact diagnostic source and raw captures.
- Root GREEN capture:
  `target/ops-r1/currentness-28-corrected-20261122-095329-501`.
  Successful observer-isolated pair:
  `target/ops-r1/currentness-28-unobserved-20261122-101635-846`.

Hosted PCG tools remain unavailable. The prior actual acquisition of OPS-001 and its
five direct governing authorities is retained at generation
`3dc5ec6cdc8cee180e58a707564612388bd2f39595cdf8ad5009a43ed192840a`.
This report does not claim fresh graph admission. Its evidence association is a
source-owned unprotected OPS-001 candidate; applicable reconciliation/admission and
final lint/CI remain deferred until authorized completion/publication preparation.
No protected grant, commit, push, PR, merge or publication was performed for this work.


## Consumption-boundary correction: verified outcome, 2026-11-23

Agent MaxHeadRoom retained the consumption-only correction as FIX. The unchanged
public work assertion detected the baseline: 176 checks versus the required 111
(preparation 88 plus consumption union 23). The candidate met 111. Eighteen focused
tracked tests passed, including real configuration mutation between preparation and
consumption, conflicting config hashes in either owner order, missing owners,
caller-hash disagreement, deletion refusal and existing authority controls.

Root source identity is
`c6fa2eeff1e27404eef7bbe284493983d448bb64bbc15705e31515e518714c02`.
The deterministic RED and GREEN retain their separate source/executable identities.
`verification/ops-r1/proposals/phase-29/results.json` associates those results with
`historical-red.zip` and `validation-and-measurement-evidence.zip`; generated receipt
paths remain provenance, while these tracked archives retain the actual evidence.

One matched 64-owner distributed-directory public-query pair measured warm latency
3,944.218 ms before and 2,136.693 ms after (45.8% lower observed latency). Warm witness
checks fell from 51,068 to 26,058. Each cold request still acquired one 64-unit project,
compiled and published all 64 owners; warm requests added no compilation, acquisition
or publication. Cold query time was 11,354.925 versus 8,624.343 ms, with cold worker
instrumentation still enabled. Whole-process duration was 16.709 versus 16.356 s;
setup and teardown are included, so query savings must not be attributed to every
part of that duration. Sampled peak working set was 45,453,312 versus 45,654,016 bytes.
There is no demonstrated memory reduction. This is one bounded matched pair, not a
statistical timing guarantee or an enterprise-workspace benchmark.

The implementation shares exact typed filesystem facts only within the existing
final-consumption closure. Owner hash, admission and semantic dependency checks
remain independently applicable. Preparation and publication retain their existing
boundaries; standalone calls allocate fresh check sets. No cross-request state,
persisted proof, global cache, early-empty shortcut or currentness waiver was added.
The small batch helper replaces repeated exact-fact checks rather than introducing
a new authority or lifecycle mechanism.

Residual work remains: preparation still validates each owner's evidence; retained
witness storage remains proportional to owner/project evidence. Those costs are
measured risks, not proof of a leak or authorization for shared durable authority.
The original real-workspace hang and reported memory growth remain unresolved.
These fixtures establish repeated acquisition/currentness work as demonstrated
local amplification mechanisms, not the exclusive historical incident cause.
No completed M2/M3 experiment was repeated. Full CI, final lint and actual document
admission are outstanding and are not implied by these focused results.


## Next attribution witness: repeated requests (not executed)

The matched pair does not establish whether repeated distinct absent selectors
accumulate retained state. The tracked test
`ops_r1_repeated_absent_queries_preserve_retained_owner_evidence` performs thirteen
public queries against one four-owner state, requiring unchanged retained witness
bytes, four total parses, one initial acquisition and no later acquisition. Each
response must retain truthful empty lower-bound semantics. This detects growth in
selected owner-evidence storage, not allocator leaks or unrelated caches.
Production source and earlier test functions are unchanged.

`verification/ops-r1/proposals/retention-30/invoke-repeated-requests.ps1` compiles
this test-only successor and runs one exact test through the existing external
60-second / 512-MiB / five-second monitor. Compilation and execution are unrun.
This is focused failure attribution rather than a full gate or another hardening
campaign. Hosted PCG knowledge retrieval remains unavailable; applicable acquired
source authority is reused. Actual document admission remains separate before
publication. A passing small fixture would not resolve the historical field hang.


## Repeated-request result and consolidated engineering disposition

Agent MaxHeadRoom inspected the actual retention-30 evidence. The all-feature
compilation exited 0. The exact tracked test passed (one passed, none failed or
ignored): thirteen distinct absent-route selectors against one four-owner state,
one initial acquisition visiting four units, no subsequent acquisition, four total
semantic parses, and byte-identical retained witnesses after every warm request.
Serialized witness sizes per owner were 13,078 / 13,078 / 19,579 / 15,598 bytes.
The process completed in 2.206 s; twelve working-set samples observed a peak of
30,449,664 bytes (29.0 MiB), with no watchdog/resource stop. This sampling is too
short and sparse to establish a memory trend or certify absence of a leak.

Source: `679a676fe2d68641ee2b52e868e15a89c8b706052248b507f169c424573a6b3d`.
Executable: `9fab55cc7df1eb53b022965fb43e7088458e221af8afd74dae70069b04164703`.
Production source is unchanged from the phase-29 verified implementation; only the
new tracked test changed. Earlier eighteen-test verification remains attributed to
its original source, not relabeled as execution of this successor. Exact build,
raw output, memory samples, source/pins and assertions are retained in
`verification/ops-r1/proposals/retention-30/repeated-request-evidence.zip` and indexed
by its `results.json`. The earlier preparation-script failure was a missing JSON
archive entry, not a compiler failure or test failure; the corrected preparation
verifies the full prior source digest as well as unchanged Rust producer bytes.

The demonstrated corrections now have bounded evidence: shared project acquisition
removes repeated acquisition; per-call exact-fact deduplication reduces sealing,
qualification and currentness work; final-consumption sharing reduces warm shared
witness work without merging preparation/publication freshness boundaries. All are
retained as FIX under the existing authority model. Remaining qualification input
traversal and independent preparation work are MONITOR: real residual costs, but no
new correction is justified by counts alone. The unchanged evidence payload does
not demonstrate an ongoing append leak in this tested index path.

Historical attribution remains qualified. Repeated project acquisition and witness
validation amplification are established defects in controlled public-path fixtures.
They are plausible contributors to the reported workspace hang, not a proven unique
cause. Project traversal on the actual workspace, unrelated caches, long-lived state,
persistence-enabled lifecycle and allocator retention remain outside this witness.
No field-scale safety or memory-resolution claim follows from the small fixture.

Recommendation: retain the validated corrections for integration review; do not add
another cache or weaken authority/currentness to chase residual counts. Additional
production repair needs concrete contradictory evidence. The original field
incident stays open pending attributable bounded production-boundary evidence.
Final scoped lint, applicable document admission and full CI remain separate,
unclaimed integration gates. No commit, push, publication or CI policy change was
performed, and no completed mutation campaign was repeated.


## Directive 31 integration candidate (checks pending)

Directive 31 closes engineering experiments and authorizes scoped integration
readiness. The review base is delivered HEAD `dbf495517db50cc83fab747c34b33ff989704e27`;
current source remains the retention-30 identity above. Cumulative changes are
limited to exact-fact per-call validation, consumption-only batch validation,
associated tracked tests, evidence definitions and mission evidence associations.
No protected declaration, public schema, persistence representation or CI policy
change is intended. Earlier production corrections remain in delivered HEAD.

`verification/ops-r1/proposals/integration-31/` contains the exact operator handoff,
source/scope reconciliation and proposed commit/PR text. It formats all affected
Rust files, reuses the accepted build when source is identical, and rebuilds and
runs nineteen affected bounded controls only if source changes. Scoped all-feature
library/test Clippy then runs once. The existing actual descriptor/context/source
acquisition route includes repository guards; its returned payloads still require
review and cannot be relabeled protected admission or field acceptance.
Completed mutations and workload timing campaigns are not repeated.

The final package will retain the source manifest, diff, build identity, lint and
regression standing, actual governance results/refusals and external gates.
At authoring these integration checks are unrun. Full CI remains CI-owned;
commit, push, PR, merge and publication remain unauthorized. The engineering
recommendation is conditional integration of demonstrated corrections while the
original field incident stays open, not a claim of final release readiness.
