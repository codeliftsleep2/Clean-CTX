# Issue #162: repeated retained-authority replay performance investigation

Agent: Agent MaxHeadRoom
Status: bounded resolver P0 focused GREEN (5 passes); cross-relation batching proposal pending.
Performance remains an acceptance blocker; generation 78 and broader/key suspension remain.

## Operator post-P0 focused run: faster filters; lint correction pending

Inspected route-resolver78-focused-20261103-124840. All 23 pre-lint filters pass:
112 scoped executions, zero failed/ignored; overlapping filters are not a unique-test
or full-repository count. Resolver semantic six-test execution is 366.84s versus
1306.47s previously; resolver78 six-test execution is 287.03s versus 1632.79s.
These are observed test-runner times, not controlled attribution of every cost.
Remaining neutral mapping/field filters take 570.77s/301.29s; performance remains
blocking despite the improvement. Cross-relation batching is not implemented yet.

Clippy stops on useless_conversion in angular_resolver78_controls_issue_162.rs:
entity_type is already &str. Removed only the redundant .into(); assertions and
production policy unchanged. Correction UNRUN. fmt and final six acquisition probes
were not reached. Logs/hashes and pre-fix test bytes are preserved under
 target/issue-162/resolver78-post-p0-lint-evidence/receipt.json.

No full rerun is needed for this lint-only change. The existing -ResumeAtLint
handoff resumes Clippy/fmt then original six probes, skipping passing filters.
This run predates the new AF baseline tests (4942 total tests in its logs); it
cannot certify those unrun tests. No agent Cargo/formatter/runtime ran.

## AF-PERF-001 architectural enforcement design

Phase 1 design/inventory is recorded in [AF_PERF_001_DESIGN.md](AF_PERF_001_DESIGN.md).
It proposes a minimal AST-aware static inspection and deterministic retained-admission
gate, initially resolver plus LoadsChildren, with mandatory policy/CI integration.
Sibling batching, shared currentness and mutex changes are not implemented here.
P0 focused GREEN does not certify broad acceptance; performance remains blocking.

## Focused P0 GREEN; cross-relation performance remains blocking

Inspected resolver-replay-p0-after-20261103-124029: five passed, zero failed,
zero ignored, 3.24s. Every source hash in the operator receipt matches the current
working tree. No repeat run was started after the owner disposition; the existing
completed After execution supplies the requested focused verification.

| Fixed-fixture measure | Before | After |
| --- | --- | --- |
| Resolver projections, four candidate edges | 4 | 1 |
| Mapping replays | 40 | 5 |
| Admission wall time | 872742us | 113975us |
| Fixture construction wall time | 288606us | 186896us |

Fixture is unchanged: 236 source bytes, four files, three route objects, five
mapping occurrences, four canonical resolver edges, 325 witness observations.
The valid measurement function and work-count assertions were preserved. Tests
also observe one projection/five replays for 1,2,4,16 requested edges and zero
for empty/unrelated/legacy-only admission. Independent runtime/demanded reference,
forged/crossed-hash/missing authority and corrected consumed-dependency stale
refusal pass. Baseline's invalid seeded-owner control remains separately classified.

Admission is 7.66x faster in this single fixed-fixture observation (86.94% lower
elapsed); it is not a distributional benchmark, enterprise measurement or proof
of the contribution to the earlier 49-minute delay. Total test-run elapsed is not
used as an unchanged-suite timing comparison because the invalid control changed.
Evidence and exact source/test/log hashes:
 target/issue-162/replay-p0-focused-green-evidence/receipt.json.

This proves the bounded P0 work invariant, not full generation-78 correctness or
performance acceptance. Existing producer duplicate/partial/exact-identity policy
is unchanged by P0, but the expensive generation-78 preservation filters have not
been rerun post-fix and are not claimed GREEN. Clippy/full repository/binary gates
remain pending. No cache, currentness/lifecycle contract or lock was changed.

The [cross-relation proposal](ISSUE_162_CROSS_RELATION_REPLAY_PROPOSAL.md) records
remaining guard/Component/LoadsChildren/provider risks and separates call-local
projection batching from broader witness-currentness/lock design. Return for
bounded disposition before implementing sibling batching. No full campaign is
requested now. Generation stays 78; public-key/broader semantic expansion suspended.

## Prior-work review complete; P0 correction authored, UNRUN

[Prior-work review](ISSUE_162_REPLAY_P0_PRIOR_WORK_REVIEW.md) records all seven
requested issue bodies/states, current code, commit history, existing deterministic
regressions, the missing admission-boundary invariant and separate sibling audit.
#173/#174 remain OPEN with the inspected problematic paths still present. Older
repairs survive at their own boundaries; no preexisting retained-authority context
is bypassed by P0. Query PreparationContext is analogous but belongs to a different
lifecycle and cannot be reused as relation authority.

Observed small baseline resolver-replay-p0-before-20261103-122658: two intended
work-count REDs, plus one invalid owner-currentness fixture control; 2 passed,
3 failed, 5.55s. The fixed measurement has 4 projections/40 mapping replays for
4 edges/5 mappings, admission 872742us, fixture 288606us, 325 witness observations.
The invalid control changed only a seeded immutable owner, bypassing the outer
current-owner-hash caller. It now edits a consumed router declaration dependency.
Pre-correction bytes/logs and hashes: target/issue-162/replay-p0-observed-red/.
Four valid work-count/timing/unrelated test functions remain unchanged, verified
by unchanged-valid-assertions.json. Corrected dependency control is UNRUN.

After isolating/restoring regression bytes/registration, P0 removes the second
mapping-aware demanded_reference_at replay. The original exact mapping proof is
retained, along with independent runtime-reference and selected-id checks. A
source/value uniqueness check preserves the extra crossed-hash competing-record
refusal found in equivalence review. No weaker witness/identity policy is added.

Canonical ResolvedBy admission now initializes one lazy Option<Vec<SemanticEdge>>
for this synchronous call and compares all candidates against that complete exact
projection. Unrelated/legacy-only admission does not initialize it. Nothing is
stored across calls or publication. Sibling projection paths, witness observation
boundaries and snapshot locks are unchanged and require separate design disposition.

Next operator command: run-resolver-replay-p0.ps1 -Phase After. Only the five small
tracked tests run, with fixed-fixture counts/timing. No full campaign until this
proves correction. All post-fix tests/counts/timing/lint remain UNRUN; no agent
runtime ran. Generation 78; full acceptance/binary/key work remain suspended.

## P0 repair approved; deterministic RED/baseline gate authored, UNRUN

Owner approval treats this as an admission-architecture correctness defect. Scope:
remove redundant mapping replay after proof-equivalence review and compute canonical
resolver projection at most once within a stable relation-admission call. No cache,
shared currentness context or lock change. Generation stays 78; key/broader work
is suspended and full acceptance/binary checkpoint remain pending.

Proof-equivalence review: resolve_mapping_at selects a unique source-qualified
mapping record and demanded occurrence, checks exact field/Object/key/pair/value
joins, reads DemandedValue::Reference(Resolved(id)), compares the retained expression
reference to that id, validates target span/hash and source/witness currentness.
The subsequent demanded_reference_at iterates the same value-associated records
and invokes the same replay again before returning that same demanded reference.
Its potential additional conflict is a competing same-value mapping record; the
first replay already refuses competing same-source/hash key/value/pair records.
The review found an extra malformed-record case: demanded_reference_at also checks
value-associated records with a mismatched hash. P0 must preserve that refusal with
a unique source/value record check, rather than simply deleting the call. The new
forged regression includes a crossed-hash borrowed same-value record. Runtime
value_reference_at remains independent and must not be removed.

Tracked src/tests/mcp/angular_resolver_replay_p0_issue_162.rs uses four files,
three route objects, five mappings and four canonical resolver edges. It captures
TypeScript authority directly rather than calling MCP hydration. Five tests cover:
zero projection for empty/unrelated/noncanonical compatibility admissions; at most
one projection for 1/2/4/16 canonical requested edges; one mapping replay per mapping;
a descriptive fixed-fixture admission/fixture timing; independent runtime/demanded/
forged/stale refusal. Timing has no fragile threshold; counters enforce architecture.
Test-only thread-local counters follow the existing extraction_metrics convention
and carry no production state, authority or cache. Baseline test/registration bytes
and hashes are in target/issue-162/replay-p0-authored-before/receipt.json.

Only test instrumentation/registration is added so far. The production repair is
not applied before observing the deterministic RED. docs/agent/architecture.md
requires: "Run the focused regression against the unfixed implementation and observe
RED for the intended behavioral assertion." Existing multi-minute passing tests
provide elapsed observations but do not replace that work-count assertion.

Operator baseline:
verification/live-acceptance/issue-162/run-resolver-replay-p0.ps1 -Phase Before.
It invokes Cargo once for only these five small tests, preserves source hashes,
work counts and fixture/admission microseconds. Expected failures are projection-
per-edge and duplicate mapping replay; compiler/fixture failures do not qualify.
After inspection/isolation, implement P0 and run the unchanged tests with -Phase
After. Do not rerun full resolver filters before the repair. If the previous broad
campaign is still running, cancel it before this gate to avoid Cargo lock contention.
No agent Cargo/test/benchmark/runtime ran; all new assertions are UNRUN.

Further static findings stay separate: guard/other admission paths rebuild their
projections per edge; mapping/arrow/native proof paths repeatedly validate witness
files; snapshot getter invokes currentness/admission while holding its mutex.
P0 does not restructure these. Shared validation observation boundaries, lifecycle
or locks require a separate architectural disposition before implementation.

## Observed execution evidence

Partial campaign route-resolver78-focused-20261103-105339:

| Filter | Result | Test runner elapsed |
| --- | --- | --- |
| issue_162_resolver_semantic_ | 6 pass | 1306.47s |
| issue_162_resolver78_ | 6 pass | 1632.79s |

These are libtest execution times, excluding Cargo compilation. The wrapper first
compiles once and later Cargo invocations reuse unchanged artifacts; rebuilding
is not the explanation for these reported seconds. Combined execution: 2939.26s,
about 49 minutes. Compile/format logs are empty. At inspection the campaign was
still in issue_162_resolve_mapping_; do not mark the full generation-78 gate GREEN.
A process snapshot showed one active clean_ctx test executable and an older
clean-ctx server. This is not evidence of duplicate test workloads, nor proof
that every concurrency/hydration path is efficient.

Copied completed-filter logs and hashes are under
 target/issue-162/authority-replay-performance-static/receipt.json.
This is a partial-running-campaign receipt, not a final run/benchmark receipt.
No agent test/build/profile/server was started or user process terminated.

## Concrete repeated-work chain

1. TypeScriptAuthority::admits_relation_evidence iterates all candidate edges.
2. Each canonical ResolvedBy calls route_resolvers::admits.
3. admits calls project(a), reconstructing the entire resolver projection for
   every edge, rather than replaying/batching the requested support once.
4. Every mapping in project calls resolve_mapping_at.
5. resolve_mapping_at performs witnesses.iter().all(Witness::current).
6. Witness::Content rereads and hashes the file; Directory enumerates filesystem
   entries; Physical canonicalizes paths; other variants issue metadata calls.
7. project then calls demanded_reference_at for that same mapping. That accessor
   calls resolve_mapping_at again, repeating its witness sweep and structural walk.
8. resolve_mapping_at also scans declaration facet trees and demanded occurrence
   vectors repeatedly for exact joins.

For R resolver edges, M mappings, W witness observations, the resolver portion of
one admission call alone performs about 2*R*M full witness sweeps on a valid
fixture, plus native-root and other relation checks. The fixture has R=8 and M=9:
144 sweeps per admission. This counts code-path invocations, not measured syscall
counts or an attributed percentage of wall time. If R grows with M, the repeated
projection adds quadratic scaling before each proof's own scans/IO.

SemanticSnapshot::is_current performs relation admission after authority currentness.
semantic_snapshot calls that predicate while holding semantic_edge_snapshots mutex.
Consequently a snapshot read can do these repeated filesystem checks under a
shared lock. Publication, pending transitions and compatibility validation also
invoke full admission. This can amplify hydration/query cost even without a second
process. Guards and other consumers have similar projection-per-edge patterns;
the new resolver path adds substantial duplicate work to an existing weakness.

This repeated resolver replay was introduced in the current agent-authored repair.
It is an implementation defect; passing semantic tests does not justify its cost.
Full hydration/compilation counts and wall-time attribution are not yet measured.
Do not claim the identified mechanism explains every second or that unrelated
provider/process issues have been ruled out.

## Bounded correction sequence

First remove redundant same-mapping replay in resolver projection: complete
resolve_mapping_at already verifies the exact demanded reference, so calling the
mapping-aware demanded_reference_at again repeats the same proof. Preserve the
independent runtime-reference check.

Then batch the canonical resolver projection once per relation-admission call,
checking every requested canonical edge against that one exact projection. Keep
all name/file/canonical/layer/call-evidence checks, duplicates and ownership policy.
Use call-local owned values; no persistent cache, TTL, global limit or weaker
currentness/completeness contract. Add a deterministic tracked work-count regression
showing projection count does not grow with the number of admitted resolver edges.
Keep existing forged/stale/duplicate/partial regressions as correctness constraints.

Separately review witness validation and lock scope across other relations. Any
request-scoped currentness proof, shared admission context, or lock/publication
change must specify its observation boundary and receive architectural disposition
if it changes shared lifecycle/currentness semantics. Do not quietly add a global
cache or turn presence into authority to obtain speed. Broad hydration redesign
is not authorized merely by this local diagnosis.

Validation should begin with one narrow work-count regression and a small fixed
before/after fixture, not another complete 1600-second group. Existing completed
logs are the baseline observation. Do not rerun whole correctness campaigns until
the repeated-work defect is addressed. Binary checkpoint/key design work waits
behind this performance investigation; generation remains 78 as authored, with
post-edit full acceptance still incomplete.
