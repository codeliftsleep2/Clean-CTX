# OPS-R1 amendment 24: shared acquisition performance and failure attribution

Agent MaxHeadRoom, 2026-11-21. Governing mission: OPS-001.
Engineering candidate; no publication, CI, or field acceptance is implied.

## Outcome and integration recommendation

Retain the two bounded corrections. Final root-source focused verification and
scoped linting passed, as recorded below. Both have attributable tracked RED/GREEN regressions and matched
public-query measurements. No global cache, timeout, retry, concurrency, discovery,
completeness, or persistence policy changed.

The accepted test-hardening baseline remains
`c9a8c4eb39fdbde19a34e320cfcc5a8222adbc91940af8aa9ce9b09f24c1ab7f`.
The completed hardening and mutation experiments were not repeated. The subsequent
preservation runs are justified by actual new production corrections.

The repairs reduce demonstrated unnecessary work. They do not establish that the
original real-workspace hang or reported leak is fixed. Qualification/currentness
still has material owner-by-evidence scaling. Integration of these scoped repairs
and closure of the original field incident are separate decisions.

## Demonstrated defects and corrections

1. `InputLog::import` appended exact duplicate witness constraints to retained
   currentness evidence. Fresh consumers repeatedly checked identical constraints.
   The correction retains first-seen order, compacts exact imported constraints,
   and preserves the complete attributed input DAG. `Witness` gains `Hash` using
   its existing exact equality. Different hashes, paths, kinds, targets and entries
   remain distinct. A temporary import-local set stores identities, never validation
   results. `push`, proof qualification, sealing and consumption freshness boundaries
   remain unchanged. Existing duplicates are not globally rewritten.
2. The operation-local grouping basis always included the requesting owner's
   directory, even with explicit empty `types`. Under that option automatic type
   acquisition performs no directory-dependent lookup. The correction omits that
   directory dimension only for an actual empty array. Missing, malformed and
   nonempty options retain directory separation. Selected configuration, effective
   options and origins, roots, aliases, extension, compiler, ownership and demand
   policy remain in the key. Every recipient still independently qualifies.

Relevant source: `src/ir/typescript_resolution/acquisition_inputs.rs`, `host.rs`,
`shared_acquisition.rs`, `automatic_types.rs`, `compiler_libraries.rs`, `program.rs`.
Library lookup uses selected config directory when a config exists; grouping
requires that config. Module/directive traversal uses each acquired unit's path.
Thus explicit empty automatic types removes the demonstrated outer-directory
dependency without granting authority to another supplied source.

## Exact tracked regressions

Both live in `src/tests/ir/ops_r1_shared_acquisition_cost.rs`, registered through
the existing `ops_r1_shared_acquisition_controls` module.

| Test suffix | Historical RED | GREEN obligation |
| --- | --- | --- |
| `ops_r1_shared_import_compacts_exact_constraints_without_losing_receipts` | 3 retained constraints versus 1; source `1270621a113d93c1b367089cbe6e2c2b9b7c255c56120c609835792d25f7431d`, exit 101 | Exact compaction, both receipt nodes retained, later file change detected, different content hash retained in stable order |
| `ops_r1_shared_empty_automatic_types_reuses_project_across_owner_directories` | [2,2,4] versus [1,1,2]; source `61d51bf59302c3688f3113da4f7ba3b6bdadac1f12398488e7519eec9d9ec794`, exit 101 | Same selected config/effective options, different actual directories, two returned units per owner, one acquisition and independent successful qualification |

GREEN source for both corrections:
`c789349a46561b40c99d8cf2ed10a29e73e88292bd8416984ee2a4b5f83cdcda`.
Executable SHA256:
`db3cda35bd7ebe977347e72a087801f7013f6bd0f07c614b3ce200830475cfac`.
All 13 selected preservation controls passed, including automatic-type directory
divergence, inherited project selection, config/root changes, independent operations,
source/package invalidation, overlays, supplied-versus-disk authority, failure
refusal, standalone equivalence and TypeScript persistence.

## Measurements and attribution

Fresh test states have 1/4/16/64 unrelated TypeScript owner cells, no resolver
registrations, explicit valid project configuration, and persistence disabled.
Flat and distributed layouts and global-script/module forms are separate dimensions.
Each public cell executes one cold and one warm forward absent-route query.
Each asserts empty registrations with lower-bound, source-incomplete semantics.
Fixed-job cells separately hold compilation count constant while project size grows.

Production dispatch, nomination, real compiler executor, publication and workspace
consumption are used. This is an in-process public-dispatch tracked test, not a
production stdio/server or Claude field run. Test-only instrumentation records
acquisition, qualification, witness checks, compilation/publication and retained
witness shape. It extends observation lifetime equally across compared candidates;
sampled process memory cannot establish a leak or production steady-state retention.

| Workload | Baseline cold/warm ms | Repaired cold/warm ms | Baseline/repaired work |
| --- | --- | --- | --- |
| Flat 16 scripts | 2,591 / 853 | 1,786 / 382 | witness checks cold 21,451 / 11,491; warm 9,024 / 4,044 |
| Distributed 16 scripts | 3,285 / 1,187 | 2,076 / 540 | acquisitions 16 / 1; project visits 256 / 16; warm checks 12,062 / 6,122 |
| Flat 64 scripts | 25,298 / 12,400 | 7,275 / 3,183 | warm checks 103,776 / 34,476; one acquisition retained |
| Distributed 64 scripts | 40,262 / 18,365 | 11,604 / 5,081 | repaired one acquisition, 64 visits, 64 compilations/publications |

These are individual attributable captures, not statistical speedup estimates or
enterprise efficiency claims. Baseline 16 rows use corrected-observation baseline B;
64 rows use baseline A, whose retained-shape accessor was wrong but query timing and
work counters were valid. The B correction only reads the actual WorkspaceIndex
owner for shape observation outside query timing. Its source identity is
`50fa4a778c08180a1ce391f9440e537d2e9d8df45c6c096d65f9625363e05bb0`.
An empty A shape sample is not evidence of missing authority.

Before repair a sampled flat-16 owner retained 298 constraints with only 58 unique;
after import correction it retained 132, still with 58 unique. Remaining duplicates
come from ordinary recording rather than shared import; no global witness rewrite
was introduced. The distributed-64 process previously reached its 60-second stop
after both query responses had been captured. It was not a passing cell and the
query itself was not observed to time out. The repaired cell completed in 16.955 s
with sampled peak working set 43.57 MiB, under unchanged external controls.

Seven repaired growth cells completed, including both 64-owner layouts and both
16-owner module layouts. All had one exact selected test, exit zero, matching
post-run source/binary identity and no wall/memory stop.

Acquisition counts alone understate remaining work. Distributed-64 qualification
performed 63 successful qualifications, 75,285 validation checks and 83,475 input
visits. Warm queries performed no acquisition, qualification, compilation or
publication but still performed 61,226 witness checks. Freshness checks occur at
preparation and final consumption boundaries; they cannot be shared indiscriminately
across those boundaries. The measured remaining scaling is approximately owners
times complete project evidence. The two corrections materially improve enclosing
query time rather than merely transferring acquisition cost to qualification.

## Architectural question and lifecycle matrix

| Question / intent | Facts, authority and closure | Public path / expected answer | Adversarial and lifecycle evidence / limit |
| --- | --- | --- | --- |
| Does this absent route have registered resolvers? | All nominated owners, current TypeScript source/config/package proofs, unchanged registration producer | `workspace_query`, `resolver_registrations`, forward absent route; empty lower-bound, never closed-world absence | Cold/warm 4/16/64 script and 16 module fixtures pass; real field failure remains open |
| Can equivalent owners reuse acquisition safely? | Same actual configuration and effective options, selected project, roots and demand; full recipient qualification | Real executor preparation feeding the same public query | New exact acquisition RED/GREEN; 13 preservation controls; no reduced owner compilation/publication |
| Does changing source/config authority refuse stale reuse? | Fresh content, directory and route witnesses; supplied source never grants sibling disk authority | Shared producer to publication admission and public consumption | Source/package, overlay, inherited config, automatic types, independent operation and persistence controls pass |
| Does witness compaction retain failure/currentness truth? | Exact constraints retain distinct identities; input receipts stay attributable | Retained authority consumed through existing currentness predicates | Same-path changed hash retained; post-change stale check fails as required; no validation-result cache |

Lifecycle: external query -> owner nomination/preparation -> executor operation ->
per-owner resolver -> immutable proof acquisition/qualification -> compilation
artifact -> publication admission -> WorkspaceIndex semantic authority -> currentness
validation -> public result. Imported constraints reach artifact witnesses through
`capture.rs` and the existing artifact/publication boundary. Operation-local shared
proofs remain ephemeral; persisted witness representation and replay qualifications
are unchanged. Persistence preservation ran against the corrected candidate.

## Historical field failure and remaining risk decisions

The owner's absent-route nonreturn and memory-growth report remains accepted field
evidence. Broad owner preparation, repeated shared-project acquisition, redundant
retained witnesses and large finite cold/warm currentness work are established
mechanisms in controlled captures. They plausibly contribute to severe field latency.
The exact historical blocking stack, allocation owner, field compiler configuration,
and accumulation after completed requests are not established by these captures.
An infinite traversal, leak, lifecycle retention or resource-exhaustion cause is not
proved or excluded. No uncontrolled field repetition was performed.

| Candidate / risk | Decision | Evidence and reassessment trigger |
| --- | --- | --- |
| Duplicate exact shared-import constraints | FIX | Intended RED/GREEN, material warm work and latency improvement, temporary bounded set only |
| Equivalent empty-types projects separated by owner directory | FIX | Intended RED/GREEN, directory-sensitive negatives preserved, material distributed workload improvement |
| Repeated qualification/currentness over complete evidence | MONITOR | Material residual cost measured; broader validation-result reuse would need a separately justified freshness boundary; reopen for representative field attribution or a new bounded exact-constraint correction |
| Single-job operation sealing overhead | MONITOR | Fixed-job measurements show cost but no paired demonstrated end-to-end benefit from removing it; avoid unrelated executor/lifecycle change |
| Original reported leak / indefinite field nonreturn | OPEN | No per-request steady-state retention or exact historical stack evidence; bounded synthetic completion does not close it |

No additional indiscriminate test audit, closed mutation repetition, mandatory CI
change, protected declaration/grant or delivery occurred. Root focused verification
is complete; actual applicable graph reconciliation/admission, CI and separate
integration authorization remain pending. PCG proposition tools were unavailable; governing
source authorities were inspected, without claiming graph acquisition or admission.

## Root successor verification and integration recommendation

The formatter was initially invoked with the wrong Rust edition (2021 rather than
Cargo.toml's 2024). That attempt stopped before source pinning, compilation or test
execution. The script was corrected to 2024; no policy exception was added.
Historical failure: `target/ops-r1/performance-root-verification-20261121-015148-188`.

The successful formatted root successor has source identity
`75d63dfb564a4f860c637160ec1d2bbbc8fcc89823cba06bbbfbae25dc71ab4c` and executable
SHA256 `40090c48894ab4ec05110d5a2aba2a2c651a128c8c57f87cb04fdc5b4d71a1b6`.
Actual root build exit: 0, zero compiler diagnostics. Four exact bounded tracked
tests passed: the two new cost regressions, the original public architectural
acquisition fitness test, and TypeScript persistence. Each receipt has verified
source/binary identity, exit zero and no resource stop. Scoped
`cargo clippy --lib --tests --all-features -- -D warnings` returned exit 0.
This is focused operator verification, not full CI.

Root formatting changes layout/import/module ordering in the new test successor;
original RED/GREEN source bytes stay in their immutable archives.
`formatted-root-source.zip` retains the exact root successor. The raw root build,
test and Clippy receipts are in `root-verification-evidence.zip`, bound through
`root-verification-results.json`. All were hash-verified after capture.

Recommend these two corrections for separate integration review: both have intended
historical RED, corresponding GREEN, preservation coverage and material enclosing
operation improvement. Keep residual qualification cost under MONITOR and the
original field incident OPEN. An uncontrolled Claude repetition or another completed
mutation campaign is not required for this recommendation. A future field gate
should batch unresolved real-workspace latency and retention questions after
integration authorization and applicable CI/admission requirements.

## Mission Directive 26 integration preparation

Agent MaxHeadRoom, 2026-11-21. The investigation phase is closed for the two
demonstrated defects. No M2/M3 campaign or performance investigation was repeated.
The owner has already committed the accepted work locally as
`a59d361e415861327412c8a25de5807003c8639e` on
`(fix)Performance-Regression-Issues`; the mission comparison base is
`0dd8d7503af3116241ad7c1e23c293b1d5849cc1`, not an assumed remote PR base.

Static re-enumeration of the committed source matches the accepted digest
`75d63dfb564a4f860c637160ec1d2bbbc8fcc89823cba06bbbfbae25dc71ab4c` exactly.
The integration review classifies 17 production/test-observation files and 12
tracked test files. Remaining paths are mission/governance records, reusable
verification/evidence assets, or the separate owner-requested size-policy change.
No workflow, dependency, protected selection or grant implementation is modified.
The existing OPS-001 corpus membership candidate remains subject to actual
admission; its presence in a local commit is not proof of authorization or admission.

One generated Python bytecode file was inadvertently included in the local commit.
Integration cleanup removes that file and adds verification-scoped bytecode ignores;
archived historical bytes remain unchanged. This cleanup does not change the
accepted Rust source identity. The earlier report's stale opening prerequisite
was corrected to reflect the successful root tests and Clippy already recorded.

The integration definitions and prepared commit/PR text are under
`verification/ops-r1/proposals/integration-26/`. The scope CSV and source record
from preparation are in `target/ops-r1/integration-26-preparation/`.
The remaining exact operator command is:

```powershell
pwsh -NoProfile -File .\verification\ops-r1\proposals\integration-26\invoke-integration-checks.ps1
```

It checks all mission-modified Rust files with Cargo's Rust 2024 edition. If
normalization changes source, it pins a separate successor, compiles it, runs
seven affected bounded controls and scoped all-feature Clippy. If source remains
unchanged, it reuses the accepted results rather than rerunning them. It also runs
the focused file-size validator regressions, source guards and the actual existing
B11 descriptor/context admission route, without submitting authorization or
publishing. Descriptor refusal and incomplete context remain explicit outcomes.
All runtime invocation is operator-owned under the current AGENTS.md boundary.

### Actual integration results

Capture: `target/ops-r1/integration-26-20261121-075244-830`.
Rustfmt normalized only `src/tests/architecture/ops_r1_observation.rs` and
`src/tests/mcp/ops_r1_absent_route.rs`. All assertion/fixture behavior is unchanged;
the exact formatting diff is retained with the new source snapshot.

Successor source SHA256:
`9c4b40d2da333eb832e03153bcdd9e1bc49994a603a3341088d9d9777bcaacb1`.
Executable SHA256:
`741e5e4e74c37288f150fd42746e06932921f00ac59405666ad12d7dbd587656`.
The fresh all-feature library test build returned exit 0 with zero compiler
diagnostics. All seven exact bounded controls passed, with source/binary identities
verified after execution and no resource stop: both new cost regressions, public
acquisition fitness, persistence, source/package invalidation, config/root isolation,
and directory-sensitive automatic types. Scoped all-feature Clippy returned exit 0.
All 11 size-validator tests passed; size and strict UTF-8 repository guards passed.

`verification/ops-r1/proposals/integration-26/integration-evidence.zip` retains exact
successor source, formatting diff, build, raw test evidence, guard/Clippy logs and
the original actual governance capture. Its index is `evidence-index.json`.
The prior 13-control preservation and seven-cell performance results retain their
original source identities; they were not relabeled as successor executions.

The actual B11 descriptor returned exit 0, zero diagnostics and digest
`9e90d27741c293b00912045b3a1723a7441089d62734b5c7a357fa984a9ea0a3`.
Its accepted/proposed protected state digests are equal; no protected state change
or grant was required for that candidate. The context was ready, complete and current
at generation `a76b83eca5d551ee256134e68ae0e706c2e3d7704395db7227b5f7256b41e5f9`,
with zero diagnostics and 221/221 registered identities represented. It explicitly
reported `content_acquired:false`; context readiness is not source acquisition.
False authorization/publication flags are preserved, not treated as proof of a
missing grant when the protected state is unchanged.

That context exposed missing explicit evidence components for later mission reports.
OPS-001 has no previously protected components or relationships in the actual
descriptor. Its unprotected evidence declaration was reconciled to include those
reports and final verification indexes. No protected declaration, corpus membership
or grant changed during integration preparation.

At this authoring checkpoint the final document-only capture remains to be run:

```powershell
pwsh -NoProfile -File .\verification\ops-r1\proposals\integration-26\invoke-final-documents.ps1
```

It uses the existing checker and bounded stdio client for a fresh descriptor/context
and returned source-acquisition requests for OPS-001 and its five direct controlling
authorities. Exact results remain in the printed capture directory. A changed
protected-state descriptor is a stop condition, not permission to submit a grant.
This document records the checkpoint before that capture; final capture results are
separate evidence. Full CI and authorized delivery remain outstanding. The original
field hang and memory-retention questions remain OPEN. No extra architectural
approval checkpoint or second golden test runner was introduced.

## Evidence association

Reusable definitions and immutable histories:
`verification/ops-r1/proposals/shared-acquisition/performance-25/`.
`cost-historical-red.zip`, `cost-verified-green.zip`, `directory-historical-red.zip`
retain exact source, pins and raw tracked-test evidence. `baseline-b-results.json`
and the preparation records bind source, executable, adapter and evidence hashes.
`performance-evidence-index.json` binds the 13 controls and seven growth cells.
`performance-captures-and-final-source.zip` retains 38 distinct capture directories
and exact final candidate source, with 2,396 members including its hash manifest.

Actual operator captures:
- `target/ops-r1/directory-validation-20261121-013451-823/receipts.json`
- `target/ops-r1/performance-25-performance-25-directory-green-20261121-013611/review.json`
- `target/ops-r1/performance-25-performance-25-baselineb-20261121-001101/review.json`
- `target/ops-r1/performance-25-performance-25-baseline-20261120-235505/review.json`
- `target/ops-r1/performance-root-verification-20261121-015331-278/receipts.json`

Historical executions keep their exact source identities. Any later formatting or
root-source execution must be recorded as a successor, never relabeled as those runs.
