# CI-VERIFY-R1 — Verification responsibility separation

Agent: **Agent MaxHeadRoom**. Candidate dated 2026-11-17, America/New_York.
Owner direction is preserved verbatim in the evidence archive. This candidate is
not B11 protected-authority authorization, admitted authority or successful CI.

<!-- pcg:CI-VERIFY-R1-decision:begin -->
## Decision and constraints

The owner approved eliminating duplicate correctness execution and separating
repeated empirical measurement. Every original semantic, frozen-equivalence,
deterministic-work, lifecycle, ownership and persistence assertion remains
mandatory. Fixture diversity and adversarial cases are retained. No production
code, performance threshold, sampling methodology, branch protection, required
job status, security permission or merge policy changes are proposed.

The single controlling authority remains [verification.md](verification.md).
Its candidate amendment requires actual predecessor-bound B11 owner authorization
before admission/publication. This document does not create that authorization.

Use existing Cargo/libtest mechanisms: list without executing, execute the
ordinary workspace/all-target/all-feature suite once with --no-fail-fast, then reconcile actual
target-qualified outcomes. Python validates evidence; it does not schedule tests.
The existing focused fitness runner and historical local-resume behavior remain
available outside the consolidated CI entry point.

The explicit empirical workflow selects enumerated measurement tests using
--ignored and --exact. Original fixture axes, warmups, alternating orders, sample
counts and reporting remain executable. Ordinary companions remain nonignored
tests. Measurement workflow failure is failure, not an allowed-failure step.
Separating workflow placement never retires an established production performance
acceptance prerequisite. Correctness/counting GREEN is not empirical improvement.
<!-- pcg:CI-VERIFY-R1-decision:end -->

<!-- pcg:CI-VERIFY-R1-baseline:begin -->
## Baseline and historical attribution

Run [44134968130](https://github.com/codeliftsleep2/Clean-CTX/actions/runs/44134968130)
belongs to b94273186a67d5f4ef8d3ec01209e71c69ecfc03, before this redesign.
It was not cancelled or reinterpreted. Elapsed job time through build/lint/format/guards was
approximately 101 seconds; dedicated semantic fitness passed in 545 seconds.
The full library suite then failed: 5,230 passed, 19 failed, 9 ignored;
5,028.75 seconds of execution. Audit was not reached. These 19 baseline failures
are preserved separately and are not resolved by execution separation.

The earlier Windows domain capture passed 159 root and 10 proxy tests, zero
failed/ignored, with all features, one test thread and both frozen oracles.
It is not equivalent to the later Linux full-suite run, which used default
libtest parallelism and lacked the dedicated stage's process-local oracle setup.

[The evidence index](../../verification/ci-verify-r1/evidence-index.json) pins
[the durable ZIP](../../verification/ci-verify-r1/baseline-evidence.zip), its exact
entries, acquired authority contents and first three mixed-companion receipts.
Returned graph content is preserved; acquisition is not proof of inspection of
every unrelated component.
<!-- pcg:CI-VERIFY-R1-baseline:end -->

<!-- pcg:CI-VERIFY-R1-membership:begin -->
## Exact membership and assertion preservation

[The reconciliation](../../verification/ci-verify-r1/membership-reconciliation.csv)
retains all 169 original domain identities, package, library target, feature mode,
source and historical configuration. Unchanged tests remain mandatory, without
reclassification solely from names or duration.

[Fitness membership](../../verification/ci-verify-r1/fitness-membership.json)
records exact identities for the 56 original groups, preserving complete
membership and minimums. One new mandatory enforcement test/group is added.
Each member must execute successfully once in its package/library target.
Fitness group attribution remains distinct within the consolidated result.

[Measurement membership](../../verification/ci-verify-r1/measurement-membership.json)
lists 18 new empirical identities with mandatory companions.
[Mixed dispositions](../../verification/ci-verify-r1/mixed-assertion-dispositions.json)
record each changed source and preservation boundary. Original fixture axes,
helper success checks, complete equality, work counters and failure/currentness
checks remain mandatory. Only repeated measurement and distribution reporting
are conditional. A4/edit companions retain a complete paired semantic evaluation
per original case; empirical paths retain all original rounds. Preserved assertion
site counts are static evidence, not behavioral-equivalence verification.

[Nine legacy ignored tests](../../verification/ci-verify-r1/legacy-ignored.json)
retain historical standing and are not newly promoted to mandatory coverage.
Other ordinary ignored tests are refused outside the exact empirical inventory.
Missing/duplicate execution, failed correctness/fitness, empty/incomplete
membership, missing empirical registration, ordinary execution of measurements
and invalid/missing/changed frozen oracles all fail validation.

The consolidated suite uses one libtest thread to preserve deterministic fitness
configuration. The focused runner retains its original local-resume source
validation. Optional historical replay roots and both mandatory frozen-oracle
variables remain configured and recorded. Different environments/features/oracle
settings are not treated as equivalent execution evidence.
<!-- pcg:CI-VERIFY-R1-membership:end -->

<!-- pcg:CI-VERIFY-R1-enforcement:begin -->
## Implementation and enforcement obligations

The existing check job/status calls the fitness runner with -Consolidated.
The duplicate cargo-test step is removed; that full workspace/all-target/all-feature
scope executes once inside the consolidated entry, with explicit oracle setup and
single-threaded libtest. All compilation, lint, formatting, tree-sitter, active-file,
encoding and audit obligations remain. Immutable action pins and permissions remain.

The validator compares compiler registration with actual outcomes per executable
and fully qualified identity. Cargo success cannot excuse missing/duplicate
outcomes. Required fitness membership is exact. Explicit measurements must be
registered and ignored ordinarily; their mandatory companions must pass. Complete
logs, configuration, source/oracle identities, receipts and scope are retained
under the existing evidence location and uploaded on failure.

The tracked src/tests/architecture/ci_verify_r1.rs exercises the actual validator
with synthetic adversarial evidence. Those fixtures verify enforcement, not the
actual execution of the synthetic regression names.

| Canonical question | Required witness | Current standing |
| --- | --- | --- |
| Can a required test disappear? | Exact membership and missing-registration/execution controls | Pending execution |
| Do correctness/fitness failures propagate? | Distinct failed-assertion controls and real CI receipt | Pending |
| Can absent/invalid mandatory oracles pass? | Exact prerequisite identities and refusal controls | Pending |
| Can ignored/empty/duplicate required selection pass? | Validator adversarial fixtures | Pending |
| Do empirical loops run ordinarily? | Ignored measurement outcomes and successful companions | Pending |
| Does explicit workflow reach preserved assets? | Actual registration plus explicit ignored-test selection | Pending |
| Is source/configuration attribution exact? | Fresh listing/execution and source/oracle inventory | Pending new execution |
| Is baseline evidence recoverable? | ZIP entries/hashes and original CI/source identities | Statically retained |
| Has CI improved? | Comparable post-admission Linux CI observations | Unrun |
<!-- pcg:CI-VERIFY-R1-enforcement:end -->

<!-- pcg:CI-VERIFY-R1-limits:begin -->
## Alternatives, governance and limits

Keeping both stages repeats tests with differing oracle environments. Deleting
the fitness runner loses membership, source attribution, receipts, scope and
resume contracts. Ignoring mixed tests omits mandatory assertions. Generic
sharding or new orchestration infrastructure does not address the responsibility
error and is outside this bounded implementation.

Both authority acquisitions were ready/current/complete at generation
9f54327cb8d494dd88ba525b9c72155eb46bc1ac9998b77593857008f375476e.
Applicable named performance, verification, engineering and completion source
constraints were inspected; broad discovery hits were not all treated as applicable.

The first three ordinary mixed companions passed with zero ignored in 66.76
seconds. Checkpoint-20261117-141734 then recorded 15 additional companion passes
and one enforcement-fixture failure in 144.40 seconds. The fixture wrongly used
a name suffix to mark optional measurements, misclassifying mandatory P0. It now
uses the exact approved empirical inventory; no P0 assertions or implementation
changed. The operator reported completion of the focused enforcement rerun;
its raw output was not supplied, so it remains operator-reported evidence.
These observations are not comparable CI performance evidence. Measurement asset
registration, scoped lint/guards, B11 descriptor, owner-only authorization, actual
admission, graph discoverability and final CI remain pending.
The 19 original full-suite failures remain separately unresolved.

The report governs associated CI-VERIFY-R1 manifests, scripts, tracked test
changes, workflows and evidence listed here. No other mission Markdown is created.
Publication waits for the real governance checkpoint and applicable verification.
No owner authorization is generated or submitted by the agent.
Agent MaxHeadRoom, 2026-11-18: CI run 44303364941 at c3a9cd303ed09ef683c286c8a71354f672538725
reported passing outer test summaries, including 5252 root-library passes and 27
approved ignores, but membership reconciliation failed. PowerShell merged stdout
and stderr out of order: the next Cargo header preceded 211 remaining library
registration lines. An inherited child libtest run split its parent outcome and
made its worker appear duplicated. The bounded parser correction associates ordered
Cargo headers with complete ordered outer libtest blocks, validates block record
counts and handles nested output only within an unfinished parent test. Required
registration, unique execution, ignored-test restrictions and fitness membership
remain enforced. No Clean-CTX server or MCP availability is required in CI.
Tracked enforcement fixtures cover reordered headers and nested workers alongside
existing failure cases. Historical evidence is retained in consolidated-parser-red.zip
with consolidated-parser-red-index.json. Corrected-parser replay and tracked
execution were initially UNRUN. Operator capture parser-repair-20261118-091856
subsequently reconciled all 5629 registered tests with 5629 outcomes and no errors.
The focused all-feature enforcement test passed (one passed, none failed or ignored),
with no compiler warnings reported. Exact logs, source bytes and hashes are retained
in consolidated-parser-green.zip and consolidated-parser-green-index.json. This is
replay of historical CI evidence plus focused current-source verification, not a
new full CI run. Scoped lint, guards and current document admission remain pending. This repairs evidence
attribution within the approved verification contract; it does not change test scope.

Agent MaxHeadRoom, 2026-11-18 runtime continuation: the owner reports numerous
individual tests exceeding 60 seconds. The retained CI library summary establishes
5949.39 seconds in total, but its buffered log does not establish individual costs.
A source-confirmed duplication was found: af002_a4_public_lookup_budget invoked
af002_a4_default_footprint_dimensions while both functions were registered tests.
The latter is now an unregistered private helper, still called by the mandatory
public-budget test. Every original matrix case, semantic assertion, work-count
assertion and final public-budget assertion is retained; the same 19-case setup
is no longer independently scheduled twice. The named public-budget fitness
membership is unchanged. Runtime benefit and patched execution are not yet verified.

The fixed five-test operator batch in verification/ci-verify-r1/invoke-runtime-triage.ps1
records separate compilation and per-test wall time, actual Cargo-selected executable
identity, exact test registration, source hashes and full output. Its 60-second
notices are diagnostic progress, not timeouts, skips, budgets or acceptance criteria.
It selects the A4 public budget plus C# skeleton degradation and delta combined frozen/unused-budget,
integrated and persistent correctness witnesses once each, with all features and one libtest thread. It does
not execute ignored measurement studies, change production behavior, reopen rejected
optimizations or claim these five tests exhaust all slow paths. Run 406 (44134968130, revision b94273186a67d5f4ef8d3ec01209e71c69ecfc03)
was subsequently pulled directly. It contains 37 named over-60-second warnings.
Warning-to-completion elapsed lower bounds include 1441 seconds for C# skeleton
degradation, 894.39 for delta frozen semantics and 789.73 for delta integrated cost.
These include shared-lock waits in its parallel run and are not CPU measurements.
The selection was corrected from these actual logs; it does not rely on
owner recollection or treat old timings as current-source results. Additional
repair selection awaits isolated cost evidence.
Source inspection also established that af002_delta_previous_frozen_semantics and
 af002_delta_previous_unused_budget each regenerated the identical 380-row corpus
under the same serial guard and Observation mode. The surviving frozen-semantics
test now invokes the unchanged unused-budget assertions before that observation
ends. Its original compare(records()) and all original fixture cases remain;
the second registration is removed. The fitness declaration now represents both
obligations in that surviving test, changing 57 groups to 56; the standalone
runner's duplicate selector is removed too. This is one execution of both required
contracts, not ignored coverage or a waiver. No production code changes. Exact
before/after membership is recorded in runtime-deduplication.json. Patched test
execution and elapsed benefit remain unverified.

The batch also verifies that neither duplicate corpus remains independently
registered, then executes the existing membership enforcement test once against
the revised declarations. Capture runtime-triage-20261118-100347 verifies all five selected mandatory tests
and revised membership enforcement. Compilation took 53.52 seconds; isolated
C# skeleton, combined delta frozen/budget, delta integrated, delta persistent and
A4 public budget took 5.77, 232.97, 49.65, 12.78 and 89.40 seconds respectively.
The C# test's run-406 warning is therefore not evidence of 24 minutes of isolated
work. These current-source Windows observations cannot be relabeled as Linux
CI costs or an old/new runtime comparison. Exact receipts and source identities
are retained in runtime-baseline-evidence.zip and runtime-baseline-index.json.

The next bounded trial uses the same two demonstrated expensive witnesses and
root test profile, with only an invocation-local Cargo override optimizing test
dependencies at level 3. It changes no repository profile defaults, production
source, fixture axes, assertions, debug-assertion settings or overflow policy.
Compilation cost and test elapsed time are recorded separately. The dependency trial subsequently passed in runtime-triage-20261118-102252.
Its clean rebuild took 322.98 seconds, and identical delta/A4 witnesses took
75.36 and 24.18 seconds (versus 232.97 and 89.40). Membership enforcement also
passed. Actual Cargo artifact metadata establishes 189 opt3 dependency artifacts
with debug assertions and overflow checks true, plus an opt0 main test crate
with both checks true. This is a material focused runtime reduction, with a
separately measured compilation penalty; it is not a full-suite or CI claim.
Exact evidence is retained in runtime-dependency-trial-evidence.zip and its index.

An invocation-local optimized-root trial additionally requests test opt-level=1,
retaining opt3 dependencies and the same two witnesses plus membership enforcement.
The main crate's Cargo profile metadata will establish whether debug assertions
and overflow checks remain enabled. The root trial passed in runtime-triage-20261118-104106: delta 56.50 seconds, A4 21.07 seconds,
and membership enforcement 5.39 seconds. Root rebuild cost was 339.34 seconds
with already-built optimized dependencies; this is not a cold total. Actual main
crate metadata is opt1 with debug assertions and overflow checks true. Source
manifests establish identical Rust source, Cargo.toml and Cargo.lock across all
three timing captures. These are bounded single-run observations, not a repeated
performance distribution or Linux-wide certification. Evidence is retained in
runtime-root-trial-evidence.zip and runtime-root-trial-index.json.

The owner-directed CI runtime correction adopts these invocation-local test
settings only inside the consolidated correctness runner: root opt1, dependencies
opt3, debug assertions true and overflow checks true. Listing and execution use
the exact same arguments, which are recorded in configuration.json. Local Cargo
profile defaults, production source and explicit empirical-study workflows remain
unchanged. The two observed test runtimes improve about 4.1 and 4.2 times relative
to the initial current-source capture, in addition to eliminating duplicate corpus
execution. Rebuild cost rises materially; the decision trades that measured cost
against repeated expensive execution across the full CI suite. A complete Linux
CI execution must establish the actual overall benefit and reveal any other slow
families. No one-to-two-minute CI promise or full slowdown resolution is claimed.
Scoped lint/guards, final source admission and fresh CI remain pending.

No live Clean-CTX service is a CI prerequisite. Full suite verification remains
CI-owned; the batch is proportionate operator investigation, not a substitute gate.

<!-- pcg:CI-VERIFY-R1-limits:end -->

```architecture-graph
node: VERIFY-001
kind: IMPLEMENTATION
status: INVESTIGATING
title: CI verification responsibility separation candidate
track: REPOSITORY
artifact: docs/agent/CI_VERIFY_R1.md
depends_on:
  - AUTH.FINAL_VERIFICATION
  - PERF-005-BOUNDED
  - PERF-004-DOCTRINE
preserves:
  - ARCH-004
uses_evidence:
  - VERIFY-002
note: Approved bounded direction; protected authority admission and complete verification remain pending.
knowledge_scope: decision
knowledge_content:
  - statement self#CI-VERIFY-R1-decision
  - observation:baseline self#CI-VERIFY-R1-baseline
  - scope:membership self#CI-VERIFY-R1-membership
  - verification_obligation self#CI-VERIFY-R1-enforcement
  - qualification self#CI-VERIFY-R1-limits
  - evidence:fitness verification/ci-verify-r1/fitness-membership.json#whole
  - evidence:measurement verification/ci-verify-r1/measurement-membership.json#whole
  - evidence:reconciliation verification/ci-verify-r1/membership-reconciliation.csv#whole
  - evidence:preservation verification/ci-verify-r1/mixed-assertion-dispositions.json#whole
  - evidence:configuration verification/ci-verify-r1/membership.json#whole
  - evidence:legacy verification/ci-verify-r1/legacy-ignored.json#whole
  - evidence:runner scripts/invoke-consolidated-verification.ps1#whole
  - evidence:validator scripts/verify-ci-membership.py#whole
  - evidence:empirical_runner verification/ci-verify-r1/invoke-measurements.ps1#whole
  - evidence:enforcement src/tests/architecture/ci_verify_r1.rs#whole
  - evidence:parser_red verification/ci-verify-r1/consolidated-parser-red-index.json#whole
  - evidence:parser_green verification/ci-verify-r1/consolidated-parser-green-index.json#whole
  - evidence:runtime_root_trial verification/ci-verify-r1/runtime-root-trial-index.json#whole
  - evidence:runtime_dependency_trial verification/ci-verify-r1/runtime-dependency-trial-index.json#whole
  - evidence:runtime_baseline verification/ci-verify-r1/runtime-baseline-index.json#whole
  - evidence:runtime_deduplication verification/ci-verify-r1/runtime-deduplication.json#whole
  - evidence:runtime_triage verification/ci-verify-r1/invoke-runtime-triage.ps1#whole
  - evidence:a4_fixture src/tests/mcp/dependency_footprint_af_perf_002.rs#whole

```

```architecture-graph
node: VERIFY-002
kind: EXPERIMENT
status: FROZEN
title: Baseline CI and authority acquisition evidence before verification redesign
track: REPOSITORY
artifact: verification/ci-verify-r1/baseline-evidence.zip
note: Frozen original failure and scoped operator successes, not candidate CI success or performance certification.
knowledge_scope: evidence
knowledge_content:
  - statement self#CI-VERIFY-R1-baseline
  - qualification self#CI-VERIFY-R1-limits
  - evidence verification/ci-verify-r1/evidence-index.json#whole
```
