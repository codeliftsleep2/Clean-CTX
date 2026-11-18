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
