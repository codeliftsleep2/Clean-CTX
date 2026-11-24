# OPS-R1 Directive 33 — directory-fragmented Angular acquisition

Agent: **Agent MaxHeadRoom**. Date: 2026-11-23.
Standing: field incident #190 unresolved; the earlier absent-key candidate does not establish a fix for the owner's `types: []` workspace.

## Scope correction from the affected repository

The owner confirmed that the affected tsconfigs contain `types: []`. The original
directory-independent acquisition rule already covers that effective value.
The measured missing-key/empty-discovery optimization below therefore does not
explain this field incident. Its controlled results remain valid for their named
scope, not evidence that the affected repository is fixed.

The owner requested checks of every null/empty/no-content case. A new tracked
matrix in `src/tests/ir/ops_r1_automatic_type_input_cases.rs` covers 26 cases:
absent values, explicit/inherited empty arrays, null resets, inheritance overrides,
visible default/named packages, blank strings and malformed/empty entries.
All 26 passed before further production edits. Explicit and inherited empty arrays
shared one acquisition across distinct owner directories. Null is an effective
option reset, which can restore nonempty default discovery; malformed values do
not grant an empty shared proof.

The follow-up removes key-presence branching from production acquisition sharing.
Every eligible miss goes through the same automatic-type producer and uses its
actual successful empty result. No raw configuration shortcut remains in the
production basis builder. This is behavior-preserving consolidation, not a new
RED/GREEN defect claim or a demonstrated #190 fix. All 33 focused Rust tests pass
after consolidation, including the three tests covering those 26 input cases,
the existing acquisition controls, cold imported-registration forms, required
publication failure, repeated absent queries and shared-consumption checks.
Scoped all-feature library/test Clippy passes with warnings denied. The new matrix
uses reviewed TypeScript 5.9.3 inputs; it does not certify every compiler release.
No new performance measurement, full CI or actual-workspace run is claimed.

Follow-up base HEAD: `5e49dc1077072ba5b4f67d9db4c61c6c308727e1`.
Current source manifest SHA-256:
`46145829e9f85cc03e3eef652fba8f969d123c133cfdbbb73b445b2a095ee14a`.
Current focused test executable SHA-256:
`86387c007757e1fcd06c495e495b3ae720a4bb3153a67e9508b7abbd51e92bdb`.
Identity, build/source manifests and before/after matrix outputs are retained in
`target/ops-r1/cost-33/type-input-*`; exact final test outcomes are in
`target/ops-r1/cost-33/preservation-20261123-133434/results.json`.
The earlier source/binary pins below remain historical and do not identify this
follow-up build. No follow-up commit or publication was performed by this agent.

The packet workflow below is retired as the active next step. The owner requires
direct investigation against the actual affected repository using its own checkout
and build, rather than substituting generated fixtures or transferring a packet.
Frozen packet archives and earlier measurements are retained as historical evidence.

## Contract answer

`resolver_registrations` is an acquire-then-query operation under its existing
[source contract](../../architecture/contracts/angular-resolver.md#approved-question-and-limit).
Lines 150–158 require selector-independent eligible TypeScript nomination through
`source_inventory`, `select_candidates` and High-semantic `compile_candidates_for`.
Current compatible owners may be reused. Required acquisition/publication failure
is non-success. Lines 160–165 separately require `source_complete=false`,
`result_semantics=lower_bound` and `omitted_possible=true`.

Consequently lower-bound output does not authorize skipping acquisition for this
operation. The contract requires its nominated coverage attempt, not universal
route completeness. A successful empty answer is not proof of workspace absence.
Separating index-only lookup from acquisition would require an explicit contract
decision. No index-only mode, selector filter, nominee reduction, chunking,
concurrency limit or publication change is implemented here.

The production handler implements that sequence in
`src/mcp/tool_handlers/query/resolver_registrations.rs`, then consumes admitted
collections with final currentness checks. Cold public-query tests require
registration discovery without preseeded index evidence.

## Measured mechanism and scope

The owner directed a local mimic rather than waiting for the proprietary field
workspace. The controlled fixture has one tsconfig, no resolver mappings, one
TypeScript owner per feature directory, one shared global type reference, and an
absent forward route selector. It exercises real source inventory, compilation,
native publication and public `workspace_query` dispatch. CBM, persistence and
proxy startup are disabled in a fresh `McpState`; this is not production stdio or
Claude verification. This small controlled fixture is not enterprise evidence.

Without an explicit `types: []`, the old acquisition key always included the owner
directory. Even when each directory's automatic-type discovery was actually empty,
64 owners therefore acquired and retained 64 copies of the same 64-unit program.
The accepted explicit-empty-types optimization did not cover this case.

| Controlled run | Owners nominated / compiled | Bases | Acquisitions / shares | Program units visited | Sealed proofs | Query seconds | Sampled peak MiB |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Before, 16 directories, implicit types | 16 / 16 | 16 | 16 / 0 | 256 | 16 | 2.8863 | 35.7813 |
| Before, 64 directories, implicit types | 64 / 64 | 64 | 64 / 0 | 4096 | 64 | 11.6272 | 94.1016 |
| Diagnostic control, 64 directories, explicit empty types | 64 / 64 | 1 | 1 / 63 | 64 | 1 | 8.3735 | 43.2148 |
| Final correction, 64 directories, implicit types | 64 / 64 | 1 | 1 / 63 | 64 | 1 | 7.7395 | 44.3438 |

Before/after use the same fixture definition and source bytes, apart from generated
temporary root paths. The explicit-empty-types row intentionally changes the
configuration and is a diagnostic control, not the before/after correction.
The actual before/after structured answers are exactly equal. Both return zero
registrations with lower-bound, incomplete, omission-possible coverage.

The before run has one acquisition per distinct conservative key, not multiple
acquisitions of one identical key. The defect is fragmentation of semantically
equivalent empty-type programs across keys: 63 repeated whole-program acquisitions
beyond the first. Fourfold owner growth caused sixteenfold program-unit visits.
The final correction reduces sampled peak working set about 53% and query time
about 33% in this one matched observation, without reducing nominated work scope.
These timings are individual observations, not a statistical latency guarantee.

## Correction and equivalence boundary

`automatic_types.rs` separates the existing automatic-name producer from package
resolution. On an operation-local program miss, each eligible owner discovers its
own actual automatic-type names and records its own filesystem/configuration
receipts. Only a successful empty result permits a directory-independent basis.
All other basis dimensions remain intact, including configuration/options,
project ownership, roots, aliases, compiler, extension, mode and demand policy.

`program.rs` retains the exact discovery result until ordinary acquisition needs
it. Discovery is not repeated for a nonempty result. Failed discovery remains a
refusal at the existing acquisition stage after program-root acquisition; it never
grants a shared proof. Nonempty results retain directory-dependent acquisition and
ordinary per-owner package resolution. Standalone/nested paths retain their normal
acquisition behavior.

Every sharing recipient retains its own negative lookup inputs in the bound
program as well as in its raw witnesses. A new local `@types` package therefore
invalidates that recipient's observation. Existing proof qualification still
checks source bytes, supplied/disk roles, roots, witness currentness and aliases.
No directory dimension is removed for unproven or nonempty discovery.

The default production lifecycle remains: handler nomination → prepared jobs →
executor's operation → resolver discovery/acquisition or qualification → returned
semantic artifacts → full batch error/source validation → ordinary publication →
current admitted registration consumption → public response. Shared program units
remain operation-owned; no cache, persisted proof or new session owner is added.
Failure atomicity, publication order and all-owner preparation remain unchanged.

AF-PERF-001 scope: one discovery per eligible owner/program miss, scaling with its
automatic-type lookup chain/explicit roots and entries; one program acquisition
per equivalent successful shared basis, plus necessary per-owner qualification.
The result is reused only inside that exact resolver operation. Existing witnesses
and operation destruction define invalidation/lifetime. The new tracked regression
asserts acquisition/completion/unit work and subsequent invalidation counts.
No claim is made that all enclosing work is linear: witness checks remain large.
AF-PERF-002 disposition: FIX for this demonstrated empty-discovery fragmentation;
no disposition certifies uninspected nonempty-type workloads.

## Memory and progress attribution

The 64-owner before/after runs both prepare 64 jobs, retain a complete 64-result
batch and publish 64 owners. Inventory owner-string payload is 4,844 bytes and
prepared source payload is 2,053 bytes in both. These are payload lower bounds,
not capacities, complete object sizes or exclusive allocation measurements.

Sealed proof source payload totals fall from 140,864 to 2,201 bytes, and program
visits fall from 4,096 to 64. Parsed units, trees, maps, receipts and allocator
overhead are not included in those source-byte totals. The intervention reduces
acquisition and operation-retained proofs together; the RSS reduction cannot be
assigned exclusively to one allocation category without a heap profile.

Peak concurrent compilations remain 7. Peak concurrent acquisitions fall from
7 to 1; no worker limit changes. Witness checks are 102,925 before and 103,681
after: this correction does not explain away remaining validation work.
Stage snapshots are sampled every 200 ms; external working-set samples every
100 ms. Stage counters distinguish attempts/completions and live/peak workers.
Proof/source/job payload counters are cumulative, not a live heap census.

The ordinary capture uses a 60-second external deadline, a sampled 512 MiB stop
ceiling, exact-child kill and a five-second termination wait. Both safety paths
were exercised separately with a 0.5-second deadline and a 16 MiB ceiling; both
stopped and reaped only their spawned child and preserved partial receipts.
Sampling can miss short peaks or overshoot between observations. The watchdog
stops are resource-control evidence, not semantic RED or failed correctness tests.

## Historical deterministic and preservation evidence

The unchanged tracked test is
`src/tests/ir/ops_r1_empty_automatic_types.rs`, registered through the normal
`src/tests/ir/ops_r1_shared_acquisition_controls.rs` module.
It observed RED at the intended acquisition assertion: actual `[2,2,4]`, expected
`[1,1,2]`. The exact test/registration were stashed before the production edit and
restored afterward. GREEN checks sharing, recipient-specific negative lookup
currentness, nonempty local type acquisition, changed source and changed options.
Raw outputs are `target/ops-r1/cost-33/red-regression*.txt` and
`target/ops-r1/cost-33/green-regression*.txt`; final controls also run it again.
An initial Cargo invocation selected zero tests due to `--exact` with a short
filter. It is not RED evidence; the subsequent one-test failure above is RED.

Final all-feature test compilation succeeds. Final acquisition controls: 21 passed,
zero failed, zero ignored. Final public preservation: 27 passed, zero failed,
zero ignored, with unchanged binary identity. This is 48 focused tracked tests;
the new regression is included in the 21 controls, not counted twice.
Detailed public results are in
`target/ops-r1/cost-33/preservation-20261123-115544/results.json`.
`cargo clippy --lib --tests --all-features -- -D warnings` passes (41.46 seconds).
Targeted formatting, active-file size and strict UTF-8 guards pass. The consolidated
full repository gate is CI-owned and was not run locally.
The new public test explicitly exercises imported barrel registrations in both
directions without explicit `types`, then checks unknown absent-route semantics.

An earlier combined public-test process hit its 60-second watchdog after eight
completed passes. A subsequent individual historical work-scaling case also hit
60 seconds. Neither constitutes a passing suite or an assertion failure. The final
focused run uses fresh individual processes with explicit 180-second deadlines.
The historical work-scaling case passes in 84.89 seconds in that final run.
No required test is ignored or weakened.

## Historical source and evidence identity

Starting HEAD: `e399118df6f948afd03c5cd9afe7ee6ded07d68a`.
The working tree was initially clean. No delivery commit, push, merge or publication
has been performed.

Before executable SHA-256:
`7844e4662d673152aceb22373f389f87b9c396e0ff32994d79c58a433cad2bad`.
Preserved executable: `target/ops-r1/cost-33/baseline.exe`.
Before source manifest SHA-256:
`eed15afac8545896d58ef0cb8ee61700914f68d91f435cc208e0766d58789ed8`.
Final executable SHA-256:
`3af4e4523adafd58b7746cbf2363949c640e8c13847fd6fdb350d5bf468fb01c`.
Preserved executable: `target/ops-r1/cost-33/candidate.exe`.
Final source manifest SHA-256:
`d8b0bbb561f539c9dec4fac8fcb0239b494cf43865026ec1f3f79761722ad421`.
Manifests enumerate exact raw-byte hashes of tracked build inputs and source files,
including new source tests. Build JSON/stdout and stderr are retained under
`target/ops-r1/cost-33/`; the final build is `final-reviewed-build.*`.
The RED executable hash was not separately recorded; its actual one-test output
and unchanged regression remain available. No historical hash is invented.
Unchanged regression source SHA-256:
`afcfd58211e343d8f9cfe47fcf5138ff7a14aacf5c7025ac95facd9c2fc71ea2`.

Measured captures under `target/ops-r1/cost-33/`:

- `fixture-20261123-114203`: 16-owner before.
- `fixture-20261123-114214`: 64-owner before.
- `fixture-20261123-114240`: explicit-empty-types diagnostic control.
- `fixture-20261123-115525`: final 64-owner correction.
- `fixture-20261123-115608`: deadline stop-control verification.
- `fixture-20261123-115620`: memory stop-control verification.

Each contains request/response where completed, source manifest, process identity,
working-set series, stage series, limits, outcome and per-file evidence hashes.
`comparison.json` retains the four measurements without relabeling synthetic runs
as field evidence. Reusable capture/verification launchers are under
`verification/ops-r1/field/`; fixtures and regressions are in `src/tests/**`.

## Knowledge preflight and remaining gates

This host did not advertise `pcg_knowledge_v1`. No graph context, generation pin,
source admission or reconciliation is claimed. Applicable repository source
contracts were inspected directly. Clean-CTX graph discovery returned
`provider collection incomplete: unresolved provider truncation/budget`; it was
not treated as absence, completeness or source acquisition. Targeted filesystem
inspection was the explicit fallback. This report is an unadmitted local artifact.

The controlled evidence establishes one concrete amplification defect, not the
unique cause of the proprietary workspace's hang. Nonempty automatic types remain
directory-specific. Inventory/job/result retention, witness cost, persistence,
startup conditions, other projects and allocator behavior can still dominate.
There is no post-query retained-heap census or proof of absence of every leak.

Recommendation: retain the narrow correction for owner
review; do not close #190 or claim field resolution. After local closure, batch one
bounded real-workspace baseline/candidate comparison on the originating machine,
with identical source/configuration and cold-state conditions, exact production
binary identity, all-owner/acquisition telemetry and existing lower-bound semantics.
Keep proprietary raw evidence there. CI remains the full repository gate; neither
full CI nor a real-workspace/Claude pass is claimed by this report.

## Owner review disposition and field handoff

The owner retained this correction for review with #190 open and required an
identical-input, cold-state, executable-pinned baseline/candidate comparison on
the originating machine before any field-fix claim. The acquire-then-query
contract remains controlling; index-only lookup is a separate architectural
decision.

The prepared [bounded pair packet](../../../verification/ops-r1/field/BOUNDED_PAIR_33.md)
uses the preserved executables above and frozen compiler-source/build identities.
It fingerprints all configured source roots, including dependencies, before and
after each request; drift or incomplete fingerprinting prevents acceptance.
Queries retain the 60-second/512 MiB controls. Fingerprint and launcher processes
also have explicit resource/termination controls. Both cases are application-cold
(fresh child/McpState, persistence disabled); OS caches are not cold and are warmed
by preflight reads. This is isolated public-dispatch evidence, not a stdio/Claude
receipt or a production-persistence comparison.

Eleven local adapter checks passed: both executable/build/source pins, stable
fingerprints, dependency/source/configuration/additional-root mutation detection,
bounded fingerprint-child completion, in-root evidence refusal, wrong executable
pin refusal and owned launcher/descendant termination. The owned-tree test needed
execution outside the agent sandbox because that sandbox denies `taskkill` process
enumeration. The packet requires the same preflight on the work machine before
any real query. These are adapter observations, not additional Rust regressions.
Receipt: `target/ops-r1/cost-33/pair-preflight-checks.json`.

No new Rust source or executable changed during the historical packet preparation. The
earlier Rust RED/GREEN, 48-test and scoped lint evidence retain their exact source
identities. No new field query, Claude pilot, full CI, commit or publication ran.
The actual paired field results remain outstanding; the packet grants no automatic
incident closure.

```architecture-graph
node: OPS-033
kind: EXPERIMENT
status: INVESTIGATING
title: Directory-fragmented empty automatic-type acquisition correction
track: OPS-R1
artifact: docs/agent/ops-r1/ENGINEERING_REPORT_33.md
consumes_authority:
  - AUTH.ANGULAR_RESOLVER
knowledge_scope: evidence
knowledge_content:
  - statement:report self#whole
```
